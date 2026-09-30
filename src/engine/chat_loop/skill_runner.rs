use std::sync::Arc;

use crate::audio::tts::TextToSpeech;
use crate::engine::actions::execute_skill_by_identifier;
use crate::llm::LlmClient;
use crate::tray::TrayState;

static RE_SKILL: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
    regex::Regex::new(r"\[(?:SKILL|CMD):([^\]]+)\]").expect("valid skill regex")
});

/// Menjalankan eksekusi skill tool parsing `[CMD:...]` atau `[SKILL:...]`,
/// termasuk 2-step agentic loop untuk tindakan lanjutan desktop/browser.
pub fn handle_skill_execution(
    _llm: &mut LlmClient,
    _tts: &Arc<TextToSpeech>,
    _asst_name: &str,
    full_reply: &str,
    effective_input: &str,
) {
    let mut executed_skill_ids = std::collections::HashSet::new();

    // Universal normalization: map any raw tool-call blocks (<dots_function_call>, <invoke>, etc.)
    // to canonical [CMD:...] so commands from any LLM model run reliably.
    let normalized_reply = crate::llm::cleaner::normalize_tool_calls_to_commands(full_reply);

    for cap in RE_SKILL.captures_iter(&normalized_reply) {
        if let Some(matched_id) = cap.get(1) {
            let id = matched_id.as_str().trim().to_string();

            // Guardrail Otomatis: Jika respon ini dipicu oleh notifikasi internal STATUS SISTEM,
            // cegah pemanggilan ulang media:play yang mereplay lagu yang baru selesai diunduh.
            if effective_input.starts_with("[SYSTEM STATUS:") && id.starts_with("media:play") {
                crate::term_log!(
                    "🛡️ [Duplicate Skill Guard]: Ignoring '{}' because the song was already played automatically by the system",
                    id
                );
                continue;
            }

            if !executed_skill_ids.insert(id.clone()) {
                continue;
            }
            crate::term_log!("\n[SKILL]: Invoking ID '{}'...", id);

            if let Some((skill_name, output)) = execute_skill_by_identifier(&id) {
                crate::term_log!(
                    "[SKILL BERHASIL: {} ({})] -> {}",
                    skill_name,
                    id,
                    output.trim()
                );
                TrayState::emit_event(
                    "assistant:skill",
                    serde_json::json!({
                        "id": id,
                        "name": skill_name,
                        "status": "success",
                        "output": output.trim()
                    }),
                );

                // Update snapshot jika berganti window atau browser
                let id_lower = id.to_lowercase();
                if id_lower == "desktop:scan_visual" {
                    if let Ok(snap) = crate::engine::window_scanner::scan_active_window_sync() {
                        let _ = snap.save_to_file();
                    }
                } else if id_lower.starts_with("desktop:focus") || id_lower.starts_with("app:open")
                {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    if let Ok(snap) = crate::engine::window_scanner::scan_active_window_sync() {
                        let _ = snap.save_to_file();
                    }
                }
            } else {
                crate::term_err!(
                    "[SKILL GAGAL]: Identifier '{}' tidak cocok di skill-command.md",
                    id
                );
                TrayState::emit_event(
                    "assistant:skill",
                    serde_json::json!({
                        "id": id,
                        "name": id,
                        "status": "failed",
                        "output": "Skill not found"
                    }),
                );
            }
        }
    }
}
