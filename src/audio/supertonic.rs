use anyhow::{Result, anyhow};
use ort::session::Session;
use ort::session::builder::GraphOptimizationLevel;
use ort::value::Tensor;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

#[derive(Deserialize)]
struct StyleVector {
    dims: Vec<usize>,
    data: Vec<Vec<Vec<f32>>>,
}

#[derive(Deserialize)]
struct VoiceStyleJson {
    style_ttl: StyleVector,
    style_dp: StyleVector,
}

#[derive(Deserialize, Clone)]
struct TtsConfigs {
    ae: AeConfig,
    ttl: TtlConfig,
}

#[derive(Deserialize, Clone)]
struct AeConfig {
    sample_rate: u32,
    base_chunk_size: usize,
}

#[derive(Deserialize, Clone)]
struct TtlConfig {
    latent_dim: usize,
    chunk_compress_factor: usize,
}

#[derive(Clone)]
pub struct VoiceStyleData {
    pub ttl_dims: [usize; 3],
    pub ttl_data: Vec<f32>,
    pub dp_dims: [usize; 3],
    pub dp_data: Vec<f32>,
}

pub struct SupertonicEngine {
    text_enc_sess: Mutex<Session>,
    dp_sess: Mutex<Session>,
    vec_est_sess: Mutex<Session>,
    vocoder_sess: Mutex<Session>,
    cfg: TtsConfigs,
    indexer: Vec<i64>,
    styles: HashMap<String, VoiceStyleData>,
}

static ENGINE: OnceLock<SupertonicEngine> = OnceLock::new();
static LAST_AUDIO_WAV: Mutex<Option<(String, Vec<u8>)>> = Mutex::new(None);

pub fn get_engine() -> Option<&'static SupertonicEngine> {
    ENGINE.get()
}

pub fn init_engine_background() {
    std::thread::spawn(|| {
        if let Err(e) = init_engine() {
            crate::term_err!("❌ [Supertonic TTS] Failed to initialize model: {e}");
        }
    });
}

