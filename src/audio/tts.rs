use anyhow::Result;
use crate::tray::{AssistantState, TrayState};

pub struct TextToSpeech;

impl TextToSpeech {
    pub fn new() -> Self {
        Self
    }

    /// Sintesis suara dialirkan langsung ke ekosistem Google Chrome Speech Synthesis (Web Speech API)
    /// Bebas dari depedensi engine TTS lokal maupun API eksternal pihak ketiga.
    pub fn speak_with_emotion(&self, text: &str, emotion: &str, asst_name: &str) -> Result<()> {
        let clean_text = clean_text_for_speech(text);
        if clean_text.trim().is_empty() {
            return Ok(());
        }

        if crate::app_state::is_voice_muted() {
            return Ok(());
        }

        // 1. Beritahu status avatar & emosi untuk sinkronisasi visual bibir
        TrayState::update_with_emotion(
            AssistantState::Speaking,
            &format!("{asst_name} Berbicara"),
            emotion,
        );
        crate::app_state::set_ai_speaking(true);

        // 2. Alirkan kalimat respon dan emosi ke Google Chrome Speech Synthesis via SSE action
        let payload = serde_json::json!({
            "text": clean_text,
            "emotion": emotion
        })
        .to_string();
        crate::app_state::push_client_action("assistant:speak", Some(&payload));

        Ok(())
    }
}

/// Memecah teks menjadi potongan kalimat secara cerdas:
/// Melindungi angka ribuan (misal: 1.000) dan pecahan desimal (misal: 0,5 atau 3.14)
/// agar tidak terputus di tengah angka.
#[allow(dead_code)]
pub fn split_sentences_aware(text: &str, split_on_comma: bool) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut sentences = Vec::new();
    let mut start = 0;

    for i in 0..len {
        let c = chars[i];
        let is_delimiter = match c {
            '!' | '?' | '\n' => true,
            ';' => true,
            '.' => {
                // JANGAN potong jika titik diapit oleh angka di kiri dan kanan (contoh: 1.000 atau 3.14)
                let prev_is_digit = i > 0 && chars[i - 1].is_ascii_digit();
                let next_is_digit = i + 1 < len && chars[i + 1].is_ascii_digit();
                !(prev_is_digit && next_is_digit)
            }
            ',' if split_on_comma => {
                // JANGAN potong jika koma diapit oleh angka di kiri dan kanan (contoh: 0,5 atau 12,5)
                let prev_is_digit = i > 0 && chars[i - 1].is_ascii_digit();
                let next_is_digit = i + 1 < len && chars[i + 1].is_ascii_digit();
                !(prev_is_digit && next_is_digit)
            }
            _ => false,
        };

        if is_delimiter {
            let slice: String = chars[start..=i].iter().collect();
            let trimmed = slice.trim();
            if !trimmed.is_empty() {
                sentences.push(trimmed.to_string());
            }
            start = i + 1;
        }
    }

    if start < len {
        let slice: String = chars[start..len].iter().collect();
        let trimmed = slice.trim();
        if !trimmed.is_empty() {
            sentences.push(trimmed.to_string());
        }
    }

    if sentences.is_empty() && !text.trim().is_empty() {
        vec![text.trim().to_string()]
    } else {
        sentences
    }
}

pub fn clean_text_for_speech(text: &str) -> String {
    let mut cleaned = String::with_capacity(text.len());
    let mut in_asterisk = false;
    let mut in_bracket = false;
    let mut in_paren = false;

    // 1. Filter karakter, buang teks dalam tanda bintang (*emosi*), kurung siku [ACTION:...], dan kurung biasa (keterangan)
    for c in text.chars() {
        if c == '*' {
            in_asterisk = !in_asterisk;
            continue;
        }
        if c == '[' {
            in_bracket = true;
            continue;
        }
        if c == ']' {
            in_bracket = false;
            continue;
        }
        if c == '(' {
            in_paren = true;
            continue;
        }
        if c == ')' {
            in_paren = false;
            continue;
        }
        if in_asterisk || in_bracket || in_paren {
            continue;
        }

        // Hapus simbol markdown / karakter non-bicara / emoji
        match c {
            '#' | '`' | '_' | '~' | '^' | '|' | '<' | '>' | '—' | '–' => cleaned.push(' '),
            '?' => cleaned.push('.'),
            c if c.is_ascii_punctuation() || c.is_alphanumeric() || c.is_whitespace() => {
                cleaned.push(c);
            }
            _ => {} // buang emoji / karakter tak dikenal
        }
    }

    // 2. Normalisasi titik berturutan atau elipsis menjadi koma agar intonasi TTS lancar
    let formatted = cleaned
        .replace(". . .", ", ")
        .replace(". .", ", ")
        .replace("....", ", ")
        .replace("...", ", ")
        .replace("..", ", ")
        .replace("  ", " ");

    let mut result = formatted.trim().to_string();

    // 3. Tangkal bug kata emosi mentah di paling awal kalimat (misal: "malu Halo...", "senang Halo...", "malu, Halo...")
    let emotion_prefixes = [
        "malu",
        "blush",
        "senang",
        "gembira",
        "bingung",
        "kaget",
        "sedih",
        "semangat",
        "tertawa",
        "tersenyum",
        "tersipu",
        "panik",
        "marah",
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

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_sentences_aware_preserves_numbers() {
        let text = "Harga barangnya Rp 1.500.000 dan diskonnya 0,5 persen. Apakah kamu setuju?";

        // Mode kalimat utuh (tanpa potong koma biasa)
        let sentences = split_sentences_aware(text, false);
        assert_eq!(sentences.len(), 2);
        assert_eq!(sentences[0], "Harga barangnya Rp 1.500.000 dan diskonnya 0,5 persen.");
        assert_eq!(sentences[1], "Apakah kamu setuju?");

        // Mode koma aktif
        let clauses = split_sentences_aware(text, true);
        assert_eq!(clauses.len(), 2);
        assert_eq!(clauses[0], "Harga barangnya Rp 1.500.000 dan diskonnya 0,5 persen.");
        assert_eq!(clauses[1], "Apakah kamu setuju?");

        // Teks dengan koma di luar angka
        let text_with_comma = "Total 1.000 unit, dan 0,5 persen cacat. Tolong cek!";
        let clauses2 = split_sentences_aware(text_with_comma, true);
        assert_eq!(clauses2.len(), 3);
        assert_eq!(clauses2[0], "Total 1.000 unit,");
        assert_eq!(clauses2[1], "dan 0,5 persen cacat.");
        assert_eq!(clauses2[2], "Tolong cek!");
    }
}
