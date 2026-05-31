use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;

use serde::Deserialize;
use serde_json::json;
use tempfile::TempDir;

mod common;
use common::SessionPreserver;

const TEST_NOTES_APP_REPO_ROOT: &str = "/Users/shafqat/projects/test-notes-app";
const EXPECTED_NOTES_FILES: &[&str] = &["src/public/index.html", "src/public/notes-ui.js"];

const OPENCODE_GO_MODELS: &[&str] = &[
    "opencode-go/glm-5.1",
    "opencode-go/kimi-k2.6",
    "opencode-go/mimo-v2.5-pro",
    "opencode-go/mimo-v2.5",
    "opencode-go/minimax-m2.7",
    "opencode-go/qwen3.6-plus",
    "opencode-go/deepseek-v4-pro",
    "opencode-go/deepseek-v4-flash",
];
const XIAOMI_MODELS: &[&str] = &["xiaomi/mimo-v2.5-pro", "xiaomi/mimo-v2.5"];

struct ProviderSpec {
    provider_id: &'static str,
    primary_api_key_env: &'static str,
    fallback_api_key_env: Option<&'static str>,
    models: &'static [&'static str],
}

const OPENCODE_GO_SPEC: ProviderSpec = ProviderSpec {
    provider_id: "opencode-go",
    primary_api_key_env: "OPENCODE_GO_LIVE_API_KEY",
    fallback_api_key_env: Some("OPENCODE_GO_API_KEY"),
    models: OPENCODE_GO_MODELS,
};

const XIAOMI_SPEC: ProviderSpec = ProviderSpec {
    provider_id: "xiaomi",
    primary_api_key_env: "XIAOMI_LIVE_API_KEY",
    fallback_api_key_env: Some("XIAOMI_API_KEY"),
    models: XIAOMI_MODELS,
};

fn code_bin() -> &'static str {
    env!("CARGO_BIN_EXE_code")
}

fn live_key(spec: &ProviderSpec) -> Option<String> {
    std::env::var(spec.primary_api_key_env)
        .or_else(|_| {
            spec.fallback_api_key_env
                .map(std::env::var)
                .unwrap_or_else(|| Err(std::env::VarError::NotPresent))
        })
        .ok()
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty())
}

fn repo_root() -> PathBuf {
    std::env::var_os("TEST_NOTES_APP_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(TEST_NOTES_APP_REPO_ROOT))
}

fn selected_models(spec: &ProviderSpec) -> Vec<&'static str> {
    let Some(raw) = std::env::var_os("TEST_NOTES_APP_MODEL_FILTER") else {
        return spec.models.to_vec();
    };

    let requested: Vec<String> = raw
        .to_string_lossy()
        .split(',')
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect();

    if requested.is_empty() {
        return spec.models.to_vec();
    }

    spec.models
        .iter()
        .copied()
        .filter(|model| requested.iter().any(|wanted| wanted == model))
        .collect()
}

