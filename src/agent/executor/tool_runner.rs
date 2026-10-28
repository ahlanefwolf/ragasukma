use anyhow::Result;
use serde_json::json;

use super::state::TurnState;
use crate::agent::compressor::{sanitize_tool_output_for_db, sanitize_tool_output_for_memory};
use crate::agent::diagnostics::{
    extract_command_diagnostics, group_diagnostics_by_file, is_command_output_error,
};
use crate::agent::events::broadcast_session_event;
use crate::agent::models::{AgentSessionEvent, Message};
use crate::agent::repository;
use crate::agent::tools::execute_tool_call;
use crate::llm::v1_engine::V1ToolCall;

pub async fn execute_turn_tool_calls(
    session_id: &str,
    project_id: &str,
    allowed_folders: &[String],
    mcp_tools: &[crate::agent::mcp::ActiveMcpTool],
    tool_calls: &[V1ToolCall],
    state: &mut TurnState,
    db_messages: &mut Vec<Message>,
) -> Result<()> {
    for tc in tool_calls {
        broadcast_session_event(
            session_id,
            AgentSessionEvent::StepInfo {
                iteration: state.iteration,
                action: format!("tool:{}", tc.function.name),
                details: Some(format!("Executing `{}`...", tc.function.name)),
            },
        );

        let args_val = serde_json::from_str::<serde_json::Value>(&tc.function.arguments)
            .unwrap_or_else(|_| json!({}));

        let is_edit_op = tc.function.name == "workspace_write_file"
            || tc.function.name == "workspace_edit_file";

        let is_inspection = tc.function.name == "workspace_read_file"
            || tc.function.name == "workspace_grep"
            || tc.function.name == "workspace_list_dir"
            || (tc.function.name == "workspace_run_command" && {
                let cmd = args_val.get("command").and_then(|c| c.as_str()).unwrap_or("").trim().to_lowercase();
                cmd.starts_with("cat ") || cmd.starts_with("ls") || cmd.starts_with("head ") || cmd.starts_with("tail ") || cmd.starts_with("grep ") || cmd.starts_with("find ")
            });

        let target_arg_file = args_val
            .get("file_path")
            .or_else(|| args_val.get("path"))
            .and_then(|p| p.as_str())
            .map(|s| s.to_string());

        if is_edit_op {
            if let Some(ref path) = target_arg_file {
                let clean = path.trim_start_matches("./").to_string();
                crate::agent::diagnostics::mark_diagnostic_file_resolved(session_id, &clean);
            }
        }

        let is_target_in_error_list = target_arg_file.as_ref().map(|p| {
            let clean = p.trim_start_matches("./");
            state.pending_error_files.iter().any(|ef| ef == clean || clean.ends_with(ef) || ef.ends_with(clean))
        }).unwrap_or(false);

        let is_accessing_unmodified_target = target_arg_file.as_ref().map(|p| {
            let clean = p.trim_start_matches("./");
            !state.modified_files.iter().any(|mf| {
                let mf_clean = mf.trim_start_matches("./");
                mf_clean == clean || mf_clean.ends_with(clean) || clean.ends_with(mf_clean)
            })
        }).unwrap_or(false);

        let is_work_on_new_target = (tc.function.name == "workspace_read_file"
            || tc.function.name == "workspace_edit_file"
            || tc.function.name == "workspace_write_file")
            && is_accessing_unmodified_target;

        let is_post_verification_terminal_command = state.has_passed_verification
            && !state.last_command_had_error
            && state.pending_error_files.is_empty()
            && tc.function.name == "workspace_run_command"
            && {
                let cmd = args_val.get("command").and_then(|c| c.as_str()).unwrap_or("").trim().to_lowercase();
                cmd.starts_with("git ") || cmd.contains("git commit") || cmd.contains("git add") || state.executed_commands.iter().any(|ec| ec.contains(&cmd) || cmd.contains(ec))
            };

        let is_repeated_command = tc.function.name == "workspace_run_command" && {
            let cmd = args_val.get("command").and_then(|c| c.as_str()).unwrap_or("").trim();
            !cmd.is_empty() && state.last_executed_command.as_deref() == Some(cmd) && state.consecutive_command_count >= 1
        };

        // HARD GATES & CIRCUIT BREAKERS
        let mut is_breaker_trip = false;
        let tool_output = if state.circuit_breaker_triggered || state.force_final_text_only {
            is_breaker_trip = true;
            "Tool execution rejected: Tool calling is closed for this turn. Conclude your turn immediately and provide your summary to the user now.".to_string()
        } else if state.has_passed_verification && !state.last_command_had_error && state.pending_error_files.is_empty() && !is_work_on_new_target && (tc.function.name != "workspace_run_command" || is_post_verification_terminal_command) {
            is_breaker_trip = true;
            "Verification has already succeeded with 0 errors. All active errors resolved. Tool calling is now closed. Conclude your turn immediately and provide your summary to the user now.".to_string()
        } else if is_repeated_command {
            is_breaker_trip = true;
            let cmd = args_val.get("command").and_then(|c| c.as_str()).unwrap_or("").trim();
            format!(
                "Command repetition limit reached for `{cmd}`. This command was already executed without subsequent file modifications. Tool calling is now closed. Conclude your turn now and report the results directly to the user."
            )
        } else if is_inspection && !is_target_in_error_list && state.global_inspection_streak >= 10 {
            is_breaker_trip = true;
            format!(
                "Inspection budget reached ({} consecutive reads without code modifications). Tool calling is closed to prevent unproductive wandering. Conclude your turn now and provide your findings, answers, or plan directly to the user.",
                state.global_inspection_streak
            )
        } else if tc.function.name == "workspace_read_file" && {
            let start_line = args_val.get("start_line").and_then(|v| v.as_i64());
            let end_line = args_val.get("end_line").and_then(|v| v.as_i64());
            let path = target_arg_file.clone().unwrap_or_default();
            let is_exact_duplicate = state.recent_read_slices.contains(&(path.clone(), start_line, end_line));
            let same_file_count = target_arg_file.as_ref().map(|p| state.consecutive_read_same_file.get(p).copied().unwrap_or(0)).unwrap_or(0);
            is_exact_duplicate && same_file_count >= 5
        } {
            let path = target_arg_file.as_deref().unwrap_or("target");
            format!(
                "Notice: You have already read this exact slice of `{path}` multiple times. Its content is available in your conversation history. Please proceed with modifying the file using `workspace_edit_file` or answering the user's question directly."
            )
        } else if tc.function.name.starts_with("mcp__") {
            // Dispatch to active MCP server
            match crate::agent::mcp::call_active_mcp_tool(
                mcp_tools,
                &tc.function.name,
                args_val.clone(),
            )
            .await
            {
                Ok(out) => out,
                Err(e) => format!("MCP execution error: {}", e),
            }
        } else {
            match execute_tool_call(
                &tc.function.name,
                &args_val,
                allowed_folders,
            ).await {
                Ok(out) => {
                    if tc.function.name == "workspace_read_file" {
                        if let Some(ref p) = target_arg_file {
                            let clean = p.trim_start_matches("./").to_string();
                            let start_line = args_val.get("start_line").and_then(|v| v.as_i64());
                            let end_line = args_val.get("end_line").and_then(|v| v.as_i64());
                            let is_outline = out.starts_with("# File outline for");
                            if start_line.is_none() && end_line.is_none() && !is_outline {
                                state.files_read_in_full.insert(clean);
                            }
                            state.recent_read_slices.insert((p.clone(), start_line, end_line));
                        }
                    }
                    out
                }
                Err(e) => format!("Tool execution error: {}", e),
            }
        };

        if is_breaker_trip {
            state.trip_circuit_breaker(
                "CIRCUIT BREAKER TRIGGERED: Tool calling is permanently closed for this turn. Do NOT attempt to invoke any more tools. Provide your final response, summary, or answer to the user now."
            );
        }

        if is_inspection {
            state.global_inspection_streak += 1;
            if state.current_phase == "discovery" {
                state.discovery_counter = repository::increment_goal_discovery(&state.goal_id).await.unwrap_or(state.discovery_counter + 1);
                if let Some(ref path) = target_arg_file {
                    let is_dir_inspection = tc.function.name == "workspace_list_dir"
                        || path.ends_with('/')
                        || !path.contains('.');
                    if !is_dir_inspection && !state.locked_targets.contains(path) {
                        state.locked_targets.push(path.clone());
                    }
                }
            }
        }

        // Auto-invalidation and tracking on file modification
        if is_edit_op {
            let is_edit_failed = tool_output.starts_with("Tool execution error")
                || tool_output.starts_with("Phase Gate")
                || tool_output.contains("'old_text' not found")
                || tool_output.contains("failed to apply edit");

            if is_edit_failed {
                state.last_tool_had_error = true;
            } else {
                if let Some(ref path) = target_arg_file {
                    state.on_successful_edit(path);
                    let _ = repository::mark_evidence_stale_by_file(project_id, path).await;
                }

                let targets_json = serde_json::to_string(&state.locked_targets).unwrap_or_else(|_| "[]".to_string());
                let _ = repository::lock_goal_targets(&state.goal_id, &targets_json).await;
                let _ = repository::update_goal_phase(&state.goal_id, "verification").await;

                let relevant_verifiers = crate::agent::languages::detect_verifiers_for_modified_files(allowed_folders, &state.modified_files);
                let primary_verify = if !relevant_verifiers.is_empty() {
                    relevant_verifiers.join(" && ")
                } else {
                    crate::agent::languages::detect_primary_verification_command(allowed_folders)
                        .unwrap_or_else(|| "test/build runner".to_string())
                };
                state.ephemeral_directive = Some(
                    format!(
                        "ACTION EXECUTED: Target file updated successfully. You are now in VERIFICATION phase. Run the verification command using `workspace_run_command` (e.g. `{}`) to verify your changes.",
                        primary_verify
                    )
                );
            }
        } else if tc.function.name == "workspace_run_command" {
            state.global_inspection_streak = 0;
            if let Some(cmd) = args_val.get("command").and_then(|c| c.as_str()) {
                let trimmed_cmd = cmd.trim();
                if state.last_executed_command.as_deref() == Some(trimmed_cmd) {
                    state.consecutive_command_count += 1;
                } else {
                    state.consecutive_command_count = 1;
                    state.last_executed_command = Some(trimmed_cmd.to_string());
                }

                if !state.executed_commands.contains(&trimmed_cmd.to_string()) {
                    state.executed_commands.push(trimmed_cmd.to_string());
                }

                let is_err = is_command_output_error(&tool_output);

                if is_err {
                    state.last_command_had_error = true;
                    state.has_passed_verification = false;

                    let (grouped_diags, general_errs) = group_diagnostics_by_file(&tool_output);
                    let is_env_err = crate::agent::diagnostics::is_environment_error(&tool_output, &grouped_diags);
                    state.last_command_is_environment_error = is_env_err;

                    if !is_env_err {
                        for g in &grouped_diags {
                            state.pending_error_files.insert(g.file_path.clone());
                        }
                    }

                    // Populate deterministic Diagnostic Queue in RAM for this session
                    let parsed_items = crate::agent::diagnostics::parse_diagnostic_items_from_output(&tool_output);
                    if !parsed_items.is_empty() {
                        crate::agent::diagnostics::set_session_diagnostic_queue(
                            session_id,
                            parsed_items,
                            Some(cmd.to_string()),
                        );
                    }

                    let raw_compiler_head: String = tool_output
                        .lines()
                        .take(45)
                        .collect::<Vec<_>>()
                        .join("\n");

                    let mut diag_bulletin = String::new();

                    if !grouped_diags.is_empty() {
                        diag_bulletin.push_str("\n\n🚨 TARGET FILES WITH COMPILER ERRORS (Batch-edit these files):\n");
                        for g in grouped_diags.iter().take(12) {
                            let lines_str = if !g.lines.is_empty() {
                                format!(" (lines: {})", g.lines.iter().map(|l| l.to_string()).collect::<Vec<_>>().join(", "))
                            } else {
                                String::new()
                            };
                            diag_bulletin.push_str(&format!("- `{}` -> {} error(s){}\n", g.file_path, g.error_count, lines_str));
                            for s in &g.sample_messages {
                                diag_bulletin.push_str(&format!("    * {}\n", s));
                            }
                        }
                    } else {
                        let extracted_diags = extract_command_diagnostics(&tool_output);
                        if !extracted_diags.is_empty() {
                            diag_bulletin.push_str(&format!(
                                "\n\n🚨 EXTRACTED COMPILER / RUNTIME DIAGNOSTICS:\n{}\n",
                                extracted_diags.iter().map(|d| format!("- {}", d)).collect::<Vec<_>>().join("\n")
                            ));
                        }
                    }

                    if !general_errs.is_empty() {
                        diag_bulletin.push_str(&format!("General: {}\n", general_errs.join(" | ")));
                    }

                    state.ephemeral_directive = Some(format!(
                        "🚨 COMPILATION / TEST FAILURE DETECTED for command `{cmd}`!\n\n\
                         RAW COMPILER ERROR OUTPUT:\n```\n{raw_compiler_head}\n```\
                         {diag_bulletin}\n\n\
                         CRITICAL BATCH-ACTION REQUIRED:\n\
                         1. Read the target files listed above in full (or use the lines indicated).\n\
                         2. Apply ALL fixes for each target file using `workspace_edit_file` (or `workspace_write_file`).\n\
                         3. DO NOT re-run compiler or verification commands until ALL broken target files have been edited!\n\
                         4. Once all target files are updated, run `{cmd}` to verify."
                    ));
                } else {
                    // Check if command modified files (e.g. python script with open(..., 'w'), touch, etc.)
                    let touched = extract_modified_paths_from_command(trimmed_cmd);
                    for path in touched {
                        state.on_successful_edit(&path);
                        let _ = repository::mark_evidence_stale_by_file(project_id, &path).await;
                    }
                }

                if crate::agent::languages::is_verification_command(cmd) {
                    let was_rejected = is_breaker_trip || tool_output.starts_with("Tool execution rejected");
                    let is_passed = !is_err && !was_rejected;
                    state.verification_attempts += 1;
                    let _ = repository::record_verification(
                        &state.goal_id,
                        state.verification_attempts as i64,
                        cmd,
                        &tool_output,
                        is_passed,
                        if is_passed { None } else { Some(&tool_output) },
                    ).await;

                    if is_passed {
                        state.has_passed_verification = true;
                        state.last_command_had_error = false;
                        state.pending_error_files.clear();
                        state.current_phase = "concluded".to_string();
                        state.force_final_text_only = true;
                        let _ = repository::update_goal_phase(&state.goal_id, "concluded").await;
                        let _ = repository::update_goal_status(&state.goal_id, "completed").await;
                        state.ephemeral_directive = Some(
                            "VERIFICATION PASSED: The build/test command succeeded with 0 errors! All active tasks and verifications are complete. Conclude your turn now with a clear summary for the user without calling any more tools.".to_string()
                        );
                    } else {
                        state.has_passed_verification = false;
                        state.current_phase = "action".to_string();
                        let _ = repository::update_goal_phase(&state.goal_id, "action").await;

                        let (grouped_diags, _) = group_diagnostics_by_file(&tool_output);
                        for g in &grouped_diags {
                            state.pending_error_files.insert(g.file_path.clone());
                        }
                        let mut diag_bulletin = String::new();
                        if !grouped_diags.is_empty() {
                            diag_bulletin.push_str("\n\n🚨 VERIFICATION FAILURE TARGET FILES:\n");
                            for g in grouped_diags.iter().take(10) {
                                diag_bulletin.push_str(&format!("- `{}` ({} errors at lines {:?})\n", g.file_path, g.error_count, g.lines));
                            }
                        }

                        state.ephemeral_directive = Some(format!(
                            "VERIFICATION FAILED: Verification command `{cmd}` reported errors.{diag_bulletin}\n\nACTION REQUIRED: Read the failing files, modify them using `workspace_edit_file` or `workspace_write_file`, and re-verify before concluding."
                        ));
                    }
                }
            }
        }

        // If edit failed due to text mismatch or duplicate match, guide model directly back to target
        if tc.function.name == "workspace_edit_file" {
            if tool_output.contains("'old_text' not found") {
                state.ephemeral_directive = Some(
                    "Edit recovery: `workspace_edit_file` failed because 'old_text' did not match the file on disk. Inspect the exact slice using `workspace_read_file` if needed, and re-call `workspace_edit_file` with matching lines and 2-3 lines of surrounding context.".to_string()
                );
            } else if tool_output.contains("matches ") || tool_output.contains("matched ") {
                state.ephemeral_directive = Some(
                    "Edit recovery: 'old_text' matched multiple locations in the file. Include the enclosing block, function declaration, or 2-3 additional surrounding lines in 'old_text' to make it unique, and re-call `workspace_edit_file`.".to_string()
                );
            }
        }

        // Check inspection streaks (advise model to apply changes or conclude)
        if state.global_inspection_streak >= 5 {
            state.ephemeral_directive = Some(format!(
                "Notice: You have inspected {} files without modifying code. Discovery budget is almost exhausted. Cease exploratory reads, focus on the identified files, and apply modifications using `workspace_edit_file` (or `workspace_write_file`), or formulate your direct answer to the user.",
                state.global_inspection_streak
            ));
        }

        if state.has_passed_verification && is_inspection && !is_accessing_unmodified_target {
            state.ephemeral_directive = Some(
                "STOP: Verification has already passed for modified files! Do not perform redundant inspections. Conclude your response or proceed directly to edit remaining target files.".to_string()
            );
        }

        let is_file_inspection = tc.function.name == "workspace_read_file" || tc.function.name == "workspace_grep";
        if is_file_inspection {
            if let Some(path) = target_arg_file.as_deref() {
                let start_line = args_val.get("start_line").and_then(|v| v.as_i64());
                let end_line = args_val.get("end_line").and_then(|v| v.as_i64());
                let is_exact_dup = state.recent_read_slices.contains(&(path.to_string(), start_line, end_line));
                if is_exact_dup || (start_line.is_none() && end_line.is_none()) {
                    let count = state.consecutive_read_same_file.entry(path.to_string()).or_insert(0);
                    *count += 1;
                    if *count >= 5 {
                        state.ephemeral_directive = Some(format!(
                            "Notice: You have inspected `{path}` {count} times this turn. If your task requires code modifications, call `workspace_edit_file` now. If answering a question, respond directly to the user."
                        ));
                    }
                }
            }
        }

        // Auto-cache discovered file contents into Evidence Cache with true SHA-256 version
        if tc.function.name == "workspace_read_file" && !tool_output.starts_with("Tool execution error") {
            if let Some(file_path) = target_arg_file.as_deref() {
                let start_line = args_val.get("start_line").and_then(|v| v.as_i64());
                let end_line = args_val.get("end_line").and_then(|v| v.as_i64());
                let preview_summary = format!("Read {} (lines {:?}-{:?})", file_path, start_line.unwrap_or(1), end_line.unwrap_or(0));
                let file_ver = crate::agent::tools::fs::resolve_allowed_path(file_path, allowed_folders)
                    .ok()
                    .and_then(|p| crate::agent::context_builder::compute_file_version(&p).ok())
                    .unwrap_or_else(|| "live_mtime".to_string());
                let snippet: String = tool_output.chars().take(1500).collect();
                if let Ok(saved_ev) = repository::save_evidence(
                    project_id,
                    file_path,
                    &file_ver,
                    start_line,
                    end_line,
                    None,
                    &preview_summary,
                    Some(&snippet),
                ).await {
                    let _ = repository::link_evidence_to_goal(&state.goal_id, &saved_ev.id, 0.8, true).await;
                }
            }
        } else if is_edit_op && !tool_output.starts_with("Tool execution error") {
            if let Some(file_path) = target_arg_file.as_deref() {
                let preview_summary = format!("Modified/Created file {}", file_path);
                let file_ver = crate::agent::tools::fs::resolve_allowed_path(file_path, allowed_folders)
                    .ok()
                    .and_then(|p| crate::agent::context_builder::compute_file_version(&p).ok())
                    .unwrap_or_else(|| "live_mtime".to_string());
                if let Ok(saved_ev) = repository::save_evidence(
                    project_id,
                    file_path,
                    &file_ver,
                    None,
                    None,
                    None,
                    &preview_summary,
                    None,
                ).await {
                    let _ = repository::link_evidence_to_goal(&state.goal_id, &saved_ev.id, 1.0, true).await;
                }
            }
        }

        // Sanitize large command/terminal output before DB persistence (avoid truncating source code reads)
        let is_cmd_run = tc.function.name == "workspace_run_command";
        let tool_output_for_db = if is_cmd_run {
            sanitize_tool_output_for_db(&tool_output)
        } else {
            tool_output.clone()
        };

        let tool_msg = repository::create_message(
            session_id,
            "tool",
            Some(&tool_output_for_db),
            None,
            None,
            Some(&tc.id),
            "success",
            None,
            None,
        )
        .await?;

        let mut in_memory_tool_msg = tool_msg;
        in_memory_tool_msg.content = Some(if is_cmd_run {
            sanitize_tool_output_for_memory(&tool_output)
        } else {
            tool_output.clone()
        });
        db_messages.push(in_memory_tool_msg);
    }

    Ok(())
}

