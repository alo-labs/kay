use std::fs;
use std::process::Command;
use std::process::Stdio;

use tempfile::TempDir;

fn code_bin() -> &'static str {
    env!("CARGO_BIN_EXE_code")
}

fn write_hermes_fixture(dir: &TempDir) {
    fs::create_dir_all(dir.path().join("providers")).expect("providers dir");
    fs::create_dir_all(dir.path().join("plugins/model-providers/novita")).expect("plugin dir");
    fs::write(
        dir.path().join("providers/base.py"),
        r#"
class ProviderProfile:
    def __init__(self, **kwargs):
        self.name = kwargs.get("name", "")
        self.api_mode = kwargs.get("api_mode", "chat_completions")
        self.aliases = kwargs.get("aliases", ())
        self.display_name = kwargs.get("display_name", "")
        self.description = kwargs.get("description", "")
        self.signup_url = kwargs.get("signup_url", "")
        self.env_vars = kwargs.get("env_vars", ())
        self.base_url = kwargs.get("base_url", "")
        self.models_url = kwargs.get("models_url", "")
        self.auth_type = kwargs.get("auth_type", "api_key")
        self.supports_vision = kwargs.get("supports_vision", False)
        self.fallback_models = kwargs.get("fallback_models", ())
        self.default_headers = kwargs.get("default_headers", {})

    def prepare_messages(self, messages): return messages
    def build_extra_body(self, **context): return {}
    def build_api_kwargs_extras(self, **context): return ({}, {})
    def get_max_tokens(self, model): return None
    def fetch_models(self, **context): return None
    def get_hostname(self): return ""
"#,
    )
    .expect("base.py");
    fs::write(
        dir.path().join("providers/__init__.py"),
        r#"
from pathlib import Path
from providers.base import ProviderProfile

_REGISTRY = []

def register_provider(profile):
    _REGISTRY.append(profile)

def list_providers():
    if not _REGISTRY:
        root = Path(__file__).resolve().parent.parent
        init = root / "plugins" / "model-providers" / "novita" / "__init__.py"
        exec(compile(init.read_text(), str(init), "exec"), {})
    return list(_REGISTRY)
"#,
    )
    .expect("__init__.py");
    fs::write(
        dir.path()
            .join("plugins/model-providers/novita/__init__.py"),
        r#"
from providers import register_provider
from providers.base import ProviderProfile

register_provider(ProviderProfile(
    name="novita",
    aliases=("novita-ai",),
    display_name="Novita",
    env_vars=("NOVITA_API_KEY",),
    base_url="https://api.novita.ai/v3/openai",
))
"#,
    )
    .expect("plugin __init__.py");
}

#[test]
fn providers_import_hermes_install_writes_compiled_profiles() {
    let kay_home = TempDir::new().expect("temp KAY_HOME");
    let hermes = TempDir::new().expect("temp Hermes source");
    write_hermes_fixture(&hermes);

    let output = Command::new(code_bin())
        .arg("providers")
        .arg("import-hermes")
        .arg("--source")
        .arg(hermes.path())
        .arg("--install")
        .env("KAY_HOME", kay_home.path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run provider import");

    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let profile_path = kay_home
        .path()
        .join("provider_profiles/hermes/novita.json");
    let profile = fs::read_to_string(profile_path).expect("compiled profile");

    assert!(profile.contains(r#""id": "novita""#), "profile:\n{profile}");
    assert!(
        profile.contains(r#""compatibility_adapter": "generic_openai_chat""#),
        "profile:\n{profile}"
    );
}
