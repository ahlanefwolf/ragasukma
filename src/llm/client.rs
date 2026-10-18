use super::cleaner::clean_ai_text;
use super::prompt::{build_system_prompt, build_system_prompt_indexed};
use super::providers::antigravity::AntigravityProvider;
use super::providers::codex::CodexProvider;
use super::types::{ChatMessage, ChatRequest, OllamaChatRequest, OllamaChatResponse};
use anyhow::{bail, Context, Result};

pub struct LlmClient {
    pub provider: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub client: reqwest::blocking::Client,
    pub history: Vec<ChatMessage>,
    pub antigravity: AntigravityProvider,
    pub codex: CodexProvider,
}

impl LlmClient {
    pub fn new() -> Self {
        let u = crate::config::load_user_config();
        let api_key = if !u.llm_api_key.trim().is_empty() {
            Some(u.llm_api_key)
        } else {
            None
        };
        let provider = u.llm_provider;
        let base_url = u.llm_base_url;
        let model = u.llm_model;

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(8)) // Max timeout 8s agar cepat auto-recover jika model offline/hang
            .build()
            .unwrap_or_default();

        let system_prompt = build_system_prompt();

        let history = vec![ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        }];

        let antigravity = AntigravityProvider::new();
        let codex = CodexProvider::new();

        Self {
            provider,
            base_url,
            api_key,
            model,
            client,
            history,
            antigravity,
            codex,
        }
    }

    pub fn reset_history(&mut self) {
        self.history = vec![ChatMessage {
            role: "system".to_string(),
            content: build_system_prompt(),
        }];
    }

    pub fn prune_history(&mut self) {
        // Hemat token in RAM/API: save system prompt + maksimal 6 conversation terakhir
        if self.history.len() > 7 {
            let keep_start = self.history.len() - 6;
            let mut pruned = vec![self.history[0].clone()];
            pruned.extend_from_slice(&self.history[keep_start..]);
            self.history = pruned;
        }
    }

    pub fn chat_stream<F>(&mut self, user_input: &str, on_sentence: F) -> Result<String>
    where
        F: FnMut(&str, &str) -> Result<()>,
    {
        self.chat_stream_with_role("user", user_input, on_sentence)
    }

    pub fn chat_stream_with_role<F>(
        &mut self,
        role: &str,
        input_text: &str,
        mut on_sentence: F,
    ) -> Result<String>
    where
        F: FnMut(&str, &str) -> Result<()>,
    {
        if input_text.trim().is_empty() {
            return Ok(String::new());
        }

        if role == "user" {
            // Dynamic Prompt Indexing: suntikkan only Ground of Truth which relevan with ucapan user!
            let current_sys_prompt = build_system_prompt_indexed(Some(input_text));
            if let Some(first_msg) = self.history.first_mut() {
                if first_msg.role == "system" {
                    first_msg.content = current_sys_prompt;
                }
            }
        }

        self.history.push(ChatMessage {
            role: role.to_string(),
            content: input_text.to_string(),
        });

        // Selalu reload akun dari disk agar sinkron real-time jika ada akun yang baru ditambah / dihapus
        let _ = self.antigravity.load_accounts();
        let _ = self.codex.load_accounts();

        // 1. Local Mode: Using internal Ragasukma engine (Antigravity pool or Codex pool)
        let is_codex_engine = self.provider.eq_ignore_ascii_case("codex")
            || self.provider.eq_ignore_ascii_case("openai-codex")
            || self.model.starts_with("cx/");

        if is_codex_engine && !self.codex.accounts.is_empty() {
            match self.codex.chat_stream(
                &self.model,
                &self.history,
                0.7,
                8192,
                &mut on_sentence,
            ) {
                Ok(reply) => {
                    let clean = clean_ai_text(&reply);
                    self.history.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: clean.clone(),
                    });
                    self.prune_history();

                    let u = crate::config::load_user_config();
                    crate::llm::history::log_conversation_turn(
                        &u.user_callsign,
                        &u.assistant_name,
                        &u.personality_role,
                        input_text,
                        &clean,
                    );
                    return Ok(reply);
                }
                Err(e) => {
                    crate::term_err!("[Codex Engine Error]: {}. Falling back to default provider...", e);
                }
            }
        }

        let is_local_engine = self.provider.eq_ignore_ascii_case("local")
            || self.provider.eq_ignore_ascii_case("antigravity");

        if is_local_engine {
            if !self.antigravity.accounts.is_empty() {
                let ag_model = if self.model.starts_with("cx/") {
                    "gemini-2.5-flash"
                } else {
                    &self.model
                };
                match self.antigravity.chat_stream(
                    ag_model,
                    &self.history,
                    0.7,
                    8192,
                    &mut on_sentence,
                ) {
                    Ok(reply) => {
                        let clean = clean_ai_text(&reply);
                        self.history.push(ChatMessage {
                            role: "assistant".to_string(),
                            content: clean.clone(),
                        });
                        self.prune_history();

                        let u = crate::config::load_user_config();
                        crate::llm::history::log_conversation_turn(
                            &u.user_callsign,
                            &u.assistant_name,
                            &u.personality_role,
                            input_text,
                            &clean,
                        );

                        return Ok(reply);
                    }
                    Err(e) => {
                        crate::term_err!(
                            "[LLM Client] Local Engine error: {e}. Attempting fallback..."
                        );
                    }
                }
            } else {
                crate::term_log!(
                    "[LLM Client] Provider set to 'local', but no local engine account is connected to the pool."
                );
            }
        }

        // 2. Mode Custom: AI Assistant kustom (OpenAI-compatible, Ollama, LM Studio, vLLM, Groq, dll)
        // Custom endpoint strictly obeys base_url configuration and user-selected model
        let custom_endpoint = if self.base_url.ends_with("/chat/completions") {
            self.base_url.clone()
        } else {
            format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
        };

        let req_body = ChatRequest {
            model: self.model.clone(),
            messages: self.history.clone(),
            temperature: 0.5,
            max_tokens: 2048,
            stream: true,
            response_format: None,
        };

        let mut req_builder = self
            .client
            .post(&custom_endpoint)
            .header("Content-Type", "application/json");

        if let Some(ref key) = self.api_key {
            if !key.trim().is_empty() {
                req_builder = req_builder.header("Authorization", format!("Bearer {key}"));
            }
        }

        use std::io::BufRead;
        match req_builder.json(&req_body).send() {
            Ok(response) => {
                if response.status().is_success() {
                    let reader = std::io::BufReader::new(response);
                    let mut full_reply = String::new();
                    let mut current_sentence = String::new();
                    let mut active_emotion = "neutral";

                    for line in reader.lines() {
                        if crate::app_state::is_interruption_requested() {
                            bail!("Interrupted by user");
                        }
                        let line = match line {
                            Ok(l) => l,
                            Err(e) => {
                                if crate::app_state::is_interruption_requested() {
                                    bail!("Interrupted by user");
                                }
                                return Err(e.into());
                            }
                        };
                        let trimmed = line.trim();
                        if let Some(data_str) = trimmed.strip_prefix("data: ") {
                            if data_str.trim() == "[DONE]" {
                                break;
                            }
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(data_str) {
                                if val.pointer("/choices/0/delta/reasoning_content").is_some() {
                                    continue;
                                }

                                let chunk_text = val
                                    .pointer("/choices/0/delta/content")
                                    .or_else(|| val.pointer("/choices/0/message/content"))
                                    .and_then(|v| v.as_str());

                                if let Some(delta) = chunk_text {
                                    full_reply.push_str(delta);
                                    current_sentence.push_str(delta);

                                    let has_skill_tag = current_sentence.contains("[SKILL:")
                                        || current_sentence.contains("[CMD:");
                                    let has_skill_closed =
                                        has_skill_tag && current_sentence.contains(']');

                                    if !has_skill_tag || has_skill_closed {
                                        while let Some(pos) =
                                            current_sentence.find(['.', '?', '!', '\n'])
                                        {
                                            if let Some(tag_idx) = current_sentence
                                                .find("[SKILL:")
                                                .or_else(|| current_sentence.find("[CMD:"))
                                            {
                                                if pos > tag_idx {
                                                    break;
                                                }
                                            }

                                            // Do not break in the middle of ellipses (... or ..)
                                            let mut end_pos = pos;
                                            if current_sentence.as_bytes().get(pos) == Some(&b'.') {
                                                while end_pos + 1 < current_sentence.len()
                                                    && current_sentence.as_bytes().get(end_pos + 1) == Some(&b'.')
                                                {
                                                    end_pos += 1;
                                                }
                                            }

                                            let sentence = current_sentence[..=end_pos].to_string();
                                            current_sentence =
                                                current_sentence[end_pos + 1..].to_string();

                                            if let Some(emo) =
                                                crate::ui::renderer::expressions::detect_emotion(
                                                    &sentence,
                                                )
                                            {
                                                active_emotion = emo;
                                            }

                                            let clean = clean_ai_text(&sentence);
                                            if !clean.is_empty() {
                                                on_sentence(&clean, active_emotion)?;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let clean_rest = clean_ai_text(&current_sentence);
                    if !clean_rest.is_empty() {
                        if let Some(emo) =
                            crate::ui::renderer::expressions::detect_emotion(&current_sentence)
                        {
                            active_emotion = emo;
                        }
                        on_sentence(&clean_rest, active_emotion)?;
                    }

                    let clean = clean_ai_text(&full_reply);
                    self.history.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: clean.clone(),
                    });
                    self.prune_history();

                    let u = crate::config::load_user_config();
                    crate::llm::history::log_conversation_turn(
                        &u.user_callsign,
                        &u.assistant_name,
                        &u.personality_role,
                        input_text,
                        &clean,
                    );

                    return Ok(full_reply);
                } else {
                    let status = response.status();
                    let err_text = response.text().unwrap_or_default();
                    anyhow::bail!("Endpoint {custom_endpoint} returned status {status}: {err_text}");
                }
            }
            Err(e) => {
                if crate::app_state::is_interruption_requested() {
                    bail!("Interrupted by user");
                }
                // Hanya fallback ke Ollama jika memang endpoint-nya mengarah ke Ollama /api/chat
                if !self.base_url.contains("/api/chat") && !self.base_url.contains(":11434") {
                    return Err(anyhow::anyhow!("Failed streaming request to {custom_endpoint}: {e}"));
                }
            }
        }

        // 3. Fallback if custom endpoint Ollama native /api/chat used
        self.chat_fallback_ollama(input_text, on_sentence)
    }

    fn chat_fallback_ollama<F>(&mut self, input_text: &str, mut on_sentence: F) -> Result<String>
    where
        F: FnMut(&str, &str) -> Result<()>,
    {
        let ollama_endpoint = if self.base_url.ends_with("/api/chat") {
            self.base_url.clone()
        } else {
            format!("{}/api/chat", self.base_url.trim_end_matches('/'))
        };

        let req_body = OllamaChatRequest {
            model: self.model.clone(),
            messages: self.history.clone(),
            stream: false,
            format: None,
        };

        let res = self
            .client
            .post(&ollama_endpoint)
            .json(&req_body)
            .send()
            .map_err(|e| anyhow::anyhow!("Failed to connect to Ollama ({ollama_endpoint}): {e}"))?;

        if !res.status().is_success() {
            let err_text = res.text().unwrap_or_default();
            anyhow::bail!("Ollama error: {err_text}");
        }

        let chat_res: OllamaChatResponse = res.json().context("Failed to parse Ollama response")?;
        let emo = crate::ui::renderer::expressions::detect_emotion_with_fallback(
            &chat_res.message.content,
        );
        let final_reply = clean_ai_text(&chat_res.message.content);

        if !final_reply.is_empty() {
            on_sentence(&final_reply, emo)?;
        }

        self.history.push(ChatMessage {
            role: "assistant".to_string(),
            content: final_reply.clone(),
        });
        self.prune_history();

        let u = crate::config::load_user_config();
        crate::llm::history::log_conversation_turn(
            &u.user_callsign,
            &u.assistant_name,
            &u.personality_role,
            input_text,
            &final_reply,
        );

        Ok(chat_res.message.content)
    }

    /// Complete a prompt or message sequence silently without triggering audio playback or modifying chat history.
    pub fn complete_messages_silent(&mut self, messages: &[ChatMessage]) -> Result<String> {
        let _ = self.antigravity.load_accounts();
        let _ = self.codex.load_accounts();

        let is_codex_engine = self.provider.eq_ignore_ascii_case("codex")
            || self.provider.eq_ignore_ascii_case("openai-codex")
            || self.model.starts_with("cx/");

        if is_codex_engine && !self.codex.accounts.is_empty() {
            let mut dummy_cb = |_text: &str, _emo: &str| -> Result<()> { Ok(()) };
            if let Ok(reply) = self.codex.chat_stream(
                &self.model,
                messages,
                0.2,
                4096,
                &mut dummy_cb,
            ) {
                return Ok(reply);
            }
        }

        let is_local_engine = self.provider.eq_ignore_ascii_case("local")
            || self.provider.eq_ignore_ascii_case("antigravity");

        if is_local_engine && !self.antigravity.accounts.is_empty() {
            let mut dummy_cb = |_text: &str, _emo: &str| -> Result<()> { Ok(()) };
            if let Ok(reply) = self.antigravity.chat_stream(
                &self.model,
                messages,
                0.2, // Low temperature for deterministic planning
                4096,
                &mut dummy_cb,
            ) {
                return Ok(reply);
            }
        }

        // Custom OpenAI-compatible endpoint
        let custom_endpoint = if self.base_url.ends_with("/chat/completions") {
            self.base_url.clone()
        } else {
            format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
        };

        let req_body = ChatRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
            temperature: 0.2,
            max_tokens: 4096,
            stream: false,
            response_format: Some(serde_json::json!({ "type": "json_object" })),
        };

        let mut req_builder = self
            .client
            .post(&custom_endpoint)
            .header("Content-Type", "application/json");

        if let Some(ref key) = self.api_key {
            if !key.trim().is_empty() {
                req_builder = req_builder.header("Authorization", format!("Bearer {key}"));
            }
        }

        if let Ok(response) = req_builder.json(&req_body).send() {
            if response.status().is_success() {
                if let Ok(res_json) = response.json::<serde_json::Value>() {
                    if let Some(reply) = res_json["choices"][0]["message"]["content"].as_str() {
                        return Ok(reply.to_string());
                    }
                }
            }
        }

        // Fallback to Ollama native format
        let ollama_endpoint = if self.base_url.ends_with("/api/chat") {
            self.base_url.clone()
        } else {
            format!("{}/api/chat", self.base_url.trim_end_matches('/'))
        };

        let req_body = OllamaChatRequest {
            model: self.model.clone(),
            messages: messages.to_vec(),
            stream: false,
            format: Some("json".to_string()),
        };

        let res = self
            .client
            .post(&ollama_endpoint)
            .json(&req_body)
            .send()
            .map_err(|e| anyhow::anyhow!("Failed to connect to Ollama ({ollama_endpoint}): {e}"))?;

        let chat_res: OllamaChatResponse = res.json().context("Failed to parse Ollama response")?;
        Ok(chat_res.message.content)
    }
}
