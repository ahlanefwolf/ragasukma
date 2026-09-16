use std::sync::atomic::{AtomicBool, Ordering};
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
static IS_YTDLP_RUNNING: AtomicBool = AtomicBool::new(false);
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

    // Dedicated worker thread dengan alokasi stack aman 16 MB
    let builder = thread::Builder::new().name("ragasukma-task-worker".to_string()).stack_size(16 * 1024 * 1024);
    builder.spawn(move || {
        crate::term_log!("🚀 [Task Worker] Dedicated Background Task Engine siap.");

        while let Ok(task) = rx.recv() {
            match task {
                BackgroundTask::DownloadAndPlayMusic { query } => {
                    IS_YTDLP_RUNNING.store(true, Ordering::SeqCst);
                    if let Ok(mut lock) = CURRENT_TASK_INFO.lock() {
                        *lock = Some(BackgroundTaskState {
                            is_busy: true,
                            task_type: "ytdlp_download".to_string(),
                            description: format!("Mengunduh lagu: {query}"),
                        });
                    }

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
                            crate::term_log!("✅ [Task Worker: yt-dlp] Selesai: '{}' -> Meminta web player memutar!", track.title);
                            push_client_action("music:play", Some(&track.title));

                            TrayState::emit_event(
                                "task:completed",
                                serde_json::json!({
                                    "type": "ytdlp_download",
                                    "title": track.title,
                                    "status": "success"
                                }),
                            );
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
                        }
                    }

                    IS_YTDLP_RUNNING.store(false, Ordering::SeqCst);
                    if let Ok(mut lock) = CURRENT_TASK_INFO.lock() {
                        *lock = None;
                    }
                }
            }
        }
    }).expect("Failed to spawn background task worker thread");
}

/// Kirim tugas download & putar musik ke Background Worker
pub fn queue_download_and_play(query: &str) -> bool {
    init_background_task_worker();
    if let Ok(lock) = TASK_SENDER.lock() {
        if let Some(tx) = lock.as_ref() {
            return tx.send(BackgroundTask::DownloadAndPlayMusic { query: query.to_string() }).is_ok();
        }
    }
    false
}

/// Mengetahui apakah yt-dlp sedang aktif memproses download di background
pub fn is_ytdlp_busy() -> bool {
    IS_YTDLP_RUNNING.load(Ordering::SeqCst)
}

/// Mendapatkan info task aktif saat ini (untuk AI context atau telemetry)
pub fn get_active_task_info() -> Option<BackgroundTaskState> {
    if let Ok(lock) = CURRENT_TASK_INFO.lock() {
        lock.clone()
    } else {
        None
    }
}
