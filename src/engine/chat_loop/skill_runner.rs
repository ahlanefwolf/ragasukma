use std::sync::Arc;

use crate::audio::tts::TextToSpeech;
use crate::engine::actions::execute_skill_by_identifier;
use crate::llm::LlmClient;

static RE_SKILL: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"\[(?:SKILL|CMD):([^\]]+)\]").expect("valid skill regex")
});

fn is_background_action_result(output: &str) -> bool {
    let lower = output.to_lowercase();
    lower.contains("in background") || lower.contains("running in background")
}

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

    // If assistant already spoke in Turn 1, only trigger Turn 2 if an action failed
    // or if a tool produced substantive query results (e.g. search, recipe, data extraction).
    skill_results.iter().any(|(name, out, success)| {
        if !success {
            return true;
        }

        let lower = name.to_lowercase();
        let out_lower = out.to_lowercase();

        // Background tasks (e.g. downloading songs, visual screen scanning) report their
        // completion asynchronously via [SYSTEM STATUS: ...], so do NOT trigger Turn 2.
        if is_background_action_result(out) {
            return false;
        }

        // Media controls (play, pause, volume, etc.) already took effect; Turn 1 already spoke.
        if lower.contains("music") || lower.contains("media") || lower.contains("volume") {
            return false;
        }

        // Simple desktop interactions (focus, click, minimize, maximize, type, keypress)
        // have already been executed; Turn 1 already spoke.
        if lower.contains("click")
            || lower.contains("focus")
            || lower.contains("minimize")
            || lower.contains("maximize")
            || lower.contains("keypress")
            || lower.contains("input")
        {
            return false;
        }

        lower.contains("recipe")
            || lower.contains("todo")
            || lower.contains("search")
            || lower.contains("status")
            || lower.contains("browse")
            || out.lines().count() > 1
            || out.len() > 80
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
    let mut executed_ids = std::collections::HashSet::new();
    handle_skill_execution_inner(llm, tts, asst_name, full_reply, effective_input, 0, &mut executed_ids);
}

fn handle_skill_execution_inner(
    llm: &mut LlmClient,
    tts: &Arc<TextToSpeech>,
    asst_name: &str,
    full_reply: &str,
    effective_input: &str,
    depth: usize,
    executed_ids: &mut std::collections::HashSet<String>,
) {
    let mut skill_results: Vec<(String, String, bool)> = Vec::new();

    // Universal normalization: map any raw tool-call blocks (<dots_function_call>, <invoke>, etc.)
    // to canonical [CMD:...] so commands from any LLM model run reliably.
    let normalized_reply = crate::llm::cleaner::normalize_tool_calls_to_commands(full_reply);

    for cap in RE_SKILL.captures_iter(&normalized_reply) {
        if let Some(matched_id) = cap.get(1) {
            let id = matched_id.as_str().trim().to_string();

            // Automatic guardrail: If this response was triggered by internal SYSTEM STATUS notification,
            // prevent repeated media:play or desktop:scan_visual calls.
            if effective_input.starts_with("[SYSTEM STATUS:")
                && (id.starts_with("media:play")
                    || id.starts_with("desktop:scan_visual")
                    || id.starts_with("vision:"))
            {
                crate::term_log!(
                    "[Duplicate Skill Guard]: Ignoring '{}' triggered by SYSTEM STATUS notification",
                    id
                );
                continue;
            }

            // Prevent executing the exact same command multiple times in the same ReAct cycle
            if executed_ids.contains(&id) {
                crate::term_log!(
                    "[Duplicate Skill Guard]: Ignoring '{}' because it was already executed in this interaction",
                    id
                );
                continue;
            }
            executed_ids.insert(id.clone());

            crate::term_log!("\n[SKILL]: Invoking ID '{}'...", id);

            if let Some((skill_name, output)) = execute_skill_by_identifier(&id) {
                let clean_out = output.trim().to_string();
                crate::term_log!(
                    "[SKILL SUCCESS: {} ({})] -> {}",
                    skill_name,
                    id,
                    clean_out
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

                skill_results.push((id.clone(), err_msg, false));
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

            if skill_results.iter().any(|(_, out, _)| is_background_action_result(out)) {
                observation.push_str("NOTE: The background action has been started and is currently running asynchronously. Inform the user briefly that it is in progress. Do NOT claim the song is already playing or that the scan is finished. Do NOT emit any command tags.");
            } else {
                observation.push_str("Summarize or report this action result naturally and concisely to the user. Do not repeat raw command tags.");
            }

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
                    handle_skill_execution_inner(
                        llm,
                        tts,
                        asst_name,
                        &followup_reply,
                        &observation,
                        depth + 1,
                        executed_ids,
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_background_actions_do_not_trigger_turn_2_if_turn_1_spoke() {
        // 1. Music download in background
        let results = vec![(
            "Play Music".to_string(),
            "Downloading & playing song 'Pujaan Hati Kangen Band' in the background".to_string(),
            true,
        )];
        assert!(!should_trigger_react_followup(true, &results));

        // But if Turn 1 had NO speech, it should trigger Turn 2 so user gets informed
        assert!(should_trigger_react_followup(false, &results));

        // 2. Screen visual scan in background
        let scan_results = vec![(
            "Scan Screen".to_string(),
            "Visual screen scan running in background.".to_string(),
            true,
        )];
        assert!(!should_trigger_react_followup(true, &scan_results));
        assert!(should_trigger_react_followup(false, &scan_results));
    }

    #[test]
    fn test_media_and_desktop_controls_skip_turn_2_when_already_spoken() {
        let media_res = vec![(
            "Play Music".to_string(),
            "Playing local music: Test Song".to_string(),
            true,
        )];
        assert!(!should_trigger_react_followup(true, &media_res));

        let click_res = vec![(
            "Click Desktop Element".to_string(),
            "Element/text 'Recipes' clicked".to_string(),
            true,
        )];
        assert!(!should_trigger_react_followup(true, &click_res));
    }

    #[test]
    fn test_failed_actions_and_substantive_tools_trigger_turn_2() {
        // Failed action must report error
        let failed_res = vec![(
            "Scan Screen".to_string(),
            "Failed to access AT-SPI accessibility tree".to_string(),
            false,
        )];
        assert!(should_trigger_react_followup(true, &failed_res));

        // Recipe or search with results should report to user
        let recipe_res = vec![(
            "Recipe Execution".to_string(),
            "Found 3 items:\n1. Laptop A\n2. Laptop B\n3. Laptop C".to_string(),
            true,
        )];
        assert!(should_trigger_react_followup(true, &recipe_res));
    }
}
