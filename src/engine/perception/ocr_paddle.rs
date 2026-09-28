use image::DynamicImage;
use serde::Deserialize;
use std::fs;
use std::process::Command;

#[derive(Deserialize)]
struct PaddleResponse {
    boxes: Option<Vec<(String, [i32; 4], f32)>>,
    error: Option<String>,
}

/// Fallback OCR universal menggunakan PaddleOCR melalui Python subprocess.
/// Bekerja di Windows, Linux (X11/Wayland), dan macOS.
pub fn run_paddleocr_fallback(image: &DynamicImage) -> Result<Vec<(String, [i32; 4], f32)>, String> {
    let temp_dir = std::env::temp_dir();
    let file_id = format!(
        "ragasukma_ocr_{}_{}.png",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0)
    );
    let temp_img_path = temp_dir.join(file_id);

    image
        .save(&temp_img_path)
        .map_err(|e| format!("Gagal menyimpan screenshot sementara untuk OCR: {e}"))?;

    let py_script = r#"
import sys, json, os, warnings
warnings.filterwarnings('ignore')

os.environ['GLOG_minloglevel'] = '3'
os.environ['FLAGS_minloglevel'] = '3'
os.environ['PADDLE_PDX_DISABLE_MODEL_SOURCE_CHECK'] = 'True'

# Alihkan stdout sementara ke stderr selama import dan load model agar log C++/OneDNN tidak mencemari JSON
real_stdout = sys.stdout
sys.stdout = sys.stderr

try:
    from paddleocr import PaddleOCR
except ImportError:
    sys.stdout = real_stdout
    print("---JSON_START---")
    print(json.dumps({"error": "PADDLEOCR_NOT_INSTALLED"}))
    print("---JSON_END---")
    sys.exit(0)

try:
    img_path = sys.argv[1]
    ocr = PaddleOCR(use_doc_orientation_classify=False, use_doc_unwarping=False, use_textline_orientation=False)
    results = ocr.predict(img_path)
    output = []
    for res in results:
        # PaddleOCR v3.7 / PP-OCRv6 dict format
        if isinstance(res, dict):
            boxes = res.get('rec_boxes', [])
            texts = res.get('rec_texts', [])
            scores = res.get('rec_scores', [])
            for box, text, score in zip(boxes, texts, scores):
                min_x, min_y, max_x, max_y = [int(v) for v in box]
                output.append([str(text), [min_x, min_y, max_x - min_x, max_y - min_y], float(score)])
        # Legacy format fallback
        elif hasattr(res, 'boxes') and res.boxes is not None:
            boxes = res.boxes
            texts = getattr(res, 'rec_text', [])
            scores = getattr(res, 'rec_score', [])
            for box, text, score in zip(boxes, texts, scores):
                xs = [p[0] for p in box]
                ys = [p[1] for p in box]
                min_x, max_x = int(min(xs)), int(max(xs))
                min_y, max_y = int(min(ys)), int(max(ys))
                output.append([str(text), [min_x, min_y, max_x - min_x, max_y - min_y], float(score)])
        elif isinstance(res, list):
            for item in res:
                if len(item) >= 2 and isinstance(item[0], list):
                    box = item[0]
                    text_info = item[1]
                    text = text_info[0] if isinstance(text_info, (list, tuple)) else str(text_info)
                    score = float(text_info[1]) if isinstance(text_info, (list, tuple)) and len(text_info) > 1 else 0.9
                    xs = [p[0] for p in box]
                    ys = [p[1] for p in box]
                    min_x, max_x = int(min(xs)), int(max(xs))
                    min_y, max_y = int(min(ys)), int(max(ys))
                    output.append([str(text), [min_x, min_y, max_x - min_x, max_y - min_y], float(score)])

    sys.stdout = real_stdout
    print("---JSON_START---")
    print(json.dumps({"boxes": output}))
    print("---JSON_END---")
except Exception as e:
    sys.stdout = real_stdout
    print("---JSON_START---")
    print(json.dumps({"error": str(e)}))
    print("---JSON_END---")
