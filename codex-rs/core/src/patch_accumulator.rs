use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io::Read;
use std::io::Seek;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use similar::TextDiff;
use tempfile::NamedTempFile;

use crate::protocol::FileChange;

/// Accumulates multiple change sets and exposes the overall unified diff.
#[derive(Default)]
pub struct PatchAccumulator {
    /// Snapshot of original file contents at first sighting.
    original_file_copies: HashMap<PathBuf, NamedTempFile>,
    /// All change sets in the order they occurred.
    changes: Vec<HashMap<PathBuf, FileChange>>,
    /// Aggregated unified diff for all accumulated changes across files.
    pub unified_diff: Option<String>,
}

impl PatchAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    /// Ensure we have an original snapshot for each file in `changes`.
    /// For files that don't exist yet (e.g., Add), we snapshot as empty.
    pub fn on_patch_begin(&mut self, changes: &HashMap<PathBuf, FileChange>) -> Result<()> {
        for path in changes.keys() {
            if !self.original_file_copies.contains_key(path) {
                let mut tmp = NamedTempFile::new().context("create temp file for snapshot")?;
                if path.exists() {
                    let content = fs::read(path)
                        .with_context(|| format!("failed to read original {}", path.display()))?;
                    tmp.write_all(&content).with_context(|| {
                        format!("failed to write snapshot for {}", path.display())
                    })?;
                } else {
                    // Represent missing file as empty baseline.
                    // Leaving the temp file empty is sufficient.
                }
                // Ensure file cursor at start for future reads.
                tmp.as_file_mut()
                    .rewind()
                    .context("rewind snapshot temp file")?;
                self.original_file_copies.insert(path.clone(), tmp);
            }
        }
        Ok(())
    }

    /// Record this change set and recompute the aggregated unified diff by
    /// applying all change sets to the snapshots in memory.
    pub fn on_patch_end(&mut self, changes: HashMap<PathBuf, FileChange>) -> Result<()> {
        self.changes.push(changes);

        // Build initial working set from original snapshots.
        let mut current: HashMap<PathBuf, String> = HashMap::new();
        for (origin, tmp) in &mut self.original_file_copies {
            let mut buf = String::new();
            tmp.as_file_mut()
                .rewind()
                .with_context(|| format!("rewind snapshot {}", origin.display()))?;
            tmp.as_file_mut()
                .read_to_string(&mut buf)
                .with_context(|| format!("read snapshot {}", origin.display()))?;
            current.insert(origin.clone(), buf);
        }

        // Track current path per origin to support moves.
        let mut current_path_by_origin: HashMap<PathBuf, PathBuf> = self
            .original_file_copies
            .keys()
            .map(|p| (p.clone(), p.clone()))
            .collect();
        // Reverse mapping for efficient lookup of origin by current path.
        let mut origin_by_current_path: HashMap<PathBuf, PathBuf> = current_path_by_origin
            .iter()
            .map(|(o, p)| (p.clone(), o.clone()))
            .collect();

        // Apply each change set in order.
        for change_set in &self.changes {
            for (path, change) in change_set {
                match change {
                    FileChange::Add { content } => {
                        // Ensure snapshot exists for added files as empty baseline.
                        if !self.original_file_copies.contains_key(path) {
                            let mut tmp = NamedTempFile::new()
                                .context("create temp file for added file baseline")?;
                            tmp.as_file_mut().rewind().ok();
                            self.original_file_copies.insert(path.clone(), tmp);
                            current_path_by_origin.insert(path.clone(), path.clone());
                            origin_by_current_path.insert(path.clone(), path.clone());
                        }
                        current.insert(path.clone(), content.clone());
                    }
                    FileChange::Delete => {
                        current.remove(path);
                        // mapping remains so we can diff baseline -> empty.
                    }
                    FileChange::Update {
                        unified_diff,
                        move_path,
                    } => {
                        // Determine source content.
                        let src_path = path;
                        let src_content = current
                            .get(src_path)
                            .cloned()
                            .or_else(|| self.read_snapshot_str(src_path).ok())
                            .unwrap_or_default();

                        let new_content = apply_unified_diff(&src_content, unified_diff)
                            .with_context(|| {
                                format!("apply unified diff for {}", src_path.display())
                            })?;

                        if let Some(dest) = move_path {
                            current.remove(src_path);
                            current.insert(dest.clone(), new_content);

                            // Update origin mapping for the move.
                            if let Some(origin) = origin_by_current_path.remove(src_path) {
                                current_path_by_origin.insert(origin.clone(), dest.clone());
                                origin_by_current_path.insert(dest.clone(), origin);
                            } else {
                                // If we did not know this path yet, seed origin as src.
                                current_path_by_origin.insert(src_path.clone(), dest.clone());
                                origin_by_current_path.insert(dest.clone(), src_path.clone());
                            }
                        } else {
                            current.insert(src_path.clone(), new_content);
                        }
                    }
                }
            }
        }

        // Compute aggregated unified diff across all origins we've seen.
        let mut all_origins: HashSet<PathBuf> = self.original_file_copies.keys().cloned().collect();
        // Include any paths that were added but had no original snapshot (should already be present).
        for p in current.keys() {
            all_origins.insert(p.clone());
        }

        let mut combined = String::new();
        for origin in all_origins {
            let old = self.read_snapshot_str(&origin).unwrap_or_default();
            let new_path = current_path_by_origin
                .get(&origin)
                .cloned()
                .unwrap_or(origin.clone());
            let new = current.get(&new_path).cloned().unwrap_or_default();

            if old == new {
                continue;
            }

            let diff = TextDiff::from_lines(&old, &new).unified_diff().to_string();
            let diff = with_paths_in_headers(&diff, &origin, &new_path);
            if !combined.is_empty() {
                combined.push('\n');
            }
            combined.push_str(&diff);
        }

        self.unified_diff = if combined.is_empty() {
            None
        } else {
            Some(combined)
        };
        Ok(())
    }

    fn read_snapshot_str(&self, path: &Path) -> Result<String> {
        if let Some(tmp) = self.original_file_copies.get(path).map(|t| t.reopen()) {
            let mut file = tmp.context("reopen temp file")?;
            file.rewind().ok();
            let mut s = String::new();
            file.read_to_string(&mut s).context("read temp file")?;
            Ok(s)
        } else {
            Ok(String::new())
        }
    }
}

fn with_paths_in_headers(diff: &str, old_path: &Path, new_path: &Path) -> String {
    let mut out = String::new();
    let mut replaced = 0usize;
    for line in diff.lines() {
        if replaced < 1 && line.starts_with("---") {
            out.push_str(&format!("--- {}\n", old_path.display()));
            replaced += 1;
            continue;
        }
        if replaced < 2 && line.starts_with("+++") {
            out.push_str(&format!("+++ {}\n", new_path.display()));
            replaced += 1;
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
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
