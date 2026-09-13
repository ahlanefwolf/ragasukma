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

        println!("=== SESI PERCAKAPAN DIMULAI ===");
        println!("   - Mode: {}", if is_auto { "Hands-Free (Bicara langsung ke mic)" } else { "Manual (Spasi)" });
        println!("   - Tekan [P] untuk PAUSE / RESUME {} (Berhenti mendengarkan sementara)", self.config.assistant_name);
        println!("   - Tekan [SPASI] saat {} bicara untuk MEMOTONG", self.config.assistant_name);
        println!("   - Tekan [ESC] atau [Q] untuk KELUAR / KEMBALI KE MENU");
        println!("----------------------------------------------------\n");

        let mut is_paused = false;

        loop {
            // Check if a command was sent from GUI or tray before starting the turn
            if let Some(cmd) = check_and_consume_cmd() {
                if cmd == "stop_chat" || cmd == "quit" {
                    TrayState::update(AssistantState::Idle, "Menu Utama");
                    println!("\n[SELESAI] Sesi obrolan dihentikan.");
                    return Ok(());
                } else if cmd == "toggle_pause" {
                    is_paused = true;
                }
            }

            // Reload konfigurasi secara reactive sebelum mulai giliran mendengar berikutnya
            self.reload_config_if_changed();

            // Sesi Ngobrol Aktif
            if is_paused {
                TrayState::update(AssistantState::Idle, "Dijeda");
                println!("\n[DIJEDA] {} berhenti mendengarkan sementara.", self.config.assistant_name);
                println!("Tekan [P] atau [SPASI] untuk MELANJUTKAN, atau [ESC/Q] untuk KELUAR.\n");
                enable_raw_mode()?;
                loop {
                    if let Some(cmd) = check_and_consume_cmd() {
                        if cmd == "toggle_pause" || cmd == "start_chat" {
                            is_paused = false;
                            break;
                        } else if cmd == "stop_chat" || cmd == "quit" {
                            disable_raw_mode()?;
                            TrayState::update(AssistantState::Idle, "Menu Utama");
                            println!("\n[SELESAI] Sesi obrolan dihentikan.");
                            return Ok(());
                        }
                    }
                    if event::poll(std::time::Duration::from_millis(50))? {
                        if let Event::Key(key_event) = event::read()? {
                            if key_event.kind == KeyEventKind::Press {
                                match key_event.code {
                                    KeyCode::Char('p') | KeyCode::Char('P') | KeyCode::Char(' ') => {
                                        is_paused = false;
                                        break;
                                    }
                                    KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                                        disable_raw_mode()?;
                                        TrayState::update(AssistantState::Idle, "Menu Utama");
                                        println!("\n[SELESAI] Sesi obrolan dihentikan.");
                                        return Ok(());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                disable_raw_mode()?;
                println!("▶️  [DILANJUTKAN] {} siap mendengarkan lagi!\n", self.config.assistant_name);
            }

            TrayState::update(AssistantState::Idle, "Siap Mendengarkan");
            if is_auto {
                println!("Siap mendengarkan (Realtime)... [P: Pause | Q/ESC: Keluar]:");
            } else {
                println!("Siap mendengarkan. Tekan [SPASI] untuk bicara [P: Pause | Q/ESC: Keluar]...");
            }

            enable_raw_mode()?;

            let mut should_exit = false;
            let mut should_pause = false;
            let stop_recording = Arc::new(AtomicBool::new(false));
            let stop_recording_clone = Arc::clone(&stop_recording);
            let recorder_ref = Arc::clone(&self.recorder);

            let record_handle = std::thread::spawn(move || {
                recorder_ref.record_turn(stop_recording_clone)
            });

            loop {
                if let Some(cmd) = check_and_consume_cmd() {
                    if cmd == "toggle_pause" {
                        should_pause = true;
                        stop_recording.store(true, Ordering::SeqCst);
                        break;
                    } else if cmd == "stop_chat" || cmd == "quit" {
                        should_exit = true;
                        stop_recording.store(true, Ordering::SeqCst);
                        break;
                    }
                }

                if event::poll(std::time::Duration::from_millis(30))? {
                    if let Event::Key(key_event) = event::read()? {
                        if key_event.kind == KeyEventKind::Press {
                            match key_event.code {
                                KeyCode::Char('p') | KeyCode::Char('P') => {
                                    should_pause = true;
                                    stop_recording.store(true, Ordering::SeqCst);
                                    break;
                                }
                                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                                    should_exit = true;
                                    stop_recording.store(true, Ordering::SeqCst);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }

                if record_handle.is_finished() {
                    break;
                }
            }

            let audio_data = record_handle
                .join()
                .map_err(|_| anyhow::anyhow!("Recording thread panicked"))??;

            disable_raw_mode()?;
            print!("\r\x1b[K");

            if should_exit {
                TrayState::update(AssistantState::Idle, "Menu Utama");
                println!("\n[SELESAI] Sesi obrolan dihentikan.");
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
            println!("⏹️  Selesai mendengar. Memproses...");

            // 1. Transkrip Whisper STT
            let stt_start = Instant::now();
            let raw_transcript = self.stt.transcribe(&audio_data)?;
            let stt_elapsed = stt_start.elapsed();

            let clean_raw = raw_transcript.trim();

            // Cek apakah ada request stop_chat / pause saat STT sedang berjalan
            if let Some(cmd) = check_and_consume_cmd() {
                if cmd == "stop_chat" || cmd == "quit" {
                    TrayState::update(AssistantState::Idle, "Menu Utama");
                    println!("\n[SELESAI] Sesi obrolan dihentikan.");
                    return Ok(());
                } else if cmd == "toggle_pause" {
                    is_paused = true;
                    continue;
                }
            }

            if clean_raw.is_empty() {
                println!("[INFO] (Suara tidak terdengar jelas / terlalu pelan)\n");
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

            println!("\n[User] Kamu [{:.2?}]:", stt_elapsed);
            println!("   \"{}\"", user_transcript);

            TrayState::emit_event("assistant:transcript", serde_json::json!({
                "role": "user",
                "text": &user_transcript
            }));

            // 2 & 3. Streaming LLM + Sentence-Level Streaming TTS (Interupsi Instan via Spasi / Q / GUI)
            TrayState::update(AssistantState::Thinking, "LLM Berpikir...");
            let llm_start = Instant::now();
            let asst_name = &self.config.assistant_name;
            println!("\n[LLM] {} sedang merespons... (Tekan [SPASI] untuk memotong)", asst_name);

            let is_interrupted = Arc::new(AtomicBool::new(false));
            let is_interrupted_clone = Arc::clone(&is_interrupted);
            let tts_clone = Arc::clone(&self.tts);

            let is_user_cut = Arc::new(AtomicBool::new(false));
            let is_user_cut_clone = Arc::clone(&is_user_cut);
            let stop_thread_flag = Arc::new(AtomicBool::new(false));
            let stop_thread_flag_clone = Arc::clone(&stop_thread_flag);
            let interrupt_speaker = Arc::clone(&is_interrupted);

            enable_raw_mode()?;
            while event::poll(std::time::Duration::from_millis(0))? {
                let _ = event::read()?;
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
                        print!("\r\n[{}] [{:.2?}]:\r\n   ", asst_name, first_elapsed);
                        printed_header = true;
                    }
                }

                TrayState::emit_event("assistant:stream-chunk", serde_json::json!({
                    "sentence": &clean_sentence,
                    "chunk": &clean_sentence,
                    "speaker": asst_name
                }));

                print!("{} ", clean_sentence);
                use std::io::Write;
                let _ = std::io::stdout().flush();

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
            disable_raw_mode()?;
            print!("\r\n");

            if is_user_cut.load(Ordering::SeqCst) && !exit_all {
                println!("[DIPOTONG] {} berhenti bicara.", asst_name);
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
                        println!("\n[MEMPROSES SKILL]: Memanggil ID '{}'...", id);
                        if let Some((skill_name, output)) = execute_skill_by_id(id) {
                            println!("[SKILL BERHASIL: {} ({})] -> {}", skill_name, id, output.trim());
                            TrayState::emit_event("assistant:skill", serde_json::json!({
                                "id": id,
                                "name": skill_name,
                                "status": "success",
                                "output": output.trim()
                            }));
                        } else {
                            eprintln!("[SKILL GAGAL]: ID '{}' tidak cocok di skill-command.md", id);
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
                        println!("\n[SMART FALLBACK]: Terdeteksi permintaan buka terminal.");
                        if let Some((skill_name, output)) = execute_skill_by_id("SYS-000") {
                            println!("[SKILL BERHASIL: {}] -> {}", skill_name, output.trim());
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

            if let Err(err) = result {
                eprintln!("[ERROR] Jaringan terganggu: {err}\n");
            }

            if exit_all {
                TrayState::update(AssistantState::Idle, "Menu Utama");
                println!("\n[SELESAI] Sesi obrolan dihentikan.");
                break;
            }

            std::thread::sleep(std::time::Duration::from_millis(300));
            self.recorder.clear_buffer();

            println!("----------------------------------------------------\n");
        }

        Ok(())
    }
}
