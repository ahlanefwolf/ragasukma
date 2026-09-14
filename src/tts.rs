use anyhow::{Context, Result};
use rodio::{buffer::SamplesBuffer, Decoder, OutputStream, Sink, Source};
use std::io::Cursor;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

/// DSP Audio Post-Processing untuk Karakter Anime Imut:
/// 1. Low-cut EQ (Memotong bass berat bapak-bapak/robot di bawah 260Hz)
/// 2. Formant Peaking Filter & High-Air (Mencerahkan vokal di frekuensi ~3.8kHz - 5.5kHz)
/// 3. Harmonic Soft Warmth (Memberikan overtone lembut khas karakter anime)
/// 4. Subtle Room Ambience Reverb (Memberi sentuhan ruang studio anime alami)
/// 5. Soft-Knee Dynamic Limiter (Mencegah clipping & menjaga volume tetap seimbang)
fn apply_anime_dsp_filter(samples: &[f32], sample_rate: u32) -> Vec<f32> {
    if samples.is_empty() || sample_rate == 0 {
        return samples.to_vec();
    }

    let dt = 1.0 / (sample_rate as f32);
    let mut out = Vec::with_capacity(samples.len());

    // 1. High-Pass Filter (Low-Cut di 260Hz untuk membuang frekuensi dada/berat)
    let cutoff_hp = 260.0f32;
    let rc_hp = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hp);
    let alpha_hp = rc_hp / (rc_hp + dt);

    let mut prev_in = samples[0];
    let mut prev_out_hp = samples[0];

    // Buffer Reverb/Delay sederhana (32ms delay + 0.16 feedback untuk ambience anime)
    let delay_samples = ((sample_rate as f32) * 0.032) as usize;
    let mut delay_buf = vec![0.0f32; delay_samples.max(1)];
    let mut delay_idx = 0;

    // 2. Formant & Air High-Shelf Booster di ~3.8kHz (Memberi kesan suara imut, renyah, dan cerah)
    let boost_gain = 1.45f32;
    let cutoff_shelf = 3800.0f32;
    let rc_shelf = 1.0 / (2.0 * std::f32::consts::PI * cutoff_shelf);
    let alpha_shelf = dt / (rc_shelf + dt);
    let mut lp_shelf = 0.0f32;

    for &s in samples {
        // Step 1: High-pass filter (Low-cut)
        let hp = alpha_hp * (prev_out_hp + s - prev_in);
        prev_in = s;
        prev_out_hp = hp;

        // Step 2: Formant High-Presence (Treble & Air boost)
        lp_shelf += alpha_shelf * (hp - lp_shelf);
        let high_part = hp - lp_shelf;
        let enhanced_voice = lp_shelf + (high_part * boost_gain);

        // Step 3: Soft Harmonic Warmth (Overtone halus)
        let warmed = (enhanced_voice * 1.08).tanh();

        // Step 4: Ambience Reverb Studio
        let delayed = delay_buf[delay_idx];
        let wet = warmed + (delayed * 0.16);
        delay_buf[delay_idx] = wet;
        delay_idx = (delay_idx + 1) % delay_buf.len();

        // Step 5: Soft-Knee Limiter / Normalization
        let mixed = warmed * 0.88 + delayed * 0.12;
        let final_sample = mixed.clamp(-0.95, 0.95);
        out.push(final_sample);
    }

    out
}

pub struct TextToSpeech {
    playback_speed: f32,
    http_client: reqwest::blocking::Client,
    tts_engine: String,
    tts_lang: String,
    elevenlabs_api_key: Option<String>,
    elevenlabs_voice_id: String,
    elevenlabs_disabled: Arc<AtomicBool>,
}

impl TextToSpeech {
    pub fn new() -> Self {
        let u = crate::config::load_user_config();
        let playback_speed = u.tts_speed;
        let tts_engine = u.tts_engine.to_lowercase();
        let tts_lang = if !u.tts_lang.trim().is_empty() {
            u.tts_lang.trim().to_string()
        } else {
            "id".to_string()
        };

        let elevenlabs_api_key = if !u.tts_eleven_labs_key.trim().is_empty() {
            Some(u.tts_eleven_labs_key.trim().to_string())
        } else {
            None
        };

        let elevenlabs_voice_id = if !u.elevenlabs_voice_id.trim().is_empty() {
            u.elevenlabs_voice_id
        } else {
            "cgSgspJ2msm6clMCkdW9".to_string()
        };

        let http_client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();

        let elevenlabs_disabled = Arc::new(AtomicBool::new(false));

        Self {
            playback_speed,
            http_client,
            tts_engine,
            tts_lang,
            elevenlabs_api_key,
            elevenlabs_voice_id,
            elevenlabs_disabled,
        }
    }

