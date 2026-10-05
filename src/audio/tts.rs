use crate::tray::{AssistantState, TrayState};
use anyhow::Result;

pub struct TextToSpeech;

impl TextToSpeech {
    pub fn new() -> Self {
        Self
    }

    /// Voice synthesis is processed natively using Supertonic-3 neural model (ONNX Runtime Rust).
    /// High-fidelity WAV audio (44.1kHz Hi-Fi) is streamed to all clients (Desktop & Mobile).
    pub fn speak_with_emotion(&self, text: &str, emotion: &str, asst_name: &str) -> Result<()> {
        self.speak_with_emotion_and_lang(text, emotion, asst_name, None)
    }

    /// Allows overriding the spoken language (e.g. explicitly forcing "en" for error notices).
    pub fn speak_with_emotion_and_lang(
        &self,
        text: &str,
        emotion: &str,
        asst_name: &str,
        lang_override: Option<&str>,
    ) -> Result<()> {
        let clean_text = clean_text_for_speech(text);
        if clean_text.trim().is_empty() {
            return Ok(());
        }

        // Notes: is_voice_muted indicates desktop speakers are muted (redirected to mobile),
        // however 'assistant:speak' voice events must still be broadcast so the mobile client can play them!

        let cfg = crate::assets::load_config();
        let voice = if !cfg.tts_voice_name.is_empty() {
            cfg.tts_voice_name.clone()
        } else {
            "F4".to_string()
        };
        let lang = if let Some(override_code) = lang_override {
            override_code.to_string()
        } else if !cfg.lang.is_empty() {
            cfg.lang.clone()
        } else {
            "id".to_string()
        };
        let speed = if cfg.tts_speed > 0.0 {
            cfg.tts_speed
        } else {
            1.05
        };

        let audio_id = format!("{:x}", simple_hash_u64(&clean_text));
        let audio_url = format!("/api/tts/audio?id={audio_id}");

        // Initialize engine if not yet ready, and synthesize Supertonic-3 audio in streaming chunks
        let mut audio_bytes_opt: Option<Vec<u8>> = None;
        let asst_name_str = asst_name.to_string();
        let emotion_str = emotion.to_string();

        let mut on_chunk = |chunk_idx: usize, total_chunks: usize, chunk_wav: Vec<u8>| {
            if !crate::app_state::is_voice_muted() {
                // Once first chunk is ready, directly set avatar & tray status to Speaking
                if chunk_idx == 0 {
                    crate::tray::TrayState::update_with_emotion(
                        crate::services::tray_service::AssistantState::Speaking,
                        &format!("{asst_name_str} Speaking"),
                        &emotion_str,
                    );
                    crate::app_state::set_ai_speaking(true);
                }
                crate::audio::player::play_native_voice_chunk(chunk_idx, total_chunks, chunk_wav);
            }
        };

        let engine_opt = crate::audio::supertonic::get_engine()
            .or_else(|| crate::audio::supertonic::init_engine().ok());

        let synthesized = if let Some(engine) = engine_opt {
            match engine.synthesize_stream(&clean_text, &voice, &lang, speed, &mut on_chunk) {
                Ok(bytes) => {
                    crate::audio::supertonic::set_last_audio_wav(audio_id.clone(), bytes.clone());
                    audio_bytes_opt = Some(bytes);
                    true
                }
                Err(e) => {
                    crate::term_err!("[Supertonic TTS] Failed to synthesize audio: {e}");
                    false
                }
            }
        } else {
            false
        };

        let actual_audio_duration = audio_bytes_opt
            .as_ref()
            .and_then(|bytes| wav_duration_secs(bytes));

        let native_played = synthesized && !crate::app_state::is_voice_muted();

        crate::app_state::record_last_spoken_text(&clean_text);

        // 1. Stream response sentence, emotion, duration, and audio_url to all clients (Desktop & Mobile) via SSE action
        let payload = serde_json::json!({
            "text": clean_text,
            "emotion": emotion,
            "audio_url": if synthesized { Some(audio_url) } else { None },
            "duration": actual_audio_duration,
            "voice": voice,
            "lang": lang,
            "native_played": native_played
        })
        .to_string();
        crate::app_state::push_client_action("assistant:speak", Some(&payload));

        // 2. Synchronize avatar state & AIUEO lip sync:
        if native_played {
            TrayState::update_with_emotion(
                AssistantState::Speaking,
                &format!("{asst_name} Speaking"),
                emotion,
            );
            crate::app_state::set_ai_speaking(true);
        } else {
            let has_web_client = crate::app_state::ACTIVE_WEB_SESSION
                .lock()
                .ok()
                .and_then(|s| s.as_ref().map(|_| true))
                .unwrap_or(false);

            if !has_web_client {
                TrayState::update_with_emotion(
                    AssistantState::Speaking,
                    &format!("{asst_name} Speaking"),
                    emotion,
                );
                crate::app_state::set_ai_speaking(true);
            }
        }

        // 3. Precision passive fail-safe watchdog based on actual WAV duration:
        // Calculated directly from duration in seconds of synthesized Supertonic-3 WAV audio.
        let epoch = crate::app_state::next_speaking_epoch();
        crate::app_state::touch_speech_activity();

        let exact_audio_secs = actual_audio_duration.unwrap_or_else(|| {
            (clean_text.chars().count() as f32 / 10.0).max(5.0)
        });
        // Network latency and audio player buffering tolerance grace period (~6 seconds)
        let timeout_limit_secs = (exact_audio_secs + 6.0).ceil() as u64;

        std::thread::spawn(move || {
            let start = std::time::Instant::now();
            loop {
                std::thread::sleep(std::time::Duration::from_millis(400));

                // Exit if speech finished normally or a new speech epoch began
                if !crate::app_state::is_ai_speaking()
                    || crate::app_state::current_speaking_epoch() != epoch
                {
                    break;
                }

                // If native PC audio is still actively playing, never force idle!
                if crate::audio::player::is_native_voice_playing() {
                    crate::app_state::touch_speech_activity();
                    continue;
                }

                // If client still sent voice activity within the last 3 seconds, extend timeout
                if crate::app_state::last_speech_activity_elapsed().as_secs() < 3 {
                    continue;
                }

                // If duration exceeds actual audio duration plus tolerance buffer
                if start.elapsed().as_secs() >= timeout_limit_secs {
                    crate::term_log!(
                        "[TTS Watchdog] Audio playback finished ({exact_audio_secs:.1}s), returning avatar to Standby."
                    );
                    crate::app_state::set_ai_speaking(false);
                    crate::tray::TrayState::update_with_emotion(
                        crate::services::tray_service::AssistantState::Idle,
                        "Standby",
                        "neutral",
                    );
                    break;
                }
            }
        });

        Ok(())
    }
}



