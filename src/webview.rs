use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::thread;
use tauri::Manager;

pub static WEB_TTS_PLAYBACK_STATUS: LazyLock<Mutex<String>> =
    LazyLock::new(|| Mutex::new("idle".to_string()));

#[tauri::command]
pub fn ats_report_status(status: String) {
    if let Ok(mut guard) = WEB_TTS_PLAYBACK_STATUS.lock() {
        *guard = status;
    }
}

#[tauri::command]
pub fn open_anytospeech_window(app: tauri::AppHandle) -> Result<(), String> {
    let u = crate::config::load_user_config();
    let eng = u.tts_engine.to_lowercase();
    let is_kiki = eng == "kiki" || eng == "kikivoice";

    let target_url_str = if is_kiki {
        "https://kikivoice.ai/ai-voice-cloning"
    } else {
        "https://anytospeech.com"
    };

    let window_title = if is_kiki {
        "Voice AI Session - KikiVoice"
    } else {
        "Voice AI Session - AnyToSpeech"
    };

    if let Some(window) = app.get_webview_window("anytospeech-session") {
        let _ = window.set_title(window_title);
        let is_already_on_target = window
            .url()
            .map(|u| {
                let s = u.as_str();
                !s.contains("about:blank")
                    && ((is_kiki && s.contains("kikivoice"))
                        || (!is_kiki && s.contains("anytospeech")))
            })
            .unwrap_or(false);

        if !is_already_on_target {
            if let Ok(target_url) = target_url_str.parse() {
                let _ = window.navigate(target_url);
            }
        }

        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
        return Ok(());
    }

    if let Ok(target_url) = target_url_str.parse() {
        let builder = tauri::WebviewWindowBuilder::new(
            &app,
            "anytospeech-session",
            tauri::WebviewUrl::External(target_url),
        )
        .title(window_title)
        .inner_size(960.0, 700.0)
        .resizable(true)
        .decorations(true)
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/124.0.0.0 Safari/537.36")
        .devtools(true)
        .focused(true);

        builder.build().map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn speak_in_anytospeech_webview(text: &str, is_interrupted: Arc<AtomicBool>) -> bool {
    let mut window = match crate::overlay::APP_HANDLE.lock() {
        Ok(guard) => {
            if let Some(app) = guard.as_ref() {
                app.get_webview_window("anytospeech-session")
            } else {
                None
            }
        }
        Err(_) => None,
    };

    // Jika jendela belum terbuka, buka otomatis
    if window.is_none() {
        if let Ok(guard) = crate::overlay::APP_HANDLE.lock() {
            if let Some(app) = guard.as_ref() {
                println!("🌐 [Web TTS] Membuka jendela WebKit otomatis...");
                let _ = open_anytospeech_window(app.clone());
                thread::sleep(std::time::Duration::from_millis(1500));
                window = app.get_webview_window("anytospeech-session");
            }
        }
    }

    let window = match window {
        Some(w) => w,
        None => return false,
    };

    if let Ok(mut guard) = WEB_TTS_PLAYBACK_STATUS.lock() {
        *guard = "generating".to_string();
    }

    crate::tray::TrayState::update(crate::tray::AssistantState::Thinking, "Sedang membuat audio...");

    // Reset hash terlebih dahulu agar status bersih
    let _ = window.eval("try { window.location.hash = '#ats_idle'; } catch(e) {}");

    let json_text = serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string());
    let js = format!(
        r#"
        (function() {{
            function triggerNativeClick(el) {{
                if (!el) return false;
                try {{
                    el.scrollIntoView({{ behavior: 'instant', block: 'center' }});
                    ['mouseover', 'mouseenter', 'mousedown', 'mouseup', 'click'].forEach(t => {{
                        el.dispatchEvent(new MouseEvent(t, {{ bubbles: true, cancelable: true, view: window, buttons: 1 }}));
                    }});
                    if (typeof el.click === 'function') el.click();
                    return true;
                }} catch(e) {{
                    return false;
                }}
            }}

            // Monitor elemen audio secara aktif
            const audios = document.querySelectorAll('audio');
            audios.forEach(audio => {{
                if (!audio.__ats_hooked) {{
                    audio.__ats_hooked = true;
                    audio.addEventListener('play', () => {{
                        document.title = '__ATS_STATE:playing';
                        window.location.hash = '#ats_playing';
                    }});
                    audio.addEventListener('pause', () => {{
                        if (audio.currentTime >= audio.duration || audio.ended) {{
                            document.title = '__ATS_STATE:ended';
                            window.location.hash = '#ats_ended';
                        }}
                    }});
                    audio.addEventListener('ended', () => {{
                        document.title = '__ATS_STATE:ended';
                        window.location.hash = '#ats_ended';
                    }});
                }}
            }});

            const text = {};
            const isKiki = window.location.hostname.includes('kikivoice');

            if (isKiki) {{
                const allTextareas = Array.from(document.querySelectorAll('textarea'));
                const kikiTextarea = allTextareas.find(t => t.placeholder && t.placeholder.toLowerCase().includes('script')) || document.querySelector('textarea');

                if (kikiTextarea) {{
                    kikiTextarea.focus();
                    const nativeSetter = Object.getOwnPropertyDescriptor(window.HTMLTextAreaElement.prototype, 'value')?.set;
                    if (nativeSetter) nativeSetter.call(kikiTextarea, text);
                    else kikiTextarea.value = text;
                    kikiTextarea.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    kikiTextarea.dispatchEvent(new Event('change', {{ bubbles: true }}));

                    setTimeout(() => {{
                        const allButtons = Array.from(document.querySelectorAll('button'));
                        const btn = document.querySelector('button.clone-primary-action')
                                    || allButtons.find(b => b.innerText && b.innerText.toLowerCase().includes('clone voice'))
                                    || allButtons.find(b => b.textContent && b.textContent.toLowerCase().includes('clone voice'))
                                    || allButtons.find(b => b.innerText && b.innerText.toLowerCase().includes('generate'));
                        if (btn) triggerNativeClick(btn);
                    }}, 250);
                    return;
                }}
            }} else {{
                const fvEditor = document.querySelector('#fvEditor');
                if (fvEditor) {{
                    fvEditor.focus();
                    fvEditor.innerText = text;
                    fvEditor.dispatchEvent(new InputEvent('input', {{ bubbles: true }}));
                    fvEditor.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    setTimeout(() => {{
                        const genBtn = document.querySelector('#fvGenerate, button.fv-generate-btn');
                        if (genBtn) triggerNativeClick(genBtn);
                    }}, 250);
                    return;
                }}

                // Fallback textarea untuk AnyToSpeech
                const anyTextarea = document.querySelector('textarea');
                if (anyTextarea) {{
                    anyTextarea.focus();
                    anyTextarea.value = text;
                    anyTextarea.dispatchEvent(new Event('input', {{ bubbles: true }}));
                    anyTextarea.dispatchEvent(new Event('change', {{ bubbles: true }}));
                    setTimeout(() => {{
                        const allButtons = Array.from(document.querySelectorAll('button'));
                        const genBtn = document.querySelector('#fvGenerate, button.fv-generate-btn')
                                    || allButtons.find(b => b.innerText && b.innerText.toLowerCase().includes('generate'));
                        if (genBtn) triggerNativeClick(genBtn);
                    }}, 250);
                }}
            }}
        }})();
        "#,
        json_text
    );
    let _ = window.eval(&js);

    let mut saw_active = false;

    loop {
        // Jika user memotong percakapan (Spasi / ESC / tombol Stop)
        if is_interrupted.load(Ordering::SeqCst) {
            let _ = window.eval("if (window.stopTTS) { window.stopTTS(); }");
            break;
        }

        let mut current_status = WEB_TTS_PLAYBACK_STATUS.lock().map(|g| g.clone()).unwrap_or_default();

        // Cek via window.url() (location.hash) - 100% realtime di Tauri WebKit
        if let Ok(url) = window.url() {
            let url_str = url.as_str();
            if url_str.contains("ats_playing") {
                current_status = "playing".to_string();
            } else if url_str.contains("ats_ended") {
                current_status = "ended".to_string();
            }
        }

        // Fallback title-based status dari dokumen WebKit
        if current_status != "playing" && current_status != "ended" {
            if let Ok(title) = window.title() {
                if title.contains("__ATS_STATE:playing") {
                    current_status = "playing".to_string();
                } else if title.contains("__ATS_STATE:ended") {
                    current_status = "ended".to_string();
                }
            }
        }

        // Ambil emosi saat ini dari tray state agar pose tidak reset ke netral saat speaking
        let current_emotion = crate::tray::TrayState::get_state()
            .and_then(|v| v.get("emotion").and_then(|e| e.as_str()).map(|s| s.to_string()))
            .unwrap_or_else(|| "happy".to_string());

        match current_status.as_str() {
            "playing" => {
                saw_active = true;
                crate::tray::TrayState::update_with_emotion(
                    crate::tray::AssistantState::Speaking,
                    text,
                    &current_emotion,
                );
            }
            "ended" => {
                // Selesai murni saat audio selesai diputar
                if saw_active {
                    break;
                }
            }
            _ => {
                if !saw_active {
                    crate::tray::TrayState::update_with_emotion(
                        crate::tray::AssistantState::Thinking,
                        "Mempersiapkan audio...",
                        &current_emotion,
                    );
                }
            }
        }

        thread::sleep(std::time::Duration::from_millis(60));
    }

    let _ = window.eval("try { window.location.hash = '#ats_idle'; } catch(e) {}");

    crate::tray::TrayState::update(crate::tray::AssistantState::Listening, "Siap Mendengarkan");
    true
}
