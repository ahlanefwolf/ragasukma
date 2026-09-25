// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(clippy::collapsible_if)]

pub mod app_state;
pub mod assets;
mod audio;
mod config;
pub mod engine;

mod llm;
pub mod server;
pub mod services;
pub mod ui;
pub use services::tray_service as tray;

use anyhow::Result;
use config::AppConfig;
use engine::AppSession;

fn init_dpi_awareness() {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::UI::HiDpi::{
            SetProcessDpiAwarenessContext, DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2,
        };
        // Mengatur proses menjadi Per-Monitor V2 DPI Aware agar koordinat
        // mouse (SetCursorPos), UI Automation (CurrentBoundingRectangle),
        // dan screenshot monitor sinkron 1:1 tanpa offset akibat display scaling (125%, 150%, dst).
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
}

fn main() -> Result<()> {
    init_dpi_awareness();
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--scan-vision") {
        println!("👁️ Menjalankan local perception scan via dedicated Vision Worker (PP-OCR + OmniParser Icon Detection)...");
        let result = engine::perception::request_vision_scan_sync();
        println!("✅ Vision Scan Berhasil!");
        println!("  Total Elemen  : {} item", result.elements.len());
        println!("  Tombol / Ikon : {} item", result.buttons.len());
        for b in result.buttons.iter().take(8) {
            println!("    - [{}] \"{}\" pada center {:?}", b.role, b.label, b.center);
        }
        println!("  Input Fields  : {} item", result.inputs.len());
        for inp in result.inputs.iter().take(5) {
            println!("    - [{}] \"{}\" pada center {:?}", inp.role, inp.label, inp.center);
        }
        println!("  Teks Terbaca  : {} item", result.texts.len());
        for t in result.texts.iter().take(10) {
            println!("    - \"{}\"", t);
        }
        println!("📁 File tersimpan di assets/active_window_app.json (source: vision_perception)");
        return Ok(());
    }
    if args.iter().any(|a| a == "--scan-window") {
        println!("🔍 Menjalankan native OS accessibility scan...");
        match engine::window_scanner::scan_active_window_sync() {
            Ok(snapshot) => {
                println!("✅ Scan Berhasil!");
                println!("  App Name     : {}", snapshot.app_name);
                println!("  Window Title : {}", snapshot.window_title);
                println!("  Window ID    : {}", snapshot.window_id);
                println!("  Tombol       : {} item", snapshot.buttons.len());
                for b in snapshot.buttons.iter().take(5) {
                    println!("    - [Button #{}] {}", b.id, b.label);
                }
                println!("  Input Form   : {} item", snapshot.inputs.len());
                for inp in snapshot.inputs.iter().take(5) {
                    println!("    - [Input #{}] {}", inp.id, inp.label);
                }
                println!("  Label Teks   : {} item", snapshot.texts.len());
                for t in snapshot.texts.iter().take(5) {
                    println!("    - {}", t);
                }

                if let Err(e) = snapshot.save_to_file() {
                    eprintln!("❌ Gagal simpan ke file: {e}");
                } else {
                    println!("📁 File tersimpan di assets/active_window_app.json");
                }
            }
            Err(e) => {
                eprintln!("❌ Error saat scan: {e}");
            }
        }
        return Ok(());
    }

    // 🔒 Single Instance Check: Verifikasi port dan handshake identitas aplikasi
    let server_port = 42128;
    let single_instance_check = match std::net::TcpListener::bind(("127.0.0.1", server_port)) {
        Ok(listener) => listener,
        Err(_) => {
            // Cek apakah port 42128 memang benar instance Ragasukma (Handshake verification)
            let is_ragasukma_instance = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_millis(500))
                .build()
                .ok()
                .and_then(|client| {
                    client
                        .get(format!("http://127.0.0.1:{}/api/ping", server_port))
                        .send()
                        .ok()
                })
                .and_then(|res| res.text().ok())
                .map(|text| text.contains("\"ragasukma\""))
                .unwrap_or(false);

            if is_ragasukma_instance {
                println!("⚠️  [Single Instance] Ragasukma sudah berjalan di latar belakang!");
                println!(
                    "🌐 Membuka Dashboard yang sedang aktif di http://127.0.0.1:{} ...",
                    server_port
                );
                crate::assets::open_browser(&format!("http://127.0.0.1:{}", server_port));
                return Ok(());
            } else {
                eprintln!(
                    "⚠️  [Port Conflict] Port {} sedang dipakai oleh aplikasi lain bukan Ragasukma!",
                    server_port
                );
                eprintln!(
                    "🛑 Harap tutup aplikasi yang menggunakan port {} tersebut.",
                    server_port
                );
                return Ok(());
            }
        }
    };

    // Pertahankan socket listener tanpa drop untuk mencegah celah TOCTOU (Time-of-Check to Time-of-Use)
    crate::server::register_prebound_listener(single_instance_check);

    // Salin otomatis semua aset default ke folder config pengguna (~/.config/ragasukma/assets)
    crate::assets::ensure_default_user_assets();

    // 🧹 Reset bersih folder snapshot browser sessions dari sesi sebelumnya
    crate::services::browser_session_service::cleanup_browser_sessions_on_startup();

    // Inisialisasi dedicated BrowserSessionWorker untuk sinkronisasi non-blocking active_tab_browser.json
    crate::services::browser_session_service::init_browser_session_worker();

    // Inisialisasi native OS active window watcher (reaktif) untuk active_window_app.json
    crate::engine::window_scanner::start_window_watcher();

    // Inisialisasi standby dedicated vision worker thread (0% CPU idle)
    crate::engine::perception::start_vision_worker();

    // Inisialisasi dedicated background task worker thread
    crate::engine::init_background_task_worker();

    let config = AppConfig::load();
    let config_clone = config.clone();

    crate::term_log!("====================================================");
    crate::term_log!(
        "  Ragasukma - Asisten Pribadi: {} (Anime ID)",
        config.assistant_name
    );
    crate::term_log!("====================================================");

    // Perbesar stack size session thread (16 MB) agar aman di AppImage/portable Linux
    let session_thread = std::thread::Builder::new()
        .name("ragasukma-session".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || match AppSession::new(config_clone) {
            Ok(mut session) => {
                if let Err(e) = session.run_interactive_menu() {
                    crate::term_err!("❌ [Session Error]: {:?}", e);
                }
            }
            Err(e) => {
                crate::term_err!("❌ [AppSession Init Error]: {:?}", e);
            }
        })
        .expect("Gagal membuat session thread");

    // 🖥️ Jalankan 3D VRM Native Overlay di OS Main Thread (Thread 0)
    // Sesuai standar industri Cocoa/macOS, Windows, dan Linux X11/Wayland
    ui::run_native_overlay();

    // Kirim perintah quit ke session thread agar shutdown secara rapi (graceful shutdown)
    tray::TrayState::push_command("quit".to_string());
    let _ = session_thread.join();
    std::process::exit(0);
}
