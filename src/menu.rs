use anyhow::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use rodio::{buffer::SamplesBuffer, OutputStream, Sink};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;

use crate::audio::AudioRecorder;
use crate::config::AppConfig;
use crate::dsp::TARGET_SAMPLE_RATE;
use crate::llm::LlmClient;
use crate::stt::SttEngine;
use crate::tray::{AssistantState, TrayState};
use crate::tts::TextToSpeech;

pub fn execute_command_by_os(command: &str) -> std::io::Result<std::process::Child> {
    #[cfg(target_os = "windows")]
    {
        // Windows: Jalankan via cmd.exe /C start "" <command>
        let win_cmd = format!("start \"\" {command}");
        std::process::Command::new("cmd")
            .args(["/C", &win_cmd])
            .spawn()
    }

    #[cfg(target_os = "macos")]
    {
        // macOS: Gunakan nohup via sh
        let spawn_cmd = format!("nohup {command} >/dev/null 2>&1 &");
        std::process::Command::new("sh")
            .arg("-c")
            .arg(&spawn_cmd)
            .spawn()
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        // Linux / Unix: Gunakan nohup via sh
        let spawn_cmd = format!("nohup {command} >/dev/null 2>&1 &");
        std::process::Command::new("sh")
            .arg("-c")
            .arg(&spawn_cmd)
            .spawn()
    }
}

pub fn execute_skill_by_id(id: &str) -> Option<(String, String)> {
    let path = std::path::Path::new("skill-command.md");
    if !path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(path).ok()?;
    for line in content.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || trimmed.contains(":---") || trimmed.contains("Nama Skill") {
            continue;
        }
        let parts: Vec<&str> = trimmed.split('|').map(|s| s.trim()).collect();
        // Kolom markdown: | (0) | ID (1) | Name (2) | Desc (3) | Cmd (4) | Cat (5) |
        if parts.len() >= 5 {
            let skill_id = parts[1].replace('`', "");
            let skill_name = parts[2].to_string();
            let command = parts[4].replace('`', "");
            if skill_id.eq_ignore_ascii_case(id) && !command.is_empty() {
                let spawn_res = execute_command_by_os(&command);

                match spawn_res {
                    Ok(_) => {
                        return Some((skill_name, "Berhasil dijalankan".to_string()));
                    }
                    Err(e) => {
                        return Some((skill_name, format!("Error: {e}")));
                    }
                }
            }
        }
    }
    None
}

pub struct AppSession {
    config: AppConfig,
    recorder: Arc<AudioRecorder>,
    stt: SttEngine,
    llm: LlmClient,
    tts: Arc<TextToSpeech>,
}

fn check_and_consume_cmd() -> Option<String> {
    crate::tray::TrayState::pop_command()
}

impl AppSession {
    pub fn new(config: AppConfig) -> Result<Self> {
        let recorder = Arc::new(AudioRecorder::new(config.clone())?);
        println!("🎤 Input Device : {}", recorder.device_name());

        let stt = SttEngine::new(&config.stt_engine)?;

        let llm = LlmClient::new();
        println!("🎀 Karakter {} (Imut & Pemalu) siap menemani {}.", config.assistant_name, config.user_callsign);

        let tts = Arc::new(TextToSpeech::new());
        println!(
            "🔊 Engine TTS: {} siap.",
            config.tts_engine.to_uppercase()
        );

        Ok(Self {
            config,
            recorder,
            stt,
            llm,
            tts,
        })
    }

    pub fn reload_config_if_changed(&mut self) {
        let latest_config = AppConfig::load();

        // Cek apakah konfigurasi audio / STT / TTS berubah
        let stt_changed = latest_config.stt_engine != self.config.stt_engine
            || latest_config.groq_api_key != self.config.groq_api_key;

        let tts_changed = latest_config.tts_engine != self.config.tts_engine
            || (latest_config.tts_speed - self.config.tts_speed).abs() > 0.01
            || latest_config.tts_eleven_labs_key != self.config.tts_eleven_labs_key
            || latest_config.elevenlabs_voice_id != self.config.elevenlabs_voice_id;

        let vad_changed = (latest_config.vad_activation_threshold - self.config.vad_activation_threshold).abs() > 0.0001
            || latest_config.vad_silence_limit_chunks != self.config.vad_silence_limit_chunks
            || latest_config.noise_reduction_level != self.config.noise_reduction_level;

        let llm_changed = latest_config.llm_base_url != self.config.llm_base_url
            || latest_config.llm_model != self.config.llm_model
            || latest_config.llm_api_key != self.config.llm_api_key
            || latest_config.assistant_name != self.config.assistant_name
            || latest_config.user_callsign != self.config.user_callsign;

        if vad_changed {
            if let Ok(new_recorder) = AudioRecorder::new(latest_config.clone()) {
                self.recorder = Arc::new(new_recorder);
                println!("🔄 [Reactive Update] VAD Threshold & Recorder diupdate realtime!");
            }
        }

        if stt_changed {
            if let Ok(new_stt) = SttEngine::new(&latest_config.stt_engine) {
                self.stt = new_stt;
                println!("🔄 [Reactive Update] STT Engine dialihkan ke: {}", latest_config.stt_engine.to_uppercase());
            }
        }

        if tts_changed {
            self.tts = Arc::new(TextToSpeech::new());
            println!("🔄 [Reactive Update] TTS Engine dialihkan ke: {}", latest_config.tts_engine.to_uppercase());
        }

        if llm_changed {
            self.llm = LlmClient::new();
            println!("🔄 [Reactive Update] LLM Persona / Model dialihkan ke: {} ({})", latest_config.llm_model, latest_config.assistant_name);
        }

        self.config = latest_config;
    }