fn login_provider(kay_home: &TempDir, spec: &ProviderSpec, api_key: &str) {
    let mut child = Command::new(code_bin())
        .arg("login")
        .arg("--provider")
        .arg(spec.provider_id)
        .arg("--with-api-key")
        .env("KAY_HOME", kay_home.path())
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
        .expect("write provider api key");

    let output = child.wait_with_output().expect("wait for kay login");
    assert!(
        output.status.success(),
        "kay login failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn clone_repo() -> (TempDir, PathBuf) {
    let temp = TempDir::new().expect("create temp workspace");
    let worktree = temp.path().join("test-notes-app");

    let output = Command::new("git")
        .arg("clone")
        .arg("--no-local")
        .arg(repo_root())
        .arg(&worktree)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("clone repo");

    assert!(
        output.status.success(),
        "git clone failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    (temp, worktree)
}

fn run_notes_feature_turn(
    kay_home: &TempDir,
    repo_dir: &Path,
    provider_id: &str,
    model: &str,
    prompt: &str,
    last_message_path: &Path,
) -> NotesWorkResponse {
    let schema = json!({
        "type": "object",
        "additionalProperties": false,
        "required": ["summary", "touched_files"],
        "properties": {
            "summary": { "type": "string" },
            "touched_files": {
                "type": "array",
                "items": { "type": "string" }
            }
        }
    });
    let schema_path = kay_home.path().join("notes-app-output-schema.json");
    fs::write(&schema_path, serde_json::to_string_pretty(&schema).unwrap())
        .expect("write output schema");

    let output = Command::new(code_bin())
        .arg("exec")
        .arg("--json")
        .arg("--max-seconds")
        .arg("1800")
        .arg("--sandbox")
        .arg("workspace-write")
        .arg("--cd")
        .arg(repo_dir)
        .arg("--output-schema")
        .arg(&schema_path)
        .arg("--output-last-message")
        .arg(last_message_path)
        .arg("-c")
        .arg(format!("model_provider={provider_id}"))
        .arg("-c")
        .arg(format!("model={model}"))
        .arg(prompt)
        .env("KAY_HOME", kay_home.path())
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

    let stdout_transcript = String::from_utf8_lossy(&output.stdout).to_string();
    save_stdout_transcript(repo_dir, model, &stdout_transcript);

    let raw_message = fs::read_to_string(last_message_path).expect("read last message file");
    notes_response_from_last_message(&raw_message)
}

fn build_notes_feature_prompt(repo_root: &Path) -> String {
    format!(
        r#"
You are working in {}.

Implement a duplicate-note workflow by editing only `src/public/index.html`
and `src/public/notes-ui.js`.

Inspect those files first, then edit the actual files in this checkout. The app
already has note CRUD, tags, pinning, archiving, and search.
Use these exact current names: `state.selectedId`, `state.notes`,
`currentNote()`, `selectNote(id)`, `saveNote(event)`,
`deleteSelectedNote()`, `renderEditor(note)`, `renderNotes()`,
`editorForm`, `deleteButton`, `newNoteButton`, `noteTitle`, `noteBody`,
`noteTags`, `notePinned`, and `noteArchived`.
Do not invent alternate names like `deleteBtn`, `selectedNoteId`, `pin`, or
`archive`.

Critical execution rule:
- Do not send any normal assistant/progress message until both files are edited
  and `node --check src/public/notes-ui.js` passes.
- Kay treats a normal assistant message as the end of this test turn. Use tool
  calls only for inspection, editing, repair, and validation until the final
  contracted output.
- If a tool call fails, repair it with another tool call; do not narrate the
  failure in an assistant message.

Output contract:
- Return exactly one JSON object and nothing else.
- The JSON object must contain only `summary` and `touched_files`.
- If the edit succeeds, `touched_files` must be exactly
  `["src/public/index.html", "src/public/notes-ui.js"]`.
- Use the `summary` string for a concise human summary of the edits.
- If you cannot complete the task, return a JSON object with an empty
  `touched_files` array and a `summary` that explains the blocker.

Add these behaviors:
- In `index.html`, insert a visible `Duplicate note` button in the editor
  action row before the Delete button.
- In `index.html`, replace the hint copy with:
  `Keyboard: N new note, D duplicate, / search, Ctrl+Enter save, Esc clear.
  Use the list to pick a note, or create a new one with the button above.
  Search and tags update the list in place.`
- In `notes-ui.js`, add `duplicateButton` to `els` and wire it up.
- In `notes-ui.js`, disable the duplicate button when no note is selected.
- Add `duplicateSelectedNote()` so it copies the currently selected note's
  title, body, tags, pinned, and archived fields into a fresh `POST`.
- The duplicated title should be clearly copied, e.g. `Copy of <title>`.
- After duplicating, keep the new note selected, rerender the editor, and
  focus the title field.
- Add a `d` keyboard shortcut for duplicating the selected note when not
  typing.
- Keep the existing shortcuts (`n`, `/`, `Esc`, `Ctrl+Enter`) working.
- Do not use `els.pin` or `els.archive`; the existing field names are
  `notePinned` and `noteArchived`.

Keep the app lightweight, preserve the existing Express + SQLite architecture,
and do not introduce new dependencies.
"#,
        repo_root.display()
    )
}

fn git_diff_names(repo_dir: &Path) -> Vec<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_dir)
        .arg("diff")
        .arg("--name-only")
        .arg("HEAD")
        .output()
        .expect("git diff");

    assert!(
        output.status.success(),
        "git diff failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|line| line.trim().to_string())
        .filter(|line| !line.is_empty())
        .collect()
}

fn is_expected_notes_file(path: &str) -> bool {
    EXPECTED_NOTES_FILES.contains(&path)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct NotesWorkResponse {
    summary: String,
    touched_files: Vec<String>,
}

fn parse_notes_work_response(input: &str) -> Option<NotesWorkResponse> {
    let trimmed = input.trim();
    if let Ok(response) = serde_json::from_str::<NotesWorkResponse>(trimmed) {
        return Some(response);
    }

    let fence_starts = trimmed
        .match_indices("```")
        .map(|(idx, _)| idx)
        .collect::<Vec<_>>();
    for start in fence_starts.into_iter().rev() {
        let after_fence = &trimmed[start + "```".len()..];
        let Some(language_end) = after_fence.find('\n') else {
            continue;
        };
        if after_fence[..language_end].trim().to_ascii_lowercase() != "json" {
            continue;
        }
        let body = &after_fence[language_end + 1..];
        if let Some(end_rel) = body.find("```")
            && let Ok(response) = serde_json::from_str::<NotesWorkResponse>(body[..end_rel].trim())
        {
            return Some(response);
        }
    }

    parse_trailing_notes_work_response(trimmed)
}

fn parse_trailing_notes_work_response(input: &str) -> Option<NotesWorkResponse> {
    for (start, ch) in input.char_indices().rev() {
        if ch != '{' {
            continue;
        }
        let candidate = input[start..].trim();
        if let Ok(response) = serde_json::from_str::<NotesWorkResponse>(candidate) {
            return Some(response);
        }
    }

    None
}

fn notes_response_from_last_message(raw_message: &str) -> NotesWorkResponse {
    if let Some(response) = parse_notes_work_response(raw_message) {
        return response;
    }

    panic!("expected JSON output from kay exec, got:\n{raw_message}");
}

fn latest_session_jsonl(kay_home: &Path) -> Option<PathBuf> {
    fn visit(dir: &Path, newest: &mut Option<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                visit(&path, newest);
                continue;
            }
            let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !name.starts_with("session-") || !name.ends_with(".jsonl") {
                continue;
            }

            let replace = match newest {
                Some(current) => {
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .map(|name| {
                            current
                                .file_name()
                                .and_then(|cur| cur.to_str())
                                .map(|cur| name > cur)
                                .unwrap_or(true)
                        })
                        .unwrap_or(true)
                }
                None => true,
            };

            if replace {
                *newest = Some(path);
            }
        }
    }

    let mut newest = None;
    visit(kay_home, &mut newest);
    newest
}

