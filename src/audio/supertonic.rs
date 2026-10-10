use anyhow::{Result, anyhow};
use ort::ep::CPU;
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
type AudioByteQueue = Mutex<std::collections::VecDeque<(String, Vec<u8>)>>;

static LAST_AUDIO_WAV_LIST: std::sync::LazyLock<AudioByteQueue> =
    std::sync::LazyLock::new(|| Mutex::new(std::collections::VecDeque::new()));
static LATEST_AUDIO_ID: Mutex<Option<String>> = Mutex::new(None);

pub fn get_engine() -> Option<&'static SupertonicEngine> {
    ENGINE.get()
}

pub fn init_engine_background() {
    std::thread::spawn(|| {
        if let Err(e) = init_engine() {
            crate::term_err!("[Supertonic TTS] Failed to initialize model: {e}");
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
        "[Supertonic TTS] Loading ONNX neural model from {:?}...",
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

    let available_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);
    let intra_threads = available_threads.min(4);

    // 3. ONNX Sessions (multi-threaded with Level3 graph optimization and disabled memory arena)
    let text_enc_sess = Session::builder()
        .map_err(|e| anyhow!("{e}"))?
        .with_execution_providers([CPU::default().with_arena_allocator(false).build()])
        .map_err(|e| anyhow!("{e}"))?
        .with_intra_threads(intra_threads)
        .map_err(|e| anyhow!("{e}"))?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| anyhow!("{e}"))?
        .with_memory_pattern(false)
        .map_err(|e| anyhow!("{e}"))?
        .commit_from_file(model_dir.join("onnx/text_encoder.onnx"))
        .map_err(|e| anyhow!("{e}"))?;

    let dp_sess = Session::builder()
        .map_err(|e| anyhow!("{e}"))?
        .with_execution_providers([CPU::default().with_arena_allocator(false).build()])
        .map_err(|e| anyhow!("{e}"))?
        .with_intra_threads(intra_threads)
        .map_err(|e| anyhow!("{e}"))?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| anyhow!("{e}"))?
        .with_memory_pattern(false)
        .map_err(|e| anyhow!("{e}"))?
        .commit_from_file(model_dir.join("onnx/duration_predictor.onnx"))
        .map_err(|e| anyhow!("{e}"))?;

    let vec_est_sess = Session::builder()
        .map_err(|e| anyhow!("{e}"))?
        .with_execution_providers([CPU::default().with_arena_allocator(false).build()])
        .map_err(|e| anyhow!("{e}"))?
        .with_intra_threads(intra_threads)
        .map_err(|e| anyhow!("{e}"))?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| anyhow!("{e}"))?
        .with_memory_pattern(false)
        .map_err(|e| anyhow!("{e}"))?
        .commit_from_file(model_dir.join("onnx/vector_estimator.onnx"))
        .map_err(|e| anyhow!("{e}"))?;

    let vocoder_sess = Session::builder()
        .map_err(|e| anyhow!("{e}"))?
        .with_execution_providers([CPU::default().with_arena_allocator(false).build()])
        .map_err(|e| anyhow!("{e}"))?
        .with_intra_threads(intra_threads)
        .map_err(|e| anyhow!("{e}"))?
        .with_optimization_level(GraphOptimizationLevel::Level3)
        .map_err(|e| anyhow!("{e}"))?
        .with_memory_pattern(false)
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
        "[Supertonic TTS] Initialized in {:.2?}! ({} voice styles ready)",
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
    ENGINE
        .get()
        .ok_or_else(|| anyhow::anyhow!("Supertonic engine not initialized"))
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
            return Err(anyhow!("Empty text after preprocessing"));
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
        // CRUCIAL GUARD: ONNX vector_estimator model has attention tensor dimension limit = 1000.
        // Clamping to .min(960) mathematically guarantees never exceeding 1000 ('1000 by 1065' error).
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
        self.synthesize_stream(text, voice, lang, speed, |_, _, _| {})
    }

    /// Incrementally synthesizes text per chunk and directly sends WAV bytes to channel sender (streaming TTS queue).
    /// Return total combined WAV audio bytes for caching and mobile client.
    pub fn synthesize_stream<F>(
        &self,
        text: &str,
        voice: &str,
        lang: &str,
        speed: f32,
        mut on_chunk_ready: F,
    ) -> Result<Vec<u8>>
    where
        F: FnMut(usize, usize, Vec<u8>), // (chunk_idx, total_chunks, wav_bytes)
    {
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
            return Err(anyhow!("Empty text after chunk splitting"));
        }

        let total_chunks = chunks.len();
        let ref_voice_opt = crate::audio::openvoice::find_ref_voice_path();
        let openvoice_opt = if ref_voice_opt.is_some() {
            crate::audio::openvoice::get_engine()
        } else {
            None
        };

        if total_chunks == 1 {
            let samples = self.synthesize_chunk(&chunks[0], style, &lang_code, speed)?;
            let (final_samples, out_sample_rate) = match (openvoice_opt, ref_voice_opt.as_ref()) {
                (Some(ov), Some(ref_path)) => {
                    match ov.convert_voice(&samples, self.cfg.ae.sample_rate, ref_path, 0.3) {
                        Ok(converted) => (converted, 22050),
                        Err(e) => {
                            crate::term_err!("[OpenVoice] Tone conversion failed: {e}");
                            (samples, self.cfg.ae.sample_rate)
                        }
                    }
                }
                _ => (samples, self.cfg.ae.sample_rate),
            };
            let wav_bytes = samples_to_wav_bytes(&final_samples, out_sample_rate);
            on_chunk_ready(0, 1, wav_bytes.clone());
            return Ok(wav_bytes);
        }

        let mut all_samples = Vec::new();
        let out_sample_rate = if ref_voice_opt.is_some() && openvoice_opt.is_some() {
            22050
        } else {
            self.cfg.ae.sample_rate
        };

        // Pre-fetch target speaker embedding once for all subsequent chunks
        let target_se_opt = match (openvoice_opt, ref_voice_opt.as_ref()) {
            (Some(ov), Some(ref_path)) => ov.get_target_se(ref_path).ok(),
            _ => None,
        };

        for (i, chunk) in chunks.iter().enumerate() {
            let (chunk_speed, chunk_pause) = compute_chunk_prosody(chunk, speed, out_sample_rate, i, total_chunks);

            match self.synthesize_chunk(chunk, style, &lang_code, chunk_speed) {
                Ok(raw_samples) => {
                    if !raw_samples.is_empty() {
                        let (chunk_samples, chunk_rate) = match (openvoice_opt, target_se_opt.as_ref()) {
                            (Some(ov), Some(target_se)) => {
                                match ov.convert_voice_with_target_se(&raw_samples, self.cfg.ae.sample_rate, target_se, 0.3) {
                                    Ok(converted) => (converted, 22050),
                                    Err(e) => {
                                        crate::term_err!("[OpenVoice] Tone conversion failed: {e}");
                                        (raw_samples, self.cfg.ae.sample_rate)
                                    }
                                }
                            }
                            _ => (raw_samples, self.cfg.ae.sample_rate),
                        };

                        let chunk_wav = samples_to_wav_bytes(&chunk_samples, chunk_rate);
                        on_chunk_ready(i, total_chunks, chunk_wav);

                        if i > 0 && !all_samples.is_empty() {
                            all_samples.extend(std::iter::repeat_n(0.0f32, chunk_pause));
                        }
                        all_samples.extend(chunk_samples);
                    }
                }
                Err(e) => {
                    crate::term_err!(
                        "[Supertonic TTS] Failed to synthesize sub-chunk \"{}\" (len {}): {e}",
                        chunk, chunk.chars().count()
                    );
                }
            }
        }

        if all_samples.is_empty() {
            return Err(anyhow!("All audio sub-chunks failed to synthesize"));
        }

        Ok(samples_to_wav_bytes(&all_samples, out_sample_rate))
    }
}

