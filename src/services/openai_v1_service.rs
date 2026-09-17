use crate::llm::providers::antigravity::AntigravityProvider;
use crate::llm::types::ChatMessage;
use crate::server::send_json_response;
use serde::{Deserialize, Serialize};
use std::io::Write;
use std::net::TcpStream;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static GLOBAL_ANTIGRAVITY: Mutex<Option<AntigravityProvider>> = Mutex::new(None);

fn get_provider() -> AntigravityProvider {
    let mut lock = GLOBAL_ANTIGRAVITY.lock().unwrap();
    if lock.is_none() {
        *lock = Some(AntigravityProvider::new());
    }
    // Return a clone or freshly synchronized instance
    let mut prov = lock.take().unwrap();
    let _ = prov.load_accounts();
    prov
}

fn release_provider(prov: AntigravityProvider) {
    let mut lock = GLOBAL_ANTIGRAVITY.lock().unwrap();
    *lock = Some(prov);
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<ContentPart>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentPart {
    #[serde(rename = "type", default)]
    pub part_type: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIMessage {
    pub role: String,
    pub content: Option<MessageContent>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAIChatRequest {
    pub model: String,
    pub messages: Vec<OpenAIMessage>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub stream: Option<bool>,
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
pub struct StreamDeltaRole {
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct StreamDeltaContent {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct StreamDeltaEmpty {}

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
        "ag/gemini-3.8-flash-high",
        "ag/gemini-3.7-flash-high",
        "ag/gemini-3.5-flash-high",
        "ag/gemini-2.5-pro",
        "ag/gemini-2.5-flash",
        "ag/claude-sonnet-4-6",
        "ag/claude-opus-4-6-thinking",
        "ag/claude-3-7-sonnet",
        "ag/claude-3-5-sonnet",
        "gemini-3.8-flash-high",
        "gemini-3.7-flash-high",
        "gemini-3.5-flash-high",
        "gemini-2.5-pro",
        "gemini-2.5-flash",
        "claude-sonnet-4-6",
        "claude-opus-4-6-thinking",
        "claude-3-7-sonnet",
        "claude-3-5-sonnet",
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
    let req: OpenAIChatRequest = match serde_json::from_str(body_str) {
        Ok(r) => r,
        Err(e) => {
            let err_json = format!("{{\"error\":{{\"message\":\"Invalid JSON request: {}\"}}}}", e);
            send_json_response(stream, 400, &err_json);
            return;
        }
    };

    let mut chat_messages: Vec<ChatMessage> = Vec::new();
    for m in req.messages {
        let content_str = match m.content {
            Some(MessageContent::Text(s)) => s,
            Some(MessageContent::Parts(parts)) => {
                let mut joined = String::new();
                for p in parts {
                    if let Some(t) = p.text {
                        joined.push_str(&t);
                    }
                }
                joined
            }
            None => String::new(),
        };

        chat_messages.push(ChatMessage {
            role: m.role,
            content: content_str,
        });
    }

    let is_stream = req.stream.unwrap_or(false);
    let temperature = req.temperature.unwrap_or(0.7);
    let max_tokens = req.max_tokens.unwrap_or(4096);
    let model = req.model.clone();
    let cmpl_id = format!("chatcmpl-{}", current_ts());
    let now = current_ts();

    let mut provider = get_provider();

    if is_stream {
        let resp_header = "HTTP/1.1 200 OK\r\n\
            Content-Type: text/event-stream\r\n\
            Cache-Control: no-cache\r\n\
            Connection: keep-alive\r\n\
            Access-Control-Allow-Origin: *\r\n\
            Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
            Access-Control-Allow-Headers: *\r\n\r\n";

        if stream.write_all(resp_header.as_bytes()).is_err() {
            release_provider(provider);
            return;
        }

        // Send initial chunk with role
        let init_chunk = StreamChunk {
            id: cmpl_id.clone(),
            object: "chat.completion.chunk".to_string(),
            created: now,
            model: model.clone(),
            choices: vec![StreamChoice {
                index: 0,
                delta: StreamDeltaRole {
                    role: "assistant".to_string(),
                },
                finish_reason: None,
            }],
        };
        if let Ok(json) = serde_json::to_string(&init_chunk) {
            let msg = format!("data: {}\n\n", json);
            let _ = stream.write_all(msg.as_bytes());
            let _ = stream.flush();
        }

        // Stream text chunks
        let mut stream_writer = stream.try_clone().ok();
        let cmpl_id_clone = cmpl_id.clone();
        let model_clone = model.clone();

        let stream_result = provider.chat_stream_raw(
            &model,
            &chat_messages,
            temperature,
            max_tokens,
            move |text_delta| {
                if let Some(ref mut sw) = stream_writer {
                    let chunk = StreamChunk {
                        id: cmpl_id_clone.clone(),
                        object: "chat.completion.chunk".to_string(),
                        created: now,
                        model: model_clone.clone(),
                        choices: vec![StreamChoice {
                            index: 0,
                            delta: StreamDeltaContent {
                                content: text_delta.to_string(),
                            },
                            finish_reason: None,
                        }],
                    };
                    if let Ok(json) = serde_json::to_string(&chunk) {
                        let msg = format!("data: {}\n\n", json);
                        let _ = sw.write_all(msg.as_bytes());
                        let _ = sw.flush();
                    }
                }
                Ok(())
            },
        );

        match stream_result {
            Ok(_) => {
                // Send stop chunk
                let finish_chunk = StreamChunk {
                    id: cmpl_id,
                    object: "chat.completion.chunk".to_string(),
                    created: now,
                    model,
                    choices: vec![StreamChoice {
                        index: 0,
                        delta: StreamDeltaEmpty {},
                        finish_reason: Some("stop".to_string()),
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

        release_provider(provider);
    } else {
        // Non-streaming response
        match provider.chat_stream_raw(&model, &chat_messages, temperature, max_tokens, |_| Ok(()))
        {
            Ok(full_reply) => {
                let approx_prompt_tokens = chat_messages
                    .iter()
                    .map(|m| m.content.len() / 4)
                    .sum::<usize>()
                    .max(1);
                let approx_comp_tokens = (full_reply.len() / 4).max(1);

                let resp = OpenAIChatResponse {
                    id: cmpl_id,
                    object: "chat.completion".to_string(),
                    created: now,
                    model,
                    choices: vec![OpenAIChatChoice {
                        index: 0,
                        message: OpenAIChatMessageOut {
                            role: "assistant".to_string(),
                            content: full_reply,
                        },
                        finish_reason: "stop".to_string(),
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

        release_provider(provider);
    }
}
