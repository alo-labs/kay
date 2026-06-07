use crate::model_provider::profile::ProviderProfile;
use std::fs;
use std::io;
use std::path::Path;
use std::path::PathBuf;

pub fn load_provider_profiles_from_home(code_home: &Path) -> io::Result<Vec<ProviderProfile>> {
    let root = code_home.join("provider_profiles");
    let mut files = Vec::new();
    collect_profile_files(&root, &mut files)?;
    files.sort();

    let mut profiles = Vec::new();
    for path in files {
        profiles.push(load_provider_profile_file(&path)?);
    }
    profiles.sort_by(|a, b| a.id.cmp(&b.id));
    Ok(profiles)
}

fn collect_profile_files(dir: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) if err.kind() == io::ErrorKind::NotFound => return Ok(()),
        Err(err) => return Err(err),
    };
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_profile_files(&path, files)?;
        } else if matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("json" | "toml")
        ) {
            files.push(path);
        }
    }
    Ok(())
}

fn load_provider_profile_file(path: &Path) -> io::Result<ProviderProfile> {
    let contents = fs::read_to_string(path)?;
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("json") => serde_json::from_str(&contents).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to parse provider profile {}: {err}", path.display()),
            )
        }),
        Some("toml") => toml::from_str(&contents).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("failed to parse provider profile {}: {err}", path.display()),
            )
        }),
        _ => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unsupported provider profile extension: {}", path.display()),
        )),
    }
}