fn remove_code_blocks(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut in_code_block = false;
    let mut backtick_count = 0;

    for c in text.chars() {
        if c == '`' {
            backtick_count += 1;
            if backtick_count == 3 {
                in_code_block = !in_code_block;
                backtick_count = 0;
                output.push(' ');
            }
        } else {
            backtick_count = 0;
            if !in_code_block {
                output.push(c);
            }
        }
    }
    output
}

fn sanitize_urls(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    for word in text.split_whitespace() {
        let lower = word.to_lowercase();
        if lower.starts_with("http://")
            || lower.starts_with("https://")
            || lower.starts_with("www.")
        {
            let trailing_punct: String = word
                .chars()
                .rev()
                .take_while(|c| matches!(c, '.' | ',' | '!' | '?' | ';' | ':' | ')'))
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            if !output.is_empty() {
                output.push(' ');
            }
            output.push_str("web link");
            output.push_str(&trailing_punct);
        } else {
            if !output.is_empty() {
                output.push(' ');
            }
            output.push_str(word);
        }
    }
    output
}

fn is_ignorable_tag(tag: &str) -> bool {
    let lower = tag.trim().to_lowercase();
    if lower.starts_with("action:") || lower.starts_with("cmd:") || lower.starts_with("skill:") {
        return true;
    }
    matches!(
        lower.as_str(),
        "happy"
            | "shy"
            | "excited"
            | "surprised"
            | "thinking"
            | "sad"
            | "angry"
            | "neutral"
            | "active tab"
            | "active"
    ) || lower.chars().all(|c| c.is_ascii_digit())
}

fn is_roleplay_or_ignorable_paren(content: &str) -> bool {
    let lower = content.trim().to_lowercase();
    if lower.is_empty() {
        return true;
    }
    // Footnote reference numbers e.g. (1), (2)
    if lower.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    // Smiley emoticons
    matches!(lower.as_str(), ":)" | ":d" | ":3" | "^^" | "^_^" | ":o" | ":(" | ";)")
}

