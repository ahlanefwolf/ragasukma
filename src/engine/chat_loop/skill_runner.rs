use std::sync::Arc;

use crate::audio::tts::TextToSpeech;
use crate::engine::actions::execute_skill_by_identifier;
use crate::llm::LlmClient;
use crate::tray::TrayState;

static RE_SKILL: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"\[(?:SKILL|CMD):([^\]]+)\]").expect("valid skill regex")
});

/// Evaluates whether a multi-turn ReAct follow-up is warranted for the executed tools.
fn should_trigger_react_followup(
    turn1_had_speech: bool,
    skill_results: &[(String, String, bool)],
) -> bool {
    if skill_results.is_empty() {
        return false;
    }
    // If the assistant did not produce spoken audio in turn 1 (e.g. emitted only tool commands),
    // a follow-up turn is mandatory so the assistant speaks back to the user.
    if !turn1_had_speech {
        return true;
    }
    skill_results.iter().any(|(name, out, success)| {
        if !success {
            return true;
        }
        let lower = name.to_lowercase();
        let out_lower = out.to_lowercase();
        lower.contains("recipe")
            || lower.contains("todo")
            || lower.contains("scan")
            || lower.contains("search")
            || lower.contains("ocr")
            || lower.contains("status")
            || out.lines().count() > 1
            || out.len() > 60
            || out_lower.contains("error")
            || out_lower.contains("failed")
    })
}

/// Running eksekusi skill tool parsing `[CMD:...]` or `[SKILL:...]`,
/// including 2-step ReAct agentic loop for follow-up desktop/browser actions.
pub fn handle_skill_execution(
    llm: &mut LlmClient,
    tts: &Arc<TextToSpeech>,
    asst_name: &str,
    full_reply: &str,
    effective_input: &str,
) {
    handle_skill_execution_with_depth(llm, tts, asst_name, full_reply, effective_input, 0);
}

pub fn handle_skill_execution_with_depth(
    llm: &mut LlmClient,
    tts: &Arc<TextToSpeech>,
    asst_name: &str,
    full_reply: &str,
    effective_input: &str,
    depth: usize,
) {
    let mut executed_skill_ids = std::collections::HashSet::new();
    let mut skill_results: Vec<(String, String, bool)> = Vec::new();

    // Universal normalization: map any raw tool-call blocks (<dots_function_call>, <invoke>, etc.)
    // to canonical [CMD:...] so commands from any LLM model run reliably.
    let normalized_reply = crate::llm::cleaner::normalize_tool_calls_to_commands(full_reply);

    for cap in RE_SKILL.captures_iter(&normalized_reply) {
        if let Some(matched_id) = cap.get(1) {
            let id = matched_id.as_str().trim().to_string();

            // Automatic guardrail: If this response was triggered by internal SYSTEM STATUS notification,
            // prevent repeated media:play calls that re-trigger tracks that just finished downloading.
            if effective_input.starts_with("[SYSTEM STATUS:") && id.starts_with("media:play") {
                crate::term_log!(
                    "[Duplicate Skill Guard]: Ignoring '{}' because the song was already played automatically by the system",
                    id
                );
                continue;
            }

            if !executed_skill_ids.insert(id.clone()) {
                continue;
            }
            crate::term_log!("\n[SKILL]: Invoking ID '{}'...", id);

            if let Some((skill_name, output)) = execute_skill_by_identifier(&id) {
                let clean_out = output.trim().to_string();
                crate::term_log!(
                    "[SKILL SUCCESS: {} ({})] -> {}",
                    skill_name,
                    id,
                    clean_out
                );
                TrayState::emit_event(
                    "assistant:skill",
                    serde_json::json!({
                        "id": id,
                        "name": skill_name,
                        "status": "success",
                        "output": &clean_out
                    }),
                );

                skill_results.push((skill_name, clean_out, true));

                // Update snapshot when switching window or browser
                let id_lower = id.to_lowercase();
                if id_lower == "desktop:scan_visual" {
                    if let Ok(snap) = crate::engine::window_scanner::scan_active_window_sync() {
                        let _ = snap.save_to_file();
                    }
                } else if id_lower.starts_with("desktop:focus") || id_lower.starts_with("app:open")
                {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    if let Ok(snap) = crate::engine::window_scanner::scan_active_window_sync() {
                        let _ = snap.save_to_file();
                    }
                }
            } else {
                let err_msg = if id.starts_with("app:") || id.starts_with("os:") {
                    format!("Application or OS command target for '{id}' could not be launched")
                } else if id.contains(':') {
                    format!("Action '{id}' execution failed or target not found")
                } else {
                    format!("Identifier '{id}' not found in skill-command.md")
                };

                crate::term_err!("[ACTION FAILED]: {err_msg}");
                TrayState::emit_event(
                    "assistant:skill",
                    serde_json::json!({
                        "id": id,
                        "name": id,
                        "status": "failed",
                        "output": &err_msg
                    }),
                );

                skill_results.push((id, err_msg, false));
            }
        }
    }

    // ReAct Loop Turn 2: Synthesize tool observation and trigger natural follow-up response
    if depth < 1 && !skill_results.is_empty() {
        let clean_first_reply = crate::llm::cleaner::clean_ai_text(full_reply);
        let turn1_had_speech = !clean_first_reply.trim().is_empty();

        if should_trigger_react_followup(turn1_had_speech, &skill_results) {
            let mut observation = String::new();
            for (name, out, success) in &skill_results {
                let status_str = if *success { "SUCCESS" } else { "FAILED" };
                observation.push_str(&format!(
                    "[Action: {} | Status: {}]\n{}\n\n",
                    name, status_str, out
                ));
            }
            observation.push_str("Summarize or report this action result naturally and concisely to the user. Do not repeat raw command tags.");

            crate::term_log!("\n[ReAct Loop]: Triggering Turn 2 with tool observations...");
            if let Ok(followup_reply) = crate::engine::chat_loop::llm_responder::process_llm_response_with_role(
                llm,
                tts,
                asst_name,
                "tool",
                &observation,
            ) {
                // If Turn 2 itself emits further action commands, execute them with incremented depth
                if !followup_reply.trim().is_empty() {
                    handle_skill_execution_with_depth(
                        llm,
                        tts,
                        asst_name,
                        &followup_reply,
                        &observation,
                        depth + 1,
                    );
                }
            }
        }
    }
}
