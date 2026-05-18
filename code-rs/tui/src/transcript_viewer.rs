use std::cmp::min;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::BufReader;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

use crossterm::event;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use crossterm::event::KeyEvent;
use crossterm::event::KeyModifiers;
use ratatui::layout::Constraint;
use ratatui::layout::Direction;
use ratatui::layout::Layout;
use ratatui::buffer::Buffer;
use ratatui::prelude::Color;
use ratatui::style::Modifier;
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::text::Text;
use ratatui::widgets::Block;
use ratatui::widgets::Borders;
use ratatui::widgets::List;
use ratatui::widgets::ListItem;
use ratatui::widgets::ListState;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;
use ratatui::widgets::StatefulWidget;
use ratatui::widgets::Widget;
use ratatui::widgets::WidgetRef;
use serde_json::Map;
use serde_json::Value;

use crate::colors;
use crate::util::buffer::fill_rect;
use code_core::config::Config;
use code_core::config::ConfigOverrides;

const DEFAULT_LATEST_GLOB_PREFIX: &str = "session-";
const DEFAULT_LATEST_GLOB_SUFFIX: &str = ".jsonl";

#[derive(Debug, Clone, Default)]
pub struct TranscriptViewerArgs {
    pub path: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct TranscriptEntry {
    pub timestamp: Option<String>,
    pub direction: Option<String>,
    pub kind: String,
    pub summary: String,
    pub raw: Value,
}

impl TranscriptEntry {
    fn from_value(value: Value, fallback_line: Option<String>) -> Self {
        match value {
            Value::Object(map) => Self::from_object(map),
            other => {
                let summary = fallback_line.unwrap_or_else(|| compact_value_summary(&other));
                Self {
                    timestamp: None,
                    direction: None,
                    kind: "value".to_string(),
                    summary,
                    raw: other,
                }
            }
        }
    }

    fn from_object(map: Map<String, Value>) -> Self {
        let timestamp = string_field(&map, &["ts", "timestamp", "created_at"]);
        let direction = string_field(&map, &["dir", "direction"]);
        let kind = string_field(&map, &["kind", "type"]).unwrap_or_else(|| "record".to_string());
        let summary = summarize_object(&map, &kind);
        let raw = Value::Object(map);

        Self {
            timestamp,
            direction,
            kind,
            summary,
            raw,
        }
    }
}

pub async fn run_main(args: TranscriptViewerArgs, _code_linux_sandbox_exe: Option<PathBuf>) -> io::Result<()> {
    let config = prepare_viewer_config()?;
    let transcript_path = resolve_transcript_path(args, &config)?;
    let entries = load_transcript_entries(&transcript_path)?;

    crate::install_unified_panic_hook();
    let (mut terminal, _terminal_info) = crate::tui::init(&config)?;

    let mut viewer = TranscriptViewer::new(entries, transcript_path);
    let result = viewer.run(&mut terminal);

    let _ = crate::tui::restore();
    result
}

fn prepare_viewer_config() -> io::Result<Config> {
    let mut config = Config::load_with_cli_overrides(Vec::new(), ConfigOverrides::default())?;
    crate::maybe_apply_terminal_theme_detection(&mut config, false);
    Ok(config)
}

fn resolve_transcript_path(
    args: TranscriptViewerArgs,
    config: &Config,
) -> io::Result<PathBuf> {
    if let Some(path) = args.path {
        return Ok(resolve_relative_path(path)?);
    }

    latest_session_log_path(config)
}

fn resolve_relative_path(path: PathBuf) -> io::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path)
    } else {
        Ok(std::env::current_dir()?.join(path))
    }
}

fn latest_session_log_path(config: &Config) -> io::Result<PathBuf> {
    let log_dir = code_core::config::log_dir(config)?;
    let mut newest: Option<PathBuf> = None;
    let mut newest_name: Option<String> = None;

    for entry in std::fs::read_dir(&log_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|s| s.to_str()).map(str::to_string) else {
            continue;
        };
        if !name.starts_with(DEFAULT_LATEST_GLOB_PREFIX) || !name.ends_with(DEFAULT_LATEST_GLOB_SUFFIX) {
            continue;
        }
        let should_replace = newest_name
            .as_ref()
            .map(|current| name > *current)
            .unwrap_or(true);
        if should_replace {
            newest = Some(path);
            newest_name = Some(name);
        }
    }

    newest.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("no transcript JSONL files found in {}", log_dir.display()),
        )
    })
}

