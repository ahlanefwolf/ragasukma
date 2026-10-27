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
    if final_content.trim().is_empty() {
        state.consecutive_empty_turns += 1;
        if state.consecutive_empty_turns <= 1 && state.iteration < state.max_safety_iterations {
            state.ephemeral_directive = Some(
                "You have completed workspace operations. Provide a concise summary of the work done or directly answer the user's inquiry.".to_string(),
            );
            return GateResult::Continue;
        }
        return GateResult::Pass;
    } else {
        state.consecutive_empty_turns = 0;
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

    let has_substantive_response = final_content.trim().len() >= 40;

    if must_apply_changes && state.modified_files.is_empty() && !has_substantive_response && state.iteration < state.max_safety_iterations - 2 {
        state.ephemeral_directive = Some(
            "Unresolved code error: A command or tool reported errors and no files were modified. If a fix is needed, apply it using python3 via `workspace_run_command` or `workspace_write_file`. If explaining the issue, provide your complete analysis to the user.".to_string()
        );
        return GateResult::Continue;
    }

    // 3. Hard Verification Enforcement Gate:
    let has_failing_checks = (!state.circuit_breaker_triggered && !state.force_final_text_only) && (
        (!state.has_passed_verification && state.verification_attempts > 0 && !is_pure_env_failure)
            || (!is_pure_env_failure && (state.last_command_had_error || state.last_tool_had_error))
    );
    if has_failing_checks && !has_substantive_response && state.iteration < state.max_safety_iterations - 2 {
        state.ephemeral_directive = Some(
            "Failing checks or unresolved errors remain. Inspect the error messages and fix the code using python3 via `workspace_run_command` or `workspace_write_file`, or explain the findings to the user.".to_string()
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
            "Deliverable verification: You updated a task/checklist file ({}). Before claiming complete status, {}physically verify that all requested deliverables and files exist and contain actual functional implementations (use `workspace_list_dir` or `workspace_read_file`). Never mark items complete without verifying the actual code on disk.",
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
    let trimmed_content = final_content.trim();

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

    // If the model provided substantive final content, preserve it directly without prepending alarming banners
    if !trimmed_content.is_empty() {
        return match generated_summary {
            Some(summary) => format!("{}\n\n{}", trimmed_content, summary),
            None => trimmed_content.to_string(),
        };
    }

    // Only if final_content is empty, construct a fallback based on execution state
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
            "⚠️ **Execution Ended with Unresolved Errors**\n\n{}\n\nPlease review the error outputs above or provide guidance.",
            error_details
        )
    } else if state.last_command_is_environment_error {
        "⚠️ **Environment / Command Execution Notice**\n\nCommand exited with an external environment or configuration error. Please check system dependencies, environment variables, or service status.".to_string()
    } else {
        match (trimmed_content.is_empty(), generated_summary.as_ref()) {
            (false, Some(summary)) => format!("{}\n\n{}", trimmed_content, summary),
            (false, None) => trimmed_content.to_string(),
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
    // Check if there are recent assistant messages with substantive partial answers/findings
    let recent_findings = repository::list_messages(session_id).await.ok().and_then(|msgs| {
        msgs.into_iter()
            .rev()
            .find(|m| m.role == "assistant" && m.content.as_ref().map(|c| c.trim().len() >= 40).unwrap_or(false))
            .and_then(|m| m.content)
    });

    let fallback_summary = if !state.modified_files.is_empty() {
        format!(
            "### ⚠️ Iteration Limit Reached — Partial Changes Applied\n\nThe following files were updated in this turn:\n{}\n\nSome sections or files may still need further modifications. Please review the current workspace changes or prompt me to continue with the remaining parts.",
            state.modified_files.iter().map(|f| format!("- `{}`", f)).collect::<Vec<_>>().join("\n")
        )
    } else if let Some(ref findings) = recent_findings {
        format!(
            "{}\n\n---\n*Note: Iteration limit reached for this turn. The findings above reflect workspace analysis completed so far.*",
            findings.trim()
        )
    } else {
        format!(
            "### ⚠️ Iteration Limit Reached\n\nCompleted workspace inspection and operations without modifying files.\n- Commands executed: {}\n\nPlease provide specific instructions if there are components or lines you want to change.",
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
        crate::agent::orchestrator::GoalState::Action
    } else if recent_findings.is_some() {
        crate::agent::orchestrator::GoalState::Completed
    } else {
        crate::agent::orchestrator::GoalState::Failed
    };
    let orch = crate::agent::orchestrator::GoalOrchestrator::new(&state.goal_id, "default");
    let _ = orch.conclude_with_status(final_fallback_status, &fallback_summary).await;

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