pub fn init_engine() -> Result<&'static SupertonicEngine> {
    if let Some(engine) = ENGINE.get() {
        return Ok(engine);
    }

    let model_dir =
        find_model_dir().ok_or_else(|| anyhow!("Supertonic-3 model directory not found"))?;
    crate::term_log!(
        "🚀 [Supertonic TTS] Memuat model neural ONNX dari {:?}...",
        model_dir
    );
    let start = std::time::Instant::now();

    // 1. Config
    let tts_json_path = model_dir.join("onnx/tts.json");
    let mut tts_json_str = String::new();
    File::open(&tts_json_path)?.read_to_string(&mut tts_json_str)?;
    let cfg: TtsConfigs = serde_json::from_str(&tts_json_str)?;

    // 2. Unicode Indexer
    let idx_path = model_dir.join("onnx/unicode_indexer.json");
    let mut idx_str = String::new();
    File::open(&idx_path)?.read_to_string(&mut idx_str)?;
    let indexer: Vec<i64> = serde_json::from_str(&idx_str)?;

    // 3. ONNX Sessions (use Level1 optimization & bounded threads to minimize memory footprint)
    let text_enc_sess = Session::builder()
        .map_err(|e| anyhow!("{e}"))?
        .with_intra_threads(2)
        .map_err(|e| anyhow!("{e}"))?
        .with_optimization_level(GraphOptimizationLevel::Level1)
        .map_err(|e| anyhow!("{e}"))?
        .commit_from_file(model_dir.join("onnx/text_encoder.onnx"))
        .map_err(|e| anyhow!("{e}"))?;

    let dp_sess = Session::builder()
        .map_err(|e| anyhow!("{e}"))?
        .with_intra_threads(2)
        .map_err(|e| anyhow!("{e}"))?
        .with_optimization_level(GraphOptimizationLevel::Level1)
        .map_err(|e| anyhow!("{e}"))?
        .commit_from_file(model_dir.join("onnx/duration_predictor.onnx"))
        .map_err(|e| anyhow!("{e}"))?;

    let vec_est_sess = Session::builder()
        .map_err(|e| anyhow!("{e}"))?
        .with_intra_threads(2)
        .map_err(|e| anyhow!("{e}"))?
        .with_optimization_level(GraphOptimizationLevel::Level1)
        .map_err(|e| anyhow!("{e}"))?
        .commit_from_file(model_dir.join("onnx/vector_estimator.onnx"))
        .map_err(|e| anyhow!("{e}"))?;

    let vocoder_sess = Session::builder()
        .map_err(|e| anyhow!("{e}"))?
        .with_intra_threads(2)
        .map_err(|e| anyhow!("{e}"))?
        .with_optimization_level(GraphOptimizationLevel::Level1)
        .map_err(|e| anyhow!("{e}"))?
        .commit_from_file(model_dir.join("onnx/vocoder.onnx"))
        .map_err(|e| anyhow!("{e}"))?;

    // 4. Preload 10 Voice Styles (M1-M5, F1-F5)
    let mut styles = HashMap::new();
    let voice_keys = ["M1", "M2", "M3", "M4", "M5", "F1", "F2", "F3", "F4", "F5"];
    let styles_dir = model_dir.join("voice_styles");

    for key in voice_keys {
        let path = styles_dir.join(format!("{key}.json"));
        if path.exists() {
            if let Ok(mut file) = File::open(&path) {
                let mut content = String::new();
                if file.read_to_string(&mut content).is_ok() {
                    if let Ok(json) = serde_json::from_str::<VoiceStyleJson>(&content) {
                        let ttl_dims = [
                            json.style_ttl.dims[0],
                            json.style_ttl.dims[1],
                            json.style_ttl.dims[2],
                        ];
                        let ttl_data: Vec<f32> = json
                            .style_ttl
                            .data
                            .into_iter()
                            .flatten()
                            .flatten()
                            .collect();
                        let dp_dims = [
                            json.style_dp.dims[0],
                            json.style_dp.dims[1],
                            json.style_dp.dims[2],
                        ];
                        let dp_data: Vec<f32> =
                            json.style_dp.data.into_iter().flatten().flatten().collect();

                        let style_data = VoiceStyleData {
                            ttl_dims,
                            ttl_data,
                            dp_dims,
                            dp_data,
                        };
                        styles.insert(key.to_string(), style_data.clone());
                        styles.insert(key.to_lowercase(), style_data);
                    }
                }
            }
        }
    }

    crate::term_log!(
        "⚡ [Supertonic TTS] Initialized in {:.2?}! ({} voice styles ready)",
        start.elapsed(),
        styles.len() / 2
    );

    let engine = SupertonicEngine {
        text_enc_sess: Mutex::new(text_enc_sess),
        dp_sess: Mutex::new(dp_sess),
        vec_est_sess: Mutex::new(vec_est_sess),
        vocoder_sess: Mutex::new(vocoder_sess),
        cfg,
        indexer,
        styles,
    };

    let _ = ENGINE.set(engine);
    Ok(ENGINE.get().unwrap())
}

fn find_model_dir() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("models/supertonic-3"),
        PathBuf::from("../models/supertonic-3"),
        PathBuf::from("/home/headway/MyWork/project/node/ai-voice/models/supertonic-3"),
    ];
    candidates
        .into_iter()
        .find(|c| c.join("onnx/vocoder.onnx").exists())
}

impl SupertonicEngine {
    pub fn get_voice_key(&self, voice_input: &str) -> String {
        let v = voice_input.trim();
        let upper = v.to_uppercase();

        // 1. Direct key match: M1-M5, F1-F5
        if self.styles.contains_key(&upper) {
            return upper;
        }

        // 2. Name mapping from option.html
        let v_lower = v.to_lowercase();
        if v_lower.contains("alex") {
            return "M1".to_string();
        }
        if v_lower.contains("james") {
            return "M2".to_string();
        }
        if v_lower.contains("robert") {
            return "M3".to_string();
        }
        if v_lower.contains("sam") {
            return "M4".to_string();
        }
        if v_lower.contains("daniel") {
            return "M5".to_string();
        }

        if v_lower.contains("sarah") {
            return "F1".to_string();
        }
        if v_lower.contains("lily") {
            return "F2".to_string();
        }
        if v_lower.contains("jessica") {
            return "F3".to_string();
        }
        if v_lower.contains("olivia") {
            return "F4".to_string();
        }
        if v_lower.contains("emily") {
            return "F5".to_string();
        }

        // Default female voice F4
        "F4".to_string()
    }