pub fn clean_text_for_speech(text: &str) -> String {
    if text.trim().is_empty() {
        return String::new();
    }

    // 1. Normalize special Unicode characters (curly quotes, ellipsis, dash, bullet)
    let pre_normalized = text
        .replace(['\u{201C}', '\u{201D}', '\u{00AB}', '\u{00BB}'], "\"") // “ ” « » -> "
        .replace(['\u{2018}', '\u{2019}'], "'") // ‘ ’ -> '
        .replace('\u{2026}', ", ") // … -> ,
        .replace(['\u{2014}', '\u{2013}'], " - ") // — – -> -
        .replace(['\u{2022}', '\u{25AA}', '\u{25CF}'], " "); // • ▪ ● -> space

    // 2. Strip markdown code blocks ```...``` so TTS does not read code syntax aloud
    let without_code_blocks = remove_code_blocks(&pre_normalized);

    // 3. Sanitize raw URLs (https://... / http://... / www...) into "web link"
    let without_urls = sanitize_urls(&without_code_blocks);

    // 4. Use clean_ai_text to remove tool calls [CMD:...], [SKILL:...], bold **text**, star tags *...*, and backticks
    let cleaned_base = crate::llm::cleaner::clean_ai_text(&without_urls);

    // 5. Clean bracket tags [happy], markdown links [label](url), ignorable parentheses (1), and filter punctuation
    let mut cleaned = String::with_capacity(cleaned_base.len());
    let mut chars = cleaned_base.chars().peekable();

    while let Some(c) = chars.next() {
        // A. Clean square bracket tags [happy], links [label](url), etc.
        if c == '[' {
            let mut tag_content = String::new();
            let mut closed = false;
            while let Some(&sc) = chars.peek() {
                chars.next();
                if sc == ']' {
                    closed = true;
                    break;
                }
                tag_content.push(sc);
            }
            if closed {
                // If square brackets are followed by a markdown link `(url)`, preserve label text and omit the url
                if chars.peek() == Some(&'(') {
                    chars.next(); // consume '('
                    while let Some(&uc) = chars.peek() {
                        chars.next();
                        if uc == ')' {
                            break;
                        }
                    }
                    cleaned.push_str(&tag_content);
                } else if is_ignorable_tag(&tag_content) {
                    cleaned.push(' ');
                } else {
                    cleaned.push_str(&tag_content);
                }
                continue;
            } else {
                cleaned.push('[');
                cleaned.push_str(&tag_content);
                continue;
            }
        }

        // B. Clean parentheses () if they contain footnote reference numbers (1) or emoticons
        if c == '(' {
            let mut candidate = String::new();
            let mut found_close = false;
            let temp_chars = chars.clone();
            for sc in temp_chars {
                if sc == ')' {
                    found_close = true;
                    break;
                }
                if sc == '\n' || candidate.len() > 60 {
                    break;
                }
                candidate.push(sc);
            }

            if found_close && is_roleplay_or_ignorable_paren(&candidate) {
                for sc in chars.by_ref() {
                    if sc == ')' {
                        break;
                    }
                }
                cleaned.push(' ');
                continue;
            } else {
                // If not ignorable parentheses, skip opening parenthesis so inner text remains readable
                cleaned.push(' ');
                continue;
            }
        }
        if c == ')' {
            cleaned.push(' ');
            continue;
        }

        // C. Normalize non-spoken markdown characters and punctuation
        match c {
            '#' | '`' | '_' | '~' | '^' | '|' | '<' | '>' | '*' => cleaned.push(' '),
            '?' => cleaned.push('.'),
            c if c.is_ascii_punctuation() || c.is_alphanumeric() || c.is_whitespace() => {
                cleaned.push(c);
            }
            _ => {} // remove emoji / unknown characters
        }
    }

    // 5. Normalize consecutive dots or ellipsis into comma for smooth TTS intonation
    let formatted = cleaned
        .replace(". . .", ", ")
        .replace(". .", ", ")
        .replace("....", ", ")
        .replace("...", ", ")
        .replace("..", ", ")
        .replace("  ", " ");

    let mut result = formatted.trim().to_string();

    // 6. Tangkal bug word emotion mentah in most start sentence (for example: "happy Halo...", "senang, Halo...", "mikir: Halo...")
    let emotion_prefixes = [
        "happy",
        "shy",
        "excited",
        "surprised",
        "thinking",
        "sad",
        "angry",
        "neutral",
    ];

    loop {
        let lower = result.to_lowercase();
        let mut matched = false;
        for prefix in &emotion_prefixes {
            if lower.starts_with(prefix) {
                let rest = &result[prefix.len()..];
                // Check whether followed by space, colon, comma, or dash
                if let Some(first_char) = rest.chars().next() {
                    if first_char.is_whitespace()
                        || first_char == ':'
                        || first_char == ','
                        || first_char == '-'
                    {
                        result = rest[first_char.len_utf8()..].trim_start().to_string();
                        matched = true;
                        break;
                    }
                }
            }
        }
        if !matched {
            break;
        }
    }

    // 7. Collapse repeated whitespace and clean up punctuation
    let mut final_result = String::with_capacity(result.len());
    let mut last_was_space = false;
    for c in result.chars() {
        if c.is_whitespace() {
            if !last_was_space {
                final_result.push(' ');
                last_was_space = true;
            }
        } else {
            // Avoid whitespace before period/comma (e.g. "Hello , friend" -> "Hello, friend")
            if (c == ',' || c == '.' || c == '!') && last_was_space && !final_result.is_empty() {
                final_result.pop();
            }
            final_result.push(c);
            last_was_space = false;
        }
    }

    final_result.trim().to_string()
}