fn copy_transcript(kay_home: &Path, repo_dir: &Path, model: &str) -> PathBuf {
    let transcripts_dir = repo_dir.join("transcripts");
    fs::create_dir_all(&transcripts_dir).expect("create transcripts dir");
    let safe_model = model.replace('/', "_");
    let dest = transcripts_dir.join(format!("{safe_model}.jsonl"));
    if let Some(transcript) = latest_session_jsonl(kay_home) {
        fs::copy(&transcript, &dest).expect("copy transcript");
    } else if !dest.exists() {
        panic!("no transcript JSONL found under {}", kay_home.display());
    }
    dest
}

fn save_stdout_transcript(repo_dir: &Path, model: &str, stdout: &str) -> PathBuf {
    let transcripts_dir = repo_dir.join("transcripts");
    fs::create_dir_all(&transcripts_dir).expect("create transcripts dir");
    let safe_model = model.replace('/', "_");
    let dest = transcripts_dir.join(format!("{safe_model}.jsonl"));
    fs::write(&dest, stdout).expect("write fallback transcript");
    dest
}

fn assert_notes_feature_change(repo_dir: &Path) {
    let diff = git_diff_names(repo_dir);
    let unexpected_files: Vec<&String> = diff
        .iter()
        .filter(|name| !is_expected_notes_file(name))
        .collect();
    assert!(
        unexpected_files.is_empty(),
        "expected only notes UI files to change, got unexpected tracked files: {unexpected_files:?}"
    );
    assert!(
        diff.iter().any(|name| name == "src/public/notes-ui.js"),
        "expected notes UI to change, diff:\n{diff:?}"
    );
    assert!(
        diff.iter().any(|name| name == "src/public/index.html"),
        "expected notes UI help copy to change, diff:\n{diff:?}"
    );
}

