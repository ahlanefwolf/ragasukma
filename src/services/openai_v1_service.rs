use crate::llm::providers::antigravity::AntigravityProvider;
use crate::llm::providers::claude::ClaudeProvider;
use crate::llm::providers::codex::CodexProvider;
use crate::llm::v1_engine::{V1ChatRequest, V1StreamEvent, V1ToolCall};
use serde::{Deserialize, Serialize};

use axum::response::IntoResponse;
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

pub fn get_v1_models() -> ModelListResponse {
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

    // Append OpenAI Codex models (cx/ prefix) dynamically from active accounts (no fallback)
    let mut cx_provider = CodexProvider::new();
    if let Ok(cx_models) = cx_provider.get_dynamic_available_models() {
        for m in cx_models {
            data.push(ModelItem {
                id: m.id,
                object: "model".to_string(),
                created: now,
                owned_by: "codex".to_string(),
            });
        }
    }

    // Append Claude models (cc/ prefix) dynamically from active accounts
    let mut cc_provider = ClaudeProvider::new();
    if let Ok(cc_models) = cc_provider.get_dynamic_available_models() {
        for m in cc_models {
            data.push(ModelItem {
                id: m.id,
                object: "model".to_string(),
                created: now,
                owned_by: "claude".to_string(),
            });
        }
    }

    ModelListResponse {
        object: "list".to_string(),
        data,
    }
}





// ===========================================================================
// NATIVE AXUM HANDLERS (Real-time SSE Streaming & Low-latency Routing)
// ===========================================================================

