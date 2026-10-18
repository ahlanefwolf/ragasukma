use std::sync::Mutex;
use std::sync::mpsc::{Sender, channel};
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
static ACTIVE_DOWNLOADS: std::sync::LazyLock<Mutex<std::collections::HashSet<String>>> =
    std::sync::LazyLock::new(|| Mutex::new(std::collections::HashSet::new()));

/// Initialize dedicated background worker thread for yt-dlp & other heavy asynchronous tasks
pub fn init_background_task_worker() {
    let mut sender_lock = match TASK_SENDER.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    if sender_lock.is_some() {
        return; // Already initialized
    }

    let (tx, rx) = channel::<BackgroundTask>();
    *sender_lock = Some(tx);

    // Dedicated worker thread with 2 MB stack allocation
    let builder = thread::Builder::new()
        .name("ragasukma-task-worker".to_string())
        .stack_size(2 * 1024 * 1024);

    builder
        .spawn(move || {
            crate::term_log!("[Task Worker] Dedicated Background Task Engine ready.");

            while let Ok(task) = rx.recv() {
                match task {
                    BackgroundTask::DownloadAndPlayMusic { query } => {
                        set_current_task(Some(BackgroundTaskState {
                            is_busy: true,
                            task_type: "ytdlp_download".to_string(),
                            description: format!("Downloading song: {query}"),
                        }));

                        crate::term_log!(" [Task Worker: yt-dlp] Downloading in background: '{}'...", query);
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
                                    " [Task Worker: yt-dlp] Done: '{}' -> Playing directly to speaker!",
                                    track.title
                                );

                                // 1. Play directly via desktop speakers (Native Audio Player)
                                let file_path = music_dir.join(&track.filename);
                                crate::audio::player::play_file(&file_path, &track.title);

                                // 2. Sync to Web Dashboard
                                push_client_action("music:play", Some(&track.title));

                                TrayState::emit_event(
                                    "task:completed",
                                    serde_json::json!({
                                        "type": "ytdlp_download",
                                        "title": track.title,
                                        "status": "success"
                                    }),
                                );

                                // 3. Send a signal to the assistant so that it speaks to inform the user that the song has finished downloading & playing
                                let notify_msg = format!(
                                    "[SYSTEM STATUS: Song '{}' has been downloaded and is now playing automatically. Inform the user briefly and warmly. DO NOT include any [CMD:media:play] or other commands — music is already playing.]",
                                    track.title
                                );
                                crate::engine::remote_text::inject_remote_text(notify_msg);
                            }
                            Err(e) => {
                                crate::term_err!(" [Task Worker: yt-dlp] Failed to download '{}': {e}", query);
                                TrayState::emit_event(
                                    "task:failed",
                                    serde_json::json!({
                                        "type": "ytdlp_download",
                                        "query": &query,
                                        "error": e.to_string(),
                                        "status": "failed"
                                    }),
                                );

                                // Send a signal to assistant so that notifies the user that the download failed
                                let err_msg = format!(
                                    "[SYSTEM STATUS: Failed to download song '{}'. Detail: {}. Inform the user briefly and warmly. DO NOT include any commands.]",
                                    query, e
                                );
                                crate::engine::remote_text::inject_remote_text(err_msg);
                            }
                        }

                        if let Ok(mut active) = ACTIVE_DOWNLOADS.lock() {
                            active.remove(&query.trim().to_lowercase());
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

/// Send download & play music tasks to Background Worker
pub fn queue_download_and_play(query: &str) -> bool {
    let q_clean = query.trim();
    if q_clean.is_empty() {
        return false;
    }
    let q_key = q_clean.to_lowercase();
    if let Ok(mut active) = ACTIVE_DOWNLOADS.lock() {
        if active.contains(&q_key) {
            crate::term_log!(
                "[Task Worker: yt-dlp] Song '{}' is already currently downloading or queued. Skipping duplicate queue.",
                q_clean
            );
            return true;
        }
        active.insert(q_key.clone());
    }

    init_background_task_worker();
    if let Ok(lock) = TASK_SENDER.lock() {
        if let Some(tx) = lock.as_ref() {
            let sent = tx
                .send(BackgroundTask::DownloadAndPlayMusic {
                    query: q_clean.to_string(),
                })
                .is_ok();
            if !sent {
                if let Ok(mut active) = ACTIVE_DOWNLOADS.lock() {
                    active.remove(&q_key);
                }
            }
            return sent;
        }
    }

    if let Ok(mut active) = ACTIVE_DOWNLOADS.lock() {
        active.remove(&q_key);
    }
    false
}

/// Find out whether a background worker is currently busy processing media downloads
pub fn is_ytdlp_busy() -> bool {
    if let Ok(lock) = CURRENT_TASK_INFO.lock() {
        lock.as_ref()
            .map(|t| t.is_busy && t.task_type == "ytdlp_download")
            .unwrap_or(false)
    } else {
        false
    }
}

/// Gets current active task info (for AI context or telemetry)
pub fn get_active_task_info() -> Option<BackgroundTaskState> {
    if let Ok(lock) = CURRENT_TASK_INFO.lock() {
        lock.clone()
    } else {
        None
    }
}
