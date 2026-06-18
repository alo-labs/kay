#![allow(dead_code)]

use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::RecvTimeoutError;
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use portable_pty::native_pty_system;
use portable_pty::CommandBuilder;
use portable_pty::PtySize;
use serde_json::Value;

pub const TUI_ROWS: u16 = 60;
pub const TUI_COLS: u16 = 180;
pub const DEFAULT_PROMPT_TIMEOUT: Duration = Duration::from_secs(45);
pub const DEFAULT_TURN_TIMEOUT: Duration = Duration::from_secs(900);

pub fn code_bin() -> &'static str {
    env!("CARGO_BIN_EXE_code")
}

pub fn env_key(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

pub fn turn_timeout() -> Duration {
    env_key("KAY_TUI_UX_LIVE_SMOKE_TURN_TIMEOUT_SECS")
        .or_else(|| env_key("KAY_TUI_PROVIDER_LIVE_SMOKE_TURN_TIMEOUT_SECS"))
        .or_else(|| env_key("KAY_ONBOARDING_LIVE_SMOKE_TURN_TIMEOUT_SECS"))
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(DEFAULT_TURN_TIMEOUT)
}

pub fn read_provider_key_from_auth(provider: &str) -> Option<String> {
    let kay_home = std::env::var_os("KAY_HOME")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".kay"))
        })?;
    let auth_path = kay_home.join("auth.json");
    let raw = fs::read_to_string(auth_path).ok()?;
    let auth: Value = serde_json::from_str(&raw).ok()?;
    auth.pointer(&format!("/provider_credentials/{provider}/api_key"))
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(str::to_string)
}

pub fn provider_api_key(env_names: &[&str], provider: &str) -> Option<String> {
    for name in env_names {
        if let Some(key) = env_key(name) {
            return Some(key);
        }
    }
    read_provider_key_from_auth(provider)
}

pub fn screen_from_output(output: &[u8]) -> String {
    let mut parser = vt100::Parser::new(TUI_ROWS, TUI_COLS, 0);
    parser.process(output);
    parser.screen().contents()
}

pub fn redact_secrets(text: &str, secrets: &[String]) -> String {
    let mut redacted = text.to_string();
    for secret in secrets.iter().filter(|secret| !secret.is_empty()) {
        redacted = redacted.replace(secret, "[REDACTED_SECRET]");
    }
    redacted
}

pub fn wait_for_screen(
    rx: &mpsc::Receiver<Vec<u8>>,
    output: &mut Vec<u8>,
    secrets: &[String],
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
            redact_secrets(&screen, secrets)
        );

        match rx.recv_timeout(remaining.min(Duration::from_millis(100))) {
            Ok(chunk) => output.extend_from_slice(&chunk),
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => {
                panic!(
                    "Kay TUI exited while waiting for {:?}; screen:\n{}",
                    expected,
                    redact_secrets(&screen, secrets)
                );
            }
        }
    }
}

pub fn write_line(writer: &mut dyn Write, value: &str) {
    writer.write_all(value.as_bytes()).expect("write pty input");
    writer.flush().expect("flush pty input");
    thread::sleep(Duration::from_millis(50));
    writer.write_all(b"\r").expect("write enter");
    writer.flush().expect("flush pty input");
}

pub fn write_key(writer: &mut dyn Write, value: &[u8]) {
    writer.write_all(value).expect("write pty key");
    writer.flush().expect("flush pty key");
}

#[derive(Clone, Default)]
pub struct TuiLiveSpawnOptions {
    pub kay_home: PathBuf,
    pub cwd: PathBuf,
    pub extra_env: HashMap<String, String>,
    pub session_log_path: Option<PathBuf>,
    pub config_toml: Option<String>,
}

pub struct TuiLiveHarness {
    child: Option<Box<dyn portable_pty::Child + Send>>,
    writer: Option<Box<dyn Write + Send>>,
    rx: mpsc::Receiver<Vec<u8>>,
    reader_handle: Option<JoinHandle<()>>,
    output: Vec<u8>,
    secrets: Vec<String>,
}

