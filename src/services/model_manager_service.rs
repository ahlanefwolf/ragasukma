use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

pub const MODEL_DOWNLOAD_URL: &str =
    "https://huggingface.co/ahlanefwolf/ragasukma-models/resolve/main/supertonic-3.zip";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDownloadStatus {
    pub status: String, // "ready", "checking", "downloading", "extracting", "error"
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percent: f32,
    pub speed_mbps: f32,
    pub message: String,
}

impl Default for ModelDownloadStatus {
    fn default() -> Self {
        Self {
            status: "ready".to_string(),
            downloaded_bytes: 0,
            total_bytes: 0,
            percent: 100.0,
            speed_mbps: 0.0,
            message: "Model is ready".to_string(),
        }
    }
}

static MODEL_STATUS: OnceLock<Mutex<ModelDownloadStatus>> = OnceLock::new();
static IS_DOWNLOADING: AtomicBool = AtomicBool::new(false);

fn get_status_lock() -> &'static Mutex<ModelDownloadStatus> {
    MODEL_STATUS.get_or_init(|| Mutex::new(ModelDownloadStatus::default()))
}

pub fn get_model_status() -> ModelDownloadStatus {
    get_status_lock().lock().unwrap_or_else(|e| e.into_inner()).clone()
}

pub fn update_model_status(status: ModelDownloadStatus) {
    let mut lock = get_status_lock().lock().unwrap_or_else(|e| e.into_inner());
    *lock = status;
}

pub fn check_model_exists() -> bool {
    let candidates = [
        PathBuf::from("models/supertonic-3"),
        PathBuf::from("../models/supertonic-3"),
        PathBuf::from("/home/headway/MyWork/project/node/ai-voice/models/supertonic-3"),
    ];
    candidates
        .into_iter()
        .any(|c| c.join("onnx/vocoder.onnx").exists())
}

pub fn ensure_model_ready_background() {
    if check_model_exists() {
        update_model_status(ModelDownloadStatus {
            status: "ready".to_string(),
            downloaded_bytes: 0,
            total_bytes: 0,
            percent: 100.0,
            speed_mbps: 0.0,
            message: "Model already installed".to_string(),
        });
        crate::audio::supertonic::init_engine_background();
        return;
    }

    if IS_DOWNLOADING.swap(true, Ordering::SeqCst) {
        return;
    }

    std::thread::Builder::new()
        .name("model-downloader".to_string())
        .spawn(move || {
            if let Err(err) = run_download_and_extract() {
                crate::term_err!("[Model Manager] Failed to download or extract model: {err}");
                update_model_status(ModelDownloadStatus {
                    status: "error".to_string(),
                    downloaded_bytes: 0,
                    total_bytes: 0,
                    percent: 0.0,
                    speed_mbps: 0.0,
                    message: format!("Download failed: {err}"),
                });
            } else {
                update_model_status(ModelDownloadStatus {
                    status: "ready".to_string(),
                    downloaded_bytes: 0,
                    total_bytes: 0,
                    percent: 100.0,
                    speed_mbps: 0.0,
                    message: "Model ready".to_string(),
                });
                crate::audio::supertonic::init_engine_background();
            }
            IS_DOWNLOADING.store(false, Ordering::SeqCst);
        })
        .expect("Failed to spawn model-downloader thread");
}

fn run_download_and_extract() -> anyhow::Result<()> {
    crate::term_log!("[Model Manager] Supertonic-3 model missing. Starting automated download...");

    let models_dir = Path::new("models");
    if !models_dir.exists() {
        std::fs::create_dir_all(models_dir)?;
    }

    let zip_path = models_dir.join("supertonic-3.zip");

    update_model_status(ModelDownloadStatus {
        status: "downloading".to_string(),
        downloaded_bytes: 0,
        total_bytes: 0,
        percent: 0.0,
        speed_mbps: 0.0,
        message: "Connecting to Hugging Face...".to_string(),
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3600))
        .build()?;

    let mut response = client.get(MODEL_DOWNLOAD_URL).send()?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "HTTP request failed with status: {}",
            response.status()
        ));
    }

    let total_bytes = response.content_length().unwrap_or(0);
    let mut dest_file = File::create(&zip_path)?;

    let mut downloaded_bytes: u64 = 0;
    let mut buffer = [0u8; 64 * 1024]; // 64KB chunks
    let start_time = Instant::now();
    let mut last_update_time = Instant::now();

    loop {
        let n = response.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        dest_file.write_all(&buffer[..n])?;
        downloaded_bytes += n as u64;

        if last_update_time.elapsed().as_millis() >= 250 {
            let elapsed_sec = start_time.elapsed().as_secs_f32().max(0.001);
            let speed_mbps = (downloaded_bytes as f32 / (1024.0 * 1024.0)) / elapsed_sec;
            let percent = if total_bytes > 0 {
                (downloaded_bytes as f32 / total_bytes as f32) * 100.0
            } else {
                0.0
            };

            update_model_status(ModelDownloadStatus {
                status: "downloading".to_string(),
                downloaded_bytes,
                total_bytes,
                percent,
                speed_mbps,
                message: format!(
                    "Downloading Supertonic-3 ({:.1} MB / {:.1} MB)...",
                    downloaded_bytes as f32 / (1024.0 * 1024.0),
                    total_bytes as f32 / (1024.0 * 1024.0)
                ),
            });
            last_update_time = Instant::now();
        }
    }
    dest_file.flush()?;
    drop(dest_file);

    crate::term_log!(
        "[Model Manager] Download complete ({:.1} MB). Extracting...",
        downloaded_bytes as f32 / (1024.0 * 1024.0)
    );

    update_model_status(ModelDownloadStatus {
        status: "extracting".to_string(),
        downloaded_bytes,
        total_bytes,
        percent: 100.0,
        speed_mbps: 0.0,
        message: "Extracting model files...".to_string(),
    });

    extract_zip(&zip_path, models_dir)?;

    let _ = std::fs::remove_file(&zip_path);

    crate::term_log!("[Model Manager] Model successfully installed to models/supertonic-3!");
    Ok(())
}

fn extract_zip(archive_path: &Path, output_dir: &Path) -> anyhow::Result<()> {
    let file = File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => output_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}