    pub fn run_interactive_menu(&mut self) -> Result<()> {
        // Langsung mulai mode ngobrol / active loop saat dijalankan
        self.start_chat_loop()?;

        // Jika keluar dari sesi chat tapi program belum quit, sediakan menu standby
        TrayState::update(AssistantState::Idle, "Menu Utama");

        println!("----------------------------------------------------");
        println!("💡 MENU UTAMA:");
        println!("   - Tekan [SPASI] / [ENTER] untuk MULAI NGOBROL");
        println!("   - Tekan [T] untuk TES SUARA {}", self.config.assistant_name.to_uppercase());
        println!("   - Tekan [M] untuk TES MIC & FILTER SUARA");
        println!("   - Tekan [ESC] atau [Q] untuk KELUAR");
        println!("====================================================\n");

        loop {
            println!("👉 Pilih menu: [SPASI/ENTER] Mulai Ngobrol | [T] Tes {} | [M] Tes Mic | [Q] Keluar", self.config.assistant_name);
            enable_raw_mode()?;

            let action;
            loop {
                if let Some(cmd) = check_and_consume_cmd() {
                    match cmd.as_str() {
                        "start_chat" | "start" | "toggle_pause" => {
                            action = 1;
                            break;
                        }
                        "test_voice" => {
                            action = 2;
                            break;
                        }
                        "test_mic" => {
                            action = 3;
                            break;
                        }
                        "quit" => {
                            action = 4;
                            break;
                        }
                        _ => {}
                    }
                }

                if event::poll(std::time::Duration::from_millis(50))? {
                    if let Event::Key(key_event) = event::read()? {
                        if key_event.kind == KeyEventKind::Press {
                            match key_event.code {
                                KeyCode::Char(' ') | KeyCode::Enter => {
                                    action = 1;
                                    break;
                                }
                                KeyCode::Char('t') | KeyCode::Char('T') => {
                                    action = 2;
                                    break;
                                }
                                KeyCode::Char('m') | KeyCode::Char('M') => {
                                    action = 3;
                                    break;
                                }
                                KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => {
                                    action = 4;
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            disable_raw_mode()?;

            match action {
                2 => {
                    self.test_assistant_voice()?;
                    continue;
                }
                3 => {
                    self.test_mic_dsp()?;
                    continue;
                }
                4 => {
                    TrayState::clear();
                    println!("\n👋 Program selesai. Sampai jumpa!");
                    return Ok(());
                }
                _ => {
                    self.start_chat_loop()?;
                }
            }
        }
    }

    fn test_assistant_voice(&self) -> Result<()> {
        let name = &self.config.assistant_name;
        println!("\n🔊 [TES SUARA {}] (Tekan ESC / SPASI kapan saja untuk kembali ke menu)", name.to_uppercase());
        let callsign = &self.config.user_callsign;
        let test_phrase = format!("Halo {callsign}, suara {name} sudah siap menemani {callsign} hari ini!");
        println!("🎀 {name}: \"{}\"", test_phrase);
        let test_interrupt = Arc::new(AtomicBool::new(false));
        self.tts.speak_interruptible(&test_phrase, test_interrupt)?;
        println!("----------------------------------------------------\n");
        Ok(())
    }

    fn test_mic_dsp(&self) -> Result<()> {
        println!("\n🎙️  [TES MIC & FILTER SUARA]");
        println!("   Silakan bicara selama 3 detik ke mic... (Tes noise reduction & normalisasi)");
        let is_recording_test = Arc::new(AtomicBool::new(true));
        let is_recording_test_clone = Arc::clone(&is_recording_test);
        let recorder_test = Arc::clone(&self.recorder);

        let record_handle = std::thread::spawn(move || {
            recorder_test.record_turn(is_recording_test_clone)
        });

        std::thread::sleep(std::time::Duration::from_millis(3200));
        is_recording_test.store(false, Ordering::SeqCst);

        let audio_samples = record_handle
            .join()
            .map_err(|_| anyhow::anyhow!("Test thread panicked"))??;

        if audio_samples.is_empty() {
            println!("⚠️  Tidak ada sampel suara yang terekam.");
        } else {
            println!("▶️  Memutar balik suara kamu setelah melalui DSP Noise Reduction Filter...");
            let (_stream, stream_handle) = OutputStream::try_default()?;
            let sink = Sink::try_new(&stream_handle)?;
            let source = SamplesBuffer::new(1, TARGET_SAMPLE_RATE, audio_samples);
            sink.append(source);
            sink.sleep_until_end();
            println!("✅ Selesai memutar balik hasil filter suara.");
        }
        println!("----------------------------------------------------\n");
        Ok(())
    }

    fn start_chat_loop(&mut self) -> Result<()> {
        let is_auto = self.config.is_auto_mode();

        println!("\n🟢 SESI PERCAKAPAN DIMULAI!");
        println!("   - Mode: {}", if is_auto { "Hands-Free (Bicara langsung ke mic)" } else { "Manual (Spasi)" });
        println!("   - Tekan [P] untuk PAUSE / RESUME {} (Berhenti mendengarkan sementara)", self.config.assistant_name);
        println!("   - Tekan [SPASI] saat {} bicara untuk MEMOTONG", self.config.assistant_name);
        println!("   - Tekan [ESC] atau [Q] untuk KELUAR / KEMBALI KE MENU");
        println!("----------------------------------------------------\n");

        let mut is_paused = false;

        let mut silent_turns_count = 0;

        loop {
            // Reload konfigurasi secara reactive sebelum mulai giliran mendengar berikutnya
            self.reload_config_if_changed();

            // Sesi Ngobrol Aktif
            if is_paused {
                TrayState::update(AssistantState::Idle, "Dijeda");
                println!("\n⏸️  [DIJEDA] {} berhenti mendengarkan sementara.", self.config.assistant_name);
                println!("👉 Tekan [P] atau [SPASI] untuk MELANJUTKAN, atau [ESC/Q] untuk KELUAR.\n");
                enable_raw_mode()?;
                loop {
                    if let Some(cmd) = check_and_consume_cmd() {
                        if cmd == "toggle_pause" || cmd == "start_chat" {
                            is_paused = false;
                            break;
                        } else if cmd == "stop_chat" || cmd == "quit" {
                            disable_raw_mode()?;
                            TrayState::clear();
                            println!("\n👋 Sesi selesai. Sampai jumpa!");
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
                                        TrayState::clear();
                                        println!("\n👋 Sesi selesai. Sampai jumpa!");
                                        return Ok(());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
                disable_raw_mode()?;
                silent_turns_count = 0;
                println!("▶️  [DILANJUTKAN] {} siap mendengarkan lagi!\n", self.config.assistant_name);
            }

            TrayState::update(AssistantState::Idle, "Siap Mendengarkan");
            if is_auto {
                println!("👉 Siap mendengarkan (Realtime)... [P: Pause | Q/ESC: Keluar]:");
            } else {
                println!("👉 Siap mendengarkan. Tekan [SPASI] untuk bicara [P: Pause | Q/ESC: Keluar]...");
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
                TrayState::clear();
                println!("\n👋 Sesi selesai. Sampai jumpa!");
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

            if clean_raw.is_empty() {
                silent_turns_count += 1;
                println!("⚠️  (Suara tidak terdengar jelas / terlalu pelan)\n");
                if silent_turns_count >= 3 {
                    TrayState::update(AssistantState::Idle, "Standby");
                }
                continue;
            }

            // Filter halusinasi hening bawaan dataset Whisper
            // Saat kondisi hening total atau suara mic bocor, Whisper sering menghalusinasi ucapan penutup video YouTube
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
                silent_turns_count += 1;
                if silent_turns_count >= 3 {
                    TrayState::update(AssistantState::Idle, "Standby");
                }
                // Hening terabaikan, lanjutkan standby tanpa spamming log
                continue;
            }

            silent_turns_count = 0;

            let user_transcript = clean_raw.to_string();
            let effective_input = user_transcript.clone();

            println!("\n👤 Kamu [{:.2?}]:", stt_elapsed);
            println!("   \"{}\"", user_transcript);

            // 2 & 3. Streaming LLM + Sentence-Level Streaming TTS (Interupsi Instan via Spasi / Q)
            TrayState::update(AssistantState::Thinking, "LLM Berpikir...");
            let llm_start = Instant::now();
            let asst_name = &self.config.assistant_name;
            println!("\n💭 {} sedang merespons... (Tekan [SPASI] untuk memotong)", asst_name);

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

            let is_anytospeech = self.config.tts_engine.eq_ignore_ascii_case("anytospeech")
                || self.config.tts_engine.eq_ignore_ascii_case("ats")
                || self.config.tts_engine.eq_ignore_ascii_case("natural_readers")
                || self.config.tts_engine.eq_ignore_ascii_case("naturalreaders")
                || self.config.tts_engine.eq_ignore_ascii_case("kiki")
                || self.config.tts_engine.eq_ignore_ascii_case("kikivoice");
            let mut ats_full_response = String::new();

            let result = self.llm.chat_stream(&effective_input, |sentence| {
                if is_interrupted_clone.load(Ordering::SeqCst) {
                    return Ok(());
                }

                let clean_sentence = sentence.trim().to_string();

                if clean_sentence.is_empty() {
                    return Ok(());
                }

                if first_token_time.is_none() {
                    // Deteksi emosi: prioritaskan tag *bintang* dari AI, fallback ke kata kunci
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
                    // Fallback kata kunci tanpa bintang
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
                        print!("\r\n🎀 {} [{:.2?}]:\r\n   ", asst_name, first_elapsed);
                        printed_header = true;
                    }
                }

                print!("{} ", clean_sentence);
                use std::io::Write;
                let _ = std::io::stdout().flush();

                if is_anytospeech {
                    // Kumpulkan semua kalimat jadi satu agar AnyToSpeech berbicara utuh tanpa terputus
                    ats_full_response.push_str(&clean_sentence);
                    ats_full_response.push(' ');
                } else {
                    // Engine streaming biasa (Google / ElevenLabs)
                    tts_clone.speak_interruptible(&clean_sentence, Arc::clone(&is_interrupted_clone))?;
                }
                Ok(())
            });

            // Jika menggunakan AnyToSpeech, bunyikan respon lengkap secara utuh sekali jalan
            if is_anytospeech && !ats_full_response.trim().is_empty() && !is_interrupted_clone.load(Ordering::SeqCst) {
                let _ = tts_clone.speak_interruptible(&ats_full_response, Arc::clone(&is_interrupted_clone));
            }

            stop_thread_flag.store(true, Ordering::SeqCst);
            if let Ok(was_exit) = keyboard_listener.join() {
                if was_exit {
                    exit_all = true;
                }
            }
            disable_raw_mode()?;
            print!("\r\n");

            if is_user_cut.load(Ordering::SeqCst) && !exit_all {
                println!("✂️  [DIPOTONG] {} berhenti bicara.", asst_name);
            }

            // Eksekusi skill jika LLM menyertakan tag [SKILL:ID]
            if let Ok(full_reply) = &result {
                let re_skill = regex::Regex::new(r"\[SKILL:([^\]]+)\]").unwrap();
                let mut found_skill = false;
                for cap in re_skill.captures_iter(full_reply) {
                    if let Some(matched_id) = cap.get(1) {
                        found_skill = true;
                        let id = matched_id.as_str().trim();
                        println!("\n🔍 [MEMPROSES SKILL]: Memanggil ID '{}'...", id);
                        if let Some((skill_name, output)) = execute_skill_by_id(id) {
                            println!("⚡ [SKILL BERHASIL: {} ({})] -> {}", skill_name, id, output.trim());
                        } else {
                            eprintln!("⚠️ [SKILL GAGAL]: ID '{}' tidak cocok di skill-command.md", id);
                        }
                    }
                }
                if !found_skill {
                    // Cek fallback teks langsung jika user meminta terminal tapi LLM kelupaan tag
                    let lower_in = effective_input.to_lowercase();
                    if lower_in.contains("buka terminal") || lower_in.contains("open terminal") || lower_in.contains("bukain terminal") || lower_in.contains("buka console") {
                        println!("\n⚡ [SMART FALLBACK]: Terdeteksi permintaan buka terminal.");
                        if let Some((skill_name, output)) = execute_skill_by_id("SYS-000") {
                            println!("⚡ [SKILL BERHASIL: {}] -> {}", skill_name, output.trim());
                        }
                    }
                }
            }

            if let Err(err) = result {
                eprintln!("⚠️  Jaringan terganggu: {err}\n");
            }

            if exit_all {
                TrayState::clear();
                println!("\n👋 Sesi selesai. Sampai jumpa!");
                break;
            }

            // Berikan jeda settling time ~300ms agar gema speaker di ruangan lenyap,
            // lalu bersihkan buffer mic sebelum turn berikutnya dibuka
            std::thread::sleep(std::time::Duration::from_millis(300));
            self.recorder.clear_buffer();

            println!("----------------------------------------------------\n");
        }

        Ok(())
    }
}
