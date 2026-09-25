use anyhow::{Context, Result};
use rodio::{Decoder, OutputStream, Sink, Source, buffer::SamplesBuffer};
use std::io::Cursor;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

/// Granular delay-line crossfading pitch shifter dengan Hann Window:
/// Menurunkan frekuensi nada (pitch) tanpa merusak tempo/durasi bicara
fn pitch_shift_granular(samples: &[f32], sample_rate: u32, pitch_ratio: f32) -> Vec<f32> {
    if samples.is_empty() || (pitch_ratio - 1.0).abs() < 0.01 {
        return samples.to_vec();
    }

    // Grain size ~50ms agar gelombang fundamental frekuensi pria (~130Hz) tertampung utuh
    let grain_size = ((sample_rate as f32) * 0.052) as usize;
    if grain_size < 16 {
        return samples.to_vec();
    }

    let mut out = Vec::with_capacity(samples.len());
    let mut phase0 = 0.0f32;
    let rate_diff = 1.0 - pitch_ratio;

    for (i, &s) in samples.iter().enumerate() {
        phase0 = (phase0 + rate_diff).rem_euclid(grain_size as f32);
        let phase1 = (phase0 + (grain_size as f32) * 0.5).rem_euclid(grain_size as f32);

        // Hann Window crossfading halus (mencegah suara robotik / kresek-kresek)
        let norm0 = phase0 / (grain_size as f32);
        let norm1 = phase1 / (grain_size as f32);
        let w0 = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * norm0).cos());
        let w1 = 0.5 * (1.0 - (2.0 * std::f32::consts::PI * norm1).cos());
        let sum_w = (w0 + w1).max(0.0001);

        let read_idx0 = (i as f32 - phase0).max(0.0) as usize;
        let read_idx1 = (i as f32 - phase1).max(0.0) as usize;

        let sample0 = samples.get(read_idx0).copied().unwrap_or(s);
        let sample1 = samples.get(read_idx1).copied().unwrap_or(s);

        let blended = (sample0 * w0 + sample1 * w1) / sum_w;
        out.push(blended);
    }

    out
}

/// DSP Audio Post-Processing untuk Karakter Anime Cowok (Cool & Masculine Male Anime Voice):
/// 1. Pitch shift down ~6 semitone (0.70x ratio) mengubah vokal cewek Google Translate jadi nada bariton cowok asli.
/// 2. High-cut formant filter di ~3200Hz (Membuang sibilance cempreng khas vokal cewek).
/// 3. Low-shelf chest resonance boost di ~180Hz (Resonansi dada pria tebal & gagah).
/// 4. Harmonic Soft Warmth (tanh saturation).
/// 5. Subtle Studio Room Reverb.
/// 6. Dynamic Limiter.
fn apply_male_anime_dsp(samples: &[f32], sample_rate: u32) -> Vec<f32> {
    if samples.is_empty() || sample_rate == 0 {
        return samples.to_vec();
    }

    // Step 1: Pitch shift turun ~6 semitone ke range vokal pria (0.70x)
    let pitched = pitch_shift_granular(samples, sample_rate, 0.70);

    let dt = 1.0 / (sample_rate as f32);
    let mut out = Vec::with_capacity(pitched.len());

    // Step 2: Formant De-Feminizer (Pangkas frekuensi tinggi cempreng di atas 3.2kHz)
    let cutoff_treble = 3200.0f32;
    let rc_treble = 1.0 / (2.0 * std::f32::consts::PI * cutoff_treble);
    let alpha_treble = dt / (rc_treble + dt);
    let mut lp_treble = 0.0f32;

    // Step 3: Chest Resonance Booster di 180Hz (Resonansi dada maskulin)
    let cutoff_low = 180.0f32;
    let rc_low = 1.0 / (2.0 * std::f32::consts::PI * cutoff_low);
    let alpha_low = dt / (rc_low + dt);
    let mut lp_bass = 0.0f32;
    let bass_gain = 1.45f32;

    let delay_samples = ((sample_rate as f32) * 0.038) as usize;
    let mut delay_buf = vec![0.0f32; delay_samples.max(1)];
    let mut delay_idx = 0;

    for &s in &pitched {
        // Redam cempreng wanita
        lp_treble += alpha_treble * (s - lp_treble);
        let tamed = lp_treble * 0.85 + s * 0.15;

        // Tebalkan nada dada pria
        lp_bass += alpha_low * (tamed - lp_bass);
        let chest_enhanced = tamed + (lp_bass * (bass_gain - 1.0));

        // Soft harmonic warmth
        let warmed = (chest_enhanced * 1.10).tanh();

        // Studio ambience
        let delayed = delay_buf[delay_idx];
        let wet = warmed + (delayed * 0.12);
        delay_buf[delay_idx] = wet;
        delay_idx = (delay_idx + 1) % delay_buf.len();

        let mixed = warmed * 0.88 + delayed * 0.12;
        let final_sample = mixed.clamp(-0.95, 0.95);
        out.push(final_sample);
    }

    out
}

