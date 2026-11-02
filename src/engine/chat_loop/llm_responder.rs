use std::sync::Arc;
use std::time::Instant;

use crate::audio::tts::TextToSpeech;
use crate::llm::LlmClient;
use crate::tray::{AssistantState, TrayState};

pub fn process_llm_response_with_role(
    llm: &mut LlmClient,
    tts: &Arc<TextToSpeech>,
    asst_name: &str,
    role: &str,
    effective_input: &str,
) -> Result<String, anyhow::Error> {
    crate::app_state::clear_interruption();
    crate::app_state::set_ai_speaking(false);
    TrayState::update(AssistantState::Thinking, "LLM Thinking...");
    let llm_start = Instant::now();
    crate::term_log!("\n[LLM] {} is responding (role: {})...", asst_name, role);

    let tts_clone = Arc::clone(tts);
    let mut first_token_time = None;
    let mut printed_header = false;

    // Pre-warm isolated TTS worker asynchronously while LLM processes first token
    crate::audio::supertonic::prewarm_tts_worker();

    // Initialize streaming voice session for parallel sentence synthesis & playback
    let mut voice_session = tts.start_streaming_session(asst_name);

    // Buffer and state for stream-to-sentence assembly
    let mut sentence_buffer = String::new();
    let mut in_think_block = false;

    let system_prompt = crate::engine::prompt::build_system_prompt_indexed(Some(effective_input));
    let result = llm.chat_stream_with_role_and_prompt(role, effective_input, Some(&system_prompt), |delta: &str| {
        sentence_buffer.push_str(delta);

        loop {
            if in_think_block {
                if let Some(end_idx) = sentence_buffer.find("</think>") {
                    sentence_buffer.drain(..end_idx + "</think>".len());
                    in_think_block = false;
                } else {
                    // Masih di dalam blok <think>, tunggu penutup
                    break;
                }
            }

            if let Some(start_idx) = sentence_buffer.find("<think>") {
                let pre_think = sentence_buffer[..start_idx].to_string();
                sentence_buffer.drain(..start_idx + "<think>".len());
                in_think_block = true;

                if !pre_think.trim().is_empty() {
                    dispatch_sentence_to_tts(
                        &pre_think,
                        &mut voice_session,
                        asst_name,
                        &mut first_token_time,
                        &mut printed_header,
                        llm_start,
                    )?;
                }
                continue;
            }

            // Cari batas kalimat di luar tag think: '.', '!', '?', '\n', '。', '！', '？'
            let mut split_pos = None;
            for (idx, ch) in sentence_buffer.char_indices() {
                if ch == '\n' || ch == '!' || ch == '?' || ch == '。' || ch == '！' || ch == '？' {
                    split_pos = Some(idx + ch.len_utf8());
                    break;
                } else if ch == '.' {
                    // Hindari pemotongan ellipsis (...) atau angka desimal (misal 3.14)
                    let next_char = sentence_buffer[idx + 1..].chars().next();
                    let is_ellipsis = next_char == Some('.');
                    let is_decimal = next_char.is_some_and(|c| c.is_ascii_digit());
                    if !is_ellipsis && !is_decimal {
                        split_pos = Some(idx + 1);
                        break;
                    }
                }
            }

            if let Some(pos) = split_pos {
                let sentence = sentence_buffer[..pos].to_string();
                sentence_buffer.drain(..pos);

                dispatch_sentence_to_tts(
                    &sentence,
                    &mut voice_session,
                    asst_name,
                    &mut first_token_time,
                    &mut printed_header,
                    llm_start,
                )?;
            } else {
                break;
            }
        }

        Ok(())
    });

    // Flush sisa teks di buffer yang tersisa setelah stream berakhir
    if !in_think_block && !sentence_buffer.trim().is_empty() {
        let _ = dispatch_sentence_to_tts(
            &sentence_buffer,
            &mut voice_session,
            asst_name,
            &mut first_token_time,
            &mut printed_header,
            llm_start,
        );
    }

    println!();

    match &result {
        Ok(txt) => {
            // Complete the voice streaming session (synchronize final sentence)
            voice_session.finish();

            let clean = txt.trim();
            if !clean.is_empty() {
                crate::term_log!("[{}] \"{}\"", asst_name, clean);
            }

            // Once the assistant finishes summarizing Google search results,
            // clear the transient text_summary so it doesn't linger in DOM memory forever
            if effective_input.contains("Google Search for") || effective_input.starts_with("Here are the search results for") {
                if let Some(mut snap) = crate::services::browser_session_service::get_latest_snapshot() {
                    snap.text_summary.clear();
                    let _ = crate::services::browser_session_service::save_browser_snapshot(snap);
                    crate::term_log!("[Browser AI Search] Cleared transient search summary from active DOM context.");
                }
            }
        }
        Err(e) => {
            // Abort audio queue if LLM connection drops or fails
            voice_session.abort();

            let err_str = e.to_string();
            crate::term_err!("[LLM Error]: {err_str}");

            // Ignore if error was caused by user interruption
            if !err_str.contains("Interrupted by user") {
                let err_voice_msg = "Sorry, I encountered an error connecting to the language model. Please check the endpoint or authentication.";
                let _ = tts_clone.speak_with_emotion_and_lang(
                    err_voice_msg,
                    "neutral",
                    asst_name,
                    Some("en"),
                );
            }
        }
    }

    result
}

