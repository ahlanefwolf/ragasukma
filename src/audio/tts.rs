use anyhow::Result;

pub struct TextToSpeech;

#[derive(Debug)]
pub enum StreamingVoiceMsg {
    Sentence {
        text: String,
        emotion: String,
    },
    Finish,
    Abort,
}

pub struct StreamingVoiceSession {
    tx: Option<std::sync::mpsc::Sender<StreamingVoiceMsg>>,
    worker_handle: Option<std::thread::JoinHandle<()>>,
}

impl StreamingVoiceSession {
    pub fn push_sentence(&self, text: &str, emotion: &str) {
        if let Some(ref tx) = self.tx {
            let _ = tx.send(StreamingVoiceMsg::Sentence {
                text: text.to_string(),
                emotion: emotion.to_string(),
            });
        }
    }

    pub fn finish(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(StreamingVoiceMsg::Finish);
        }
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }

    pub fn abort(&mut self) {
        if let Some(tx) = self.tx.take() {
            let _ = tx.send(StreamingVoiceMsg::Abort);
        }
        crate::audio::player::stop_native_voice();
        if let Some(handle) = self.worker_handle.take() {
            let _ = handle.join();
        }
    }
}

impl Drop for StreamingVoiceSession {
    fn drop(&mut self) {
        self.finish();
    }
}

impl TextToSpeech {
    pub fn new() -> Self {
        Self
    }

