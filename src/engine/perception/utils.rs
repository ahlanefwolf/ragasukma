#[cfg(any(target_os = "macos", target_os = "linux"))]
use xcap::Monitor;

/// Gets coordinate scale divisor so visual scan results
/// sync with tool klik mouse in masing-masing sistem operasi.
pub fn get_coordinate_scale_divisor(orig_w: u32, orig_h: u32) -> f32 {
    #[cfg(target_os = "macos")]
    {
        let _ = (orig_w, orig_h);
        // On macOS Retina, captured images are in Physical Pixels (e.g. 2880x1800),
        // while click utilities (cliclick & AppleScript) take inputs in Points (1440x900).
        if let Ok(monitors) = Monitor::all() {
            let primary_or_first = monitors
                .iter()
                .find(|m| m.is_primary())
                .or_else(|| monitors.first());
            if let Some(p) = primary_or_first {
                let s = p.scale_factor();
                if s > 0.5 {
                    return s;
                }
            }
        }
        2.0 // Standar Retina display fallback
    }

    #[cfg(target_os = "windows")]
    {
        // Windows in Per-Monitor V2 DPI Aware mode operates purely in 1:1 Physical Pixels
        let _ = (orig_w, orig_h);
        1.0
    }

    #[cfg(target_os = "linux")]
    {
        let _ = orig_h;
        let is_wayland = std::env::var("WAYLAND_DISPLAY").is_ok()
            || std::env::var("XDG_SESSION_TYPE")
                .map(|s| s.eq_ignore_ascii_case("wayland"))
                .unwrap_or(false);

        if is_wayland && orig_w > 0 {
            // 1. Check Sway if session using Sway compositor: take value scale direct
            if let Ok(output) = std::process::Command::new("swaymsg")
                .args(["-t", "get_outputs", "--raw"])
                .output()
            {
                if output.status.success() {
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                        if let Some(arr) = val.as_array() {
                            let out_opt = arr
                                .iter()
                                .find(|o| o["focused"].as_bool().unwrap_or(false))
                                .or_else(|| arr.first());
                            if let Some(out) = out_opt {
                                if let Some(scale) = out["scale"].as_f64() {
                                    if scale > 0.5 {
                                        return scale as f32;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 2. Check Hyprland if session using Hyprland compositor: take value scale direct
            if let Ok(output) = std::process::Command::new("hyprctl")
                .args(["monitors", "-j"])
                .output()
            {
                if output.status.success() {
                    if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&output.stdout) {
                        if let Some(arr) = val.as_array() {
                            let mon_opt = arr
                                .iter()
                                .find(|m| m["focused"].as_bool().unwrap_or(false))
                                .or_else(|| arr.first());
                            if let Some(mon) = mon_opt {
                                if let Some(scale) = mon["scale"].as_f64() {
                                    if scale > 0.5 {
                                        return scale as f32;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // 3. Check GNOME Wayland via gsettings
            if let Ok(output) = std::process::Command::new("gsettings")
                .args(["get", "org.gnome.desktop.interface", "scaling-factor"])
                .output()
            {
                if output.status.success() {
                    let out_str = String::from_utf8_lossy(&output.stdout);
                    if let Some(num_str) = out_str.split_whitespace().last() {
                        if let Ok(factor) = num_str.parse::<f32>() {
                            if factor > 1.0 {
                                return factor;
                            }
                        }
                    }
                }
            }
            if let Ok(output) = std::process::Command::new("gsettings")
                .args(["get", "org.gnome.desktop.interface", "text-scaling-factor"])
                .output()
            {
                if output.status.success() {
                    let out_str = String::from_utf8_lossy(&output.stdout);
                    if let Ok(factor) = out_str.trim().parse::<f32>() {
                        if (factor - 1.0).abs() > 0.05 && factor > 0.5 {
                            return factor;
                        }
                    }
                }
            }

            // 4. Check KDE Plasma Wayland via kreadconfig6 / kreadconfig5
            for cmd in ["kreadconfig6", "kreadconfig5"] {
                if let Ok(output) = std::process::Command::new(cmd)
                    .args(["--group", "KScreen", "--key", "ScaleFactor"])
                    .output()
                {
                    if output.status.success() {
                        let out_str = String::from_utf8_lossy(&output.stdout);
                        if let Ok(factor) = out_str.trim().parse::<f32>() {
                            if (factor - 1.0).abs() > 0.05 && factor > 0.5 {
                                return factor;
                            }
                        }
                    }
                }
            }

            // 5. Check monitor xcap: in XWayland, xcap reports logical desktop rect
            if let Ok(monitors) = Monitor::all() {
                let mon_opt = monitors
                    .iter()
                    .find(|m| m.is_primary())
                    .or_else(|| monitors.first());
                if let Some(m) = mon_opt {
                    let mon_w = m.width();
                    if mon_w > 0 {
                        let ratio = orig_w as f32 / mon_w as f32;
                        if ratio > 0.5 && (ratio - 1.0).abs() > 0.05 {
                            return ratio;
                        }
                    }
                    let s = m.scale_factor();
                    if s > 0.5 && (s - 1.0).abs() > 0.05 {
                        return s;
                    }
                }
            }

            // 6. Check environment variables (GDK_SCALE, QT_SCALE_FACTOR)
            if let Ok(gdk) = std::env::var("GDK_SCALE") {
                if let Ok(s) = gdk.trim().parse::<f32>() {
                    if s > 0.5 {
                        return s;
                    }
                }
            }
            if let Ok(qt) = std::env::var("QT_SCALE_FACTOR") {
                if let Ok(s) = qt.trim().parse::<f32>() {
                    if s > 0.5 {
                        return s;
                    }
                }
            }
        } else {
            // Mode X11 (i3, bspwm, XFCE, Openbox, dll): check xrdb for DPI scaling
            if let Ok(output) = std::process::Command::new("xrdb").args(["-query"]).output() {
                if output.status.success() {
                    let out_str = String::from_utf8_lossy(&output.stdout);
                    for line in out_str.lines() {
                        if line.to_lowercase().starts_with("xft.dpi:") {
                            if let Some(dpi_str) = line.split(':').nth(1) {
                                if let Ok(dpi) = dpi_str.trim().parse::<f32>() {
                                    if dpi > 30.0 {
                                        let ratio = dpi / 96.0;
                                        if (ratio - 1.0).abs() > 0.05 {
                                            return ratio;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Linux X11 or 100% scale
        1.0
    }

    #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
    {
        let _ = (orig_w, orig_h);
        1.0
    }
}

pub fn is_button_like_text(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }

    // 1. Calculator & numeric keypad digits (0-9, 00, etc.)
    if trimmed.chars().all(|c| c.is_ascii_digit()) && trimmed.len() <= 4 {
        return true;
    }

    // 2. Calculator math & function symbols
    let math_symbols = [
        "+", "-", "*", "/", "x", "×", "÷", "=", "%", "±", "+/-", ".", ",", "c", "ce", "del", "mc",
        "mr", "m+", "m-", "ms", "m", "sqrt", "√", "x²", "1/x", "exp", "mod", "log", "ln", "(", ")",
        "pi", "π", "e", "deg", "rad",
    ];
    let lower = trimmed.to_lowercase();
    for sym in &math_symbols {
        if lower == *sym {
            return true;
        }
    }

    // 3. Very short action or navigation tokens (1-2 characters)
    if trimmed.len() <= 2
        && trimmed
            .chars()
            .all(|c| c.is_alphanumeric() || c.is_ascii_punctuation())
    {
        return true;
    }

    // 4. Universal UI button heuristics (language-agnostic):
    // - Buttons are concise labels (typically 1 to 4 words, length <= 25 chars)
    // - Cannot contain multiline breaks or paragraph punctuation (e.g. '.', ';', '?', '!')
    // - Cannot be a long descriptive sentence
    let char_count = trimmed.chars().count();
    let word_count = trimmed.split_whitespace().count();

    if char_count <= 25
        && word_count <= 4
        && !trimmed.contains('\n')
        && !trimmed.ends_with('.')
        && !trimmed.ends_with('?')
        && !trimmed.ends_with('!')
        && !trimmed.ends_with(';')
    {
        return true;
    }

    false
}

/// Calculates similarity score between two texts (0.0 to 1.0)
/// using kombinasi token normalization, substring containment, Levenshtein distance,
/// and token-level similarity for mendukung frasa with typo / variasi word.
pub fn string_similarity(a: &str, b: &str) -> f32 {
    let a_clean: String = a
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase();
    let b_clean: String = b
        .chars()
        .filter(|c| c.is_alphanumeric())
        .collect::<String>()
        .to_lowercase();

    if a_clean.is_empty() && b_clean.is_empty() {
        return 1.0;
    }
    if a_clean.is_empty() || b_clean.is_empty() {
        return 0.0;
    }
    if a_clean == b_clean {
        return 1.0;
    }

    // 1. Direct whole-string Levenshtein similarity
    let lev = levenshtein_distance(&a_clean, &b_clean);
    let max_len = a_clean.chars().count().max(b_clean.chars().count()) as f32;
    let whole_sim = (1.0 - (lev as f32 / max_len)).max(0.0);

    // 2. Direct clean substring containment
    let sub_sim = if a_clean.contains(&b_clean) || b_clean.contains(&a_clean) {
        let min_len = a_clean.len().min(b_clean.len()) as f32;
        let max_len = a_clean.len().max(b_clean.len()) as f32;
        if min_len >= 3.0 && (min_len / max_len) >= 0.3 {
            (min_len / max_len).max(0.75)
        } else {
            min_len / max_len
        }
    } else {
        0.0
    };

    // 3. Token-level similarity for frasa sub-query (for example: "tagian listrik" vs "Bayar tagihan listrik PLN")
    let token_sim = token_similarity(a, b);

    whole_sim.max(sub_sim).max(token_sim)
}

fn token_similarity(a: &str, b: &str) -> f32 {
    let extract_tokens = |s: &str| -> Vec<String> {
        s.split(|c: char| !c.is_alphanumeric())
            .filter_map(|w| {
                let clean: String = w.chars().filter(|c| c.is_alphanumeric()).collect();
                if clean.is_empty() {
                    None
                } else {
                    Some(clean.to_lowercase())
                }
            })
            .collect()
    };

    let a_tokens = extract_tokens(a);
    let b_tokens = extract_tokens(b);

    if a_tokens.is_empty() || b_tokens.is_empty() {
        return 0.0;
    }

    let cov_a = score_token_coverage(&a_tokens, &b_tokens);
    let cov_b = score_token_coverage(&b_tokens, &a_tokens);

    cov_a.max(cov_b)
}

fn score_token_coverage(source: &[String], target: &[String]) -> f32 {
    if source.is_empty() || target.is_empty() {
        return 0.0;
    }

    // If source only 1 word short (<= 2 huruf, for example "in", "to"),
    // do not match against long targets containing many words.
    if source.len() == 1 && source[0].chars().count() <= 2 && target.len() >= 3 {
        return 0.0;
    }

    let mut total = 0.0;
    for s in source {
        let s_len = s.chars().count();
        let mut best = 0.0f32;
        for t in target {
            if s == t {
                best = 1.0;
                break;
            }
            let t_len = t.chars().count();
            let max_len = s_len.max(t_len);
            if max_len == 0 {
                continue;
            }
            // For very short tokens (<= 2 chars), only allow exact match
            if s_len <= 2 || t_len <= 2 {
                continue;
            }
            let lev = levenshtein_distance(s, t);
            let sim = 1.0 - (lev as f32 / max_len as f32);
            if sim > best {
                best = sim;
            }
        }
        total += best.max(0.0);
    }
    total / source.len() as f32
}

fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let v1: Vec<char> = s1.chars().collect();
    let v2: Vec<char> = s2.chars().collect();
    let len1 = v1.len();
    let len2 = v2.len();

    if len1 == 0 {
        return len2;
    }
    if len2 == 0 {
        return len1;
    }

    let mut prev_row: Vec<usize> = (0..=len2).collect();
    let mut curr_row: Vec<usize> = vec![0; len2 + 1];

    for (i, &ch1) in v1.iter().enumerate() {
        curr_row[0] = i + 1;
        for (j, &ch2) in v2.iter().enumerate() {
            let cost = if ch1 == ch2 { 0 } else { 1 };
            curr_row[j + 1] = (curr_row[j] + 1)
                .min(prev_row[j + 1] + 1)
                .min(prev_row[j] + cost);
        }
        prev_row.copy_from_slice(&curr_row);
    }

    prev_row[len2]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_scale_divisor_handles_dimensions() {
        let div_zero = get_coordinate_scale_divisor(0, 0);
        assert!(div_zero >= 1.0);

        #[cfg(target_os = "linux")]
        {
            let div_1920 = get_coordinate_scale_divisor(1920, 1080);
            let div_1536 = get_coordinate_scale_divisor(1536, 864);
            assert!(div_1920 >= 1.0);
            assert!(div_1536 >= 1.0);
        }
    }

    #[test]
    fn test_is_button_like_text() {
        assert!(is_button_like_text("OK"));
        assert!(is_button_like_text("Save"));
        assert!(is_button_like_text("Batal"));
        assert!(is_button_like_text("送信")); // Japanese: Submit
        assert!(is_button_like_text("Guardar")); // Spanish: Save
        assert!(is_button_like_text("7"));
        assert!(is_button_like_text("+"));
        assert!(!is_button_like_text(
            "This is a very long descriptive paragraph that should not be detected as a button."
        ));
        assert!(!is_button_like_text("Click here to proceed to the next step."));
    }

    #[test]
    fn test_string_similarity_phonetic_and_typo() {
        // Real-world cases faced by users:
        // "skill kode.rd" vs "skillcode.md"
        let sim = string_similarity("skill kode.rd", "skillcode.md");
        assert!(
            sim >= 0.70,
            "Similarity 'skill kode.rd' vs 'skillcode.md' should be >= 70%, got: {sim}"
        );

        // Music case: "tumhiana" vs "Tum Hi Aana"
        let sim_music = string_similarity("tumhiana", "Tum Hi Aana");
        assert!(
            sim_music >= 0.70,
            "Similarity 'tumhiana' vs 'Tum Hi Aana' should be >= 70%, got: {sim_music}"
        );

        // Not similar: "Save" vs "Cancel"
        let sim_diff = string_similarity("Simpan", "Batal");
        assert!(sim_diff < 0.30);
    }

    #[test]
    fn test_string_similarity_token_and_subphrase() {
        // Sub-query with typo vs long to-do title
        let sim_todo = string_similarity("tagian listrik", "Bayar tagihan listrik PLN");
        assert!(
            sim_todo >= 0.70,
            "Similarity 'tagian listrik' vs 'Bayar tagihan listrik PLN' should be >= 70%, got: {sim_todo}"
        );

        // Partial exact sub-query
        let sim_bayar = string_similarity("bayar listrik", "Bayar tagihan listrik PLN");
        assert!(
            sim_bayar >= 0.70,
            "Similarity 'bayar listrik' vs 'Bayar tagihan listrik PLN' should be >= 70%, got: {sim_bayar}"
        );

        // Local music: song title with typo vs full audio filename
        let sim_music = string_similarity("sepia", "01 - Sheila On 7 - Sephia (Remastered).mp3");
        assert!(
            sim_music >= 0.70,
            "Similarity 'sepia' vs '01 - Sheila On 7 - Sephia (Remastered).mp3' should be >= 70%, got: {sim_music}"
        );
    }
}
