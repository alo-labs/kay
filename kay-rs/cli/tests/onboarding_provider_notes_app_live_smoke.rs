#![cfg(test)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;
use std::sync::mpsc;
use std::sync::mpsc::RecvTimeoutError;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;

use code_core::config_types::ReasoningEffort;
use portable_pty::CommandBuilder;
use portable_pty::PtySize;
use portable_pty::native_pty_system;
use serde_json::Value;
use tempfile::TempDir;

mod common;
use common::SessionPreserver;

const TEST_NOTES_APP_REPO_ROOT: &str = "/Users/shafqat/projects/test-notes-app";
const ONBOARDING_TIMEOUT: Duration = Duration::from_secs(45);
const DEFAULT_LIVE_TURN_TIMEOUT: Duration = Duration::from_secs(900);
const TUI_ROWS: u16 = 60;
const TUI_COLS: u16 = 180;

struct LiveKeySet {
    opencode_go: Option<String>,
    minimax: Option<String>,
}

struct LiveModelSpec {
    provider_label: &'static str,
    provider_id: &'static str,
    model: &'static str,
    header_label: &'static str,
    reasoning_effort: Option<ReasoningEffort>,
}

const LIVE_MODELS: &[LiveModelSpec] = &[
    LiveModelSpec {
        provider_label: "OpenCode Go",
        provider_id: "opencode-go",
        model: "opencode-go/glm-5.1",
        header_label: "opencode-go/glm-5.1",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenCode Go",
        provider_id: "opencode-go",
        model: "opencode-go/glm-5",
        header_label: "opencode-go/glm-5",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenCode Go",
        provider_id: "opencode-go",
        model: "opencode-go/kimi-k2.7-code",
        header_label: "opencode-go/kimi-k2.7-code",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenCode Go",
        provider_id: "opencode-go",
        model: "opencode-go/kimi-k2.6",
        header_label: "opencode-go/kimi-k2.6",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenCode Go",
        provider_id: "opencode-go",
        model: "opencode-go/mimo-v2.5-pro",
        header_label: "opencode-go/mimo-v2.5-pro",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenCode Go",
        provider_id: "opencode-go",
        model: "opencode-go/mimo-v2.5",
        header_label: "opencode-go/mimo-v2.5",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenCode Go",
        provider_id: "opencode-go",
        model: "opencode-go/minimax-m2.7",
        header_label: "opencode-go/minimax-m2.7",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenCode Go",
        provider_id: "opencode-go",
        model: "opencode-go/minimax-m3",
        header_label: "opencode-go/minimax-m3",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenCode Go",
        provider_id: "opencode-go",
        model: "opencode-go/qwen3.7-max",
        header_label: "opencode-go/qwen3.7-max",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenCode Go",
        provider_id: "opencode-go",
        model: "opencode-go/qwen3.7-plus",
        header_label: "opencode-go/qwen3.7-plus",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenCode Go",
        provider_id: "opencode-go",
        model: "opencode-go/qwen3.6-plus",
        header_label: "opencode-go/qwen3.6-plus",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenCode Go",
        provider_id: "opencode-go",
        model: "opencode-go/deepseek-v4-pro",
        header_label: "opencode-go/deepseek-v4-pro",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenCode Go",
        provider_id: "opencode-go",
        model: "opencode-go/deepseek-v4-flash",
        header_label: "opencode-go/deepseek-v4-flash",
        reasoning_effort: Some(ReasoningEffort::XHigh),
    },
    LiveModelSpec {
        provider_label: "MiniMax",
        provider_id: "minimax",
        model: "MiniMax-M3",
        header_label: "MiniMax-M3",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenAI",
        provider_id: "openai",
        model: "gpt-5.5",
        header_label: "GPT-5.5",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenAI",
        provider_id: "openai",
        model: "gpt-5.4",
        header_label: "GPT-5.4",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenAI",
        provider_id: "openai",
        model: "gpt-5.4-mini",
        header_label: "GPT-5.4-Mini",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenAI",
        provider_id: "openai",
        model: "gpt-5.3-codex-spark",
        header_label: "GPT-5.3-Codex-Spark",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenAI",
        provider_id: "openai",
        model: "gpt-5.2",
        header_label: "GPT-5.2",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenAI",
        provider_id: "openai",
        model: "gpt-5.1-codex-max",
        header_label: "GPT-5.1-Codex-Max",
        reasoning_effort: None,
    },
    LiveModelSpec {
        provider_label: "OpenAI",
        provider_id: "openai",
        model: "gpt-5.1-codex-mini",
        header_label: "GPT-5.1-Codex-Mini",
        reasoning_effort: None,
    },
];

