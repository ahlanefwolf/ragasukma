use crate::llm::providers::antigravity::AntigravityProvider;
use crate::llm::v1_engine::{V1ChatRequest, V1StreamEvent, V1ToolCall};
use crate::server::send_json_response;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::net::TcpStream;
use std::sync::Mutex;
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

pub fn handle_v1_options(stream: &mut TcpStream) {
    let resp = "HTTP/1.1 200 OK\r\n\
        Access-Control-Allow-Origin: *\r\n\
        Access-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS\r\n\
        Access-Control-Allow-Headers: *\r\n\
        Content-Length: 0\r\n\r\n";
    let _ = stream.write_all(resp.as_bytes());
}

pub fn handle_v1_models(stream: &mut TcpStream) {
    let now = current_ts();
    let model_names = vec![
        "ag/claude-3-5-sonnet",
        "ag/claude-3-7-sonnet",
        "ag/claude-sonnet-4-6",
        "ag/claude-opus-4-6-thinking",
        "ag/gemini-3.8-flash-high",
        "ag/gemini-3.8-pro-high",
        "ag/gemini-3.7-flash-high",
        "ag/gemini-3.5-flash-high",
        "ag/gemini-2.5-pro",
        "ag/gemini-2.5-flash",
        "ag/gpt-4o",
        "ag/gpt-4o-mini",
        "ag/gpt-oss-120b-medium",
    ];

    let data = model_names
        .into_iter()
        .map(|name| ModelItem {
            id: name.to_string(),
            object: "model".to_string(),
            created: now,
            owned_by: "antigravity".to_string(),
        })
        .collect();

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

pub fn handle_v1_chat_completions(body_str: &str, stream: &mut TcpStream) {
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

    let mut provider = AntigravityProvider::new();

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

        let mut stream_writer = stream.try_clone().ok();
        let cmpl_id_clone = cmpl_id.clone();
        let model_clone = model.clone();
        let final_finish_reason = std::sync::Arc::new(Mutex::new("stop".to_string()));
        let final_reason_clone = final_finish_reason.clone();

        let stream_result = provider.chat_stream_v1(&req, move |event| {
            if let Some(ref mut sw) = stream_writer {
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
                            let _ = sw.write_all(msg.as_bytes());
                            let _ = sw.flush();
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
                            let _ = sw.write_all(msg.as_bytes());
                            let _ = sw.flush();
                        }
                    }
                    V1StreamEvent::KeepAlive => {
                        // Send SSE comment to keep TCP connection alive during thinking phases
                        let _ = sw.write_all(b":\n\n");
                        let _ = sw.flush();
                    }
                    V1StreamEvent::Finish { finish_reason } => {
                        if let Ok(mut lock) = final_reason_clone.lock() {
                            *lock = finish_reason;
                        }
                    }
                }
            }
            Ok(())
        });

        match stream_result {
            Ok(v1_resp) => {
                let finish_reason = final_finish_reason
                    .lock()
                    .map(|l| l.clone())
                    .unwrap_or(v1_resp.finish_reason);

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
        match provider.chat_stream_v1(&req, |_| Ok(())) {
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