fn extract_modified_paths_from_command(cmd: &str) -> Vec<String> {
    let mut paths = Vec::new();

    // 1. Python open('path', 'w'/'a') or open("path", "w"/"a")
    if cmd.contains("open(") {
        for part in cmd.split("open(") {
            let trimmed = part.trim_start();
            if trimmed.starts_with('\'') || trimmed.starts_with('"') {
                if let Some(quote) = trimmed.chars().next() {
                    let rest = &trimmed[1..];
                    if let Some((p, post)) = rest.split_once(quote) {
                        if post.contains("'w'") || post.contains("\"w\"")
                            || post.contains("'a'") || post.contains("\"a\"")
                            || post.contains("'wb'") || post.contains("\"wb\"")
                        {
                            let clean = p.trim_start_matches("./").trim();
                            if !clean.is_empty() && !paths.contains(&clean.to_string()) {
                                paths.push(clean.to_string());
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. touch file.txt
    if cmd.starts_with("touch ") {
        for arg in cmd.strip_prefix("touch ").unwrap_or("").split_whitespace() {
            let clean = arg.trim_start_matches("./").trim();
            if !clean.starts_with('-') && !clean.is_empty() && !paths.contains(&clean.to_string()) {
                paths.push(clean.to_string());
            }
        }
    }

    paths
}
