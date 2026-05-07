use dirs::home_dir;
use std::env;
use std::path::Path;
use std::path::PathBuf;

const HOST_CODEX_DIR_NAME: &str = ".codex";

/// Returns the host Codex home directory.
///
/// By default this is `~/.codex`, but tests and launchers may override it with
/// `CODEX_HOST_HOME`.
pub fn host_codex_home_dir() -> Option<PathBuf> {
    if let Some(path) = env::var_os("CODEX_HOST_HOME") {
        let path = PathBuf::from(path);
        if path.as_os_str().is_empty() {
            return None;
        }
        return Some(path);
    }

    home_dir().map(|home| home.join(HOST_CODEX_DIR_NAME))
}

/// Resolve a path inside the host Codex home directory.
pub fn host_codex_path(relative: &Path) -> Option<PathBuf> {
    host_codex_home_dir().map(|home| home.join(relative))
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[serial_test::serial]
    #[test]
    fn host_codex_home_dir_prefers_explicit_env_var() {
        let tmp = tempdir().expect("tempdir");
        let _guard = EnvVarGuard::new("CODEX_HOST_HOME");
        unsafe {
            std::env::set_var("CODEX_HOST_HOME", tmp.path());
        }

        assert_eq!(host_codex_home_dir().as_deref(), Some(tmp.path()));
    }

    #[serial_test::serial]
    #[test]
    fn host_codex_path_resolves_inside_host_home() {
        let tmp = tempdir().expect("tempdir");
        let _guard = EnvVarGuard::new("CODEX_HOST_HOME");
        unsafe {
            std::env::set_var("CODEX_HOST_HOME", tmp.path());
        }

        let resolved = host_codex_path(Path::new("skills/example/SKILL.md"));
        assert_eq!(
            resolved.as_deref(),
            Some(tmp.path().join("skills/example/SKILL.md").as_path())
        );
    }
}