    pub fn speak_interruptible(&self, text: &str, is_interrupted: Arc<AtomicBool>) -> Result<()> {
        let clean_text = clean_text_for_speech(text);
        if clean_text.trim().is_empty() || is_interrupted.load(Ordering::SeqCst) {
            return Ok(());
        }

        if crate::native::server::is_voice_muted() {
            crate::tray::TrayState::update(crate::tray::AssistantState::Speaking, &clean_text);
            return Ok(());
        }

        crate::tray::TrayState::update(crate::tray::AssistantState::Speaking, &clean_text);

        let mut played_successfully = false;

        // 1. Coba synthesize via ElevenLabs jika API Key diset dan belum dinonaktifkan karena kuota habis/kritis
        if !played_successfully && self.tts_engine == "elevenlabs" {
            if let Some(ref api_key) = self.elevenlabs_api_key {
                if !self.elevenlabs_disabled.load(Ordering::Relaxed) {
                    match self.synthesize_elevenlabs(&clean_text, api_key) {
                        Ok(audio_bytes) => {
                            // Terapkan DSP Anime Filter (Formant Treble Booster + Pitch Shaper)
                            if let Err(e) = self.play_audio_buffer(audio_bytes, is_interrupted.clone(), true) {
                                crate::term_err!("⚠️ [ElevenLabs Playback Error]: {}", e);
                            } else {
                                played_successfully = true;
                            }
                        }
                        Err(e) => {
                            let err_str = e.to_string();
                            if err_str.contains("quota_exceeded") || err_str.contains("credits remaining") || err_str.contains("Status 401") || err_str.contains("Status 429") {
                                crate::term_err!("⚠️ [ElevenLabs Kuota Habis/Kritis] Menonaktifkan ElevenLabs untuk sesi ini, langsung beralih ke Google TTS.");
                                self.elevenlabs_disabled.store(true, Ordering::Relaxed);
                            } else {
                                crate::term_err!("⚠️ [ElevenLabs Fallback -> Google TTS]: {}", err_str);
                            }
                        }
                    }
                }
            }
        }

        // 3. Seamless Fallback ke Google Translate TTS + Custom DSP Anime
        let res = if !played_successfully && !is_interrupted.load(Ordering::SeqCst) {
            self.play_google_stream(&clean_text, is_interrupted)
        } else {
            Ok(())
        };

        res
    }

    fn synthesize_elevenlabs(&self, text: &str, api_key: &str) -> Result<Vec<u8>> {
        let url = format!(
            "https://api.elevenlabs.io/v1/text-to-speech/{}/stream?optimize_streaming_latency=3",
            self.elevenlabs_voice_id
        );

        let body = serde_json::json!({
            "text": text,
            "model_id": "eleven_multilingual_v2",
            "voice_settings": {
                "stability": 0.50,
                "similarity_boost": 0.80,
                "style": 0.20,
                "use_speaker_boost": true
            }
        });

        let resp = self
            .http_client
            .post(&url)
            .header("xi-api-key", api_key)
            .header("Content-Type", "application/json")
            .header("Accept", "audio/mpeg")
            .json(&body)
            .send()
            .context("Gagal menghubungi endpoint ElevenLabs")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let err_body = resp.text().unwrap_or_default();
            anyhow::bail!("Status {status}: {err_body}");
        }

