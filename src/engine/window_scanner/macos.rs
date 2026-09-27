#[cfg(target_os = "macos")]
use super::{current_epoch_ms, default_source, ActiveWindowSnapshot, DesktopElement, WindowItem};

#[cfg(target_os = "macos")]
pub fn get_focused_window_quick_info_macos() -> Option<(String, String)> {
    let script = r#"
    tell application "System Events"
        set frontProc to first process whose frontmost is true
        set procName to name of frontProc
        set winTitle to ""
        if (count of windows of frontProc) > 0 then
            set winTitle to name of front window of frontProc
        end if
        return procName & "|||" & winTitle
    end tell
    "#;

    if let Ok(output) = std::process::Command::new("osascript")
        .args(["-e", script])
        .output()
    {
        if output.status.success() {
            let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
            let parts: Vec<&str> = raw.split("|||").collect();
            if parts.len() >= 2 {
                return Some((parts[0].trim().to_string(), parts[1].trim().to_string()));
            }
        }
    }

    None
}

#[cfg(target_os = "macos")]
pub async fn scan_active_window_macos() -> Result<ActiveWindowSnapshot, String> {
    let script = r#"
    set jsonOut to "{\"app\":\"\",\"title\":\"\",\"windows\":[],\"buttons\":[]}"
    try
        tell application "System Events"
            set frontProc to first process whose frontmost is true
            set procName to name of frontProc
            set winTitle to ""
            set winList to {}
            set btnList to {}

            -- Daftar seluruh proses ber-jendela (open windows)
            set allProcs to every process whose background only is false
            repeat with p in allProcs
                try
                    set pName to name of p
                    repeat with w in windows of p
                        set wTitle to name of w
                        if wTitle is not "" then
                            set end of winList to "{\"app\":\"" & pName & "\",\"title\":\"" & wTitle & "\",\"active\":" & ((frontmost of p) as string) & "}"
                        end if
                    end repeat
                end try
            end repeat

            -- Detail tombol jendela aktif
            if (count of windows of frontProc) > 0 then
                set targetWin to front window of frontProc
                set winTitle to name of targetWin
                set btnIdx to 1
                try
                    repeat with b in (every button of targetWin)
                        set bName to name of b
                        if bName is not "" then
                            set {bx, by} to position of b
                            set {bw, bh} to size of b
                            set cx to (bx + (bw / 2)) as integer
                            set cy to (by + (bh / 2)) as integer
                            set end of btnList to "{\"id\":" & btnIdx & ",\"label\":\"" & bName & "\",\"role\":\"button\",\"center\":[" & cx & "," & cy & "]}"
                            set btnIdx to btnIdx + 1
                        end if
                    end repeat
                end try
            end if

            -- Format output sederhana
            return procName & "|||" & winTitle & "|||[" & (my joinList(winList, ",")) & "]|||[" & (my joinList(btnList, ",")) & "]"
        end tell
    on error errMsg
        return "||||||[]|||[]"
    end try

    on joinList(lst, sep)
        set oldTID to AppleScript's text item delimiters
        set AppleScript's text item delimiters to sep
        set str to lst as string
        set AppleScript's text item delimiters to oldTID
        return str
    end joinList
    "#;

    let output = std::process::Command::new("osascript")
        .args(["-e", script])
        .output()
        .map_err(|e| format!("Gagal memanggil osascript: {e}"))?;

    let raw = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let parts: Vec<&str> = raw.split("|||").collect();

    let mut snapshot = ActiveWindowSnapshot {
        timestamp: current_epoch_ms(),
        source: default_source(),
        ..Default::default()
    };

    if parts.len() >= 4 {
        snapshot.app_name = parts[0].trim().to_string();
        snapshot.window_title = parts[1].trim().to_string();
        snapshot.window_id = format!("macos:{}", snapshot.app_name);

        if let Ok(wins) = serde_json::from_str::<Vec<serde_json::Value>>(parts[2]) {
            let mut counter = 1;
            for w in wins {
                let app = w.get("app").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let title = w.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
                let active = w.get("active").and_then(|v| v.as_bool()).unwrap_or(false);
                if !app.is_empty() {
                    snapshot.windows.push(WindowItem {
                        id: counter,
                        app_name: app.clone(),
                        title,
                        active,
                        window_id: format!("macos:{}", app),
                    });
                    counter += 1;
                }
            }
        }

        if let Ok(btns) = serde_json::from_str::<Vec<DesktopElement>>(parts[3]) {
            snapshot.buttons = btns;
        }
    }

    let _ = snapshot.save_to_file();
    Ok(snapshot)
}
