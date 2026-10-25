use anyhow::Result;

use super::state::TurnState;
use crate::agent::events::broadcast_session_event;
use crate::agent::models::AgentSessionEvent;
use crate::agent::repository;

pub enum GateResult {
    Continue,
    Pass,
}

pub fn evaluate_turn_gates(
    state: &mut TurnState,
    final_content: &str,
    allowed_folders: &[String],
    detected_verifier: Option<&str>,
) -> GateResult {
    // 1. If the model returned no conversational content to the user yet
    if final_content.trim().is_empty() && state.iteration < state.max_safety_iterations {
        state.ephemeral_directive = Some(
            "You have finished running tools or inspecting the workspace. Now provide a clear, comprehensive summary of the work done, code changes, or answers to the user's request.".to_string(),
        );
        return GateResult::Continue;
    }

    // 2. Action Gate:
    // Only require code edits if there was a local code/compiler failure or tool execution error.
    // If the failure was purely an external environment, network, port, or missing binary issue
    // and no workspace code files are broken, allow the agent to explain the environment problem.
    let is_pure_env_failure = state.last_command_is_environment_error && state.pending_error_files.is_empty();
    let must_apply_changes = (state.last_command_had_error || state.last_tool_had_error)
        && !is_pure_env_failure
        && !state.circuit_breaker_triggered
        && !state.force_final_text_only;

    if must_apply_changes && state.modified_files.is_empty() && state.iteration < state.max_safety_iterations - 2 {
        state.ephemeral_directive = Some(
            "CRITICAL REJECTION — UNRESOLVED CODE ERROR WITHOUT REPAIR: A terminal command or tool execution failed with compiler/code errors, but you have NOT successfully modified any files to fix it! You cannot conclude without applying the fix. You MUST invoke `workspace_edit_file` or `workspace_write_file` NOW to resolve the issue on disk.".to_string()
        );
        return GateResult::Continue;
    }

    // 3. Hard Verification Enforcement Gate:
    let has_failing_checks = (!state.circuit_breaker_triggered && !state.force_final_text_only) && (
        (!state.has_passed_verification && state.verification_attempts > 0 && !is_pure_env_failure)
            || (!is_pure_env_failure && (state.last_command_had_error || state.last_tool_had_error))
    );
    if has_failing_checks && state.iteration < state.max_safety_iterations - 2 {
        state.ephemeral_directive = Some(
            "CRITICAL REJECTION — FAILING CHECKS OR UNRESOLVED ERRORS: A verification check, terminal command, or edit operation failed! You are strictly forbidden from concluding or claiming success while errors remain. Inspect the error messages, fix the code using `workspace_edit_file` or `workspace_write_file`, and re-run verification until all checks pass cleanly.".to_string()
        );
        return GateResult::Continue;
    }

    // 4. Verification Gate (Adaptive / Soft Gate):
    // Only prompt for verification if:
    // a) Files were modified,
    // b) Automated verification commands are actually available in the project,
    // c) The modified files are not exclusively non-executable assets (e.g. docs, configs, markup).
    let has_detected_verifiers = detected_verifier.is_some() || {
        let relevant = crate::agent::languages::detect_verifiers_for_modified_files(allowed_folders, &state.modified_files);
        !relevant.is_empty()
    };

    let is_task_or_checklist_file = |path: &str| -> bool {
        let lower = path.to_lowercase();
        (lower.ends_with(".md") || lower.ends_with(".txt"))
            && (lower.contains("task") || lower.contains("todo") || lower.contains("checklist") || lower.contains("plan"))
    };

    let has_task_or_checklist_mod = state.modified_files.iter().any(|f| is_task_or_checklist_file(f));

    let are_all_docs_or_configs = !state.modified_files.is_empty()
        && !has_task_or_checklist_mod
        && state.modified_files.iter().all(|f| {
            let lower = f.to_lowercase();
            lower.ends_with(".md")
                || lower.ends_with(".txt")
                || lower.ends_with(".json")
                || lower.ends_with(".yml")
                || lower.ends_with(".yaml")
                || lower.ends_with(".toml")
                || lower.ends_with(".css")
                || lower.ends_with(".scss")
                || lower.ends_with(".html")
                || lower.ends_with(".svg")
                || lower.ends_with(".env")
                || lower.contains(".env.")
                || lower.ends_with(".lock")
                || lower.ends_with(".gitignore")
        });

    if has_task_or_checklist_mod
        && !state.has_passed_verification
        && !state.verification_prompted
        && !state.circuit_breaker_triggered
        && !state.force_final_text_only
        && state.iteration < state.max_safety_iterations - 1
    {
        state.verification_prompted = true;
        let relevant_verifiers = crate::agent::languages::detect_verifiers_for_modified_files(allowed_folders, &state.modified_files);
        let verifier_hint = if !relevant_verifiers.is_empty() {
            format!("run verification (`{}`) and ", relevant_verifiers.join(" && "))
        } else if let Some(pv) = detected_verifier {
            format!("run verification (`{}`) and ", pv)
        } else {
            String::new()
        };
        state.ephemeral_directive = Some(format!(
            "DELIVERABLE GROUND-TRUTH VERIFICATION: You updated a task/checklist file ({}). Before claiming complete status, {}physically verify that all requested deliverables and files exist and contain actual functional implementations (use `workspace_list_dir` or `workspace_read_file`). Never mark items complete without verifying the actual code on disk.",
            state.modified_files.join(", "),
            verifier_hint
        ));
        return GateResult::Continue;
    }

    if !state.modified_files.is_empty()
        && has_detected_verifiers
        && !are_all_docs_or_configs
        && !state.has_passed_verification
        && !state.verification_prompted
        && !state.circuit_breaker_triggered
        && !state.force_final_text_only
        && state.iteration < state.max_safety_iterations - 1
    {
        state.verification_prompted = true;
        let relevant_verifiers = crate::agent::languages::detect_verifiers_for_modified_files(allowed_folders, &state.modified_files);
        let verifier_hint = if !relevant_verifiers.is_empty() {
            relevant_verifiers.join(" && ")
        } else if let Some(pv) = detected_verifier {
            pv.to_string()
        } else {
            "appropriate test or build commands".to_string()
        };
        state.ephemeral_directive = Some(format!(
            "VERIFICATION RECOMMENDATION: You modified {} file(s) ({}). Run `{}` using `workspace_run_command` to verify that there are no compilation or test errors. If verification is not applicable, you may proceed to conclude your summary.",
            state.modified_files.len(),
            state.modified_files.join(", "),
            verifier_hint
        ));
        return GateResult::Continue;
    }

    GateResult::Pass
}

