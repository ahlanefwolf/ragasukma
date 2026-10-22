use anyhow::{Context, Result};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use tokio::sync::broadcast;

use super::models::AgentSessionEvent;
use super::repository;
use super::tools::{execute_tool_call, get_agent_tools_schema};
use crate::llm::providers::antigravity::AntigravityProvider;
use crate::llm::providers::codex::CodexProvider;
use crate::llm::v1_engine::{V1ChatRequest, V1Message, V1StreamEvent, V1ToolCall, V1ToolCallFunction};

static SESSION_BROADCASTERS: OnceLock<Mutex<HashMap<String, broadcast::Sender<AgentSessionEvent>>>> =
    OnceLock::new();

fn get_broadcasters_map() -> &'static Mutex<HashMap<String, broadcast::Sender<AgentSessionEvent>>> {
    SESSION_BROADCASTERS.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn get_session_sender(session_id: &str) -> broadcast::Sender<AgentSessionEvent> {
    let mut map = get_broadcasters_map()
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    if let Some(sender) = map.get(session_id) {
        sender.clone()
    } else {
        // High capacity channel to prevent Lagged errors during rapid streaming / large tool calls
        let (sender, _) = broadcast::channel(2048);
        map.insert(session_id.to_string(), sender.clone());
        sender
    }
}

pub fn broadcast_session_event(session_id: &str, event: AgentSessionEvent) {
    let sender = get_session_sender(session_id);
    let _ = sender.send(event);
}

pub fn parse_project_folders(root_path: Option<&str>) -> Vec<String> {
    let Some(raw) = root_path else {
        return Vec::new();
    };
    if let Ok(arr) = serde_json::from_str::<Vec<String>>(raw) {
        return arr;
    }
    let trimmed = raw.trim();
    if !trimmed.is_empty() {
        vec![trimmed.to_string()]
    } else {
        Vec::new()
    }
}

/// Prunes older tool output results to optimize context window, token cost, and LLM inference latency.
/// Preserves the live tail (the most recent 3 tool results) intact while eliding bulky older outputs,
/// inspired by the `oh-my-pi` context pruning & shake architecture.
fn compact_tool_output(content: &str, is_recent_tail: bool) -> String {
    if is_recent_tail {
        return content.to_string();
    }
    // For older turns: if content is large (> 600 chars), prune the middle
    if content.len() > 600 {
        let prefix: String = content.chars().take(200).collect();
        let suffix: String = content.chars().rev().take(150).collect::<Vec<_>>().into_iter().rev().collect();
        format!(
            "{}\n\n[... Tool output pruned for speed & context optimization ({} bytes total) ...]\n\n{}",
            prefix,
            content.len(),
            suffix
        )
    } else {
        content.to_string()
    }
}