impl TuiLiveHarness {
    pub fn spawn(options: &TuiLiveSpawnOptions) -> Self {
        if let Some(config_toml) = options.config_toml.as_deref() {
            fs::write(options.kay_home.join("config.toml"), config_toml)
                .expect("write Kay config for TUI live harness");
        }

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
        command.arg(&options.cwd);
        command.env("KAY_HOME", &options.kay_home);
        command.env("TERM", "xterm-256color");
        command.env("NO_COLOR", "1");
        command.env("CODE_SKIP_TUI_TERMINAL_QUERIES", "1");
        command.env("CODE_DISABLE_THEME_AUTODETECT", "1");
        command.env("CODE_DISABLE_FOCUS", "1");
        command.env("CODE_DISABLE_KBD_ENHANCEMENT", "1");
        command.env("CODEX_TUI_FAKE_HOUR", "12");
        command.env("CODEX_TUI_RECORD_SESSION", "1");
        let session_log_path = options
            .session_log_path
            .clone()
            .unwrap_or_else(|| options.kay_home.join("tui-live-session.jsonl"));
        command.env("CODEX_TUI_SESSION_LOG_PATH", &session_log_path);
        for (key, value) in &options.extra_env {
            command.env(key, value);
        }

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

        Self {
            child: Some(child),
            writer: Some(writer),
            rx,
            reader_handle: Some(reader_handle),
            output: Vec::new(),
            secrets: Vec::new(),
        }
    }

    pub fn track_secret(&mut self, secret: impl Into<String>) {
        let secret = secret.into();
        if !secret.is_empty() {
            self.secrets.push(secret);
        }
    }

    pub fn wait_for(&mut self, expected: &[&str], timeout: Duration) -> String {
        wait_for_screen(
            &self.rx,
            &mut self.output,
            &self.secrets,
            expected,
            timeout,
        )
    }

    pub fn write_line(&mut self, value: &str) {
        write_line(
            self.writer.as_mut().expect("pty writer").as_mut(),
            value,
        );
    }