fn contains_any(haystack: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| haystack.contains(needle))
}

fn function_body<'a>(source: &'a str, function_name: &str) -> Option<&'a str> {
    let signatures = [
        format!("function {function_name}"),
        format!("const {function_name}"),
        format!("let {function_name}"),
        format!("var {function_name}"),
    ];
    let signature_start = signatures
        .iter()
        .filter_map(|signature| source.find(signature))
        .min()?;
    let body_start = source[signature_start..].find('{')? + signature_start;
    let mut depth = 0usize;
    let mut body_end = None;

    for (offset, ch) in source[body_start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    body_end = Some(body_start + offset);
                    break;
                }
            }
            _ => {}
        }
    }

    body_end.map(|end| &source[body_start + 1..end])
}

fn branch_after_key_line(source: &str) -> &str {
    let after_first_line = source
        .find('\n')
        .map(|idx| &source[idx + 1..])
        .unwrap_or_default();
    let end = after_first_line
        .find("\n    } else")
        .or_else(|| after_first_line.find("\n  } else"))
        .or_else(|| after_first_line.find("\n    }"))
        .or_else(|| after_first_line.find("\n  }"))
        .unwrap_or(after_first_line.len());
    &after_first_line[..end]
}

fn line_has_typing_return_guard(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    lower.trim_start().starts_with("if")
        && lower.contains("return")
        && [
            "typing",
            "editing",
            "input",
            "textarea",
            "contenteditable",
            "activeelement",
            "target",
        ]
        .iter()
        .any(|needle| lower.contains(needle))
}

fn text_has_typing_return_guard(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    lower.contains("if")
        && lower.contains("return")
        && [
            "typing",
            "editing",
            "input",
            "textarea",
            "contenteditable",
            "activeelement",
            "target",
        ]
        .iter()
        .any(|needle| lower.contains(needle))
}

fn assert_duplicate_shortcut_respects_typing_guard(source: &str) {
    let keydown_start = source
        .find("keydown")
        .expect("notes UI should install a keydown handler");
    let after_keydown = &source[keydown_start..];
    let duplicate_key_index = [
        "event.key === 'd'",
        "event.key === \"d\"",
        "event.key.toLowerCase() === 'd'",
        "event.key.toLowerCase() === \"d\"",
    ]
    .iter()
    .filter_map(|needle| after_keydown.find(needle))
    .min()
    .expect("notes UI should handle the d duplicate shortcut");
    let before_duplicate_key = &after_keydown[..duplicate_key_index];
    let duplicate_key_line = after_keydown[duplicate_key_index..]
        .lines()
        .next()
        .unwrap_or_default();
    let guarded_by_top_level_return = before_duplicate_key
        .lines()
        .any(line_has_typing_return_guard)
        || text_has_typing_return_guard(before_duplicate_key);
    let duplicate_key_line_lower = duplicate_key_line.to_ascii_lowercase();
    let duplicate_branch = branch_after_key_line(&after_keydown[duplicate_key_index..]);
    let guarded_inside_duplicate_branch = duplicate_branch
        .lines()
        .any(line_has_typing_return_guard)
        || text_has_typing_return_guard(duplicate_branch);

    assert!(
        guarded_by_top_level_return
            || guarded_inside_duplicate_branch
            || duplicate_key_line_lower.contains("!istyping")
            || duplicate_key_line_lower.contains("!typing")
            || duplicate_key_line_lower.contains("!isediting")
            || duplicate_key_line_lower.contains("!isinput")
            || duplicate_key_line_lower.contains("!istextinput"),
        "duplicate shortcut should be gated so it does not fire while typing"
    );
}