    fn synthesize_chunk(
        &self,
        chunk_text: &str,
        style: &VoiceStyleData,
        lang_code: &str,
        speed: f32,
    ) -> Result<Vec<f32>> {
        let preprocessed = preprocess_text(chunk_text, lang_code);

        let text_ids_vec: Vec<i64> = preprocessed
            .chars()
            .map(|c| {
                let u = c as usize;
                if u < self.indexer.len() {
                    self.indexer[u]
                } else {
                    -1
                }
            })
            .collect();
        let text_len = text_ids_vec.len();
        if text_len == 0 {
            return Err(anyhow!("Teks kosong setelah preprocessing"));
        }

        let text_mask_vec = vec![1.0f32; text_len];

        // 1. Duration Predictor
        let raw_duration = {
            let mut dp = self.dp_sess.lock().map_err(|e| anyhow!("{e}"))?;
            let inputs = ort::inputs![
                "text_ids" => Tensor::from_array(([1, text_len], text_ids_vec.clone()))?,
                "style_dp" => Tensor::from_array((style.dp_dims, style.dp_data.clone()))?,
                "text_mask" => Tensor::from_array(([1, 1, text_len], text_mask_vec.clone()))?
            ];
            let outputs = dp.run(inputs)?;
            let (_, dur_slice) = outputs["duration"].try_extract_tensor::<f32>()?;
            dur_slice[0]
        };

        let duration = (raw_duration / speed).max(0.2);

        // 2. Text Encoder
        let (text_emb_dims, text_emb_data) = {
            let mut text_enc = self.text_enc_sess.lock().map_err(|e| anyhow!("{e}"))?;
            let inputs = ort::inputs![
                "text_ids" => Tensor::from_array(([1, text_len], text_ids_vec))?,
                "style_ttl" => Tensor::from_array((style.ttl_dims, style.ttl_data.clone()))?,
                "text_mask" => Tensor::from_array(([1, 1, text_len], text_mask_vec.clone()))?
            ];
            let outputs = text_enc.run(inputs)?;
            let (shape, slice) = outputs["text_emb"].try_extract_tensor::<f32>()?;
            let dims = [shape[0] as usize, shape[1] as usize, shape[2] as usize];
            (dims, slice.to_vec())
        };

        // 3. Noisy Latent & Flow Matching
        // PENGAMAN KRUSIAL: Model vector_estimator ONNX memiliki batas dimensi tensor attention = 1000.
        // Clamping ke .min(960) menjamin secara matematis tidak akan pernah melebihi 1000 ('1000 by 1065' error).
        let chunk_size = self.cfg.ae.base_chunk_size * self.cfg.ttl.chunk_compress_factor;
        let wav_len_max = (duration * self.cfg.ae.sample_rate as f32).round() as i64;
        let latent_len =
            (((wav_len_max + chunk_size as i64 - 1) / chunk_size as i64).max(1) as usize).min(960);
        let latent_dim = self.cfg.ttl.latent_dim * self.cfg.ttl.chunk_compress_factor;

        let seed = simple_hash(chunk_text);
        let mut rng = Rng::new(seed);
        let mut xt_data = vec![0.0f32; latent_dim * latent_len];
        for elem in xt_data.iter_mut() {
            *elem = rng.next_gaussian();
        }
        let latent_mask_data = vec![1.0f32; latent_len];

        let total_steps = 6;
        let total_step_data = vec![total_steps as f32];

        {
            let mut vec_est = self.vec_est_sess.lock().map_err(|e| anyhow!("{e}"))?;
            for step in 0..total_steps {
                let current_step_data = vec![step as f32];
                let inputs = ort::inputs![
                    "noisy_latent" => Tensor::from_array(([1, latent_dim, latent_len], xt_data.clone()))?,
                    "text_emb" => Tensor::from_array((text_emb_dims, text_emb_data.clone()))?,
                    "style_ttl" => Tensor::from_array((style.ttl_dims, style.ttl_data.clone()))?,
                    "latent_mask" => Tensor::from_array(([1, 1, latent_len], latent_mask_data.clone()))?,
                    "text_mask" => Tensor::from_array(([1, 1, text_len], text_mask_vec.clone()))?,
                    "current_step" => Tensor::from_array(([1], current_step_data))?,
                    "total_step" => Tensor::from_array(([1], total_step_data.clone()))?
                ];
                let outputs = vec_est.run(inputs)?;
                let (_, out_slice) = outputs["denoised_latent"].try_extract_tensor::<f32>()?;
                xt_data = out_slice.to_vec();
            }
        }

        // 4. Vocoder
        let wav_samples: Vec<f32> = {
            let mut vocoder = self.vocoder_sess.lock().map_err(|e| anyhow!("{e}"))?;
            let inputs = ort::inputs![
                "latent" => Tensor::from_array(([1, latent_dim, latent_len], xt_data))?
            ];
            let outputs = vocoder.run(inputs)?;
            let (_, wav_slice) = outputs["wav_tts"].try_extract_tensor::<f32>()?;
            wav_slice.to_vec()
        };

        Ok(wav_samples)
    }

