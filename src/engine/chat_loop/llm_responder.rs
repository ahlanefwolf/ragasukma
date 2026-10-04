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
    crate::app_state::clear_interruption();
    crate::app_state::set_ai_speaking(false);
    TrayState::update(AssistantState::Thinking, "LLM Thinking...");
    let llm_start = Instant::now();
    crate::term_log!("\n[LLM] {} is responding...", asst_name);

    let tts_clone = Arc::clone(tts);
    let mut first_token_time = None;
    let mut printed_header = false;

    let result = llm.chat_stream(effective_input, |sentence, emotion| {
        let clean_sentence = sentence.trim().to_string();
        if clean_sentence.is_empty() {
            return Ok(());
        }

        if crate::app_state::is_interruption_requested() {
            return Err(anyhow::anyhow!("Interrupted by user"));
        }

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
            let clean = txt.trim();
            if !clean.is_empty() {
                crate::term_log!("[{}] \"{}\"", asst_name, clean);
                let emo = detect_emotion(clean);
                let _ = tts_clone.speak_with_emotion(clean, &emo, asst_name);
            }
            txt.clone()
        }
        Err(e) => {
            crate::term_err!("[LLM Error]: {e}");
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
