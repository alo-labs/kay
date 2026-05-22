use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

static RUN_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct SessionPreserver {
    kay_home: PathBuf,
    test_id: String,
}

impl SessionPreserver {
    pub fn new(kay_home: &Path, test_id: impl AsRef<str>) -> Self {
        Self {
            kay_home: kay_home.to_path_buf(),
            test_id: sanitize_component(test_id.as_ref()),
        }
    }
}

impl Drop for SessionPreserver {
    fn drop(&mut self) {
        if let Err(err) = preserve_test_sessions(&self.kay_home, &self.test_id) {
            eprintln!(
                "warning: failed to preserve Kay test sessions for {}: {err}",
                self.test_id
            );
        }
    }
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("cli crate should live under kay-rs/cli")
        .to_path_buf()
}

fn test_sessions_root() -> PathBuf {
    repo_root().join("tests").join("sessions")
}

fn sanitize_component(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>();
    let trimmed = sanitized.trim_matches('_');
    if trimmed.is_empty() {
        "unknown-test".to_string()
    } else {
        trimmed.to_string()
    }
}

fn unique_run_id() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let count = RUN_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!(
        "{}.{:09}-{}-{count}",
        now.as_secs(),
        now.subsec_nanos(),
        std::process::id()
    )
}

fn copy_dir_recursive(src: &Path, dest: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dest)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dest_path = dest.join(entry.file_name());
        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dest_path)?;
        } else if src_path.is_file() {
            if let Some(parent) = dest_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(&src_path, &dest_path)?;
        }
    }
    Ok(())
}

fn copy_root_jsonl_logs(kay_home: &Path, dest: &Path) -> std::io::Result<usize> {
    let mut copied = 0;
    let Ok(entries) = fs::read_dir(kay_home) else {
        return Ok(0);
    };
    let logs_dest = dest.join("root-jsonl");
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !name.ends_with(".jsonl") {
            continue;
        }
        fs::create_dir_all(&logs_dest)?;
        fs::copy(&path, logs_dest.join(name))?;
        copied += 1;
    }
    Ok(copied)
}

fn prune_old_runs(test_dir: &Path) -> std::io::Result<()> {
    let mut runs = fs::read_dir(test_dir)?
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_dir() {
                return None;
            }
            let modified = entry
                .metadata()
                .and_then(|metadata| metadata.modified())
                .unwrap_or(UNIX_EPOCH);
            Some((modified, path))
        })
        .collect::<Vec<_>>();
    runs.sort_by(|(left_time, left_path), (right_time, right_path)| {
        right_time
            .cmp(left_time)
            .then_with(|| right_path.cmp(left_path))
    });
    for (_, path) in runs.into_iter().skip(3) {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

pub fn preserve_test_sessions(kay_home: &Path, test_id: &str) -> std::io::Result<Option<PathBuf>> {
    let sessions_src = kay_home.join("sessions");
    let has_sessions = sessions_src.is_dir();
    let has_root_jsonl = fs::read_dir(kay_home)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .any(|entry| {
            entry.path().is_file()
                && entry
                    .path()
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.ends_with(".jsonl"))
        });
    if !has_sessions && !has_root_jsonl {
        return Ok(None);
    }

    let test_dir = test_sessions_root().join(sanitize_component(test_id));
    let run_dir = test_dir.join(unique_run_id());
    fs::create_dir_all(&run_dir)?;

    if has_sessions {
        copy_dir_recursive(&sessions_src, &run_dir.join("sessions"))?;
    }
    copy_root_jsonl_logs(kay_home, &run_dir)?;
    prune_old_runs(&test_dir)?;

    Ok(Some(run_dir))
}
