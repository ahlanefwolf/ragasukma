use super::{current_epoch_ms, ActiveWindowSnapshot, DesktopElement, WindowItem};

#[cfg(target_os = "windows")]
pub fn get_focused_window_quick_info_windows() -> Option<(String, String)> {
    use std::path::Path;
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
    };

    unsafe {
        let fg_hwnd = GetForegroundWindow();
        if fg_hwnd.0.is_null() {
            return None;
        }

        let mut title_buf = [0u16; 512];
        let len = GetWindowTextW(fg_hwnd, &mut title_buf);
        let title = if len > 0 {
            String::from_utf16_lossy(&title_buf[..len as usize])
        } else {
            String::new()
        };

        let mut pid = 0u32;
        GetWindowThreadProcessId(fg_hwnd, Some(&mut pid));
        let mut app_name = String::new();

        if pid != 0 {
            if let Ok(process_handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                let mut path_buf = [0u16; 1024];
                let mut path_len = path_buf.len() as u32;
                if QueryFullProcessImageNameW(
                    process_handle,
                    PROCESS_NAME_FORMAT(0),
                    windows::core::PWSTR(path_buf.as_mut_ptr()),
                    &mut path_len,
                )
                .is_ok()
                    && path_len > 0
                {
                    let full_path = String::from_utf16_lossy(&path_buf[..path_len as usize]);
                    let exe_name = Path::new(&full_path)
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(&full_path);
                    app_name = exe_name.to_string();
                }
            }
        }

        Some((app_name, title))
    }
}

