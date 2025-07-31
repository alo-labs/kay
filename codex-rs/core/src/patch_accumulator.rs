use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use tempfile::TempDir;
use uuid::Uuid;

use crate::protocol::FileChange;

/// Tracks sets of changes to files and exposes the overall unified diff.
/// Internally, the way this works is now:
/// 1. Create a temp directory to store baseline snapshots of files when they are first seen.
/// 2. When a path is first observed, copy its current contents into the baseline dir if it exists on disk.
///    For new additions, do not create a baseline file so that diffs are shown as proper additions (using /dev/null).
/// 3. Keep a stable internal filename (uuid + same extension) per external path for path rewrite in diffs.
/// 4. To compute the aggregated unified diff, compare each baseline snapshot to the current file on disk using
///    `git diff --no-index` and rewrite paths to external paths.
#[derive(Default)]
pub struct PatchAccumulator {
    /// Temp directory holding baseline snapshots of files as first seen.
    baseline_files_dir: Option<TempDir>,
    /// Map external path -> internal filename (uuid + same extension).
    external_to_temp_name: HashMap<PathBuf, String>,
    /// Internal filename -> external path as of baseline snapshot.
    temp_name_to_baseline_external: HashMap<String, PathBuf>,
    /// Internal filename -> external path as of current accumulated state (after applying all changes).
    /// This is where renames are tracked.
    temp_name_to_current_external: HashMap<String, PathBuf>,
    /// Aggregated unified diff for all accumulated changes across files.
    pub unified_diff: Option<String>,
}

