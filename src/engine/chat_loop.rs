use anyhow::Result;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use super::actions::execute_skill_by_identifier;
use super::session::{AppSession, check_and_consume_cmd};
use crate::audio::dsp::TARGET_SAMPLE_RATE;
use crate::tray::{AssistantState, TrayState};

impl AppSession {
    pub fn start_chat_loop(&mut self) -> Result<()> {
        crate::app_state::update_session_running(true);
        TrayState::update(AssistantState::Idle, "Standby Siap");
        crate::term_log!("====================================================");
        crate::term_log!("✨ SESI PERCAKAPAN RAGASUKMA DIMULAI");
        crate::term_log!("   - Asisten : {}", self.config.assistant_name);
        crate::term_log!("   - Dashboard: http://127.0.0.1:42128");
        crate::term_log!("====================================================\n");

        let mut is_paused = false;

        let (turn_tx, turn_rx) = std::sync::mpsc::channel::<Vec<f32>>();
        let (text_tx, text_rx) = std::sync::mpsc::channel::<String>();
        crate::engine::remote_text::set_remote_text_sender(text_tx);
        let stop_worker = Arc::new(AtomicBool::new(false));
        let pause_worker = Arc::new(AtomicBool::new(false));

        let stop_worker_clone = Arc::clone(&stop_worker);
        let pause_worker_clone = Arc::clone(&pause_worker);
        let recorder_ref = Arc::clone(&self.recorder);

        // Spawn Continuous Audio Background Pipeline
        let worker_handle = std::thread::spawn(move || {
            recorder_ref.start_continuous_worker(stop_worker_clone, pause_worker_clone, turn_tx);
        });

        loop {
            // Check if a command was sent from GUI or tray before starting the turn
            if let Some(cmd) = check_and_consume_cmd() {
                if cmd == "stop_chat" || cmd == "quit" {
                    stop_worker.store(true, Ordering::SeqCst);
                    let _ = worker_handle.join();
                    crate::app_state::update_session_running(false);
                    TrayState::update(AssistantState::Idle, "Standby");
                    crate::term_log!("\n[SELESAI] Sesi obrolan dihentikan.");
                    return Ok(());
                } else if cmd == "toggle_pause" {
                    is_paused = true;
                } else if cmd == "reset_session" || cmd == "reset_chat" {
                    self.llm.reset_history();
                    crate::term_log!(
                        "🔄 [Reset Session] Riwayat memori obrolan AI berhasil di-reset!"
                    );
                }
            }

            // Reload konfigurasi secara reactive sebelum mulai giliran mendengar berikutnya
            self.reload_config_if_changed();

            // Sesi Ngobrol Aktif
            if is_paused {
                pause_worker.store(true, Ordering::SeqCst);
                TrayState::update(AssistantState::Idle, "Dijeda");
                crate::term_log!(
                    "\n[DIJEDA] {} berhenti mendengarkan sementara.",
                    self.config.assistant_name
                );
                let mut last_pause_check = Instant::now();
                loop {
                    if last_pause_check.elapsed() >= std::time::Duration::from_millis(500) {
                        self.reload_config_if_changed();
                        last_pause_check = Instant::now();
                    }
                    if let Some(cmd) = check_and_consume_cmd() {
                        if cmd == "toggle_pause" || cmd == "start_chat" {
                            is_paused = false;
                            pause_worker.store(false, Ordering::SeqCst);
                            break;
                        } else if cmd == "stop_chat" || cmd == "quit" {
                            stop_worker.store(true, Ordering::SeqCst);
                            let _ = worker_handle.join();
                            crate::app_state::update_session_running(false);
                            TrayState::update(AssistantState::Idle, "Standby");
                            crate::term_log!("\n[SELESAI] Sesi obrolan dihentikan.");
                            return Ok(());
                        } else if cmd == "reset_session" || cmd == "reset_chat" {
                            self.llm.reset_history();
                            crate::term_log!(
                                "🔄 [Reset Session] Riwayat memori obrolan AI berhasil di-reset!"
                            );
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
                crate::term_log!(
                    "▶️  [DILANJUTKAN] {} siap mendengarkan lagi!\n",
                    self.config.assistant_name
                );
            }

            // Tetap di state Idle saat menunggu input suara
            TrayState::update(AssistantState::Idle, "Standby Siap");

            let mut should_exit = false;
            let mut should_pause = false;
            let mut audio_data = Vec::new();
            let mut direct_text: Option<String> = None;

            // Cek antrean audio dari continuous audio worker & direct text dari Mobile Web STT tanpa blocking
            let mut last_audio_check = Instant::now();
            loop {
                if last_audio_check.elapsed() >= std::time::Duration::from_millis(500) {
                    self.reload_config_if_changed();
                    last_audio_check = Instant::now();
                }

                if let Some(cmd) = check_and_consume_cmd() {
                    if cmd == "toggle_pause" {
                        should_pause = true;
                        break;
                    } else if cmd == "stop_chat" || cmd == "quit" {
                        should_exit = true;
                        break;
                    } else if cmd == "reset_session" || cmd == "reset_chat" {
                        self.llm.reset_history();
                        crate::term_log!(
                            "🔄 [Reset Session] Riwayat memori obrolan AI berhasil di-reset!"
                        );
                    }
                }

                if let Ok(incoming_text) = text_rx.try_recv() {
                    let trimmed = incoming_text.trim();
                    if !trimmed.is_empty() {
                        direct_text = Some(trimmed.to_string());
                        break;
                    }
                }

                if let Ok(ready_audio) = turn_rx.recv_timeout(std::time::Duration::from_millis(30))
                {
                    audio_data = ready_audio;
                    break;
                }
            }

            if should_exit {
                stop_worker.store(true, Ordering::SeqCst);
                let _ = worker_handle.join();
                crate::app_state::update_session_running(false);
                TrayState::update(AssistantState::Idle, "Standby");
                crate::term_log!("\n[SELESAI] Sesi obrolan dihentikan.");
                break;
            }

            if should_pause {
                is_paused = true;
                continue;
            }

            let (clean_raw, stt_elapsed) = if let Some(txt) = direct_text {
                // Input teks langsung dari Web Speech API smartphone (0 KB audio upload, < 50ms)
                // Bersihkan buffer recorder desktop agar mic PC tidak menangkap suara gema dari speaker/ruangan
                self.recorder.clear_buffer();
                let elapsed = std::time::Duration::from_millis(5);
                (txt, elapsed)
            } else {
                if audio_data.is_empty() || audio_data.len() < (TARGET_SAMPLE_RATE as usize * 4 / 10) {
                    // Audio < 0.4 detik lewati tanpa panggil API
                    continue;
                }

                // Hitung energi RMS keseluruhan audio
                let sum_sq: f32 = audio_data.iter().map(|&x| x * x).sum();
                let audio_rms = (sum_sq / audio_data.len() as f32).sqrt();

                // Jika energi audio sangat rendah (< 0.006), ini hanya desis / noise background
                if audio_rms < 0.006 {
                    TrayState::update(AssistantState::Idle, "Standby Siap");
                    continue;
                }

                TrayState::update(AssistantState::Thinking, "Transkripsi Audio...");
                crate::term_log!("⏹️  Selesai mendengar. Memproses...");

                // 1. Transkrip Whisper STT
                let stt_start = Instant::now();
                let raw_transcript = match self.stt.transcribe(&audio_data) {
                    Ok(t) => t,
                    Err(e) => {
                        crate::term_err!("⚠️ [STT Error]: {e}");
                        TrayState::update(AssistantState::Idle, "Standby Siap");
                        continue;
                    }
                };
                let stt_elapsed = stt_start.elapsed();
                (raw_transcript.trim().to_string(), stt_elapsed)
            };

            // Cek apakah ada request stop_chat / pause saat STT sedang berjalan
            if let Some(cmd) = check_and_consume_cmd() {
                if cmd == "stop_chat" || cmd == "quit" {
                    crate::app_state::update_session_running(false);
                    TrayState::update(AssistantState::Idle, "Menu Utama");
                    crate::term_log!("\n[SELESAI] Sesi obrolan dihentikan.");
                    return Ok(());
                } else if cmd == "toggle_pause" {
                    is_paused = true;
                    continue;
                }
            }

            if clean_raw.is_empty() {
                crate::term_log!("[INFO] (Suara tidak terdengar jelas / terlalu pelan)\n");
                TrayState::update(AssistantState::Idle, "Standby Siap");
                continue;
            }

            // Filter halusinasi hening bawaan dataset Whisper
            let lower = clean_raw.to_lowercase();
            let stripped_punctuation: String = lower
                .chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                .collect();
            let clean_trimmed = stripped_punctuation.trim();

            let is_hallucination = clean_trimmed == "terima kasih"
                || clean_trimmed == "makasih"
                || clean_trimmed == "terimakasih"
                || clean_trimmed == "terima kasih banyak"
                || clean_trimmed == "thank you"
                || clean_trimmed == "thanks"
                || clean_trimmed == "terima kasih telah menonton"
                || lower.contains("menonton")
                || lower.contains("subtitles by")
                || lower.contains("subscribe")
                || lower.contains("thank you for watching")
                || lower.contains("terima kasih kerana")
                || lower.contains("terima kasih sudah")
                || (lower.contains("terima kasih") && lower.matches("terima kasih").count() > 1);

            if is_hallucination {
                TrayState::update(AssistantState::Idle, "Standby Siap");
                continue;
            }

            let user_transcript = clean_raw.to_string();
            let effective_input = user_transcript.clone();

            crate::term_log!("\n[User] Kamu [{:.2?}]:", stt_elapsed);
            crate::term_log!("   \"{}\"", user_transcript);

            TrayState::emit_event(
                "assistant:transcript",
                serde_json::json!({
                    "role": "user",
                    "text": &user_transcript
                }),
            );

            // 2 & 3. Streaming LLM + Sentence-Level Streaming TTS
            crate::app_state::clear_interruption();
            if !self.config.voice_interruption {
                pause_worker.store(true, Ordering::SeqCst);
                self.recorder.clear_buffer();
            }

            TrayState::update(AssistantState::Thinking, "LLM Berpikir...");
            let llm_start = Instant::now();
            let asst_name = &self.config.assistant_name;
            crate::term_log!("\n[LLM] {} sedang merespons...", asst_name);

            let tts_clone = Arc::clone(&self.tts);
            let mut first_token_time = None;
            let mut printed_header = false;

            let result = self.llm.chat_stream(&effective_input, |sentence, emotion| {
                let clean_sentence = sentence.trim().to_string();

                if clean_sentence.is_empty() {
                    return Ok(());
                }

                if crate::app_state::is_interruption_requested() {
                    return Err(anyhow::anyhow!("Interrupted by user"));
                }

                // Update emosi avatar real-time ke renderer 3D dalam pose Thinking selagi audio disiapkan
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

                tts_clone.speak_with_emotion(&clean_sentence, emotion, asst_name)?;
                Ok(())
            });

            println!();

            let full_response_text = match &result {
                Ok(txt) => {
                    let clean = txt.trim();
                    if !clean.is_empty() {
                        crate::term_log!("[{}] \"{}\"", asst_name, clean);
                    }
                    txt.clone()
                }
                Err(e) => {
                    crate::term_err!("⚠️ [LLM Error]: {e}");
                    String::new()
                }
            };

            TrayState::emit_event(
                "assistant:response-done",
                serde_json::json!({
                    "text": &full_response_text
                }),
            );

            let was_interrupted = crate::app_state::is_interruption_requested();
            crate::app_state::clear_interruption();
            pause_worker.store(false, Ordering::SeqCst);

            if was_interrupted {
                crate::term_log!("⚡ [Interupsi Diterima] Memproses giliran bicara baru...");
                continue;
            }

            // Eksekusi skill jika LLM menyertakan tag [CMD:ID] atau [SKILL:ID]
            if let Ok(full_reply) = &result {
                static RE_SKILL: std::sync::LazyLock<regex::Regex> =
                    std::sync::LazyLock::new(|| {
                        regex::Regex::new(r"\[(?:SKILL|CMD):([^\]]+)\]").expect("valid skill regex")
                    });
                let mut found_skill = false;
                let mut follow_up_prompt = None;
                let mut executed_skill_ids = std::collections::HashSet::new();
                for cap in RE_SKILL.captures_iter(full_reply) {
                    if let Some(matched_id) = cap.get(1) {
                        let id = matched_id.as_str().trim();
                        // Cegah eksekusi ganda untuk skill ID yang sama dalam satu giliran respon
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

                            // Two-Step Agentic Loop: Jika aksi investigasi / fokus berhasil,
                            // kumpulkan observasi tombol/elemen dari jendela yang baru aktif untuk dieksekusi pada Step 2!
                            let id_lower = id.to_lowercase();
                            if id_lower.starts_with("desktop:focus")
                                || id_lower.starts_with("app:open")
                                || id_lower.starts_with("desktop:scan")
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
                            } else if id_lower.starts_with("desktop:scan_visual") {
                                if let Ok(snap) = crate::engine::window_scanner::ActiveWindowSnapshot::load_from_file() {
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
                                    follow_up_prompt = Some(format!(
                                        "[OBSERVASI SISTEM: Hasil pindai visual layar: {} tombol terdeteksi: {}. Lanjutkan perintah pengguna: '{}']",
                                        snap.buttons.len(),
                                        btns.join(", "),
                                        effective_input
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

                // Step 2: Content-Level Agentic Loop (Jika ada observasi baru dan user meminta tindakan lanjutan)
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
                        let tts_step2 = Arc::clone(&self.tts);
                        let asst = self.config.assistant_name.clone();

                        let step2_result = self.llm.chat_stream(&obs, |sentence, emotion| {
                            let clean = sentence.trim().to_string();
                            if clean.is_empty() {
                                return Ok(());
                            }
                            print!("{} ", clean);
                            use std::io::Write;
                            let _ = std::io::stdout().flush();
                            tts_step2.speak_with_emotion(&clean, emotion, &asst)?;
                            Ok(())
                        });

                        println!();

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
                        crate::term_log!(
                            "\n[SMART FALLBACK]: Terdeteksi permintaan buka terminal."
                        );
                        if let Some((skill_name, output)) =
                            execute_skill_by_identifier("buka terminal")
                        {
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

            crate::term_log!("----------------------------------------------------\n");

            // Buang semua sisa buffer / pantulan audio speaker sebelum mic diaktifkan kembali
            self.recorder.clear_buffer();
            while turn_rx.try_recv().is_ok() {}
            std::thread::sleep(std::time::Duration::from_millis(150));
            self.recorder.clear_buffer();
            while turn_rx.try_recv().is_ok() {}
            pause_worker.store(false, Ordering::SeqCst);

            if let Err(err) = result {
                crate::term_err!("[ERROR] Jaringan terganggu: {err}\n");
            }

            std::thread::sleep(std::time::Duration::from_millis(300));
            self.recorder.clear_buffer();

            crate::term_log!("----------------------------------------------------\n");
        }

        Ok(())
    }
}