#[cfg(target_os = "windows")]
pub fn scan_active_window_windows() -> Result<ActiveWindowSnapshot, String> {
    use std::path::Path;
    use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows::Win32::System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
    };
    use windows::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_NAME_FORMAT, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows::Win32::UI::Accessibility::{
        CUIAutomation, IUIAutomation, TreeScope_Descendants, UIA_ButtonControlTypeId,
        UIA_CheckBoxControlTypeId, UIA_DocumentControlTypeId, UIA_EditControlTypeId,
        UIA_HyperlinkControlTypeId, UIA_MenuItemControlTypeId, UIA_RadioButtonControlTypeId,
        UIA_TextControlTypeId, UIA_CONTROLTYPE_ID,
    };
    use windows::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetForegroundWindow, GetWindowTextLengthW, GetWindowTextW,
        GetWindowThreadProcessId, IsWindowVisible,
    };

    let mut snapshot = ActiveWindowSnapshot {
        timestamp: current_epoch_ms(),
        ..Default::default()
    };

    unsafe {
        let fg_hwnd = GetForegroundWindow();

        // 1. Dapatkan Foreground Window Info
        if !fg_hwnd.0.is_null() {
            let mut title_buf = [0u16; 512];
            let len = GetWindowTextW(fg_hwnd, &mut title_buf);
            if len > 0 {
                snapshot.window_title = String::from_utf16_lossy(&title_buf[..len as usize]);
            }
            snapshot.window_id = format!("win:0x{:x}", fg_hwnd.0 as usize);

            let mut pid = 0u32;
            GetWindowThreadProcessId(fg_hwnd, Some(&mut pid));
            if pid != 0 {
                if let Ok(process_handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                    let mut path_buf = [0u16; 1024];
                    let mut path_len = path_buf.len() as u32;
                    if QueryFullProcessImageNameW(
                        process_handle,
                        PROCESS_NAME_FORMAT(0),
                        windows::core::PWSTR(path_buf.as_mut_ptr()),
                        &mut path_len,
                    )
                    .is_ok()
                        && path_len > 0
                    {
                        let full_path = String::from_utf16_lossy(&path_buf[..path_len as usize]);
                        let exe_name = Path::new(&full_path)
                            .file_name()
                            .and_then(|n| n.to_str())
                            .unwrap_or(&full_path);
                        snapshot.app_name = exe_name.to_string();
                    }
                }
            }
        }

        // 2. Enum semua jendela yang terlihat di Windows
        struct EnumContext {
            fg: HWND,
            windows: Vec<WindowItem>,
            counter: usize,
        }

        unsafe extern "system" fn enum_windows_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
            unsafe {
                let ctx = &mut *(lparam.0 as *mut EnumContext);

                if IsWindowVisible(hwnd).as_bool() {
                    let len = GetWindowTextLengthW(hwnd);
                    if len > 0 {
                        let mut title_buf = vec![0u16; (len + 1) as usize];
                        let read_len = GetWindowTextW(hwnd, &mut title_buf);
                        if read_len > 0 {
                            let title = String::from_utf16_lossy(&title_buf[..read_len as usize]);

                            let mut pid = 0u32;
                            GetWindowThreadProcessId(hwnd, Some(&mut pid));
                            let mut app_name = String::new();
                            if pid != 0 {
                                if let Ok(proc) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) {
                                    let mut path_buf = [0u16; 512];
                                    let mut path_len = path_buf.len() as u32;
                                    if QueryFullProcessImageNameW(
                                        proc,
                                        PROCESS_NAME_FORMAT(0),
                                        windows::core::PWSTR(path_buf.as_mut_ptr()),
                                        &mut path_len,
                                    )
                                    .is_ok()
                                        && path_len > 0
                                    {
                                        let full = String::from_utf16_lossy(&path_buf[..path_len as usize]);
                                        app_name = Path::new(&full)
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("")
                                            .to_string();
                                    }
                                }
                            }

                            let is_active = hwnd == ctx.fg;
                            ctx.windows.push(WindowItem {
                                id: ctx.counter,
                                app_name,
                                title,
                                active: is_active,
                                window_id: format!("win:0x{:x}", hwnd.0 as usize),
                            });
                            ctx.counter += 1;
                        }
                    }
                }

                BOOL(1)
            }
        }

        let mut enum_ctx = EnumContext {
            fg: fg_hwnd,
            windows: Vec::new(),
            counter: 1,
        };

        let _ = EnumWindows(Some(enum_windows_callback), LPARAM(&mut enum_ctx as *mut _ as isize));
        snapshot.windows = enum_ctx.windows;

        // 3. UI Automation untuk mengekstrak Buttons & Inputs dari Foreground Window
        if !fg_hwnd.0.is_null() {
            let _ = CoInitializeEx(None, COINIT_MULTITHREADED);

            if let Ok(automation) =
                CoCreateInstance::<_, IUIAutomation>(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
            {
                if let Ok(root_elem) = automation.ElementFromHandle(fg_hwnd) {
                    let mut element_id = 1;

                    if let Ok(condition) = automation.CreateTrueCondition() {
                        if let Ok(elements) = root_elem.FindAll(TreeScope_Descendants, &condition) {
                            if let Ok(count) = elements.Length() {
                                for i in 0..count {
                                    if let Ok(elem) = elements.GetElement(i) {
                                        let name: String = elem
                                            .CurrentName()
                                            .map(|b| b.to_string())
                                            .unwrap_or_default()
                                            .trim()
                                            .to_string();
                                        let control_type =
                                            elem.CurrentControlType().unwrap_or(UIA_CONTROLTYPE_ID(0));

                                        if !name.is_empty() {
                                            let (center, bbox) = if let Ok(rect) =
                                                elem.CurrentBoundingRectangle()
                                            {
                                                let w = rect.right - rect.left;
                                                let h = rect.bottom - rect.top;
                                                if w > 0 && h > 0 {
                                                    let cx = rect.left + w / 2;
                                                    let cy = rect.top + h / 2;
                                                    (Some([cx, cy]), Some([rect.left, rect.top, w, h]))
                                                } else {
                                                    (None, None)
                                                }
                                            } else {
                                                (None, None)
                                            };

                                            #[allow(non_upper_case_globals)]
                                            match control_type {
                                                UIA_ButtonControlTypeId
                                                | UIA_CheckBoxControlTypeId
                                                | UIA_RadioButtonControlTypeId
                                                | UIA_MenuItemControlTypeId
                                                | UIA_HyperlinkControlTypeId => {
                                                    snapshot.buttons.push(DesktopElement {
                                                        id: element_id,
                                                        label: name,
                                                        role: "button".to_string(),
                                                        center,
                                                        bbox,
                                                    });
                                                    element_id += 1;
                                                }
                                                UIA_EditControlTypeId => {
                                                    snapshot.inputs.push(DesktopElement {
                                                        id: element_id,
                                                        label: name,
                                                        role: "input".to_string(),
                                                        center,
                                                        bbox,
                                                    });
                                                    element_id += 1;
                                                }
                                                UIA_TextControlTypeId
                                                | UIA_DocumentControlTypeId => {
                                                    snapshot.texts.push(name);
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            CoUninitialize();
        }
    }

    if snapshot.texts.is_empty() {
        // Cek jika snapshot sebelumnya berasal dari perception/OCR dan untuk jendela yang sama
        if let Ok(prev) = ActiveWindowSnapshot::load_from_file() {
            if prev.source == "vision_perception"
                && prev.app_name == snapshot.app_name
                && !prev.texts.is_empty()
            {
                snapshot.texts = prev.texts;
                snapshot.text_summary = prev.text_summary;
                if snapshot.buttons.len() <= 4 && !prev.buttons.is_empty() {
                    snapshot.buttons = prev.buttons;
                }
            }
        }
    }

    if !snapshot.texts.is_empty() && snapshot.text_summary.is_empty() {
        snapshot.text_summary = if snapshot.texts.len() > 30 {
            format!(
                "{} ... ({} teks terdeteksi)",
                snapshot.texts[..30].join(" | "),
                snapshot.texts.len()
            )
        } else {
            snapshot.texts.join(" | ")
        };
    }

    if snapshot.app_name.is_empty() {
        snapshot.text_summary = "Tidak ada jendela aplikasi yang terfokus".to_string();
    }

    let _ = snapshot.save_to_file();
    Ok(snapshot)
}
