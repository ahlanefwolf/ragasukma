use super::constants::*;
use crate::llm::types::ChatMessage;
use crate::llm::v1_engine::{V1ChatRequest, extract_text_content};
use serde_json::json;

pub fn build_codex_payload_from_messages(
    model: &str,
    messages: &[ChatMessage],
    _temperature: f32,
    _max_tokens: u32,
) -> serde_json::Value {
    let clean_model = model.trim().trim_start_matches("cx/");
    let mut input_items = Vec::new();
    let mut instructions = CODEX_DEFAULT_INSTRUCTIONS.to_string();

    for msg in messages {
        if msg.role == "system" {
            if !msg.content.trim().is_empty() {
                instructions = msg.content.clone();
                // Inject pure system prompt into input as role="developer" identical to Antigravity
                input_items.push(json!({
                    "type": "message",
                    "role": "developer",
                    "content": [
                        {
                            "type": "input_text",
                            "text": msg.content
                        }
                    ]
                }));
            }
            continue;
        }

        let content_type = if msg.role == "assistant" {
            "output_text"
        } else {
            "input_text"
        };

        let text = if msg.role == "tool" {
            format!("[TOOL OBSERVATION]:\n{}", msg.content)
        } else {
            msg.content.clone()
        };

        input_items.push(json!({
            "type": "message",
            "role": if msg.role == "assistant" { "assistant" } else { "user" },
            "content": [
                {
                    "type": content_type,
                    "text": text
                }
            ]
        }));
    }

    json!({
        "model": clean_model,
        "input": input_items,
        "instructions": instructions,
        "stream": true,
        "store": false,
    })
}

pub fn build_codex_payload_from_v1(req: &V1ChatRequest) -> serde_json::Value {
    let clean_model = req.model.trim().trim_start_matches("cx/");
    let mut input_items = Vec::new();
    let mut system_instructions = CODEX_DEFAULT_INSTRUCTIONS.to_string();

    for msg in &req.messages {
        let content_str = extract_text_content(&msg.content);
        if msg.role == "system" {
            if !content_str.is_empty() {
                system_instructions = content_str.clone();
                input_items.push(json!({
                    "type": "message",
                    "role": "developer",
                    "content": [
                        {
                            "type": "input_text",
                            "text": content_str
                        }
                    ]
                }));
            }
            continue;
        }

        let content_type = if msg.role == "assistant" {
            "output_text"
        } else {
            "input_text"
        };

        input_items.push(json!({
            "type": "message",
            "role": if msg.role == "assistant" { "assistant" } else { "user" },
            "content": [
                {
                    "type": content_type,
                    "text": content_str
                }
            ]
        }));
    }

    json!({
        "model": clean_model,
        "input": input_items,
        "instructions": system_instructions,
        "stream": true,
        "store": false
    })
}

pub fn parse_codex_sse_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.starts_with("data:") {
        return None;
    }
    let data_str = trimmed.trim_start_matches("data:").trim();
    if data_str == "[DONE]" || data_str.is_empty() {
        return None;
    }

    let parsed: serde_json::Value = serde_json::from_str(data_str).ok()?;

    let event_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or("");

    // Ignore completion events that repeat the entire accumulated text
    if event_type.ends_with(".done") || event_type == "response.completed" {
        return None;
    }

    // 1. Direct delta field: {"type":"response.output_text.delta", "delta":"..."}
    if let Some(delta) = parsed.get("delta").and_then(|v| v.as_str()) {
        return Some(delta.to_string());
    }

    // 2. Nested text delta (only if type is a delta event): {"type":"response.text.delta", "text":"..."}
    if event_type.ends_with(".delta") {
        if let Some(text) = parsed.get("text").and_then(|v| v.as_str()) {
            return Some(text.to_string());
        }
    }

    // 3. Alternative OpenAI streaming format: choices[0].delta.content
    if let Some(content) = parsed.pointer("/choices/0/delta/content").and_then(|v| v.as_str()) {
        return Some(content.to_string());
    }

    None
}