pub async fn execute_session_turn(session_id: &str, model_override: Option<&str>) -> Result<String> {
    // 1. Load Session & Project
    let session = repository::get_session(session_id)
        .await?
        .context("Session not found")?;

    let project = repository::get_project(&session.project_id)
        .await?
        .context("Project not found")?;

    let allowed_folders = parse_project_folders(project.root_path.as_deref());
    let prompt_skills = repository::get_project_skills(&project.id).await.unwrap_or_default();

    // 2. Mark session as running
    let _ = repository::update_session_status(session_id, "running").await;
    broadcast_session_event(
        session_id,
        AgentSessionEvent::StatusChange {
            status: "running".to_string(),
        },
    );

    // 3. Assemble Dense RFC 2119 System Prompt (inspired by oh-my-pi architecture)
    let mut system_prompt = String::from(
        "<system-conventions>\n\
         RFC 2119 applies: MUST, REQUIRED, NEVER, SHOULD, RECOMMENDED. NEVER and AVOID MUST be interpreted as aliases for MUST NOT and SHOULD NOT.\n\
         </system-conventions>\n\n\
         <role>\n\
         You are THE autonomous expert staff software engineer operating directly in the user's workspace environment.\n\
         You read code, explore directories, search patterns, run terminal commands, and create/modify files using provided workspace tools.\n\
         </role>\n\n\
         <critical>\n\
         1. ACTION OVER PASSIVITY: When asked to build, fix, modify, or generate code, you MUST immediately call `workspace_write_file` or edit tools. NEVER just describe what to do.\n\
         2. NEVER STOP PREMATURELY: Never inspect or list files with `workspace_list_dir` and halt. If directories are empty or missing targets, PROCEED IMMEDIATELY to create and write the files.\n\
         3. VERIFICATION & COMMANDS: Use `workspace_run_command` to inspect repo state, verify build/tests (`cargo check`, `npm test`, etc.), or list directory items when needed.\n\
         4. COMPLETE IMPLEMENTATION: Always write complete, production-ready, clean code. NEVER emit placeholders, TODO comments, or truncated snippets.\n\
         5. CONVERSATIONAL QUERIES: If the user asks a conceptual question or clarification, respond thoroughly and conversationally in Markdown. NEVER reply with empty content or a generic one-liner.\n\
         6. COMPREHENSIVE FINAL SUMMARY: Once all tool calls conclude, you MUST provide a clear, concise report in English summarizing tasks completed, modified files, and usage tips.\n\
         </critical>\n\n"
    );

    if !allowed_folders.is_empty() {
        system_prompt.push_str("### Workspace Attached Folders:\n");
        for f in &allowed_folders {
            system_prompt.push_str(&format!("- `{}`\n", f));
        }
        system_prompt.push('\n');
    }

    if let Some(ref custom) = project.custom_instructions {
        if !custom.trim().is_empty() {
            system_prompt.push_str("### Project Instructions:\n");
            system_prompt.push_str(custom.trim());
            system_prompt.push_str("\n\n");
        }
    }

    if !prompt_skills.is_empty() {
        system_prompt.push_str("### Active Prompt Skills & Persona Guidelines:\n");
        for skill in &prompt_skills {
            system_prompt.push_str(&format!("#### Skill: {}\n{}\n\n", skill.name, skill.system_prompt.trim()));
        }
    }

    // 4. Autonomous Tool Execution Loop (aligned with oh-my-pi architecture)
    // Runs autonomously until model has no more tool calls or user pauses/cancels
    let max_safety_iterations = 60;
    let mut iteration = 0;
    let mut ephemeral_directive: Option<String> = None;
    let mut modified_files: Vec<String> = Vec::new();
    let mut executed_commands: Vec<String> = Vec::new();
    let model = model_override
        .filter(|m| !m.trim().is_empty())
        .map(|m| m.to_string())
        .or_else(|| session.model.filter(|m| !m.trim().is_empty()))
        .unwrap_or_else(|| {
            let u = crate::config::load_user_config();
            if !u.llm_model.trim().is_empty() {
                u.llm_model
            } else {
                "ag/gemini-2.5-flash".to_string()
            }
        });

    // Update session model to the latest chosen model
    let _ = repository::update_session_model(session_id, &model).await;
    let is_codex = model.starts_with("cx/") || model.to_lowercase().contains("codex");

    while iteration < max_safety_iterations {
        iteration += 1;

        // Check if session was stopped or aborted externally
        if let Ok(Some(current_sess)) = repository::get_session(session_id).await {
            if current_sess.status != "running" {
                break;
            }
        }

        broadcast_session_event(
            session_id,
            AgentSessionEvent::StepInfo {
                iteration,
                action: "thinking".to_string(),
                details: Some(format!(
                    "Model {} is thinking & selecting actions...",
                    model
                )),
            },
        );

        // Fetch latest messages from DB and apply tool result pruning (oh-my-pi compaction)
        let db_messages = repository::list_messages(session_id).await?;
        let total_tool_msgs = db_messages.iter().filter(|m| m.role == "tool").count();
        let mut tool_counter = 0;

        let mut v1_messages = Vec::new();

        // Push System prompt first
        v1_messages.push(V1Message {
            role: "system".to_string(),
            content: Some(json!(system_prompt)),
            tool_calls: None,
            tool_call_id: None,
            name: None,
            thought_signature: None,
        });

        for msg in &db_messages {
            let tc_parsed: Option<Vec<V1ToolCall>> = msg
                .tool_calls
                .as_deref()
                .and_then(|raw| serde_json::from_str(raw).ok());

            let content_val = if msg.role == "user" {
                msg.content.as_ref().map(|c| {
                    let expanded = super::tools::expand_file_tags(c, &allowed_folders);
                    json!(expanded)
                })
            } else if msg.role == "tool" {
                tool_counter += 1;
                let is_recent_tail = tool_counter > total_tool_msgs.saturating_sub(3);
                msg.content.as_ref().map(|c| {
                    let compacted = compact_tool_output(c, is_recent_tail);
                    json!(compacted)
                })
            } else {
                msg.content.as_ref().map(|c| json!(c))
            };

            v1_messages.push(V1Message {
                role: msg.role.clone(),
                content: content_val,
                tool_calls: tc_parsed,
                tool_call_id: msg.tool_call_id.clone(),
                name: None,
                thought_signature: None,
            });
        }

        // Ephemeral in-memory prodding directive without polluting DB messages
        if let Some(directive) = ephemeral_directive.take() {
            v1_messages.push(V1Message {
                role: "user".to_string(),
                content: Some(json!(directive)),
                tool_calls: None,
                tool_call_id: None,
                name: None,
                thought_signature: None,
            });
        }

        let tools_schema = get_agent_tools_schema();
        let chat_req = V1ChatRequest {
            model: model.clone(),
            messages: v1_messages,
            temperature: session.temperature.map(|t| t as f32).or(Some(0.7)),
            max_tokens: Some(4096),
            stream: Some(true),
            tools: Some(tools_schema),
            tool_choice: None,
        };

        let input_chars: usize = chat_req.messages.iter().map(|m| {
            m.content.as_ref().map(|c| c.to_string().len()).unwrap_or(0)
        }).sum();
        let estimated_input_tokens = (input_chars / 4).max(1);
        let turn_start_time = std::time::Instant::now();

        // Call Provider inside spawn_blocking with automatic retry (up to 3 attempts)
        let req_clone = chat_req.clone();
        let sess_id_clone = session_id.to_string();
        let call_res = tokio::task::spawn_blocking(move || {
            let mut last_err = None;
            for attempt in 1..=3 {
                let mut accumulated_content = String::new();
                let mut accumulated_thought = String::new();
                let mut collected_tool_calls: Vec<V1ToolCall> = Vec::new();

                let attempt_result = if is_codex {
                    let mut provider = CodexProvider::new();
                    let res = provider.chat_stream_v1(&req_clone, |event| {
                        match event {
                            V1StreamEvent::ContentDelta(delta) => {
                                accumulated_content.push_str(&delta);
                                broadcast_session_event(
                                    &sess_id_clone,
                                    AgentSessionEvent::Delta { delta },
                                );
                            }
                            V1StreamEvent::ThoughtDelta(thought) => {
                                accumulated_thought.push_str(&thought);
                            }
                            V1StreamEvent::ToolCallDelta { index, id, name, arguments } => {
                                if index >= collected_tool_calls.len() {
                                    collected_tool_calls.resize(
                                        index + 1,
                                        V1ToolCall {
                                            id: id.clone().unwrap_or_else(|| format!("call_{}", index)),
                                            call_type: "function".to_string(),
                                            function: V1ToolCallFunction {
                                                name: name.clone().unwrap_or_default(),
                                                arguments: String::new(),
                                            },
                                            thought_signature: None,
                                            extra_content: None,
                                            provider_specific_fields: None,
                                        },
                                    );
                                }
                                if let Some(tc_id) = id {
                                    collected_tool_calls[index].id = tc_id;
                                }
                                if let Some(tc_name) = name {
                                    collected_tool_calls[index].function.name = tc_name;
                                }
                                collected_tool_calls[index].function.arguments.push_str(&arguments);
                            }
                            _ => {}
                        }
                        Ok(())
                    });
                    res.map(|v1_res| {
                        let tc = if v1_res.tool_calls.is_some() {
                            v1_res.tool_calls
                        } else if !collected_tool_calls.is_empty() {
                            Some(collected_tool_calls)
                        } else {
                            None
                        };
                        (accumulated_content, accumulated_thought, tc)
                    })
                } else {
                    let mut provider = AntigravityProvider::new();
                    let res = provider.chat_stream_v1(&req_clone, |event| {
                        match event {
                            V1StreamEvent::ContentDelta(delta) => {
                                accumulated_content.push_str(&delta);
                                broadcast_session_event(
                                    &sess_id_clone,
                                    AgentSessionEvent::Delta { delta },
                                );
                            }
                            V1StreamEvent::ThoughtDelta(thought) => {
                                accumulated_thought.push_str(&thought);
                            }
                            V1StreamEvent::ToolCallDelta { index, id, name, arguments } => {
                                if index >= collected_tool_calls.len() {
                                    collected_tool_calls.resize(
                                        index + 1,
                                        V1ToolCall {
                                            id: id.clone().unwrap_or_else(|| format!("call_{}", index)),
                                            call_type: "function".to_string(),
                                            function: V1ToolCallFunction {
                                                name: name.clone().unwrap_or_default(),
                                                arguments: String::new(),
                                            },
                                            thought_signature: None,
                                            extra_content: None,
                                            provider_specific_fields: None,
                                        },
                                    );
                                }
                                if let Some(tc_id) = id {
                                    collected_tool_calls[index].id = tc_id;
                                }
                                if let Some(tc_name) = name {
                                    collected_tool_calls[index].function.name = tc_name;
                                }
                                collected_tool_calls[index].function.arguments.push_str(&arguments);
                            }
                            _ => {}
                        }
                        Ok(())
                    });
                    res.map(|v1_res| {
                        let tc = if v1_res.tool_calls.is_some() {
                            v1_res.tool_calls
                        } else if !collected_tool_calls.is_empty() {
                            Some(collected_tool_calls)
                        } else {
                            None
                        };
                        (accumulated_content, accumulated_thought, tc)
                    })
                };

                match attempt_result {
                    Ok(val) => return Ok(val),
                    Err(e) => {
                        let err_str = e.to_string();
                        let is_retryable = err_str.contains("429")
                            || err_str.contains("503")
                            || err_str.contains("502")
                            || err_str.contains("timeout")
                            || err_str.contains("connection closed")
                            || err_str.contains("broken pipe");

                        last_err = Some(e);
                        if attempt < 3 && is_retryable {
                            std::thread::sleep(std::time::Duration::from_millis(800 * attempt as u64));
                            continue;
                        } else {
                            break;
                        }
                    }
                }
            }
            Err(last_err.unwrap_or_else(|| anyhow::anyhow!("Unknown upstream connection error")))
        })
        .await?;

        let (final_content, final_thought, tool_calls_opt) = match call_res {
            Ok(triplet) => triplet,
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

        let duration_ms = turn_start_time.elapsed().as_millis() as u64;
        let output_chars = final_content.len() + final_thought.len()
            + tool_calls_opt.as_ref().map(|tc| serde_json::to_string(tc).unwrap_or_default().len()).unwrap_or(0);
        let estimated_output_tokens = (output_chars / 4).max(1);

        // Record turn usage to llm_usage table
        let provider_name = if is_codex { "codex" } else { "antigravity" };
        let _ = crate::llm::usage::record_usage_async(
            provider_name,
            &model,
            estimated_input_tokens,
            0,
            estimated_output_tokens,
            duration_ms,
            "success",
        )
        .await;

        let thought_opt = if final_thought.trim().is_empty() {
            None
        } else {
            Some(final_thought.as_str())
        };

        // Check if model returned tool calls
        if let Some(tool_calls) = tool_calls_opt {
            if !tool_calls.is_empty() {
                // 1. Save assistant message containing the tool calls
                let tc_json = serde_json::to_string(&tool_calls).unwrap_or_default();
                let assistant_text = if final_content.is_empty() {
                    None
                } else {
                    Some(final_content.as_str())
                };

                let assistant_msg = repository::create_message(
                    session_id,
                    "assistant",
                    assistant_text,
                    thought_opt,
                    Some(&tc_json),
                    None,
                    "success",
                    Some(estimated_input_tokens as i64),
                    Some(estimated_output_tokens as i64),
                )
                .await?;

                broadcast_session_event(
                    session_id,
                    AgentSessionEvent::MessageCreated(assistant_msg),
                );

                // 2. Execute each tool call and save results
                for tc in &tool_calls {
                    broadcast_session_event(
                        session_id,
                        AgentSessionEvent::StepInfo {
                            iteration,
                            action: format!("tool:{}", tc.function.name),
                            details: Some(format!("Executing `{}`...", tc.function.name)),
                        },
                    );

                    let args_val = serde_json::from_str::<serde_json::Value>(&tc.function.arguments)
                        .unwrap_or_else(|_| json!({}));

                    // Track operations for automatic summary report
                    if tc.function.name == "workspace_write_file" {
                        if let Some(path) = args_val.get("path").and_then(|p| p.as_str()) {
                            if !modified_files.contains(&path.to_string()) {
                                modified_files.push(path.to_string());
                            }
                        }
                    } else if tc.function.name == "workspace_run_command" {
                        if let Some(cmd) = args_val.get("command").and_then(|c| c.as_str()) {
                            if !executed_commands.contains(&cmd.to_string()) {
                                executed_commands.push(cmd.to_string());
                            }
                        }
                    }

                    let tool_output = match execute_tool_call(
                        &tc.function.name,
                        &args_val,
                        &allowed_folders,
                    ) {
                        Ok(out) => out,
                        Err(e) => format!("Tool execution error: {}", e),
                    };

                    let tool_msg = repository::create_message(
                        session_id,
                        "tool",
                        Some(&tool_output),
                        None,
                        None,
                        Some(&tc.id),
                        "success",
                        None,
                        None,
                    )
                    .await?;

                    broadcast_session_event(
                        session_id,
                        AgentSessionEvent::MessageCreated(tool_msg),
                    );
                }

                // Loop again with tool results
                continue;
            }
        }

        // If the model returned no conversational content to the user yet:
        if final_content.trim().is_empty() && iteration < max_safety_iterations {
            ephemeral_directive = Some(
                "You have finished running tools or inspecting the workspace. Now provide a clear, comprehensive summary of the work done, code changes, or answers to the user's request.".to_string(),
            );
            continue;
        }

        // Final assistant turn reached (no tool calls)
        let generated_summary = if !modified_files.is_empty() || !executed_commands.is_empty() {
            let mut summary_lines = Vec::new();
            summary_lines.push("### 📋 Execution Summary".to_string());
            if !modified_files.is_empty() {
                summary_lines.push("\n**Modified / Created Files:**".to_string());
                for f in &modified_files {
                    summary_lines.push(format!("- `{}`", f));
                }
            }
            if !executed_commands.is_empty() {
                summary_lines.push("\n**Executed Commands:**".to_string());
                for c in &executed_commands {
                    summary_lines.push(format!("- `$ {}`", c));
                }
            }
            Some(summary_lines.join("\n"))
        } else {
            None
        };

        let composed_content = match (final_content.trim().is_empty(), generated_summary.as_ref()) {
            (false, Some(summary)) => format!("{}\n\n{}", final_content.trim(), summary),
            (false, None) => final_content.trim().to_string(),
            (true, Some(summary)) => summary.clone(),
            (true, None) if !final_thought.trim().is_empty() => final_thought.trim().to_string(),
            (true, None) => "I have completed the operations and workspace inspection as requested.".to_string(),
        };

        let content_to_save = composed_content.as_str();

        let final_msg = repository::create_message(
            session_id,
            "assistant",
            Some(content_to_save),
            thought_opt,
            None,
            None,
            "success",
            Some(estimated_input_tokens as i64),
            Some(estimated_output_tokens as i64),
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
            reply: "Tool iteration limit reached.".to_string(),
        },
    );
    Ok("Tool iteration limit reached.".to_string())
}
