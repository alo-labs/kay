use std::io::Write;
use std::process::Command;
use std::process::Stdio;

use code_core::auth::get_auth_file;
use code_core::auth::try_read_auth_json;
use tempfile::TempDir;

fn code_bin() -> &'static str {
    env!("CARGO_BIN_EXE_code")
}

fn run_login(code_home: &TempDir, args: &[&str], stdin: Option<&str>) -> std::process::Output {
    let mut command = Command::new(code_bin());
    command
        .arg("login")
        .env("CODE_HOME", code_home.path())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if stdin.is_some() {
        command.stdin(Stdio::piped());
    } else {
        command.stdin(Stdio::null());
    }

    for arg in args {
        command.arg(arg);
    }

    let mut child = command.spawn().expect("spawn code login");

    if let Some(input) = stdin {
        child
            .stdin
            .as_mut()
            .expect("stdin should be piped")
            .write_all(input.as_bytes())
            .expect("write api key");
    }

    child.wait_with_output().expect("wait for code login")
}

fn auth_state(code_home: &TempDir) -> code_core::auth::AuthDotJson {
    let auth_file = get_auth_file(code_home.path());
    try_read_auth_json(&auth_file).expect("read auth.json")
}

fn assert_success(output: &std::process::Output) {
    assert!(
        output.status.success(),
        "command failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn provider_api_key_entry_help_shows_both_modes() {
    let output = Command::new(code_bin())
        .arg("login")
        .arg("--help")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("run code login --help");

    assert_success(&output);

    let help = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(help.contains("--api-key <API_KEY>"), "help output:\n{help}");
    assert!(
        help.contains("--with-api-key"),
        "help output:\n{help}"
    );
    assert!(
        !help.to_ascii_lowercase().contains("deprecated"),
        "help output still mentions deprecation:\n{help}"
    );
}

#[test]
fn provider_api_key_entry_direct_openai_uses_openai_login_path() {
    let code_home = TempDir::new().expect("temp CODE_HOME");

    let output = run_login(&code_home, &["--api-key", "sk-openai"], None);
    assert_success(&output);

    let auth = auth_state(&code_home);
    assert_eq!(auth.openai_api_key.as_deref(), Some("sk-openai"));
    assert!(auth.provider_credentials.is_empty());
}

#[test]
fn provider_api_key_entry_direct_minimax_uses_provider_storage() {
    let code_home = TempDir::new().expect("temp CODE_HOME");

    let output = run_login(
        &code_home,
        &["--provider", "minimax", "--api-key", "sk-minimax"],
        None,
    );
    assert_success(&output);

    let auth = auth_state(&code_home);
    assert_eq!(
        auth.provider_api_key("minimax"),
        Some("sk-minimax"),
        "provider credentials should be stored for minimax"
    );
    assert!(auth.openai_api_key.is_none());
}

#[test]
fn provider_api_key_entry_stdin_openai_uses_openai_login_path() {
    let code_home = TempDir::new().expect("temp CODE_HOME");

    let output = run_login(&code_home, &["--with-api-key"], Some("sk-openai"));
    assert_success(&output);

    let auth = auth_state(&code_home);
    assert_eq!(auth.openai_api_key.as_deref(), Some("sk-openai"));
    assert!(auth.provider_credentials.is_empty());
}

#[test]
fn provider_api_key_entry_stdin_opencode_go_uses_provider_storage() {
    let code_home = TempDir::new().expect("temp CODE_HOME");

    let output = run_login(
        &code_home,
        &["--provider", "opencode-go", "--with-api-key"],
        Some("sk-opencode-go"),
    );
    assert_success(&output);

    let auth = auth_state(&code_home);
    assert_eq!(
        auth.provider_api_key("opencode-go"),
        Some("sk-opencode-go"),
        "provider credentials should be stored for opencode-go"
    );
    assert!(auth.openai_api_key.is_none());
}
