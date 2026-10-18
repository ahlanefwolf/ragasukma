use std::sync::Arc;
use std::time::Instant;

use crate::audio::tts::TextToSpeech;
use crate::llm::LlmClient;
use crate::tray::{AssistantState, TrayState};

pub fn detect_emotion(text: &str) -> String {
    crate::ui::renderer::expressions::detect_emotion_with_fallback(text).to_string()
}

pub fn process_llm_response(
    llm: &mut LlmClient,
    tts: &Arc<TextToSpeech>,
    asst_name: &str,
    effective_input: &str,
) -> Result<String, anyhow::Error> {
    process_llm_response_with_role(llm, tts, asst_name, "user", effective_input)
}

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

    // Inisialisasi streaming voice session untuk sintesis & playback paralel per kalimat
    let mut voice_session = tts.start_streaming_session(asst_name);

    let result = llm.chat_stream_with_role(role, effective_input, |sentence, emotion| {
        // Strip any action tags or internal markdown before sending to TTS and printing
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

        // Push potongan kalimat langsung ke background TTS worker
        voice_session.push_sentence(&clean_sentence, emotion);

        // Update emotion avatar real-time to renderer 3D
        TrayState::update_with_emotion(
            AssistantState::Thinking,
            &format!("{asst_name} Berpikir"),
            emotion,
        );

        if first_token_time.is_none() {
            let first_elapsed = llm_start.elapsed();
            first_token_time = Some(first_elapsed);
            if !printed_header {
                print!("\r\n[{}] [{:.2?}]:\r\n   ", asst_name, first_elapsed);
                printed_header = true;
            }
        }

        TrayState::emit_event(
            "assistant:stream-chunk",
            serde_json::json!({
                "sentence": &clean_sentence,
                "chunk": &clean_sentence,
                "speaker": asst_name
            }),
        );

        print!("{} ", clean_sentence);
        use std::io::Write;
        let _ = std::io::stdout().flush();

        Ok(())
    });

    println!();

    let full_response_text = match &result {
        Ok(txt) => {
            // Selesaikan session streaming voice (sinkronisasi kalimat terakhir)
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

            txt.clone()
        }
        Err(e) => {
            // Hentikan antrean audio jika LLM terputus atau gagal
            voice_session.abort();

            let err_str = e.to_string();
            crate::term_err!("[LLM Error]: {err_str}");

            // Abaikan jika error disebabkan oleh interupsi pengguna
            if !err_str.contains("Interrupted by user") {
                let err_voice_msg = "Sorry, I encountered an error connecting to the language model. Please check the endpoint or authentication.";
                let _ = tts_clone.speak_with_emotion_and_lang(
                    err_voice_msg,
                    "neutral",
                    asst_name,
                    Some("en"),
                );
            }

            String::new()
        }
    };

    TrayState::emit_event(
        "assistant:response-done",
        serde_json::json!({
            "text": &full_response_text
        }),
    );

    result
}