pub async fn axum_v1_options_handler() -> axum::response::Response {
    axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header("Access-Control-Allow-Origin", "*")
        .header("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
        .header("Access-Control-Allow-Headers", "*")
        .header("Content-Length", "0")
        .body(axum::body::Body::empty())
        .unwrap_or_else(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

pub async fn axum_v1_root_handler() -> axum::response::Response {
    let json = serde_json::json!({
        "status": "ok",
        "service": "ragasukma-openai-v1",
        "version": "0.2.0"
    });
    axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(axum::body::Body::from(json.to_string()))
        .unwrap_or_else(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

pub async fn axum_v1_models_handler() -> axum::response::Response {
    let models_resp = tokio::task::spawn_blocking(get_v1_models)
        .await
        .unwrap_or_else(|_| ModelListResponse {
            object: "list".to_string(),
            data: Vec::new(),
        });

    let json = serde_json::to_string(&models_resp).unwrap_or_else(|_| "{}".to_string());
    axum::response::Response::builder()
        .status(axum::http::StatusCode::OK)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(axum::body::Body::from(json))
        .unwrap_or_else(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

pub async fn axum_v1_chat_completions_handler(
    body_bytes: axum::body::Bytes,
) -> axum::response::Response {
    let req: V1ChatRequest = match serde_json::from_slice(&body_bytes) {
        Ok(r) => r,
        Err(e) => {
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&body_bytes) {
                if let Some(prompt) = val.get("prompt").and_then(|p| p.as_str()) {
                    let model = val
                        .get("model")
                        .and_then(|m| m.as_str())
                        .unwrap_or("ag/gemini-3.8-flash-high")
                        .to_string();
                    let stream = val.get("stream").and_then(|s| s.as_bool());
                    let temperature = val
                        .get("temperature")
                        .and_then(|t| t.as_f64())
                        .map(|t| t as f32);
                    let max_tokens = val
                        .get("max_tokens")
                        .and_then(|m| m.as_u64())
                        .map(|m| m as u32);
                    V1ChatRequest {
                        model,
                        messages: vec![crate::llm::v1_engine::V1Message {
                            role: "user".to_string(),
                            content: Some(serde_json::Value::String(prompt.to_string())),
                            tool_calls: None,
                            tool_call_id: None,
                            name: None,
                            thought_signature: None,
                        }],
                        temperature,
                        max_tokens,
                        stream,
                        tools: None,
                        tool_choice: None,
                    }
                } else {
                    return json_error_response(
                        axum::http::StatusCode::BAD_REQUEST,
                        &format!("Invalid JSON request: {}", e),
                    );
                }
            } else {
                return json_error_response(
                    axum::http::StatusCode::BAD_REQUEST,
                    &format!("Invalid JSON request: {}", e),
                );
            }
        }
    };

    let is_stream = req.stream.unwrap_or(false);
    let model = req.model.clone();
    let cmpl_id = format!("chatcmpl-{}", current_ts());
    let now = current_ts();
    let is_claude = model.starts_with("cc/");
    let is_codex = model.starts_with("cx/");
    let req_start_time = std::time::Instant::now();

    if is_stream {
        let (tx, mut rx) = tokio::sync::mpsc::channel::<axum::body::Bytes>(64);

        tokio::task::spawn_blocking(move || {
            // Initial chunk with role: "assistant"
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
                if tx.blocking_send(axum::body::Bytes::from(msg)).is_err() {
                    return;
                }
            }

            let cmpl_id_clone = cmpl_id.clone();
            let model_clone = model.clone();
            let mut final_finish_reason = "stop".to_string();
            let tx_clone = tx.clone();

            let stream_result = if is_claude {
                let mut claude_provider = ClaudeProvider::new();
                claude_provider.chat_stream_v1(&req, |event| {
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
                                if tx_clone.blocking_send(axum::body::Bytes::from(msg)).is_err() {
                                    return Err(anyhow::anyhow!("Client disconnected"));
                                }
                            }
                        }
                        V1StreamEvent::ThoughtDelta(thought_text) => {
                            let chunk = StreamChunk {
                                id: cmpl_id_clone.clone(),
                                object: "chat.completion.chunk".to_string(),
                                created: now,
                                model: model_clone.clone(),
                                choices: vec![StreamChoice {
                                    index: 0,
                                    delta: serde_json::json!({ "reasoning_content": thought_text }),
                                    finish_reason: None,
                                }],
                            };
                            if let Ok(json) = serde_json::to_string(&chunk) {
                                let msg = format!("data: {}\n\n", json);
                                if tx_clone.blocking_send(axum::body::Bytes::from(msg)).is_err() {
                                    return Err(anyhow::anyhow!("Client disconnected"));
                                }
                            }
                        }
                        V1StreamEvent::Finish { finish_reason } => {
                            final_finish_reason = finish_reason;
                        }
                        _ => {}
                    }
                    Ok(())
                })
            } else if is_codex {
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
                                if tx_clone.blocking_send(axum::body::Bytes::from(msg)).is_err() {
                                    return Err(anyhow::anyhow!("Client disconnected"));
                                }
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
                                if tx_clone.blocking_send(axum::body::Bytes::from(msg)).is_err() {
                                    return Err(anyhow::anyhow!("Client disconnected"));
                                }
                            }
                        }
                        V1StreamEvent::ThoughtDelta(thought_text) => {
                            let chunk = StreamChunk {
                                id: cmpl_id_clone.clone(),
                                object: "chat.completion.chunk".to_string(),
                                created: now,
                                model: model_clone.clone(),
                                choices: vec![StreamChoice {
                                    index: 0,
                                    delta: serde_json::json!({ "reasoning_content": thought_text }),
                                    finish_reason: None,
                                }],
                            };
                            if let Ok(json) = serde_json::to_string(&chunk) {
                                let msg = format!("data: {}\n\n", json);
                                if tx_clone.blocking_send(axum::body::Bytes::from(msg)).is_err() {
                                    return Err(anyhow::anyhow!("Client disconnected"));
                                }
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
                                if tx_clone.blocking_send(axum::body::Bytes::from(msg)).is_err() {
                                    return Err(anyhow::anyhow!("Client disconnected"));
                                }
                            }
                        }
                        V1StreamEvent::KeepAlive => {
                            if tx_clone
                                .blocking_send(axum::body::Bytes::from_static(b":\n\n"))
                                .is_err()
                            {
                                return Err(anyhow::anyhow!("Client disconnected"));
                            }
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
                        model: model.clone(),
                        choices: vec![StreamChoice {
                            index: 0,
                            delta: serde_json::json!({}),
                            finish_reason: Some(finish_reason),
                        }],
                    };
                    if let Ok(json) = serde_json::to_string(&finish_chunk) {
                        let msg = format!("data: {}\n\n", json);
                        let _ = tx.blocking_send(axum::body::Bytes::from(msg));
                    }
                    let _ = tx.blocking_send(axum::body::Bytes::from_static(b"data: [DONE]\n\n"));

                    let duration_ms = req_start_time.elapsed().as_millis() as u64;
                    let approx_prompt_tokens = req
                        .messages
                        .iter()
                        .map(|m| serde_json::to_string(&m.content).unwrap_or_default().len() / 4)
                        .sum::<usize>()
                        .max(1);
                    let approx_comp_tokens = (v1_resp.content.len() / 4).max(1);
                    let provider = if is_claude {
                        "claude"
                    } else if is_codex {
                        "codex"
                    } else {
                        "antigravity"
                    };
                    crate::llm::usage::record_usage(
                        provider,
                        &model,
                        approx_prompt_tokens,
                        0,
                        approx_comp_tokens,
                        duration_ms,
                        "success",
                    );
                }
                Err(e) => {
                    let err_chunk = serde_json::json!({
                        "error": {
                            "message": format!("Stream error: {e}")
                        }
                    });
                    let msg = format!("data: {}\n\n", err_chunk);
                    let _ = tx.blocking_send(axum::body::Bytes::from(msg));
                    let _ = tx.blocking_send(axum::body::Bytes::from_static(b"data: [DONE]\n\n"));
                }
            }
        });

        let body_stream = async_stream::stream! {
            while let Some(chunk) = rx.recv().await {
                yield Ok::<axum::body::Bytes, std::convert::Infallible>(chunk);
            }
        };

        axum::response::Response::builder()
            .status(axum::http::StatusCode::OK)
            .header("Content-Type", "text/event-stream; charset=utf-8")
            .header("Cache-Control", "no-cache, no-transform")
            .header("Connection", "keep-alive")
            .header("X-Accel-Buffering", "no")
            .header("Access-Control-Allow-Origin", "*")
            .header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
            .header("Access-Control-Allow-Headers", "*")
            .body(axum::body::Body::from_stream(body_stream))
            .unwrap_or_else(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response())
    } else {
        let approx_prompt_tokens = req
            .messages
            .iter()
            .map(|m| serde_json::to_string(&m.content).unwrap_or_default().len() / 4)
            .sum::<usize>()
            .max(1);

        // Non-streaming response
        let non_stream_res = tokio::task::spawn_blocking(move || {
            if is_claude {
                let mut claude_provider = ClaudeProvider::new();
                claude_provider.chat_stream_v1(&req, |_| Ok(()))
            } else if is_codex {
                let mut codex_provider = CodexProvider::new();
                codex_provider.chat_stream_v1(&req, |_| Ok(()))
            } else {
                let mut provider = AntigravityProvider::new();
                provider.chat_stream_v1(&req, |_| Ok(()))
            }
        })
        .await;

        let v1_resp_res = match non_stream_res {
            Ok(r) => r,
            Err(join_err) => {
                return json_error_response(
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    &format!("Task join error: {}", join_err),
                );
            }
        };

        match v1_resp_res {
            Ok(v1_resp) => {
                let approx_comp_tokens = (v1_resp.content.len() / 4).max(1);
                let duration_ms = req_start_time.elapsed().as_millis() as u64;
                let provider = if is_claude {
                    "claude"
                } else if is_codex {
                    "codex"
                } else {
                    "antigravity"
                };
                crate::llm::usage::record_usage(
                    provider,
                    &model,
                    approx_prompt_tokens,
                    0,
                    approx_comp_tokens,
                    duration_ms,
                    "success",
                );

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

                match serde_json::to_string(&resp) {
                    Ok(json) => axum::response::Response::builder()
                        .status(axum::http::StatusCode::OK)
                        .header("Content-Type", "application/json")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(axum::body::Body::from(json))
                        .unwrap_or_else(|_| {
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
                        }),
                    Err(_) => json_error_response(
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        "Failed to serialize response",
                    ),
                }
            }
            Err(e) => json_error_response(
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                &format!("{}", e),
            ),
        }
    }
}