pub fn load_transcript_entries(path: &Path) -> io::Result<Vec<TranscriptEntry>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let mut entries = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        match serde_json::from_str::<Value>(trimmed) {
            Ok(value) => entries.push(TranscriptEntry::from_value(value, None)),
            Err(err) => entries.push(TranscriptEntry {
                timestamp: None,
                direction: None,
                kind: "parse_error".to_string(),
                summary: format!("failed to parse JSONL line: {err}"),
                raw: serde_json::json!({
                    "kind": "parse_error",
                    "error": err.to_string(),
                    "line": trimmed,
                }),
            }),
        }
    }

    Ok(entries)
}

fn string_field(map: &Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| map.get(*key).and_then(value_to_string))
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        Value::Null => None,
        other => Some(compact_value_summary(other)),
    }
}

fn summarize_object(map: &Map<String, Value>, kind: &str) -> String {
    if kind == "session_start" {
        let cwd = string_field(map, &["cwd"]).unwrap_or_else(|| "<unknown cwd>".to_string());
        let model = string_field(map, &["model"]).unwrap_or_else(|| "<unknown model>".to_string());
        let provider = string_field(map, &["model_provider_name", "model_provider_id"])
            .unwrap_or_else(|| "<unknown provider>".to_string());
        return format!("session started in {cwd} with {model} via {provider}");
    }

    for key in [
        "summary",
        "status_title",
        "status",
        "goal",
        "prompt",
        "text",
        "message",
        "user_response",
        "cli_command",
        "command",
        "event",
        "title",
    ] {
        if let Some(summary) = string_field(map, &[key]) {
            return summary;
        }
    }

    if let Some(Value::Array(items)) = map.get("agents") {
        return format!("{} agent(s) attached", items.len());
    }

    if let Some(Value::Array(items)) = map.get("transcript") {
        return format!("embedded transcript with {} item(s)", items.len());
    }

    if let Some(Value::Array(items)) = map.get("items") {
        return format!("{} item(s)", items.len());
    }

    compact_value_summary(&Value::Object(map.clone()))
}

fn compact_value_summary(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut parts = Vec::new();
            for key in ["kind", "dir", "ts", "status", "goal"] {
                if let Some(value) = map.get(key).and_then(value_to_string) {
                    parts.push(format!("{key}={value}"));
                }
            }
            if parts.is_empty() {
                serde_json::to_string_pretty(value).unwrap_or_else(|_| "<unrenderable JSON>".to_string())
            } else {
                parts.join(" ")
            }
        }
        Value::Array(items) => format!("{} item(s)", items.len()),
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Null => "<null>".to_string(),
    }
}

pub struct TranscriptViewer {
    entries: Vec<TranscriptEntry>,
    selected: usize,
    source_path: PathBuf,
}

impl TranscriptViewer {
    pub fn new(entries: Vec<TranscriptEntry>, source_path: PathBuf) -> Self {
        Self {
            entries,
            selected: 0,
            source_path,
        }
    }

    pub fn from_path(path: PathBuf) -> io::Result<Self> {
        let entries = load_transcript_entries(&path)?;
        Ok(Self::new(entries, path))
    }

