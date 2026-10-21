use anyhow::{bail, Context, Result};
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
        let (sender, _) = broadcast::channel(128);
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

pub async fn execute_session_turn(session_id: &str, model_override: Option<&str>) -> Result<String> {
    // 1. Load Session & Project
    let session = repository::get_session(session_id)
        .await?
        .context("Session tidak ditemukan")?;
    let project = repository::get_project(&session.project_id)
        .await?
        .context("Project tidak ditemukan")?;

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

    // 3. Assemble System Prompt
    let mut system_prompt = String::from(
        "You are an expert autonomous AI Coding Agent operating directly in the user's workspace environment.\n\
         You can read code, list directory contents, search files, and create/modify files using available tools.\n\n\
         ### CORE OPERATIONAL DIRECTIVES:\n\
         1. DIRECT ACTION OVER PASSIVITY: When asked to build, create, or modify code, you MUST actively call `workspace_write_file`.\n\
         2. DO NOT STOP PREMATURELY: Never just inspect or list files with `workspace_list_dir` and stop! If a directory is empty or missing target files, IMMEDIATELY proceed in the subsequent turn to create and write the necessary files.\n\
         3. WORKSPACE PATHS: Paths passed to tools can be relative to the workspace root or absolute within attached folders.\n\
         4. COMPLETE IMPLEMENTATION: Always write complete, production-ready, beautifully designed code with zero placeholders or truncated snippets.\n\
         5. CONCISE SUMMARY: Once all file writes and operations are completed, provide a concise summary of what was done.\n\n"
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

    // 4. Multi-turn Tool Execution Loop (max 8 turns)
    let max_iterations = 8;
    let mut iteration = 0;
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

    while iteration < max_iterations {
        iteration += 1;

        broadcast_session_event(
            session_id,
            AgentSessionEvent::StepInfo {
                iteration,
                action: "thinking".to_string(),
                details: Some(format!(
                    "Model {} sedang memproses & memilih tindakan...",
                    model
                )),
            },
        );

        // Fetch latest messages from DB
        let db_messages = repository::list_messages(session_id).await?;
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

        // Call Provider inside spawn_blocking
        let req_clone = chat_req.clone();
        let call_res = tokio::task::spawn_blocking(move || {
            let mut accumulated_content = String::new();
            let mut accumulated_thought = String::new();
            let mut collected_tool_calls: Vec<V1ToolCall> = Vec::new();

            if is_codex {
                let mut provider = CodexProvider::new();
                let res = provider.chat_stream_v1(&req_clone, |event| {
                    match event {
                        V1StreamEvent::ContentDelta(delta) => {
                            accumulated_content.push_str(&delta);
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
                    let tc = if !collected_tool_calls.is_empty() {
                        Some(collected_tool_calls)
                    } else {
                        v1_res.tool_calls
                    };
                    (accumulated_content, accumulated_thought, tc)
                })
            } else {
                let mut provider = AntigravityProvider::new();
                let res = provider.chat_stream_v1(&req_clone, |event| {
                    match event {
                        V1StreamEvent::ContentDelta(delta) => {
                            accumulated_content.push_str(&delta);
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
                    let tc = if !collected_tool_calls.is_empty() {
                        Some(collected_tool_calls)
                    } else {
                        v1_res.tool_calls
                    };
                    (accumulated_content, accumulated_thought, tc)
                })
            }
        })
        .await?;

        let (final_content, final_thought, tool_calls_opt) = match call_res {
            Ok(triplet) => triplet,
            Err(e) => {
                let err_str = e.to_string();
                let _ = repository::update_session_status(session_id, "failed").await;
                broadcast_session_event(
                    session_id,
                    AgentSessionEvent::Error {
                        message: err_str.clone(),
                    },
                );
                broadcast_session_event(
                    session_id,
                    AgentSessionEvent::StatusChange {
                        status: "failed".to_string(),
                    },
                );
                bail!("Gagal memanggil model LLM: {}", err_str);
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
                            details: Some(format!("Menjalankan `{}`...", tc.function.name)),
                        },
                    );

                    let args_val = serde_json::from_str::<serde_json::Value>(&tc.function.arguments)
                        .unwrap_or_else(|_| json!({}));

                    let tool_output = match execute_tool_call(
                        &tc.function.name,
                        &args_val,
                        &allowed_folders,
                    ) {
                        Ok(out) => out,
                        Err(e) => format!("Error eksekusi tool: {}", e),
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

        // If after executing tools (iteration > 1), the model returned no tool calls AND empty content
        // (e.g. it paused after listing directory or only thought), prod it to proceed with file generation.
        if final_content.trim().is_empty() && iteration > 1 && iteration < max_iterations {
            if thought_opt.is_some() {
                if let Ok(thought_msg) = repository::create_message(
                    session_id,
                    "assistant",
                    None,
                    thought_opt,
                    None,
                    None,
                    "success",
                    None,
                    None,
                )
                .await {
                    broadcast_session_event(
                        session_id,
                        AgentSessionEvent::MessageCreated(thought_msg),
                    );
                }
            }

            // Prompt the model to proceed with creating/editing the requested files
            if let Ok(prod_msg) = repository::create_message(
                session_id,
                "user",
                Some("Hasil pemeriksaan selesai. Sekarang lanjutkan untuk membuat atau memodifikasi file yang diminta menggunakan `workspace_write_file` secara lengkap."),
                None,
                None,
                None,
                "success",
                None,
                None,
            )
            .await {
                broadcast_session_event(
                    session_id,
                    AgentSessionEvent::MessageCreated(prod_msg),
                );
            }
            continue;
        }

        // Final assistant turn reached (no tool calls)
        let content_to_save = if !final_content.trim().is_empty() {
            final_content
        } else if thought_opt.is_some() {
            "Task telah diproses oleh agent.".to_string()
        } else {
            "Selesai.".to_string()
        };

        let final_msg = repository::create_message(
            session_id,
            "assistant",
            Some(&content_to_save),
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
                reply: content_to_save.clone(),
            },
        );

        let _ = repository::update_session_status(session_id, "idle").await;
        broadcast_session_event(
            session_id,
            AgentSessionEvent::StatusChange {
                status: "idle".to_string(),
            },
        );
        return Ok(content_to_save);
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
            reply: "Batas iterasi tool tercapai.".to_string(),
        },
    );
    Ok("Batas iterasi tool tercapai.".to_string())
}
