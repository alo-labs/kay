#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::mpsc::RecvTimeoutError;
use std::thread;
use std::time::{Duration, Instant};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
use tempfile::TempDir;

fn code_bin() -> &'static str {
    env!("CARGO_BIN_EXE_code")
}

fn make_transcript_fixture(root: &Path) -> PathBuf {
    let path = root.join("session-20260513T100000Z.jsonl");
    fs::write(
        &path,
        concat!(
            r#"{"ts":"2026-05-13T10:00:00Z","dir":"meta","kind":"session_start","cwd":"/Users/shafqat/projects/test-notes-app","model":"opencode-go/kimi-k2.6","model_provider_name":"OpenCode Go"}"#,
            "\n",
            r#"{"ts":"2026-05-13T10:01:00Z","dir":"to_tui","kind":"key_event","event":"Char('n')"}"#,
            "\n",
            r#"{"ts":"2026-05-13T10:01:30Z","dir":"from_tui","kind":"assistant_message","message":"Create a notes sidebar and keyboard shortcuts"}"#,
            "\n",
            r#"{"ts":"2026-05-13T10:02:00Z","dir":"to_tui","kind":"slash_command","command":"transcript"}"#,
            "\n",
        ),
    )
    .expect("write transcript fixture");
    path
}

fn spawn_transcript_viewer(
    code_home: &TempDir,
    transcript_path: &Path,
) -> (
    Box<dyn portable_pty::Child + Send>,
    Box<dyn std::io::Read + Send>,
    Box<dyn std::io::Write + Send>,
) {
    let pty_system = native_pty_system();
    let pair = pty_system
        .openpty(PtySize {
            rows: 32,
            cols: 120,
            pixel_width: 0,
            pixel_height: 0,
        })
        .expect("open pty");

    let mut command = CommandBuilder::new(code_bin());
    command.arg("transcript");
    command.arg(transcript_path);
    command.env("CODE_HOME", code_home.path());
    command.env("TERM", "xterm-256color");
    command.env("NO_COLOR", "1");
    command.env("CODE_SKIP_TUI_TERMINAL_QUERIES", "1");
    command.env("CODE_DISABLE_THEME_AUTODETECT", "1");
    command.env("CODE_DISABLE_FOCUS", "1");
    command.env("CODE_DISABLE_KBD_ENHANCEMENT", "1");

    let child = pair.slave.spawn_command(command).expect("spawn transcript viewer");
    let reader = pair.master.try_clone_reader().expect("clone reader");
    let writer = pair.master.take_writer().expect("take writer");
    (child, reader, writer)
}

#[test]
fn transcript_viewer_cli_opens_a_real_jsonl_transcript() {
    let transcript_dir = TempDir::new().expect("transcript dir");
    let code_home = TempDir::new().expect("code home");
    let transcript_path = make_transcript_fixture(transcript_dir.path());
    let baseline = fs::read_to_string(&transcript_path).expect("read transcript baseline");

    let (mut child, mut reader, mut writer) =
        spawn_transcript_viewer(&code_home, &transcript_path);

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

    let mut output = Vec::new();
    let screen_ready_deadline = Instant::now() + Duration::from_secs(10);
    while {
        let text = String::from_utf8_lossy(&output);
        !(text.contains("Kay transcript viewer")
            && text.contains("Timeline")
            && text.contains("raw JSON:"))
    } {
        let remaining = screen_ready_deadline
            .checked_duration_since(Instant::now())
            .unwrap_or_default();
        assert!(
            remaining > Duration::ZERO,
            "timed out waiting for transcript viewer screen; output so far:\n{}",
            String::from_utf8_lossy(&output)
        );
        match rx.recv_timeout(remaining.min(Duration::from_millis(200))) {
            Ok(chunk) => output.extend_from_slice(&chunk),
            Err(RecvTimeoutError::Timeout) => continue,
            Err(RecvTimeoutError::Disconnected) => {
                panic!(
                    "transcript viewer exited before rendering the initial screen; output so far:\n{}",
                    String::from_utf8_lossy(&output)
                );
            }
        }
    }

    let screen_bytes = output.clone();
    writer.write_all(b"q\x1b").expect("write quit key");
    writer.flush().expect("flush quit key");
    drop(writer);

    let exit_deadline = Instant::now() + Duration::from_secs(10);
    loop {
        if let Some(status) = child.try_wait().expect("poll transcript viewer exit") {
            assert_eq!(status.exit_code(), 0, "viewer should exit cleanly");
            break;
        }
        assert!(
            Instant::now() < exit_deadline,
            "transcript viewer did not exit after quit key; output so far:\n{}",
            String::from_utf8_lossy(&output)
        );
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(chunk) => output.extend_from_slice(&chunk),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    while let Ok(chunk) = rx.recv_timeout(Duration::from_millis(50)) {
        output.extend_from_slice(&chunk);
    }
    reader_handle.join().expect("join pty reader");

    let mut parser = vt100::Parser::new(32, 120, 0);
    parser.process(&screen_bytes);
    let screen = parser.screen().contents();
    assert!(
        screen.contains("Kay transcript viewer"),
        "viewer title missing from output:\n{screen}"
    );
    assert!(
        screen.contains("Timeline"),
        "timeline missing from output:\n{screen}"
    );
    assert!(
        screen.contains("session started in /Users/shafqat/projects/test-notes-app"),
        "session provenance missing from output:\n{screen}"
    );
    assert!(
        screen.contains("Create a notes sidebar")
            && screen.contains("keyboard shortcuts"),
        "message summary missing from output:\n{screen}"
    );

    let after = fs::read_to_string(&transcript_path).expect("re-read transcript");
    assert_eq!(after, baseline, "transcript viewer must not modify the JSONL");
}