/// Blends two voice styles using Magnitude-Preserving Spherical Interpolation (Slerp).
/// Unlike simple linear interpolation (Lerp) which diminishes latent variance and muffles high frequencies,
/// geometric Slerp preserves the vector manifold curvature and full acoustic energy.
#[allow(dead_code)]
pub fn blend_voice_styles(style_a: &VoiceStyleData, style_b: &VoiceStyleData, ratio_b: f32) -> VoiceStyleData {
    let t = ratio_b.clamp(0.0, 1.0);

    let slerp_vec = |a: &[f32], b: &[f32], t: f32| -> Vec<f32> {
        if a.len() != b.len() || a.is_empty() {
            return a.to_vec();
        }

        let mut dot = 0.0f32;
        let mut norm_a_sq = 0.0f32;
        let mut norm_b_sq = 0.0f32;

        for (&x, &y) in a.iter().zip(b.iter()) {
            dot += x * y;
            norm_a_sq += x * x;
            norm_b_sq += y * y;
        }

        let norm_a = norm_a_sq.sqrt();
        let norm_b = norm_b_sq.sqrt();

        if norm_a < 1e-7 || norm_b < 1e-7 {
            return a.iter().zip(b.iter()).map(|(&x, &y)| (1.0 - t) * x + t * y).collect();
        }

        // Cosine of angle between vectors
        let cos_theta = (dot / (norm_a * norm_b)).clamp(-1.0, 1.0);
        let target_norm = (1.0 - t) * norm_a + t * norm_b;

        // If vectors are nearly collinear, fall back to linear interpolation with norm preservation
        if cos_theta.abs() > 0.9995 {
            let mut out: Vec<f32> = a.iter().zip(b.iter()).map(|(&x, &y)| (1.0 - t) * x + t * y).collect();
            let mut out_norm_sq = 0.0f32;
            for &val in &out {
                out_norm_sq += val * val;
            }
            let out_norm = out_norm_sq.sqrt();
            if out_norm > 1e-7 {
                let scale = target_norm / out_norm;
                for val in &mut out {
                    *val *= scale;
                }
            }
            return out;
        }

        let theta = cos_theta.acos();
        let sin_theta = theta.sin();
        let scale_a = ((1.0 - t) * theta).sin() / sin_theta;
        let scale_b = (t * theta).sin() / sin_theta;

        let mut out = Vec::with_capacity(a.len());
        for (&x, &y) in a.iter().zip(b.iter()) {
            out.push(scale_a * x + scale_b * y);
        }

        // Ensure resulting magnitude matches expected blended target norm
        let mut res_norm_sq = 0.0f32;
        for &val in &out {
            res_norm_sq += val * val;
        }
        let res_norm = res_norm_sq.sqrt();
        if res_norm > 1e-7 {
            let scale = target_norm / res_norm;
            for val in &mut out {
                *val *= scale;
            }
        }
        out
    };

    let ttl_data = slerp_vec(&style_a.ttl_data, &style_b.ttl_data, t);
    let dp_data = slerp_vec(&style_a.dp_data, &style_b.dp_data, t);

    VoiceStyleData {
        ttl_dims: style_a.ttl_dims,
        ttl_data,
        dp_dims: style_a.dp_dims,
        dp_data,
    }
}

