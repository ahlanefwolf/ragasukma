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
import sys, json

try:
    from paddleocr import PaddleOCR
except ImportError:
    print(json.dumps({"error": "PADDLEOCR_NOT_INSTALLED"}))
    sys.exit(0)

try:
    img_path = sys.argv[1]
    ocr = PaddleOCR(use_doc_orientation_classify=False, use_doc_unwarping=False, use_textline_orientation=False)
    results = ocr.predict(img_path)
    output = []
    for res in results:
        # PaddleOCR v3 / v6 output format
        if hasattr(res, 'boxes') and res.boxes is not None:
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
    print(json.dumps({"boxes": output}))
except Exception as e:
    print(json.dumps({"error": str(e)}))
"#;

    let path_str = temp_img_path.to_string_lossy().to_string();
    let python_execs = ["python3", "python", "py"];
    let mut last_err = String::new();
    let mut raw_output = None;

    for exe in python_execs {
        match Command::new(exe).args(["-c", py_script, &path_str]).output() {
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

    if let Ok(resp) = serde_json::from_str::<PaddleResponse>(output_str.trim()) {
        if let Some(err) = resp.error {
            if err == "PADDLEOCR_NOT_INSTALLED" {
                return Err(
                    "Library 'paddleocr' belum terpasang. Silakan jalankan 'pip install paddleocr' pada terminal."
                        .to_string(),
                );
            }
            return Err(format!("PaddleOCR Error: {err}"));
        }

        if let Some(boxes) = resp.boxes {
            return Ok(boxes);
        }
    }

    Err("Output PaddleOCR tidak sesuai format JSON yang diharapkan.".to_string())
}