/// DSP Audio Post-Processing untuk Karakter Anime Cewek (Sweet & Crisp Anime Voice):
fn apply_female_anime_dsp(samples: &[f32], sample_rate: u32) -> Vec<f32> {
    if samples.is_empty() || sample_rate == 0 {
        return samples.to_vec();
    }

    let dt = 1.0 / (sample_rate as f32);
    let mut out = Vec::with_capacity(samples.len());

    // 1. High-Pass Filter (Low-Cut di 260Hz untuk membuang frekuensi dada berat)
    let cutoff_hp = 260.0f32;
    let rc_hp = 1.0 / (2.0 * std::f32::consts::PI * cutoff_hp);
    let alpha_hp = rc_hp / (rc_hp + dt);

    let mut prev_in = samples[0];
    let mut prev_out_hp = samples[0];

    // Buffer Reverb/Delay sederhana (32ms delay + 0.16 feedback untuk ambience anime)
    let delay_samples = ((sample_rate as f32) * 0.032) as usize;
    let mut delay_buf = vec![0.0f32; delay_samples.max(1)];
    let mut delay_idx = 0;

    // 2. Formant & Air High-Shelf Booster di ~3.8kHz
    let boost_gain = 1.45f32;
    let cutoff_shelf = 3800.0f32;
    let rc_shelf = 1.0 / (2.0 * std::f32::consts::PI * cutoff_shelf);
    let alpha_shelf = dt / (rc_shelf + dt);
    let mut lp_shelf = 0.0f32;

    for &s in samples {
        let hp = alpha_hp * (prev_out_hp + s - prev_in);
        prev_in = s;
        prev_out_hp = hp;

        lp_shelf += alpha_shelf * (hp - lp_shelf);
        let high_part = hp - lp_shelf;
        let enhanced_voice = lp_shelf + (high_part * boost_gain);

        let warmed = (enhanced_voice * 1.08).tanh();

        let delayed = delay_buf[delay_idx];
        let wet = warmed + (delayed * 0.16);
        delay_buf[delay_idx] = wet;
        delay_idx = (delay_idx + 1) % delay_buf.len();

        let mixed = warmed * 0.88 + delayed * 0.12;
        let final_sample = mixed.clamp(-0.95, 0.95);
        out.push(final_sample);
    }

    out
}

fn apply_anime_dsp_filter(samples: &[f32], sample_rate: u32) -> Vec<f32> {
    let u = crate::config::load_user_config();
    let gender = u.assistant_gender.trim().to_lowercase();
    let is_male = gender == "male" || gender == "cowok" || gender == "pria" || gender == "laki-laki";

    if is_male {
        apply_male_anime_dsp(samples, sample_rate)
    } else {
        apply_female_anime_dsp(samples, sample_rate)
    }
}

