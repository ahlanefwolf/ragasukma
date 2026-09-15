use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
    pub max_tokens: u32,
    pub stream: bool,
}

// Ollama native API fallback format
#[derive(Serialize)]
pub struct OllamaChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub stream: bool,
}

#[derive(Deserialize)]
pub struct OllamaChatMessage {
    pub content: String,
}

#[derive(Deserialize)]
pub struct OllamaChatResponse {
    pub message: OllamaChatMessage,
}