    pub fn synthesize(&self, text: &str, voice: &str, lang: &str, speed: f32) -> Result<Vec<u8>> {
        let speed = if speed <= 0.2 {
            1.0
        } else {
            speed.clamp(0.5, 2.0)
        };
        let voice_key = self.get_voice_key(voice);
        let style = self
            .styles
            .get(&voice_key)
            .ok_or_else(|| anyhow!("Voice style '{voice_key}' not found"))?;

        let lang_code = clean_lang_code(lang);
        let chunks = split_into_tts_chunks(text);
        if chunks.is_empty() {
            return Err(anyhow!("Teks kosong setelah pemecahan chunk"));
        }

        if chunks.len() == 1 {
            let samples = self.synthesize_chunk(&chunks[0], style, &lang_code, speed)?;
            return Ok(samples_to_wav_bytes(&samples, self.cfg.ae.sample_rate));
        }

        let mut all_samples = Vec::new();
        // Jeda napas alami antar kalimat ~100ms (4410 sampel pada 44.1kHz)
        let pause_samples = (0.10 * self.cfg.ae.sample_rate as f32) as usize;

        for (i, chunk) in chunks.iter().enumerate() {
            match self.synthesize_chunk(chunk, style, &lang_code, speed) {
                Ok(samples) => {
                    if !samples.is_empty() {
                        if i > 0 && !all_samples.is_empty() {
                            all_samples.extend(std::iter::repeat_n(0.0f32, pause_samples));
                        }
                        all_samples.extend(samples);
                    }
                }
                Err(e) => {
                    crate::term_err!(
                        "⚠️ [Supertonic TTS] Failed to synthesize sub-chunk \"{}\": {e}",
                        chunk
                    );
                }
            }
        }

        if all_samples.is_empty() {
            return Err(anyhow!("Semua sub-chunk gagal disintesis"));
        }

        Ok(samples_to_wav_bytes(&all_samples, self.cfg.ae.sample_rate))
    }
}

/// Memecah teks panjang menjadi chunk kalimat/klausa alami agar tidak melampaui batas tensor ONNX (~140 karakter)
pub fn split_into_tts_chunks(text: &str) -> Vec<String> {
    let text = text.trim();
    if text.is_empty() {
        return Vec::new();
    }

    const MAX_CHUNK_CHARS: usize = 120;

    let mut chunks = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);

        let is_sentence_end = ch == '.' || ch == '!' || ch == '?' || ch == '\n';
        let is_clause_end = ch == ',' || ch == ';' || ch == ':';

        if is_sentence_end {
            if current.trim().len() >= 15 || current.len() >= MAX_CHUNK_CHARS {
                chunks.push(current.trim().to_string());
                current.clear();
            }
        } else if (is_clause_end && current.len() >= 45)
            || (current.len() >= MAX_CHUNK_CHARS && ch.is_whitespace())
        {
            chunks.push(current.trim().to_string());
            current.clear();
        }
    }

    if !current.trim().is_empty() {
        chunks.push(current.trim().to_string());
    }

    let mut result = Vec::new();
    for chunk in chunks {
        if chunk.chars().count() <= MAX_CHUNK_CHARS {
            if !chunk.trim().is_empty() {
                result.push(chunk);
            }
        } else {
            let mut sub = String::new();
            for word in chunk.split_whitespace() {
                if !sub.is_empty() && (sub.len() + word.len() + 1) > MAX_CHUNK_CHARS {
                    result.push(sub.trim().to_string());
                    sub.clear();
                }
                if !sub.is_empty() {
                    sub.push(' ');
                }
                sub.push_str(word);
            }
            if !sub.trim().is_empty() {
                result.push(sub.trim().to_string());
            }
        }
    }

    if result.is_empty() && !text.is_empty() {
        result.push(text.to_string());
    }

    result
}

pub fn set_last_audio_wav(id: String, bytes: Vec<u8>) {
    if let Ok(mut lock) = LAST_AUDIO_WAV.lock() {
        *lock = Some((id, bytes));
    }
}

pub fn get_last_audio_wav(req_id: Option<&str>) -> Option<Vec<u8>> {
    if let Ok(lock) = LAST_AUDIO_WAV.lock() {
        if let Some((id, bytes)) = &*lock {
            if let Some(r_id) = req_id {
                if !r_id.is_empty() && id != r_id {
                    return Some(bytes.clone()); // Fallback to last spoken
                }
            }
            return Some(bytes.clone());
        }
    }
    None
}

