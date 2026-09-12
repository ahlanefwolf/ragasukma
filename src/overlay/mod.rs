pub mod assets;
pub mod position;
pub mod state;
pub mod updater;
pub mod windows;

use std::sync::Mutex;
use std::thread;

use tauri::Emitter;

use crate::skills::*;

pub use assets::*;
pub use position::*;
pub use state::*;
pub use updater::*;
pub use windows::*;

pub struct NativeOverlay;

impl NativeOverlay {
    pub fn run() {
        let pos_cfg = load_position_config();
        let overlay_state = OverlayState {
            position_config: Mutex::new(pos_cfg.clone()),
            is_panel_mode: std::sync::atomic::AtomicBool::new(false),
        };

        let app = tauri::Builder::default()
            .manage(overlay_state)
            .on_window_event(|window, event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window.emit("request_close_panel", ());
                }
                if let tauri::WindowEvent::Moved(_) = event {
                    use tauri::Manager;
                    let state = window.state::<OverlayState>();
                    if !state.is_panel_mode.load(std::sync::atomic::Ordering::SeqCst) {
                        if let Some((x, y)) = get_actual_window_position(window) {
                            let mut cfg = match state.position_config.lock() {
                                Ok(guard) => guard,
                                Err(poisoned) => poisoned.into_inner(),
                            };
                            cfg.custom_x = Some(x);
                            cfg.custom_y = Some(y);
                            save_position_config(&cfg);
                        }
                    }
                }
            })
            .invoke_handler(tauri::generate_handler![
                get_app_config,
                get_app_state,
                send_command,
                start_drag,
                drag_window_delta,
                save_current_position,
                set_overlay_size,
                change_size_delta,
                reset_size,
                set_corner,
                get_skills,
                save_skill,
                delete_skill,
                execute_skill,
                open_skills_window,
                open_settings_window,
                get_full_user_config,
                save_full_user_config,
                close_current_window,
                open_menu_window,
                open_tuner_window,
                get_poses_config,
                save_poses_config,
                get_external_asset,
                get_external_asset_bytes,
                save_avatar_file,
                open_browser_url,
                check_for_updates,
                set_window_mode,
                minimize_window,
                toggle_maximize_window
            ])
            .setup(move |app| {
                if let Ok(mut guard) = APP_HANDLE.lock() {
                    *guard = Some(app.handle().clone());
                }

                // Otomatis spawn dan tampilkan avatar overlay window (main) saat startup
                let handle = app.handle().clone();
                if let Err(e) = create_avatar_window(&handle) {
                    eprintln!("⚠️ Gagal membuat avatar overlay window saat startup: {e}");
                }

                // Background watcher untuk sinyal quit (100% in-memory thread-safe)
                thread::spawn(move || {
                    loop {
                        thread::sleep(std::time::Duration::from_millis(100));
                        if crate::tray::TrayState::is_quit_requested() {
                            handle.exit(0);
                            break;
                        }
                    }
                });

                Ok(())
            })
            .build(tauri::generate_context!());

        match app {
            Ok(app) => {
                app.run(|_app_handle, _event| {});
            }
            Err(err) => {
                eprintln!("⚠️ Gagal menginisialisasi Tauri overlay: {}", err);
            }
        }
    }
}
