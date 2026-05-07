use dirs::home_dir;
use std::env;
use std::path::Path;
use std::path::PathBuf;

const HOST_CODEX_DIR_NAME: &str = ".codex";

pub(crate) fn host_codex_home_dir() -> Option<PathBuf> {
    if let Some(path) = env::var_os("CODEX_HOST_HOME") {
        let path = PathBuf::from(path);
        if path.as_os_str().is_empty() {
            return None;
        }
        return Some(path);
    }

    home_dir().map(|home| home.join(HOST_CODEX_DIR_NAME))
}

pub(crate) fn host_codex_path(relative: &Path) -> Option<PathBuf> {
    host_codex_home_dir().map(|home| home.join(relative))
}
