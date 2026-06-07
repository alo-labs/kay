use crate::model_provider::hermes::HermesProviderExport;
use std::io;
use std::path::Path;
use std::process::Command;
use std::process::Stdio;

pub fn load_hermes_provider_exports(source: &Path) -> io::Result<Vec<HermesProviderExport>> {
    let script = r#"
import json
import sys
from pathlib import Path

source = Path(sys.argv[1]).resolve()
sys.path.insert(0, str(source))

from providers import list_providers
from providers.base import ProviderProfile

hooks = [
    "prepare_messages",
    "build_extra_body",
    "build_api_kwargs_extras",
    "get_max_tokens",
    "fetch_models",
    "get_hostname",
]

exports = []
for profile in sorted(list_providers(), key=lambda item: item.name):
    overridden = [
        hook
        for hook in hooks
        if getattr(type(profile), hook, None) is not getattr(ProviderProfile, hook, None)
    ]
    exports.append({
        "name": profile.name,
        "aliases": list(profile.aliases or ()),
        "display_name": profile.display_name or "",
        "description": profile.description or "",
        "signup_url": profile.signup_url or "",
        "api_mode": profile.api_mode or "chat_completions",
        "env_vars": list(profile.env_vars or ()),
        "base_url": profile.base_url or "",
        "models_url": profile.models_url or "",
        "auth_type": profile.auth_type or "api_key",
        "supports_vision": bool(profile.supports_vision),
        "fallback_models": list(profile.fallback_models or ()),
        "default_headers": dict(profile.default_headers or {}),
        "overridden_hooks": overridden,
    })

print(json.dumps(exports, sort_keys=True))
"#;

    let output = Command::new("python3")
        .arg("-c")
        .arg(script)
        .arg(source)
        .stdin(Stdio::null())
        .output()
        .map_err(|err| io::Error::other(format!("failed to run python3: {err}")))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(io::Error::other(format!(
            "Hermes provider export failed with status {}: {stderr}",
            output.status
        )));
    }

    serde_json::from_slice(&output.stdout).map_err(|err| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to parse Hermes provider export JSON: {err}"),
        )
    })
}