    pub fn handle_key(&mut self, key: KeyEvent, visible_rows: usize) -> bool {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return true,
            KeyCode::Down => self.move_selection(1),
            KeyCode::Up => self.move_selection(-1),
            KeyCode::PageDown => self.move_selection(visible_rows as isize),
            KeyCode::PageUp => self.move_selection(-(visible_rows as isize)),
            KeyCode::Home => {
                self.selected = 0;
            }
            KeyCode::End => {
                self.selected = self.entries.len().saturating_sub(1);
            }
            KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn render(&self, frame: &mut ratatui::Frame<'_>) {
        let area = frame.area();
        frame.render_widget_ref(self, area);
    }

    fn render_header(&self, buf: &mut Buffer, area: ratatui::layout::Rect) {
        let bg_style = Style::default().bg(colors::background());
        fill_rect(buf, area, Some(' '), bg_style);
        let title = Line::from(vec![
            Span::styled("Kay transcript viewer", Style::default().add_modifier(Modifier::BOLD)),
            Span::raw("  "),
            Span::styled(
                format!("{} records", self.entries.len()),
                Style::default().fg(Color::Cyan),
            ),
        ]);
        let subtitle = Line::from(vec![
            Span::raw("source: "),
            Span::styled(
                self.source_path.display().to_string(),
                Style::default().fg(Color::Yellow),
            ),
        ]);
        let help = Line::from(vec![
            Span::raw("read-only transcript review • "),
            Span::styled("↑↓ select", Style::default().fg(Color::Green)),
            Span::raw(" • "),
            Span::styled("PgUp/PgDn page", Style::default().fg(Color::Green)),
            Span::raw(" • "),
            Span::styled("Home/End", Style::default().fg(Color::Green)),
            Span::raw(" • "),
            Span::styled("q/Esc quit", Style::default().fg(Color::Green)),
        ]);

        let block = Block::default().style(bg_style).title(title).borders(Borders::ALL);
        let text = Text::from(vec![subtitle, help]);
        Paragraph::new(text)
            .style(bg_style)
            .block(block)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }

    fn render_body(&self, buf: &mut Buffer, area: ratatui::layout::Rect) {
        let split = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area);

        self.render_entry_list(buf, split[0]);
        self.render_selected_details(buf, split[1]);
    }

