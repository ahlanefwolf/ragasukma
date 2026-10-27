use anyhow::Result;
use serde_json::json;
use std::collections::HashSet;

use crate::agent::events::broadcast_session_event;
use crate::agent::models::AgentSessionEvent;
use crate::llm::providers::antigravity::AntigravityProvider;
use crate::llm::providers::codex::CodexProvider;
use crate::llm::v1_engine::{V1ChatRequest, V1Message, V1StreamEvent, V1ToolCall, V1ToolCallFunction};

pub struct ModelTurnResult {
    pub content: String,
    pub thought: String,
    pub tool_calls: Option<Vec<V1ToolCall>>,
    pub estimated_input_tokens: usize,
    pub estimated_output_tokens: usize,
}

#[allow(clippy::too_many_arguments)]
pub async fn execute_model_stream(
    session_id: &str,
    model: &str,
    is_codex: bool,
    temperature: Option<f64>,
    messages: Vec<V1Message>,
    tools_schema: Vec<serde_json::Value>,
    force_final_text_only: bool,
    iteration: usize,
) -> Result<ModelTurnResult> {
    let (tools_payload, tool_choice_payload) = if force_final_text_only {
        (None, Some(json!("none")))
    } else {
        (Some(tools_schema), None)
    };

    let chat_req = V1ChatRequest {
        model: model.to_string(),
        messages,
        temperature: temperature.map(|t| (t as f32).min(0.2)).or(Some(0.1)),
        max_tokens: Some(8192),
        stream: Some(true),
        tools: tools_payload,
        tool_choice: tool_choice_payload,
    };

    let input_chars: usize = chat_req.messages.iter().map(|m| {
        m.content.as_ref().map(|c| c.to_string().len()).unwrap_or(0)
    }).sum();
    let estimated_input_tokens = (input_chars / 4).max(1);
    let turn_start_time = std::time::Instant::now();

    let req_clone = chat_req.clone();
    let sess_id_clone = session_id.to_string();
    let current_iteration = iteration;
    let cancellation_flag = crate::agent::events::get_session_cancellation_flag(session_id);

    let call_res = tokio::task::spawn_blocking(move || {
        let mut last_err = None;
        let mut deltas_broadcasted = false;

        for attempt in 1..=3 {
            if cancellation_flag.load(std::sync::atomic::Ordering::Relaxed) {
                anyhow::bail!("Session cancelled by user");
            }

            // Ghost Streaming mitigation: If retrying after tokens were already streamed to UI,
            // send a StreamReset event so UI clears partial accumulated tokens and avoids duplicate output.
            if attempt > 1 && deltas_broadcasted {
                broadcast_session_event(
                    &sess_id_clone,
                    AgentSessionEvent::StreamReset,
                );
                deltas_broadcasted = false;
            }

            let mut accumulated_content = String::new();
            let mut accumulated_thought = String::new();
            let mut collected_tool_calls: Vec<V1ToolCall> = Vec::new();

            let attempt_result = if is_codex {
                let mut announced_tool_indices: HashSet<usize> = HashSet::new();
                let mut provider = CodexProvider::new();
                let res = provider.chat_stream_v1(&req_clone, |event| {
                    if cancellation_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        anyhow::bail!("Session cancelled by user");
                    }
                    match event {
                        V1StreamEvent::ContentDelta(delta) => {
                            accumulated_content.push_str(&delta);
                            deltas_broadcasted = true;
                            broadcast_session_event(
                                &sess_id_clone,
                                AgentSessionEvent::Delta { delta },
                            );
                        }
                        V1StreamEvent::ThoughtDelta(thought) => {
                            accumulated_thought.push_str(&thought);
                            deltas_broadcasted = true;
                            broadcast_session_event(
                                &sess_id_clone,
                                AgentSessionEvent::ThoughtDelta { delta: thought },
                            );
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
                                if !tc_name.is_empty() && announced_tool_indices.insert(index) {
                                    broadcast_session_event(
                                        &sess_id_clone,
                                        AgentSessionEvent::StepInfo {
                                            iteration: current_iteration,
                                            action: format!("tool:{}", tc_name),
                                            details: Some(format!("Preparing `{}`...", tc_name)),
                                        },
                                    );
                                }
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
                let mut announced_tool_indices: HashSet<usize> = HashSet::new();
                let mut provider = AntigravityProvider::new();
                let res = provider.chat_stream_v1(&req_clone, |event| {
                    if cancellation_flag.load(std::sync::atomic::Ordering::Relaxed) {
                        anyhow::bail!("Session cancelled by user");
                    }
                    match event {
                        V1StreamEvent::ContentDelta(delta) => {
                            accumulated_content.push_str(&delta);
                            deltas_broadcasted = true;
                            broadcast_session_event(
                                &sess_id_clone,
                                AgentSessionEvent::Delta { delta },
                            );
                        }
                        V1StreamEvent::ThoughtDelta(thought) => {
                            accumulated_thought.push_str(&thought);
                            deltas_broadcasted = true;
                            broadcast_session_event(
                                &sess_id_clone,
                                AgentSessionEvent::ThoughtDelta { delta: thought },
                            );
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
                                if !tc_name.is_empty() && announced_tool_indices.insert(index) {
                                    broadcast_session_event(
                                        &sess_id_clone,
                                        AgentSessionEvent::StepInfo {
                                            iteration: current_iteration,
                                            action: format!("tool:{}", tc_name),
                                            details: Some(format!("Preparing `{}`...", tc_name)),
                                        },
                                    );
                                }
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
                    if cancellation_flag.load(std::sync::atomic::Ordering::Relaxed)
                        || err_str.contains("Session cancelled")
                    {
                        return Err(e);
                    }

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
    .await??;

    let (final_content, final_thought, tool_calls_opt) = call_res;

    let duration_ms = turn_start_time.elapsed().as_millis() as u64;
    let tool_chars: usize = tool_calls_opt.as_ref().map(|calls| {
        calls.iter().map(|tc| tc.function.name.len() + tc.function.arguments.len()).sum()
    }).unwrap_or(0);
    let output_chars = final_content.len() + final_thought.len() + tool_chars;
    let estimated_output_tokens = (output_chars / 4).max(1);

    // Record turn usage to llm_usage table
    let provider_name = if is_codex { "codex" } else { "antigravity" };
    let _ = crate::llm::usage::record_usage_async(
        provider_name,
        model,
        estimated_input_tokens,
        0,
        estimated_output_tokens,
        duration_ms,
        "success",
    )
    .await;

    Ok(ModelTurnResult {
        content: final_content,
        thought: final_thought,
        tool_calls: tool_calls_opt,
        estimated_input_tokens,
        estimated_output_tokens,
    })
}