        let bytes = resp.bytes().context("Gagal membaca audio bytes ElevenLabs")?;
        Ok(bytes.to_vec())
    }

    fn play_audio_buffer(&self, raw_bytes: Vec<u8>, is_interrupted: Arc<AtomicBool>, apply_dsp: bool) -> Result<()> {
        let (_stream, stream_handle) =
            OutputStream::try_default().context("Gagal menginisialisasi output audio (speaker)")?;
        let sink = Sink::try_new(&stream_handle).context("Gagal membuat audio sink")?;

        // ElevenLabs dipatenkan selalu kecepatan normal 1.0 agar intonasi alami tidak rusak
        sink.set_speed(1.0);

        let cursor = Cursor::new(raw_bytes);
        let decoder = Decoder::new(cursor).context("Gagal decode audio stream")?;
        let sample_rate = decoder.sample_rate();
        let channels = decoder.channels();
        let samples: Vec<f32> = decoder.convert_samples::<f32>().collect();

        if apply_dsp {
            let processed = apply_anime_dsp_filter(&samples, sample_rate);
            sink.append(SamplesBuffer::new(channels, sample_rate, processed));
        } else {
            sink.append(SamplesBuffer::new(channels, sample_rate, samples));
        }

        while !sink.empty() {
            if is_interrupted.load(Ordering::SeqCst) {
                sink.stop();
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }

        Ok(())
    }

    fn play_google_stream(&self, text: &str, is_interrupted: Arc<AtomicBool>) -> Result<()> {
        let (_stream, stream_handle) =
            OutputStream::try_default().context("Gagal menginisialisasi output audio (speaker)")?;
        let sink = Sink::try_new(&stream_handle).context("Gagal membuat audio sink")?;

        let speed = self.playback_speed;
        if (speed - 1.0).abs() > 0.01 {
            sink.set_speed(speed);
        }

        let sentences: Vec<&str> = text
            .split(&['.', '?', '!', '\n', ';'][..])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty() && s.len() > 1)
            .collect();

        let chunks = if sentences.is_empty() {
            vec![text]
        } else {
            sentences
        };

        for chunk in chunks {
            if is_interrupted.load(Ordering::SeqCst) {
                sink.stop();
                return Ok(());
            }

            let safe_chunk = if chunk.len() > 150 {
                // Potong string berdasarkan UTF-8 char boundary yang aman agar tidak panic
                let mut cut_idx = 150;
                while !chunk.is_char_boundary(cut_idx) && cut_idx > 0 {
                    cut_idx -= 1;
                }
                &chunk[..cut_idx]
            } else {
                chunk
            };

            let encoded: String = form_urlencoded::byte_serialize(safe_chunk.as_bytes()).collect();
            let url = format!(
                "https://translate.google.com/translate_tts?ie=UTF-8&q={}&tl={}&client=tw-ob",
                encoded, self.tts_lang
            );

            if let Ok(res) = self
                .http_client
                .get(&url)
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
                .header("Referer", "http://translate.google.com/")
                .send()
            {
                if res.status().is_success() {
                    if let Ok(raw_bytes) = res.bytes() {
                        let cursor = Cursor::new(raw_bytes.to_vec());
                        match Decoder::new(cursor) {
                            Ok(decoder) => {
                                let sample_rate = decoder.sample_rate();
                                let channels = decoder.channels();
                                let samples: Vec<f32> = decoder.convert_samples::<f32>().collect();

                                // Terapkan DSP Anime Enhancer (Formant Treble Booster + Low-cut + Ambience)
                                let processed = apply_anime_dsp_filter(&samples, sample_rate);
                                let source = SamplesBuffer::new(channels, sample_rate, processed);
                                sink.append(source);
                            }
                            Err(e) => {
                                crate::term_err!("⚠️ [Audio Decode Error]: {}", e);
                            }
                        }
                    }
                }
            }

            while sink.len() > 1 && !is_interrupted.load(Ordering::SeqCst) {
                std::thread::sleep(Duration::from_millis(50));
            }
        }

        while !sink.empty() {
            if is_interrupted.load(Ordering::SeqCst) {
                sink.stop();
                break;
            }
            std::thread::sleep(Duration::from_millis(20));
        }

        Ok(())
    }
}

fn clean_text_for_speech(text: &str) -> String {
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
        "malu", "blush", "senang", "gembira", "bingung", "kaget",
        "sedih", "semangat", "tertawa", "tersenyum", "tersipu", "panik", "marah"
    ];

    loop {
        let lower = result.to_lowercase();
        let mut matched = false;
        for prefix in &emotion_prefixes {
            if lower.starts_with(prefix) {
                let rest = &result[prefix.len()..];
                // Cek apakah diikuti spasi, titik dua, koma, atau tanda strip
                if let Some(first_char) = rest.chars().next() {
                    if first_char.is_whitespace() || first_char == ':' || first_char == ',' || first_char == '-' {
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
