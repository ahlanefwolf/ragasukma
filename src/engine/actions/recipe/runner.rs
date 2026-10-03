use std::collections::HashMap;
use std::thread;
use std::time::{Duration, Instant};

use crate::services::browser_session_service::{
    BrowserDomSnapshot, DomElementInfo, get_latest_snapshot,
};

use super::types::{ClickTarget, RecipeStep, find_recipe};

/// Menghitung kesamaan sederhana antara dua teks (case-insensitive substring & token match)
pub fn match_element_score(element_text: &str, candidate: &str) -> f32 {
    let el = element_text.trim().to_lowercase();
    let cand = candidate.trim().to_lowercase();
    if el.is_empty() || cand.is_empty() {
        return 0.0;
    }
    if el == cand {
        return 1.0;
    }
    if el.contains(&cand) || cand.contains(&el) {
        return 0.85;
    }
    // Token overlap match
    let el_tokens: Vec<&str> = el.split_whitespace().collect();
    let cand_tokens: Vec<&str> = cand.split_whitespace().collect();
    let mut matches = 0;
    for c in &cand_tokens {
        if el_tokens.contains(c) {
            matches += 1;
        }
    }
    if !cand_tokens.is_empty() && matches > 0 {
        return (matches as f32) / (cand_tokens.len() as f32) * 0.75;
    }
    0.0
}

/// Cari target klik terbaik dengan prioritas:
/// Cari target klik terbaik dengan prioritas:
/// 1. Cari dulu di `snapshot.buttons` menggunakan selector_primary.
/// 2. Jika tidak ditemukan, coba cari setiap kandidat di `fallback_selectors` pada buttons.
/// 3. Jika belum ditemukan, cari kecocokan label di `snapshot.buttons` (Buttons First).
/// 4. Jika masih tidak ada yang cocok di buttons, cari di `snapshot.texts` (Texts Fallback).
pub fn resolve_best_click_target(
    snap: Option<&BrowserDomSnapshot>,
    selector_primary: &str,
    fallback_selectors: &[String],
    target_text: Option<&str>,
) -> Option<ClickTarget> {
    let is_css_selector = |sel: &str| {
        (sel.starts_with('.')
            || sel.starts_with('#')
            || sel.starts_with('[')
            || sel.contains('>')
            || sel.contains(' ')
            || sel.contains(':')
            || sel.contains('.'))
            && sel != "button"
            && sel != "a"
            && sel != "div"
    };

    if let Some(s) = snap {
        // --- 1. PRIMARY SELECTOR IN SNAPSHOT BUTTONS ---
        if !selector_primary.is_empty() && is_css_selector(selector_primary) {
            if let Some(matched) = s.buttons.iter().find(|b| {
                b.selector.as_deref().unwrap_or("").eq_ignore_ascii_case(selector_primary)
            }) {
                if let Some(ref sel) = matched.selector {
                    return Some(ClickTarget::Button(sel.clone()));
                }
            }
        }

        // --- 1b. FALLBACK SELECTORS IN SNAPSHOT BUTTONS (Multi-Strategy) ---
        for fb in fallback_selectors {
            if !fb.is_empty() && is_css_selector(fb) {
                if let Some(matched) = s.buttons.iter().find(|b| {
                    b.selector.as_deref().unwrap_or("").eq_ignore_ascii_case(fb)
                }) {
                    if let Some(ref sel) = matched.selector {
                        return Some(ClickTarget::Button(sel.clone()));
                    }
                }
            }
        }

        // --- 1c. TARGET_TEXT PADA BUTTONS ---
        if let Some(txt) = target_text {
            let clean_target = txt.trim();
            if !clean_target.is_empty() {
                let mut best_button_score = 0.0;
                let mut best_button_selector = None;
                for b in &s.buttons {
                    let combined = format!(
                        "{} {}",
                        b.text.as_deref().unwrap_or(""),
                        b.aria_label.as_deref().unwrap_or("")
                    );
                    let score = match_element_score(&combined, clean_target);
                    if score > best_button_score {
                        best_button_score = score;
                        best_button_selector = b.selector.clone();
                    }
                }

                if best_button_score >= 0.5 {
                    if let Some(sel) = best_button_selector {
                        return Some(ClickTarget::Button(sel));
                    }
                }
            }
        }

        // --- 2. TEXTS FALLBACK (Gunakan target_text untuk klik teks) ---
        if let Some(txt) = target_text {
            let clean_target = txt.trim();
            if !clean_target.is_empty() {
                let mut best_text_score = 0.0;
                let mut best_matched_text = None;

                for text_item in &s.texts {
                    let score = match_element_score(text_item, clean_target);
                    if score > best_text_score {
                        best_text_score = score;
                        best_matched_text = Some(text_item.clone());
                    }
                }

                if best_text_score >= 0.5 {
                    if let Some(matched) = best_matched_text {
                        return Some(ClickTarget::Text(matched));
                    }
                }
            }
        }
    }

    // --- 3. DEFAULT FALLBACK ---
    if !selector_primary.is_empty() && is_css_selector(selector_primary) {
        Some(ClickTarget::Button(selector_primary.to_string()))
    } else if let Some(fb) = fallback_selectors.iter().find(|s| is_css_selector(s)) {
        Some(ClickTarget::Button(fb.clone()))
    } else if let Some(txt) = target_text {
        Some(ClickTarget::Text(txt.to_string()))
    } else if !selector_primary.is_empty() {
        Some(ClickTarget::Button(selector_primary.to_string()))
    } else {
        None
    }
}