impl PatchAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Front-run apply patch calls to track the starting contents of any modified files.
    /// - Creates a baseline snapshot for files that already exist on disk when first seen.
    /// - For additions, we intentionally do not create a baseline snapshot so that diffs are proper additions.
    /// - Also updates internal mappings for move/rename events.
    pub fn on_patch_begin(&mut self, changes: &HashMap<PathBuf, FileChange>) -> Result<()> {
        self.ensure_baseline_dir()?;
        let baseline_dir = self.baseline_dir()?.to_path_buf();

        for (path, change) in changes.iter() {
            // Ensure a stable internal filename exists for this external path.
            if !self.external_to_temp_name.contains_key(path) {
                let internal = uuid_filename_for(path);
                self.external_to_temp_name
                    .insert(path.clone(), internal.clone());
                self.temp_name_to_baseline_external
                    .insert(internal.clone(), path.clone());
                self.temp_name_to_current_external
                    .insert(internal.clone(), path.clone());

                // If the file exists on disk now, snapshot as baseline; else leave missing to represent /dev/null.
                if path.exists() {
                    let contents = fs::read(path)
                        .with_context(|| format!("failed to read original {}", path.display()))?;
                    let internal_path = baseline_dir.join(&internal);
                    fs::write(&internal_path, contents).with_context(|| {
                        format!("failed to write baseline file {}", internal_path.display())
                    })?;
                }
            }

            // Track rename/move in current mapping if provided in an Update.
            let move_path = match change {
                FileChange::Update {
                    move_path: Some(dest),
                    ..
                } => Some(dest),
                _ => None,
            };
            if let Some(dest) = move_path {
                let uuid_filename = match self.external_to_temp_name.get(path) {
                    Some(i) => i.clone(),
                    None => {
                        // This should be rare, but if we haven't mapped the source, create it with no baseline.
                        let i = uuid_filename_for(path);
                        self.external_to_temp_name.insert(path.clone(), i.clone());
                        self.temp_name_to_baseline_external
                            .insert(i.clone(), path.clone());
                        i
                    }
                };
                // Update current external mapping for temp file name.
                self.temp_name_to_current_external
                    .insert(uuid_filename.clone(), dest.clone());
                // Update forward file_mapping: external current -> internal name.
                self.external_to_temp_name.remove(path);
                self.external_to_temp_name
                    .insert(dest.clone(), uuid_filename);
            }
        }

        Ok(())
    }

    /// Recompute the aggregated unified diff by comparing all baseline snapshots against
    /// current files on disk using `git diff --no-index` and rewriting paths to external paths.
    pub fn update_unified_diff(&mut self) -> Result<()> {
        let baseline_dir = self.baseline_dir()?.to_path_buf();
        let current_dir = baseline_dir.join("current");
        if current_dir.exists() {
            // Best-effort cleanup of previous run's mirror.
            let _ = fs::remove_dir_all(&current_dir);
        }
        fs::create_dir_all(&current_dir).with_context(|| {
            format!(
                "failed to create current mirror dir {}",
                current_dir.display()
            )
        })?;

        let mut aggregated = String::new();

        // Compute diffs per tracked internal file.
        for (internal, baseline_external) in &self.temp_name_to_baseline_external {
            let baseline_path = baseline_dir.join(internal);
            let current_external = self
                .temp_name_to_current_external
                .get(internal)
                .cloned()
                .unwrap_or_else(|| baseline_external.clone());

            let left_is_dev_null = !baseline_path.exists();
            let right_exists = current_external.exists();

            // Prepare right side mirror file if exists; otherwise use /dev/null for deletions.
            let right_arg = if right_exists {
                let mirror_path = current_dir.join(internal);
                let contents = fs::read(&current_external).with_context(|| {
                    format!(
                        "failed to read current file for diff {}",
                        current_external.display()
                    )
                })?;
                fs::write(&mirror_path, contents).with_context(|| {
                    format!(
                        "failed to write current mirror file {}",
                        mirror_path.display()
                    )
                })?;
                // Use relative path from baseline_dir (so headers say a/<uuid> b/current/<uuid>).
                format!("current/{internal}")
            } else {
                // Deletion: right side is /dev/null to show proper deleted file diff.
                "/dev/null".to_string()
            };

            // Prepare left arg: baseline file path or /dev/null for additions.
            let left_arg = if left_is_dev_null {
                "/dev/null".to_string()
            } else {
                internal.clone()
            };

            // Run git diff --no-index from baseline_dir to keep paths predictable.
            let raw = run_git_allow_exit_codes(
                &baseline_dir,
                &[
                    "-c",
                    "color.ui=false",
                    "diff",
                    "--no-color",
                    "--no-index",
                    "--",
                    &left_arg,
                    &right_arg,
                ],
                &[0, 1], // 0: no changes, 1: differences
            )?;

            if raw.trim().is_empty() {
                continue;
            }
            let rewritten = self.rewrite_diff_paths(&raw);
            if !rewritten.trim().is_empty() {
                if !aggregated.is_empty() && !aggregated.ends_with('\n') {
                    aggregated.push('\n');
                }
                aggregated.push_str(&rewritten);
            }
        }

        self.unified_diff = if aggregated.trim().is_empty() {
            None
        } else {
            Some(aggregated)
        };

        // Clean up the curent dir.
        let _ = fs::remove_dir_all(&current_dir);

        Ok(())
    }

    fn baseline_dir(&self) -> Result<&Path> {
        self.baseline_files_dir
            .as_ref()
            .map(|d| d.path())
            .ok_or_else(|| anyhow::anyhow!("baseline temp dir not initialized"))
    }

    fn ensure_baseline_dir(&mut self) -> Result<()> {
        if self.baseline_files_dir.is_some() {
            return Ok(());
        }
        let tmp = TempDir::new().context("create baseline temp dir")?;
        self.baseline_files_dir = Some(tmp);
        Ok(())
    }

    /// Rewrites the internal filenames to external paths in diff headers.
    /// Handles inputs like:
    ///   diff --git a/<uuid> b/current/<uuid>
    ///   --- a/<uuid> | /dev/null
    ///   +++ b/current/<uuid> | /dev/null
    /// and replaces uuid with the external paths tracking baseline/current.
    fn rewrite_diff_paths(&self, diff: &str) -> String {
        let mut out = String::new();
        for line in diff.lines() {
            if let Some(rest) = line.strip_prefix("diff --git ") {
                // Format: diff --git a/<f> b/<f>
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if parts.len() == 2 {
                    let a = parts[0].strip_prefix("a/").unwrap_or(parts[0]);
                    let b = parts[1].strip_prefix("b/").unwrap_or(parts[1]);

                    let a_ext_display = if a == "/dev/null" {
                        "/dev/null".to_string()
                    } else {
                        let a_base = Path::new(a)
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or(a);
                        let mapped = self
                            .temp_name_to_baseline_external
                            .get(a_base)
                            .cloned()
                            .unwrap_or_else(|| PathBuf::from(a));
                        mapped.display().to_string()
                    };

                    let b_ext_display = if b == "/dev/null" {
                        "/dev/null".to_string()
                    } else {
                        let b_base = Path::new(b)
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or(b);
                        let mapped = self
                            .temp_name_to_current_external
                            .get(b_base)
                            .cloned()
                            .unwrap_or_else(|| PathBuf::from(b));
                        mapped.display().to_string()
                    };

                    out.push_str(&format!("diff --git a/{a_ext_display} b/{b_ext_display}\n"));
                    continue;
                }
            }
            if let Some(rest) = line.strip_prefix("--- ") {
                if let Some(path) = rest.strip_prefix("a/") {
                    let external_display = if path == "/dev/null" {
                        "/dev/null".to_string()
                    } else {
                        let p_base = Path::new(path)
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or(path);
                        self.temp_name_to_baseline_external
                            .get(p_base)
                            .cloned()
                            .unwrap_or_else(|| PathBuf::from(path))
                            .display()
                            .to_string()
                    };
                    out.push_str(&format!("--- {external_display}\n"));
                    continue;
                }
            }
            if let Some(rest) = line.strip_prefix("+++ ") {
                if let Some(path) = rest.strip_prefix("b/") {
                    let external_display = if path == "/dev/null" {
                        "/dev/null".to_string()
                    } else {
                        let p_base = Path::new(path)
                            .file_name()
                            .and_then(|s| s.to_str())
                            .unwrap_or(path);
                        self.temp_name_to_current_external
                            .get(p_base)
                            .cloned()
                            .unwrap_or_else(|| PathBuf::from(path))
                            .display()
                            .to_string()
                    };
                    out.push_str(&format!("+++ {external_display}\n"));
                    continue;
                }
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    }
}

fn uuid_filename_for(path: &Path) -> String {
    let id = Uuid::new_v4().to_string();
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) if !ext.is_empty() => format!("{id}.{ext}"),
        _ => id,
    }
}

