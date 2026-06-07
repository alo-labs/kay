use std::io::Write;
use std::process::Command;
use std::process::Stdio;

use tempfile::TempDir;

mod common;
use common::SessionPreserver;

fn live_key() -> Option<String> {
    std::env::var("MINIMAX_LIVE_API_KEY")
        .ok()
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty())
}

fn code_bin() -> &'static str {
    env!("CARGO_BIN_EXE_code")
}

fn login_minimax(code_home: &TempDir, api_key: &str) {
    let mut child = Command::new(code_bin())
        .arg("login")
        .arg("--provider")
        .arg("minimax")
        .arg("--with-api-key")
        .env("KAY_HOME", code_home.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn kay login");

    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(api_key.as_bytes())
        .expect("write minimax api key");

    let output = child.wait_with_output().expect("wait for kay login");
    assert!(
        output.status.success(),
        "kay login failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn run_minimax_exec(code_home: &TempDir, prompt: &str) -> String {
    let output = Command::new(code_bin())
        .arg("exec")
        .arg("--skip-git-repo-check")
        .arg("-c")
        .arg("model_provider=minimax")
        .arg("-c")
        .arg("model=MiniMax-M3")
        .arg(prompt)
        .env("KAY_HOME", code_home.path())
        .env_remove("OPENAI_API_KEY")
        .stdin(Stdio::null())
        .output()
        .expect("run kay exec");

    assert!(
        output.status.success(),
        "kay exec failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).to_string()
}

fn output_has_exact_line(output: &str, expected: &str) -> bool {
    output.lines().any(|line| line.trim() == expected)
}

fn first_json_object(output: &str) -> Option<serde_json::Value> {
    output
        .lines()
        .map(str::trim)
        .filter(|line| line.starts_with('{') && line.ends_with('}'))
        .find_map(|line| serde_json::from_str(line).ok())
}

#[test]
fn minimax_m3_live_exec_edge_cases() {
    let Some(api_key) = live_key() else {
        eprintln!("skipping MiniMax live E2E: MINIMAX_LIVE_API_KEY is not set");
        return;
    };

    let code_home = TempDir::new().expect("temp KAY_HOME");
    let _sessions = SessionPreserver::new(code_home.path(), "minimax_live_e2e");
    login_minimax(&code_home, &api_key);

    let exact = run_minimax_exec(&code_home, "Reply with exactly OK.");
    assert!(
        output_has_exact_line(&exact, "OK"),
        "expected exact OK line, got:\n{exact}"
    );

    let json = run_minimax_exec(
        &code_home,
        "Return only this compact JSON object, with no markdown: {\"provider\":\"minimax\",\"model\":\"M3\",\"ok\":true}",
    );
    let parsed =
        first_json_object(&json).unwrap_or_else(|| panic!("expected JSON object, got:\n{json}"));
    assert_eq!(parsed["provider"], "minimax");
    assert_eq!(parsed["model"], "M3");
    assert_eq!(parsed["ok"], true);

    let role_collapse = run_minimax_exec(
        &code_home,
        "Reply with exactly ROLE_OK. Do not add punctuation or explanation.",
    );
    assert!(
        output_has_exact_line(&role_collapse, "ROLE_OK"),
        "expected exact ROLE_OK line, got:\n{role_collapse}"
    );
}
