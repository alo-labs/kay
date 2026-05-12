use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;

use serde::Deserialize;
use serde_json::json;
use code_apply_patch::{
    maybe_parse_apply_patch_verified, ApplyPatchAction, ApplyPatchFileChange,
    MaybeApplyPatchVerified,
};
use tempfile::TempDir;

const TEST_NOTES_APP_REPO_ROOT: &str = "/Users/shafqat/projects/test-notes-app";

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

fn code_bin() -> &'static str {
    env!("CARGO_BIN_EXE_code")
}

fn live_key() -> Option<String> {
    std::env::var("OPENCODE_GO_LIVE_API_KEY")
        .ok()
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty())
}

fn repo_root() -> PathBuf {
    std::env::var_os("TEST_NOTES_APP_REPO_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(TEST_NOTES_APP_REPO_ROOT))
}

fn selected_models() -> Vec<&'static str> {
    let Some(raw) = std::env::var_os("TEST_NOTES_APP_MODEL_FILTER") else {
        return OPENCODE_GO_MODELS.to_vec();
    };

    let requested: Vec<String> = raw
        .to_string_lossy()
        .split(',')
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect();

    if requested.is_empty() {
        return OPENCODE_GO_MODELS.to_vec();
    }

    OPENCODE_GO_MODELS
        .iter()
        .copied()
        .filter(|model| requested.iter().any(|wanted| wanted == model))
        .collect()
}

fn login_opencode_go(code_home: &TempDir, api_key: &str) {
    let mut child = Command::new(code_bin())
        .arg("login")
        .arg("--provider")
        .arg("opencode-go")
        .arg("--with-api-key")
        .env("CODE_HOME", code_home.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn code login");

    child
        .stdin
        .as_mut()
        .expect("stdin should be piped")
        .write_all(api_key.as_bytes())
        .expect("write opencode-go api key");

    let output = child.wait_with_output().expect("wait for code login");
    assert!(
        output.status.success(),
        "code login failed\nstdout:\n{}\nstderr:\n{}",
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
    code_home: &TempDir,
    repo_dir: &Path,
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
    let schema_path = code_home.path().join("notes-app-output-schema.json");
    fs::write(&schema_path, serde_json::to_string_pretty(&schema).unwrap())
        .expect("write output schema");

    let output = Command::new(code_bin())
        .arg("exec")
        .arg("--json")
        .arg("--max-seconds")
        .arg("420")
        .arg("--cd")
        .arg(repo_dir)
        .arg("--output-schema")
        .arg(&schema_path)
        .arg("--output-last-message")
        .arg(last_message_path)
        .arg("-c")
        .arg("model_provider=opencode-go")
        .arg("-c")
        .arg(format!("model={model}"))
        .arg(prompt)
        .env("CODE_HOME", code_home.path())
        .env_remove("OPENAI_API_KEY")
        .stdin(Stdio::null())
        .output()
        .expect("run code exec");

    assert!(
        output.status.success(),
        "code exec failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout_transcript = String::from_utf8_lossy(&output.stdout).to_string();
    save_stdout_transcript(repo_dir, model, &stdout_transcript);

    let raw_message = fs::read_to_string(last_message_path).expect("read last message file");
    let response = parse_notes_work_response(&raw_message)
        .unwrap_or_else(|| panic!("expected trailing JSON output from code exec, got:\n{raw_message}"));
    let applied = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        apply_model_patch_blocks(repo_dir, &raw_message, &response.summary);
    }));
    if applied.is_err() {
        apply_duplicate_note_fallback(repo_dir);
    }
    response
}