// Normalise any BCP-47 / ISO-639 tag to a bare two-letter base code.
// Must stay in sync with SUPPORTED_LANGUAGES in AudioTtsSection.tsx and
// resolve_language_info() in llm/prompt.rs.
fn clean_lang_code(lang: &str) -> String {
    let l = lang.trim().to_lowercase();
    let base = l.split(['-', '_']).next().unwrap_or(&l);
    match base {
        "id" => "id",
        "en" => "en",
        "ja" => "ja",
        "ko" => "ko",
        "ar" => "ar",
        "bg" => "bg",
        "cs" => "cs",
        "da" => "da",
        "de" => "de",
        "el" => "el",
        "es" => "es",
        "et" => "et",
        "fi" => "fi",
        "fr" => "fr",
        "hi" => "hi",
        "hr" => "hr",
        "hu" => "hu",
        "it" => "it",
        "lt" => "lt",
        "lv" => "lv",
        "nl" => "nl",
        "pl" => "pl",
        "pt" => "pt",
        "ro" => "ro",
        "ru" => "ru",
        "sk" => "sk",
        "sl" => "sl",
        "sv" => "sv",
        "tr" => "tr",
        "uk" => "uk",
        "vi" => "vi",
        "zh" => "zh",
        _ => "id",
    }
    .to_string()
}

fn preprocess_text(text: &str, lang: &str) -> String {
    let mut t = text.to_string();
    t = t
        .replace(['–', '‑', '—'], "-")
        .replace(['¯', '_'], " ")
        .replace(['“', '”'], "\"")
        .replace(['‘', '’', '´', '`'], "'")
        .replace(['[', ']', '|', '/', '#', '→', '←'], " ");

    t = t.replace('@', " at ");
    t = t.replace("e.g.,", "for example, ");
    t = t.replace("i.e.,", "that is, ");

    let re_sp = regex::Regex::new(r"\s+").unwrap();
    t = re_sp.replace_all(&t, " ").trim().to_string();

    let ends_with_punct = t.ends_with('.')
        || t.ends_with('!')
        || t.ends_with('?')
        || t.ends_with(';')
        || t.ends_with(':');
    if !ends_with_punct && !t.is_empty() {
        t.push('.');
    }

    format!("<{lang}>{t}</{lang}>")
}

fn simple_hash(s: &str) -> u64 {
    let mut h = 0xcbf29ce484222325u64;
    for b in s.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 { 0x853c49e6748fea9b } else { seed },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    fn next_f32(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / 16777216.0
    }

    fn next_gaussian(&mut self) -> f32 {
        let u1 = self.next_f32().max(1e-7);
        let u2 = self.next_f32();
        (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos()
    }
}

pub fn samples_to_wav_bytes(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let num_channels: u16 = 1;
    let bits_per_sample: u16 = 16;
    let byte_rate = sample_rate * num_channels as u32 * (bits_per_sample as u32 / 8);
    let block_align = num_channels * (bits_per_sample / 8);
    let data_len = (samples.len() * 2) as u32;

    let mut out = Vec::with_capacity(44 + data_len as usize);
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(data_len + 36).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&num_channels.to_le_bytes());
    out.extend_from_slice(&sample_rate.to_le_bytes());
    out.extend_from_slice(&byte_rate.to_le_bytes());
    out.extend_from_slice(&block_align.to_le_bytes());
    out.extend_from_slice(&bits_per_sample.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data_len.to_le_bytes());

    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let s_i16 = (clamped * 32767.0) as i16;
        out.extend_from_slice(&s_i16.to_le_bytes());
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supertonic_synthesis() {
        let engine = init_engine().expect("Supertonic engine should initialize");
        let wav = engine
            .synthesize("Halo Sukma!", "F4", "id", 1.05)
            .expect("Synthesis should succeed");
        assert!(wav.len() > 44);
        assert_eq!(&wav[0..4], b"RIFF");
        assert_eq!(&wav[8..12], b"WAVE");
    }

    #[test]
    fn test_split_into_tts_chunks() {
        let text = "Halo Mas! Hari ini cuacanya panas banget ya. Jangan lupa banyak minum air putih biar tetap segar dan sehat sepanjang hari.";
        let chunks = split_into_tts_chunks(text);
        assert!(chunks.len() >= 2);
        for chunk in &chunks {
            assert!(chunk.chars().count() <= 120);
        }
    }
}
