use regex::Regex;
use std::sync::LazyLock;

static RE_SKILL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[SKILL:[^\]]+\]").expect("valid regex"));

/// Membersihkan teks dari teks aksi roleplay ekspresi tanpa memotong judul atau kata penting
pub fn clean_ai_text(text: &str) -> String {
    let text_clean = RE_SKILL.replace_all(text, "");

    let mut result = String::new();
    let mut chars = text_clean.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '[' {
            let mut inside = String::new();
            inside.push('[');
            while let Some(&next_c) = chars.peek() {
                chars.next();
                inside.push(next_c);
                if next_c == ']' {
                    break;
                }
            }
            // Tag aksi tidak boleh dimodifikasi/dihapus
            result.push_str(&inside);
            continue;
        }

        if c == '*' {
            let mut inside = String::new();
            let mut closed = false;
            while let Some(&next_c) = chars.peek() {
                chars.next();
                if next_c == '*' {
                    closed = true;
                    if chars.peek() == Some(&'*') {
                        chars.next();
                    }
                    break;
                }
                inside.push(next_c);
            }

            if !closed {
                result.push('*');
                result.push_str(&inside);
            }
            continue;
        }

        result.push(c);
    }

    result.trim().to_string()
}
