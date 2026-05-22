#![cfg(test)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use code_tui::test_backend::VT100Backend;
use code_tui::transcript_viewer::{TranscriptEntry, TranscriptViewer};
use ratatui::Terminal;

fn sample_entries() -> Vec<TranscriptEntry> {
    vec![
        TranscriptEntry {
            timestamp: Some("2026-05-13T10:00:00Z".to_string()),
            direction: Some("meta".to_string()),
            kind: "session_start".to_string(),
            summary: "session started in /Users/shafqat/projects/test-notes-app with opencode-go/kimi-k2.6 via OpenCode Go".to_string(),
            raw: serde_json::json!({
                "ts": "2026-05-13T10:00:00Z",
                "dir": "meta",
                "kind": "session_start",
                "cwd": "/Users/shafqat/projects/test-notes-app",
                "model": "opencode-go/kimi-k2.6",
                "model_provider_name": "OpenCode Go",
            }),
        },
        TranscriptEntry {
            timestamp: Some("2026-05-13T10:01:00Z".to_string()),
            direction: Some("to_tui".to_string()),
            kind: "key_event".to_string(),
            summary: "Char('n')".to_string(),
            raw: serde_json::json!({
                "ts": "2026-05-13T10:01:00Z",
                "dir": "to_tui",
                "kind": "key_event",
                "event": "Char('n')",
            }),
        },
        TranscriptEntry {
            timestamp: Some("2026-05-13T10:01:30Z".to_string()),
            direction: Some("from_tui".to_string()),
            kind: "assistant_message".to_string(),
            summary: "Create a notes sidebar and keyboard shortcuts".to_string(),
            raw: serde_json::json!({
                "ts": "2026-05-13T10:01:30Z",
                "dir": "from_tui",
                "kind": "assistant_message",
                "message": "Create a notes sidebar and keyboard shortcuts",
            }),
        },
        TranscriptEntry {
            timestamp: Some("2026-05-13T10:02:00Z".to_string()),
            direction: Some("to_tui".to_string()),
            kind: "slash_command".to_string(),
            summary: "transcript".to_string(),
            raw: serde_json::json!({
                "ts": "2026-05-13T10:02:00Z",
                "dir": "to_tui",
                "kind": "slash_command",
                "command": "transcript",
            }),
        },
    ]
}

fn render_viewer() -> String {
    let viewer = TranscriptViewer::new(
        sample_entries(),
        PathBuf::from("/Users/shafqat/projects/test-notes-app/transcripts/session-20260513.jsonl"),
    );
    let backend = VT100Backend::new(112, 30);
    let mut terminal = Terminal::new(backend).expect("terminal");

    terminal
        .draw(|frame| viewer.render(frame))
        .expect("render transcript viewer");

    terminal.backend().to_string()
}

#[test]
fn transcript_viewer_renders_readable_chronology_and_provenance() {
    let output = render_viewer();

    assert!(
        output.contains("Kay transcript viewer"),
        "viewer title missing from output:\n{output}"
    );
    assert!(
        output.contains("Timeline"),
        "timeline list missing from output:\n{output}"
    );
    assert!(
        output.contains("session started in /Users/shafqat/projects/test-notes-app"),
        "session provenance missing from output:\n{output}"
    );
    assert!(
        output.contains("Create a notes sidebar and keyboard shortcuts"),
        "chronological message summary missing from output:\n{output}"
    );
    assert!(
        output.contains("raw JSON:"),
        "raw provenance section missing from output:\n{output}"
    );

    insta::assert_snapshot!("transcript_viewer_readable_chronology_and_provenance", output);
}