"#;

    let path_str = temp_img_path.to_string_lossy().to_string();

    // Susun kandidat interpreter python dengan prioritas:
    // 1. Virtual environments khusus PaddleOCR (misal ~/.venvs/paddleocr, ~/.venv, .venv)
    // 2. Binary eksplisit python3.12 / python3.11 / python3.10
    // 3. Fallback sistem (python3, python, py) jika versinya cocok (>= 3.8 dan <= 3.12)
    let mut candidate_execs = Vec::new();

    if let Ok(home) = std::env::var("HOME") {
        let venv_paddle = std::path::Path::new(&home).join(".venvs/paddleocr/bin/python");
        if venv_paddle.exists() {
            candidate_execs.push(venv_paddle.to_string_lossy().to_string());
        }
        let venv_alt = std::path::Path::new(&home).join(".venv/bin/python");
        if venv_alt.exists() {
            candidate_execs.push(venv_alt.to_string_lossy().to_string());
        }
    }
    if let Ok(cur_dir) = std::env::current_dir() {
        let local_venv = cur_dir.join(".venv/bin/python");
        if local_venv.exists() {
            candidate_execs.push(local_venv.to_string_lossy().to_string());
        }
    }

    // Binary versi spesifik yang didukung resmi oleh PaddlePaddle
    for ver in ["python3.12", "python3.11", "python3.10", "python3.9", "python3.8"] {
        candidate_execs.push(ver.to_string());
    }

    // Cek python default sistem: jika versinya bukan 3.8-3.12, jangan prioritaskan
    for sys_cmd in ["python3", "python", "py"] {
        candidate_execs.push(sys_cmd.to_string());
    }

    let mut last_err = String::new();
    let mut raw_output = None;

    // Filter dan pilih interpreter yang valid & compatible
    let mut tested_execs = Vec::new();
    for exe in candidate_execs {
        if tested_execs.contains(&exe) {
            continue;
        }
        tested_execs.push(exe.clone());

        // Validasi versi Python kandidat: hanya izinkan versi 3.8 <= version <= 3.12
        // kecuali jika itu executable venv eksplisit yang sudah kita temukan
        let is_venv = exe.contains(".venvs") || exe.contains(".venv");
        if !is_venv {
            let ver_check = Command::new(&exe)
                .args(["-c", "import sys; print(f'{sys.version_info.major}.{sys.version_info.minor}')"])
                .output();

            match ver_check {
                Ok(out) if out.status.success() => {
                    let ver = String::from_utf8_lossy(&out.stdout).trim().to_string();
                    let parts: Vec<&str> = ver.split('.').collect();
                    if parts.len() == 2 {
                        let major: i32 = parts[0].parse().unwrap_or(0);
                        let minor: i32 = parts[1].parse().unwrap_or(0);
                        // PaddleOCR stabil di 3.8 - 3.12 (Python 3.13 & 3.14 belum ada wheel paddlepaddle)
                        if major != 3 || !(8..=12).contains(&minor) {
                            continue;
                        }
                    }
                }
                _ => continue,
            }
        }

        match Command::new(&exe).args(["-c", py_script, &path_str]).output() {
            Ok(out) if out.status.success() => {
                let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                if !stdout.trim().is_empty() {
                    raw_output = Some(stdout);
                    break;
                }
            }
            Ok(out) => {
                last_err = String::from_utf8_lossy(&out.stderr).to_string();
            }
            Err(e) => {
                last_err = e.to_string();
            }
        }
    }

    // Selalu bersihkan file temporary
    let _ = fs::remove_file(&temp_img_path);

    let output_str = match raw_output {
        Some(s) => s,
        None => {
            return Err(format!(
                "Python tidak ditemukan atau gagal dieksekusi. Detail: {last_err}"
            ));
        }
    };

    // Ekstrak string JSON di antara delimiter ---JSON_START--- dan ---JSON_END---
    // Hal ini untuk mencegah tercampurnya log C++ OneDNN / warning bawaan PaddlePaddle
    let json_content = if let (Some(start_idx), Some(end_idx)) = (
        output_str.find("---JSON_START---"),
        output_str.find("---JSON_END---"),
    ) {
        if start_idx < end_idx {
            &output_str[start_idx + "---JSON_START---".len()..end_idx]
        } else {
            output_str.as_str()
        }
    } else {
        output_str.as_str()
    };

    match serde_json::from_str::<PaddleResponse>(json_content.trim()) {
        Ok(resp) => {
            if let Some(err) = resp.error {
                if err == "PADDLEOCR_NOT_INSTALLED" {
                    return Err(
                        "Library 'paddleocr' belum terpasang. Silakan jalankan setup di ~/.venvs/paddleocr."
                            .to_string(),
                    );
                }
                return Err(format!("PaddleOCR Python Error: {err}"));
            }

            if let Some(boxes) = resp.boxes {
                return Ok(boxes);
            }
            Err("PaddleOCR mengembalikan format kosong tanpa deteksi kotak teks.".to_string())
        }
        Err(parse_err) => {
            Err(format!(
                "Output PaddleOCR tidak sesuai format JSON yang diharapkan ({parse_err}).\nIsi yang diparsing:\n{}\n\nOutput Mentah:\n{}",
                json_content.trim(),
                output_str.trim()
            ))
        }
    }
}
