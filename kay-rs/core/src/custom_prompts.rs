use code_protocol::custom_prompts::CustomPrompt;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;

/// Return the default prompts directories, ordered from highest to lowest
/// precedence. Kay's local prompts are the only implicit source so the
/// end-user install stays isolated from any local Codex environment.
pub fn default_prompts_dirs() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    if let Ok(kay_home) = crate::config::find_kay_home() {
        let kay_prompts = kay_home.join("prompts");
        if kay_prompts.is_dir() {
            roots.push(kay_prompts);
        }
    }

    let mut seen: HashSet<PathBuf> = HashSet::new();
    roots.retain(|root| seen.insert(root.clone()));
    roots
}

/// Return the first available default prompts directory, if any.
pub fn default_prompts_dir() -> Option<PathBuf> {
    default_prompts_dirs().into_iter().next()
}

/// Discover prompt files in the given directory, returning entries sorted by name.
/// Non-files are ignored. If the directory does not exist or cannot be read, returns empty.
pub async fn discover_prompts_in(dir: &Path) -> Vec<CustomPrompt> {
    discover_prompts_in_excluding(dir, &HashSet::new()).await
}

/// Discover prompt files across multiple directories, preserving the first
/// occurrence of each prompt name.
pub async fn discover_prompts_in_roots(dirs: &[PathBuf]) -> Vec<CustomPrompt> {
    let mut out: Vec<CustomPrompt> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    for dir in dirs {
        for prompt in discover_prompts_in(dir).await {
            if seen.insert(prompt.name.clone()) {
                out.push(prompt);
            }
        }
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

/// Discover prompt files in the given directory, excluding any with names in `exclude`.
/// Returns entries sorted by name. Non-files are ignored. Missing/unreadable dir yields empty.
pub async fn discover_prompts_in_excluding(
    dir: &Path,
    exclude: &HashSet<String>,
) -> Vec<CustomPrompt> {
    let mut out: Vec<CustomPrompt> = Vec::new();
    let mut entries = match fs::read_dir(dir).await {
        Ok(entries) => entries,
        Err(_) => return out,
    };

    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        let is_file = entry
            .file_type()
            .await
            .map(|ft| ft.is_file())
            .unwrap_or(false);
        if !is_file {
            continue;
        }
        // Only include Markdown files with a .md extension.
        let is_md = path
            .extension()
            .and_then(|s| s.to_str())
            .map(|ext| ext.eq_ignore_ascii_case("md"))
            .unwrap_or(false);
        if !is_md {
            continue;
        }
        let Some(name) = path
            .file_stem()
            .and_then(|s| s.to_str())
            .map(str::to_string)
        else {
            continue;
        };
        if exclude.contains(&name) {
            continue;
        }
        let content = match fs::read_to_string(&path).await {
            Ok(s) => s,
            Err(_) => continue,
        };
        out.push(CustomPrompt {
            name,
            path,
            content,
            description: None,
            argument_hint: None,
        });
    }
    out.sort_by(|a, b| a.name.cmp(&b.name));
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    struct EnvVarGuard(&'static str);

    impl EnvVarGuard {
        fn new(name: &'static str) -> Self {
            Self(name)
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            unsafe {
                std::env::remove_var(self.0);
            }
        }
    }

    #[tokio::test]
    async fn empty_when_dir_missing() {
        let tmp = tempdir().expect("create TempDir");
        let missing = tmp.path().join("nope");
        let found = discover_prompts_in(&missing).await;
        assert!(found.is_empty());
    }

    #[tokio::test]
    async fn discovers_and_sorts_files() {
        let tmp = tempdir().expect("create TempDir");
        let dir = tmp.path();
        fs::write(dir.join("b.md"), b"b").unwrap();
        fs::write(dir.join("a.md"), b"a").unwrap();
        fs::create_dir(dir.join("subdir")).unwrap();
        let found = discover_prompts_in(dir).await;
        let names: Vec<String> = found.into_iter().map(|e| e.name).collect();
        assert_eq!(names, vec!["a", "b"]);
    }

    #[tokio::test]
    async fn excludes_builtins() {
        let tmp = tempdir().expect("create TempDir");
        let dir = tmp.path();
        fs::write(dir.join("init.md"), b"ignored").unwrap();
        fs::write(dir.join("foo.md"), b"ok").unwrap();
        let mut exclude = HashSet::new();
        exclude.insert("init".to_string());
        let found = discover_prompts_in_excluding(dir, &exclude).await;
        let names: Vec<String> = found.into_iter().map(|e| e.name).collect();
        assert_eq!(names, vec!["foo"]);
    }

    #[tokio::test]
    async fn skips_non_utf8_files() {
        let tmp = tempdir().expect("create TempDir");
        let dir = tmp.path();
        // Valid UTF-8 file
        fs::write(dir.join("good.md"), b"hello").unwrap();
        // Invalid UTF-8 content in .md file (e.g., lone 0xFF byte)
        fs::write(dir.join("bad.md"), vec![0xFF, 0xFE, b'\n']).unwrap();
        let found = discover_prompts_in(dir).await;
        let names: Vec<String> = found.into_iter().map(|e| e.name).collect();
        assert_eq!(names, vec!["good"]);
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn ignores_host_prompts_when_kay_home_missing() {
        let host_home = tempdir().expect("create host tempdir");
        let kay_home = tempdir().expect("create kay tempdir");
        let host_prompts = host_home.path().join(".codex/prompts");
        fs::create_dir_all(&host_prompts).unwrap();
        fs::write(host_prompts.join("foo.md"), "host").unwrap();

        let _host_guard = EnvVarGuard::new("CODEX_HOST_HOME");
        let _kay_guard = EnvVarGuard::new("KAY_HOME");
        unsafe {
            std::env::set_var("CODEX_HOST_HOME", host_home.path().join(".codex"));
            std::env::set_var("KAY_HOME", kay_home.path());
        }

        let dirs = default_prompts_dirs();
        assert!(dirs.is_empty(), "host prompts should not be inherited");
    }

    #[serial_test::serial]
    #[tokio::test]
    async fn prefers_local_prompts_over_host_prompts() {
        let host_home = tempdir().expect("create host tempdir");
        let kay_home = tempdir().expect("create kay tempdir");
        let kay_prompts = kay_home.path().join("prompts");
        let host_prompts = host_home.path().join(".codex/prompts");
        fs::create_dir_all(&kay_prompts).unwrap();
        fs::create_dir_all(&host_prompts).unwrap();
        fs::write(kay_prompts.join("active.md"), "local").unwrap();
        fs::write(host_prompts.join("legacy.md"), "host").unwrap();

        let _host_guard = EnvVarGuard::new("CODEX_HOST_HOME");
        let _kay_guard = EnvVarGuard::new("KAY_HOME");
        unsafe {
            std::env::set_var("CODEX_HOST_HOME", host_home.path().join(".codex"));
            std::env::set_var("KAY_HOME", kay_home.path());
        }

        let dirs = default_prompts_dirs();
        let expected_dirs = vec![
            std::fs::canonicalize(&kay_prompts).unwrap_or(kay_prompts),
        ];
        assert_eq!(dirs, expected_dirs);

        let found = discover_prompts_in_roots(&dirs).await;
        let names: Vec<String> = found.iter().map(|e| e.name.clone()).collect();
        assert_eq!(names, vec!["active"]);
    }
}