/// Computes natural, expressive prosody variations (tempo & inter-chunk pause)
/// analyzing sentence punctuation, ellipsis, word density, and utterance position.
fn compute_chunk_prosody(
    chunk: &str,
    base_speed: f32,
    out_sample_rate: u32,
    chunk_idx: usize,
    total_chunks: usize,
) -> (f32, usize) {
    let trimmed = chunk.trim();
    if trimmed.is_empty() {
        return (base_speed, (0.08 * out_sample_rate as f32) as usize);
    }

    let char_count = trimmed.chars().count();
    let is_first = chunk_idx == 0;
    let is_last = chunk_idx + 1 >= total_chunks;

    // 1. Position-aware Base Tempo:
    // Opening phrase speaks with crisp clarity, mid-phrase flows naturally, ending phrase gently slows down.
    let mut speed_mult = if is_first && total_chunks > 1 {
        1.01
    } else if is_last && total_chunks > 1 {
        0.97
    } else {
        1.00
    };

    // 2. Length & Word Density modulation:
    // Short exclamation/greeting gets expressive pace; dense long sentence gets slight speedup for fluid delivery.
    if char_count < 15 {
        speed_mult *= 0.96; // more deliberate, expressive cadence
    } else if char_count > 60 {
        speed_mult *= 1.03; // slight momentum for long sentences
    }

    // 3. Punctuation & Emotion Parsing:
    let (punct_mult, pause_secs) = if trimmed.ends_with("...") || trimmed.ends_with('…') {
        // Hesitation / trailing thought / suspense
        (0.92, 0.22)
    } else if trimmed.ends_with("?!") || trimmed.ends_with("!?") {
        // Shock / excited question
        (1.08, 0.16)
    } else if trimmed.ends_with('?') || trimmed.ends_with('？') {
        // Question intonation
        (1.04, 0.13)
    } else if trimmed.ends_with('!') || trimmed.ends_with('！') {
        // Exclamation / excitement
        (1.06, 0.14)
    } else if trimmed.ends_with(',') || trimmed.ends_with('，') || trimmed.ends_with(';') || trimmed.ends_with('；') {
        // Intra-sentence breathing pause
        (1.00, 0.07)
    } else if trimmed.ends_with(':') || trimmed.ends_with('：') {
        // Colon explanation pause
        (0.98, 0.12)
    } else {
        // Standard period / sentence closure
        let final_pause = if is_last { 0.05 } else { 0.11 };
        (1.00, final_pause)
    };

    let final_speed = (base_speed * speed_mult * punct_mult).clamp(0.5, 2.0);
    let pause_samples = (pause_secs * out_sample_rate as f32) as usize;

    (final_speed, pause_samples)
}

