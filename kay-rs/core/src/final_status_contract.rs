//! Prompt-driven final STATUS block detection, salvage, and repair prompts.

use code_protocol::models::ContentItem;
use code_protocol::models::ResponseInputItem;

pub const FINAL_STATUS_REPAIR_PROMPT: &str = "STOP. Your last reply did not begin with STATUS:. Reply with ONLY a final status block—no narration or tool calls. Start the first line with STATUS: SUCCESS or STATUS: BLOCKED. Include FILES_CHANGED: and TESTS_RUN: lines when the original task required them.";

pub fn message_starts_with_status(message: &str) -> bool {
    message
        .trim_start_matches(|ch: char| ch.is_whitespace() || ch == '`')
        .to_ascii_uppercase()
        .starts_with("STATUS:")
}

pub fn salvage_final_status_message(message: &str) -> Option<String> {
    if message_starts_with_status(message) {
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
            if upper_line.starts_with("STATUS:") {
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
    status_repair_attempted: bool,
) -> bool {
    !status_repair_attempted
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
    let trimmed = message.trim_start_matches(|ch: char| ch.is_whitespace() || ch == '`');
    let upper = trimmed.to_ascii_uppercase();
    if upper.starts_with("STATUS:") {
        return true;
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
}