/// Accurately calculates duration in seconds from a Supertonic-3 WAV audio file
pub fn wav_duration_secs(wav_bytes: &[u8]) -> Option<f32> {
    if wav_bytes.len() > 44 && &wav_bytes[0..4] == b"RIFF" && &wav_bytes[8..12] == b"WAVE" {
        let sample_rate = u32::from_le_bytes(wav_bytes[24..28].try_into().ok()?) as f32;
        let channels = u16::from_le_bytes(wav_bytes[22..24].try_into().ok()?) as f32;
        let bits_per_sample = u16::from_le_bytes(wav_bytes[34..36].try_into().ok()?) as f32;
        let bytes_per_sec = sample_rate * channels * (bits_per_sample / 8.0);
        if bytes_per_sec > 0.0 {
            let data_size = (wav_bytes.len().saturating_sub(44)) as f32;
            return Some(data_size / bytes_per_sec);
        }
    }
    None
}

fn simple_hash_u64(s: &str) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

/// Global helper for reactive and non-blocking assistant speech
pub fn speak_assistant(text: &str, emotion: &str) {
    let cfg = crate::assets::load_config();
    let asst_name = if !cfg.assistant_name.is_empty() {
        cfg.assistant_name
    } else {
        "Sukma".to_string()
    };
    let tts = TextToSpeech::new();
    let _ = tts.speak_with_emotion(text, emotion, &asst_name);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_text_for_speech_strips_emotion_tags_and_prefixes() {
        assert_eq!(
            clean_text_for_speech("[happy] Halo apa kabar!"),
            "Halo apa kabar!"
        );
        assert_eq!(
            clean_text_for_speech("excited: Halo apa kabar!"),
            "Halo apa kabar!"
        );
        assert_eq!(
            clean_text_for_speech("angry, kenapa begitu?"),
            "kenapa begitu."
        );
        assert_eq!(
            clean_text_for_speech("*thinking* sebentar ya"),
            "sebentar ya"
        );
        assert_eq!(
            clean_text_for_speech("*malu-malu* halo juga Kak"),
            "halo juga Kak"
        );
        assert_eq!(
            clean_text_for_speech("Iya dong (1) terima kasih"),
            "Iya dong terima kasih"
        );
    }

    #[test]
    fn test_clean_text_for_speech_preserves_bold_and_bullets() {
        assert_eq!(
            clean_text_for_speech("Ini **sangat penting** lho!"),
            "Ini sangat penting lho!"
        );
        assert_eq!(clean_text_for_speech("* Poin pertama"), "Poin pertama");
    }

    #[test]
    fn test_clean_text_for_speech_sanitizes_urls_and_code() {
        assert_eq!(
            clean_text_for_speech("Kunjungi https://github.com/hlnsptr untuk info."),
            "Kunjungi web link untuk info."
        );
        assert_eq!(
            clean_text_for_speech("Markdown [Dokumentasi](https://example.com) bagus."),
            "Markdown Dokumentasi bagus."
        );
        assert_eq!(
            clean_text_for_speech("Jalankan ```bash\ncargo test\n``` selesai."),
            "Jalankan selesai."
        );
    }

}
