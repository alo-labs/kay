//! Prompt-driven final STATUS block detection, salvage, and repair prompts.

use code_protocol::models::ContentItem;
use code_protocol::models::ResponseInputItem;

pub const FINAL_STATUS_REPAIR_PROMPT: &str = "Your last reply did not begin with STATUS:. If required code or verify scripts are still incomplete, continue with tool calls now—do not emit STATUS: BLOCKED just because of this reminder. When (and only when) both required verify scripts exit 0, reply with ONLY a final status block: STATUS: SUCCESS, FILES_CHANGED:, and TESTS_RUN:. Use STATUS: BLOCKED only when a true external blocker remains after honest effort.";

pub fn message_starts_with_status(message: &str) -> bool {
    status_head_line(message)
        .map(|line| line.to_ascii_uppercase().starts_with("STATUS:"))
        .unwrap_or(false)
}

pub fn status_head_line(message: &str) -> Option<&str> {
    let first = message
        .trim_start_matches(|ch: char| ch.is_whitespace() || ch == '`' || ch == '*')
        .lines()
        .next()?
        .trim();
    if first.is_empty() {
        None
    } else {
        Some(first)
    }
}

pub fn status_head_is_complete(message: &str) -> bool {
    status_head_line(message)
        .map(|line| {
            let upper = line.to_ascii_uppercase();
            upper.starts_with("STATUS: SUCCESS")
                || upper.starts_with("STATUS: BLOCKED")
                || upper.starts_with("STATUS: PASS")
                || upper.starts_with("STATUS: FAIL")
        })
        .unwrap_or(false)
}

/// Returns true when the model's last turn looks like mid-task narration rather
/// than an attempted final closeout, so a STATUS repair would abort real work.
pub fn prompt_requires_verify_scripts(prompt: &str) -> bool {
    let lower = prompt.to_ascii_lowercase();
    lower.contains("scripts/verify-")
        || lower.contains("verify scripts exit 0")
        || lower.contains("both verify scripts")
}

pub fn status_message_is_blocked(message: &str) -> bool {
    status_head_line(message)
        .map(|line| line.to_ascii_uppercase().starts_with("STATUS: BLOCKED"))
        .unwrap_or(false)
}

/// When a prompt requires local verify scripts, `STATUS: BLOCKED` before they
/// pass is a premature closeout — nudge the model to keep using tools.
pub fn should_nudge_premature_blocked_closeout(
    prompt: &str,
    last_agent_message: Option<&str>,
) -> bool {
    let Some(message) = last_agent_message else {
        return false;
    };
    prompt_requires_verify_scripts(prompt) && status_message_is_blocked(message)
}

pub fn should_defer_turn_final_status_repair(last_agent_message: Option<&str>) -> bool {
    let Some(message) = last_agent_message else {
        return false;
    };
    let trimmed = message.trim();
    if trimmed.is_empty() || message_starts_with_status(trimmed) {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    if lower.contains("status:") {
        return false;
    }
    const IN_PROGRESS_MARKERS: &[&str] = &[
        "let me ",
        "i'll ",
        "i will ",
        "now i ",
        "first,",
        "step 1",
        "next i",
        "reading ",
        "implementing ",
        "patching ",
        "exploring ",
    ];
    IN_PROGRESS_MARKERS
        .iter()
        .any(|marker| lower.contains(marker))
}

pub fn salvage_final_status_message(message: &str) -> Option<String> {
    if status_head_is_complete(message) {
        return None;
    }

    let upper = message.to_ascii_uppercase();
    let status_idx = upper.find("STATUS:")?;
    let tail = message[status_idx..].trim_start();
    let mut lines: Vec<&str> = Vec::new();

    for line in tail.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            if !lines.is_empty() {
                break;
            }
            continue;
        }

        let upper_line = trimmed.to_ascii_uppercase();
        if lines.is_empty() {
            if upper_line.starts_with("STATUS: SUCCESS")
                || upper_line.starts_with("STATUS: BLOCKED")
                || upper_line.starts_with("STATUS: PASS")
                || upper_line.starts_with("STATUS: FAIL")
            {
                lines.push(trimmed);
            } else {
                return None;
            }
            continue;
        }

        if upper_line.starts_with("FILES_CHANGED:")
            || upper_line.starts_with("TESTS_RUN:")
            || upper_line.starts_with("BLOCKER:")
            || upper_line.starts_with("NOTES:")
        {
            lines.push(trimmed);
            continue;
        }

        break;
    }

    if lines.is_empty() {
        return None;
    }

    Some(lines.join("\n"))
}

pub fn try_salvage_last_task_message(last_task_message: &mut Option<String>) {
    let Some(message) = last_task_message.as_ref() else {
        return;
    };
    if let Some(salvaged) = salvage_final_status_message(message) {
        *last_task_message = Some(salvaged);
    }
}

pub fn should_request_final_status_repair(
    prompt: &str,
    last_agent_message: Option<&str>,
    status_repair_attempts: usize,
) -> bool {
    status_repair_attempts < 2
        && prompt_requires_final_status(prompt)
        && final_status_contract_missing(prompt, last_agent_message)
}

pub fn final_status_contract_missing(prompt: &str, last_agent_message: Option<&str>) -> bool {
    if !prompt_requires_final_status(prompt) {
        return false;
    }

    match last_agent_message {
        Some(message) => !final_status_contract_satisfied(prompt, message),
        None => true,
    }
}