    fn render_entry_list(&self, buf: &mut Buffer, area: ratatui::layout::Rect) {
        let bg_style = Style::default().bg(colors::background());
        fill_rect(buf, area, Some(' '), bg_style);
        let block = Block::default().style(bg_style).title("Timeline").borders(Borders::ALL);
        if self.entries.is_empty() {
            let paragraph = Paragraph::new("No transcript entries found.")
                .style(bg_style)
                .block(block)
                .wrap(Wrap { trim: true });
            paragraph.render(area, buf);
            return;
        }

        let visible_rows = area.height.saturating_sub(2).max(1) as usize;
        let page_size = visible_rows.max(1) / 2 + 1;
        let start = self.visible_start(page_size.max(1));
        let end = min(self.entries.len(), start + page_size.max(1));
        let mut state = ListState::default();
        state.select(Some(self.selected.saturating_sub(start)));

        let items: Vec<ListItem> = self.entries[start..end]
            .iter()
            .map(|entry| {
                let header = render_entry_header(entry);
                let summary = render_entry_summary(entry);
                let lines = vec![
                    Line::from(vec![Span::styled(
                        header,
                        Style::default().fg(direction_color(entry.direction.as_deref())),
                    )]),
                    Line::from(summary),
                ];
                ListItem::new(Text::from(lines))
            })
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            )
            .highlight_symbol("❯ ");
        StatefulWidget::render(list, area, buf, &mut state);
    }

    fn render_selected_details(&self, buf: &mut Buffer, area: ratatui::layout::Rect) {
        let bg_style = Style::default().bg(colors::background());
        fill_rect(buf, area, Some(' '), bg_style);
        let block = Block::default().style(bg_style).title("Selected record").borders(Borders::ALL);
        if let Some(entry) = self.entries.get(self.selected) {
            let mut detail_lines = Vec::new();
            if let Some(ts) = &entry.timestamp {
                detail_lines.push(Line::from(vec![
                    Span::styled("ts: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(ts.clone()),
                ]));
            }
            if let Some(dir) = &entry.direction {
                detail_lines.push(Line::from(vec![
                    Span::styled("dir: ", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(dir.clone()),
                ]));
            }
            detail_lines.push(Line::from(vec![
                Span::styled("kind: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(entry.kind.clone()),
            ]));
            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(vec![
                Span::styled("summary: ", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(entry.summary.clone()),
            ]));
            detail_lines.push(Line::from(""));
            detail_lines.push(Line::from(vec![
                Span::styled("raw JSON:", Style::default().add_modifier(Modifier::BOLD)),
            ]));
            let raw = serde_json::to_string_pretty(&entry.raw).unwrap_or_else(|_| "<unrenderable JSON>".to_string());
            for line in raw.lines() {
                detail_lines.push(Line::from(line.to_string()));
            }

            let paragraph = Paragraph::new(Text::from(detail_lines))
                .style(bg_style)
                .block(block)
                .wrap(Wrap { trim: false });
            paragraph.render(area, buf);
        } else {
            let paragraph = Paragraph::new("Select an entry to inspect its raw provenance.")
                .style(bg_style)
                .block(block)
                .wrap(Wrap { trim: true });
            paragraph.render(area, buf);
        }
    }

    fn render_footer(&self, buf: &mut Buffer, area: ratatui::layout::Rect) {
        let bg_style = Style::default().bg(colors::background());
        fill_rect(buf, area, Some(' '), bg_style);
        let footer = if let Some(entry) = self.entries.get(self.selected) {
            format!(
                "selected {} / {} • {}",
                self.selected.saturating_add(1),
                self.entries.len().max(1),
                entry.summary
            )
        } else {
            "no transcript entries loaded".to_string()
        };

        let block = Block::default().style(bg_style).title("Status").borders(Borders::ALL);
        Paragraph::new(footer)
            .style(bg_style)
            .block(block)
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }

    fn move_selection(&mut self, delta: isize) {
        if self.entries.is_empty() {
            self.selected = 0;
            return;
        }

        let max = self.entries.len().saturating_sub(1) as isize;
        let current = self.selected as isize;
        let next = (current + delta).clamp(0, max);
        self.selected = next as usize;
    }

    fn visible_start(&self, page_size: usize) -> usize {
        if self.entries.len() <= page_size {
            return 0;
        }

        let mut start = self.selected.saturating_sub(page_size.saturating_sub(1));
        let max_start = self.entries.len().saturating_sub(page_size);
        if start > max_start {
            start = max_start;
        }
        start
    }

    fn run(&mut self, terminal: &mut crate::tui::Tui) -> io::Result<()> {
        loop {
            terminal.draw(|frame| frame.render_widget_ref(&*self, frame.area()))?;

            if event::poll(Duration::from_millis(100))? {
                match event::read()? {
                    Event::Key(key) => {
                        let visible_rows = terminal.size().map(|size| size.height as usize).unwrap_or(24);
                        if self.handle_key(key, visible_rows) {
                            return Ok(());
                        }
                    }
                    Event::Resize(_, _) => {}
                    _ => {}
                }
            }
        }
    }
}

impl WidgetRef for &TranscriptViewer {
    fn render_ref(&self, area: ratatui::layout::Rect, buf: &mut Buffer) {
        let bg_style = Style::default().bg(colors::background());
        fill_rect(buf, area, Some(' '), bg_style);
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(4),
                Constraint::Min(8),
                Constraint::Length(3),
            ])
            .split(area);

        self.render_header(buf, chunks[0]);
        self.render_body(buf, chunks[1]);
        self.render_footer(buf, chunks[2]);
    }
}

fn render_entry_header(entry: &TranscriptEntry) -> String {
    let mut parts = Vec::new();
    if let Some(ts) = &entry.timestamp {
        parts.push(ts.clone());
    }
    if let Some(dir) = &entry.direction {
        parts.push(format!("[{dir}]"));
    }
    parts.push(entry.kind.clone());
    parts.join(" • ")
}

fn render_entry_summary(entry: &TranscriptEntry) -> String {
    if entry.summary.is_empty() {
        "<empty>".to_string()
    } else {
        entry.summary.clone()
    }
}