const DEFAULT_LIVE_MODEL_IDS: &[&str] = &[
    "opencode-go/mimo-v2.5",
    "opencode-go/mimo-v2.5-pro",
    "opencode-go/deepseek-v4-flash",
    "opencode-go/minimax-m2.7",
];

fn code_bin() -> &'static str {
    env!("CARGO_BIN_EXE_code")
}

fn repo_root() -> PathBuf {
    std::env::var_os("TEST_NOTES_APP_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(TEST_NOTES_APP_REPO_ROOT))
}

fn env_key(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn live_keys() -> Option<LiveKeySet> {
    let models = selected_live_models();
    let needs_opencode_go = models
        .iter()
        .any(|spec| spec.provider_id == "opencode-go");
    let needs_minimax = models.iter().any(|spec| spec.provider_id == "minimax");
    if !needs_opencode_go && !needs_minimax {
        return None;
    }

    let opencode_go = if needs_opencode_go {
        Some(env_key("OPENCODE_GO_LIVE_API_KEY")?)
    } else {
        None
    };
    let minimax = if needs_minimax {
        Some(env_key("MINIMAX_LIVE_API_KEY")?)
    } else {
        None
    };

    Some(LiveKeySet {
        opencode_go,
        minimax,
    })
}

fn live_turn_timeout() -> Duration {
    env_key("KAY_ONBOARDING_LIVE_SMOKE_TURN_TIMEOUT_SECS")
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_LIVE_TURN_TIMEOUT)
}

fn default_live_models() -> Vec<&'static LiveModelSpec> {
    DEFAULT_LIVE_MODEL_IDS
        .iter()
        .filter_map(|wanted| {
            LIVE_MODELS
                .iter()
                .find(|spec| spec.model.eq_ignore_ascii_case(wanted))
        })
        .collect()
}

fn selected_live_models() -> Vec<&'static LiveModelSpec> {
    let Some(filter) = env_key("KAY_ONBOARDING_LIVE_SMOKE_MODEL_FILTER") else {
        return default_live_models();
    };
    let requested = filter
        .split(',')
        .map(|value| value.trim().to_ascii_lowercase())
        .filter(|value| !value.is_empty())
        .collect::<Vec<_>>();
    if requested.is_empty() {
        return default_live_models();
    }
    LIVE_MODELS
        .iter()
        .filter(|spec| {
            requested.iter().any(|wanted| {
                let provider_label = spec.provider_label.to_ascii_lowercase();
                let provider_id = spec.provider_id.to_ascii_lowercase();
                let model = spec.model.to_ascii_lowercase();
                let header = spec.header_label.to_ascii_lowercase();
                provider_label == *wanted
                    || provider_id == *wanted
                    || model == *wanted
                    || header == *wanted
            })
        })
        .collect()
}