/// Cari selector elemen terbaik dari snapshot aktif jika selector utama tidak ditemukan atau berubah (khusus input/elemen spesifik)
pub fn resolve_best_element_selector(
    elements: &[DomElementInfo],
    selector_primary: &str,
    fallback_selectors: &[String],
    fallback_candidates: &[String],
) -> Option<String> {
    if !selector_primary.is_empty() {
        if let Some(matched) = elements.iter().find(|e| {
            e.selector.as_deref().unwrap_or("").eq_ignore_ascii_case(selector_primary)
        }) {
            if let Some(ref s) = matched.selector {
                return Some(s.clone());
            }
        }
    }

    // Cek setiap selector alternatif di fallback_selectors
    for fb in fallback_selectors {
        if !fb.is_empty() {
            if let Some(matched) = elements.iter().find(|e| {
                e.selector.as_deref().unwrap_or("").eq_ignore_ascii_case(fb)
            }) {
                if let Some(ref s) = matched.selector {
                    return Some(s.clone());
                }
            }
        }
    }

    let mut best_score = 0.0;
    let mut best_selector = None;

    for el in elements {
        let combined_label = format!(
            "{} {} {}",
            el.text.as_deref().unwrap_or(""),
            el.placeholder.as_deref().unwrap_or(""),
            el.aria_label.as_deref().unwrap_or("")
        );

        for cand in fallback_candidates {
            let score = match_element_score(&combined_label, cand);
            if score > best_score {
                best_score = score;
                best_selector = el.selector.clone();
            }
        }
    }

    if best_score >= 0.5 {
        best_selector
    } else if !selector_primary.is_empty() {
        Some(selector_primary.to_string())
    } else {
        fallback_selectors.first().cloned()
    }
}

/// Tunggu hingga browser melaporkan snapshot baru atau halaman sesuai domain target termuat
pub fn wait_for_target_page(expected_domain: &str, previous_ts: u64, timeout_secs: u64) -> Option<BrowserDomSnapshot> {
    let start = Instant::now();
    let timeout = Duration::from_secs(timeout_secs);

    while start.elapsed() < timeout {
        if let Some(snap) = get_latest_snapshot() {
            let domain_match = expected_domain.is_empty()
                || snap.domain.to_lowercase().contains(&expected_domain.to_lowercase())
                || snap.url.to_lowercase().contains(&expected_domain.to_lowercase());

            if snap.timestamp > previous_ts && domain_match {
                crate::term_log!(
                    "🌐 [Recipe Runner] Page ready: '{}' (url: {})",
                    snap.title,
                    snap.url
                );
                return Some(snap);
            }
        }
        thread::sleep(Duration::from_millis(50));
    }
    get_latest_snapshot()
}

