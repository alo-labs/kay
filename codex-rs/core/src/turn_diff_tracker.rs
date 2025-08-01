use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use uuid::Uuid;

use crate::protocol::FileChange;

/// Tracks sets of changes to files and exposes the overall unified diff.
/// Internally, the way this works is now:
/// 1. Maintain an in-memory baseline snapshot of files when they are first seen.
///    For new additions, do not create a baseline so that diffs are shown as proper additions (using /dev/null).
/// 2. Keep a stable internal filename (uuid + same extension) per external path for rename tracking.
/// 3. To compute the aggregated unified diff, compare each baseline snapshot to the current file on disk entirely in-memory
///    using the `similar` crate and emit unified diffs with rewritten external paths.
#[derive(Default)]
pub struct TurnDiffTracker {
    /// Map external path -> internal filename (uuid + same extension).
    external_to_temp_name: HashMap<PathBuf, String>,
    /// Internal filename -> external path as of baseline snapshot.
    temp_name_to_baseline_external: HashMap<String, PathBuf>,
    /// Internal filename -> external path as of current accumulated state (after applying all changes).
    /// This is where renames are tracked.
    temp_name_to_current_external: HashMap<String, PathBuf>,
    /// Internal filename -> baseline file contents (None means the file did not exist, i.e. /dev/null).
    baseline_contents: HashMap<String, Option<String>>,
    /// Aggregated unified diff for all accumulated changes across files.
    pub unified_diff: Option<String>,
}

impl TurnDiffTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Front-run apply patch calls to track the starting contents of any modified files.
    /// - Creates an in-memory baseline snapshot for files that already exist on disk when first seen.
    /// - For additions, we intentionally do not create a baseline snapshot so that diffs are proper additions.
    /// - Also updates internal mappings for move/rename events.
    pub fn on_patch_begin(&mut self, changes: &HashMap<PathBuf, FileChange>) -> Result<()> {
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
                let baseline = if path.exists() {
                    let contents = fs::read(path)
                        .with_context(|| format!("failed to read original {}", path.display()))?;
                    Some(String::from_utf8_lossy(&contents).into_owned())
                } else {
                    None
                };
                self.baseline_contents.insert(internal.clone(), baseline);
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
                        // No on-disk file read here; treat as addition.
                        self.baseline_contents.insert(i.clone(), None);
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
    /// current files on disk using the `similar` crate and rewriting paths to external paths.
    pub fn update_and_get_unified_diff(&mut self) -> Result<Option<String>> {
        let mut aggregated = String::new();

        // Compute diffs per tracked internal file.
        for (internal, baseline_external) in &self.temp_name_to_baseline_external {
            let current_external = self
                .temp_name_to_current_external
                .get(internal)
                .cloned()
                .unwrap_or_else(|| baseline_external.clone());

            let left_content = self
                .baseline_contents
                .get(internal)
                .cloned()
                .unwrap_or(None);

            let right_content = if current_external.exists() {
                let contents = fs::read(&current_external).with_context(|| {
                    format!(
                        "failed to read current file for diff {}",
                        current_external.display()
                    )
                })?;
                Some(String::from_utf8_lossy(&contents).into_owned())
            } else {
                None
            };

            let left_text = left_content.as_deref().unwrap_or("");
            let right_text = right_content.as_deref().unwrap_or("");

            if left_text == right_text {
                continue;
            }

            let left_display = baseline_external.display().to_string();
            let right_display = current_external.display().to_string();

            // Diff the contents.
            let diff = similar::TextDiff::from_lines(left_text, right_text);

            // Emit a git-style header for better readability and parity with previous behavior.
            aggregated.push_str(&format!("diff --git a/{left_display} b/{right_display}\n"));

            let old_header = if left_content.is_some() {
                format!("a/{left_display}")
            } else {
                "/dev/null".to_string()
            };
            let new_header = if right_content.is_some() {
                format!("b/{right_display}")
            } else {
                "/dev/null".to_string()
            };

            let unified = diff
                .unified_diff()
                .context_radius(3)
                .header(&old_header, &new_header)
                .to_string();

            aggregated.push_str(&unified);
            if !aggregated.ends_with('\n') {
                aggregated.push('\n');
            }
        }

        self.unified_diff = if aggregated.trim().is_empty() {
            None
        } else {
            Some(aggregated)
        };

        Ok(self.unified_diff.clone())
    }
}

fn uuid_filename_for(path: &Path) -> String {
    let id = Uuid::new_v4().to_string();
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) if !ext.is_empty() => format!("{id}.{ext}"),
        _ => id,
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn accumulates_add_and_update() {
        let mut acc = TurnDiffTracker::new();

        let dir = tempdir().unwrap();
        let file = dir.path().join("a.txt");

        // First patch: add file (baseline should be /dev/null).
        let add_changes = HashMap::from([(
            file.clone(),
            FileChange::Add {
                content: "foo\n".to_string(),
            },
        )]);
        acc.on_patch_begin(&add_changes).unwrap();

        // Simulate apply: create the file on disk.
        // This must happen after on_patch_begin.
        fs::write(&file, "foo\n").unwrap();
        acc.update_and_get_unified_diff().unwrap();
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
        acc.update_and_get_unified_diff().unwrap();
        let combined = acc.unified_diff.clone().unwrap();
        assert!(combined.contains("+bar"));
    }

    #[test]
    fn accumulates_delete() {
        let dir = tempdir().unwrap();
        let file = dir.path().join("b.txt");
        fs::write(&file, "x\n").unwrap();

        let mut acc = TurnDiffTracker::new();
        let del_changes = HashMap::from([(file.clone(), FileChange::Delete)]);
        acc.on_patch_begin(&del_changes).unwrap();

        // Simulate apply: delete the file from disk.
        fs::remove_file(&file).unwrap();
        acc.update_and_get_unified_diff().unwrap();
        let diff = acc.unified_diff.clone().unwrap();
        assert!(diff.contains("-x"));
    }

    #[test]
    fn accumulates_move_and_update() {
        let dir = tempdir().unwrap();
        let src = dir.path().join("src.txt");
        let dest = dir.path().join("dst.txt");
        fs::write(&src, "line\n").unwrap();

        let mut acc = TurnDiffTracker::new();
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

        acc.update_and_get_unified_diff().unwrap();
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

        let mut acc = TurnDiffTracker::new();

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
        acc.update_and_get_unified_diff().unwrap();
        let first = acc.unified_diff.clone().unwrap();
        assert!(first.contains("+bar"));

        // Next: introduce a brand-new path b.txt into baseline snapshots via a delete change.
        let del_b = HashMap::from([(b.clone(), FileChange::Delete)]);
        acc.on_patch_begin(&del_b).unwrap();
        // Simulate apply: delete b.txt.
        fs::remove_file(&b).unwrap();
        acc.update_and_get_unified_diff().unwrap();

        let combined = acc.unified_diff.clone().unwrap();
        // The combined diff must still include the update to a.txt.
        assert!(combined.contains("+bar"));
        // And also reflect the deletion of b.txt.
        assert!(combined.contains("-z"));
    }
}