fn json_error_response(status: axum::http::StatusCode, msg: &str) -> axum::response::Response {
    let err_json = serde_json::json!({
        "error": {
            "message": msg
        }
    });
    axum::response::Response::builder()
        .status(status)
        .header("Content-Type", "application/json")
        .header("Access-Control-Allow-Origin", "*")
        .body(axum::body::Body::from(err_json.to_string()))
        .unwrap_or_else(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_v1_models() {
        let resp = get_v1_models();
        assert_eq!(resp.object, "list");
        assert!(!resp.data.is_empty());
        assert!(resp.data.iter().any(|m| m.id.starts_with("ag/")));
    }

    #[tokio::test]
    async fn test_axum_v1_options_handler() {
        let res = axum_v1_options_handler().await;
        assert_eq!(res.status(), axum::http::StatusCode::OK);
        let headers = res.headers();
        assert_eq!(headers.get("access-control-allow-origin").unwrap(), "*");
        assert!(headers.contains_key("access-control-allow-methods"));
    }

    #[tokio::test]
    async fn test_axum_v1_root_handler() {
        use axum::body::to_bytes;
        let res = axum_v1_root_handler().await;
        assert_eq!(res.status(), axum::http::StatusCode::OK);
        let bytes = to_bytes(res.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(json["status"], "ok");
        assert_eq!(json["service"], "ragasukma-openai-v1");
    }

    #[tokio::test]
    async fn test_axum_v1_chat_completions_invalid_json() {
        use axum::body::to_bytes;
        let body_bytes = axum::body::Bytes::from_static(b"{invalid-json");
        let res = axum_v1_chat_completions_handler(body_bytes).await;
        assert_eq!(res.status(), axum::http::StatusCode::BAD_REQUEST);
        let bytes = to_bytes(res.into_body(), 1024).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(json["error"]["message"].as_str().unwrap().contains("Invalid JSON request"));
    }
}
