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
    let mut found_skill = false;
    let mut executed_skill_ids = std::collections::HashSet::new();

    for cap in RE_SKILL.captures_iter(full_reply) {
        if let Some(matched_id) = cap.get(1) {
            let mut id = matched_id.as_str().trim().to_string();

            // Guardrail Otomatis: Jika user eksplisit menunjuk layar/desktop, tapi LLM ceroboh memilih browser:click_*
            let user_lower = effective_input.to_lowercase();
            let is_explicit_desktop_intent = user_lower.contains("di layar")
                || user_lower.contains("di layarnya")
                || user_lower.contains("pada layar")
                || user_lower.contains("di desktop")
                || user_lower.contains("pada desktop")
                || user_lower.contains("di window")
                || user_lower.contains("pada window");

            if is_explicit_desktop_intent && id.starts_with("browser:click_") {
                if let Some(target) = id.split_once('|').map(|x| x.1) {
                    crate::term_log!(
                        "🛡️ [Smart Reroute]: Perintah user eksplisit menunjuk layar/desktop. Mengalihkan '{}' ke 'desktop:click_text|{}'",
                        id,
                        target
                    );
                    id = format!("desktop:click_text|{}", target);
                }
            }

            if !executed_skill_ids.insert(id.clone()) {
                continue;
            }
            found_skill = true;
            crate::term_log!("\n[MEMPROSES SKILL]: Memanggil ID '{}'...", id);

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
                } else if id_lower.starts_with("desktop:focus")
                    || id_lower.starts_with("app:open")
                    || id_lower == "desktop:scan"
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
                        "output": "Skill tidak ditemukan"
                    }),
                );
            }
        }
    }

    if !found_skill {
        let lower_in = effective_input.to_lowercase();
        if lower_in.contains("buka terminal")
            || lower_in.contains("open terminal")
            || lower_in.contains("bukain terminal")
            || lower_in.contains("buka console")
        {
            crate::term_log!("\n[SMART FALLBACK]: Terdeteksi permintaan buka terminal.");
            if let Some((skill_name, output)) = execute_skill_by_identifier("buka terminal") {
                crate::term_log!(
                    "[SKILL BERHASIL: {}] -> {}",
                    skill_name,
                    output.trim()
                );
                TrayState::emit_event(
                    "assistant:skill",
                    serde_json::json!({
                        "id": "buka terminal",
                        "name": skill_name,
                        "status": "success",
                        "output": output.trim()
                    }),
                );
            }
        }
    }
}