fn clone_notes_app() -> (TempDir, PathBuf) {
    let temp = TempDir::new().expect("create temp notes-app workspace");
    let worktree = temp.path().join("test-notes-app");
    let output = Command::new("git")
        .arg("clone")
        .arg("--no-local")
        .arg(repo_root())
        .arg(&worktree)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("clone notes app");

    assert!(
        output.status.success(),
        "git clone failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    (temp, worktree)
}

fn redact(text: &str, keys: &LiveKeySet) -> String {
    let mut redacted = text.to_string();
    for key in keys
        .opencode_go
        .iter()
        .chain(keys.minimax.iter())
        .filter(|key| !key.is_empty())
    {
        redacted = redacted.replace(key, "[REDACTED_API_KEY]");
    }
    redacted
}

fn screen_from_output(output: &[u8]) -> String {
    let mut parser = vt100::Parser::new(TUI_ROWS, TUI_COLS, 0);
    parser.process(output);
    parser.screen().contents()
}

fn wait_for_screen(
    rx: &mpsc::Receiver<Vec<u8>>,
    output: &mut Vec<u8>,
    keys: &LiveKeySet,
    expected: &[&str],
    timeout: Duration,
) -> String {
    let deadline = Instant::now() + timeout;
    loop {
        let screen = screen_from_output(output);
        if expected.iter().all(|needle| screen.contains(needle)) {
            return screen;
        }

        let remaining = deadline.checked_duration_since(Instant::now()).unwrap_or_default();
        assert!(
            remaining > Duration::ZERO,
            "timed out waiting for {:?}; screen:\n{}",
            expected,
            redact(&screen, keys)
        );

        match rx.recv_timeout(remaining.min(Duration::from_millis(100))) {
            Ok(chunk) => output.extend_from_slice(&chunk),
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => {
                panic!(
                    "Kay TUI exited while waiting for {:?}; screen:\n{}",
                    expected,
                    redact(&screen, keys)
                );
            }
        }
    }
}

fn write_line(writer: &mut dyn Write, value: &str) {
    writer.write_all(value.as_bytes()).expect("write pty input");
    writer.flush().expect("flush pty input");
    thread::sleep(Duration::from_millis(50));
    writer.write_all(b"\r").expect("write enter");
    writer.flush().expect("flush pty input");
}

fn write_key(writer: &mut dyn Write, value: &[u8]) {
    writer.write_all(value).expect("write pty key");
    writer.flush().expect("flush pty key");
}

struct TuiHarness {
    child: Option<Box<dyn portable_pty::Child + Send>>,
    writer: Option<Box<dyn Write + Send>>,
    rx: mpsc::Receiver<Vec<u8>>,
    reader_handle: Option<JoinHandle<()>>,
    output: Vec<u8>,
}

impl TuiHarness {
    fn wait_for(&mut self, keys: &LiveKeySet, expected: &[&str], timeout: Duration) -> String {
        wait_for_screen(&self.rx, &mut self.output, keys, expected, timeout)
    }

    fn write_line(&mut self, value: &str) {
        write_line(
            self.writer.as_mut().expect("pty writer").as_mut(),
            value,
        );
    }

    fn write_composer_line(&mut self, value: &str) {
        let writer = self.writer.as_mut().expect("pty writer");
        for byte in value.as_bytes() {
            writer.write_all(&[*byte]).expect("write pty input");
            writer.flush().expect("flush pty input");
            thread::sleep(Duration::from_millis(12));
        }
        thread::sleep(Duration::from_millis(180));
        writer.write_all(b"\r").expect("write submit key");
        writer.flush().expect("flush pty input");
    }

    fn write_key(&mut self, value: &[u8]) {
        write_key(
            self.writer.as_mut().expect("pty writer").as_mut(),
            value,
        );
    }

    fn current_screen(&self) -> String {
        screen_from_output(&self.output)
    }

    fn shutdown(&mut self) {
        if let Some(mut writer) = self.writer.take() {
            let _ = writer.write_all(b"\x03");
            let _ = writer.flush();
        }
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
        if let Some(handle) = self.reader_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for TuiHarness {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn start_onboarding_provider_setup(
    kay_home: &Path,
    repo_dir: &Path,
    keys: &LiveKeySet,
) -> TuiHarness {
    fs::write(
        kay_home.join("config.toml"),
        "[tools]\nbrowser = false\nview_image = false\n\n[subagents]\nenabled = false\n",
    )
    .expect("write live smoke Kay config");

    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: TUI_ROWS,
            cols: TUI_COLS,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("open pty");

    let mut command = CommandBuilder::new(code_bin());
    command.arg("--cd");
    command.arg(repo_dir);
    command.env("KAY_HOME", kay_home);
    command.env("TERM", "xterm-256color");
    command.env("NO_COLOR", "1");
    command.env("CODE_SKIP_TUI_TERMINAL_QUERIES", "1");
    command.env("CODE_DISABLE_THEME_AUTODETECT", "1");
    command.env("CODE_DISABLE_FOCUS", "1");
    command.env("CODE_DISABLE_KBD_ENHANCEMENT", "1");
    command.env("CODEX_TUI_RECORD_SESSION", "1");
    command.env(
        "CODEX_TUI_SESSION_LOG_PATH",
        kay_home.join("live-smoke-session.jsonl"),
    );

    let child = pair.slave.spawn_command(command).expect("spawn Kay TUI");
    let mut reader = pair.master.try_clone_reader().expect("clone pty reader");
    let writer = pair.master.take_writer().expect("take pty writer");

    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    let reader_handle = thread::spawn(move || {
        let mut buf = [0_u8; 8192];
        loop {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send(buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(err) if err.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(_) => break,
            }
        }
    });

    let mut harness = TuiHarness {
        child: Some(child),
        writer: Some(writer),
        rx,
        reader_handle: Some(reader_handle),
        output: Vec::new(),
    };

    harness.wait_for(
        keys,
        &["Welcome to Kay", "Connect provider API keys"],
        ONBOARDING_TIMEOUT,
    );

    harness.write_key(b"2");
    harness.wait_for(
        keys,
        &["You are running Kay in", "allow Kay to work in this folder"],
        ONBOARDING_TIMEOUT,
    );
    harness.write_key(b"\r");
    harness.wait_for(
        keys,
        &["Manage Providers", "Xiaomi", "OpenCode Go", "MiniMax", "OpenAI"],
        ONBOARDING_TIMEOUT,
    );

    if let Some(opencode_go_key) = keys.opencode_go.as_deref() {
        harness.write_key(b"\x1b[B");
        harness.write_key(b"\r");
        harness.wait_for(
            keys,
            &["Editing OpenCode Go provider key"],
            ONBOARDING_TIMEOUT,
        );
        harness.write_line(opencode_go_key);
        harness.wait_for(
            keys,
            &["OpenCode Go API key saved", "OpenCode Go", "(configured)"],
            ONBOARDING_TIMEOUT,
        );
    }

    if let Some(minimax_key) = keys.minimax.as_deref() {
        let down_steps = if keys.opencode_go.is_some() { 1 } else { 2 };
        for _ in 0..down_steps {
            harness.write_key(b"\x1b[B");
        }
        harness.write_key(b"\r");
        harness.wait_for(
            keys,
            &["Editing MiniMax provider key"],
            ONBOARDING_TIMEOUT,
        );
        harness.write_line(minimax_key);
        harness.wait_for(
            keys,
            &["MiniMax API key saved", "MiniMax", "(configured)"],
            ONBOARDING_TIMEOUT,
        );
    }

    harness.write_key(b"\x1b");
    harness.wait_for(keys, &["Model:", "What can I code"], ONBOARDING_TIMEOUT);
    harness
}

fn auth_json(kay_home: &Path) -> Value {
    let auth_path = kay_home.join("auth.json");
    let raw = fs::read_to_string(&auth_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", auth_path.display()));
    serde_json::from_str(&raw).expect("parse auth.json")
}

fn assert_onboarding_saved_credentials(kay_home: &Path, keys: &LiveKeySet) {
    let auth = auth_json(kay_home);
    let mut expected = Vec::new();
    if keys.opencode_go.is_some() {
        expected.push("opencode-go");
    }
    if keys.minimax.is_some() {
        expected.push("minimax");
    }
    for provider in expected {
        assert!(
            auth.pointer(&format!("/provider_credentials/{provider}/api_key"))
                .and_then(Value::as_str)
                .is_some_and(|key| !key.trim().is_empty()),
            "{provider} API key was not saved by onboarding provider manager"
        );
    }
}

fn build_live_work_prompt(run_id: &str) -> String {
    "You are validating Kay against the live note-taking app in this repository. \
Use only shell commands to inspect package.json and src/server.js. \
Do not use browser, web_fetch, image, screenshot, network, or URL tools. Do not edit files. \
Return only one compact JSON object with exactly these keys: \
run_id, app_name, evidence, ok. \
Set run_id exactly to RUN_ID_PLACEHOLDER. Set ok to true only if you inspected the files. \
The evidence value must be one concise fact from src/server.js."
        .replace("RUN_ID_PLACEHOLDER", run_id)
}

fn json_objects(text: &str) -> Vec<(Value, String)> {
    let mut objects = Vec::new();
    for (start, _) in text.match_indices('{') {
        let mut depth = 0_i32;
        let mut in_string = false;
        let mut escaped = false;
        for (offset, ch) in text[start..].char_indices() {
            if in_string {
                if escaped {
                    escaped = false;
                } else if ch == '\\' {
                    escaped = true;
                } else if ch == '"' {
                    in_string = false;
                }
                continue;
            }
            match ch {
                '"' => in_string = true,
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        let end = start + offset + ch.len_utf8();
                        let raw = &text[start..end];
                        if let Some((value, parsed_raw)) = parse_json_candidate(raw) {
                            objects.push((value, parsed_raw));
                        }
                        break;
                    }
                }
                _ => {}
            }
        }
    }
    objects
}

fn parse_json_candidate(raw: &str) -> Option<(Value, String)> {
    if let Ok(value) = serde_json::from_str(raw) {
        return Some((value, raw.to_string()));
    }

    let joined = raw.lines().map(str::trim).collect::<String>();
    serde_json::from_str(&joined)
        .ok()
        .map(|value| (value, joined))
}

fn strip_tui_box_edges(text: &str) -> String {
    text.lines()
        .map(|line| {
            let trimmed = line.trim();
            let unboxed = trimmed
                .strip_prefix('│')
                .unwrap_or(trimmed)
                .trim()
                .strip_suffix('│')
                .unwrap_or_else(|| trimmed.strip_prefix('│').unwrap_or(trimmed).trim())
                .trim();
            unboxed.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn live_work_response_from_screen(screen: &str, run_id: &str) -> Option<(Value, String)> {
    let is_identity_response = |(value, _): &(Value, String)| {
        value.get("run_id").and_then(Value::as_str) == Some(run_id)
            && value.get("app_name").is_some()
            && value.get("evidence").is_some()
            && value.get("ok").is_some()
    };

    json_objects(&strip_tui_box_edges(screen))
        .into_iter()
        .rev()
        .find(is_identity_response)
        .or_else(|| json_objects(screen).into_iter().rev().find(is_identity_response))
}

fn wait_for_identity_response(
    harness: &mut TuiHarness,
    keys: &LiveKeySet,
    spec: &LiveModelSpec,
    run_id: &str,
) -> String {
    let deadline = Instant::now() + live_turn_timeout();
    loop {
        let screen = harness.current_screen();
        for failure in [
            "Kay runtime error:",
            "Authentication expired.",
            "You exceeded your current quota",
        ] {
            assert!(
                !screen.contains(failure),
                "live turn failed for {} / {} after matching `{failure}`; screen:\n{}",
                spec.provider_label,
                spec.model,
                redact(&screen, keys)
            );
        }

        if let Some((parsed, raw)) = live_work_response_from_screen(&screen, run_id) {
            assert_eq!(parsed["ok"], true);
            assert!(
                parsed["app_name"].as_str().is_some_and(|name| !name.is_empty()),
                "expected app_name in response for {} / {}: {raw}",
                spec.provider_label,
                spec.model
            );
            assert!(
                parsed["evidence"]
                    .as_str()
                    .is_some_and(|evidence| !evidence.is_empty()),
                "expected evidence in response for {} / {}: {raw}",
                spec.provider_label,
                spec.model
            );
            return raw;
        }

        let remaining = deadline.checked_duration_since(Instant::now()).unwrap_or_default();
        assert!(
            remaining > Duration::ZERO,
            "timed out waiting for live-work JSON from {} / {}; screen:\n{}",
            spec.provider_label,
            spec.model,
            redact(&screen, keys)
        );

        match harness.rx.recv_timeout(remaining.min(Duration::from_millis(200))) {
            Ok(chunk) => harness.output.extend_from_slice(&chunk),
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => {
                panic!(
                    "Kay TUI exited while waiting for live-work JSON from {} / {}; screen:\n{}",
                    spec.provider_label,
                    spec.model,
                    redact(&screen, keys)
                );
            }
        }
    }
}

fn extract_model_header_line(screen: &str) -> String {
    screen
        .lines()
        .find(|line| line.contains("Model:"))
        .unwrap_or("")
        .trim()
        .to_string()
}

fn header_shows_exact_model(header_line: &str, model: &str) -> bool {
    let Some((_, after_model)) = header_line.split_once("Model:") else {
        return false;
    };
    let displayed = after_model
        .split(['(', '•'])
        .next()
        .unwrap_or(after_model)
        .trim();
    displayed.eq_ignore_ascii_case(model)
}

fn wait_for_model_header(
    harness: &mut TuiHarness,
    keys: &LiveKeySet,
    spec: &LiveModelSpec,
) -> String {
    let deadline = Instant::now() + ONBOARDING_TIMEOUT;
    loop {
        let screen = harness.current_screen();
        let header_line = extract_model_header_line(&screen);
        if header_shows_exact_model(&header_line, spec.header_label)
            || header_shows_exact_model(&header_line, spec.model)
        {
            return header_line;
        }

        let remaining = deadline.checked_duration_since(Instant::now()).unwrap_or_default();
        assert!(
            remaining > Duration::ZERO,
            "TUI header did not show selected model {} after /model {}; header: {header_line}\nscreen:\n{}",
            spec.header_label,
            spec.model,
            redact(&screen, keys)
        );

        match harness.rx.recv_timeout(remaining.min(Duration::from_millis(100))) {
            Ok(chunk) => harness.output.extend_from_slice(&chunk),
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => {
                panic!(
                    "Kay TUI exited while waiting for selected model {} after /model {}; header: {header_line}\nscreen:\n{}",
                    spec.header_label,
                    spec.model,
                    redact(&screen, keys)
                );
            }
        }
    }
}

fn session_log_path(kay_home: &Path) -> PathBuf {
    kay_home.join("live-smoke-session.jsonl")
}

fn session_configure_count(kay_home: &Path) -> usize {
    let path = session_log_path(kay_home);
    let Ok(raw) = fs::read_to_string(path) else {
        return 0;
    };
    raw.lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|value| value.get("kind").and_then(Value::as_str) == Some("configure_session_model"))
        .count()
}

fn config_record_matches(record: &Value, spec: &LiveModelSpec) -> bool {
    record.get("model").and_then(Value::as_str) == Some(spec.model)
        && record.get("model_provider_id").and_then(Value::as_str) == Some(spec.provider_id)
}

fn assert_session_configured_model(
    kay_home: &Path,
    before_count: usize,
    keys: &LiveKeySet,
    spec: &LiveModelSpec,
) {
    let path = session_log_path(kay_home);
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read session log {}: {err}", path.display()));
    let records = raw
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|value| value.get("kind").and_then(Value::as_str) == Some("configure_session_model"))
        .collect::<Vec<_>>();

    let matching_new = records
        .iter()
        .skip(before_count)
        .any(|record| config_record_matches(record, spec));
    let matching_existing = records
        .iter()
        .any(|record| config_record_matches(record, spec));

    assert!(
        matching_new || matching_existing,
        "no session log record proved TUI configured provider `{}` with model `{}`; configure records:\n{}",
        spec.provider_id,
        spec.model,
        records
            .iter()
            .map(|record| redact(&record.to_string(), keys))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

fn run_tui_model_switch_and_live_notes_turn(
    harness: &mut TuiHarness,
    kay_home: &Path,
    keys: &LiveKeySet,
    spec: &LiveModelSpec,
    turn_index: usize,
) {
    let configure_count_before = session_configure_count(kay_home);
    harness.write_composer_line(&format!("/model {}", spec.model));
    let header_line = wait_for_model_header(harness, keys, spec);
    if let Some(reasoning_effort) = spec.reasoning_effort {
        harness.write_composer_line(&format!("/reasoning {}", reasoning_effort));
        let expected_value = format!("Value: {}", reasoning_effort);
        harness.wait_for(
            keys,
            &["Reasoning Effort", expected_value.as_str()],
            ONBOARDING_TIMEOUT,
        );
    }

    let run_id = format!(
        "kay-live-smoke-{}-{}",
        turn_index,
        spec.model.replace('/', "_")
    );
    let prompt = build_live_work_prompt(&run_id);
    println!(
        "\nLIVE_SMOKE_TUI_HEADER[{} / {}]\n{}\n",
        spec.provider_label, spec.model, header_line
    );
    println!("\nLIVE_SMOKE_PROMPT[{} / {}]\n{}\n", spec.provider_label, spec.model, prompt);

    harness.write_composer_line(&prompt);
    let response_wait = Instant::now();
    let response = wait_for_identity_response(harness, keys, spec, &run_id);
    let response_wait = response_wait.elapsed();
    assert_session_configured_model(kay_home, configure_count_before, keys, spec);

    println!(
        "LIVE_SMOKE_RESPONSE_WAIT[{} / {}]\n{:.3}s\n",
        spec.provider_label,
        spec.model,
        response_wait.as_secs_f64()
    );
    println!(
        "LIVE_SMOKE_RESPONSE[{} / {}]\n{}\n",
        spec.provider_label,
        spec.model,
        response
    );
}

fn assert_no_notes_app_diff(repo_dir: &Path) {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_dir)
        .arg("diff")
        .arg("--name-only")
        .arg("HEAD")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("git diff");
    assert!(
        output.status.success(),
        "git diff failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let diff = String::from_utf8_lossy(&output.stdout);
    assert!(
        diff.trim().is_empty(),
        "live smoke should inspect, not edit, the notes app; diff:\n{diff}"
    );
}

#[test]
fn live_work_response_parser_handles_box_wrapped_json() {
    let screen = r#"
   │  {"run_id":"kay-live-smoke-0-opencode-go_glm-5.1","app_name":"test-notes-app","evidence":"Server listens on PORT env var defaulting to 3456 and exposes /api/health returni  │
   │  ng {status:'ok'}","ok":true}                                                                                                                                                │
"#;

    let (parsed, raw) = live_work_response_from_screen(
        screen,
        "kay-live-smoke-0-opencode-go_glm-5.1",
    )
    .expect("parse wrapped JSON");

    assert_eq!(parsed["ok"], true);
    assert!(raw.contains("returning"));
}

#[test]
fn model_header_parser_rejects_prefix_model_match() {
    let header =
        "Kay  •  Model: opencode-go/mimo-v2.5-pro (Medium)  •  Directory: app";

    assert!(header_shows_exact_model(header, "opencode-go/mimo-v2.5-pro"));
    assert!(!header_shows_exact_model(header, "opencode-go/mimo-v2.5"));
}

#[test]
fn default_live_models_are_curated_opencode_go_release_matrix() {
    let models = default_live_models();
    let model_ids = models.iter().map(|spec| spec.model).collect::<Vec<_>>();

    assert_eq!(model_ids.as_slice(), DEFAULT_LIVE_MODEL_IDS);
    assert!(models.iter().all(|spec| spec.provider_id == "opencode-go"));
    assert_eq!(
        models
            .iter()
            .find(|spec| spec.model == "opencode-go/deepseek-v4-flash")
            .and_then(|spec| spec.reasoning_effort),
        Some(ReasoningEffort::XHigh)
    );
}

#[test]
fn onboarding_provider_keys_then_live_notes_turns_for_ocg_mm() {
    if env_key("KAY_ONBOARDING_LIVE_SMOKE").as_deref() != Some("1") {
        eprintln!("skipping onboarding provider live smoke: set KAY_ONBOARDING_LIVE_SMOKE=1");
        return;
    }
    let Some(keys) = live_keys() else {
        eprintln!(
            "skipping onboarding provider live smoke: set API keys for the selected model filter (OPENCODE_GO_LIVE_API_KEY and/or MINIMAX_LIVE_API_KEY)"
        );
        return;
    };

    let kay_home = TempDir::new().expect("temp KAY_HOME");
    let _sessions = SessionPreserver::new(kay_home.path(), "onboarding_provider_notes_app_live_smoke");
    let (_workspace_guard, repo_dir) = clone_notes_app();

    let mut harness = start_onboarding_provider_setup(kay_home.path(), &repo_dir, &keys);
    assert_onboarding_saved_credentials(kay_home.path(), &keys);

    for (idx, spec) in selected_live_models().into_iter().enumerate() {
        run_tui_model_switch_and_live_notes_turn(&mut harness, kay_home.path(), &keys, spec, idx);
        assert_no_notes_app_diff(&repo_dir);
    }
}
