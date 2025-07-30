use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Context;
use anyhow::Result;
use tempfile::TempDir;
use uuid::Uuid;

use crate::protocol::FileChange;

/// Accumulates multiple change sets and exposes the overall unified diff.
#[derive(Default)]
pub struct PatchAccumulator {
    /// Temporary git repository for building accumulated diffs.
    temp_git_repo: Option<TempDir>,
    /// Baseline commit that includes snapshots of all files seen so far.
    baseline_commit: Option<String>,
    /// Map external path -> internal filename (uuid + same extension).
    file_mapping: HashMap<PathBuf, String>,
    /// Internal filename -> external path as of baseline commit.
    internal_to_baseline_external: HashMap<String, PathBuf>,
    /// Internal filename -> external path as of current accumulated state (after applying all changes).
    internal_to_current_external: HashMap<String, PathBuf>,
    /// All change sets in the order they occurred.
    changes: Vec<HashMap<PathBuf, FileChange>>,
    /// Aggregated unified diff for all accumulated changes across files.
    pub unified_diff: Option<String>,
}

impl PatchAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ensure we have an initialized repository and a baseline snapshot of any new files.
    pub fn on_patch_begin(&mut self, changes: &HashMap<PathBuf, FileChange>) -> Result<()> {
        self.ensure_repo_init()?;
        let repo_dir = self.repo_dir()?.to_path_buf();

        let mut added_any = false;
        for path in changes.keys() {
            if !self.file_mapping.contains_key(path) {
                // Assign a stable internal filename for this external path.
                let internal = uuid_filename_for(path);
                self.file_mapping.insert(path.clone(), internal.clone());
                self.internal_to_baseline_external
                    .insert(internal.clone(), path.clone());
                self.internal_to_current_external
                    .insert(internal.clone(), path.clone());

                // If the file exists on disk, copy its contents into the repo and stage it.
                if path.exists() {
                    let contents = fs::read(path)
                        .with_context(|| format!("failed to read original {}", path.display()))?;
                    let internal_path = repo_dir.join(&internal);
                    if let Some(parent) = internal_path.parent() {
                        if !parent.as_os_str().is_empty() {
                            fs::create_dir_all(parent).ok();
                        }
                    }
                    fs::write(&internal_path, contents).with_context(|| {
                        format!("failed to write baseline file {}", internal_path.display())
                    })?;
                    run_git(&repo_dir, &["add", &internal])?;
                    added_any = true;
                }
            }
        }

        // Commit baseline additions if any.
        if added_any {
            run_git(&repo_dir, &["commit", "-m", "baseline"])?;
            // Update baseline commit id to HEAD.
            let id = run_git_capture(&repo_dir, &["rev-parse", "HEAD"])?;
            self.baseline_commit = Some(id.trim().to_string());
        }

        Ok(())
    }

    /// Record this change set and recompute the aggregated unified diff by
    /// applying all change sets to the repo working tree and diffing against the baseline commit.
    pub fn on_patch_end(&mut self, changes: HashMap<PathBuf, FileChange>) -> Result<()> {
        self.changes.push(changes);

        self.ensure_repo_init()?;
        let repo_dir = self.repo_dir()?.to_path_buf();

        // Reset working tree to baseline commit state.
        if let Some(commit) = &self.baseline_commit {
            run_git(&repo_dir, &["reset", "--hard", commit])?;
        }

        // Start with current external mapping reflective of baseline.
        // Rebuild current mapping from baseline mapping first.
        self.internal_to_current_external = self.internal_to_baseline_external.clone();

        // Apply all change sets in order to working tree.
        let mut current_contents: HashMap<String, String> = HashMap::new();
        // Preload current content for any baseline-tracked files.
        for (internal, _ext_path) in self.internal_to_current_external.clone() {
            let file_path = repo_dir.join(&internal);
            if file_path.exists() {
                let mut s = String::new();
                fs::File::open(&file_path)
                    .and_then(|mut f| f.read_to_string(&mut s))
                    .ok();
                current_contents.insert(internal.clone(), s);
            }
        }

        for change_set in &self.changes {
            for (ext_path, change) in change_set {
                let internal = match self.file_mapping.get(ext_path) {
                    Some(i) => i.clone(),
                    None => {
                        // Newly referenced path; create mapping (no baseline tracked -> add shows up as new file).
                        let i = uuid_filename_for(ext_path);
                        self.file_mapping.insert(ext_path.clone(), i.clone());
                        self.internal_to_baseline_external
                            .insert(i.clone(), ext_path.clone());
                        self.internal_to_current_external
                            .insert(i.clone(), ext_path.clone());
                        i
                    }
                };
                match change {
                    FileChange::Add { content } => {
                        // Create/overwrite internal file with provided content.
                        let file_path = repo_dir.join(&internal);
                        if let Some(parent) = file_path.parent() {
                            if !parent.as_os_str().is_empty() {
                                fs::create_dir_all(parent).ok();
                            }
                        }
                        fs::write(&file_path, content)
                            .with_context(|| format!("failed to write {}", file_path.display()))?;
                        current_contents.insert(internal.clone(), content.clone());
                        // Update current external path mapping (no move, but ensure it's present)
                        self.internal_to_current_external
                            .insert(internal.clone(), ext_path.clone());
                    }
                    FileChange::Delete => {
                        let file_path = repo_dir.join(&internal);
                        if file_path.exists() {
                            let _ = fs::remove_file(&file_path);
                        }
                        current_contents.remove(&internal);
                        // Keep current mapping entry as-is; diff will show deletion.
                    }
                    FileChange::Update {
                        unified_diff,
                        move_path,
                    } => {
                        // Apply unified diff to the current contents of internal file.
                        let base = current_contents.get(&internal).cloned().unwrap_or_default();
                        let new_content =
                            apply_unified_diff(&base, unified_diff).with_context(|| {
                                format!("apply unified diff for {}", ext_path.display())
                            })?;
                        let file_path = repo_dir.join(&internal);
                        if let Some(parent) = file_path.parent() {
                            if !parent.as_os_str().is_empty() {
                                fs::create_dir_all(parent).ok();
                            }
                        }
                        fs::write(&file_path, &new_content)
                            .with_context(|| format!("failed to write {}", file_path.display()))?;
                        current_contents.insert(internal.clone(), new_content);

                        if let Some(dest_path) = move_path {
                            // Update current external mapping for this internal id to the new external path.
                            self.internal_to_current_external
                                .insert(internal.clone(), dest_path.clone());
                            // Also update forward file_mapping: external current -> internal name.
                            self.file_mapping.remove(ext_path);
                            self.file_mapping
                                .insert(dest_path.clone(), internal.clone());
                        }
                    }
                }
            }
        }

        // Generate unified diff with git and rewrite internal paths to external paths.
        let raw = run_git_capture(&repo_dir, &["diff", "--no-color", "HEAD"])?;
        let rewritten = self.rewrite_diff_paths(&raw);
        self.unified_diff = if rewritten.trim().is_empty() {
            None
        } else {
            Some(rewritten)
        };
        Ok(())
    }

    fn repo_dir(&self) -> Result<&Path> {
        self.temp_git_repo
            .as_ref()
            .map(|d| d.path())
            .ok_or_else(|| anyhow::anyhow!("temp git repo not initialized"))
    }

    fn ensure_repo_init(&mut self) -> Result<()> {
        if self.temp_git_repo.is_some() {
            return Ok(());
        }
        let tmp = TempDir::new().context("create temp git dir")?;
        // Initialize git repo.
        run_git(tmp.path(), &["init"])?;
        // Configure identity to allow commits.
        let _ = run_git(tmp.path(), &["config", "user.email", "codex@example.com"]);
        let _ = run_git(tmp.path(), &["config", "user.name", "Codex"]);
        // Create an initial empty commit.
        run_git(tmp.path(), &["commit", "--allow-empty", "-m", "init"])?;
        let id = run_git_capture(tmp.path(), &["rev-parse", "HEAD"])?;
        self.baseline_commit = Some(id.trim().to_string());
        self.temp_git_repo = Some(tmp);
        Ok(())
    }

    /// Rewrites the internal repo filenames to external paths in diff headers.
    fn rewrite_diff_paths(&self, diff: &str) -> String {
        let mut out = String::new();
        for line in diff.lines() {
            if let Some(rest) = line.strip_prefix("diff --git ") {
                // Format: diff --git a/<f> b/<f>
                let parts: Vec<&str> = rest.split_whitespace().collect();
                if parts.len() == 2 {
                    let a = parts[0].strip_prefix("a/").unwrap_or(parts[0]);
                    let b = parts[1].strip_prefix("b/").unwrap_or(parts[1]);
                    let (a_ext, b_ext) = (
                        self.internal_to_baseline_external
                            .get(a)
                            .cloned()
                            .unwrap_or_else(|| PathBuf::from(a)),
                        self.internal_to_current_external
                            .get(b)
                            .cloned()
                            .unwrap_or_else(|| PathBuf::from(b)),
                    );
                    out.push_str(&format!(
                        "diff --git a/{} b/{}\n",
                        a_ext.display(),
                        b_ext.display()
                    ));
                    continue;
                }
            }
            if let Some(rest) = line.strip_prefix("--- ") {
                if let Some(path) = rest.strip_prefix("a/") {
                    let external = self
                        .internal_to_baseline_external
                        .get(path)
                        .cloned()
                        .unwrap_or_else(|| PathBuf::from(path));
                    out.push_str(&format!("--- {}\n", external.display()));
                    continue;
                }
            }
            if let Some(rest) = line.strip_prefix("+++ ") {
                if let Some(path) = rest.strip_prefix("b/") {
                    let external = self
                        .internal_to_current_external
                        .get(path)
                        .cloned()
                        .unwrap_or_else(|| PathBuf::from(path));
                    out.push_str(&format!("+++ {}\n", external.display()));
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
        Some(ext) if !ext.is_empty() => format!("{}.{}", id, ext),
        _ => id,
    }
}

fn run_git(repo: &Path, args: &[&str]) -> Result<()> {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .status()
        .with_context(|| format!("failed to run git {args:?} in {}", repo.display()))?;
    if !status.success() {
        anyhow::bail!("git {args:?} failed with status {:?}", status);
    }
    Ok(())
}

fn run_git_capture(repo: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git {args:?} in {}", repo.display()))?;
    if !output.status.success() {
        anyhow::bail!(
            "git {args:?} failed with status {:?}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn apply_unified_diff(base: &str, unified_diff: &str) -> Result<String> {
    let base_lines: Vec<&str> = if base.is_empty() {
        Vec::new()
    } else {
        base.split_inclusive('\n').collect()
    };

    let mut result: Vec<String> = Vec::new();
    let mut pos: usize = 0; // index in base_lines

    let mut it = unified_diff.lines().peekable();
    while let Some(line) = it.next() {
        if line.starts_with("---") || line.starts_with("+++") {
            continue;
        }
        if line.starts_with("@@") {
            // Parse old start index from header: "@@ -a,b +c,d @@"
            let middle = if let (Some(s), Some(e)) = (line.find("@@ "), line.rfind(" @@")) {
                &line[s + 3..e]
            } else {
                ""
            };
            let old_range = middle.split_whitespace().next().unwrap_or(""); // "-a,b"
            let old_start_str = old_range
                .strip_prefix('-')
                .unwrap_or(old_range)
                .split(',')
                .next()
                .unwrap_or("1");
            let old_start: usize = old_start_str.parse().unwrap_or(1);

            // Append unchanged lines up to this hunk
            let target = old_start.saturating_sub(1);
            while pos < target && pos < base_lines.len() {
                result.push(base_lines[pos].to_string());
                pos += 1;
            }

            // Apply hunk body until next header or EOF
            while let Some(peek) = it.peek() {
                let body_line = *peek;
                if body_line.starts_with("@@")
                    || body_line.starts_with("---")
                    || body_line.starts_with("+++")
                {
                    break;
                }
                let _ = it.next();
                if body_line.starts_with(' ') {
                    if let Some(src) = base_lines.get(pos) {
                        result.push((*src).to_string());
                    }
                    pos += 1;
                } else if body_line.starts_with('-') {
                    pos += 1;
                } else if body_line.starts_with('+') {
                    result.push(format!(
                        "{}\n",
                        body_line.strip_prefix('+').unwrap_or(body_line)
                    ));
                } else if body_line.is_empty() {
                    result.push("\n".to_string());
                } else {
                    if let Some(src) = base_lines.get(pos) {
                        result.push((*src).to_string());
                    }
                    pos += 1;
                }
            }
        }
    }

    // Append remaining
    while pos < base_lines.len() {
        result.push(base_lines[pos].to_string());
        pos += 1;
    }

    Ok(result.concat())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use similar::TextDiff;
    use tempfile::tempdir;

    #[test]
    fn accumulates_add_and_update() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");

        let mut acc = PatchAccumulator::new();

        // First patch: add file
        let add_changes = HashMap::from([(
            file.clone(),
            FileChange::Add {
                content: "foo\n".to_string(),
            },
        )]);
        acc.on_patch_begin(&add_changes).unwrap();
        acc.on_patch_end(add_changes).unwrap();
        let first = acc.unified_diff.clone().unwrap();
        assert!(first.contains("+foo"));

        // Second patch: update
        let old = "foo\n";
        let new = "foo\nbar\n";
        let diff = TextDiff::from_lines(old, new).unified_diff().to_string();
        let update_changes = HashMap::from([(
            file.clone(),
            FileChange::Update {
                unified_diff: diff,
                move_path: None,
            },
        )]);
        acc.on_patch_begin(&update_changes).unwrap();
        acc.on_patch_end(update_changes).unwrap();
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
        acc.on_patch_end(del_changes).unwrap();
        let diff = acc.unified_diff.clone().unwrap();
        assert!(diff.contains("-x"));
    }

    #[test]
    fn accumulates_move_and_update() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src.txt");
        let dest = dir.path().join("dst.txt");
        fs::write(&src, "line\n").unwrap();

        let old = "line\n";
        let new = "line2\n";
        let diff = TextDiff::from_lines(old, new).unified_diff().to_string();

        let mut acc = PatchAccumulator::new();
        let mv_changes = HashMap::from([(
            src.clone(),
            FileChange::Update {
                unified_diff: diff,
                move_path: Some(dest.clone()),
            },
        )]);
        acc.on_patch_begin(&mv_changes).unwrap();
        acc.on_patch_end(mv_changes).unwrap();
        let out = acc.unified_diff.clone().unwrap();
        assert!(out.contains("-line"));
        assert!(out.contains("+line2"));
    }
}
