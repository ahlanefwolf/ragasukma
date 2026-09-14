use anyhow::{Context, Result};
use reqwest::blocking::multipart;
use serde::Deserialize;
use std::io::Write;

pub enum SttEngine {
    Groq {
        api_key: String,
        client: reqwest::blocking::Client,
    },
    Google {
        client: reqwest::blocking::Client,
    },
}

#[derive(Deserialize)]
struct GroqTranscriptionResponse {
    text: String,
}

impl SttEngine {
    pub fn new(engine_type: &str) -> Result<Self> {
        if engine_type.eq_ignore_ascii_case("groq") {
            let u = crate::config::load_user_config();
            let api_key = if !u.groq_api_key.trim().is_empty() {
                u.groq_api_key
            } else {
                std::env::var("GROQ_API_KEY").unwrap_or_default()
            };
            if api_key.trim().is_empty() {
                anyhow::bail!("GROQ_API_KEY tidak ditemukan di config-ai.json atau GUI Settings! Silakan isi GROQ_API_KEY untuk menggunakan mode Groq STT.");
            }
            let client = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(15))
                .build()?;
            crate::term_log!("☁️  Engine STT: GROQ Cloud API (whisper-large-v3, Ultra Fast) siap.");
            return Ok(Self::Groq { api_key, client });
        }

        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()?;
        crate::term_log!("🌐 Engine STT: GOOGLE Speech API (Free, No API Key, Bahasa Indonesia) siap.");
        Ok(Self::Google { client })
    }

    pub fn transcribe(&self, audio_data: &[f32]) -> Result<String> {
        if audio_data.is_empty() {
            return Ok(String::new());
        }

        match self {
            Self::Google { client } => {
                let wav_bytes = encode_pcm_to_wav(audio_data, 16000)?;
                let url = "https://www.google.com/speech-api/v2/recognize?client=chromium&lang=id-ID&key=AIzaSyBOti4mM-6x9WDnZIjIeyEU21OpBXqWBgw&pFilter=0";

                let res = client
                    .post(url)
                    .header("Content-Type", "audio/l16; rate=16000")
                    .header("User-Agent", "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
                    .body(wav_bytes)
                    .send()
                    .context("Gagal menghubungi Google Speech Recognition API")?;

                if !res.status().is_success() {
                    let status = res.status();
                    let err_text = res.text().unwrap_or_default();
                    crate::term_err!("⚠️ [Google Speech HTTP {}]: {}", status, err_text);
                    anyhow::bail!("Google Speech API Error: {err_text}");
                }

                let body = res.text().unwrap_or_default();
                for line in body.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(trimmed) {
                        if let Some(transcript) = val
                            .pointer("/result/0/alternative/0/transcript")
                            .and_then(|t| t.as_str())
                        {
                            return Ok(transcript.trim().to_string());
                        }
                    }
                }
                Ok(String::new())
            }
            Self::Groq { api_key, client } => {
                let wav_bytes = encode_pcm_to_wav(audio_data, 16000)?;
                let part = multipart::Part::bytes(wav_bytes)
                    .file_name("audio.wav")
                    .mime_str("audio/wav")?;

                let form = multipart::Form::new()
                    .text("model", "whisper-large-v3")
                    .text("language", "id")
                    .text("prompt", "Obrolan santai, percakapan bahasa Indonesia, Linux, Fedora, dan coding.")
                    .text("temperature", "0.0")
                    .part("file", part);

                let res = client
                    .post("https://api.groq.com/openai/v1/audio/transcriptions")
                    .bearer_auth(api_key)
                    .multipart(form)
                    .send()
                    .context("Gagal menghubungi endpoint Groq Whisper API")?;

                if !res.status().is_success() {
                    let err_text = res.text().unwrap_or_default();
                    anyhow::bail!("Groq API Error: {err_text}");
                }

                let parsed: GroqTranscriptionResponse = res.json().context("Gagal parse response Groq STT")?;
                Ok(parsed.text.trim().to_string())
            }
        }
    }
}

pub fn encode_pcm_to_wav(samples: &[f32], sample_rate: u32) -> Result<Vec<u8>> {
    let mut buffer = Vec::with_capacity(44 + samples.len() * 2);

    let num_samples = samples.len() as u32;
    let data_len = num_samples * 2; // 16-bit = 2 bytes per sample
    let file_len = 36 + data_len;

    // RIFF header
    buffer.write_all(b"RIFF")?;
    buffer.write_all(&file_len.to_le_bytes())?;
    buffer.write_all(b"WAVE")?;

    // fmt chunk
    buffer.write_all(b"fmt ")?;
    buffer.write_all(&16u32.to_le_bytes())?; // Chunk size
    buffer.write_all(&1u16.to_le_bytes())?;  // Audio format 1 = PCM
    buffer.write_all(&1u16.to_le_bytes())?;  // 1 channel (mono)
    buffer.write_all(&sample_rate.to_le_bytes())?;
    let byte_rate = sample_rate * 2;
    buffer.write_all(&byte_rate.to_le_bytes())?;
    buffer.write_all(&2u16.to_le_bytes())?;  // Block align
    buffer.write_all(&16u16.to_le_bytes())?; // Bits per sample

    // data chunk
    buffer.write_all(b"data")?;
    buffer.write_all(&data_len.to_le_bytes())?;

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let sample_i16 = (clamped * 32767.0) as i16;
        buffer.write_all(&sample_i16.to_le_bytes())?;
    }

    Ok(buffer)
}