pub fn final_status_contract_satisfied(prompt: &str, message: &str) -> bool {
    if status_head_is_complete(message) {
        return true;
    }

    if message_starts_with_status(message) {
        return false;
    }

    let trimmed = message.trim_start_matches(|ch: char| ch.is_whitespace() || ch == '`' || ch == '*');
    let upper = trimmed.to_ascii_uppercase();
    if upper.starts_with("STATUS:") {
        return false;
    }

    let lower_prompt = prompt.to_ascii_lowercase();
    if lower_prompt.contains("must include") || lower_prompt.contains("include status") {
        return upper.contains("STATUS: SUCCESS") || upper.contains("STATUS: BLOCKED");
    }

    false
}

pub fn prompt_requires_final_status(prompt: &str) -> bool {
    let lower = prompt.to_ascii_lowercase();
    let mentions_final = lower.contains("final response")
        || lower.contains("final answer")
        || lower.contains("final message")
        || lower.contains("emit status block")
        || lower.contains("delegation closeout");
    let mentions_required_status = lower.contains("status:");
    let requires_status_contract = lower.contains("begin")
        || lower.contains("start")
        || lower.contains("must include")
        || lower.contains("must be exactly")
        || lower.contains("include status")
        || lower.contains("status: success")
        || lower.contains("status: blocked")
        || lower.contains("status: pass")
        || lower.contains("status: fail")
        || (lower.contains("success criteria") && lower.contains("status:"));

    mentions_final && mentions_required_status && requires_status_contract
}

pub const TURN_CONTINUE_NUDGE_PROMPT: &str = "Continue the task with tool calls. Do not stop for narration—finish wiring (including notes-ui.js if needed), ensure scripts/verify-*.sh exist via apply_patch, run both verify scripts with PORT exported, then reply with only STATUS: SUCCESS (or STATUS: BLOCKED if truly stuck) plus FILES_CHANGED and TESTS_RUN.";

pub fn turn_continue_nudge_input() -> ResponseInputItem {
    ResponseInputItem::Message {
        role: "developer".to_string(),
        content: vec![ContentItem::InputText {
            text: TURN_CONTINUE_NUDGE_PROMPT.to_string(),
        }],
    }
}

pub fn final_status_repair_input(attempt: usize) -> ResponseInputItem {
    let text = if attempt == 1 {
        FINAL_STATUS_REPAIR_PROMPT.to_string()
    } else {
        format!(
            "{FINAL_STATUS_REPAIR_PROMPT}\n\nThis is repair attempt {attempt}. Do not add any other text."
        )
    };
    ResponseInputItem::Message {
        role: "developer".to_string(),
        content: vec![ContentItem::InputText { text }],
    }
}

pub fn status_contract_prompt_from_input(input: &[crate::protocol::InputItem]) -> String {
    input
        .iter()
        .filter_map(|item| match item {
            crate::protocol::InputItem::Text { text } => Some(text.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn salvage_promotes_mid_message_status_to_prefix() {
        let message = "Rewriting routes file cleanly.\n\nSTATUS: SUCCESS\nFILES_CHANGED: src/routes/notes.js";
        let salvaged = salvage_final_status_message(message).expect("status block");
        assert!(message_starts_with_status(&salvaged));
        assert!(salvaged.contains("FILES_CHANGED:"));
        assert!(!final_status_contract_missing(
            "Final message STATUS: SUCCESS with FILES_CHANGED and TESTS_RUN.",
            Some(&salvaged),
        ));
    }

    #[test]
    fn detects_success_criteria_status_requirement() {
        let prompt = "SUCCESS CRITERIA:\n- Both verify scripts exit 0.\n- STATUS: SUCCESS with FILES_CHANGED and TESTS_RUN in final message.";
        assert!(prompt_requires_final_status(prompt));
        assert!(final_status_contract_missing(
            prompt,
            Some("Still implementing sort API."),
        ));
    }

    #[test]
    fn repair_input_is_developer_role() {
        let ResponseInputItem::Message { role, .. } = final_status_repair_input(1) else {
            panic!("expected developer repair message");
        };
        assert_eq!(role, "developer");
    }

    #[test]
    fn markdown_bold_status_prefix_counts_as_contract() {
        let message = "**STATUS: SUCCESS**\nFILES_CHANGED: src/routes/notes.js";
        assert!(message_starts_with_status(message));
        assert!(final_status_contract_satisfied(
            "Final message STATUS: SUCCESS with FILES_CHANGED.",
            message,
        ));
    }

    #[test]
    fn nudges_premature_blocked_when_verify_scripts_required() {
        let prompt = "SUCCESS CRITERIA:\n- Both verify scripts exit 0.\n- STATUS: SUCCESS with FILES_CHANGED and TESTS_RUN in final message.";
        assert!(should_nudge_premature_blocked_closeout(
            prompt,
            Some("STATUS: BLOCKED\nFILES_CHANGED: []"),
        ));
        assert!(!should_nudge_premature_blocked_closeout(
            prompt,
            Some("STATUS: SUCCESS\nFILES_CHANGED: []"),
        ));
        assert!(!should_nudge_premature_blocked_closeout(
            "No verify requirement here.",
            Some("STATUS: BLOCKED\nFILES_CHANGED: []"),
        ));
    }

    #[test]
    fn defers_status_repair_for_in_progress_narration() {
        assert!(should_defer_turn_final_status_repair(Some(
            "Let me read the key files to understand the existing structure."
        )));
        assert!(!should_defer_turn_final_status_repair(Some(
            "STATUS: BLOCKED\nFILES_CHANGED: []"
        )));
    }

    #[test]
    fn partial_status_head_is_not_contract_satisfied() {
        assert!(!status_head_is_complete("status:"));
        assert!(!final_status_contract_satisfied(
            "Final message STATUS: SUCCESS with FILES_CHANGED.",
            "status:",
        ));
        assert!(final_status_contract_satisfied(
            "Final message STATUS: SUCCESS with FILES_CHANGED.",
            "STATUS: SUCCESS\nFILES_CHANGED: []",
        ));
    }
}
