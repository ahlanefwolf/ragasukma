use super::cleaner::clean_ai_text;
use super::prompt::build_system_prompt;
use super::providers::antigravity::AntigravityProvider;
use super::types::{ChatMessage, ChatRequest, OllamaChatRequest, OllamaChatResponse};
use anyhow::{Context, Result};

pub struct LlmClient {
    pub provider: String,
    pub base_url: String,
    pub api_key: Option<String>,
    pub model: String,
    pub client: reqwest::blocking::Client,
    pub history: Vec<ChatMessage>,
    pub antigravity: AntigravityProvider,
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
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .unwrap_or_default();

        let system_prompt = build_system_prompt();

        let history = vec![ChatMessage {
            role: "system".to_string(),
            content: system_prompt,
        }];

        let antigravity = AntigravityProvider::new();

        Self {
            provider,
            base_url,
            api_key,
            model,
            client,
            history,
            antigravity,
        }
    }

    pub fn reset_history(&mut self) {
        if let Some(system_msg) = self.history.first_mut() {
            system_msg.content = build_system_prompt();
        }
    }

    pub fn chat_stream<F>(&mut self, user_input: &str, mut on_sentence: F) -> Result<String>
    where
        F: FnMut(&str, &str) -> Result<()>,
    {
        if user_input.trim().is_empty() {
            return Ok(String::new());
        }

        // Selalu update System Prompt pesan pertama agar tab browser & skill selalu fresh
        let current_sys_prompt = build_system_prompt();
        if let Some(first_msg) = self.history.first_mut() {
            if first_msg.role == "system" {
                first_msg.content = current_sys_prompt;
            }
        }

        self.history.push(ChatMessage {
            role: "user".to_string(),
            content: user_input.to_string(),
        });

        // 1. Direct Antigravity Provider jika mode antigravity atau ada akun terdaftar
        let is_ag_mode = self.provider.eq_ignore_ascii_case("antigravity")
            || self.model.starts_with("ag/")
            || (self.api_key.is_none() && !self.antigravity.accounts.is_empty());

        if is_ag_mode && !self.antigravity.accounts.is_empty() {
            match self.antigravity.chat_stream(
                &self.model,
                &self.history,
                0.7,
                8192,
                &mut on_sentence,
            ) {
                Ok(reply) => {
                    self.history.push(ChatMessage {
                        role: "assistant".to_string(),
                        content: clean_ai_text(&reply),
                    });
                    return Ok(reply);
                }
                Err(e) => {
                    crate::term_err!("⚠️ [LLM Client] Antigravity error: {e}. Fallback ke provider berikutnya...");
                }
            }
        }

        // 2. OpenAI / 9router compatible proxy
        if let Some(ref key) = self.api_key {
            let endpoint = if self.base_url.ends_with("/chat/completions") {
                self.base_url.clone()
            } else if self.base_url.ends_with("/v1") {
                format!("{}/chat/completions", self.base_url.trim_end_matches('/'))
            } else {
                self.base_url.clone()
            };

            let req_body = ChatRequest {
                model: self.model.clone(),
                messages: self.history.clone(),
                temperature: 0.5,
                max_tokens: 160,
                stream: true, // Gunakan live streaming
            };

            use std::io::BufRead;
            let response = self
                .client
                .post(&endpoint)
                .header("Authorization", format!("Bearer {key}"))
                .header("Content-Type", "application/json")
                .json(&req_body)
                .send()
                .context("Gagal mengirim request streaming ke LLM (9router/OpenAI)")?;

            if !response.status().is_success() {
                let err_text = response.text().unwrap_or_default();
                anyhow::bail!("API Stream error: {err_text}");
            }

            let reader = std::io::BufReader::new(response);
            let mut full_reply = String::new();
            let mut current_sentence = String::new();
            let mut active_emotion = "neutral";

            for line in reader.lines() {
                let line = line?;
                let trimmed = line.trim();
                if let Some(data_str) = trimmed.strip_prefix("data: ") {
                    if data_str.trim() == "[DONE]" {
                        break;
                    }
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(data_str) {
                        // Abaikan field reasoning_content (CoT) jika ada
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

                            // Tangani buffering kalimat dan proteksi jika ada tag [SKILL: ...] yang belum tertutup
                            let has_skill_tag = current_sentence.contains("[SKILL:");
                            let has_skill_closed = has_skill_tag && current_sentence.contains(']');

                            if !has_skill_tag || has_skill_closed {
                                while let Some(pos) = current_sentence.find(['.', '?', '!', '\n']) {
                                    // Cegah pemotongan jika tanda baca berada di dalam tag skill
                                    if let Some(skill_idx) = current_sentence.find("[SKILL:") {
                                        if pos > skill_idx {
                                            break;
                                        }
                                    }
                                    let sentence = current_sentence[..=pos].to_string();
                                    current_sentence = current_sentence[pos + 1..].to_string();

                                    // Deteksi emosi dari teks mentah SEBELUM tag bintang dibersihkan
                                    if let Some(emo) =
                                        crate::ui::renderer::expressions::detect_emotion(&sentence)
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

            // Sisa potongan kalimat terakhir
            let clean_rest = clean_ai_text(&current_sentence);
            if !clean_rest.is_empty() {
                if let Some(emo) =
                    crate::ui::renderer::expressions::detect_emotion(&current_sentence)
                {
                    active_emotion = emo;
                }
                on_sentence(&clean_rest, active_emotion)?;
            }

            self.history.push(ChatMessage {
                role: "assistant".to_string(),
                content: clean_ai_text(&full_reply),
            });

            return Ok(full_reply);
        }

        // 3. Fallback Ollama Lokal
        self.chat_fallback_ollama(user_input, on_sentence)
    }

    fn chat_fallback_ollama<F>(&mut self, _user_input: &str, mut on_sentence: F) -> Result<String>
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
        };

        let res = self
            .client
            .post(&ollama_endpoint)
            .json(&req_body)
            .send()
            .map_err(|e| anyhow::anyhow!("Gagal menghubungi Ollama ({ollama_endpoint}): {e}"))?;

        if !res.status().is_success() {
            let err_text = res.text().unwrap_or_default();
            anyhow::bail!("Ollama error: {err_text}");
        }

        let chat_res: OllamaChatResponse = res.json().context("Gagal parse response Ollama")?;
        let emo = crate::ui::renderer::expressions::detect_emotion_with_fallback(
            &chat_res.message.content,
        );
        let final_reply = clean_ai_text(&chat_res.message.content);

        if !final_reply.is_empty() {
            on_sentence(&final_reply, emo)?;
        }

        self.history.push(ChatMessage {
            role: "assistant".to_string(),
            content: final_reply,
        });
        Ok(chat_res.message.content)
    }
}