/// Menunggu secara reaktif hingga elemen target untuk step ini muncul di snapshot DOM.
/// Polling hingga timeout (misal 6 detik). Jika langsung ada, lanjut seketika tanpa jeda!
pub fn wait_for_target_ready(step: &RecipeStep, timeout_secs: u64) -> Option<BrowserDomSnapshot> {
    let start = Instant::now();
    let max_dur = Duration::from_secs(timeout_secs);

    while start.elapsed() < max_dur {
        if let Some(snap) = get_latest_snapshot() {
            let found = if step.action == "type" {
                snap.inputs.iter().any(|inp| {
                    let sel_match = !step.selector.is_empty()
                        && inp
                            .selector
                            .as_deref()
                            .unwrap_or("")
                            .eq_ignore_ascii_case(&step.selector);
                    let name_match = inp
                        .name
                        .as_deref()
                        .map(|n| step.selector.contains(n))
                        .unwrap_or(false);
                    let ph_match = step
                        .target_text
                        .as_deref()
                        .map(|target| {
                            inp.placeholder
                                .as_deref()
                                .unwrap_or("")
                                .to_lowercase()
                                .contains(&target.to_lowercase())
                        })
                        .unwrap_or(false);
                    let fb_match = step.fallback_selectors.iter().any(|fb| {
                        !fb.is_empty()
                            && inp
                                .selector
                                .as_deref()
                                .unwrap_or("")
                                .eq_ignore_ascii_case(fb)
                    });
                    sel_match || name_match || ph_match || fb_match
                })
            } else {
                let btn_match = snap.buttons.iter().any(|b| {
                    let sel_match = !step.selector.is_empty()
                        && b.selector
                            .as_deref()
                            .unwrap_or("")
                            .eq_ignore_ascii_case(&step.selector);
                    let text_match = step
                        .target_text
                        .as_deref()
                        .map(|target| {
                            let tl = target.to_lowercase();
                            b.text
                                .as_deref()
                                .unwrap_or("")
                                .to_lowercase()
                                .contains(&tl)
                                || b.aria_label
                                    .as_deref()
                                    .unwrap_or("")
                                    .to_lowercase()
                                    .contains(&tl)
                        })
                        .unwrap_or(false);
                    sel_match || text_match
                });
                let text_match = snap.texts.iter().any(|t| {
                    let tl = t.to_lowercase();
                    step.target_text
                        .as_deref()
                        .map(|target| tl.contains(&target.to_lowercase()))
                        .unwrap_or(false)
                        || (!step.selector.is_empty()
                            && tl.contains(&step.selector.to_lowercase()))
                });
                btn_match || text_match
            };

            if found {
                return Some(snap);
            }
        }
        thread::sleep(Duration::from_millis(80));
    }
    get_latest_snapshot()
}

/// Tunggu secara reaktif dan ping-pong dengan browser:
/// Mengirim REFRESH_SNAPSHOT ke ekstensi, lalu menunggu hingga timestamp snapshot > previous_ts
/// serta halaman dalam kondisi settled (tidak loading dan pending requests = 0).
pub fn wait_for_dom_change(previous_ts: u64, max_wait_ms: u64) -> Option<BrowserDomSnapshot> {
    let start = Instant::now();
    let timeout = Duration::from_millis(max_wait_ms);
    let mut last_ping = Instant::now();

    // Langsung tembak REFRESH_SNAPSHOT pertama (Ping)
    crate::services::browser_session_service::send_browser_command(
        crate::services::browser_session_service::BrowserCommand {
            action: "REFRESH_SNAPSHOT".to_string(),
            ..Default::default()
        },
    );

    while start.elapsed() < timeout {
        // Ping ulang setiap 250ms jika browser belum merespon
        if last_ping.elapsed() >= Duration::from_millis(250) {
            crate::services::browser_session_service::send_browser_command(
                crate::services::browser_session_service::BrowserCommand {
                    action: "REFRESH_SNAPSHOT".to_string(),
                    ..Default::default()
                },
            );
            last_ping = Instant::now();
        }

        if let Some(snap) = get_latest_snapshot() {
            let settled = !snap.is_loading && snap.pending_requests == 0;
            if snap.timestamp > previous_ts && settled {
                return Some(snap);
            }
        }
        thread::sleep(Duration::from_millis(30));
    }
    get_latest_snapshot()
}