fn definition_brace_depth(source: &str, markers: &[&str]) -> Option<usize> {
    let definition_start = markers
        .iter()
        .filter_map(|marker| source.find(marker))
        .min()?;
    let mut depth = 0usize;
    for ch in source[..definition_start].chars() {
        match ch {
            '{' => depth = depth.saturating_add(1),
            '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    Some(depth)
}

fn assert_duplicate_function_is_callable_from_event_handlers(source: &str) {
    let depth = definition_brace_depth(source, &[
        "async function duplicateSelectedNote",
        "function duplicateSelectedNote",
        "const duplicateSelectedNote",
        "let duplicateSelectedNote",
        "var duplicateSelectedNote",
    ])
    .expect("duplicateSelectedNote should be defined");

    assert_eq!(
        depth, 1,
        "duplicateSelectedNote should be defined at the top level of the notes UI module, not nested inside another function"
    );
}

fn assert_notes_feature_behavior(repo_dir: &Path) {
    let index_html = fs::read_to_string(repo_dir.join("src/public/index.html"))
        .expect("read src/public/index.html");
    let notes_ui_js = fs::read_to_string(repo_dir.join("src/public/notes-ui.js"))
        .expect("read src/public/notes-ui.js");

    let duplicate_button_index = index_html
        .find("id=\"duplicateButton\"")
        .expect("duplicate button should be present in index.html");
    let delete_button_index = index_html
        .find("id=\"deleteButton\"")
        .expect("delete button should remain present in index.html");
    assert!(
        duplicate_button_index < delete_button_index,
        "duplicate button should appear before delete button"
    );
    assert!(
        index_html.contains("Duplicate note"),
        "duplicate button label should be visible"
    );
    assert!(
        index_html.contains("D duplicate") && index_html.contains("Ctrl+Enter"),
        "keyboard hint should mention duplicate and existing shortcuts"
    );

    assert!(
        contains_any(
            &notes_ui_js,
            &[
                "duplicateButton: document.getElementById('duplicateButton')",
                "duplicateButton: document.getElementById(\"duplicateButton\")",
            ],
        ),
        "notes UI should cache duplicateButton in els"
    );
    assert!(
        contains_any(
            &notes_ui_js,
            &[
                "els.duplicateButton.addEventListener('click', duplicateSelectedNote)",
                "els.duplicateButton.addEventListener(\"click\", duplicateSelectedNote)",
            ],
        ),
        "duplicate button should be wired to duplicateSelectedNote"
    );
    assert!(
        notes_ui_js.contains("duplicateSelectedNote"),
        "duplicateSelectedNote should be implemented"
    );
    assert_duplicate_function_is_callable_from_event_handlers(&notes_ui_js);
    let duplicate_body = function_body(&notes_ui_js, "duplicateSelectedNote")
        .expect("duplicateSelectedNote should be a function body");
    assert!(
        notes_ui_js.contains("els.duplicateButton.disabled = true")
            && notes_ui_js.contains("els.duplicateButton.disabled = false"),
        "duplicate button should be disabled without a note and enabled with a note"
    );
    assert!(
        contains_any(
            duplicate_body,
            &["method: 'POST'", "method: \"POST\"", "method: 'post'", "method: \"post\""],
        ),
        "duplicateSelectedNote should create a fresh note via POST"
    );
    for expected in [
        "Copy of",
        "note.title",
        "note.body",
        "note.tags",
        "note.pinned",
        "note.archived",
        "state.selectedId",
        "renderEditor",
        "els.noteTitle.focus()",
    ] {
        assert!(
            duplicate_body.contains(expected),
            "notes UI should preserve duplicate workflow detail: {expected}"
        );
    }
    assert!(
        contains_any(
            &notes_ui_js,
            &[
                "addEventListener('keydown'",
                "addEventListener(\"keydown\"",
                "onkeydown",
            ],
        ),
        "notes UI should install a keyboard handler"
    );
    assert!(
        contains_any(
            &notes_ui_js,
            &[
                "event.key === 'd'",
                "event.key === \"d\"",
                "event.key.toLowerCase() === 'd'",
                "event.key.toLowerCase() === \"d\"",
            ],
        ),
        "notes UI should handle the d duplicate shortcut"
    );
    assert_duplicate_shortcut_respects_typing_guard(&notes_ui_js);
    assert!(
        !contains_any(
            &notes_ui_js,
            &[
                "els.pin.checked",
                "els.archive.checked",
                "getElementById('pin')",
                "getElementById(\"pin\")",
                "getElementById('archive')",
                "getElementById(\"archive\")",
            ],
        ),
        "notes UI should keep using notePinned and noteArchived"
    );
}

fn assert_notes_js_syntax(repo_dir: &Path) {
    let js_files = [
        repo_dir.join("src/public/notes-ui.js"),
        repo_dir.join("src/server.js"),
        repo_dir.join("src/routes/notes.js"),
    ];

    for js_file in js_files {
        let output = Command::new("node")
            .arg("--check")
            .arg(&js_file)
            .output()
            .expect("node --check");
        assert!(
            output.status.success(),
            "node --check failed for {}\nstdout:\n{}\nstderr:\n{}",
            js_file.display(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

fn run_notes_app_live_feature_workflow(spec: &ProviderSpec) {
    let Some(api_key) = live_key(spec) else {
        eprintln!(
            "skipping notes-app live E2E for {}: {} is not set",
            spec.provider_id, spec.primary_api_key_env
        );
        return;
    };

    for &model in &selected_models(spec) {
        let kay_home = TempDir::new().expect("temp KAY_HOME");
        let _sessions = SessionPreserver::new(
            kay_home.path(),
            format!(
                "test_notes_app_live_e2e_{}_{}",
                spec.provider_id,
                model.replace('/', "_")
            ),
        );
        login_provider(&kay_home, spec, &api_key);

        let (_workspace_guard, repo_dir) = clone_repo();
        let last_message_dir = TempDir::new().expect("last message tempdir");
        let last_message = last_message_dir
            .path()
            .join(format!("{}.txt", model.replace('/', "_")));

        let prompt = build_notes_feature_prompt(&repo_dir);

        let response = run_notes_feature_turn(
            &kay_home,
            &repo_dir,
            spec.provider_id,
            model,
            prompt.trim(),
            &last_message,
        );

        assert!(
            !response.summary.trim().is_empty(),
            "expected structured summary for {model}"
        );
        let mut touched_files = response.touched_files.clone();
        touched_files.sort();
        touched_files.dedup();
        let expected_files = EXPECTED_NOTES_FILES
            .iter()
            .map(|file| file.to_string())
            .collect::<Vec<_>>();
        assert!(
            touched_files == expected_files,
            "expected the model to touch only the notes UI files for {model}, got {touched_files:?}"
        );

        assert_notes_feature_change(&repo_dir);
        assert_notes_feature_behavior(&repo_dir);
        assert_notes_js_syntax(&repo_dir);

        let transcript_path = copy_transcript(kay_home.path(), &repo_dir, model);
        assert!(
            transcript_path.exists(),
            "expected transcript copy at {}",
            transcript_path.display()
        );
    }
}

#[test]
fn notes_response_parser_accepts_trailing_contracted_json() {
    let response = parse_notes_work_response(
        "All edits are complete.\n\n{\"summary\":\"done\",\"touched_files\":[\"src/public/index.html\",\"src/public/notes-ui.js\"]}",
    )
    .expect("MiMo may return prose before the final contracted JSON object");

    assert_eq!(response.summary, "done");
    assert_eq!(
        response.touched_files,
        vec!["src/public/index.html", "src/public/notes-ui.js"]
    );
}

#[test]
fn notes_response_parser_rejects_unknown_trailing_json_fields() {
    assert!(
        parse_notes_work_response(
            "Done.\n{\"summary\":\"done\",\"touched_files\":[],\"extra\":true}"
        )
        .is_none(),
        "trailing JSON extraction should still enforce the exact contract"
    );
}

#[test]
fn duplicate_function_scope_check_rejects_nested_function() {
    let source = r#"
      (function () {
        function selectNote(id) {
          async function duplicateSelectedNote() {}
        }
        els.duplicateButton.addEventListener('click', duplicateSelectedNote);
      })();
    "#;

    assert_ne!(
        definition_brace_depth(source, &["async function duplicateSelectedNote"]),
        Some(1),
        "nested duplicateSelectedNote should not look module-callable"
    );
}

#[test]
fn duplicate_shortcut_guard_accepts_inline_typing_flag() {
    let source = r#"
      document.addEventListener('keydown', (event) => {
        const tag = document.activeElement.tagName;
        const typing = tag === 'INPUT' || tag === 'TEXTAREA';
        if (event.key === 'd' && !typing) {
          event.preventDefault();
          duplicateSelectedNote();
        }
      });
    "#;

    assert_duplicate_shortcut_respects_typing_guard(source);
}

#[test]
fn opencode_go_notes_app_live_feature_workflow() {
    run_notes_app_live_feature_workflow(&OPENCODE_GO_SPEC);
}

#[test]
fn xiaomi_notes_app_live_feature_workflow() {
    run_notes_app_live_feature_workflow(&XIAOMI_SPEC);
}