fn run_git_allow_exit_codes(
    repo: &Path,
    args: &[&str],
    allowed_exit_codes: &[i32],
) -> Result<String> {
    let output = Command::new("git")
        .current_dir(repo)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git {:?} in {}", args, repo.display()))?;
    let code = output.status.code().unwrap_or(-1);
    if !allowed_exit_codes.contains(&code) {
        anyhow::bail!(
            "git {:?} failed with status {:?}: {}",
            args,
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn accumulates_add_and_update() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");

        let mut acc = PatchAccumulator::new();

        // First patch: add file (baseline should be /dev/null).
        let add_changes = HashMap::from([(
            file.clone(),
            FileChange::Add {
                content: "foo\n".to_string(),
            },
        )]);
        acc.on_patch_begin(&add_changes).unwrap();

        // Simulate apply: create the file on disk.
        fs::write(&file, "foo\n").unwrap();
        acc.update_unified_diff().unwrap();
        let first = acc.unified_diff.clone().unwrap();
        assert!(first.contains("+foo"));
        assert!(first.contains("/dev/null") || first.contains("new file"));

        // Second patch: update the file on disk.
        let update_changes = HashMap::from([(
            file.clone(),
            FileChange::Update {
                unified_diff: "".to_owned(),
                move_path: None,
            },
        )]);
        acc.on_patch_begin(&update_changes).unwrap();

        // Simulate apply: append a new line.
        fs::write(&file, "foo\nbar\n").unwrap();
        acc.update_unified_diff().unwrap();
        let combined = acc.unified_diff.clone().unwrap();
        assert!(combined.contains("+bar"));
    }

    #[test]
    fn accumulates_delete() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("b.txt");
        fs::write(&file, "x\n").unwrap();

        let mut acc = PatchAccumulator::new();
        let del_changes = HashMap::from([(file.clone(), FileChange::Delete)]);
        acc.on_patch_begin(&del_changes).unwrap();

        // Simulate apply: delete the file from disk.
        fs::remove_file(&file).unwrap();
        acc.update_unified_diff().unwrap();
        let diff = acc.unified_diff.clone().unwrap();
        assert!(diff.contains("-x"));
    }

    #[test]
    fn accumulates_move_and_update() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src.txt");
        let dest = dir.path().join("dst.txt");
        fs::write(&src, "line\n").unwrap();

        let mut acc = PatchAccumulator::new();
        let mv_changes = HashMap::from([(
            src.clone(),
            FileChange::Update {
                unified_diff: "".to_owned(),
                move_path: Some(dest.clone()),
            },
        )]);
        acc.on_patch_begin(&mv_changes).unwrap();

        // Simulate apply: move and update content.
        fs::rename(&src, &dest).unwrap();
        fs::write(&dest, "line2\n").unwrap();

        acc.update_unified_diff().unwrap();
        let out = acc.unified_diff.clone().unwrap();
        assert!(out.contains("-line"));
        assert!(out.contains("+line2"));
    }

    #[test]
    fn update_persists_across_new_baseline_for_new_file() {
        let dir = tempdir().unwrap();
        let a = dir.path().join("a.txt");
        let b = dir.path().join("b.txt");
        fs::write(&a, "foo\n").unwrap();
        fs::write(&b, "z\n").unwrap();

        let mut acc = PatchAccumulator::new();

        // First: update existing a.txt (baseline snapshot is created for a).
        let update_a = HashMap::from([(
            a.clone(),
            FileChange::Update {
                unified_diff: "".to_owned(),
                move_path: None,
            },
        )]);
        acc.on_patch_begin(&update_a).unwrap();
        // Simulate apply: modify a.txt on disk.
        fs::write(&a, "foo\nbar\n").unwrap();
        acc.update_unified_diff().unwrap();
        let first = acc.unified_diff.clone().unwrap();
        assert!(first.contains("+bar"));

        // Next: introduce a brand-new path b.txt into baseline snapshots via a delete change.
        let del_b = HashMap::from([(b.clone(), FileChange::Delete)]);
        acc.on_patch_begin(&del_b).unwrap();
        // Simulate apply: delete b.txt.
        fs::remove_file(&b).unwrap();
        acc.update_unified_diff().unwrap();

        let combined = acc.unified_diff.clone().unwrap();
        // The combined diff must still include the update to a.txt.
        assert!(combined.contains("+bar"));
        // And also reflect the deletion of b.txt.
        assert!(combined.contains("-z"));
    }
}
