use std::sync::mpsc::{Sender, channel};
use std::sync::Mutex;
use std::thread;

use crate::app_state::push_client_action;
use crate::engine::ytdlp::download_audio;
use crate::tray::TrayState;

#[derive(Debug, Clone)]
pub enum BackgroundTask {
    DownloadAndPlayMusic { query: String },
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct BackgroundTaskState {
    pub is_busy: bool,
    pub task_type: String,
    pub description: String,
}

static TASK_SENDER: Mutex<Option<Sender<BackgroundTask>>> = Mutex::new(None);
static CURRENT_TASK_INFO: Mutex<Option<BackgroundTaskState>> = Mutex::new(None);

/// Inisialisasi dedicated background worker thread untuk yt-dlp & tugas asinkron berat lainnya
pub fn init_background_task_worker() {
    let mut sender_lock = match TASK_SENDER.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if sender_lock.is_some() {
        return; // Sudah diinisialisasi
    }

    let (tx, rx) = channel::<BackgroundTask>();
    *sender_lock = Some(tx);

    // Dedicated worker thread dengan alokasi stack 16 MB
    let builder = thread::Builder::new()
        .name("ragasukma-task-worker".to_string())
        .stack_size(16 * 1024 * 1024);

    builder
        .spawn(move || {
            crate::term_log!("🚀 [Task Worker] Dedicated Background Task Engine siap.");

            while let Ok(task) = rx.recv() {
                match task {
                    BackgroundTask::DownloadAndPlayMusic { query } => {
                        set_current_task(Some(BackgroundTaskState {
                            is_busy: true,
                            task_type: "ytdlp_download".to_string(),
                            description: format!("Mengunduh lagu: {query}"),
                        }));

                        crate::term_log!("📥 [Task Worker: yt-dlp] Mengunduh di background: '{}'...", query);
                        TrayState::emit_event(
                            "task:started",
                            serde_json::json!({
                                "type": "ytdlp_download",
                                "query": &query,
                                "status": "downloading"
                            }),
                        );

                        let music_dir = crate::config::get_music_dir();
                        match download_audio(&query, &music_dir) {
                            Ok(track) => {
                                crate::term_log!(
                                    "✅ [Task Worker: yt-dlp] Selesai: '{}' -> Memutar langsung ke speaker!",
                                    track.title
                                );

                                // 1. Putar langsung lewat speaker desktop (Native Audio Player)
                                let file_path = music_dir.join(&track.filename);
                                crate::audio::player::play_file(&file_path, &track.title);

                                // 2. Sinkronkan ke Web Dashboard
                                push_client_action("music:play", Some(&track.title));

                                TrayState::emit_event(
                                    "task:completed",
                                    serde_json::json!({
                                        "type": "ytdlp_download",
                                        "title": track.title,
                                        "status": "success"
                                    }),
                                );

                                // 3. Kirim sinyal ke asisten agar berbicara mengabari pengguna bahwa lagu telah selesai diunduh & diputar
                                let notify_msg = format!(
                                    "[STATUS SISTEM: Lagu '{}' telah berhasil diunduh dan sedang diputar. Beritahu pengguna dengan ramah dan sangat singkat!]",
                                    track.title
                                );
                                crate::engine::remote_text::inject_remote_text(notify_msg);
                            }
                            Err(e) => {
                                crate::term_err!("❌ [Task Worker: yt-dlp] Gagal mengunduh '{}': {e}", query);
                                TrayState::emit_event(
                                    "task:failed",
                                    serde_json::json!({
                                        "type": "ytdlp_download",
                                        "query": &query,
                                        "error": e.to_string(),
                                        "status": "failed"
                                    }),
                                );

                                // Kirim sinyal ke asisten agar memberi tahu pengguna bahwa unduhan gagal
                                let err_msg = format!(
                                    "[STATUS SISTEM: Gagal mengunduh lagu '{}'. Detail: {}. Beritahu pengguna dengan sopan dan singkat!]",
                                    query, e
                                );
                                crate::engine::remote_text::inject_remote_text(err_msg);
                            }
                        }

                        set_current_task(None);
                    }
                }
            }
        })
        .expect("Failed to spawn background task worker thread");
}

fn set_current_task(state: Option<BackgroundTaskState>) {
    if let Ok(mut lock) = CURRENT_TASK_INFO.lock() {
        *lock = state;
    }
}

/// Kirim tugas download & putar musik ke Background Worker
pub fn queue_download_and_play(query: &str) -> bool {
    init_background_task_worker();
    if let Ok(lock) = TASK_SENDER.lock() {
        if let Some(tx) = lock.as_ref() {
            return tx
                .send(BackgroundTask::DownloadAndPlayMusic {
                    query: query.to_string(),
                })
                .is_ok();
        }
    }
    false
}

/// Mengetahui apakah background worker sedang sibuk memproses unduhan media
pub fn is_ytdlp_busy() -> bool {
    if let Ok(lock) = CURRENT_TASK_INFO.lock() {
        lock.as_ref().map(|t| t.is_busy && t.task_type == "ytdlp_download").unwrap_or(false)
    } else {
        false
    }
}

/// Mendapatkan info task aktif saat ini (untuk AI context atau telemetry)
pub fn get_active_task_info() -> Option<BackgroundTaskState> {
    if let Ok(lock) = CURRENT_TASK_INFO.lock() {
        lock.clone()
    } else {
        None
    }
}