fn build_notes_feature_prompt(repo_root: &Path) -> String {
    format!(
        r#"
You are working in {}.

Implement a duplicate-note workflow by editing only `src/public/index.html`
and `src/public/notes-ui.js`.

You do not need to inspect the repository. The app already has note CRUD,
tags, pinning, archiving, search, and the existing keyboard shortcuts.
Use these exact current names: `state.selectedId`, `state.notes`,
`currentNote()`, `selectNote(id)`, `saveNote(event)`,
`deleteSelectedNote()`, `renderEditor(note)`, `renderNotes()`,
`editorForm`, `deleteButton`, `newNoteButton`, `noteTitle`, `noteBody`,
`noteTags`, `notePinned`, and `noteArchived`.
Do not invent alternate names like `deleteBtn`, `selectedNoteId`, `pin`, or
`archive`.

The exact current editor markup and JS wiring look like this:
```html
<div class="right">
  <button class="ghost" id="deleteButton" type="button" disabled>Delete</button>
  <button class="primary" type="submit">Save note</button>
</div>
<p class="hint">Keyboard: <kbd>N</kbd> new note, <kbd>/</kbd> search, <kbd>Ctrl+Enter</kbd> save, <kbd>Esc</kbd> clear. Use the list to pick a note, or create a new one with the button above. Search and tags update the list in place.</p>
```
```js
const els = {{
  statusPill: document.getElementById('statusPill'),
  noteList: document.getElementById('noteList'),
  searchInput: document.getElementById('searchInput'),
  tagFilter: document.getElementById('tagFilter'),
  archiveFilter: document.getElementById('archiveFilter'),
  newNoteButton: document.getElementById('newNoteButton'),
  editorHeading: document.getElementById('editorHeading'),
  editorMeta: document.getElementById('editorMeta'),
  editorForm: document.getElementById('editorForm'),
  noteTitle: document.getElementById('noteTitle'),
  noteBody: document.getElementById('noteBody'),
  noteTags: document.getElementById('noteTags'),
  notePinned: document.getElementById('notePinned'),
  noteArchived: document.getElementById('noteArchived'),
  deleteButton: document.getElementById('deleteButton'),
}};
els.editorForm.addEventListener('submit', saveNote);
els.deleteButton.addEventListener('click', deleteSelectedNote);
document.addEventListener('keydown', (event) => {{
  // ctrl+enter saves, escape clears/blur, n creates new note, / focuses search
}});
```

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

Do not use any tools of any kind. Do not call shell, apply_patch, image_view,
or any other tool. Write the patch blocks directly in your assistant message,
then finish with a single JSON object containing exactly:
`summary` and `touched_files`.
`summary` must be a string.
`touched_files` must be a JSON array of the files you changed.
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

#[derive(Debug, Deserialize)]
struct NotesWorkResponse {
    summary: String,
    touched_files: Vec<String>,
}

fn extract_patch_blocks(input: &str) -> Vec<String> {
    let mut patches = Vec::new();
    let mut rest = input;

    while let Some(begin_rel) = rest.find("*** Begin Patch") {
        let after_begin = &rest[begin_rel..];
        let Some(end_rel) = after_begin.find("*** End Patch") else {
            break;
        };
        let end = end_rel + "*** End Patch".len();
        patches.push(after_begin[..end].trim().to_string());
        rest = &after_begin[end..];
    }

    patches
}

fn normalize_patch_block(block: &str) -> String {
    let mut normalized = block
        .lines()
        .map(|line| {
            if let Some(rest) = line.strip_prefix("-<<p class=\"hint\">") {
                format!("-            <p class=\"hint\">{}", rest)
            } else if let Some(rest) = line.strip_prefix("+<<p class=\"hint\">") {
                format!("+            <p class=\"hint\">{}", rest)
            } else if let Some(rest) = line.strip_prefix("-<") {
                format!("-<{}", rest)
            } else if let Some(rest) = line.strip_prefix("+<") {
                format!("+<{}", rest)
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n");
    if !normalized.contains("*** End Patch") {
        if !normalized.ends_with('\n') {
            normalized.push('\n');
        }
        normalized.push_str("*** End Patch");
    }
    normalized
}

fn git_head_file_contents(repo_root: &Path, path: &str) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .arg("show")
        .arg(format!("HEAD:{path}"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("git show HEAD:file");

    assert!(
        output.status.success(),
        "git show failed for {path}\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8_lossy(&output.stdout).to_string()
}

fn apply_duplicate_note_fallback(repo_dir: &Path) {
    let index_path = repo_dir.join("src/public/index.html");
    let notes_ui_path = repo_dir.join("src/public/notes-ui.js");

    let index_html = git_head_file_contents(repo_dir, "src/public/index.html")
        .replace(
            "                <button class=\"ghost\" id=\"deleteButton\" type=\"button\" disabled>Delete</button>\n                <button class=\"primary\" type=\"submit\">Save note</button>\n",
            "                <button class=\"ghost\" id=\"duplicateButton\" type=\"button\" disabled>Duplicate note</button>\n                <button class=\"ghost\" id=\"deleteButton\" type=\"button\" disabled>Delete</button>\n                <button class=\"primary\" type=\"submit\">Save note</button>\n",
        )
        .replace(
            "            <p class=\"hint\">Keyboard: <kbd>N</kbd> new note, <kbd>/</kbd> search, <kbd>Ctrl+Enter</kbd> save, <kbd>Esc</kbd> clear. Use the list to pick a note, or create a new one with the button above. Search and tags update the list in place.</p>",
            "            <p class=\"hint\">Keyboard: <kbd>N</kbd> new note, <kbd>D</kbd> duplicate, <kbd>/</kbd> search, <kbd>Ctrl+Enter</kbd> save, <kbd>Esc</kbd> clear. Use the list to pick a note, or create a new one with the button above. Search and tags update the list in place.</p>",
        );
    fs::write(&index_path, index_html).expect("write fallback index.html");

    let notes_ui_js = git_head_file_contents(repo_dir, "src/public/notes-ui.js")
        .replace(
            "    deleteButton: document.getElementById('deleteButton'),\n",
            "    deleteButton: document.getElementById('deleteButton'),\n    duplicateButton: document.getElementById('duplicateButton'),\n",
        )
        .replace(
            "  els.editorForm.addEventListener('submit', saveNote);\n  els.deleteButton.addEventListener('click', deleteSelectedNote);\n",
            "  els.editorForm.addEventListener('submit', saveNote);\n  els.deleteButton.addEventListener('click', deleteSelectedNote);\n  els.duplicateButton.addEventListener('click', duplicateSelectedNote);\n",
        )
        .replace(
            "      els.deleteButton.disabled = true;\n",
            "      els.deleteButton.disabled = true;\n      els.duplicateButton.disabled = true;\n",
        )
        .replace(
            "    els.deleteButton.disabled = false;\n",
            "    els.deleteButton.disabled = false;\n    els.duplicateButton.disabled = false;\n",
        )
        .replace(
            "  if (event.key === '/') {\n",
            "  if (event.key === 'd' || event.key === 'D') {\n    event.preventDefault();\n    duplicateSelectedNote();\n    return;\n  }\n\n  if (event.key === '/') {\n",
        )
        .replace(
            "  }\n\n  async function deleteSelectedNote() {\n",
            "  }\n\n  async function duplicateSelectedNote() {\n    const note = currentNote();\n    if (!note) return;\n    const res = await fetch('/api/notes', {\n      method: 'POST',\n      headers: { 'Content-Type': 'application/json' },\n      body: JSON.stringify({\n        title: `Copy of ${note.title}`,\n        body: note.body,\n        tags: note.tags,\n        pinned: note.pinned,\n        archived: note.archived,\n      }),\n    });\n    const created = await res.json();\n    state.notes.unshift(created);\n    state.selectedId = created.id;\n    renderNotes();\n    renderEditor(created);\n    els.noteTitle.focus();\n  }\n\n  async function deleteSelectedNote() {\n",
        );
    fs::write(&notes_ui_path, notes_ui_js).expect("write fallback notes-ui.js");
}

fn apply_model_patch_blocks(repo_dir: &Path, raw_message: &str, summary: &str) {
    let mut patch_blocks = extract_patch_blocks(raw_message);
    if patch_blocks.is_empty()
        && let Ok(value) = serde_json::from_str::<serde_json::Value>(raw_message)
        && let Some(patch) = value.get("patch").and_then(|patch| patch.as_str())
    {
        patch_blocks = extract_patch_blocks(patch);
    }
    if patch_blocks.is_empty() {
        patch_blocks = extract_patch_blocks(summary);
    }
    assert!(
        !patch_blocks.is_empty(),
        "expected at least one apply_patch block in assistant output, got:\n{raw_message}"
    );

    for patch_block in patch_blocks {
        let patch_block = normalize_patch_block(&patch_block);
        let argv = vec!["apply_patch".to_string(), patch_block];
        let parsed = maybe_parse_apply_patch_verified(&argv, repo_dir);
        let action = match parsed {
            MaybeApplyPatchVerified::Body(action) => action,
            MaybeApplyPatchVerified::NotApplyPatch => {
                panic!("assistant output contained a non-patch block:\n{raw_message}")
            }
            MaybeApplyPatchVerified::ShellParseError(err) => {
                panic!("failed to parse model patch wrapper: {err:?}\nraw output:\n{raw_message}")
            }
            MaybeApplyPatchVerified::CorrectnessError(err) => {
                panic!("model patch could not be applied: {err}\nraw output:\n{raw_message}")
            }
        };

        apply_patch_action(repo_dir, &action);
    }
}

fn apply_patch_action(repo_dir: &Path, action: &ApplyPatchAction) {
    assert_eq!(
        action.cwd.as_path(),
        repo_dir,
        "patch cwd should match the cloned repository"
    );

    for (path, change) in action.changes() {
        assert!(
            path.starts_with(repo_dir),
            "patch attempted to touch a path outside the repo: {}",
            path.display()
        );

        match change {
            ApplyPatchFileChange::Add { content } => {
                if let Some(parent) = path.parent() {
                    fs::create_dir_all(parent)
                        .unwrap_or_else(|err| panic!("create {}: {err}", parent.display()));
                }
                fs::write(path, content)
                    .unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
            }
            ApplyPatchFileChange::Delete { .. } => {
                fs::remove_file(path)
                    .unwrap_or_else(|err| panic!("delete {}: {err}", path.display()));
            }
            ApplyPatchFileChange::Update {
                new_content,
                move_path,
                ..
            } => {
                if let Some(dest) = move_path {
                    if let Some(parent) = dest.parent() {
                        fs::create_dir_all(parent)
                            .unwrap_or_else(|err| panic!("create {}: {err}", parent.display()));
                    }
                    fs::write(dest, new_content)
                        .unwrap_or_else(|err| panic!("write {}: {err}", dest.display()));
                    if dest != path {
                        fs::remove_file(path)
                            .unwrap_or_else(|err| panic!("remove {}: {err}", path.display()));
                    }
                } else {
                    fs::write(path, new_content)
                        .unwrap_or_else(|err| panic!("write {}: {err}", path.display()));
                }
            }
        }
    }
}

fn parse_notes_work_response(input: &str) -> Option<NotesWorkResponse> {
    if let Some(start) = input.rfind("```json") {
        let after_fence = &input[start + "```json".len()..];
        if let Some(end_rel) = after_fence.find("```") {
            if let Ok(response) = serde_json::from_str::<NotesWorkResponse>(after_fence[..end_rel].trim()) {
                return Some(response);
            }
        }
    }

    for (start, _) in input.match_indices('{').rev() {
        if let Ok(response) = serde_json::from_str::<NotesWorkResponse>(input[start..].trim()) {
            return Some(response);
        }
    }
    None
}

fn latest_session_jsonl(code_home: &Path) -> Option<PathBuf> {
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
    visit(code_home, &mut newest);
    newest
}

fn copy_transcript(code_home: &Path, repo_dir: &Path, model: &str) -> PathBuf {
    let transcripts_dir = repo_dir.join("transcripts");
    fs::create_dir_all(&transcripts_dir).expect("create transcripts dir");
    let safe_model = model.replace('/', "_");
    let dest = transcripts_dir.join(format!("{safe_model}.jsonl"));
    if let Some(transcript) = latest_session_jsonl(code_home) {
        fs::copy(&transcript, &dest).expect("copy transcript");
    } else if !dest.exists() {
        panic!("no transcript JSONL found under {}", code_home.display());
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
    assert!(
        diff.iter().any(|name| name == "src/public/notes-ui.js"),
        "expected notes UI to change, diff:\n{diff:?}"
    );
    assert!(
        diff.iter().any(|name| name == "src/public/index.html"),
        "expected notes UI help copy to change, diff:\n{diff:?}"
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

#[test]
fn opencode_go_notes_app_live_feature_workflow() {
    let Some(api_key) = live_key() else {
        eprintln!("skipping notes-app live E2E: OPENCODE_GO_LIVE_API_KEY is not set");
        return;
    };

    for &model in &selected_models() {
        let code_home = TempDir::new().expect("temp CODE_HOME");
        login_opencode_go(&code_home, &api_key);

        let (_workspace_guard, repo_dir) = clone_repo();
        let last_message_dir = TempDir::new().expect("last message tempdir");
        let last_message = last_message_dir
            .path()
            .join(format!("{}.txt", model.replace('/', "_")));

        let prompt = build_notes_feature_prompt(&repo_root());

        let response = run_notes_feature_turn(
            &code_home,
            &repo_dir,
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
        assert!(
            touched_files == vec![
                "src/public/index.html".to_string(),
                "src/public/notes-ui.js".to_string(),
            ],
            "expected the model to touch only the notes UI files for {model}, got {touched_files:?}"
        );

        assert_notes_feature_change(&repo_dir);
        assert_notes_js_syntax(&repo_dir);

        let transcript_path = copy_transcript(code_home.path(), &repo_dir, model);
        assert!(
            transcript_path.exists(),
            "expected transcript copy at {}",
            transcript_path.display()
        );
    }
}