/// Eksekusi satu langkah otomasi browser
pub fn execute_single_step(
    step: &RecipeStep,
    params: &HashMap<String, String>,
    snap: Option<&BrowserDomSnapshot>,
) {
    let fallback_cmd_list = if step.fallback_selectors.is_empty() {
        None
    } else {
        Some(step.fallback_selectors.clone())
    };

    match step.action.as_str() {
        "click" => {
            let target = resolve_best_click_target(
                snap,
                &step.selector,
                &step.fallback_selectors,
                step.target_text.as_deref(),
            );

            match target {
                Some(ClickTarget::Button(sel)) => {
                    crate::term_log!("  🖱️ [Click Target: Button] {}", sel);
                    crate::services::browser_session_service::send_browser_command(
                        crate::services::browser_session_service::BrowserCommand {
                            action: "CLICK_BUTTON".to_string(),
                            selector: Some(sel),
                            fallback_selectors: fallback_cmd_list,
                            ..Default::default()
                        },
                    );
                }
                Some(ClickTarget::Text(txt)) => {
                    crate::term_log!("  🖱️ [Click Target: Text Fallback] \"{}\"", txt);
                    crate::services::browser_session_service::send_browser_command(
                        crate::services::browser_session_service::BrowserCommand {
                            action: "CLICK_TEXT".to_string(),
                            selector: Some(txt),
                            fallback_selectors: fallback_cmd_list,
                            ..Default::default()
                        },
                    );
                }
                None => {
                    crate::term_err!("⚠️ [Step Executor] No button or text found for step");
                }
            }
        }

        "type" => {
            let mut text_to_type = step.value.clone().unwrap_or_default();
            for (k, v) in params {
                text_to_type = text_to_type.replace(&format!("{{{{{}}}}}", k), v);
            }

            let fallback_vec: Vec<String> = step.target_text.clone().into_iter().collect();
            let resolved_selector = if let Some(s) = snap {
                resolve_best_element_selector(
                    &s.inputs,
                    &step.selector,
                    &step.fallback_selectors,
                    &fallback_vec,
                )
            } else if !step.selector.is_empty() {
                Some(step.selector.clone())
            } else if let Some(fb) = step.fallback_selectors.first() {
                Some(fb.clone())
            } else {
                step.target_text.clone()
            };

            if let Some(ref sel) = resolved_selector {
                crate::services::browser_session_service::send_browser_command(
                    crate::services::browser_session_service::BrowserCommand {
                        action: "CLICK_TEXT".to_string(),
                        selector: Some(sel.clone()),
                        fallback_selectors: fallback_cmd_list.clone(),
                        ..Default::default()
                    },
                );
                thread::sleep(Duration::from_millis(150));
            }

            crate::services::browser_session_service::send_browser_command(
                crate::services::browser_session_service::BrowserCommand {
                    action: "TYPE_TEXT".to_string(),
                    selector: resolved_selector,
                    fallback_selectors: fallback_cmd_list,
                    text_to_type: Some(text_to_type),
                    press_enter: Some(false),
                    ..Default::default()
                },
            );
        }

        "enter" => {
            crate::services::browser_session_service::send_browser_command(
                crate::services::browser_session_service::BrowserCommand {
                    action: "PRESS_ENTER".to_string(),
                    selector: if step.selector.is_empty() { None } else { Some(step.selector.clone()) },
                    fallback_selectors: fallback_cmd_list,
                    press_enter: Some(true),
                    ..Default::default()
                },
            );
        }

        "scroll" => {
            let amount = step.selector.parse::<i32>().unwrap_or(400);
            crate::services::browser_session_service::send_browser_command(
                crate::services::browser_session_service::BrowserCommand {
                    action: "SCROLL".to_string(),
                    scroll_y: Some(amount),
                    ..Default::default()
                },
            );
        }

        other => {
            crate::term_err!("⚠️ [Step Executor] Unknown step action: {}", other);
        }
    }
}

