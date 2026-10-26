use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub content: String,
    pub reasoning_content: Option<String>,
    pub model: String,
}

/// Generic trait for LLM providers (OpenAI, Anthropic Claude, Ollama, etc.)
/// Language and vendor agnostic.
pub trait LlmProvider: Send + Sync {
    fn provider_name(&self) -> &str;
    fn chat_complete<'a>(
        &'a self,
        request: &'a ChatCompletionRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ChatCompletionResponse>> + Send + 'a>>;
}

/// Generic OpenAI-compatible API provider (OpenAI, vLLM, DeepSeek, LocalAI)
pub struct GenericOpenAiProvider {
    pub base_url: String,
    pub api_key: String,
    client: reqwest::Client,
}

impl GenericOpenAiProvider {
    pub fn new(base_url: String, api_key: String) -> Self {
        Self {
            base_url,
            api_key,
            client: reqwest::Client::new(),
        }
    }
}

impl LlmProvider for GenericOpenAiProvider {
    fn provider_name(&self) -> &str {
        "openai_compatible"
    }

    fn chat_complete<'a>(
        &'a self,
        request: &'a ChatCompletionRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ChatCompletionResponse>> + Send + 'a>> {
        Box::pin(async move {
            let endpoint = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
            let mut req = self.client.post(&endpoint).json(request);
            if !self.api_key.is_empty() {
                req = req.bearer_auth(&self.api_key);
            }

            let resp = req.send().await?;
            if !resp.status().is_success() {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                anyhow::bail!("OpenAI API error [{}]: {}", status, err_text);
            }

            let json_body: serde_json::Value = resp.json().await?;
            let content = json_body["choices"][0]["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string();
            let reasoning_content = json_body["choices"][0]["message"]["reasoning_content"]
                .as_str()
                .map(|s| s.to_string());
            let model = json_body["model"].as_str().unwrap_or(&request.model).to_string();

            Ok(ChatCompletionResponse {
                content,
                reasoning_content,
                model,
            })
        })
    }
}

/// Generic Ollama provider for local LLM inference
pub struct OllamaProvider {
    pub base_url: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }
}

impl LlmProvider for OllamaProvider {
    fn provider_name(&self) -> &str {
        "ollama"
    }

    fn chat_complete<'a>(
        &'a self,
        request: &'a ChatCompletionRequest,
    ) -> Pin<Box<dyn Future<Output = Result<ChatCompletionResponse>> + Send + 'a>> {
        Box::pin(async move {
            let endpoint = format!("{}/api/chat", self.base_url.trim_end_matches('/'));
            let body = serde_json::json!({
                "model": request.model,
                "messages": request.messages,
                "stream": false
            });

            let resp = self.client.post(&endpoint).json(&body).send().await?;
            if !resp.status().is_success() {
                let status = resp.status();
                let err_text = resp.text().await.unwrap_or_default();
                anyhow::bail!("Ollama API error [{}]: {}", status, err_text);
            }

            let json_body: serde_json::Value = resp.json().await?;
            let content = json_body["message"]["content"]
                .as_str()
                .unwrap_or("")
                .to_string();

            Ok(ChatCompletionResponse {
                content,
                reasoning_content: None,
                model: request.model.clone(),
            })
        })
    }
}
