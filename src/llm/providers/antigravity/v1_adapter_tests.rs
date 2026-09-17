use super::*;
use crate::llm::providers::antigravity::constants::DEFAULT_THINKING_AG_SIGNATURE;
use crate::llm::v1_engine::{V1ChatRequest, V1ToolCall, V1ToolCallFunction};

#[test]
fn test_clean_parameters_schema_with_zed_types() {
    let raw_schema = serde_json::json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "title": "ZedToolParameters",
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "path": {
                "type": "string",
                "description": "File path"
            },
            "offset": {
                "type": ["integer", "null"],
                "description": "Offset in bytes"
            },
            "options": {
                "anyOf": [
                    { "type": "string" },
                    { "type": "null" }
                ]
            }
        },
        "required": ["path", "nonexistent"]
    });

    let cleaned = clean_parameters_schema(&raw_schema);

    // Assert root properties
    assert_eq!(cleaned["type"], "object");
    assert!(cleaned.get("$schema").is_none());
    assert!(cleaned.get("title").is_none());
    assert!(cleaned.get("additionalProperties").is_none());

    // Assert path
    assert_eq!(cleaned["properties"]["path"]["type"], "string");

    // Assert offset flattened from ["integer", "null"] to "integer" with nullable: true
    assert_eq!(cleaned["properties"]["offset"]["type"], "integer");
    assert_eq!(cleaned["properties"]["offset"]["nullable"], true);

    // Assert options flattened from anyOf to "string" with nullable: true
    assert_eq!(cleaned["properties"]["options"]["type"], "string");
    assert_eq!(cleaned["properties"]["options"]["nullable"], true);

    // Assert required pruned non-existent field
    let required = cleaned["required"].as_array().unwrap();
    assert_eq!(required.len(), 1);
    assert_eq!(required[0], "path");
}

#[test]
fn test_thought_signature_cache_and_resolve() {
    let call_id = "call_test_12345";
    let sig = "test_signature_abc_xyz";
    store_thought_signature(call_id, sig);

    // Retrieve from cache
    assert_eq!(get_thought_signature(call_id), Some(sig.to_string()));
    assert_eq!(resolve_call_signature(call_id), Some(sig.to_string()));

    // Non-existent ID returns None
    assert_eq!(get_thought_signature("call_non_existent"), None);

    // Embedded signature in ID takes priority
    let embedded_id = "call_test_12345__thought__embedded_sig_999";
    assert_eq!(
        resolve_call_signature(embedded_id),
        Some("embedded_sig_999".to_string())
    );
}

#[test]
fn test_build_gemini_payload_attaches_thought_signature() {
    let req = V1ChatRequest {
        model: "ag/gemini-3.8-flash-high".to_string(),
        messages: vec![
            crate::llm::v1_engine::V1Message {
                role: "user".to_string(),
                content: Some(serde_json::json!("list directory contents")),
                tool_calls: None,
                tool_call_id: None,
                name: None,
                thought_signature: None,
            },
            crate::llm::v1_engine::V1Message {
                role: "assistant".to_string(),
                content: None,
                tool_calls: Some(vec![V1ToolCall {
                    id: "call_list_dir_1".to_string(),
                    call_type: "function".to_string(),
                    function: V1ToolCallFunction {
                        name: "default_api:list_directory".to_string(),
                        arguments: "{\"dir_path\":\"/workspace\"}".to_string(),
                    },
                    thought_signature: None,
                    extra_content: None,
                    provider_specific_fields: None,
                }]),
                tool_call_id: None,
                name: None,
                thought_signature: None,
            },
            crate::llm::v1_engine::V1Message {
                role: "tool".to_string(),
                content: Some(serde_json::json!("[\"file1.txt\",\"file2.txt\"]")),
                tool_calls: None,
                tool_call_id: Some("call_list_dir_1".to_string()),
                name: None,
                thought_signature: None,
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(4096),
        stream: Some(true),
        tools: None,
        tool_choice: None,
    };

    let payload = build_gemini_payload(&req, "test-project");
    let contents = payload["request"]["contents"].as_array().unwrap();

    // Model turn must have functionCall with thoughtSignature
    let model_turn = contents.iter().find(|c| c["role"] == "model").unwrap();
    let parts = model_turn["parts"].as_array().unwrap();
    let fc_part = parts.iter().find(|p| p.get("functionCall").is_some()).unwrap();

    assert_eq!(
        fc_part["thoughtSignature"].as_str().unwrap(),
        DEFAULT_THINKING_AG_SIGNATURE
    );
    assert_eq!(
        fc_part["functionCall"]["name"].as_str().unwrap(),
        "default_api:list_directory"
    );
}

#[test]
fn test_build_gemini_payload_uses_cached_signature() {
    let call_id = "call_cached_sig_test";
    let custom_sig = "EuwGCukGAXLI2custom_stored_signature";
    store_thought_signature(call_id, custom_sig);

    let req = V1ChatRequest {
        model: "gemini-3.8-flash-high".to_string(),
        messages: vec![
            crate::llm::v1_engine::V1Message {
                role: "user".to_string(),
                content: Some(serde_json::json!("hi")),
                tool_calls: None,
                tool_call_id: None,
                name: None,
                thought_signature: None,
            },
            crate::llm::v1_engine::V1Message {
                role: "assistant".to_string(),
                content: None,
                tool_calls: Some(vec![V1ToolCall {
                    id: call_id.to_string(),
                    call_type: "function".to_string(),
                    function: V1ToolCallFunction {
                        name: "test_tool".to_string(),
                        arguments: "{}".to_string(),
                    },
                    thought_signature: None,
                    extra_content: None,
                    provider_specific_fields: None,
                }]),
                tool_call_id: None,
                name: None,
                thought_signature: None,
            },
        ],
        temperature: None,
        max_tokens: None,
        stream: None,
        tools: None,
        tool_choice: None,
    };

    let payload = build_gemini_payload(&req, "test-project");
    let contents = payload["request"]["contents"].as_array().unwrap();
    let model_turn = contents.iter().find(|c| c["role"] == "model").unwrap();
    let fc_part = &model_turn["parts"][0];

    assert_eq!(fc_part["thoughtSignature"].as_str().unwrap(), custom_sig);
}