/// Eksekusi resep otomasi secara mandiri (Local Repetitive Worker)
/// Payload format: `recipe_id` atau `recipe_id|{"param1": "value1"}`
pub fn execute_recipe(payload: &str) -> Option<(String, String)> {
    let clean = payload.trim();
    if clean.is_empty() {
        return None;
    }

    let (recipe_id, raw_json) = if let Some((id, json_str)) = clean.split_once('|') {
        (id.trim(), json_str.trim())
    } else {
        (clean, "")
    };

    let recipe = match find_recipe(recipe_id) {
        Some(r) => r,
        None => {
            crate::term_err!("⚠️ [Recipe Runner] Recipe '{}' not found in assets/recipes/", recipe_id);
            return Some((
                format!("Recipe Not Found: {recipe_id}"),
                format!("Recipe '{recipe_id}' is not registered"),
            ));
        }
    };

    // Parse parameter dinamis (jika ada)
    let params: HashMap<String, String> = if !raw_json.is_empty() {
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(raw_json) {
            if let Some(obj) = val.as_object() {
                obj.iter()
                    .map(|(k, v)| {
                        let str_val = match v {
                            serde_json::Value::String(s) => s.clone(),
                            other => other.to_string(),
                        };
                        (k.clone(), str_val)
                    })
                    .collect()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        }
    } else {
        HashMap::new()
    };

    // Validasi parameter ketat
    for req in &recipe.required_params {
        let provided = params
            .get(req)
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .or_else(|| {
                if req == "query" {
                    params.get("keyword").map(|s| s.trim()).filter(|s| !s.is_empty())
                } else if req == "keyword" {
                    params.get("query").map(|s| s.trim()).filter(|s| !s.is_empty())
                } else {
                    None
                }
            });

        if provided.is_none() {
            crate::term_err!(
                "⚠️ [Recipe Runner] Recipe '{}' requires parameter '{}', but it was missing in payload: '{}'",
                recipe.recipe_id,
                req,
                clean
            );
            return Some((
                format!("Missing Parameter: {req}"),
                format!(
                    "Recipe '{}' requires parameter '{}' (e.g. [CMD:recipe:run|{}|{{\"{}\": \"...\"}}])",
                    recipe.name, req, recipe.recipe_id, req
                ),
            ));
        }
    }

    crate::term_log!(
        "⚡ [Recipe Runner] Executing recipe '{}' ({}) with {} params...",
        recipe.recipe_id,
        recipe.name,
        params.len()
    );

    let recipe_name = recipe.name.clone();
    let recipe_id_clone = recipe.recipe_id.clone();

    // Jalankan eksekusi langkah-langkah di background thread agar tidak memblokir chat loop utama (Silent Worker)
    thread::spawn(move || {
        let initial_ts = get_latest_snapshot().map(|s| s.timestamp).unwrap_or(0);

        // Step 0: Buka Tab Baru (Isolated Clean State) sesuai aturan mandatory
        if recipe.initial_action.action == "open_new_tab" {
            let initial_url = &recipe.initial_action.url;
            crate::term_log!("🌐 [Recipe Runner] Opening new tab to: {}", initial_url);
            crate::services::browser_session_service::smart_open_or_switch_tab(
                initial_url,
                None,
                None,
            );
            // Tunggu snapshot aktif terbaru dari active_tab_browser.json
            wait_for_target_page(&recipe.domain, initial_ts, 6);
        }

        let total_steps = recipe.steps.len();

        // Jalankan setiap langkah secara berurutan secara senyap di background
        for (idx, step) in recipe.steps.iter().enumerate() {
            let target_label = step.target_text.as_deref().unwrap_or(&step.selector);
            crate::term_log!(
                "  ▶️ Step {}/{}: {} ({})",
                step.step,
                total_steps,
                step.action,
                target_label
            );

            // Tunggu elemen target ready di DOM sebelum mengeksekusi aksi
            let snap = wait_for_target_ready(step, 8);
            let before_step_ts = snap.as_ref().map(|s| s.timestamp).unwrap_or(0);

            execute_single_step(step, &params, snap.as_ref());

            // 1. PING-PONG REAKTIF: Minta paksa snapshot baru dan tunggu DOM merespon
            let _updated_snap = wait_for_dom_change(before_step_ts, 2500);

            // 2. VERIFIKASI HASIL EKSEKUSI (Loop Check ping-pong):
            // Jika aksi adalah "type", pastikan input di DOM benar-benar sudah berisi nilai teks yang diketikkan
            if step.action == "type" {
                let mut expected_val = step.value.clone().unwrap_or_default();
                for (k, v) in &params {
                    expected_val = expected_val.replace(&format!("{{{{{}}}}}", k), v);
                }
                let clean_expected = expected_val.trim().to_lowercase();

                if !clean_expected.is_empty() {
                    let verify_start = Instant::now();
                    while verify_start.elapsed() < Duration::from_millis(1500) {
                        let cur_snap = get_latest_snapshot();
                        let input_filled = cur_snap.as_ref().map(|s| {
                            s.inputs.iter().any(|inp| {
                                let val_match = inp.text.as_deref().unwrap_or("").to_lowercase().contains(&clean_expected);
                                let sel_match = !step.selector.is_empty() && inp.selector.as_deref().unwrap_or("").eq_ignore_ascii_case(&step.selector);
                                val_match || (sel_match && inp.text.is_some())
                            })
                        }).unwrap_or(false);

                        if input_filled {
                            break;
                        }
                        // Refresh snapshot ulang jika belum sinkron
                        crate::services::browser_session_service::send_browser_command(
                            crate::services::browser_session_service::BrowserCommand {
                                action: "REFRESH_SNAPSHOT".to_string(),
                                ..Default::default()
                            },
                        );
                        thread::sleep(Duration::from_millis(100));
                    }
                }
            }

            // 3. ANTI RACE CONDITION: Jika ada langkah berikutnya, verifikasi bahwa elemen langkah berikutnya
            // benar-benar sudah muncul di DOM sebelum kita mengumumkan langkah ini selesai dan mengeksekusi langkah depan!
            if let Some(next_step) = recipe.steps.get(idx + 1) {
                let _ = wait_for_target_ready(next_step, 6);
            }
        }

        crate::term_log!("✨ [Recipe Runner] Recipe '{}' completed successfully in background!", recipe_id_clone);
    });

    Some((
        format!("Run Recipe: {recipe_name}"),
        format!("Executing recipe '{recipe_name}'"),
    ))
}
