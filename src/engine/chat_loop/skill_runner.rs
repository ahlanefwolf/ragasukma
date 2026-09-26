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
    llm: &mut LlmClient,
    tts: &Arc<TextToSpeech>,
    asst_name: &str,
    full_reply: &str,
    effective_input: &str,
) {
    let mut found_skill = false;
    let mut follow_up_prompt = None;
    let mut executed_skill_ids = std::collections::HashSet::new();

    for cap in RE_SKILL.captures_iter(full_reply) {
        if let Some(matched_id) = cap.get(1) {
            let id = matched_id.as_str().trim();
            if !executed_skill_ids.insert(id.to_string()) {
                continue;
            }
            found_skill = true;
            crate::term_log!("\n[MEMPROSES SKILL]: Memanggil ID '{}'...", id);

            if let Some((skill_name, output)) = execute_skill_by_identifier(id) {
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

                // Two-Step Agentic Loop: Kumpulkan observasi visual / window / browser
                let id_lower = id.to_lowercase();
                if id_lower.starts_with("desktop:scan_visual")
                    || id_lower.starts_with("vision:scan")
                    || id_lower.starts_with("desktop:ocr")
                {
                    if let Ok(snap) = crate::engine::window_scanner::ActiveWindowSnapshot::load_from_file() {
                        let btns: Vec<String> = snap
                            .buttons
                            .iter()
                            .take(35)
                            .map(|b| {
                                if let Some(c) = b.center {
                                    format!("#{}: \"{}\" [center: {},{}]", b.id, b.label, c[0], c[1])
                                } else {
                                    format!("#{}: \"{}\"", b.id, b.label)
                                }
                            })
                            .collect();
                        let texts: Vec<String> = snap.texts.iter().take(25).cloned().collect();
                        follow_up_prompt = Some(format!(
                            "[OBSERVASI SISTEM: Hasil pindai visual layar: {} tombol/ikon terdeteksi: [{}]. Teks di layar: [{}]. Lanjutkan perintah pengguna: '{}']",
                            snap.buttons.len(),
                            btns.join(", "),
                            texts.join(", "),
                            effective_input
                        ));
                    }
                } else if id_lower.starts_with("desktop:focus")
                    || id_lower.starts_with("app:open")
                    || id_lower == "desktop:scan"
                    || id_lower.starts_with("desktop:scan|")
                {
                    std::thread::sleep(std::time::Duration::from_millis(350));
                    if let Ok(snap) = crate::engine::window_scanner::scan_active_window_sync() {
                        let _ = snap.save_to_file();
                        let btns: Vec<String> = snap
                            .buttons
                            .iter()
                            .take(35)
                            .map(|b| {
                                if let Some(c) = b.center {
                                    format!("#{}: \"{}\" [{},{}]", b.id, b.label, c[0], c[1])
                                } else {
                                    format!("#{}: \"{}\"", b.id, b.label)
                                }
                            })
                            .collect();
                        let btn_str = if btns.is_empty() {
                            "(Tidak ada tombol native terdeteksi)".to_string()
                        } else {
                            btns.join(", ")
                        };
                        follow_up_prompt = Some(format!(
                            "[OBSERVASI SISTEM: Jendela '{}' ({}) sekarang aktif di depan. Tombol tersedia: {}. Lanjutkan perintah pengguna: '{}']",
                            snap.app_name, snap.window_title, btn_str, effective_input
                        ));
                    }
                } else if id_lower.starts_with("browser:open")
                    || id_lower.starts_with("browser:switch")
                    || id_lower.starts_with("browser:search")
                {
                    std::thread::sleep(std::time::Duration::from_millis(600));
                    if let Some(snap) = crate::services::get_latest_snapshot() {
                        let inputs: Vec<String> = snap
                            .inputs
                            .iter()
                            .take(5)
                            .map(|i| {
                                let ph = i.placeholder.as_deref().unwrap_or("");
                                format!("input(placeholder: \"{}\")", ph)
                            })
                            .collect();
                        let btns: Vec<String> = snap
                            .buttons
                            .iter()
                            .take(20)
                            .filter_map(|b| b.text.as_deref().map(|t| format!("\"{}\"", t.trim())))
                            .filter(|s| s.len() > 2)
                            .collect();
                        let texts: Vec<String> = snap.texts.iter().take(15).cloned().collect();
                        follow_up_prompt = Some(format!(
                            "[OBSERVASI SISTEM: Halaman web '{}' ({}) aktif. Teks/Link terlihat: [{}]. Tombol: [{}]. Form: [{}]. Lanjutkan perintah pengguna: '{}']",
                            snap.title,
                            snap.url,
                            texts.join(", "),
                            btns.join(", "),
                            inputs.join(", "),
                            effective_input
                        ));
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

    // Step 2: Content-Level Agentic Loop
    if let Some(obs) = follow_up_prompt {
        let raw_in = effective_input.to_lowercase();
        let needs_second_step = raw_in.contains("klik")
            || raw_in.contains("pencet")
            || raw_in.contains("tekan")
            || raw_in.contains("tulis")
            || raw_in.contains("ketik")
            || raw_in.contains("angka")
            || raw_in.contains("tombol")
            || raw_in.contains("hitung")
            || raw_in.contains("tambah")
            || raw_in.contains("video")
            || raw_in.contains("link")
            || raw_in.contains("scroll")
            || raw_in.contains("enter")
            || raw_in.contains("terus")
            || raw_in.contains("lalu");

        if needs_second_step {
            crate::term_log!("\n🔄 [Agentic Loop Step 2]: Menjalankan evaluasi lanjutan berdasarkan isi jendela aktif...");
            let tts_step2 = Arc::clone(tts);
            let asst = asst_name.to_string();

            let step2_result = llm.chat_stream(&obs, |sentence, _emotion| {
                let clean = sentence.trim().to_string();
                if clean.is_empty() {
                    return Ok(());
                }
                print!("{} ", clean);
                use std::io::Write;
                let _ = std::io::stdout().flush();
                Ok(())
            });
            println!();

            if let Ok(step2_text) = &step2_result {
                let clean = step2_text.trim();
                if !clean.is_empty() {
                    let _ = tts_step2.speak_with_emotion(clean, "neutral", &asst);
                }
            }

            if let Ok(step2_text) = step2_result {
                for cap2 in RE_SKILL.captures_iter(&step2_text) {
                    if let Some(matched_id2) = cap2.get(1) {
                        let id2 = matched_id2.as_str().trim();
                        crate::term_log!("\n[MEMPROSES SKILL TAHAP 2]: Memanggil ID '{}'...", id2);
                        if let Some((skill_name2, output2)) = execute_skill_by_identifier(id2) {
                            crate::term_log!(
                                "[SKILL BERHASIL: {} ({})] -> {}",
                                skill_name2,
                                id2,
                                output2.trim()
                            );
                            TrayState::emit_event(
                                "assistant:skill",
                                serde_json::json!({
                                    "id": id2,
                                    "name": skill_name2,
                                    "status": "success",
                                    "output": output2.trim()
                                }),
                            );
                        }
                    }
                }
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