pub fn split_into_tts_chunks(text: &str) -> Vec<String> {
    let text = text.trim();
    if text.is_empty() {
        return Vec::new();
    }

    let mut sentences = Vec::new();
    let mut current = String::new();

    for ch in text.chars() {
        current.push(ch);

        // Natural sentence endings
        let is_terminal = ch == '.' || ch == '!' || ch == '?' || ch == '\n';
        // Natural pause clauses (semicolon, colon)
        let is_clause = ch == ';' || ch == ':';

        if is_terminal || is_clause {
            let trimmed = current.trim();
            // Ignore single-character dots like middle initials if too short
            if trimmed.len() >= 4 {
                sentences.push(trimmed.to_string());
                current.clear();
            }
        }
    }

    let remainder = current.trim();
    if !remainder.is_empty() {
        sentences.push(remainder.to_string());
    }

    if sentences.is_empty() && !text.is_empty() {
        sentences.push(text.to_string());
    }

    sentences
}

pub fn set_last_audio_wav(id: String, bytes: Vec<u8>) {
    if let Ok(mut id_lock) = LATEST_AUDIO_ID.lock() {
        *id_lock = Some(id.clone());
    }
    if let Ok(mut list) = LAST_AUDIO_WAV_LIST.lock() {
        // Keep maximum 3 recent audio items in memory (auto-evict older raw WAVs to prevent RAM growth)
        while list.len() >= 3 {
            list.pop_front();
        }
        list.push_back((id, bytes));
    }
}

pub fn get_last_audio_wav(req_id: Option<&str>) -> Option<Vec<u8>> {
    let key = if let Some(r_id) = req_id {
        if !r_id.is_empty() {
            Some(r_id.to_string())
        } else {
            LATEST_AUDIO_ID.lock().ok().and_then(|g| g.clone())
        }
    } else {
        LATEST_AUDIO_ID.lock().ok().and_then(|g| g.clone())
    };

    let target_key = key?;
    if let Ok(list) = LAST_AUDIO_WAV_LIST.lock() {
        for (id, bytes) in list.iter().rev() {
            if id == &target_key {
                return Some(bytes.clone());
            }
        }
    }
    None
}

pub fn clear_all_audio_cache() {
    if let Ok(mut list) = LAST_AUDIO_WAV_LIST.lock() {
        list.clear();
    }
}

pub fn evict_audio(target_id: &str) {
    if let Ok(mut list) = LAST_AUDIO_WAV_LIST.lock() {
        list.retain(|(id, _)| id != target_id);
    }
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
        _ => "id",
    }
    .to_string()
}