pub fn compose_final_turn_content(
    state: &TurnState,
    final_content: &str,
    final_thought: &str,
) -> String {
    let generated_summary = if !state.modified_files.is_empty() {
        let mut summary_lines = Vec::new();
        summary_lines.push("### 📋 Successfully Applied File Modifications:".to_string());
        for f in &state.modified_files {
            summary_lines.push(format!("- `{}`", f));
        }
        Some(summary_lines.join("\n"))
    } else {
        None
    };

    let is_unresolved = (state.last_command_had_error && !state.last_command_is_environment_error)
        || state.last_tool_had_error
        || (state.verification_attempts > 0 && !state.has_passed_verification && !state.last_command_is_environment_error);

    if is_unresolved {
        let error_details = if state.last_command_had_error {
            "Terminal verification/build command failed with code/compiler errors."
        } else if state.last_tool_had_error {
            "A file modification or tool execution failed."
        } else {
            "Required verification check has not passed cleanly."
        };
        format!(
            "⚠️ **Execution Ended with Unresolved Errors**\n\n{}\n\n{}",
            error_details,
            if final_content.trim().is_empty() {
                "Please review the error outputs above or provide guidance."
            } else {
                final_content.trim()
            }
        )
    } else if state.last_command_is_environment_error {
        format!(
            "⚠️ **Environment / Command Execution Notice**\n\nCommand exited with an external environment or configuration error:\n\n{}",
            if final_content.trim().is_empty() {
                "Please check system dependencies, environment variables, or service status."
            } else {
                final_content.trim()
            }
        )
    } else {
        match (final_content.trim().is_empty(), generated_summary.as_ref()) {
            (false, Some(summary)) => format!("{}\n\n{}", final_content.trim(), summary),
            (false, None) => final_content.trim().to_string(),
            (true, Some(summary)) => summary.clone(),
            (true, None) if !final_thought.trim().is_empty() => final_thought.trim().to_string(),
            (true, None) => "I have completed the operations and workspace inspection as requested.".to_string(),
        }
    }
}

pub async fn handle_iteration_limit_fallback(
    session_id: &str,
    state: &TurnState,
) -> Result<String> {
    let fallback_summary = if !state.modified_files.is_empty() {
        format!(
            "### ⚠️ Iteration Limit Reached — Partial Changes Applied\n\nThe following files were updated in this turn:\n{}\n\nSome sections or files may still need further modifications. Please review the current workspace changes or prompt me to continue with the remaining parts.",
            state.modified_files.iter().map(|f| format!("- `{}`", f)).collect::<Vec<_>>().join("\n")
        )
    } else {
        format!(
            "### ⚠️ Completed Without File Modifications\n\nNo files were modified in this turn.\n- Commands executed: {}\n\nPlease provide specific instructions if there are components or lines you want to change.",
            if state.executed_commands.is_empty() { "None".to_string() } else { state.executed_commands.join(", ") }
        )
    };

    let final_fallback_msg = repository::create_message(
        session_id,
        "assistant",
        Some(&fallback_summary),
        None,
        None,
        None,
        "success",
        None,
        None,
    )
    .await?;

    let final_fallback_status = if !state.modified_files.is_empty() {
        "in_progress"
    } else {
        "failed"
    };
    let _ = repository::conclude_goal(&state.goal_id, final_fallback_status, &fallback_summary).await;

    broadcast_session_event(
        session_id,
        AgentSessionEvent::MessageCreated(final_fallback_msg),
    );

    let _ = repository::update_session_status(session_id, "idle").await;
    broadcast_session_event(
        session_id,
        AgentSessionEvent::StatusChange {
            status: "idle".to_string(),
        },
    );
    broadcast_session_event(
        session_id,
        AgentSessionEvent::Done {
            reply: fallback_summary.clone(),
        },
    );

    Ok(fallback_summary)
}
