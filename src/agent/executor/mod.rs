use anyhow::{Context, Result};

pub mod gates;
pub mod helpers;
pub mod messages;
pub mod prompt;
pub mod snapshot;
pub mod state;
pub mod stream;
pub mod tool_runner;

// Re-export submodules for full backwards compatibility
pub use super::compressor::*;
pub use super::diagnostics::*;
pub use super::events::*;
pub use helpers::{detect_project_verification_commands, is_verification_command, parse_project_folders};
pub use snapshot::WorkspaceSnapshot;
pub use state::TurnState;

use crate::agent::models::AgentSessionEvent;
use crate::agent::repository;

pub async fn execute_session_turn(session_id: &str, model_override: Option<&str>) -> Result<String> {
    // 1. Load Session & Project
    let session = repository::get_session(session_id)
        .await?
        .context("Session not found")?;

    let project = repository::get_project(&session.project_id)
        .await?
        .context("Project not found")?;

    let allowed_folders = parse_project_folders(project.root_path.as_deref());
    for folder in &allowed_folders {
        let _ = crate::agent::tools::ensure_git_and_gitignore(std::path::Path::new(folder));
    }
    let prompt_skills = crate::agent::skills::scan_project_skills(project.root_path.as_deref(), &allowed_folders);
    let active_mcp_servers = repository::get_project_active_mcp_servers(&project.id).await.unwrap_or_default();
    let mcp_tools = crate::agent::mcp::resolve_active_mcp_tools(&active_mcp_servers).await;
    let dynamic_tools_schema = crate::agent::tools::get_dynamic_agent_tools_schema(&mcp_tools);

    // 2. Mark session as running
    let _ = repository::update_session_status(session_id, "running").await;
    broadcast_session_event(
        session_id,
        AgentSessionEvent::StatusChange {
            status: "running".to_string(),
        },
    );

    // 3. Load message history and resolve active goal
    let mut db_messages = repository::list_messages(session_id).await?;
    let (goal_id, current_phase, locked_targets, discovery_counter) =
        state::resolve_or_create_goal(&session, &project, &db_messages, &allowed_folders).await?;

    // 4. Assemble system prompt
    let system_prompt = prompt::build_system_prompt(
        &project,
        &allowed_folders,
        &prompt_skills,
        &active_mcp_servers,
        &mcp_tools,
        session_id,
        &goal_id,
    ).await?;

    // 5. Initialize turn state & resolve model
    let prior_verifs = repository::list_goal_verifications(&goal_id).await.unwrap_or_default();
    let mut state = TurnState::new(
        goal_id.clone(),
        current_phase,
        locked_targets,
        discovery_counter,
        prior_verifs.len(),
    );

    let (model, is_codex) = state::resolve_model(&session, model_override).await;
    let detected_verifier = crate::agent::languages::detect_primary_verification_command(&allowed_folders);

    // 6. Autonomous Tool Execution Loop
    while state.iteration < state.max_safety_iterations {
        state.iteration += 1;

        // Check if session was stopped or aborted externally
        if let Ok(Some(status)) = repository::get_session_status(session_id).await {
            if status != "running" {
                println!("[Agent] Session {} status changed to '{}', breaking iteration loop.", session_id, status);
                break;
            }
        }

        broadcast_session_event(
            session_id,
            AgentSessionEvent::StepInfo {
                iteration: state.iteration,
                action: "thinking".to_string(),
                details: Some(format!(
                    "Model {} is thinking & selecting actions...",
                    model
                )),
            },
        );

        // Prepare context messages for model
        let current_directive = state.ephemeral_directive.take();
        let v1_messages = messages::prepare_turn_messages(
            session_id,
            &system_prompt,
            &db_messages,
            &allowed_folders,
            current_directive,
        );

        // Stream from provider
        // 7. Stream LLM with automatic retry for transient endpoint glitches
        let mut retry_count = 0;
        let max_stream_retries = 2;
        let turn_res = loop {
            match stream::execute_model_stream(
                session_id,
                &model,
                is_codex,
                session.temperature,
                v1_messages.clone(),
                dynamic_tools_schema.clone(),
                state.force_final_text_only,
                state.iteration,
            ).await {
                Ok(res) => break Ok(res),
                Err(e) => {
                    let err_str = e.to_string();
                    let is_transient_glitch = err_str.contains("stream endpoint")
                        || err_str.contains("Failed to send request")
                        || err_str.contains("timeout")
                        || err_str.contains("connection reset")
                        || err_str.contains("broken pipe")
                        || err_str.contains("502")
                        || err_str.contains("503")
                        || err_str.contains("504");

                    if is_transient_glitch && retry_count < max_stream_retries {
                        retry_count += 1;
                        eprintln!(
                            "[Agent Stream Retry] Transient error on iteration {}: {}. Retrying attempt {}/{}...",
                            state.iteration, err_str, retry_count, max_stream_retries
                        );
                        tokio::time::sleep(std::time::Duration::from_millis(1500 * retry_count as u64)).await;
                        continue;
                    }
                    break Err(e);
                }
            }
        };

        let turn_res = match turn_res {
            Ok(res) => res,
            Err(e) => {
                let err_str = e.to_string();
                let err_msg_content = format!(
                    "⚠️ **Error communicating with AI model**: {}\n\nYou can switch the model using the dropdown above or send another prompt to retry.",
                    err_str
                );

                if let Ok(err_msg) = repository::create_message(
                    session_id,
                    "assistant",
                    Some(&err_msg_content),
                    None,
                    None,
                    None,
                    "error",
                    None,
                    None,
                )
                .await
                {
                    broadcast_session_event(
                        session_id,
                        AgentSessionEvent::MessageCreated(err_msg),
                    );
                }

                let _ = repository::update_session_status(session_id, "idle").await;
                broadcast_session_event(
                    session_id,
                    AgentSessionEvent::StatusChange {
                        status: "idle".to_string(),
                    },
                );
                broadcast_session_event(
                    session_id,
                    AgentSessionEvent::Error {
                        message: err_str.clone(),
                    },
                );

                return Ok(err_msg_content);
            }
        };

        let thought_opt = if turn_res.thought.trim().is_empty() {
            None
        } else {
            Some(turn_res.thought.as_str())
        };

        // If tool calls were returned, execute them
        if let Some(mut tool_calls) = turn_res.tool_calls {
            if !tool_calls.is_empty() {
                // Enforce strictly 1 tool call per turn to guarantee stability and prevent empty argument errors
                if tool_calls.len() > 1 {
                    eprintln!(
                        "[Agent Turn] Model returned {} parallel tool calls; enforcing single tool invocation",
                        tool_calls.len()
                    );
                    // Prefer the first tool call with non-empty arguments, or fallback to the first call
                    let chosen = tool_calls
                        .into_iter()
                        .find(|tc| !tc.function.arguments.trim().is_empty())
                        .unwrap_or_else(|| crate::llm::v1_engine::V1ToolCall {
                            id: format!("call_{}", crate::llm::v1_engine::generate_v1_uuid("tc")),
                            call_type: "function".to_string(),
                            function: crate::llm::v1_engine::V1ToolCallFunction {
                                name: String::new(),
                                arguments: String::new(),
                            },
                            thought_signature: None,
                            extra_content: None,
                            provider_specific_fields: None,
                        });
                    tool_calls = if chosen.function.name.is_empty() {
                        Vec::new()
                    } else {
                        vec![chosen]
                    };
                }
            }

            if !tool_calls.is_empty() {
                let tc_json = serde_json::to_string(&tool_calls).unwrap_or_default();
                let assistant_text = if turn_res.content.is_empty() {
                    None
                } else {
                    Some(turn_res.content.as_str())
                };

                let assistant_msg = repository::create_message(
                    session_id,
                    "assistant",
                    assistant_text,
                    thought_opt,
                    Some(&tc_json),
                    None,
                    "success",
                    Some(turn_res.estimated_input_tokens as i64),
                    Some(turn_res.estimated_output_tokens as i64),
                )
                .await?;

                broadcast_session_event(
                    session_id,
                    AgentSessionEvent::MessageCreated(assistant_msg.clone()),
                );
                db_messages.push(assistant_msg);

                tool_runner::execute_turn_tool_calls(
                    session_id,
                    &project.id,
                    &allowed_folders,
                    &mcp_tools,
                    &tool_calls,
                    &mut state,
                    &mut db_messages,
                ).await?;

                // Approaching safety limit notice
                if state.iteration >= state.max_safety_iterations.saturating_sub(2) {
                    state.ephemeral_directive = Some(
                        "TURN LIMIT APPROACHING: Please finalize your actions now. Summarize your findings, code modifications, or answer the user directly.".to_string()
                    );
                }

                continue;
            }
        }

        // Evaluate completion gates
        match gates::evaluate_turn_gates(
            &mut state,
            &turn_res.content,
            &allowed_folders,
            detected_verifier.as_deref(),
        ) {
            gates::GateResult::Continue => continue,
            gates::GateResult::Pass => {}
        }

        // Final turn conclusion
        let composed_content = gates::compose_final_turn_content(
            &state,
            &turn_res.content,
            &turn_res.thought,
        );
        let content_to_save = composed_content.as_str();

        let final_goal_status = if state.last_command_had_error || state.last_tool_had_error {
            crate::agent::orchestrator::GoalState::Failed
        } else {
            crate::agent::orchestrator::GoalState::Completed
        };
        let orch = crate::agent::orchestrator::GoalOrchestrator::new(&goal_id, &project.id);
        let _ = orch.conclude_with_status(final_goal_status, content_to_save).await;

        let final_msg = repository::create_message(
            session_id,
            "assistant",
            Some(content_to_save),
            thought_opt,
            None,
            None,
            "success",
            Some(turn_res.estimated_input_tokens as i64),
            Some(turn_res.estimated_output_tokens as i64),
        )
        .await?;

        broadcast_session_event(
            session_id,
            AgentSessionEvent::MessageCreated(final_msg),
        );
        broadcast_session_event(
            session_id,
            AgentSessionEvent::Done {
                reply: content_to_save.to_string(),
            },
        );

        let _ = repository::update_session_status(session_id, "idle").await;
        broadcast_session_event(
            session_id,
            AgentSessionEvent::StatusChange {
                status: "idle".to_string(),
            },
        );

        return Ok(content_to_save.to_string());
    }

    // Fallback: If iteration limit reached while still running tools
    gates::handle_iteration_limit_fallback(session_id, &state).await
}