/// Decomposes Hangul syllables (U+AC00..U+D7A3) into Unicode Conjoining Jamo (U+1100..U+11FF)
/// matching Supertonic 3's unicode indexer expectations.
fn decompose_hangul(text: &str) -> String {
    let mut out = String::with_capacity(text.len() * 3);
    for ch in text.chars() {
        let code = ch as u32;
        if (0xAC00..=0xD7A3).contains(&code) {
            let s_index = code - 0xAC00;
            let l_index = s_index / 588;
            let v_index = (s_index % 588) / 28;
            let t_index = s_index % 28;

            if let Some(l_char) = char::from_u32(0x1100 + l_index) {
                out.push(l_char);
            }
            if let Some(v_char) = char::from_u32(0x1161 + v_index) {
                out.push(v_char);
            }
            if t_index > 0 {
                if let Some(t_char) = char::from_u32(0x11A7 + t_index) {
                    out.push(t_char);
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn preprocess_text(text: &str, lang: &str) -> String {
    let mut t = decompose_hangul(text);
    t = t
        .replace(['–', '‑', '—', '−'], "-")
        .replace(['¯', '_', '~', '^'], " ")
        .replace(['“', '”', '„', '«', '»', '"'], " ")
        .replace(['‘', '’', '‚', '´', '`', '′'], "'")
        .replace(['[', ']', '{', '}', '(', ')', '<', '>', '|', '/', '\\', '#', '→', '←', '•', '·', '…', '★', '☆'], " ");

    t = t.replace('@', " at ");
    t = t.replace('&', " dan ");
    t = t.replace('%', " persen ");
    t = t.replace('+', " plus ");
    t = t.replace('=', " sama dengan ");
    t = t.replace("e.g.,", "for example, ");
    t = t.replace("i.e.,", "that is, ");

    static RE_SP: std::sync::LazyLock<regex::Regex> =
        std::sync::LazyLock::new(|| regex::Regex::new(r"\s+").unwrap_or_else(|_| regex::Regex::new(" ").unwrap()));
    t = RE_SP.replace_all(&t, " ").trim().to_string();

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

    // Constant scaling factor across all chunks to prevent volume fluctuation
    // Supertonic raw waveform peak is typically ~0.30, constant 2.5x gain provides
    // consistent loudness without dynamic chunk-to-chunk pumping.
    const CONST_GAIN: f32 = 2.5;

    for &sample in samples {
        let scaled = sample * CONST_GAIN;
        let clamped = scaled.clamp(-1.0, 1.0);
        let s_i16 = (clamped * 32767.0) as i16;
        out.extend_from_slice(&s_i16.to_le_bytes());
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blend_voice_styles() {
        let style_a = VoiceStyleData {
            ttl_dims: [1, 1, 2],
            ttl_data: vec![1.0, 0.0],
            dp_dims: [1, 1, 2],
            dp_data: vec![2.0, 0.0],
        };
        let style_b = VoiceStyleData {
            ttl_dims: [1, 1, 2],
            ttl_data: vec![0.0, 1.0],
            dp_dims: [1, 1, 2],
            dp_data: vec![0.0, 2.0],
        };
        let blended = blend_voice_styles(&style_a, &style_b, 0.5);
        // Slerp for orthogonal vectors at t=0.5 produces cos(45°)=sin(45°) ≈ 0.7071
        // and preserves original vector magnitudes (norm 1.0 for ttl, 2.0 for dp)
        assert!((blended.ttl_data[0] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-4);
        assert!((blended.ttl_data[1] - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-4);
        assert!((blended.dp_data[0] - 2.0 * std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-4);
        assert!((blended.dp_data[1] - 2.0 * std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-4);
    }

    #[test]
    fn test_hangul_decomposition() {
        let decomposed = decompose_hangul("안녕하세요");
        // Each of the 5 Hangul characters should decompose into 2 or 3 Jamo characters
        assert!(decomposed.chars().count() >= 10);
        let preprocessed = preprocess_text("안녕하세요", "ko");
        assert!(preprocessed.starts_with("<ko>"));
        assert!(preprocessed.ends_with(".</ko>"));
    }

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
