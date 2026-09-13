use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crate::dsp::TARGET_SAMPLE_RATE;
use crate::tray::{AssistantState, TrayState};
use super::executor::execute_skill_by_id;
use super::session::{check_and_consume_cmd, AppSession};

impl AppSession {
    pub fn start_chat_loop(&mut self) -> Result<()> {
        let is_auto = self.config.is_auto_mode();

        crate::term_log!("=== SESI PERCAKAPAN DIMULAI ===");
        crate::term_log!("   - Mode: {}", if is_auto { "Hands-Free (Bicara langsung ke mic)" } else { "Manual (Spasi)" });
        crate::term_log!("   - Tekan [P] untuk PAUSE / RESUME {} (Berhenti mendengarkan sementara)", self.config.assistant_name);
        crate::term_log!("   - Tekan [SPASI] saat {} bicara untuk MEMOTONG", self.config.assistant_name);
        crate::term_log!("   - Tekan [ESC] atau [Q] untuk KELUAR / KEMBALI KE MENU");
        crate::term_log!("----------------------------------------------------\n");

        let mut is_paused = false;

        let (turn_tx, turn_rx) = std::sync::mpsc::channel::<Vec<f32>>();
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
                    TrayState::update(AssistantState::Idle, "Menu Utama");
                    crate::term_log!("\n[SELESAI] Sesi obrolan dihentikan.");
                    return Ok(());
                } else if cmd == "toggle_pause" {
                    is_paused = true;
                }
            }

            // Reload konfigurasi secara reactive sebelum mulai giliran mendengar berikutnya
            self.reload_config_if_changed();

            // Sesi Ngobrol Aktif
            if is_paused {
                pause_worker.store(true, Ordering::SeqCst);
                TrayState::update(AssistantState::Idle, "Dijeda");
                crate::term_log!("\n[DIJEDA] {} berhenti mendengarkan sementara.", self.config.assistant_name);
                crate::term_log!("Tekan [P] atau [SPASI] untuk MELANJUTKAN, atau [ESC/Q] untuk KELUAR.\n");
                let _ = enable_raw_mode();
                loop {
                    if let Some(cmd) = check_and_consume_cmd() {
                        if cmd == "toggle_pause" || cmd == "start_chat" {
                            is_paused = false;
                            pause_worker.store(false, Ordering::SeqCst);
                            break;
                        } else if cmd == "stop_chat" || cmd == "quit" {
                            let _ = disable_raw_mode();
                            stop_worker.store(true, Ordering::SeqCst);
                            let _ = worker_handle.join();
                            TrayState::update(AssistantState::Idle, "Menu Utama");
                            crate::term_log!("\n[SELESAI] Sesi obrolan dihentikan.");
                            return Ok(());
                        }
                    }
                    if let Ok(true) = event::poll(std::time::Duration::from_millis(50)) {
                        if let Ok(Event::Key(key_event)) = event::read() {
                            if key_event.kind == KeyEventKind::Press {
                                match key_event.code {
                                    KeyCode::Char('p') | KeyCode::Char('P') | KeyCode::Char(' ') => {
                                        is_paused = false;
                                        pause_worker.store(false, Ordering::SeqCst);
                                        break;
                                    }
                                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                                        let _ = disable_raw_mode();
                                        stop_worker.store(true, Ordering::SeqCst);
                                        let _ = worker_handle.join();
                                        TrayState::update(AssistantState::Idle, "Menu Utama");
                                        crate::term_log!("\n[SELESAI] Sesi obrolan dihentikan.");
                                        return Ok(());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    } else {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                }
                let _ = disable_raw_mode();
                crate::term_log!("▶️  [DILANJUTKAN] {} siap mendengarkan lagi!\n", self.config.assistant_name);
            }

            TrayState::update(AssistantState::Idle, "Siap Mendengarkan");
            if is_auto {
                crate::term_log!("Siap mendengarkan (Realtime)... [P: Pause | Q/ESC: Keluar]:");
            } else {
                crate::term_log!("Siap mendengarkan. Tekan [SPASI] untuk bicara [P: Pause | Q/ESC: Keluar]...");
            }

            let _ = enable_raw_mode();

            let mut should_exit = false;
            let mut should_pause = false;
            let mut audio_data = Vec::new();

            loop {
                if let Some(cmd) = check_and_consume_cmd() {
                    if cmd == "toggle_pause" {
                        should_pause = true;
                        break;
                    } else if cmd == "stop_chat" || cmd == "quit" {
                        should_exit = true;
                        break;
                    }
                }

                if let Ok(true) = event::poll(std::time::Duration::from_millis(30)) {
                    if let Ok(Event::Key(key_event)) = event::read() {
                        if key_event.kind == KeyEventKind::Press {
                            match key_event.code {
                                KeyCode::Char('p') | KeyCode::Char('P') => {
                                    should_pause = true;
                                    break;
                                }
                                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                                    should_exit = true;
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }

                // Cek antrean audio dari continuous audio worker tanpa blocking
                if let Ok(ready_audio) = turn_rx.try_recv() {
                    audio_data = ready_audio;
                    break;
                }
            }

            let _ = disable_raw_mode();
            if crate::config::is_terminal_log_enabled() {
                print!("\r\x1b[K");
            }

            if should_exit {
                stop_worker.store(true, Ordering::SeqCst);
                let _ = worker_handle.join();
                TrayState::update(AssistantState::Idle, "Menu Utama");
                crate::term_log!("\n[SELESAI] Sesi obrolan dihentikan.");
                break;
            }

            if should_pause {
                is_paused = true;
                continue;
            }

            if audio_data.is_empty() || audio_data.len() < (TARGET_SAMPLE_RATE as usize * 4 / 10) {
                // Audio < 0.4 detik lewati tanpa panggil API
                continue;
            }

            // Hitung energi RMS keseluruhan audio
            let sum_sq: f32 = audio_data.iter().map(|&x| x * x).sum();
            let audio_rms = (sum_sq / audio_data.len() as f32).sqrt();

            // Jika energi audio sangat rendah (< 0.006), ini hanya desis / noise background
            // Langsung lewati agar TIDAK buang-buang kuota atau kena rate limit Google/Groq STT
            if audio_rms < 0.006 {
                TrayState::update(AssistantState::Idle, "Siap Mendengarkan");
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
                    TrayState::update(AssistantState::Idle, "Siap Mendengarkan");
                    continue;
                }
            };
            let stt_elapsed = stt_start.elapsed();

            let clean_raw = raw_transcript.trim();

            // Cek apakah ada request stop_chat / pause saat STT sedang berjalan
            if let Some(cmd) = check_and_consume_cmd() {
                if cmd == "stop_chat" || cmd == "quit" {
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
                TrayState::update(AssistantState::Idle, "Siap Mendengarkan");
                continue;
            }

            // Filter halusinasi hening bawaan dataset Whisper
            let lower = clean_raw.to_lowercase();
            let stripped_punctuation: String = lower.chars().filter(|c| c.is_alphanumeric() || c.is_whitespace()).collect();
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
                TrayState::update(AssistantState::Idle, "Siap Mendengarkan");
                continue;
            }

            let user_transcript = clean_raw.to_string();
            let effective_input = user_transcript.clone();

            crate::term_log!("\n[User] Kamu [{:.2?}]:", stt_elapsed);
            crate::term_log!("   \"{}\"", user_transcript);

            TrayState::emit_event("assistant:transcript", serde_json::json!({
                "role": "user",
                "text": &user_transcript
            }));

            // 2 & 3. Streaming LLM + Sentence-Level Streaming TTS (Interupsi Instan via Spasi / Q / GUI)
            // Pause background audio worker saat asisten sedang berbicara agar suaranya sendiri tidak masuk ke mic (Echo Loop)
            pause_worker.store(true, Ordering::SeqCst);
            self.recorder.clear_buffer();

            TrayState::update(AssistantState::Thinking, "LLM Berpikir...");
            let llm_start = Instant::now();
            let asst_name = &self.config.assistant_name;
            crate::term_log!("\n[LLM] {} sedang merespons... (Tekan [SPASI] untuk memotong)", asst_name);

            let is_interrupted = Arc::new(AtomicBool::new(false));
            let is_interrupted_clone = Arc::clone(&is_interrupted);
            let tts_clone = Arc::clone(&self.tts);

            let is_user_cut = Arc::new(AtomicBool::new(false));
            let is_user_cut_clone = Arc::clone(&is_user_cut);
            let stop_thread_flag = Arc::new(AtomicBool::new(false));
            let stop_thread_flag_clone = Arc::clone(&stop_thread_flag);
            let interrupt_speaker = Arc::clone(&is_interrupted);

            let _ = enable_raw_mode();
            while let Ok(true) = event::poll(std::time::Duration::from_millis(0)) {
                let _ = event::read();
            }

            let keyboard_listener = std::thread::spawn(move || -> bool {
                let start_time = Instant::now();
                let mut requested_exit = false;
                loop {
                    if stop_thread_flag_clone.load(Ordering::SeqCst) {
                        break;
                    }
                    if let Some(cmd) = check_and_consume_cmd() {
                        if cmd == "toggle_pause" {
                            interrupt_speaker.store(true, Ordering::SeqCst);
                            is_user_cut_clone.store(true, Ordering::SeqCst);
                            break;
                        } else if cmd == "stop_chat" || cmd == "quit" {
                            interrupt_speaker.store(true, Ordering::SeqCst);
                            is_user_cut_clone.store(true, Ordering::SeqCst);
                            requested_exit = true;
                            break;
                        }
                    }
                    if let Ok(true) = event::poll(std::time::Duration::from_millis(15)) {
                        if let Ok(Event::Key(key_event)) = event::read() {
                            if key_event.kind == KeyEventKind::Press && start_time.elapsed().as_millis() > 50 {
                                match key_event.code {
                                    KeyCode::Char(' ') => {
                                        interrupt_speaker.store(true, Ordering::SeqCst);
                                        is_user_cut_clone.store(true, Ordering::SeqCst);
                                        break;
                                    }
                                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                                        interrupt_speaker.store(true, Ordering::SeqCst);
                                        is_user_cut_clone.store(true, Ordering::SeqCst);
                                        requested_exit = true;
                                        break;
                                    }
                                    _ => {}
                                }
                            }
                        }
                    } else {
                        std::thread::sleep(std::time::Duration::from_millis(15));
                    }
                }
                requested_exit
            });

            let mut exit_all = false;
            let mut first_token_time = None;
            let mut printed_header = false;

            let result = self.llm.chat_stream(&effective_input, |sentence| {
                if is_interrupted_clone.load(Ordering::SeqCst) {
                    anyhow::bail!("Interrupted by user");
                }

                let clean_sentence = sentence.trim().to_string();

                if clean_sentence.is_empty() {
                    return Ok(());
                }

                if first_token_time.is_none() {
                    let lower_s = clean_sentence.to_lowercase();
                    let emotion = if lower_s.contains("*malu*") || lower_s.contains("*blush*") || lower_s.contains("*tersipu*") {
                        "shy"
                    } else if lower_s.contains("*senang*") || lower_s.contains("*gembira*") {
                        "happy"
                    } else if lower_s.contains("*semangat*") || lower_s.contains("*excited*") {
                        "excited"
                    } else if lower_s.contains("*kaget*") || lower_s.contains("*terkejut*") {
                        "surprised"
                    } else if lower_s.contains("*bingung*") || lower_s.contains("*confused*") {
                        "confused"
                    } else if lower_s.contains("*sedih*") || lower_s.contains("*sad*") {
                        "sad"
                    } else if lower_s.contains("*thinking*") || lower_s.contains("*mikir*") {
                        "thinking"
                    } else if lower_s.contains("tersipu") || lower_s.contains("malu") || lower_s.contains("aduh") {
                        "shy"
                    } else if lower_s.contains("bingung") || lower_s.contains("kurang paham") || lower_s.contains("hmm") {
                        "confused"
                    } else if lower_s.contains("senang") || lower_s.contains("suka") || lower_s.contains("alhamdulillah") || lower_s.contains("hehe") {
                        "happy"
                    } else if lower_s.contains("wah") || lower_s.contains("siap") || lower_s.contains("tentu") || lower_s.contains("hebat") || lower_s.contains("dong") {
                        "excited"
                    } else if lower_s.contains("kaget") || lower_s.contains("astaga") || lower_s.contains("masa") || lower_s.contains("beneran") {
                        "surprised"
                    } else if lower_s.contains("sedih") || lower_s.contains("kecewa") || lower_s.contains("maaf") {
                        "sad"
                    } else {
                        "happy"
                    };

                    TrayState::update_with_emotion(AssistantState::Speaking, &format!("{asst_name} Berbicara"), emotion);
                    let first_elapsed = llm_start.elapsed();
                    first_token_time = Some(first_elapsed);
                    if !printed_header {
                        if crate::config::is_terminal_log_enabled() {
                            print!("\r\n[{}] [{:.2?}]:\r\n   ", asst_name, first_elapsed);
                        }
                        printed_header = true;
                    }
                }

                TrayState::emit_event("assistant:stream-chunk", serde_json::json!({
                    "sentence": &clean_sentence,
                    "chunk": &clean_sentence,
                    "speaker": asst_name
                }));

                if crate::config::is_terminal_log_enabled() {
                    print!("{} ", clean_sentence);
                    use std::io::Write;
                    let _ = std::io::stdout().flush();
                }

                tts_clone.speak_interruptible(&clean_sentence, Arc::clone(&is_interrupted_clone))?;
                if is_interrupted_clone.load(Ordering::SeqCst) {
                    anyhow::bail!("Interrupted by user");
                }
                Ok(())
            });

            stop_thread_flag.store(true, Ordering::SeqCst);
            if let Ok(was_exit) = keyboard_listener.join() {
                if was_exit {
                    exit_all = true;
                }
            }
            let _ = disable_raw_mode();
            if crate::config::is_terminal_log_enabled() {
                print!("\r\n");
            }

            if is_user_cut.load(Ordering::SeqCst) && !exit_all {
                crate::term_log!("[DIPOTONG] {} berhenti bicara.", asst_name);
            }

            let full_response_text = match &result {
                Ok(txt) => txt.clone(),
                Err(_) => String::new(),
            };

            TrayState::emit_event("assistant:response-done", serde_json::json!({
                "text": &full_response_text,
                "interrupted": is_user_cut.load(Ordering::SeqCst)
            }));

            // Eksekusi skill jika LLM menyertakan tag [SKILL:ID]
            if let Ok(full_reply) = &result {
                static RE_SKILL: std::sync::LazyLock<regex::Regex> =
                    std::sync::LazyLock::new(|| regex::Regex::new(r"\[SKILL:([^\]]+)\]").expect("valid skill regex"));
                let mut found_skill = false;
                for cap in RE_SKILL.captures_iter(full_reply) {
                    if let Some(matched_id) = cap.get(1) {
                        found_skill = true;
                        let id = matched_id.as_str().trim();
                        crate::term_log!("\n[MEMPROSES SKILL]: Memanggil ID '{}'...", id);
                        if let Some((skill_name, output)) = execute_skill_by_id(id) {
                            crate::term_log!("[SKILL BERHASIL: {} ({})] -> {}", skill_name, id, output.trim());
                            TrayState::emit_event("assistant:skill", serde_json::json!({
                                "id": id,
                                "name": skill_name,
                                "status": "success",
                                "output": output.trim()
                            }));
                        } else {
                            crate::term_err!("[SKILL GAGAL]: ID '{}' tidak cocok di skill-command.md", id);
                            TrayState::emit_event("assistant:skill", serde_json::json!({
                                "id": id,
                                "name": id,
                                "status": "failed",
                                "output": "ID skill tidak ditemukan"
                            }));
                        }
                    }
                }
                if !found_skill {
                    let lower_in = effective_input.to_lowercase();
                    if lower_in.contains("buka terminal") || lower_in.contains("open terminal") || lower_in.contains("bukain terminal") || lower_in.contains("buka console") {
                        crate::term_log!("\n[SMART FALLBACK]: Terdeteksi permintaan buka terminal.");
                        if let Some((skill_name, output)) = execute_skill_by_id("SYS-000") {
                            crate::term_log!("[SKILL BERHASIL: {}] -> {}", skill_name, output.trim());
                            TrayState::emit_event("assistant:skill", serde_json::json!({
                                "id": "SYS-000",
                                "name": skill_name,
                                "status": "success",
                                "output": output.trim()
                            }));
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

            if exit_all {
                TrayState::update(AssistantState::Idle, "Menu Utama");
                crate::term_log!("\n[SELESAI] Sesi obrolan dihentikan.");
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(300));
            self.recorder.clear_buffer();

            crate::term_log!("----------------------------------------------------\n");
        }

        Ok(())
    }
}
