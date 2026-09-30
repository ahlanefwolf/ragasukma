use image::DynamicImage;
use std::io::{Cursor, Write};
use std::process::{Command, Stdio};
use std::sync::OnceLock;

/// Cache bahasa yang didukung oleh tesseract pada sistem (misal "eng+ind" atau "eng")
static DETECTED_LANGS: OnceLock<String> = OnceLock::new();

fn get_tesseract_languages() -> &'static str {
    DETECTED_LANGS.get_or_init(|| {
        let output = Command::new("tesseract").arg("--list-langs").output();
        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            let mut has_eng = false;
            let mut has_ind = false;
            for line in text.lines() {
                let trimmed = line.trim();
                if trimmed == "eng" {
                    has_eng = true;
                } else if trimmed == "ind" {
                    has_ind = true;
                }
            }
            if has_eng && has_ind {
                "eng+ind".to_string()
            } else if has_ind {
                "ind".to_string()
            } else {
                "eng".to_string()
            }
        } else {
            "eng".to_string()
        }
    })
}

#[derive(Debug)]
struct WordBox {
    block: i32,
    line: i32,
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    conf: f32,
    text: String,
}

/// Fallback OCR universal menggunakan Tesseract OCR melalui streaming pipe (stdin -> stdout TSV).
/// Bekerja di Linux (X11/Wayland), Windows, dan macOS tanpa file sementara di disk.
pub fn run_tesseract_fallback(
    image: &DynamicImage,
) -> Result<Vec<(String, [i32; 4], f32)>, String> {
    // 1. Encode image ke memory buffer PNG tanpa menulis ke disk
    let mut png_bytes = Vec::new();
    image
        .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
        .map_err(|e| format!("Failed to encode image to PNG buffer: {e}"))?;

    let langs = get_tesseract_languages();

    // 2. Spawn tesseract process dengan input stdin dan output stdout TSV
    let mut child = Command::new("tesseract")
        .arg("stdin")
        .arg("stdout")
        .arg("-l")
        .arg(langs)
        .arg("tsv")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|e| {
            format!(
                "Executable 'tesseract' not found on system: {e}. Please install tesseract (e.g. 'sudo dnf install tesseract tesseract-langpack-eng' or 'sudo apt install tesseract-ocr')."
            )
        })?;

    // 3. Tulis PNG bytes ke child stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&png_bytes)
            .map_err(|e| format!("Failed to send image buffer to tesseract stdin: {e}"))?;
    }

    // 4. Tunggu eksekusi dan baca stdout TSV
    let output = child
        .wait_with_output()
        .map_err(|e| format!("Failed to read tesseract output: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "Tesseract exited with failure status code: {:?}",
            output.status.code()
        ));
    }

    let tsv_text = String::from_utf8_lossy(&output.stdout);
    Ok(parse_tsv_output(&tsv_text))
}

/// Parse TSV format output dari Tesseract dan kelompokkan kata berdekatan menjadi frasa UI
pub fn parse_tsv_output(tsv_text: &str) -> Vec<(String, [i32; 4], f32)> {
    let mut words = Vec::new();

    // Format: level, page_num, block_num, par_num, line_num, word_num, left, top, width, height, conf, text
    for line in tsv_text.lines().skip(1) {
        let cols: Vec<&str> = line.split('\t').collect();
        if cols.len() < 12 {
            continue;
        }

        let level = cols[0];
        // Level 5 = Word level
        if level != "5" {
            continue;
        }

        let conf: f32 = match cols[10].parse() {
            Ok(v) => v,
            Err(_) => continue,
        };

        let text = cols[11].trim();
        // Filter noise: abaikan confidence sangat rendah (< 30.0%) atau teks kosong
        if conf < 30.0 || text.is_empty() {
            continue;
        }

        let block: i32 = cols[2].parse().unwrap_or(0);
        let line_num: i32 = cols[4].parse().unwrap_or(0);
        let x: i32 = cols[6].parse().unwrap_or(0);
        let y: i32 = cols[7].parse().unwrap_or(0);
        let w: i32 = cols[8].parse().unwrap_or(0);
        let h: i32 = cols[9].parse().unwrap_or(0);

        words.push(WordBox {
            block,
            line: line_num,
            x,
            y,
            w,
            h,
            conf,
            text: text.to_string(),
        });
    }

    // Kelompokkan kata-kata berdekatan pada baris yang sama menjadi frasa/elemen UI utuh
    let mut phrases: Vec<(String, [i32; 4], f32)> = Vec::new();
    struct CurrentPhrase {
        block: i32,
        line: i32,
        texts: Vec<String>,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        confs: Vec<f32>,
    }

    let mut current: Option<CurrentPhrase> = None;

    for w in words {
        if let Some(ref mut curr) = current {
            let gap = w.x - curr.x2;
            let same_line = w.block == curr.block && w.line == curr.line;
            let max_gap = (curr.y2 - curr.y1).max(25);

            if same_line && gap >= -5 && gap <= max_gap {
                curr.texts.push(w.text);
                curr.x2 = curr.x2.max(w.x + w.w);
                curr.y1 = curr.y1.min(w.y);
                curr.y2 = curr.y2.max(w.y + w.h);
                curr.confs.push(w.conf);
                continue;
            }

            // Simpan frasa sebelumnya
            let phrase_text = curr.texts.join(" ");
            let bbox = [curr.x1, curr.y1, curr.x2 - curr.x1, curr.y2 - curr.y1];
            let avg_conf = (curr.confs.iter().sum::<f32>() / curr.confs.len() as f32) / 100.0;
            phrases.push((phrase_text, bbox, avg_conf.clamp(0.0, 1.0)));
        }

        current = Some(CurrentPhrase {
            block: w.block,
            line: w.line,
            texts: vec![w.text],
            x1: w.x,
            y1: w.y,
            x2: w.x + w.w,
            y2: w.y + w.h,
            confs: vec![w.conf],
        });
    }

    if let Some(curr) = current {
        let phrase_text = curr.texts.join(" ");
        let bbox = [curr.x1, curr.y1, curr.x2 - curr.x1, curr.y2 - curr.y1];
        let avg_conf = (curr.confs.iter().sum::<f32>() / curr.confs.len() as f32) / 100.0;
        phrases.push((phrase_text, bbox, avg_conf.clamp(0.0, 1.0)));
    }

    phrases
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tsv_output_grouping() {
        let sample_tsv = "\
level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext
1\t1\t0\t0\t0\t0\t0\t0\t1920\t1080\t-1\t
5\t1\t1\t1\t1\t1\t100\t50\t40\t15\t95.0\tSign
5\t1\t1\t1\t1\t2\t145\t50\t20\t15\t90.0\tIn
5\t1\t1\t1\t1\t3\t500\t50\t50\t15\t85.0\tCancel
";
        let results = parse_tsv_output(sample_tsv);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, "Sign In");
        assert_eq!(results[0].1, [100, 50, 65, 15]);
        assert_eq!(results[1].0, "Cancel");
        assert_eq!(results[1].1, [500, 50, 50, 15]);
    }
}
