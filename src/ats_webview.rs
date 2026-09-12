use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::thread;
use tauri::Manager;

pub static ATS_PLAYBACK_STATUS: LazyLock<Mutex<String>> =
    LazyLock::new(|| Mutex::new("idle".to_string()));

#[tauri::command]
pub fn ats_report_status(status: String) {
    if let Ok(mut guard) = ATS_PLAYBACK_STATUS.lock() {
        *guard = status;
    }
}

#[tauri::command]
pub fn open_anytospeech_window(app: tauri::AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("anytospeech-session") {
        let _ = window.show();
        let _ = window.set_focus();
        return Ok(());
    }

    let target_url_str = if std::env::var("TTS_ENGINE").unwrap_or_default().to_lowercase() == "kiki"
        || std::env::var("TTS_ENGINE").unwrap_or_default().to_lowercase() == "kikivoice" {
        "https://kikivoice.ai/ai-voice-cloning"
    } else {
        "https://anytospeech.com"
    };

    if let Ok(target_url) = target_url_str.parse() {
        let init_script = r##"
            window.__ats_status = 'idle';

            function reportStatus(st) {
                if (window.__ats_status === st) return;
                window.__ats_status = st;
                try {
                    document.title = '__ATS_STATE:' + st;
                    window.location.hash = '#ats_' + st;
                } catch(e) {}
                console.log('📢 [Voice WebKit Status]:', st);
            }

            function triggerNativeClick(el) {
                if (!el) return false;
                try {
                    el.scrollIntoView({ behavior: 'instant', block: 'center' });

                    // Pemicu 1: MouseEvent lengkap
                    const mouseEvents = ['mouseover', 'mouseenter', 'mousedown', 'mouseup', 'click'];
                    mouseEvents.forEach(type => {
                        const me = new MouseEvent(type, {
                            bubbles: true,
                            cancelable: true,
                            view: window,
                            buttons: 1
                        });
                        el.dispatchEvent(me);
                    });

                    // Pemicu 2: PointerEvent modern
                    if (window.PointerEvent) {
                        ['pointerover', 'pointerenter', 'pointerdown', 'pointerup'].forEach(type => {
                            const pe = new PointerEvent(type, {
                                bubbles: true,
                                cancelable: true,
                                view: window,
                                isPrimary: true,
                                button: 0,
                                buttons: 1
                            });
                            el.dispatchEvent(pe);
                        });
                    }

                    // Pemicu 3: Method click bawaan DOM
                    if (typeof el.click === 'function') {
                        el.click();
                    }

                    // Pemicu 4: Trigger parent form submit jika berada di dalam form
                    const parentForm = el.closest('form');
                    if (parentForm) {
                        parentForm.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }));
                    }

                    return true;
                } catch(e) {
                    console.error('Error clicking element:', e);
                    return false;
                }
            }

            let kikiRetryInterval = null;

            window.stopTTS = function() {
                if (kikiRetryInterval) {
                    clearInterval(kikiRetryInterval);
                    kikiRetryInterval = null;
                }
                try {
                    const audioList = document.querySelectorAll('audio');
                    audioList.forEach(a => {
                        if (!a.paused) a.pause();
                    });
                } catch(e) {}
                reportStatus('ended');
            };

            // Menghubungkan audio element listener di halaman TTS WebKit
            function attachAudioListeners() {
                const audios = document.querySelectorAll('audio');
                audios.forEach(audio => {
                    if (audio.__ats_attached) return;
                    audio.__ats_attached = true;

                    audio.addEventListener('play', () => {
                        console.log('🔊 [Voice WebKit] Audio playing...');
                        if (kikiRetryInterval) {
                            clearInterval(kikiRetryInterval);
                            kikiRetryInterval = null;
                        }
                        reportStatus('playing');
                    });

                    audio.addEventListener('pause', () => {
                        if (audio.currentTime >= audio.duration || audio.ended) {
                            console.log('⏹️ [Voice WebKit] Audio ended on pause.');
                            onPlaybackFinished();
                        }
                    });

                    audio.addEventListener('ended', () => {
                        console.log('⏹️ [Voice WebKit] Audio ended.');
                        onPlaybackFinished();
                    });
                });
            }

            function onPlaybackFinished() {
                if (kikiRetryInterval) {
                    clearInterval(kikiRetryInterval);
                    kikiRetryInterval = null;
                }
                reportStatus('ended');
                // Klik tombol 'Coba Lagi' (#fvTryAgain) jika AnyToSpeech
                setTimeout(() => {
                    const tryAgainBtn = document.querySelector('#fvTryAgain, button.fv-try-again-btn');
                    if (tryAgainBtn) {
                        console.log('🔄 [AnyToSpeech] Menekan tombol Coba Lagi (#fvTryAgain)...');
                        triggerNativeClick(tryAgainBtn);
                    }
                }, 300);
            }

            window.speakWithAnyToSpeech = function(text) {
                console.log('🗣️ [Voice WebKit] Input text (' + text.length + ' chars):', text);
                reportStatus('generating');

                if (kikiRetryInterval) {
                    clearInterval(kikiRetryInterval);
                    kikiRetryInterval = null;
                }

                // 1. KikiVoice.ai (React/Tailwind controlled textarea & button)
                const allTextareas = Array.from(document.querySelectorAll('textarea'));
                const kikiTextarea = allTextareas.find(t => t.placeholder && t.placeholder.toLowerCase().includes('script'))
                                     || document.querySelector('textarea');

                if (kikiTextarea) {
                    kikiTextarea.focus();

                    // Bersihkan dulu isi textarea dengan prototype setter
                    const nativeSetter = Object.getOwnPropertyDescriptor(window.HTMLTextAreaElement.prototype, 'value')?.set;
                    if (nativeSetter) {
                        nativeSetter.call(kikiTextarea, text);
                    } else {
                        kikiTextarea.value = text;
                    }

                    kikiTextarea.dispatchEvent(new Event('focus', { bubbles: true }));
                    kikiTextarea.dispatchEvent(new InputEvent('input', { bubbles: true, inputType: 'insertText', data: text }));
                    kikiTextarea.dispatchEvent(new Event('input', { bubbles: true }));
                    kikiTextarea.dispatchEvent(new Event('change', { bubbles: true }));
                    kikiTextarea.dispatchEvent(new Event('blur', { bubbles: true }));

                    function clickKikiSubmit() {
                        const allButtons = Array.from(document.querySelectorAll('button'));
                        const btn = document.querySelector('button.clone-primary-action')
                                    || allButtons.find(b => b.className && typeof b.className === 'string' && b.className.includes('clone-primary-action'))
                                    || allButtons.find(b => b.innerText && b.innerText.toLowerCase().includes('clone voice'))
                                    || allButtons.find(b => b.textContent && b.textContent.toLowerCase().includes('clone voice'));

                        if (btn) {
                            console.log('▶️ [KikiVoice] Menekan tombol Clone Voice Now...');
                            triggerNativeClick(btn);
                            const innerSpan = btn.querySelector('span');
                            if (innerSpan) {
                                triggerNativeClick(innerSpan);
                            }
                        }
                    }

                    // Klik awal pertama setelah 250ms
                    setTimeout(() => {
                        clickKikiSubmit();
                    }, 250);

                    // Auto-retry klik setiap 8 detik jika audio belum kunjung berbunyi / server error
                    let retryCount = 0;
                    kikiRetryInterval = setInterval(() => {
                        retryCount++;
                        // Cek apakah ada audio yang sudah berputar
                        const audios = document.querySelectorAll('audio');
                        let isPlaying = false;
                        audios.forEach(a => {
                            if (!a.paused && a.currentTime > 0 && !a.ended) {
                                isPlaying = true;
                            }
                        });

                        if (isPlaying || window.__ats_status === 'playing') {
                            console.log('✅ [KikiVoice] Audio sudah berbunyi, stop auto-retry.');
                            clearInterval(kikiRetryInterval);
                            kikiRetryInterval = null;
                            return;
                        }

                        console.log('🔄 [KikiVoice Server Busy/Pending] Belum berbunyi, submit ulang (Retry #' + retryCount + ')...');

                        // Pastikan isi teks tetap fresh di textarea
                        if (kikiTextarea.value !== text) {
                            const ns = Object.getOwnPropertyDescriptor(window.HTMLTextAreaElement.prototype, 'value')?.set;
                            if (ns) ns.call(kikiTextarea, text);
                            else kikiTextarea.value = text;
                            kikiTextarea.dispatchEvent(new Event('input', { bubbles: true }));
                        }

                        clickKikiSubmit();
                    }, 8000);

                    return;
                }

                // 2. AnyToSpeech editor: #fvEditor
                const fvEditor = document.querySelector('#fvEditor');
                if (fvEditor) {
                    // Jika ada tombol 'Coba Lagi' yang aktif, tekan dulu agar editor fresh
                    const tryAgainBtn = document.querySelector('#fvTryAgain, button.fv-try-again-btn');
                    if (tryAgainBtn && tryAgainBtn.offsetParent !== null) {
                        triggerNativeClick(tryAgainBtn);
                    }

                    setTimeout(() => {
                        const ed = document.querySelector('#fvEditor') || fvEditor;
                        ed.focus();
                        ed.innerText = text;
                        ed.dispatchEvent(new InputEvent('input', { bubbles: true }));
                        ed.dispatchEvent(new Event('change', { bubbles: true }));

                        setTimeout(() => {
                            const genBtn = document.querySelector('#fvGenerate, button.fv-generate-btn');
                            if (genBtn) {
                                console.log('▶️ [AnyToSpeech] Menekan tombol Hasilkan ucapan (#fvGenerate)...');
                                triggerNativeClick(genBtn);
                            } else {
                                console.warn('⚠️ [AnyToSpeech] Tombol #fvGenerate tidak ditemukan!');
                            }
                        }, 250);
                    }, 200);
                    return;
                }

                // 3. Fallback jika user membuka editor teks lain
                const fallbackEditor = document.querySelector('textarea, .edit-content, .pw-editor, [contenteditable="true"]');
                if (fallbackEditor) {
                    fallbackEditor.focus();
                    if (fallbackEditor.tagName === 'TEXTAREA' || fallbackEditor.tagName === 'INPUT') {
                        fallbackEditor.value = text;
                        fallbackEditor.dispatchEvent(new Event('input', { bubbles: true }));
                        fallbackEditor.dispatchEvent(new Event('change', { bubbles: true }));
                    } else {
                        fallbackEditor.innerText = text;
                        fallbackEditor.dispatchEvent(new InputEvent('input', { bubbles: true }));
                    }

                    setTimeout(() => {
                        const fallbackBtn = document.querySelector('button.clone-primary-action, #fvGenerate, button[type="button"]');
                        if (fallbackBtn) {
                            triggerNativeClick(fallbackBtn);
                        }
                    }, 250);
                }
            };

            // Observer berkala untuk audio element & status
            setInterval(() => {
                attachAudioListeners();

                // Deteksi jika WebKit sedang memutar audio
                const audios = document.querySelectorAll('audio');
                let isAnyPlaying = false;
                audios.forEach(a => {
                    if (!a.paused && a.currentTime > 0 && !a.ended) {
                        isAnyPlaying = true;
                    }
                });

                if (isAnyPlaying) {
                    reportStatus('playing');
                }
            }, 60);
        "##;

        let window_title = if target_url_str.contains("kikivoice") {
            "Voice AI WebKit Session - KikiVoice"
        } else {
            "Voice AI WebKit Session - AnyToSpeech"
        };

        tauri::WebviewWindowBuilder::new(
            &app,
            "anytospeech-session",
            tauri::WebviewUrl::External(target_url),
        )
        .title(window_title)
        .inner_size(900.0, 650.0)
        .resizable(true)
        .visible(true)
        .initialization_script(init_script)
        .build()
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn speak_in_anytospeech_webview(text: &str, is_interrupted: Arc<AtomicBool>) -> bool {
    let window = match crate::overlay::APP_HANDLE.lock() {
        Ok(guard) => {
            if let Some(app) = guard.as_ref() {
                app.get_webview_window("anytospeech-session")
            } else {
                None
            }
        }
        Err(_) => None,
    };

    let window = match window {
        Some(w) => w,
        None => return false,
    };

    if let Ok(mut guard) = ATS_PLAYBACK_STATUS.lock() {
        *guard = "generating".to_string();
    }

    crate::tray::TrayState::update(crate::tray::AssistantState::Thinking, "AnyToSpeech: Sedang membuat audio...");

    // Reset hash terlebih dahulu agar status bersih
    let _ = window.eval("try { window.location.hash = '#ats_idle'; } catch(e) {}");

    let json_text = serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_string());
    let js = format!(
        "try {{ if (window.speakWithAnyToSpeech) {{ window.speakWithAnyToSpeech({}); }} }} catch(err) {{ console.error(err); }}",
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

        let mut current_status = ATS_PLAYBACK_STATUS.lock().map(|g| g.clone()).unwrap_or_default();

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
                        "AnyToSpeech: Mempersiapkan audio...",
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