/// Splits long sentences into smaller responsive chunks based on punctuation clauses
/// and word boundaries so TTS synthesizes immediately and keeps lip sync perfectly synced.
fn split_sentence_for_streaming(sentence: &str, max_chars: usize) -> Vec<String> {
    let trimmed = sentence.trim();
    if trimmed.chars().count() <= max_chars {
        return vec![trimmed.to_string()];
    }

    let mut result = Vec::new();
    let mut current_buf = String::new();

    // Split candidate segments by primary clause delimiters
    for part in trimmed.split_inclusive(&[',', ';', ':', '—', '-'][..]) {
        let part_trimmed = part.trim();
        if part_trimmed.is_empty() {
            continue;
        }

        if current_buf.is_empty() {
            current_buf.push_str(part_trimmed);
        } else if current_buf.chars().count() + part_trimmed.chars().count() < max_chars {
            current_buf.push(' ');
            current_buf.push_str(part_trimmed);
        } else {
            result.push(current_buf.trim().to_string());
            current_buf.clear();
            current_buf.push_str(part_trimmed);
        }
    }

    if !current_buf.is_empty() {
        result.push(current_buf.trim().to_string());
    }

    // Secondary fallback: if any segment is still longer than max_chars (e.g. no punctuation),
    // split cleanly at word boundaries.
    let mut final_chunks = Vec::new();
    for seg in result {
        if seg.chars().count() <= max_chars {
            final_chunks.push(seg);
        } else {
            let mut word_buf = String::new();
            for word in seg.split_whitespace() {
                if word_buf.is_empty() {
                    word_buf.push_str(word);
                } else if word_buf.chars().count() + word.chars().count() < max_chars {
                    word_buf.push(' ');
                    word_buf.push_str(word);
                } else {
                    final_chunks.push(word_buf.trim().to_string());
                    word_buf.clear();
                    word_buf.push_str(word);
                }
            }
            if !word_buf.is_empty() {
                final_chunks.push(word_buf.trim().to_string());
            }
        }
    }

    if final_chunks.is_empty() && !trimmed.is_empty() {
        vec![trimmed.to_string()]
    } else {
        final_chunks
    }
}

fn dispatch_sentence_to_tts(
    sentence: &str,
    voice_session: &mut crate::audio::tts::StreamingVoiceSession,
    asst_name: &str,
    first_token_time: &mut Option<std::time::Duration>,
    printed_header: &mut bool,
    llm_start: std::time::Instant,
) -> anyhow::Result<()> {
    let active_emotion = crate::ui::renderer::expressions::detect_emotion(sentence).unwrap_or("neutral");

    let clean_sentence = crate::llm::cleaner::clean_ai_text(sentence)
        .trim()
        .to_string();
    if !clean_sentence.chars().any(|c| c.is_alphanumeric()) {
        return Ok(());
    }

    if crate::app_state::is_interruption_requested() {
        voice_session.abort();
        return Err(anyhow::anyhow!("Interrupted by user"));
    }

    let sub_chunks = split_sentence_for_streaming(&clean_sentence, 70);

    for chunk in sub_chunks {
        let chunk_clean = chunk.trim();
        if chunk_clean.is_empty() {
            continue;
        }

        voice_session.push_sentence(chunk_clean, active_emotion);

        let state = if crate::app_state::is_ai_speaking() {
            AssistantState::Speaking
        } else {
            AssistantState::Thinking
        };
        TrayState::update_with_emotion(
            state,
            &format!(
                "{asst_name} {}",
                if state == AssistantState::Speaking {
                    "Speaking"
                } else {
                    "Thinking"
                }
            ),
            active_emotion,
        );
    }

    if first_token_time.is_none() {
        let first_elapsed = llm_start.elapsed();
        *first_token_time = Some(first_elapsed);
        if !*printed_header {
            print!("\r\n[{}] [{:.2?}]:\r\n   ", asst_name, first_elapsed);
            *printed_header = true;
        }
    }

    print!("{} ", clean_sentence);
    use std::io::Write;
    let _ = std::io::stdout().flush();

    Ok(())
}