    /// Creates an asynchronous, sentence-level streaming TTS session.
    /// Each sentence is synthesized concurrently while subsequent tokens/sentences are still streaming from LLM.
    pub fn start_streaming_session(&self, asst_name: &str) -> StreamingVoiceSession {
        let (tx, rx) = std::sync::mpsc::channel::<StreamingVoiceMsg>();
        let asst_name = asst_name.to_string();

        let worker_handle = std::thread::Builder::new()
            .name("ragasukma-tts-stream".to_string())
            .stack_size(1024 * 1024)
            .spawn(move || {
                let session_id = format!(
                    "{:x}_{}",
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis(),
                    std::process::id()
                );
                let cfg = crate::assets::load_config();
                let voice = if !cfg.tts_voice_name.is_empty() {
                    cfg.tts_voice_name.clone()
                } else {
                    "F5".to_string()
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
                if cfg.tts_volume > 0.0 {
                    crate::audio::player::set_voice_volume(cfg.tts_volume);
                }

                let mut native_pc_stream: Option<crate::audio::player::VoiceStreamSender> = None;
                let mut accumulated_full_text = String::new();
                let mut first_emotion = String::new();
                let mut sentence_count = 0usize;
                let mut global_chunk_counter = 0usize;
                let mut total_audio_duration = 0.0f32;

                while let Ok(msg) = rx.recv() {
                    match msg {
                        StreamingVoiceMsg::Sentence { text, emotion } => {
                            if crate::app_state::is_interruption_requested() {
                                if let Some(ref stream) = native_pc_stream {
                                    stream.finish();
                                }
                                crate::audio::player::stop_native_voice();
                                break;
                            }

                            let clean_text = clean_text_for_speech(&text);
                            if !clean_text.chars().any(|c| c.is_alphanumeric()) {
                                continue;
                            }

                            if first_emotion.is_empty() {
                                first_emotion = emotion.clone();
                            }

                            if !accumulated_full_text.is_empty() {
                                accumulated_full_text.push(' ');
                            }
                            accumulated_full_text.push_str(&clean_text);

                            // Lazily initialize native PC stream upon first spoken sentence
                            if native_pc_stream.is_none() && !crate::app_state::is_voice_muted() {
                                native_pc_stream = Some(crate::audio::player::start_voice_stream(
                                    emotion.clone(),
                                    asst_name.clone(),
                                ));
                            }

                            let engine_opt = crate::audio::supertonic::get_engine()
                                .or_else(|| crate::audio::supertonic::init_engine().ok());

                            if let Some(engine) = engine_opt {
                                crate::engine::os_efficiency::notify_active();
                                let pc_stream_ref = native_pc_stream.as_ref();
                                let clean_for_closure = clean_text.clone();
                                let emo_for_closure = emotion.clone();
                                let voice_for_closure = voice.clone();
                                let lang_for_closure = lang.clone();
                                let cur_sent_idx = sentence_count;
                                let sess_id_closure = session_id.clone();

                                let synth_res = engine.synthesize_stream(
                                    &clean_text,
                                    &voice,
                                    &lang,
                                    speed,
                                    &mut |chunk_idx: usize,
                                          total_chunks: usize,
                                          chunk_samples: Vec<f32>,
                                          chunk_rate: u32,
                                          chunk_wav: Vec<u8>| {
                                        if crate::app_state::is_interruption_requested() {
                                            return;
                                        }

                                        if let Some(pc_stream) = pc_stream_ref {
                                            pc_stream.push_chunk_pcm(chunk_samples, chunk_rate);
                                        }

                                        let chunk_global_idx = global_chunk_counter;
                                        global_chunk_counter += 1;

                                        let chunk_id = format!(
                                            "{:x}_{}_{}",
                                            simple_hash_u64(&clean_for_closure),
                                            cur_sent_idx,
                                            chunk_idx
                                        );
                                        crate::audio::supertonic::set_last_audio_wav(
                                            chunk_id.clone(),
                                            chunk_wav.clone(),
                                        );

                                        let chunk_audio_url = format!("/api/tts/audio?id={}", chunk_id);
                                        let chunk_dur = wav_duration_secs(&chunk_wav).unwrap_or(0.0);

                                        let chunk_payload = serde_json::json!({
                                            "text": clean_for_closure,
                                            "emotion": emo_for_closure,
                                            "audio_url": Some(chunk_audio_url),
                                            "duration": chunk_dur,
                                            "voice": voice_for_closure,
                                            "lang": lang_for_closure,
                                            "chunk_index": chunk_global_idx,
                                            "sentence_index": cur_sent_idx,
                                            "session_id": sess_id_closure,
                                            "sub_chunk_index": chunk_idx,
                                            "sub_total_chunks": total_chunks,
                                            "total_chunks": chunk_global_idx + 1,
                                            "is_chunk": true,
                                            "native_played": !crate::app_state::is_voice_muted()
                                        })
                                        .to_string();
                                        crate::app_state::push_client_action("assistant:speak", Some(&chunk_payload));
                                    },
                                );

                                if let Ok(wav_bytes) = synth_res {
                                    if let Some(dur) = wav_duration_secs(&wav_bytes) {
                                        total_audio_duration += dur;
                                    }
                                }
                                sentence_count += 1;
                            }
                        }
                        StreamingVoiceMsg::Finish => {
                            if let Some(pc_stream) = native_pc_stream.take() {
                                pc_stream.finish();
                            }

                            if !accumulated_full_text.is_empty() {
                                let payload = serde_json::json!({
                                    "text": accumulated_full_text,
                                    "emotion": if first_emotion.is_empty() { "neutral" } else { &first_emotion },
                                    "audio_url": None::<String>,
                                    "duration": if total_audio_duration > 0.0 { Some(total_audio_duration) } else { None },
                                    "voice": voice,
                                    "lang": lang,
                                    "session_id": session_id,
                                    "total_chunks": global_chunk_counter,
                                    "is_chunk": false,
                                    "is_finish": true,
                                    "native_played": !crate::app_state::is_voice_muted()
                                })
                                .to_string();
                                crate::app_state::push_client_action("assistant:speak", Some(&payload));
                            }
                            break;
                        }
                        StreamingVoiceMsg::Abort => {
                            if let Some(pc_stream) = native_pc_stream.take() {
                                pc_stream.finish();
                            }
                            crate::audio::player::stop_native_voice();
                            break;
                        }
                    }
                }

                // Purge heap caches and return freed OS pages immediately after synthesis session completes
                crate::engine::os_efficiency::trim_process_memory();
            })
            .ok();

        StreamingVoiceSession {
            tx: Some(tx),
            worker_handle,
        }
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
        if !clean_text.chars().any(|c| c.is_alphanumeric()) {
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
        if cfg.tts_volume > 0.0 {
            crate::audio::player::set_voice_volume(cfg.tts_volume);
        }

        let audio_id = format!("{:x}", simple_hash_u64(&clean_text));
        let audio_url = format!("/api/tts/audio?id={audio_id}");

        // Initialize engine if not yet ready, and synthesize Supertonic-3 audio in streaming chunks
        let mut audio_bytes_opt: Option<Vec<u8>> = None;
        let asst_name_str = asst_name.to_string();
        let emotion_str = emotion.to_string();

        // ---------------------------------------------------------------------
        // Worker 3 (Mobile Opus Network Worker): Background Task via MPSC Channel
        // ---------------------------------------------------------------------
        struct MobileChunkItem {
            chunk_id: String,
            chunk_wav: Vec<u8>,
            chunk_idx: usize,
            total_chunks: usize,
        }

        let (mobile_tx, mobile_rx) = std::sync::mpsc::channel::<MobileChunkItem>();

        let clean_text_for_worker = clean_text.clone();
        let emotion_for_worker = emotion_str.clone();
        let voice_for_worker = voice.clone();
        let lang_for_worker = lang.clone();

        let mobile_worker_handle = std::thread::Builder::new()
            .name("ragasukma-tts-mobile".to_string())
            .stack_size(512 * 1024)
            .spawn(move || {
            while let Ok(item) = mobile_rx.recv() {
                // 1. Simpan audio chunk WAV ke in-memory cache
                crate::audio::supertonic::set_last_audio_wav(item.chunk_id.clone(), item.chunk_wav.clone());

                // 2. Broadcast chunk immediately to mobile clients over WebSocket
                let chunk_audio_url = format!("/api/tts/audio?id={}", item.chunk_id);
                let chunk_duration = wav_duration_secs(&item.chunk_wav);
                let chunk_payload = serde_json::json!({
                    "text": clean_text_for_worker,
                    "emotion": emotion_for_worker,
                    "audio_url": Some(chunk_audio_url),
                    "duration": chunk_duration,
                    "voice": voice_for_worker,
                    "lang": lang_for_worker,
                    "chunk_index": item.chunk_idx,
                    "total_chunks": item.total_chunks,
                    "session_id": format!("{:x}", simple_hash_u64(&clean_text_for_worker)),
                    "is_chunk": true,
                    "native_played": !crate::app_state::is_voice_muted()
                })
                .to_string();
                crate::app_state::push_client_action("assistant:speak", Some(&chunk_payload));
            }
        });

        // ---------------------------------------------------------------------
        // Worker 2 (Native Audio & Lip-Sync Worker): Start Stream Receiver
        // ---------------------------------------------------------------------
        let native_pc_stream = if !crate::app_state::is_voice_muted() {
            Some(crate::audio::player::start_voice_stream(
                emotion_str,
                asst_name_str,
            ))
        } else {
            None
        };

        // ---------------------------------------------------------------------
        // Worker 1 (Chunk Inference Producer): Generate WAV Chunks via ONNX
        // ---------------------------------------------------------------------
        let mut on_chunk = |chunk_idx: usize, total_chunks: usize, chunk_samples: Vec<f32>, chunk_rate: u32, chunk_wav: Vec<u8>| {
            // Jika ada interupsi dari user, jangan teruskan chunk ke player/mobile
            if crate::app_state::is_interruption_requested() {
                return;
            }

            let chunk_id = if total_chunks > 1 {
                format!("{}_{}", audio_id, chunk_idx)
            } else {
                audio_id.clone()
            };

            // Stream raw PCM samples directly to PC speakers without WAV decoding latency
            if let Some(ref pc_stream) = native_pc_stream {
                pc_stream.push_chunk_pcm(chunk_samples, chunk_rate);
            }

            // Push to mobile background worker for async Opus encoding & WebSocket transmission
            let _ = mobile_tx.send(MobileChunkItem {
                chunk_id,
                chunk_wav,
                chunk_idx,
                total_chunks,
            });
        };

        let engine_opt = crate::audio::supertonic::get_engine()
            .or_else(|| crate::audio::supertonic::init_engine().ok());

        let synthesized = if let Some(engine) = engine_opt {
            crate::engine::os_efficiency::notify_active();
            match engine.synthesize_stream(&clean_text, &voice, &lang, speed, &mut on_chunk) {
                Ok(bytes) => {
                    crate::engine::os_efficiency::notify_active();
                    // Close sender to signal mobile worker when all chunks are pushed
                    drop(mobile_tx);

                    // Signal PC native audio stream that all chunks have been pushed
                    if let Some(ref pc_stream) = native_pc_stream {
                        pc_stream.finish();
                    }

                    // Simpan full WAV bytes ke in-memory cache
                    crate::audio::supertonic::set_last_audio_wav(audio_id.clone(), bytes.clone());

                    audio_bytes_opt = Some(bytes);
                    true
                }
                Err(e) => {
                    drop(mobile_tx);
                    if let Some(ref pc_stream) = native_pc_stream {
                        pc_stream.finish();
                    }
                    crate::term_err!("[Supertonic TTS] Failed to synthesize audio: {e}");
                    false
                }
            }
        } else {
            drop(mobile_tx);
            if let Some(ref pc_stream) = native_pc_stream {
                pc_stream.finish();
            }
            false
        };

        // Ensure background mobile worker thread finishes cleanly
        if let Ok(handle) = mobile_worker_handle {
            let _ = handle.join();
        }

        let actual_audio_duration = audio_bytes_opt
            .as_ref()
            .and_then(|bytes| wav_duration_secs(bytes));

        let native_played = synthesized && !crate::app_state::is_voice_muted();

        // 1. Sync full text and fallback combined audio_url to clients (Desktop & Mobile) via action
        let payload = serde_json::json!({
            "text": clean_text,
            "emotion": emotion,
            "audio_url": if synthesized { Some(audio_url) } else { None },
            "duration": actual_audio_duration,
            "voice": voice,
            "lang": lang,
            "is_chunk": false,
            "native_played": native_played
        })
        .to_string();
        crate::app_state::push_client_action("assistant:speak", Some(&payload));

        // 2. Precision passive fail-safe watchdog based on actual WAV duration:
        // Calculated directly from duration in seconds of synthesized Supertonic-3 WAV audio.
        let epoch = crate::app_state::next_speaking_epoch();
        crate::app_state::touch_speech_activity();

        let exact_audio_secs = actual_audio_duration.unwrap_or_else(|| {
            (clean_text.chars().count() as f32 / 10.0).max(5.0)
        });
        // Generous tolerance buffer based on length to account for long text playback
        let tolerance_buffer = (exact_audio_secs * 0.4).clamp(10.0, 60.0);
        let timeout_limit_secs = (exact_audio_secs + tolerance_buffer).ceil() as u64;

        let _ = std::thread::Builder::new()
            .name("ragasukma-tts-timeout".to_string())
            .stack_size(512 * 1024)
            .spawn(move || {
            let mut last_activity = std::time::Instant::now();
            loop {
                std::thread::sleep(std::time::Duration::from_millis(400));

                // Exit if speech finished normally or a new speech epoch began
                if !crate::app_state::is_ai_speaking()
                    || crate::app_state::current_speaking_epoch() != epoch
                {
                    break;
                }

                // If native PC audio is still actively playing, refresh activity timer
                if crate::audio::player::is_native_voice_playing() {
                    crate::app_state::touch_speech_activity();
                    last_activity = std::time::Instant::now();
                    continue;
                }

                // If duration since last active audio playback exceeds timeout buffer, force avatar back to Standby
                if last_activity.elapsed().as_secs() >= timeout_limit_secs {
                    crate::term_log!(
                        "[TTS Watchdog] Audio playback timeout reached ({exact_audio_secs:.1}s), returning avatar to Standby."
                    );
                    crate::audio::player::stop_native_voice();
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
