// Prevents console window on Windows (runs silently in background)
#![windows_subsystem = "windows"]
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
            DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetProcessDpiAwarenessContext,
        };
        // Configure process to Per-Monitor V2 DPI Aware so coordinates
        // mouse (SetCursorPos), UI Automation (CurrentBoundingRectangle),
        // and monitor screenshots remain 1:1 synchronized without display scaling offset (125%, 150%, etc).
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
    }
}

fn main() -> Result<()> {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows::Win32::System::Console::FreeConsole;
        // Jika aplikasi dijalankan tanpa argument console/debug (misal via GUI/portable double-click),
        // paksa lepaskan console handle agar Windows 11 / Windows Terminal tidak memunculkan window CMD.
        if !std::env::args().any(|a| a == "--console" || a == "--debug") {
            let _ = FreeConsole();
        } else {
            use windows::Win32::System::Console::{AllocConsole, AttachConsole, ATTACH_PARENT_PROCESS};
            if AttachConsole(ATTACH_PARENT_PROCESS).is_err() {
                let _ = AllocConsole();
            }
        }
    }

    init_dpi_awareness();
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--scan-vision") {
        println!(
            " Running local perception scan via dedicated Vision Worker (PP-OCR + OmniParser Icon Detection)..."
        );
        let result = engine::perception::request_vision_scan_sync();
        println!(" Vision Scan Successful!");
        println!("  Total Elements : {} items", result.elements.len());
        println!("  Buttons / Icons: {} items", result.buttons.len());
        for b in result.buttons.iter().take(8) {
            println!(
                "    - [{}] \"{}\" at center {:?}",
                b.role, b.label, b.center
            );
        }
        println!("  Input Fields   : {} items", result.inputs.len());
        for inp in result.inputs.iter().take(5) {
            println!(
                "    - [{}] \"{}\" at center {:?}",
                inp.role, inp.label, inp.center
            );
        }
        println!("  Detected Texts : {} items", result.texts.len());
        for t in result.texts.iter().take(10) {
            println!("    - \"{}\"", t);
        }
        println!(" File saved to assets/active_window_app.json (source: vision_perception)");
        return Ok(());
    }
    if args.iter().any(|a| a == "--scan-window") {
        println!(" Running native OS accessibility scan...");
        match engine::window_scanner::scan_active_window_sync() {
            Ok(snapshot) => {
                println!(" Scan Successful!");
                println!("  App Name     : {}", snapshot.app_name);
                println!("  Window Title : {}", snapshot.window_title);
                println!("  Window ID    : {}", snapshot.window_id);
                println!("  Buttons        : {} items", snapshot.buttons.len());
                for b in snapshot.buttons.iter().take(5) {
                    println!("    - [Button #{}] {}", b.id, b.label);
                }
                println!("  Input Forms    : {} items", snapshot.inputs.len());
                for inp in snapshot.inputs.iter().take(5) {
                    println!("    - [Input #{}] {}", inp.id, inp.label);
                }
                println!("  Text Labels    : {} items", snapshot.texts.len());
                for t in snapshot.texts.iter().take(5) {
                    println!("    - {}", t);
                }
                if let Some(target) = args.get(2) {
                    println!(" File saved to {}", target);
                } else {
                    println!(" File saved to assets/active_window_app.json");
                }
            }
            Err(e) => {
                eprintln!(" Error during scan: {e}");
            }
        }
        return Ok(());
    }

    //  Single Instance Check: Verify port and application identity handshake
    let server_port = 42128;
    // Check if port 42128 is already active using lightweight HTTP handshake
    let is_ragasukma_instance = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_millis(300))
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
        println!("[Single Instance] Ragasukma is already running in background!");
        println!(
            " Opening active Dashboard at http://127.0.0.1:{} ...",
            server_port
        );
        crate::assets::open_browser(&format!("http://127.0.0.1:{}", server_port));
        return Ok(());
    }

    // Automatically copy all default assets to user config folder (~/.config/ragasukma/assets)
    crate::assets::ensure_default_user_assets();

    //  Clean reset browser session snapshot folder from previous sessions
    crate::services::browser_session_service::cleanup_browser_sessions_on_startup();

    // Initialize dedicated BrowserSessionWorker for non-blocking active_tab_browser.json synchronization
    crate::services::browser_session_service::init_browser_session_worker();

    // Initialize reactive native OS active window watcher for active_window_app.json
    crate::engine::window_scanner::start_window_watcher();

    // Initialize standby dedicated vision worker thread (0% CPU idle)
    crate::engine::perception::start_vision_worker();

    // Initialize dedicated background task worker thread
    crate::engine::init_background_task_worker();

    // Check & download Supertonic-3 TTS model if missing, then init engine
    crate::services::model_manager_service::ensure_model_ready_background();

    let config = AppConfig::load();
    let config_clone = config.clone();

    crate::term_log!("====================================================");
    crate::term_log!(
        "  Ragasukma - Personal Assistant: {} (Anime ID)",
        config.assistant_name
    );
    crate::term_log!("====================================================");

    // Expand session thread stack size (16 MB) for safety on AppImage/portable Linux
    // Equipped with auto-recovery loop so internal panics (e.g. audio driver) do not kill the session
    let session_thread = std::thread::Builder::new()
        .name("ragasukma-session".to_string())
        .stack_size(16 * 1024 * 1024)
        .spawn(move || loop {
            let cfg = config_clone.clone();
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                match AppSession::new(cfg) {
                    Ok(mut session) => {
                        if let Err(e) = session.run_interactive_menu() {
                            crate::term_err!("[Session Error]: {:?}", e);
                        }
                    }
                    Err(e) => {
                        crate::term_err!("[AppSession Init Error]: {:?}", e);
                    }
                }
            }));
            if res.is_err() {
                crate::term_err!(
                    "[Session Auto-Recover] Session thread recovered from unexpected error, restarting session loop..."
                );
                std::thread::sleep(std::time::Duration::from_millis(500));
            } else {
                break;
            }
        })
        .expect("Failed to create session thread");

    //  Run 3D VRM Native Overlay on OS Main Thread (Thread 0)
    // Following Cocoa/macOS, Windows, and Linux X11/Wayland industry standards
    ui::run_native_overlay();

    // Send quit command to session thread for graceful shutdown
    let _ = engine::cloudflared::stop_cloudflared_tunnel();
    tray::TrayState::push_command("quit".to_string());
    let _ = session_thread.join();
    std::process::exit(0);
}