fn direction_color(direction: Option<&str>) -> Color {
    match direction {
        Some("to_tui") => Color::Green,
        Some("meta") => Color::Magenta,
        Some("from_tui") => Color::Cyan,
        Some(_) => Color::Yellow,
        None => Color::White,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_backend::VT100Backend;
    use code_core::config_types::ThemeName;
    use ratatui::Terminal;
    use std::fs;
    use std::sync::Mutex;
    use tempfile::TempDir;
    use uuid::Uuid;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    struct EnvGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvGuard {
        fn new(key: &'static str) -> Self {
            Self {
                key,
                previous: std::env::var(key).ok(),
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.previous {
                Some(value) => unsafe {
                    std::env::set_var(self.key, value);
                },
                None => unsafe {
                    std::env::remove_var(self.key);
                },
            }
        }
    }

    #[test]
    fn load_transcript_entries_parses_jsonl_and_reports_errors() {
        let path = std::env::temp_dir().join(format!(
            "kay-transcript-viewer-{}.jsonl",
            Uuid::new_v4()
        ));
        fs::write(
            &path,
            concat!(
                r#"{"kind":"session_start","ts":"2026-05-13T10:00:00Z","dir":"meta","cwd":"/tmp/demo","model":"gpt-4.1","model_provider_name":"OpenAI"}"#,
                "\n",
                "not-json",
                "\n",
            ),
        )
        .expect("write transcript fixture");

        let entries = load_transcript_entries(&path).expect("load transcript");
        fs::remove_file(&path).ok();

        assert_eq!(entries.len(), 2);
        assert!(entries[0].summary.contains("session started in /tmp/demo"));
        assert_eq!(entries[0].kind, "session_start");
        assert_eq!(entries[1].kind, "parse_error");
        assert!(entries[1].summary.contains("failed to parse JSONL line"));
    }

    #[test]
    fn transcript_viewer_renders_a_readable_screen() {
        let entries = vec![
            TranscriptEntry {
                timestamp: Some("2026-05-13T10:00:00Z".to_string()),
                direction: Some("meta".to_string()),
                kind: "session_start".to_string(),
                summary: "session started in /tmp/demo with gpt-4.1 via OpenAI".to_string(),
                raw: serde_json::json!({
                    "kind": "session_start",
                    "ts": "2026-05-13T10:00:00Z",
                    "dir": "meta",
                    "cwd": "/tmp/demo",
                    "model": "gpt-4.1",
                    "model_provider_name": "OpenAI",
                }),
            },
            TranscriptEntry {
                timestamp: Some("2026-05-13T10:01:00Z".to_string()),
                direction: Some("to_tui".to_string()),
                kind: "agent_message".to_string(),
                summary: "Ask the model to scaffold the notes app".to_string(),
                raw: serde_json::json!({
                    "kind": "agent_message",
                    "dir": "to_tui",
                    "ts": "2026-05-13T10:01:00Z",
                    "summary": "Ask the model to scaffold the notes app",
                }),
            },
        ];

        let viewer = TranscriptViewer::new(entries, PathBuf::from("/tmp/session-abc.jsonl"));
        let backend = VT100Backend::new(100, 28);
        let mut terminal = Terminal::new(backend).expect("terminal");

        terminal
            .draw(|frame| viewer.render(frame))
            .expect("render viewer");

        let screen = terminal.backend().to_string();
        assert!(screen.contains("Kay transcript viewer"));
        assert!(screen.contains("Timeline"));
        assert!(screen.contains("session started in /tmp/demo"));
        assert!(screen.contains("Ask the model to scaffold the notes app"));
    }

    #[test]
    fn prepare_viewer_config_uses_dark_theme_in_dumb_term() -> io::Result<()> {
        let _lock = ENV_LOCK.lock().unwrap();
        let _term_guard = EnvGuard::new("TERM");
        let _kay_home_guard = EnvGuard::new("KAY_HOME");
        let kay_home = TempDir::new().expect("temp kay home");

        unsafe {
            std::env::set_var("TERM", "dumb");
            std::env::set_var("KAY_HOME", kay_home.path());
        }

        let config = prepare_viewer_config()?;
        assert!(matches!(config.tui.theme.name, ThemeName::DarkCarbonNight));
        Ok(())
    }
}