    pub fn write_composer_line(&mut self, value: &str) {
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

    pub fn write_key(&mut self, value: &[u8]) {
        write_key(
            self.writer.as_mut().expect("pty writer").as_mut(),
            value,
        );
    }

    pub fn current_screen(&self) -> String {
        screen_from_output(&self.output)
    }

    pub fn session_log_path(kay_home: &Path) -> PathBuf {
        kay_home.join("tui-live-session.jsonl")
    }

    pub fn drain_output_until<F>(&mut self, timeout: Duration, mut predicate: F) -> String
    where
        F: FnMut(&str) -> bool,
    {
        let deadline = Instant::now() + timeout;
        loop {
            let screen = self.current_screen();
            if predicate(&screen) {
                return screen;
            }

            let remaining = deadline.checked_duration_since(Instant::now()).unwrap_or_default();
            assert!(
                remaining > Duration::ZERO,
                "timed out waiting for screen predicate; screen:\n{}",
                redact_secrets(&screen, &self.secrets)
            );

            match self.rx.recv_timeout(remaining.min(Duration::from_millis(200))) {
                Ok(chunk) => self.output.extend_from_slice(&chunk),
                Err(RecvTimeoutError::Timeout) => continue,
                Err(RecvTimeoutError::Disconnected) => {
                    panic!(
                        "Kay TUI exited while waiting for screen predicate; screen:\n{}",
                        redact_secrets(&screen, &self.secrets)
                    );
                }
            }
        }
    }

    pub fn shutdown(&mut self) {
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

impl Drop for TuiLiveHarness {
    fn drop(&mut self) {
        self.shutdown();
    }
}

pub fn default_live_config_toml(provider_id: &str, model: &str) -> String {
    format!(
        "model_provider = \"{provider_id}\"\n\
model = \"{model}\"\n\
approval_policy = \"never\"\n\
sandbox_mode = \"danger-full-access\"\n\
\n\
[notice]\n\
hide_gpt5_1_migration_prompt = true\n\
hide_gpt-5.1-codex-max_migration_prompt = true\n\
hide_gpt5_2_migration_prompt = true\n\
hide_gpt5_2_codex_migration_prompt = true\n\
\n\
[tools]\n\
browser = false\n\
view_image = false\n\
\n\
[subagents]\n\
enabled = false\n"
    )
}

pub fn extract_model_header_line(screen: &str) -> String {
    screen
        .lines()
        .find(|line| line.contains("Model:"))
        .unwrap_or("")
        .trim()
        .to_string()
}

pub fn header_shows_exact_model(header_line: &str, model: &str) -> bool {
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

pub fn wait_for_model_header(
    harness: &mut TuiLiveHarness,
    model: &str,
    timeout: Duration,
) -> String {
    let deadline = Instant::now() + timeout;
    loop {
        let screen = harness.current_screen();
        let header_line = extract_model_header_line(&screen);
        if header_shows_exact_model(&header_line, model) {
            return header_line;
        }

        let remaining = deadline.checked_duration_since(Instant::now()).unwrap_or_default();
        assert!(
            remaining > Duration::ZERO,
            "TUI header did not show selected model {model}; header: {header_line}\nscreen:\n{}",
            redact_secrets(&screen, &harness.secrets)
        );

        match harness.rx.recv_timeout(remaining.min(Duration::from_millis(100))) {
            Ok(chunk) => harness.output.extend_from_slice(&chunk),
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => {
                panic!(
                    "Kay TUI exited while waiting for selected model {model}; header: {header_line}\nscreen:\n{}",
                    redact_secrets(&screen, &harness.secrets)
                );
            }
        }
    }
}

pub fn session_configure_count(kay_home: &Path) -> usize {
    let path = TuiLiveHarness::session_log_path(kay_home);
    let Ok(raw) = fs::read_to_string(path) else {
        return 0;
    };
    raw.lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|value| value.get("kind").and_then(Value::as_str) == Some("configure_session_model"))
        .count()
}

pub fn config_record_matches(record: &Value, provider_id: &str, model: &str) -> bool {
    record.get("model").and_then(Value::as_str) == Some(model)
        && record.get("model_provider_id").and_then(Value::as_str) == Some(provider_id)
}

pub fn assert_session_start_metadata(
    kay_home: &Path,
    provider_id: &str,
    model: &str,
) {
    let path = TuiLiveHarness::session_log_path(kay_home);
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("read session log {}: {err}", path.display()));
    let records = raw
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|value| value.get("kind").and_then(Value::as_str) == Some("session_start"))
        .collect::<Vec<_>>();

    assert!(
        records.iter().any(|record| {
            record.get("model").and_then(Value::as_str) == Some(model)
                && record.get("model_provider_id").and_then(Value::as_str) == Some(provider_id)
        }),
        "no session_start record matched provider `{provider_id}` with model `{model}`; records:\n{}",
        records
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
}

pub fn wait_for_screen_contains(
    harness: &mut TuiLiveHarness,
    needle: &str,
    timeout: Duration,
) -> String {
    harness.drain_output_until(timeout, |screen| screen.contains(needle))
}

pub fn screen_shows_streaming_content(screen: &str) -> bool {
    screen.contains("Reasoning")
        || screen.contains("reasoning")
        || screen.lines().any(|line| {
            let trimmed = line.trim();
            trimmed.starts_with('▌') || trimmed.contains("│ OK") || trimmed.contains("│OK")
        })
}

pub fn assert_session_configured_model(
    kay_home: &Path,
    before_count: usize,
    provider_id: &str,
    model: &str,
) {
    let path = TuiLiveHarness::session_log_path(kay_home);
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
        .any(|record| config_record_matches(record, provider_id, model));
    let matching_existing = records
        .iter()
        .any(|record| config_record_matches(record, provider_id, model));

    assert!(
        matching_new || matching_existing,
        "no session log record proved TUI configured provider `{provider_id}` with model `{model}`; configure records:\n{}",
        records
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
}

pub fn screen_has_standalone_ok(screen: &str) -> bool {
    screen.lines().any(|line| {
        if line.contains("Reply with exactly") {
            return false;
        }
        let trimmed = line
            .trim()
            .trim_start_matches(['│', '▌', ' ', '•', '★'])
            .trim_end_matches(['│', ' ']);
        trimmed.eq_ignore_ascii_case("ok")
            || trimmed.eq_ignore_ascii_case("ok.")
            || trimmed == "OK"
    })
}

pub fn screen_has_exact_line(screen: &str, expected: &str) -> bool {
    screen.lines().any(|line| {
        line.trim()
            .trim_start_matches(['│', '▌', ' ', '•'])
            .trim_end_matches(['│', ' '])
            .eq_ignore_ascii_case(expected)
            || line.contains(expected)
    })
}

pub fn wait_for_exact_response(
    harness: &mut TuiLiveHarness,
    model: &str,
    expected: &str,
    timeout: Duration,
) -> String {
    wait_for_exact_response_with_failures(
        harness,
        model,
        expected,
        timeout,
        &[
            "Kay runtime error:",
            "Authentication expired.",
            "You exceeded your current quota",
            "Unknown model preset:",
        ],
    )
}

pub fn wait_for_exact_response_after_recovery(
    harness: &mut TuiLiveHarness,
    model: &str,
    expected: &str,
    timeout: Duration,
) -> String {
    wait_for_exact_response_with_failures(
        harness,
        model,
        expected,
        timeout,
        &[
            "Kay runtime error:",
            "Authentication expired.",
            "You exceeded your current quota",
        ],
    )
}

fn wait_for_exact_response_with_failures(
    harness: &mut TuiLiveHarness,
    model: &str,
    expected: &str,
    timeout: Duration,
    failures: &[&str],
) -> String {
    let secrets = harness.secrets.clone();
    harness.drain_output_until(timeout, |screen| {
        if failures.iter().any(|failure| screen.contains(failure)) {
            panic!(
                "live TUI turn failed for {model} after matching a failure marker; screen:\n{}",
                redact_secrets(screen, &secrets)
            );
        }
        screen_has_standalone_ok(screen) || screen_has_exact_line(screen, expected)
    })
}