use crate::tray::{AssistantState, TrayState};

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

    pub fn speak_with_emotion(&self, text: &str, emotion: &str, asst_name: &str) -> Result<()> {
        let clean_text = clean_text_for_speech(text);
        if clean_text.trim().is_empty() {
            return Ok(());
        }

        if crate::app_state::is_voice_muted() {
            return Ok(());
        }

        let mut played_successfully = false;

        // 1. Coba synthesize via ElevenLabs jika API Key diset dan belum dinonaktifkan
        if self.tts_engine == "elevenlabs" {
            if let Some(ref api_key) = self.elevenlabs_api_key {
                if !self.elevenlabs_disabled.load(Ordering::Relaxed) {
                    match self.synthesize_elevenlabs(&clean_text, api_key) {
                        Ok(audio_bytes) => {
                            // Putar suara ElevenLabs asli murni sesuai Voice ID tanpa terkena DSP filter
                            if let Err(e) = self.play_audio_buffer(audio_bytes, false, emotion, asst_name) {
                                crate::term_err!("⚠️ [ElevenLabs Playback Error]: {}", e);
                            } else {
                                played_successfully = true;
                            }
                        }
                        Err(e) => {
                            let err_str = e.to_string();
                            if err_str.contains("quota_exceeded")
                                || err_str.contains("credits remaining")
                                || err_str.contains("Status 401")
                                || err_str.contains("Status 429")
                            {
                                crate::term_err!(
                                    "⚠️ [ElevenLabs Kuota Habis/Kritis] Menonaktifkan ElevenLabs untuk sesi ini, langsung beralih ke Google TTS."
                                );
                                self.elevenlabs_disabled.store(true, Ordering::Relaxed);
                            } else {
                                crate::term_err!(
                                    "⚠️ [ElevenLabs Fallback -> Google TTS]: {}",
                                    err_str
                                );
                            }
                        }
                    }
                }
            }
        }

        // 2. Jika Asisten Cowok (Male), gunakan Edge TTS (id-ID-ArdiNeural) yang 100% gratis suara cowok asli
        let u = crate::config::load_user_config();
        let is_male = u.assistant_gender.trim().eq_ignore_ascii_case("male")
            || u.assistant_gender.trim().eq_ignore_ascii_case("cowok")
            || u.assistant_gender.trim().eq_ignore_ascii_case("pria");

        if !played_successfully && is_male {
            match self.synthesize_edge_tts(&clean_text) {
                Ok(audio_bytes) => {
                    if let Err(e) = self.play_audio_buffer(audio_bytes, false, emotion, asst_name) {
                        crate::term_err!("⚠️ [Edge TTS Playback Error]: {}", e);
                    } else {
                        played_successfully = true;
                    }
                }
                Err(e) => {
                    crate::term_err!("⚠️ [Edge TTS Gagal -> Fallback Google TTS]: {}", e);
                }
            }
        }

        // 3. Suara Cewek (Female) atau fallback: gunakan Google Translate TTS + DSP Anime Filter
        if !played_successfully {
            self.play_google_stream(&clean_text, emotion, asst_name)?;
        }

        Ok(())
    }

    fn synthesize_edge_tts(&self, text: &str) -> Result<Vec<u8>> {
        static CACHED_CONFIG: std::sync::Mutex<Option<msedge_tts::tts::SpeechConfig>> =
            std::sync::Mutex::new(None);

        let config = {
            let mut lock = CACHED_CONFIG.lock().unwrap();
            if let Some(ref cfg) = *lock {
                cfg.clone()
            } else {
                let voices = msedge_tts::voice::get_voices_list()
                    .map_err(|e| anyhow::anyhow!("Gagal mengambil daftar suara Edge TTS: {e:?}"))?;
                let ardi = voices
                    .iter()
                    .find(|v| {
                        v.name.contains("ArdiNeural")
                            || v.short_name
                                .as_deref()
                                .map(|s| s.contains("ArdiNeural"))
                                .unwrap_or(false)
                    })
                    .or_else(|| {
                        voices.iter().find(|v| {
                            v.locale
                                .as_deref()
                                .map(|l| l.starts_with("id"))
                                .unwrap_or(false)
                        })
                    })
                    .ok_or_else(|| anyhow::anyhow!("Suara pria Indonesia (id-ID-ArdiNeural) tidak ditemukan"))?;
                let cfg = msedge_tts::tts::SpeechConfig::from(ardi);
                *lock = Some(cfg.clone());
                cfg
            }
        };

        let mut client = msedge_tts::tts::client::connect()
            .map_err(|e| anyhow::anyhow!("Gagal menghubungkan ke Edge TTS: {e:?}"))?;

        let audio = client
            .synthesize(text, &config)
            .map_err(|e| anyhow::anyhow!("Gagal sintesis Edge TTS: {e:?}"))?;

        Ok(audio.audio_bytes)
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

        let bytes = resp
            .bytes()
            .context("Gagal membaca audio bytes ElevenLabs")?;
        Ok(bytes.to_vec())
    }

    fn play_audio_buffer(
        &self,
        raw_bytes: Vec<u8>,
        apply_dsp: bool,
        emotion: &str,
        asst_name: &str,
    ) -> Result<()> {
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

        // Mulai animasi mulut berbicara HANYA saat audio benar-benar siap dan masuk ke sink speaker
        TrayState::update_with_emotion(
            AssistantState::Speaking,
            &format!("{asst_name} Berbicara"),
            emotion,
        );
        crate::app_state::set_ai_speaking(true);

        if apply_dsp {
            let processed = apply_anime_dsp_filter(&samples, sample_rate);
            sink.append(SamplesBuffer::new(channels, sample_rate, processed));
        } else {
            sink.append(SamplesBuffer::new(channels, sample_rate, samples));
        }

        while !sink.empty() {
            if crate::app_state::is_interruption_requested() {
                sink.stop();
                break;
            }
            std::thread::sleep(Duration::from_millis(15));
        }

        crate::app_state::set_ai_speaking(false);

        // Begitu suara selesai berbunyi, tutup mulut dan kembalikan ke state Thinking jika LLM masih merespons
        TrayState::update_with_emotion(AssistantState::Thinking, "LLM Berpikir...", emotion);
        Ok(())
    }

    fn play_google_stream(&self, text: &str, emotion: &str, asst_name: &str) -> Result<()> {
        let (_stream, stream_handle) =
            OutputStream::try_default().context("Gagal menginisialisasi output audio (speaker)")?;
        let sink = Sink::try_new(&stream_handle).context("Gagal membuat audio sink")?;

        let speed = self.playback_speed;
        if (speed - 1.0).abs() > 0.01 {
            sink.set_speed(speed);
        }

        // Potong kalimat panjang menjadi sub-kalimat
        let chunks: Vec<String> = text
            .split(&[',', ';', '!', '?', '\n', '.'][..])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        let chunks = if chunks.is_empty() {
            vec![text.to_string()]
        } else {
            chunks
        };

        for chunk in chunks {
            let safe_chunk = if chunk.len() > 150 {
                // Potong string berdasarkan UTF-8 char boundary yang aman agar tidak panic
                let mut cut_idx = 150;
                while !chunk.is_char_boundary(cut_idx) && cut_idx > 0 {
                    cut_idx -= 1;
                }
                &chunk[..cut_idx]
            } else {
                &chunk
            };

            let encoded_text: String =
                form_urlencoded::byte_serialize(safe_chunk.as_bytes()).collect();
            let url = format!(
                "https://translate.google.com/translate_tts?ie=UTF-8&q={}&tl={}&client=tw-ob",
                encoded_text, self.tts_lang
            );

            let mut attempts = 0;
            let max_attempts = 2;
            let mut fetch_success = false;

            while attempts < max_attempts && !fetch_success {
                attempts += 1;
                let req_builder = self
                    .http_client
                    .get(&url)
                    .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)");

                if let Ok(resp) = req_builder.send() {
                    if resp.status().is_success() {
                        if let Ok(bytes) = resp.bytes() {
                            let cursor = Cursor::new(bytes.to_vec());
                            if let Ok(decoder) = Decoder::new(cursor) {
                                let sample_rate = decoder.sample_rate();
                                let channels = decoder.channels();
                                let samples: Vec<f32> = decoder.convert_samples::<f32>().collect();

                                // DSP Anime Voice Filter
                                let processed_samples =
                                    apply_anime_dsp_filter(&samples, sample_rate);

                                // Sinkronisasi bibir: Aktifkan Speaking TEPAT saat potongan chunk audio masuk speaker
                                TrayState::update_with_emotion(
                                    AssistantState::Speaking,
                                    &format!("{asst_name} Berbicara"),
                                    emotion,
                                );
                                crate::app_state::set_ai_speaking(true);

                                sink.append(SamplesBuffer::new(
                                    channels,
                                    sample_rate,
                                    processed_samples,
                                ));
                                fetch_success = true;
                            }
                        }
                    }
                }
                if !fetch_success && attempts < max_attempts {
                    std::thread::sleep(Duration::from_millis(50));
                }
            }

            if crate::app_state::is_interruption_requested() {
                sink.stop();
                break;
            }

            while sink.len() > 1 {
                if crate::app_state::is_interruption_requested() {
                    sink.stop();
                    break;
                }
                std::thread::sleep(Duration::from_millis(15));
            }
        }

        while !sink.empty() {
            if crate::app_state::is_interruption_requested() {
                sink.stop();
                break;
            }
            std::thread::sleep(Duration::from_millis(15));
        }

        crate::app_state::set_ai_speaking(false);

        // Selesai bicara seluruh potongan kalimat, segera hentikan flap mulut
        TrayState::update_with_emotion(AssistantState::Thinking, "LLM Berpikir...", emotion);
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
