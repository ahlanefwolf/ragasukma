use crate::llm::providers::antigravity::AntigravityProvider;
use crate::llm::providers::codex::CodexProvider;
use crate::llm::providers::codex::constants::CODEX_DEFAULT_MODELS;
use crate::llm::v1_engine::{V1ChatRequest, V1StreamEvent, V1ToolCall};
use crate::server::send_json_response;
use serde::{Deserialize, Serialize};
use std::io::Write;

use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelItem {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub owned_by: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelListResponse {
    pub object: String,
    pub data: Vec<ModelItem>,
}

#[derive(Debug, Serialize)]
pub struct OpenAIChatChoice {
    pub index: usize,
    pub message: OpenAIChatMessageOut,
    pub finish_reason: String,
}

#[derive(Debug, Serialize)]
pub struct OpenAIChatMessageOut {
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<V1ToolCall>>,
}

#[derive(Debug, Serialize)]
pub struct OpenAIChatUsage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Serialize)]
pub struct OpenAIChatResponse {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<OpenAIChatChoice>,
    pub usage: OpenAIChatUsage,
}

#[derive(Debug, Serialize)]
pub struct StreamChoice<T: Serialize> {
    pub index: usize,
    pub delta: T,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StreamChunk<T: Serialize> {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<StreamChoice<T>>,
}

fn current_ts() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

pub fn handle_v1_options(stream: &mut dyn Write) {
    let resp = "HTTP/1.1 200 OK\r\n\
        Access-Control-Allow-Origin: *\r\n\
        Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n\
        Access-Control-Allow-Headers: *\r\n\
        Content-Length: 0\r\n\r\n";
    let _ = stream.write_all(resp.as_bytes());
}

pub fn handle_v1_models(stream: &mut dyn Write) {
    let now = current_ts();
    let mut provider = AntigravityProvider::new();

    let mut model_names = match provider.get_dynamic_available_models() {
        Ok(dynamic_models) => dynamic_models
            .into_iter()
            .map(|m| format!("ag/{}", m.model_id))
            .collect::<Vec<String>>(),
        Err(_) => Vec::new(),
    };

    if model_names.is_empty() {
        model_names.push("ag/gemini-3.8-flash-high".to_string());
    }

    let mut data = model_names
        .into_iter()
        .map(|name| ModelItem {
            id: name,
            object: "model".to_string(),
            created: now,
            owned_by: "antigravity".to_string(),
        })
        .collect::<Vec<ModelItem>>();

    // Append OpenAI Codex models (cx/ prefix)
    for (model_id, _) in CODEX_DEFAULT_MODELS {
        data.push(ModelItem {
            id: format!("cx/{}", model_id),
            object: "model".to_string(),
            created: now,
            owned_by: "codex".to_string(),
        });
    }

    let resp = ModelListResponse {
        object: "list".to_string(),
        data,
    };

    if let Ok(json) = serde_json::to_string(&resp) {
        send_json_response(stream, 200, &json);
    } else {
        send_json_response(stream, 500, "{\"error\":\"Failed to serialize models\"}");
    }
}

pub fn handle_v1_chat_completions(body_str: &str, stream: &mut dyn Write) {
    let req: V1ChatRequest = match serde_json::from_str(body_str) {
        Ok(r) => r,
        Err(e) => {
            let err_json = format!(
                "{{\"error\":{{\"message\":\"Invalid JSON request: {}\"}}}}",
                e
            );
            send_json_response(stream, 400, &err_json);
            return;
        }
    };

    let is_stream = req.stream.unwrap_or(false);
    let model = req.model.clone();
    let cmpl_id = format!("chatcmpl-{}", current_ts());
    let now = current_ts();

    let is_codex = model.starts_with("cx/") || model.to_lowercase().contains("codex");

    if is_stream {
        let resp_header = "HTTP/1.1 200 OK\r\n\
            Content-Type: text/event-stream\r\n\
            Cache-Control: no-cache\r\n\
            Connection: keep-alive\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
            Access-Control-Allow-Headers: *\r\n\r\n";

        if stream.write_all(resp_header.as_bytes()).is_err() {
            return;
        }

        // Send initial chunk with role: "assistant"
        let init_chunk = StreamChunk {
            id: cmpl_id.clone(),
            object: "chat.completion.chunk".to_string(),
            created: now,
            model: model.clone(),
            choices: vec![StreamChoice {
                index: 0,
                delta: serde_json::json!({ "role": "assistant" }),
                finish_reason: None,
            }],
        };
        if let Ok(json) = serde_json::to_string(&init_chunk) {
            let msg = format!("data: {}\n\n", json);
            let _ = stream.write_all(msg.as_bytes());
            let _ = stream.flush();
        }

        let cmpl_id_clone = cmpl_id.clone();
        let model_clone = model.clone();
        let mut final_finish_reason = "stop".to_string();

        let stream_result = if is_codex {
            let mut codex_provider = CodexProvider::new();
            codex_provider.chat_stream_v1(&req, |event| {
                match event {
                    V1StreamEvent::ContentDelta(delta_text) => {
                        let chunk = StreamChunk {
                            id: cmpl_id_clone.clone(),
                            object: "chat.completion.chunk".to_string(),
                            created: now,
                            model: model_clone.clone(),
                            choices: vec![StreamChoice {
                                index: 0,
                                delta: serde_json::json!({ "content": delta_text }),
                                finish_reason: None,
                            }],
                        };
                        if let Ok(json) = serde_json::to_string(&chunk) {
                            let msg = format!("data: {}\n\n", json);
                            let _ = stream.write_all(msg.as_bytes());
                            let _ = stream.flush();
                        }
                    }
                    V1StreamEvent::Finish { finish_reason } => {
                        final_finish_reason = finish_reason;
                    }
                    _ => {}
                }
                Ok(())
            })
        } else {
            let mut provider = AntigravityProvider::new();
            provider.chat_stream_v1(&req, |event| {
                match event {
                    V1StreamEvent::ContentDelta(delta_text) => {
                        let chunk = StreamChunk {
                            id: cmpl_id_clone.clone(),
                            object: "chat.completion.chunk".to_string(),
                            created: now,
                            model: model_clone.clone(),
                            choices: vec![StreamChoice {
                                index: 0,
                                delta: serde_json::json!({ "content": delta_text }),
                                finish_reason: None,
                            }],
                        };
                        if let Ok(json) = serde_json::to_string(&chunk) {
                            let msg = format!("data: {}\n\n", json);
                            let _ = stream.write_all(msg.as_bytes());
                            let _ = stream.flush();
                        }
                    }
                    V1StreamEvent::ToolCallDelta {
                        index,
                        id,
                        name,
                        arguments,
                    } => {
                        let mut tc_json = serde_json::json!({
                            "index": index,
                            "type": "function",
                            "function": {
                                "arguments": arguments
                            }
                        });
                        if let Some(ref call_id) = id {
                            tc_json["id"] = serde_json::json!(call_id);
                        }
                        if let Some(ref fn_name) = name {
                            tc_json["function"]["name"] = serde_json::json!(fn_name);
                        }

                        let chunk = StreamChunk {
                            id: cmpl_id_clone.clone(),
                            object: "chat.completion.chunk".to_string(),
                            created: now,
                            model: model_clone.clone(),
                            choices: vec![StreamChoice {
                                index: 0,
                                delta: serde_json::json!({ "tool_calls": [tc_json] }),
                                finish_reason: None,
                            }],
                        };
                        if let Ok(json) = serde_json::to_string(&chunk) {
                            let msg = format!("data: {}\n\n", json);
                            let _ = stream.write_all(msg.as_bytes());
                            let _ = stream.flush();
                        }
                    }
                    V1StreamEvent::KeepAlive => {
                        let _ = stream.write_all(b":\n\n");
                        let _ = stream.flush();
                    }
                    V1StreamEvent::Finish { finish_reason } => {
                        final_finish_reason = finish_reason;
                    }
                }
                Ok(())
            })
        };

        match stream_result {
            Ok(v1_resp) => {
                let finish_reason = if final_finish_reason.is_empty() {
                    v1_resp.finish_reason
                } else {
                    final_finish_reason
                };

                let finish_chunk = StreamChunk {
                    id: cmpl_id,
                    object: "chat.completion.chunk".to_string(),
                    created: now,
                    model,
                    choices: vec![StreamChoice {
                        index: 0,
                        delta: serde_json::json!({}),
                        finish_reason: Some(finish_reason),
                    }],
                };
                if let Ok(json) = serde_json::to_string(&finish_chunk) {
                    let msg = format!("data: {}\n\n", json);
                    let _ = stream.write_all(msg.as_bytes());
                }
                let _ = stream.write_all(b"data: [DONE]\n\n");
                let _ = stream.flush();
            }
            Err(e) => {
                let err_chunk = serde_json::json!({
                    "error": {
                        "message": format!("Stream error: {e}")
                    }
                });
                let msg = format!("data: {}\n\n", err_chunk);
                let _ = stream.write_all(msg.as_bytes());
                let _ = stream.write_all(b"data: [DONE]\n\n");
                let _ = stream.flush();
            }
        }
    } else {
        // Non-streaming response
        let non_stream_res = if is_codex {
            let mut codex_provider = CodexProvider::new();
            codex_provider.chat_stream_v1(&req, |_| Ok(()))
        } else {
            let mut provider = AntigravityProvider::new();
            provider.chat_stream_v1(&req, |_| Ok(()))
        };

        match non_stream_res {
            Ok(v1_resp) => {
                let approx_prompt_tokens = req
                    .messages
                    .iter()
                    .map(|m| serde_json::to_string(&m.content).unwrap_or_default().len() / 4)
                    .sum::<usize>()
                    .max(1);
                let approx_comp_tokens = (v1_resp.content.len() / 4).max(1);

                let resp = OpenAIChatResponse {
                    id: cmpl_id,
                    object: "chat.completion".to_string(),
                    created: now,
                    model,
                    choices: vec![OpenAIChatChoice {
                        index: 0,
                        message: OpenAIChatMessageOut {
                            role: "assistant".to_string(),
                            content: v1_resp.content,
                            tool_calls: v1_resp.tool_calls,
                        },
                        finish_reason: v1_resp.finish_reason,
                    }],
                    usage: OpenAIChatUsage {
                        prompt_tokens: approx_prompt_tokens,
                        completion_tokens: approx_comp_tokens,
                        total_tokens: approx_prompt_tokens + approx_comp_tokens,
                    },
                };

                if let Ok(json) = serde_json::to_string(&resp) {
                    send_json_response(stream, 200, &json);
                } else {
                    send_json_response(stream, 500, "{\"error\":\"Failed to serialize response\"}");
                }
            }
            Err(e) => {
                let err_json = format!("{{\"error\":{{\"message\":\"{}\"}}}}", e);
                send_json_response(stream, 500, &err_json);
            }
        }
    }
}
