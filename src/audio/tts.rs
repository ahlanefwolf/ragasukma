use crate::tray::{AssistantState, TrayState};
use anyhow::Result;

pub struct TextToSpeech;

impl TextToSpeech {
    pub fn new() -> Self {
        Self
    }

    /// Sintesis suara diproses secara native menggunakan model neural Supertonic-3 (ONNX Runtime Rust).
    /// Audio WAV berkualitas tinggi (44.1kHz Hi-Fi) dialirkan ke semua client (Desktop & Mobile).
    pub fn speak_with_emotion(&self, text: &str, emotion: &str, asst_name: &str) -> Result<()> {
        let clean_text = clean_text_for_speech(text);
        if clean_text.trim().is_empty() {
            return Ok(());
        }

        // Catatan: is_voice_muted menandakan speaker desktop dimatikan (dialihkan ke mobile),
        // namun event suara 'assistant:speak' harus tetap dipancarkan agar PWA HP dapat memainkannya!

        let cfg = crate::assets::load_config();
        let voice = if !cfg.tts_voice_name.is_empty() {
            cfg.tts_voice_name.clone()
        } else {
            "F4".to_string()
        };
        let lang = if !cfg.lang.is_empty() {
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

        // Inisialisasi engine jika belum, dan sintesis audio Supertonic-3 secara streaming chunk
        let mut audio_bytes_opt: Option<Vec<u8>> = None;
        let asst_name_str = asst_name.to_string();
        let emotion_str = emotion.to_string();

        let mut on_chunk = |chunk_idx: usize, total_chunks: usize, chunk_wav: Vec<u8>| {
            if !crate::app_state::is_voice_muted() {
                // Begitu chunk pertama siap, langsung ubah status avatar & tray menjadi Speaking
                if chunk_idx == 0 {
                    crate::tray::TrayState::update_with_emotion(
                        crate::services::tray_service::AssistantState::Speaking,
                        &format!("{asst_name_str} Berbicara"),
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
                    crate::term_err!("⚠️ [Supertonic TTS] Failed to synthesize audio: {e}");
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

        // 1. Alirkan kalimat respon, emosi, durasi, dan audio_url ke semua client (Desktop & Mobile) via SSE action
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

        // 2. Sinkronisasi status avatar & bibir AIUEO:
        if native_played {
            TrayState::update_with_emotion(
                AssistantState::Speaking,
                &format!("{asst_name} Berbicara"),
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
                    &format!("{asst_name} Berbicara"),
                    emotion,
                );
                crate::app_state::set_ai_speaking(true);
            }
        }

        // 3. Passive fail-safe watchdog presisi berbasis durasi aktual WAV:
        // Dihitung langsung dari panjang detik audio WAV hasil sintesis Supertonic-3 neural engine.
        let epoch = crate::app_state::next_speaking_epoch();
        crate::app_state::touch_speech_activity();

        let exact_audio_secs = actual_audio_duration.unwrap_or_else(|| {
            (clean_text.chars().count() as f32 / 10.0).max(5.0)
        });
        // Grace period toleransi latensi jaringan & buffering audio player (~6 detik)
        let timeout_limit_secs = (exact_audio_secs + 6.0).ceil() as u64;

        std::thread::spawn(move || {
            let start = std::time::Instant::now();
            loop {
                std::thread::sleep(std::time::Duration::from_millis(400));

                // Jika sudah selesai bicara secara normal atau ada epoch bicara baru, keluar
                if !crate::app_state::is_ai_speaking()
                    || crate::app_state::current_speaking_epoch() != epoch
                {
                    break;
                }

                // Jika audio native PC masih aktif berputar, jangan pernah force idle!
                if crate::audio::player::is_native_voice_playing() {
                    crate::app_state::touch_speech_activity();
                    continue;
                }

                // Jika client masih mengirim aktivitas suara dalam 3 detik terakhir, perpanjang waktu
                if crate::app_state::last_speech_activity_elapsed().as_secs() < 3 {
                    continue;
                }

                // Jika durasi melebihi durasi audio aktual + buffer toleransi
                if start.elapsed().as_secs() >= timeout_limit_secs {
                    crate::term_log!(
                        "⏱️ [TTS Watchdog] Audio playback finished ({exact_audio_secs:.1}s), returning avatar to Standby."
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
            output.push_str("tautan web");
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
    // Angka referensi footnote misal (1), (2)
    if lower.chars().all(|c| c.is_ascii_digit()) {
        return true;
    }
    // Emoticon smiley
    matches!(lower.as_str(), ":)" | ":d" | ":3" | "^^" | "^_^" | ":o" | ":(" | ";)")
}

pub fn clean_text_for_speech(text: &str) -> String {
    if text.trim().is_empty() {
        return String::new();
    }

    // 1. Normalisasi karakter Unicode khusus (kutip lengkung, elipsis, dash, bullet)
    let pre_normalized = text
        .replace(['\u{201C}', '\u{201D}', '\u{00AB}', '\u{00BB}'], "\"") // “ ” « » -> "
        .replace(['\u{2018}', '\u{2019}'], "'") // ‘ ’ -> '
        .replace('\u{2026}', ", ") // … -> ,
        .replace(['\u{2014}', '\u{2013}'], " - ") // — – -> -
        .replace(['\u{2022}', '\u{25AA}', '\u{25CF}'], " "); // • ▪ ● -> spasi

    // 2. Bersihkan blok kode markdown ```...``` agar TTS tidak mengeja sintaks kode
    let without_code_blocks = remove_code_blocks(&pre_normalized);

    // 3. Sanitasi URL mentah (https://... / http://... / www...) menjadi "tautan web"
    let without_urls = sanitize_urls(&without_code_blocks);

    // 4. Manfaatkan clean_ai_text untuk membuang tool calls [CMD:...], [SKILL:...], bold **teks**, tag bintang *...*, dan backticks
    let cleaned_base = crate::llm::cleaner::clean_ai_text(&without_urls);

    // 5. Bersihkan tag bracket [happy], link markdown [label](url), kurung ignorable (1), dan saring tanda baca
    let mut cleaned = String::with_capacity(cleaned_base.len());
    let mut chars = cleaned_base.chars().peekable();

    while let Some(c) = chars.next() {
        // A. Bersihkan tag kurung siku [happy], link [label](url), dsb.
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
                // Jika tag kurung siku diikuti markdown link `(url)`, pertahankan teks label dan abaikan url
                if chars.peek() == Some(&'(') {
                    chars.next(); // konsumsi '('
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

        // B. Bersihkan kurung biasa () jika berisi nomor referensi footnote (1) atau emoticon
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
                // Jika bukan kurung ignorable, lewati kurung bukanya saja agar teks di dalamnya tetap terbaca
                cleaned.push(' ');
                continue;
            }
        }
        if c == ')' {
            cleaned.push(' ');
            continue;
        }

        // C. Normalisasi karakter markdown non-bicara & tanda baca
        match c {
            '#' | '`' | '_' | '~' | '^' | '|' | '<' | '>' | '*' => cleaned.push(' '),
            '?' => cleaned.push('.'),
            c if c.is_ascii_punctuation() || c.is_alphanumeric() || c.is_whitespace() => {
                cleaned.push(c);
            }
            _ => {} // buang emoji / karakter tak dikenal
        }
    }

    // 5. Normalisasi titik berturutan atau elipsis menjadi koma agar intonasi TTS lancar
    let formatted = cleaned
        .replace(". . .", ", ")
        .replace(". .", ", ")
        .replace("....", ", ")
        .replace("...", ", ")
        .replace("..", ", ")
        .replace("  ", " ");

    let mut result = formatted.trim().to_string();

    // 6. Tangkal bug kata emosi mentah di paling awal kalimat (misal: "happy Halo...", "senang, Halo...", "mikir: Halo...")
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
                // Cek apakah diikuti spasi, titik dua, koma, atau tanda strip
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

    // 7. Collapse whitespace berulang & rapikan tanda baca
    let mut final_result = String::with_capacity(result.len());
    let mut last_was_space = false;
    for c in result.chars() {
        if c.is_whitespace() {
            if !last_was_space {
                final_result.push(' ');
                last_was_space = true;
            }
        } else {
            // Hindari spasi sebelum titik/koma (misal: "Halo , teman" -> "Halo, teman")
            if (c == ',' || c == '.' || c == '!') && last_was_space && !final_result.is_empty() {
                final_result.pop();
            }
            final_result.push(c);
            last_was_space = false;
        }
    }

    final_result.trim().to_string()
}

/// Menghitung durasi detik secara presisi dari file audio WAV Supertonic-3
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

/// Helper global untuk berbicara suara asisten secara reaktif dan non-blocking
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
            "Kunjungi tautan web untuk info."
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

