use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;

pub const SUPERTONIC_DOWNLOAD_URL: &str =
    "https://huggingface.co/ahlanefwolf/ragasukma-models/resolve/main/supertonic-3.zip";
pub const OPENVOICE_DOWNLOAD_URL: &str =
    "https://huggingface.co/ahlanefwolf/ragasukma-models/resolve/main/openvoice-v2.zip";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelDownloadStatus {
    pub status: String, // "ready", "checking", "downloading", "extracting", "error"
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub percent: f32,
    pub speed_mbps: f32,
    pub message: String,
    #[serde(default)]
    pub model_name: Option<String>,
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
            model_name: None,
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

pub fn check_supertonic_exists() -> bool {
    let mut candidates = vec![
        PathBuf::from("models/supertonic-3"),
        PathBuf::from("../models/supertonic-3"),
        crate::config::get_models_dir().join("supertonic-3"),
        crate::config::get_app_data_dir().join("models/supertonic-3"),
        crate::config::get_app_config_dir().join("models/supertonic-3"),
    ];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join("models/supertonic-3"));
            candidates.push(parent.join("../models/supertonic-3"));
        }
    }
    candidates
        .into_iter()
        .any(|c| c.join("onnx/vocoder.onnx").exists())
}

pub fn check_openvoice_exists() -> bool {
    let mut candidates = vec![
        PathBuf::from("models/openvoice-v2"),
        PathBuf::from("../models/openvoice-v2"),
        crate::config::get_models_dir().join("openvoice-v2"),
        crate::config::get_app_data_dir().join("models/openvoice-v2"),
        crate::config::get_app_config_dir().join("models/openvoice-v2"),
    ];
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            candidates.push(parent.join("models/openvoice-v2"));
            candidates.push(parent.join("../models/openvoice-v2"));
        }
    }
    candidates
        .into_iter()
        .any(|c| c.join("openvoice_tone_converter.onnx").exists())
}

pub fn check_model_exists() -> bool {
    check_supertonic_exists() && check_openvoice_exists()
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
            model_name: None,
        });
        // Note: TTS engine is now isolated in on-demand worker process (--worker-tts)
        // to prevent allocating ~400MB ONNX models in the host process memory.
        return;
    }

    if IS_DOWNLOADING.swap(true, Ordering::SeqCst) {
        return;
    }

    let spawn_res = std::thread::Builder::new()
        .name("model-downloader".to_string())
        .stack_size(1024 * 1024)
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
                    model_name: None,
                });
            } else {
                update_model_status(ModelDownloadStatus {
                    status: "ready".to_string(),
                    downloaded_bytes: 0,
                    total_bytes: 0,
                    percent: 100.0,
                    speed_mbps: 0.0,
                    message: "Model ready".to_string(),
                    model_name: None,
                });
            }
            IS_DOWNLOADING.store(false, Ordering::SeqCst);
        });

    if let Err(err) = spawn_res {
        crate::term_err!("[Model Manager] Failed to spawn model-downloader thread: {err}");
        IS_DOWNLOADING.store(false, Ordering::SeqCst);
        update_model_status(ModelDownloadStatus {
            status: "error".to_string(),
            downloaded_bytes: 0,
            total_bytes: 0,
            percent: 0.0,
            speed_mbps: 0.0,
            message: format!("Failed to spawn downloader thread: {err}"),
            model_name: None,
        });
    }
}

fn get_target_models_dir() -> PathBuf {
    crate::config::get_models_dir()
}

fn run_download_and_extract() -> anyhow::Result<()> {
    let models_dir = get_target_models_dir();
    if !models_dir.exists() {
        std::fs::create_dir_all(&models_dir)?;
    }

    // 1. Check and download Supertonic-3 if missing
    if !check_supertonic_exists() {
        crate::term_log!("[Model Manager] Supertonic-3 model missing. Starting automated download to {:?}...", models_dir);
        download_and_extract_item(
            "Supertonic-3",
            SUPERTONIC_DOWNLOAD_URL,
            &models_dir.join("supertonic-3.zip"),
            &models_dir,
        )?;
        crate::term_log!("[Model Manager] Supertonic-3 model successfully installed!");
    }

    // 2. Check and download OpenVoice-V2 if missing
    if !check_openvoice_exists() {
        crate::term_log!("[Model Manager] OpenVoice-V2 model missing. Starting automated download to {:?}...", models_dir);
        let openvoice_dir = models_dir.join("openvoice-v2");
        if !openvoice_dir.exists() {
            std::fs::create_dir_all(&openvoice_dir)?;
        }
        download_and_extract_item(
            "OpenVoice-V2",
            OPENVOICE_DOWNLOAD_URL,
            &models_dir.join("openvoice-v2.zip"),
            &openvoice_dir,
        )?;
        crate::term_log!("[Model Manager] OpenVoice-V2 model successfully installed!");
    }

    Ok(())
}

fn download_and_extract_item(
    model_name: &str,
    download_url: &str,
    zip_path: &Path,
    extract_to_dir: &Path,
) -> anyhow::Result<()> {
    update_model_status(ModelDownloadStatus {
        status: "downloading".to_string(),
        downloaded_bytes: 0,
        total_bytes: 0,
        percent: 0.0,
        speed_mbps: 0.0,
        message: format!("Connecting to download {model_name}..."),
        model_name: Some(model_name.to_string()),
    });

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3600))
        .build()?;

    let mut response = client.get(download_url).send()?;
    if !response.status().is_success() {
        return Err(anyhow::anyhow!(
            "Failed to download {model_name}: HTTP {}",
            response.status()
        ));
    }

    let total_bytes = response.content_length().unwrap_or(0);
    let mut dest_file = File::create(zip_path)?;

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
                    "Downloading {} ({:.1} MB / {:.1} MB)...",
                    model_name,
                    downloaded_bytes as f32 / (1024.0 * 1024.0),
                    total_bytes as f32 / (1024.0 * 1024.0)
                ),
                model_name: Some(model_name.to_string()),
            });
            last_update_time = Instant::now();
        }
    }
    dest_file.flush()?;
    drop(dest_file);

    crate::term_log!(
        "[Model Manager] {} download complete ({:.1} MB). Extracting...",
        model_name,
        downloaded_bytes as f32 / (1024.0 * 1024.0)
    );

    update_model_status(ModelDownloadStatus {
        status: "extracting".to_string(),
        downloaded_bytes,
        total_bytes,
        percent: 100.0,
        speed_mbps: 0.0,
        message: format!("Extracting {model_name} files..."),
        model_name: Some(model_name.to_string()),
    });

    extract_zip(zip_path, extract_to_dir)?;
    let _ = std::fs::remove_file(zip_path);

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
