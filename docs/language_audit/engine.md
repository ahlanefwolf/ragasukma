# Language Audit: `engine` Module

- **Directory**: `src/engine`
- **Total Files**: 37
- **Total Comments**: 460
- **Total String Literals**: 1777

## Table of Contents
- [engine/actions/browser.rs](#engineactionsbrowserrs) (1 comments, 66 strings)
- [engine/actions/catalog.rs](#engineactionscatalogrs) (2 comments, 55 strings)
- [engine/actions/desktop/click.rs](#engineactionsdesktopclickrs) (20 comments, 48 strings)
- [engine/actions/desktop/input.rs](#engineactionsdesktopinputrs) (6 comments, 75 strings)
- [engine/actions/desktop/mod.rs](#engineactionsdesktopmodrs) (0 comments, 48 strings)
- [engine/actions/desktop/window.rs](#engineactionsdesktopwindowrs) (23 comments, 95 strings)
- [engine/actions/media.rs](#engineactionsmediars) (8 comments, 32 strings)
- [engine/actions/mod.rs](#engineactionsmodrs) (2 comments, 20 strings)
- [engine/actions/os_apps.rs](#engineactionsos-appsrs) (33 comments, 125 strings)
- [engine/actions/recipe/mod.rs](#engineactionsrecipemodrs) (0 comments, 0 strings)
- [engine/actions/recipe/runner.rs](#engineactionsreciperunnerrs) (39 comments, 44 strings)
- [engine/actions/recipe/trainer.rs](#engineactionsrecipetrainerrs) (38 comments, 134 strings)
- [engine/actions/recipe/types.rs](#engineactionsrecipetypesrs) (9 comments, 9 strings)
- [engine/actions/todo_actions.rs](#engineactionstodo-actionsrs) (1 comments, 8 strings)
- [engine/app_scanner.rs](#engineapp-scannerrs) (102 comments, 250 strings)
- [engine/chat_loop/input_listener.rs](#enginechat-loopinput-listenerrs) (5 comments, 14 strings)
- [engine/chat_loop/llm_responder.rs](#enginechat-loopllm-responderrs) (1 comments, 14 strings)
- [engine/chat_loop/mod.rs](#enginechat-loopmodrs) (8 comments, 35 strings)
- [engine/chat_loop/skill_runner.rs](#enginechat-loopskill-runnerrs) (7 comments, 24 strings)
- [engine/cloudflared.rs](#enginecloudflaredrs) (10 comments, 103 strings)
- [engine/mod.rs](#enginemodrs) (0 comments, 0 strings)
- [engine/perception/capture.rs](#engineperceptioncapturers) (7 comments, 24 strings)
- [engine/perception/mod.rs](#engineperceptionmodrs) (19 comments, 34 strings)
- [engine/perception/ocr_macos.rs](#engineperceptionocr-macosrs) (2 comments, 11 strings)
- [engine/perception/ocr_tesseract.rs](#engineperceptionocr-tesseractrs) (13 comments, 22 strings)
- [engine/perception/ocr_windows.rs](#engineperceptionocr-windowsrs) (0 comments, 21 strings)
- [engine/perception/utils.rs](#engineperceptionutilsrs) (39 comments, 105 strings)
- [engine/remote_text.rs](#engineremote-textrs) (2 comments, 1 strings)
- [engine/session.rs](#enginesessionrs) (3 comments, 12 strings)
- [engine/skills.rs](#engineskillsrs) (6 comments, 21 strings)
- [engine/tasks.rs](#enginetasksrs) (10 comments, 31 strings)
- [engine/todo.rs](#enginetodors) (1 comments, 40 strings)
- [engine/window_scanner/linux.rs](#enginewindow-scannerlinuxrs) (1 comments, 43 strings)
- [engine/window_scanner/macos.rs](#enginewindow-scannermacosrs) (0 comments, 17 strings)
- [engine/window_scanner/mod.rs](#enginewindow-scannermodrs) (5 comments, 45 strings)
- [engine/window_scanner/windows.rs](#enginewindow-scannerwindowsrs) (4 comments, 13 strings)
- [engine/ytdlp.rs](#engineytdlprs) (33 comments, 138 strings)

---

## `engine/actions/browser.rs`

- **Path**: `src/engine/actions/browser.rs`
- **Comments**: 1
- **Strings**: 66

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 1 | `///` | Execute browser commands based on canonical action |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 4 | `open` |
| 9 | `http://` |
| 9 | `https://` |
| 12 | `https://{val}` |
| 14 | `https://www.{val}.com` |
| 21 | `Open Tab: {val}` |
| 21 | `Opening tab {val}` |
| 24 | `search` |
| 36 | `{site}.com` |
| 38 | `http://` |
| 38 | `https://` |
| 41 | `https://www.{domain}` |
| 53 | `[Universal In-Page Search] Switching to active tab '{}' (ID: {}) and searching '{}'` |
| 58 | `SWITCH_OR_NAVIGATE` |
| 73 | `Search on {site}: {query}` |
| 74 | `Searching '{query}' on {site}` |
| 79 | `IN_PAGE_SEARCH` |
| 85 | `Search on Web Page: {query}` |
| 86 | `Searching '{query}' on active page` |
| 91 | `click_text` |
| 95 | `CLICK_TEXT` |
| 101 | `Click Text: {sel}` |
| 102 | `Clicking text '{sel}' in browser` |
| 106 | `click_button` |
| 110 | `CLICK_BUTTON` |
| 116 | `Click Button: {sel}` |
| 117 | `Clicking button '{sel}' in browser` |
| 121 | `type` |
| 129 | `TYPE_TEXT` |
| 136 | `Type Text: {text}` |
| 137 | `Typing '{text}' into web field` |
| 141 | `enter` |
| 144 | `PRESS_ENTER` |
| 150 | `Press Enter` |
| 151 | `Pressing Enter key on web page` |
| 155 | `scroll` |
| 159 | `SCROLL` |
| 165 | `Scroll Web Page` |
| 166 | `Scrolling page by {amount}px` |
| 170 | `switch` |
| 174 | `SWITCH_TAB` |
| 180 | `Switch Tab: {q}` |
| 181 | `Switching to tab '{q}'` |
| 185 | `close` |
| 189 | `CLOSE_TAB` |
| 194 | `Close Tab: {q}` |
| 194 | `Closing tab '{q}'` |
| 197 | `install_ext` |
| 197 | `open_ext_folder` |
| 201 | `windows` |
| 203 | `explorer` |
| 208 | `install_ext` |
| 209 | `cmd` |
| 210 | `/C` |
| 210 | `start` |
| 210 | `chrome://extensions` |
| 216 | `macos` |
| 218 | `open` |
| 223 | `install_ext` |
| 224 | `open` |
| 225 | `chrome://extensions` |
| 231 | `windows` |
| 231 | `macos` |
| 233 | `xdg-open` |
| 240 | `Open Extension Folder` |
| 241 | `Opening extension folder ({abs_str})` |

---

## `engine/actions/catalog.rs`

- **Path**: `src/engine/actions/catalog.rs`
- **Comments**: 2
- **Strings**: 55

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 3 | `///` | Built-in system actions, auto-activated per OS (Linux / Windows / macOS) |
| 123 | `///` | Helper to open URL safely in dedicated user browser profile |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 5 | `windows` |
| 9 | `Volume Up` |
| 10 | `Increase computer audio volume` |
| 12 | `powershell -c "(New-Object -ComObject Wscript.Shell).SendKeys([char]175)"` |
| 16 | `Volume Down` |
| 17 | `Decrease computer audio volume` |
| 19 | `powershell -c "(New-Object -ComObject Wscript.Shell).SendKeys([char]174)"` |
| 23 | `Mute Audio` |
| 24 | `Toggle mute/unmute computer audio` |
| 26 | `powershell -c "(New-Object -ComObject Wscript.Shell).SendKeys([char]173)"` |
| 30 | `Lock Screen` |
| 31 | `Lock the computer screen / workstation` |
| 32 | `rundll32.exe user32.dll,LockWorkStation` |
| 35 | `Scan Screen` |
| 36 | `Read text and buttons/icons on screen via ONNX OCR & Vision` |
| 38 | `desktop:scan_visual` |
| 43 | `macos` |
| 47 | `Volume Up` |
| 48 | `Increase macOS audio volume (+10%)` |
| 49 | `osascript -e "set volume output volume ((output volume of (get volume settings)) + 10)"` |
| 52 | `Volume Down` |
| 53 | `Decrease macOS audio volume (-10%)` |
| 54 | `osascript -e "set volume output volume ((output volume of (get volume settings)) - 10)"` |
| 57 | `Mute Audio` |
| 58 | `Toggle mute/unmute macOS audio` |
| 59 | `osascript -e "set volume output muted not (output muted of (get volume settings))"` |
| 62 | `Screenshot` |
| 63 | `Capture a screenshot on macOS` |
| 64 | `screencapture -i -c` |
| 67 | `Lock Screen` |
| 68 | `Lock the macOS screen session` |
| 69 | `pmset displaysleepnow` |
| 72 | `Scan Screen` |
| 73 | `Read text and buttons/icons on screen via ONNX OCR & Vision` |
| 74 | `desktop:scan_visual` |
| 79 | `windows` |
| 79 | `macos` |
| 83 | `Volume Up` |
| 84 | `Increase system audio volume (+10%)` |
| 85 | `pactl set-sink-volume @DEFAULT_SINK@ +10% \|\| amixer set Master 10%+` |
| 89 | `Volume Down` |
| 90 | `Decrease system audio volume (-10%)` |
| 91 | `pactl set-sink-volume @DEFAULT_SINK@ -10% \|\| amixer set Master 10%-` |
| 95 | `Mute Audio` |
| 96 | `Toggle mute/unmute system audio` |
| 97 | `pactl set-sink-mute @DEFAULT_SINK@ toggle \|\| amixer set Master toggle` |
| 101 | `Screenshot` |
| 102 | `Capture a screenshot of the monitor` |
| 103 | `gnome-screenshot \|\| spectacle \|\| flameshot gui \|\| xfce4-screenshooter` |
| 107 | `Lock Screen` |
| 108 | `Lock the computer screen session` |
| 110 | `loginctl lock-session \|\| xdg-screensaver lock \|\| gnome-screensaver-command -l` |
| 114 | `Scan Screen` |
| 115 | `Read text and buttons/icons on screen via ONNX OCR & Vision` |
| 117 | `desktop:scan_visual` |

---

## `engine/actions/desktop/click.rs`

- **Path**: `src/engine/actions/desktop/click.rs`
- **Comments**: 20
- **Strings**: 48

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 1 | `//` | ========================================== |
| 2 | `//` | MOUSE & COORDINATE ACTIONS |
| 3 | `//` | ========================================== |
| 55 | `//` | 1. If in Sway Wayland: use native Sway IPC seat cursor (highest precision and integration) |
| 77 | `//` | 2. If in Hyprland: set cursor via hyprctl movecursor |
| 90 | `//` | 3. Fallback: ydotool (Wayland uinput daemon - bekerja in GNOME, KDE, Hyprland, Sway) |
| 104 | `//` | 4. Fallback: wlrctl (standar Wayland wlroots / Hyprland) |
| 114 | `//` | 5. Fallback: xdotool (Standar X11 & XWayland) |
| 128 | `//` | 1. Try popular cliclick utility if available |
| 137 | `//` | 2. Fallback: AppleScript System Events native (without perlu install binary eksternal) |
| 165 | `//` | Priority 1: Exact Button Number ID Match |
| 168 | `//` | Priority 2: Exact Button Label Match (case-insensitive) |
| 173 | `//` | Priority 3: Starts-with match (for example "Save" matches "Save Changes") |
| 178 | `//` | Priority 4: Substring match ONLY if search length is greater than 1 character |
| 187 | `//` | Priority 5: Fuzzy Similarity Match (Typo tolerance / phonetic variation) |
| 227 | `//` | 1. Try searching existing window scanner snapshot (OS Accessibility Tree) |
| 246 | `//` | 2. Multiplatform Fallback: Run Visual OCR Perception Scan aktual |
| 247 | `//` | Useful if accessibility tree has no coordinates (e.g. Linux AT-SPI without extents), |
| 248 | `//` | canvas kustom, game, aplikasi electron without bridge accessibility, dll.) |
| 256 | `//` | Map VisualElement to DesktopElement for consistent search |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 6 | `windows` |
| 53 | `linux` |
| 56 | `SWAYSOCK` |
| 57 | `XDG_CURRENT_DESKTOP` |
| 58 | `sway` |
| 62 | `seat * cursor set {} {}, seat * cursor press button1` |
| 63 | `swaymsg` |
| 70 | `swaymsg` |
| 71 | `seat` |
| 71 | `*` |
| 71 | `cursor` |
| 71 | `release` |
| 71 | `button1` |
| 78 | `HYPRLAND_INSTANCE_SIGNATURE` |
| 79 | `XDG_CURRENT_DESKTOP` |
| 80 | `hyprland` |
| 84 | `hyprctl` |
| 85 | `dispatch` |
| 85 | `movecursor` |
| 91 | `ydotool` |
| 92 | `mousemove` |
| 92 | `-a` |
| 98 | `ydotool` |
| 99 | `click` |
| 99 | `0xC0` |
| 105 | `wlrctl` |
| 106 | `pointer` |
| 106 | `click` |
| 106 | `left` |
| 115 | `xdotool` |
| 116 | `mousemove` |
| 116 | `click` |
| 116 | `1` |
| 122 | `[Desktop Click] Failed to simulate mouse click on Linux.` |
| 126 | `macos` |
| 129 | `c:{},{}` |
| 130 | `cliclick` |
| 139 | `tell application "System Events" to click at {{{}, {}}}` |
| 142 | `osascript` |
| 143 | `-e` |
| 149 | `windows` |
| 149 | `linux` |
| 149 | `macos` |
| 203 | `[Fuzzy Match UI] Found similar element '{}' -> '{}' (score: {:.1}%)` |
| 236 | `[Desktop Click] Found via accessibility tree: '{}' at [{}, {}]` |
| 250 | `[Desktop Click] Element '{}' not found with coordinates in accessibility tree. Running visual OCR scan...` |
| 272 | `[Desktop Click] Found via visual OCR: '{}' at [{}, {}]` |
| 282 | `[Desktop Click] Element or text '{}' not found on screen (checked accessibility + visual OCR).` |

---

## `engine/actions/desktop/input.rs`

- **Path**: `src/engine/actions/desktop/input.rs`
- **Comments**: 6
- **Strings**: 75

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 1 | `//` | ========================================== |
| 2 | `//` | KEYBOARD & INPUT ACTIONS |
| 3 | `//` | ========================================== |
| 52 | `//` | 1. Coba wlrctl (Wayland wlroots / Sway) |
| 62 | `//` | 2. Coba ydotool (Wayland uinput) |
| 72 | `//` | 3. Coba xdotool (X11) |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 10 | `windows` |
| 50 | `linux` |
| 53 | `wlrctl` |
| 54 | `keyboard` |
| 54 | `type` |
| 63 | `ydotool` |
| 64 | `type` |
| 73 | `xdotool` |
| 74 | `type` |
| 74 | `--delay` |
| 74 | `20` |
| 79 | `macos` |
| 81 | `\"` |
| 83 | `tell application "System Events" to keystroke "{}"` |
| 86 | `osascript` |
| 87 | `-e` |
| 93 | `windows` |
| 93 | `linux` |
| 93 | `macos` |
| 102 | `windows` |
| 110 | `enter` |
| 110 | `return` |
| 111 | `esc` |
| 111 | `escape` |
| 112 | `tab` |
| 113 | `backspace` |
| 113 | `back` |
| 114 | `space` |
| 115 | `up` |
| 116 | `down` |
| 117 | `left` |
| 118 | `right` |
| 157 | `linux` |
| 160 | `enter` |
| 160 | `return` |
| 160 | `Return` |
| 161 | `esc` |
| 161 | `escape` |
| 161 | `Escape` |
| 162 | `tab` |
| 162 | `Tab` |
| 163 | `backspace` |
| 163 | `back` |
| 163 | `BackSpace` |
| 164 | `space` |
| 164 | `space` |
| 165 | `up` |
| 165 | `Up` |
| 166 | `down` |
| 166 | `Down` |
| 167 | `left` |
| 167 | `Left` |
| 168 | `right` |
| 168 | `Right` |
| 171 | `xdotool` |
| 172 | `key` |
| 178 | `macos` |
| 181 | `enter` |
| 181 | `return` |
| 182 | `esc` |
| 182 | `escape` |
| 183 | `tab` |
| 184 | `backspace` |
| 184 | `back` |
| 185 | `space` |
| 186 | `up` |
| 187 | `down` |
| 188 | `left` |
| 189 | `right` |
| 192 | `tell application "System Events" to key code {}` |
| 193 | `osascript` |
| 194 | `-e` |
| 200 | `windows` |
| 200 | `linux` |
| 200 | `macos` |

---

## `engine/actions/desktop/mod.rs`

- **Path**: `src/engine/actions/desktop/mod.rs`
- **Comments**: 0
- **Strings**: 48

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 14 | `scan_visual` |
| 14 | `ocr` |
| 17 | `Scan Screen` |
| 18 | `Visual screen scan running in background.` |
| 21 | `focus` |
| 25 | `Focus Window` |
| 26 | `Window '{target}' focused` |
| 30 | `Focus Window` |
| 31 | `Failed to find or focus window '{target}'` |
| 35 | `minimize` |
| 39 | `Minimize Window` |
| 40 | `Window minimized successfully.` |
| 44 | `Minimize Window` |
| 45 | `Failed to minimize window` |
| 49 | `maximize` |
| 53 | `Maximize Window` |
| 54 | `Window maximized/restored.` |
| 58 | `Maximize Window` |
| 59 | `Failed to maximize window` |
| 63 | `close` |
| 67 | `Close Window` |
| 68 | `Window '{target}' closed` |
| 72 | `Close Window` |
| 73 | `Failed to close window '{target}'` |
| 77 | `click_button` |
| 77 | `click_text` |
| 81 | `Click Desktop Element` |
| 82 | `Element/text '{target}' clicked` |
| 86 | `Click Desktop Element` |
| 87 | `Element/text '{target}' not found on screen` |
| 91 | `click_at` |
| 98 | `Click Coordinates` |
| 99 | `Clicked at ({x}, {y})` |
| 103 | `Click Coordinates` |
| 104 | `Failed to click at point ({x}, {y})` |
| 109 | `Click Coordinates` |
| 110 | `Invalid format, use desktop:click_at\|x\|y` |
| 114 | `type` |
| 117 | `Type Text` |
| 117 | `Typed text: {text}` |
| 119 | `Type Text` |
| 119 | `Failed to type text` |
| 122 | `key` |
| 125 | `Press Key` |
| 125 | `Key '{key_name}' pressed` |
| 128 | `Press Key` |
| 129 | `Key '{key_name}' not supported` |
| 138 | `active_window_app.json` |

---

## `engine/actions/desktop/window.rs`

- **Path**: `src/engine/actions/desktop/window.rs`
- **Comments**: 23
- **Strings**: 95

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 1 | `//` | ========================================== |
| 2 | `//` | WINDOW CONTROL (CROSS-PLATFORM) |
| 3 | `//` | ========================================== |
| 29 | `//` | Trik Bypass Windows 10/11 Foreground Lock: |
| 30 | `//` | Pressing the ALT key notifies the Windows kernel that the user is actively interacting, |
| 31 | `//` | so SetForegroundWindow not in-block / not only berkedip oranye in taskbar. |
| 43 | `//` | Release ALT key |
| 69 | `//` | 1. Coba Sway IPC (swaymsg) if berada in lingkungan Sway Wayland |
| 111 | `//` | 2. Coba Hyprland IPC (hyprctl) if berada in lingkungan Hyprland |
| 122 | `//` | 3. Coba wmctrl (X11 / XWayland) |
| 132 | `//` | 4. Coba xdotool (X11 / XWayland) |
| 234 | `//` | 1. Coba Sway fullscreen toggle (Sway wayland) |
| 244 | `//` | 2. Coba Hyprland fullscreen toggle |
| 254 | `//` | 3. Coba wmctrl (X11) |
| 317 | `//` | Hyprland close window by class/regex |
| 364 | `//` | 1. Check format hex window_id (for example "win:0x30354" or "0x30354") |
| 375 | `//` | 2. Check list jendela in snapshot JSON active (Ground of Truth) |
| 385 | `//` | Pure dynamic matching: |
| 386 | `//` | 1. Does the app name or title contain the query? |
| 387 | `//` | 2. Does the query contain the app name stem (for example query "zed editor" contains "zed")? |
| 388 | `//` | 3. Does any word in query match the app name or title? |
| 419 | `//` | 3. Fallback: Live EnumWindows direct to OS |
| 442 | `//` | Stop searching |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 6 | `windows` |
| 62 | `linux` |
| 70 | `sway:` |
| 71 | `swaymsg` |
| 72 | `[con_id="{}"] focus` |
| 81 | `[app_id="(?i).*{sway_target}.*"] focus` |
| 82 | `swaymsg` |
| 91 | `[title="(?i).*{sway_target}.*"] focus` |
| 92 | `swaymsg` |
| 101 | `[class="(?i).*{sway_target}.*"] focus` |
| 102 | `swaymsg` |
| 112 | `HYPRLAND_INSTANCE_SIGNATURE` |
| 113 | `hyprctl` |
| 114 | `dispatch` |
| 114 | `focuswindow` |
| 123 | `wmctrl` |
| 124 | `-a` |
| 133 | `xdotool` |
| 134 | `search` |
| 134 | `--name` |
| 134 | `windowactivate` |
| 145 | `macos` |
| 150 | `tell application "{}" to activate` |
| 151 | `osascript` |
| 152 | `-e` |
| 158 | `windows` |
| 158 | `linux` |
| 158 | `macos` |
| 166 | `windows` |
| 186 | `linux` |
| 188 | `xdotool` |
| 189 | `getactivewindow` |
| 189 | `windowminimize` |
| 194 | `macos` |
| 196 | `tell application "System Events" to set miniaturized of window 1 of (first process whose frontmost is true) to true` |
| 197 | `osascript` |
| 198 | `-e` |
| 204 | `windows` |
| 204 | `linux` |
| 204 | `macos` |
| 212 | `windows` |
| 232 | `linux` |
| 235 | `swaymsg` |
| 236 | `fullscreen` |
| 236 | `toggle` |
| 245 | `hyprctl` |
| 246 | `dispatch` |
| 246 | `fullscreen` |
| 246 | `1` |
| 255 | `wmctrl` |
| 257 | `-r` |
| 258 | `:ACTIVE:` |
| 259 | `-b` |
| 260 | `toggle,maximized_vert,maximized_horz` |
| 266 | `macos` |
| 268 | `tell application "System Events" to set zoomed of window 1 of (first process whose frontmost is true) to true` |
| 269 | `osascript` |
| 270 | `-e` |
| 276 | `windows` |
| 276 | `linux` |
| 276 | `macos` |
| 283 | `windows` |
| 304 | `linux` |
| 308 | `sway:` |
| 309 | `swaymsg` |
| 310 | `[con_id="{}"] kill` |
| 313 | `swaymsg` |
| 314 | `[app_id="(?i).*{sway_target}.*"] kill` |
| 318 | `hyprctl` |
| 319 | `dispatch` |
| 319 | `closewindow` |
| 321 | `wmctrl` |
| 322 | `-c` |
| 325 | `swaymsg` |
| 325 | `kill` |
| 326 | `hyprctl` |
| 327 | `dispatch` |
| 327 | `killactive` |
| 329 | `xdotool` |
| 330 | `getactivewindow` |
| 330 | `windowclose` |
| 336 | `macos` |
| 338 | `tell application "System Events" to click button 1 of window 1 of (first process whose frontmost is true)` |
| 339 | `osascript` |
| 340 | `-e` |
| 346 | `windows` |
| 346 | `linux` |
| 346 | `macos` |
| 352 | `windows` |
| 365 | `win:` |
| 366 | `0x` |
| 382 | `.exe` |
| 397 | `win:` |
| 398 | `0x` |
| 426 | `system` |

---

## `engine/actions/media.rs`

- **Path**: `src/engine/actions/media.rs`
- **Comments**: 8
- **Strings**: 32

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 1 | `///` | Eksekusi kontrol media lokal & background audio player (yt-dlp & internal player) |
| 15 | `//` | Check track in local directory |
| 32 | `//` | Fallback: Fuzzy Similarity Matching for judul track lokal |
| 59 | `//` | 1. Play direct lewat speaker desktop (Native Audio Player) |
| 62 | `//` | 2. Synchronize event to Web Dashboard & Smartphone Client |
| 63 | `//` | Always emit music:play event so mobile/web audio resumes playback if previously interrupted |
| 71 | `//` | Not available locally yet -> queue download & play via background worker |
| 98 | `//` | Play next track from local list |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 4 | `play` |
| 8 | `music:play` |
| 10 | `Play Music` |
| 11 | `Playing internal music` |
| 48 | `[Fuzzy Match Music] Found similar song '{}' -> '{}' (score: {:.1}%)` |
| 64 | `music:play` |
| 66 | `Play Music` |
| 67 | `Playing local music: {}` |
| 74 | `Play Music` |
| 75 | `Downloading & playing song '{}' in the background` |
| 79 | `pause` |
| 81 | `music:pause` |
| 83 | `Pause Music` |
| 84 | `Internal music paused` |
| 88 | `resume` |
| 90 | `music:resume` |
| 92 | `Resume Music` |
| 93 | `Resuming internal music` |
| 97 | `next` |
| 111 | `music:play` |
| 113 | `music:next` |
| 116 | `Next Song` |
| 117 | `Switching to next track` |
| 121 | `prev` |
| 138 | `music:play` |
| 140 | `music:prev` |
| 143 | `Previous Song` |
| 144 | `Switching to previous track` |
| 148 | `stop` |
| 150 | `music:stop` |
| 152 | `Stop Music` |
| 153 | `Internal music stopped` |

---

## `engine/actions/mod.rs`

- **Path**: `src/engine/actions/mod.rs`
- **Comments**: 2
- **Strings**: 20

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 19 | `///` | Executes skill based on canonical format \`namespace:action\|payload\` or \`namespace:action\` |
| 50 | `//` | Check Custom Skills in skill-command.md |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 39 | `recipe` |
| 39 | `run` |
| 40 | `recipe` |
| 40 | `train` |
| 40 | `recipe` |
| 40 | `learn` |
| 40 | `recipe` |
| 40 | `record` |
| 41 | `browser` |
| 42 | `media` |
| 43 | `todo` |
| 44 | `desktop` |
| 44 | `vision` |
| 45 | `os` |
| 45 | `app` |
| 46 | `internal` |
| 46 | `open_extension_installer` |
| 47 | `open_ext_folder` |
| 55 | `Executed successfully` |
| 56 | `Error: {e}` |

---

## `engine/actions/os_apps.rs`

- **Path**: `src/engine/actions/os_apps.rs`
- **Comments**: 33
- **Strings**: 125

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 5 | `///` | Combined handler for OS system control and application launching |
| 14 | `//` | 1. Pure OS utility action |
| 24 | `//` | 2. All application requests (both payload and action) are routed to dynamic app scanner |
| 40 | `///` | Executes native OS system action by action name |
| 57 | `///` | Eksekusi launcher aplikasi OS which terinstal |
| 58 | `///` | Format: \`app:open\|<name>\` |
| 67 | `//` | 1. Search application based on matching hierarchy (Exact -> Contains -> Aliases -> Fuzzy) |
| 77 | `//` | 2. Fallback: Directly execute binary name in system PATH if available |
| 133 | `///` | Search installed applications using a 4-tier hierarchy: |
| 134 | `///` | 1. Exact match (name / desktop_id / generic_name) |
| 135 | `///` | 2. Substring & StartsWith match |
| 136 | `///` | 3. Generic category alias (terminal -> foot, browser -> chrome, editor -> zed) |
| 137 | `///` | 4. Fuzzy Levenshtein similarity (>= 70%) |
| 147 | `//` | Clean label formats like "Foot (Terminal)" -> ["Foot", "Terminal"] |
| 155 | `//` | Priority 1: Exact match (case-insensitive) |
| 173 | `//` | Priority 2: Substring & StartsWith match |
| 192 | `//` | Priority 3: Generic category alias matching (terminal -> foot, browser -> chrome, etc.) |
| 202 | `//` | Priority 4: Fuzzy Levenshtein Similarity (threshold >= 0.70) |
| 238 | `///` | Peluncur aplikasi multiplatform, multidistro, multi-DE, and multi-WM |
| 296 | `//` | 1. Check Compositor Wayland: Sway (Sway IPC seat spawn) |
| 312 | `//` | 2. Check Compositor Wayland: Hyprland (Hyprland IPC dispatcher) |
| 328 | `//` | 3. Check Window Manager X11: i3 |
| 344 | `//` | 4. Check XDG Desktop Environment standard launcher (GNOME / KDE / XFCE / MATE) |
| 373 | `//` | 5. Universal Detached Fallback (bekerja in all distro & lingkungan desktop) |
| 384 | `//` | Check Sway |
| 396 | `//` | Check Hyprland |
| 408 | `//` | Check i3 |
| 420 | `//` | Fallback universal detached spawn |
| 473 | `//` | 1. Exact Name |
| 476 | `//` | 2. Generic Name / Alias |
| 479 | `//` | 3. Label with Parenthesis from prompt ground of truth |
| 485 | `//` | 4. Substring & Generic |
| 495 | `//` | 5. Fuzzy typo |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 16 | `volume_up` |
| 16 | `Volume Up` |
| 17 | `volume_down` |
| 17 | `Volume Down` |
| 18 | `mute` |
| 18 | `Mute Audio` |
| 19 | `lock` |
| 19 | `Lock Screen` |
| 20 | `screenshot` |
| 20 | `Screenshot` |
| 27 | `app` |
| 27 | `os` |
| 42 | `os` |
| 49 | `Executed successfully` |
| 50 | `Error: {e}` |
| 71 | `Open {}` |
| 72 | `Successfully opened application '{}'` |
| 80 | `windows` |
| 84 | `Open {q_clean}` |
| 85 | `Successfully launched application from system` |
| 88 | `explorer.exe "{}"` |
| 91 | `Open {q_clean}` |
| 92 | `Successfully launched application from system` |
| 97 | `macos` |
| 99 | `open -a "{q_clean}" 2>/dev/null \|\| ({q_clean} &)` |
| 100 | `sh` |
| 101 | `-c` |
| 105 | `Open {q_clean}` |
| 106 | `Successfully launched application from system` |
| 111 | `windows` |
| 111 | `macos` |
| 113 | `-` |
| 114 | `command -v {binary_candidate} >/dev/null 2>&1` |
| 115 | `sh` |
| 116 | `-c` |
| 123 | `Open {q_clean}` |
| 124 | `Successfully launched application from system` |
| 227 | `[Fuzzy Match App] Found similar application '{}' -> '{}' (score: {:.1}%)` |
| 240 | `windows` |
| 245 | `macos` |
| 247 | `open -a "{}" 2>/dev/null \|\| ({})` |
| 248 | `sh` |
| 249 | `-c` |
| 255 | `windows` |
| 255 | `macos` |
| 261 | `windows` |
| 287 | `cmd` |
| 288 | `/S` |
| 288 | `/C` |
| 288 | `"{trimmed}"` |
| 294 | `windows` |
| 294 | `macos` |
| 297 | `SWAYSOCK` |
| 298 | `swaymsg` |
| 299 | `exec` |
| 299 | `--` |
| 304 | `[Linux Launcher] Opening '{}' via Sway IPC (swaymsg exec)` |
| 313 | `HYPRLAND_INSTANCE_SIGNATURE` |
| 314 | `hyprctl` |
| 315 | `dispatch` |
| 315 | `exec` |
| 315 | `--` |
| 320 | `[Linux Launcher] Opening '{}' via Hyprland IPC (hyprctl dispatch exec)` |
| 329 | `I3SOCK` |
| 330 | `i3-msg` |
| 331 | `exec` |
| 336 | `[Linux Launcher] Opening '{}' via i3 IPC (i3-msg exec)` |
| 346 | `gtk-launch` |
| 352 | `[Linux Launcher] Opening '{}' via gtk-launch ({})` |
| 362 | `gio` |
| 363 | `launch` |
| 367 | `[Linux Launcher] Opening '{}' via gio launch` |
| 376 | `[Linux Launcher] Opening '{}' via background detached process` |
| 382 | `windows` |
| 382 | `macos` |
| 385 | `SWAYSOCK` |
| 386 | `swaymsg` |
| 387 | `exec` |
| 387 | `--` |
| 397 | `HYPRLAND_INSTANCE_SIGNATURE` |
| 398 | `hyprctl` |
| 399 | `dispatch` |
| 399 | `exec` |
| 399 | `--` |
| 409 | `I3SOCK` |
| 410 | `i3-msg` |
| 411 | `exec` |
| 421 | `nohup {cmd_str} >/dev/null 2>&1 &` |
| 422 | `sh` |
| 423 | `-c` |
| 438 | `Foot` |
| 439 | `foot` |
| 441 | `A wayland native terminal emulator` |
| 442 | `Terminal` |
| 444 | `shell` |
| 445 | `command` |
| 446 | `terminalemulator` |
| 448 | `foot` |
| 452 | `Google Chrome` |
| 453 | `google-chrome` |
| 455 | `Access the internet` |
| 456 | `Web Browser` |
| 457 | `internet` |
| 457 | `webbrowser` |
| 458 | `google-chrome` |
| 462 | `Zed` |
| 463 | `zed` |
| 466 | `Text Editor` |
| 467 | `code` |
| 467 | `texteditor` |
| 468 | `dev.zed.Zed` |
| 474 | `Foot` |
| 474 | `Foot` |
| 477 | `terminal` |
| 477 | `Foot` |
| 481 | `Foot (Terminal)` |
| 482 | `Foot` |
| 487 | `chrome` |
| 488 | `Google Chrome` |
| 491 | `browser` |
| 492 | `Google Chrome` |
| 496 | `termnl` |
| 496 | `Foot` |
| 497 | `zedd` |
| 497 | `Zed` |

---

## `engine/actions/recipe/mod.rs`

- **Path**: `src/engine/actions/recipe/mod.rs`
- **Comments**: 0
- **Strings**: 0

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

_No string literals found._

---

## `engine/actions/recipe/runner.rs`

- **Path**: `src/engine/actions/recipe/runner.rs`
- **Comments**: 39
- **Strings**: 44

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 11 | `///` | Calculates simple similarity between two texts (case-insensitive substring & token match) |
| 24 | `//` | Token overlap match |
| 39 | `///` | Search target klik terbaik with prioritas: |
| 40 | `///` | Search target klik terbaik with prioritas: |
| 41 | `///` | 1. Search dulu in \`snapshot.buttons\` using selector_primary. |
| 42 | `///` | 2. If not found, search each candidate in \`fallback_selectors\` on buttons. |
| 43 | `///` | 3. If not found yet, search for matching label in \`snapshot.buttons\` (Buttons First). |
| 44 | `///` | 4. If still no match in buttons, search in \`snapshot.texts\` (Texts Fallback). |
| 65 | `//` | --- 1. PRIMARY SELECTOR IN SNAPSHOT BUTTONS --- |
| 76 | `//` | --- 1b. FALLBACK SELECTORS IN SNAPSHOT BUTTONS (Multi-Strategy) --- |
| 89 | `//` | --- 1c. TARGET_TEXT ON BUTTONS --- |
| 116 | `//` | --- 2. TEXTS FALLBACK (Use target_text to click text) --- |
| 140 | `//` | --- 3. DEFAULT FALLBACK --- |
| 154 | `///` | Search selector elemen terbaik from snapshot active if selector utama not found or berubah (khusus input/elemen spesi... |
| 171 | `//` | Check each alternative selector in fallback_selectors |
| 213 | `///` | Waits until browser reports new snapshot or page matching target domain loads |
| 238 | `///` | Reactively waits until the target element for this step appears in DOM snapshot. |
| 239 | `///` | Poll until timeout (e.g. 6s). If already available, continue immediately without delay! |
| 327 | `///` | Reactively wait and ping-pong with browser: |
| 328 | `///` | Send REFRESH_SNAPSHOT to extension, then wait until snapshot timestamp > previous_ts |
| 329 | `///` | and page is in settled state (not loading and pending requests = 0). |
| 335 | `//` | Direct tembak REFRESH_SNAPSHOT pertama (Ping) |
| 344 | `//` | Re-ping every 250ms if browser has not responded yet |
| 366 | `///` | Executes a single browser automation step |
| 491 | `///` | Executes automation recipe autonomously (Local Repetitive Worker) |
| 492 | `///` | Payload format: \`recipe_id\` or \`recipe_id\|{"param1": "value1"}\` |
| 516 | `//` | Parse dynamic parameters (if any) |
| 539 | `//` | Validasi parameter ketat |
| 582 | `//` | Execute steps in background thread without blocking main chat loop (Silent Worker) |
| 586 | `//` | Step 0: Open Tab New (Isolated Clean State) sesuai aturan mandatory |
| 595 | `//` | Tunggu snapshot active terbaru from active_tab_browser.json |
| 601 | `//` | Execute each step sequentially and quietly in the background |
| 612 | `//` | Wait for target element to be ready in DOM before executing action |
| 618 | `//` | 1. PING-PONG REAKTIF: Minta paksa snapshot new and tunggu DOM merespon |
| 621 | `//` | 2. VERIFIKASI HASIL EKSEKUSI (Loop Check ping-pong): |
| 622 | `//` | If action is "type", ensure DOM input strictly contains the typed text value |
| 645 | `//` | Refresh snapshot repeat if not yet sync |
| 657 | `//` | 3. ANTI RACE CONDITION: If there is a subsequent step, verify that the subsequent element |
| 658 | `//` | has strictly appeared in DOM before declaring this step complete and proceeding! |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 59 | `button` |
| 60 | `a` |
| 61 | `div` |
| 97 | `{} {}` |
| 189 | `{} {} {}` |
| 226 | `[Recipe Runner] Page ready: '{}' (url: {})` |
| 246 | `type` |
| 338 | `REFRESH_SNAPSHOT` |
| 348 | `REFRESH_SNAPSHOT` |
| 379 | `click` |
| 389 | `   [Click Target: Button] {}` |
| 392 | `CLICK_BUTTON` |
| 400 | `   [Click Target: Text Fallback] "{}"` |
| 403 | `CLICK_TEXT` |
| 411 | `[Step Executor] No button or text found for step` |
| 416 | `type` |
| 419 | `{{{{{}}}}}` |
| 441 | `CLICK_TEXT` |
| 452 | `TYPE_TEXT` |
| 462 | `enter` |
| 465 | `PRESS_ENTER` |
| 474 | `scroll` |
| 478 | `SCROLL` |
| 486 | `[Step Executor] Unknown step action: {}` |
| 508 | `[Recipe Runner] Recipe '{}' not found in assets/recipes/` |
| 510 | `Recipe Not Found: {recipe_id}` |
| 511 | `Recipe '{recipe_id}' is not registered` |
| 546 | `query` |
| 547 | `keyword` |
| 548 | `keyword` |
| 549 | `query` |
| 557 | `[Recipe Runner] Recipe '{}' requires parameter '{}', but it was missing in payload: '{}'` |
| 563 | `Missing Parameter: {req}` |
| 565 | `Recipe '{}' requires parameter '{}' (e.g. [CMD:recipe:run\|{}\|{{"{}": "..."}}])` |
| 573 | `[Recipe Runner] Executing recipe '{}' ({}) with {} params...` |
| 587 | `open_new_tab` |
| 589 | `[Recipe Runner] Opening new tab to: {}` |
| 605 | `  ▶ Step {}/{}: {} ({})` |
| 623 | `type` |
| 626 | `{{{{{}}}}}` |
| 648 | `REFRESH_SNAPSHOT` |
| 664 | `[Recipe Runner] Recipe '{}' completed successfully in background!` |
| 668 | `Run Recipe: {recipe_name}` |
| 669 | `Executing recipe '{recipe_name}'` |

---

## `engine/actions/recipe/trainer.rs`

- **Path**: `src/engine/actions/recipe/trainer.rs`
- **Comments**: 38
- **Strings**: 134

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 11 | `///` | Calculates representative hash from DOM state to detect if page actually changed |
| 36 | `///` | Helper function to strip markdown code blocks from LLM JSON output |
| 53 | `///` | Fixes common JSON output errors from LLM: |
| 54 | `///` | - Escaped quotes liar in array: e.g. \`\"button[...]\"\` |
| 55 | `///` | - Trailing commas on array/object: e.g. \`[1, 2,]\` |
| 56 | `///` | - Unquoted string values: e.g. "recipe_name": Video Title, |
| 58 | `//` | 1. Clean stray escaped quotes placed outside strings |
| 68 | `//` | 2. Clean trailing commas: \`, ]\` -> \`]\` and \`, }\` -> \`}\` |
| 110 | `///` | Fallback parser toleran (Lenient Regex / Lexer): |
| 111 | `///` | If serde_json fails to parse broken LLM output, extract primary fields manually |
| 115 | `//` | Helper to extract string value by key |
| 140 | `//` | Unquoted string fallback |
| 159 | `//` | If status DONE, not perlu next_step |
| 171 | `//` | Ekstrak next_step |
| 178 | `//` | Extract fallback_selectors from array if present |
| 222 | `///` | Diagnoses, prunes, and optimizes recorded steps before writing to recipe JSON |
| 327 | `///` | Worker Train / Interactive Recorder (Agentic Training Loop): |
| 328 | `///` | Interactively explores web pages step-by-step (Reasoning Loop): |
| 329 | `///` | 1. Take actual DOM snapshot at each step. |
| 330 | `///` | 2. LLM Planner reasons to determine the next single action. |
| 331 | `///` | 3. Action is executed directly in real browser. |
| 332 | `///` | 4. Page changed -> loop continues reading new DOM until goal is reached (status: "DONE"). |
| 333 | `///` | 5. All step traces are compiled into 1 AutomationRecipe JSON blueprint for future 0-token execution! |
| 367 | `//` | Step 0: Open Tab New Clean (Isolated Clean State) |
| 384 | `//` | Tunggu halaman start termuat sesuai target domain |
| 414 | `//` | Take latest DOM snapshot from active page |
| 423 | `//` | Summarize current interactive elements |
| 545 | `//` | Check whether LLM states flow is already complete |
| 551 | `//` | Execute next action if status is CONTINUE |
| 553 | `//` | 1. Detect consecutive duplicate "type" actions on the same element without submit action |
| 569 | `//` | Auto-adopt target_text and fallback_selectors direct from snapshot DOM |
| 611 | `//` | Eksekusi aksi nyata in browser |
| 617 | `//` | Reactively wait until DOM timestamp is updated by background extension (max 2500ms) |
| 628 | `//` | 2. Validate DOM mutation for 'click' action: if no mutation occurred, do not add to steps! |
| 642 | `//` | Valid action that produced results: record to recipe! |
| 672 | `//` | Compile all successfully recorded steps into a complete Recipe |
| 674 | `//` | Step Diagnosis & Optimization Phase (Pruning redundant clicks/inputs) |
| 694 | `//` | Determine required_params strictly: inspect {{...}} templates across all steps |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 39 | `\`\`\`json` |
| 40 | `\`\`\`` |
| 44 | `\`\`\`` |
| 45 | `\`\`\`` |
| 60 | `, \"` |
| 60 | `, "` |
| 61 | `[\"` |
| 61 | `["` |
| 62 | `\"]` |
| 62 | `"]` |
| 63 | `\" ` |
| 63 | `" ` |
| 64 | `\",` |
| 64 | `",` |
| 65 | `: \"` |
| 65 | `: "` |
| 66 | `\", ` |
| 66 | `", ` |
| 69 | `,\n]` |
| 69 | `,\n }` |
| 69 | `,]` |
| 69 | `,}` |
| 70 | `,\n]` |
| 70 | `\n]` |
| 71 | `,\n }` |
| 71 | `\n }` |
| 72 | `,]` |
| 72 | `]` |
| 73 | `,}` |
| 73 | `}` |
| 92 | `true` |
| 93 | `false` |
| 94 | `null` |
| 99 | `,` |
| 100 | `{indent}{key_trim}: "{val_body}"{comma_str}` |
| 107 | `\n` |
| 117 | `"{key}"` |
| 143 | `null` |
| 147 | `status` |
| 148 | `"DONE"` |
| 148 | `STATUS: DONE` |
| 149 | `DONE` |
| 151 | `CONTINUE` |
| 155 | `reasoning` |
| 156 | `recipe_id` |
| 157 | `recipe_name` |
| 160 | `DONE` |
| 172 | `action` |
| 174 | `selector` |
| 175 | `target_text` |
| 176 | `value` |
| 180 | `"fallback_selectors"` |
| 200 | `{{query}}` |
| 200 | `query` |
| 239 | `You are an expert Web Automation Quality Engineer and Recipe Optimizer. Your task is to audit, diagnose, and streamli...` |
| 277 | `USER GOAL: "{user_goal}"\nTARGET DOMAIN: "{domain}"\nINITIAL URL: "{initial_url}"\n\nRECORDED RAW STEPS ({count} step...` |
| 283 | `system` |
| 287 | `user` |
| 292 | `[Recipe Diagnostic Pass] Auditing and optimizing {} recorded steps...` |
| 300 | `[Recipe Diagnosis Summary]: {}` |
| 306 | `[Recipe Optimized]: Refined steps from {} -> {}` |
| 312 | `[Recipe Diagnosis]: Diagnostic returned empty steps array, falling back to recorded steps.` |
| 316 | `[Recipe Diagnosis]: Failed to parse diagnosis JSON, falling back to recorded steps.` |
| 321 | `[Recipe Diagnosis]: Failed to complete diagnosis prompt: {e}, falling back to recorded steps.` |
| 341 | `url` |
| 342 | `goal` |
| 347 | `Automate page task` |
| 351 | `[Recipe Train] Missing target URL in payload: {payload}` |
| 356 | `[Recipe Train Loop] Starting interactive training loop for URL: '{}' \| Goal: '{}'` |
| 368 | `[Recipe Train Loop] Step 0: Opening clean tab to: {}` |
| 376 | `https://` |
| 377 | `http://` |
| 378 | `www.` |
| 388 | `id` |
| 390 | `Starting recipe training for {clean_domain}.` |
| 392 | `Starting to learn workflow for {clean_domain}.` |
| 394 | `excited` |
| 409 | `[Recipe Train Loop] Iteration {}/{} — Inspecting current DOM state...` |
| 418 | `[Recipe Train Loop] Failed to acquire page snapshot from browser` |
| 429 | `- button: "{}" \| selector: "{}"` |
| 440 | `- input: "{}" \| selector: "{}"` |
| 446 | `- "{}"` |
| 449 | `(None — this is the first step on the landing page)` |
| 451 | `\n` |
| 455 | `PAGE CONTEXT:\nDomain: {}\nTitle: {}\nCurrent URL: {}\n\nPREVIOUSLY EXECUTED STEPS:\n{}\n\nDETECTED INTERACTIVE INPUT...` |
| 460 | `(none)` |
| 460 | `\n` |
| 461 | `(none)` |
| 461 | `\n` |
| 462 | `(none)` |
| 462 | `\n` |
| 465 | `You are an expert interactive browser automation trainer (Agentic Recorder). Your role: Look at the current live web ...` |
| 491 | `\nCRITICAL OBSERVATION FROM LAST STEP:\n {}\n` |
| 497 | `OVERALL USER GOAL: "{}"\n\n{}{}` |
| 505 | `system` |
| 509 | `user` |
| 521 | `[Recipe Train Loop] Standard JSON parser failed, attempting lenient regex parser...` |
| 529 | `   [Train Iteration {} Reasoning]: {}` |
| 533 | `Step {}: {}` |
| 546 | `DONE` |
| 547 | `[Recipe Train Loop] Goal accomplished according to LLM reasoning!` |
| 554 | `type` |
| 556 | `type` |
| 564 | `[Train Validator] Skipped duplicate 'type' step on same element without intermediate submit/click.` |
| 565 | `The query text is ALREADY typed into the search box in the previous step! Do NOT type again. Your next step MUST be s...` |
| 570 | `click` |
| 584 | `type` |
| 602 | `  ▶ [Executing Live Action] Step {}: {} ({})` |
| 613 | `query` |
| 614 | `keyword` |
| 624 | `[DOM Reactive] New DOM snapshot detected (ts: {}), advancing immediately!` |
| 629 | `click` |
| 631 | `[Train Validator] Click on '{}' did NOT produce any DOM/URL change (hash unchanged: {}). Skipping step from recording.` |
| 636 | `Clicking on '{}' did NOT change the page DOM or URL. The element might not be interactive or didn't respond. Try a di...` |
| 645 | `Step {}: {} ({})` |
| 650 | `Step {} completed, {}` |
| 651 | `happy` |
| 655 | `[Recipe Train Loop] No next_step provided, ending loop.` |
| 660 | `[Recipe Train Loop] Failed to parse decision JSON from LLM.\nRaw: {clean_json}` |
| 666 | `[Recipe Train Loop] LLM call failed: {e}` |
| 685 | `recipe_{}` |
| 710 | `open_new_tab` |
| 716 | `{}.json` |
| 721 | `[Recipe Train Complete] Compiled {} interactive steps into '{:?}'! Future runs will cost 0 tokens.` |
| 725 | `All steps successfully learned and saved to recipe {}!` |
| 726 | `excited` |
| 730 | `[Recipe Train Complete] No steps were recorded.` |
| 735 | `Train Recipe: {user_goal}` |
| 736 | `Interactive training loop started for {target_url}` |
| 746 | `{   "reasoning": "Looking for video",   "status": "CONTINUE",   "recipe_id": "search_and_play_youtube_video",   "reci...` |
| 760 | `"recipe_name": "Search and play video on YouTube",` |
| 763 | `Failed to parse repaired json: {:?}` |
| 765 | `Search and play video on YouTube` |
| 766 | `CONTINUE` |

---

## `engine/actions/recipe/types.rs`

- **Path**: `src/engine/actions/recipe/types.rs`
- **Comments**: 9
- **Strings**: 9

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 15 | `//` | "click", "type", "enter", "scroll" |
| 44 | `///` | Step-by-step decision structure in Agentic Training Loop |
| 49 | `//` | "CONTINUE" or "DONE" |
| 60 | `///` | Extracts all parameter names found within \`{{param_name}}\` templates in steps |
| 84 | `//` | Selector or button identifier |
| 85 | `//` | Fallback text string found in snapshot.texts |
| 88 | `///` | Gets storage directory for recipes: assets/recipes/ |
| 93 | `///` | Load all recipes stored in assets/recipes/*.json |
| 118 | `///` | Search recipe by recipe_id |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 18 | `Vec::is_empty` |
| 20 | `Option::is_none` |
| 22 | `Option::is_none` |
| 32 | `default_version` |
| 66 | `{{` |
| 67 | `}}` |
| 90 | `assets` |
| 90 | `recipes` |
| 104 | `json` |

---

## `engine/actions/todo_actions.rs`

- **Path**: `src/engine/actions/todo_actions.rs`
- **Comments**: 1
- **Strings**: 8

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 1 | `///` | Eksekusi perintah To-Do berbasis canonical action |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 4 | `add` |
| 10 | `Note Task` |
| 11 | `Note Task` |
| 11 | `Error: {e}` |
| 15 | `done` |
| 21 | `Task Done` |
| 22 | `Task Done` |
| 22 | `Error: {e}` |

---

## `engine/app_scanner.rs`

- **Path**: `src/engine/app_scanner.rs`
- **Comments**: 102
- **Strings**: 250

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 23 | `//` | Unicode-safe comparison: covers non-ASCII names (Cyrillic, CJK, etc.) |
| 32 | `///` | Fetches installed application list with 60s memory cache to avoid disk I/O overhead |
| 47 | `//` | ── Category word helpers ───────────────────────────────────────────────────── |
| 48 | `//` | Each fn returns true when the given word (already lowercased) belongs to that |
| 49 | `//` | category in any of the 30 languages supported by Chrome STT / SUPPORTED_LANGUAGES. |
| 54 | `//` | Latin-script languages |
| 56 | `//` | id |
| 58 | `//` | de, fr, es, pt, pl, nl, sv, da, cs, sk, sl, ro, hr, hu, lt, lv, et, fi, vi — all use "terminal" |
| 59 | `//` | fi |
| 61 | `//` | cs, sk, hu |
| 63 | `//` | Non-Latin scripts |
| 64 | `//` | ja |
| 65 | `//` | ko |
| 66 | `//` | ar |
| 67 | `//` | ru, uk, bg |
| 68 | `//` | zh |
| 69 | `//` | hi |
| 70 | `//` | el |
| 78 | `//` | id |
| 79 | `//` | fr |
| 80 | `//` | es, pt |
| 81 | `//` | sv |
| 82 | `//` | fi |
| 83 | `//` | cs |
| 84 | `//` | sk |
| 85 | `//` | sl |
| 86 | `//` | hr |
| 87 | `//` | hu |
| 88 | `//` | lt |
| 89 | `//` | lv |
| 90 | `//` | pl |
| 91 | `//` | vi |
| 92 | `//` | ja |
| 93 | `//` | ko |
| 94 | `//` | ar |
| 95 | `//` | ru, uk |
| 96 | `//` | bg |
| 97 | `//` | zh |
| 98 | `//` | hi |
| 99 | `//` | el |
| 107 | `//` | id |
| 108 | `//` | fr |
| 109 | `//` | sv |
| 110 | `//` | fi |
| 111 | `//` | sl |
| 112 | `//` | hr |
| 113 | `//` | hu |
| 114 | `//` | lt |
| 115 | `//` | lv |
| 116 | `//` | vi |
| 117 | `//` | ja |
| 118 | `//` | ko |
| 119 | `//` | ar |
| 120 | `//` | ru, uk, bg |
| 121 | `//` | zh |
| 122 | `//` | hi |
| 123 | `//` | el |
| 131 | `//` | id |
| 132 | `//` | de |
| 133 | `//` | fr |
| 134 | `//` | es |
| 135 | `//` | pl |
| 136 | `//` | nl |
| 137 | `//` | sv, da, no |
| 138 | `//` | fi |
| 139 | `//` | cs |
| 140 | `//` | sk |
| 141 | `//` | sl, hr |
| 142 | `//` | ro |
| 143 | `//` | hu |
| 144 | `//` | lt |
| 145 | `//` | lv |
| 146 | `//` | et |
| 147 | `//` | vi |
| 148 | `//` | ja |
| 149 | `//` | ko |
| 150 | `//` | ar |
| 151 | `//` | ru, uk, bg |
| 152 | `//` | zh |
| 153 | `//` | hi |
| 154 | `//` | el |
| 158 | `//` | ── App matching logic ───────────────────────────────────────────────────────── |
| 215 | `///` | Checks whether application matches search keyword including generic category aliases |
| 232 | `//` | Generic category aliases across all supported languages |
| 249 | `///` | Checks whether an application is mentioned in user input sentence in a language-agnostic manner |
| 256 | `//` | 1. Name lengkap aplikasi muncul in query (for example: "google chrome", "visual studio code", "zed", "spotify") |
| 261 | `//` | 2. Token utama name aplikasi (length >= 3, not word umum/noise) |
| 264 | `//` | Noise: common short words across supported languages that appear in app names |
| 267 | `//` | en |
| 268 | `//` | de |
| 269 | `//` | fr |
| 270 | `//` | es |
| 271 | `//` | nl |
| 272 | `//` | sv |
| 279 | `//` | 3. Desktop ID (for example: "dev.zed.Zed" -> "zed", "google-chrome" -> "chrome") |
| 288 | `//` | 4. Kategori generik umum lintas bahasa (terminal, browser, editor, file manager) |
| 313 | `///` | Formats installed applications list with index filtering based on user input context |
| 394 | `//` | Essential Windows built-in applications (guaranteed to exist & directly accessible) |
| 424 | `//` | Scan shortcuts (.lnk and .url) from folder Start Menu |
| 427 | `//` | Start Menu all user |
| 435 | `//` | Current user Start Menu |
| 503 | `//` | \`open\` is cara standar open .app bundle in macOS |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 25 | `{} ({})` |
| 55 | `terminal` |
| 55 | `console` |
| 55 | `shell` |
| 55 | `bash` |
| 57 | `konsol` |
| 60 | `pääte` |
| 62 | `terminál` |
| 64 | `ターミナル` |
| 65 | `터미널` |
| 66 | `طرفية` |
| 66 | `طرفي` |
| 67 | `терминал` |
| 68 | `终端` |
| 68 | `終端` |
| 69 | `टर्मिनल` |
| 70 | `τερματικό` |
| 77 | `browser` |
| 77 | `web` |
| 77 | `internet` |
| 78 | `peramban` |
| 79 | `navigateur` |
| 80 | `navegador` |
| 81 | `webbläsare` |
| 82 | `selain` |
| 83 | `prohlížeč` |
| 84 | `prehliadač` |
| 85 | `brskalnik` |
| 86 | `preglednik` |
| 87 | `böngésző` |
| 88 | `naršyklė` |
| 89 | `pārlūks` |
| 90 | `przeglądarka` |
| 91 | `trình duyệt` |
| 92 | `ブラウザ` |
| 92 | `ブラウザー` |
| 93 | `브라우저` |
| 94 | `متصفح` |
| 95 | `браузер` |
| 96 | `браузър` |
| 97 | `浏览器` |
| 97 | `瀏覽器` |
| 98 | `ब्राउज़र` |
| 99 | `πρόγραμμα περιήγησης` |
| 106 | `editor` |
| 106 | `code` |
| 106 | `text` |
| 107 | `kode` |
| 107 | `teks` |
| 108 | `éditeur` |
| 109 | `textredigerare` |
| 110 | `tekstieditori` |
| 111 | `urejevalnik` |
| 112 | `uređivač` |
| 113 | `szövegszerkesztő` |
| 114 | `redaktorius` |
| 115 | `redaktors` |
| 116 | `trình soạn thảo` |
| 117 | `エディタ` |
| 117 | `エディター` |
| 118 | `편집기` |
| 119 | `محرر` |
| 120 | `редактор` |
| 121 | `编辑器` |
| 121 | `編輯器` |
| 122 | `संपादक` |
| 123 | `επεξεργαστής` |
| 130 | `file` |
| 130 | `folder` |
| 130 | `explorer` |
| 130 | `manager` |
| 131 | `berkas` |
| 132 | `datei` |
| 132 | `dateien` |
| 133 | `fichier` |
| 133 | `fichiers` |
| 134 | `archivo` |
| 134 | `archivos` |
| 135 | `pliki` |
| 135 | `plik` |
| 136 | `bestand` |
| 136 | `bestanden` |
| 137 | `filer` |
| 137 | `fil` |
| 138 | `tiedosto` |
| 138 | `tiedostot` |
| 139 | `soubor` |
| 139 | `soubory` |
| 140 | `súbor` |
| 140 | `súbory` |
| 141 | `datoteka` |
| 141 | `datoteke` |
| 142 | `fișier` |
| 142 | `fișiere` |
| 143 | `fájl` |
| 143 | `fájlok` |
| 144 | `failas` |
| 144 | `failai` |
| 145 | `fails` |
| 145 | `faili` |
| 146 | `fail` |
| 146 | `failid` |
| 147 | `tệp` |
| 148 | `ファイル` |
| 149 | `파일` |
| 150 | `ملفات` |
| 150 | `ملف` |
| 151 | `файлы` |
| 151 | `файл` |
| 152 | `文件` |
| 152 | `檔案` |
| 153 | `फ़ाइल` |
| 154 | `αρχεία` |
| 154 | `αρχείο` |
| 161 | `terminal` |
| 162 | `terminal` |
| 163 | `foot` |
| 164 | `alacritty` |
| 165 | `kitty` |
| 166 | `wezterm` |
| 167 | `konsole` |
| 168 | `xterm` |
| 172 | `terminal` |
| 172 | `terminalemulator` |
| 176 | `browser` |
| 177 | `chrome` |
| 178 | `firefox` |
| 179 | `chromium` |
| 180 | `brave` |
| 181 | `edge` |
| 185 | `webbrowser` |
| 189 | `editor` |
| 190 | `code` |
| 191 | `zed` |
| 192 | `gedit` |
| 193 | `kate` |
| 194 | `sublime` |
| 198 | `texteditor` |
| 202 | `file manager` |
| 203 | `thunar` |
| 204 | `nautilus` |
| 205 | `dolphin` |
| 206 | `pcmanfm` |
| 207 | `nemo` |
| 208 | `explorer` |
| 212 | `filemanager` |
| 267 | `the` |
| 267 | `app` |
| 267 | `pro` |
| 267 | `for` |
| 267 | `and` |
| 268 | `de` |
| 268 | `von` |
| 268 | `für` |
| 269 | `le` |
| 269 | `la` |
| 269 | `les` |
| 269 | `et` |
| 270 | `el` |
| 270 | `los` |
| 270 | `las` |
| 270 | `por` |
| 271 | `van` |
| 271 | `voor` |
| 272 | `för` |
| 281 | `.desktop` |
| 339 | `\n###  INSTALLED SYSTEM APPLICATIONS (GROUND OF TRUTH):\n\         * Can be opened directly with \`[CMD:app:open\|AppNa...` |
| 340 | `, ` |
| 348 | `linux` |
| 352 | `/usr/share/applications` |
| 353 | `/usr/local/share/applications` |
| 354 | `/var/lib/flatpak/exports/share/applications` |
| 355 | `/var/lib/snapd/desktop/applications` |
| 358 | `HOME` |
| 360 | `.local/share/applications` |
| 361 | `.local/share/flatpak/exports/share/applications` |
| 373 | `desktop` |
| 389 | `windows` |
| 396 | `Calculator` |
| 396 | `calc.exe` |
| 397 | `Notepad` |
| 397 | `notepad.exe` |
| 398 | `Paint` |
| 398 | `mspaint.exe` |
| 399 | `Task Manager` |
| 399 | `taskmgr.exe` |
| 400 | `Settings` |
| 400 | `explorer.exe ms-settings:` |
| 401 | `File Explorer` |
| 401 | `explorer.exe` |
| 402 | `Command Prompt` |
| 402 | `cmd.exe` |
| 403 | `PowerShell` |
| 403 | `powershell.exe` |
| 404 | `Microsoft Edge` |
| 404 | `explorer.exe microsoft-edge:` |
| 429 | `ProgramData` |
| 429 | `ALLUSERSPROFILE` |
| 432 | `Microsoft\Windows\Start Menu\Programs` |
| 436 | `APPDATA` |
| 438 | `Microsoft\Windows\Start Menu\Programs` |
| 439 | `USERPROFILE` |
| 442 | `AppData\Roaming\Microsoft\Windows\Start Menu\Programs` |
| 453 | `macos` |
| 459 | `/Applications` |
| 460 | `/System/Applications` |
| 461 | `/System/Applications/Utilities` |
| 464 | `HOME` |
| 465 | `Applications` |
| 478 | `macos` |
| 493 | `app` |
| 504 | `open -a "{}"` |
| 520 | `windows` |
| 536 | `lnk` |
| 536 | `url` |
| 540 | `uninstall` |
| 541 | `readme` |
| 542 | `release note` |
| 543 | `help` |
| 544 | `documentation` |
| 553 | `explorer.exe "{}"` |
| 570 | `linux` |
| 586 | `[Desktop Entry]` |
| 598 | `Type` |
| 599 | `Application` |
| 603 | `NoDisplay` |
| 603 | `Hidden` |
| 604 | `true` |
| 608 | `Name` |
| 611 | `GenericName` |
| 614 | `Keywords` |
| 622 | `Categories` |
| 630 | `Exec` |
| 633 | `%f` |
| 634 | `%F` |
| 635 | `%u` |
| 636 | `%U` |
| 637 | `%d` |
| 638 | `%D` |
| 639 | `%n` |
| 640 | `%N` |
| 641 | `%i` |
| 642 | `%c` |
| 643 | `%k` |
| 644 | `%v` |
| 645 | `%m` |
| 646 | `@@u %U @@` |
| 647 | `@@` |
| 657 | `Icon` |
| 660 | `Comment` |

---

## `engine/chat_loop/input_listener.rs`

- **Path**: `src/engine/chat_loop/input_listener.rs`
- **Comments**: 5
- **Strings**: 14

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 12 | `///` | Waits for transcript text input from Web STT / Mobile STT |
| 13 | `///` | while simultaneously responding to GUI / Tray controls non-blockingly with 0% idle CPU. |
| 44 | `///` | Handles loop when assistant is in paused state |
| 65 | `//` | unpause |
| 67 | `//` | exit |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 23 | `toggle_pause` |
| 25 | `stop_chat` |
| 25 | `quit` |
| 27 | `reset_session` |
| 29 | `[Reset Session] AI conversation history has been reset!` |
| 46 | `Paused` |
| 48 | `\n[PAUSED] {} paused listening.` |
| 60 | `toggle_pause` |
| 60 | `start_chat` |
| 62 | `▶  [RESUMED] {} ready to listen again!\n` |
| 66 | `stop_chat` |
| 66 | `quit` |
| 68 | `reset_session` |
| 70 | `[Reset Session] AI conversation history has been reset!` |

---

## `engine/chat_loop/llm_responder.rs`

- **Path**: `src/engine/chat_loop/llm_responder.rs`
- **Comments**: 1
- **Strings**: 14

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 38 | `//` | Update emotion avatar real-time to renderer 3D |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 20 | `LLM Thinking...` |
| 22 | `\n[LLM] {} is responding...` |
| 35 | `Interrupted by user` |
| 41 | `{asst_name} Berpikir` |
| 49 | `\r\n[{}] [{:.2?}]:\r\n   ` |
| 55 | `assistant:stream-chunk` |
| 57 | `sentence` |
| 58 | `chunk` |
| 59 | `speaker` |
| 63 | `{} ` |
| 76 | `[{}] "{}"` |
| 83 | `[LLM Error]: {e}` |
| 89 | `assistant:response-done` |
| 91 | `text` |

---

## `engine/chat_loop/mod.rs`

- **Path**: `src/engine/chat_loop/mod.rs`
- **Comments**: 8
- **Strings**: 35

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 16 | `///` | Loop utama conversation voice and aksi assistant AI Ragasukma |
| 32 | `//` | 1. Check external commands before turn (GUI / Tray) |
| 49 | `//` | 2. If in paused state |
| 65 | `//` | 3. Wait for text input turn from Chrome Web Speech API / Mobile STT |
| 80 | `//` | Check interupsi stop_chat/pause when STT new masuk |
| 93 | `//` | 4. Validate text |
| 117 | `//` | 5. Inferensi LLM and respons voice Supertonic-3 Neural TTS |
| 130 | `//` | 6. Eksekusi Tools / Skills & Agentic Loop |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 19 | `Standby` |
| 20 | `====================================================` |
| 21 | ` RAGASUKMA CHAT SESSION STARTED` |
| 22 | `   - Assistant: {}` |
| 23 | `   - Dashboard: http://127.0.0.1:42128` |
| 24 | `====================================================\n` |
| 34 | `stop_chat` |
| 34 | `quit` |
| 36 | `Standby` |
| 37 | `\n[DONE] Chat session stopped.` |
| 39 | `toggle_pause` |
| 41 | `reset_session` |
| 43 | `[Reset Session] AI conversation history has been reset!` |
| 54 | `Standby` |
| 55 | `\n[DONE] Chat session stopped.` |
| 62 | `Standby` |
| 69 | `Standby` |
| 70 | `\n[DONE] Chat session stopped.` |
| 82 | `stop_chat` |
| 82 | `quit` |
| 84 | `Main Menu` |
| 85 | `\n[DONE] Chat session stopped.` |
| 87 | `toggle_pause` |
| 95 | `[INFO] (Voice unclear or too low)\n` |
| 97 | `Standby` |
| 106 | `\n[User] You [5.00ms]:` |
| 107 | `   "{}"` |
| 110 | `assistant:transcript` |
| 112 | `role` |
| 112 | `user` |
| 113 | `text` |
| 126 | `[Interrupt] Processing new speech turn...` |
| 141 | `----------------------------------------------------\n` |
| 144 | `[ERROR] Network error: {err}\n` |
| 148 | `----------------------------------------------------\n` |

---

## `engine/chat_loop/skill_runner.rs`

- **Path**: `src/engine/chat_loop/skill_runner.rs`
- **Comments**: 7
- **Strings**: 24

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 12 | `///` | Running eksekusi skill tool parsing \`[CMD:...]\` or \`[SKILL:...]\`, |
| 13 | `///` | including 2-step agentic loop for follow-up desktop/browser actions. |
| 23 | `//` | Universal normalization: map any raw tool-call blocks (<dots_function_call>, <invoke>, etc.) |
| 24 | `//` | to canonical [CMD:...] so commands from any LLM model run reliably. |
| 31 | `//` | Automatic guardrail: If this response was triggered by internal SYSTEM STATUS notification, |
| 32 | `//` | prevent repeated media:play calls that re-trigger tracks that just finished downloading. |
| 63 | `//` | Update snapshot when switching window or browser |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 9 | `\[(?:SKILL\|CMD):([^\]]+)\]` |
| 9 | `valid skill regex` |
| 33 | `[SYSTEM STATUS:` |
| 33 | `media:play` |
| 35 | `[Duplicate Skill Guard]: Ignoring '{}' because the song was already played automatically by the system` |
| 44 | `\n[SKILL]: Invoking ID '{}'...` |
| 48 | `[SKILL SUCCESS: {} ({})] -> {}` |
| 54 | `assistant:skill` |
| 56 | `id` |
| 57 | `name` |
| 58 | `status` |
| 58 | `success` |
| 59 | `output` |
| 65 | `desktop:scan_visual` |
| 69 | `desktop:focus` |
| 69 | `app:open` |
| 78 | `[SKILL FAILED]: Identifier '{}' not found in skill-command.md` |
| 82 | `assistant:skill` |
| 84 | `id` |
| 85 | `name` |
| 86 | `status` |
| 86 | `failed` |
| 87 | `output` |
| 87 | `Skill not found` |

---

## `engine/cloudflared.rs`

- **Path**: `src/engine/cloudflared.rs`
- **Comments**: 10
- **Strings**: 103

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 56 | `//` | Check if child exited |
| 298 | `//` | Already running |
| 306 | `//` | Refresh auth key for this tunnel session |
| 311 | `//` | Check cloudflared binary |
| 326 | `//` | CREATE_NO_WINDOW |
| 347 | `//` | Read stderr & stdout in background threads to find trycloudflare.com URL |
| 350 | `//` | Spawn stderr reader |
| 370 | `//` | Spawn stdout reader as well (in case cloudflared outputs URL to stdout) |
| 390 | `//` | Return immediately with "starting" status and auth_key |
| 391 | `//` | The frontend will poll /api/remote/tunnel/status until URL is available |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 33 | `stopped` |
| 46 | `{:x}{:x}` |
| 60 | `stopped` |
| 82 | `windows` |
| 84 | `LOCALAPPDATA` |
| 86 | `ragasukma` |
| 87 | `bin` |
| 88 | `cloudflared.exe` |
| 93 | `USERPROFILE` |
| 95 | `.local` |
| 96 | `bin` |
| 97 | `cloudflared.exe` |
| 104 | `PATH` |
| 107 | `cloudflared.exe` |
| 109 | `cloudflared` |
| 118 | `/usr/local/bin/cloudflared` |
| 119 | `/usr/bin/cloudflared` |
| 120 | `/bin/cloudflared` |
| 121 | `/opt/homebrew/bin/cloudflared` |
| 132 | `cloudflared.exe` |
| 134 | `cloudflared` |
| 141 | `version` |
| 158 | `--version` |
| 182 | `windows` |
| 184 | `LOCALAPPDATA` |
| 185 | `USERPROFILE` |
| 185 | `{}\AppData\Local` |
| 186 | `Environment variable LOCALAPPDATA / USERPROFILE not found` |
| 188 | `ragasukma` |
| 189 | `bin` |
| 191 | `Failed to create folder {}: {}` |
| 192 | `cloudflared.exe` |
| 194 | `x86` |
| 195 | `386` |
| 197 | `amd64` |
| 200 | `https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-windows-{}.exe` |
| 207 | `Failed to create HTTP client: {}` |
| 212 | `Failed to download cloudflared: {}` |
| 216 | `Failed to download cloudflared from {}: status {}` |
| 224 | `Failed to read cloudflared payload: {}` |
| 227 | `Failed to write cloudflared binary: {}` |
| 230 | `cloudflared successfully downloaded to {}` |
| 234 | `macos` |
| 236 | `brew` |
| 237 | `install` |
| 237 | `cloudflared` |
| 239 | `Failed to run brew: {}` |
| 241 | `cloudflared successfully installed via brew` |
| 246 | `linux` |
| 248 | `HOME` |
| 249 | `HOME environment variable not found` |
| 250 | `.local` |
| 250 | `bin` |
| 252 | `Failed to create folder ~/.local/bin: {}` |
| 253 | `cloudflared` |
| 255 | `aarch64` |
| 256 | `arm64` |
| 258 | `amd64` |
| 261 | `https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-{}` |
| 265 | `curl` |
| 266 | `-L` |
| 266 | `-o` |
| 268 | `Failed to run curl: {}` |
| 271 | `Failed to download cloudflared binary via curl` |
| 274 | `chmod` |
| 275 | `+x` |
| 279 | `cloudflared successfully downloaded to {}` |
| 283 | `windows` |
| 283 | `macos` |
| 283 | `linux` |
| 285 | `Operating system not supported for automatic installation` |
| 308 | `starting` |
| 315 | `tunnel` |
| 316 | `--url` |
| 317 | `http://127.0.0.1:{}` |
| 318 | `--no-autoupdate` |
| 332 | `Failed to run cloudflared: {}. Make sure cloudflared is installed and available in PATH.` |
| 340 | `Failed to capture cloudflared stderr` |
| 344 | `Failed to capture cloudflared stdout` |
| 353 | `https://[a-zA-Z0-9-]+\.trycloudflare\.com` |
| 356 | `[Cloudflared] {}` |
| 362 | `running` |
| 363 | `[Cloudflared Tunnel Ready] {}` |
| 373 | `https://[a-zA-Z0-9-]+\.trycloudflare\.com` |
| 376 | `[Cloudflared stdout] {}` |
| 382 | `running` |
| 383 | `[Cloudflared Tunnel Ready] {}` |
| 407 | `taskkill` |
| 408 | `/F` |
| 408 | `/T` |
| 408 | `/PID` |
| 414 | `stopped` |
| 415 | `[Cloudflared Tunnel Stopped]` |
| 431 | `taskkill` |
| 432 | `/F` |
| 432 | `/T` |
| 432 | `/PID` |
| 446 | `cloudflared version 2026.8.3 (Checksum f29324fe934d1e100617484c78deef803c4dc2cd351d645bbde42e96b4fccc5e)` |
| 447 | `2026.8.3` |
| 449 | `cloudflared version 2024.1.5 (built 2024-01-15-1200)` |
| 450 | `2024.1.5` |
| 452 | `cloudflared 2023.10.0` |
| 453 | `2023.10.0` |

---

## `engine/mod.rs`

- **Path**: `src/engine/mod.rs`
- **Comments**: 0
- **Strings**: 0

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

_No string literals found._

---

## `engine/perception/capture.rs`

- **Path**: `src/engine/perception/capture.rs`
- **Comments**: 7
- **Strings**: 24

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 6 | `//` | 1. Coba grim (standar emas Wayland: Sway, Hyprland, wlroots) |
| 15 | `//` | 2. Coba wayshot (Wayland alternatif) |
| 27 | `//` | 3. Coba maim (X11 modern) |
| 36 | `//` | 4. Coba scrot (X11 standar) |
| 48 | `//` | 5. Coba ImageMagick import |
| 63 | `///` | Capture screenshot of the primary display or target window |
| 97 | `///` | Capture specific active window image |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 4 | `linux` |
| 7 | `grim` |
| 7 | `-` |
| 16 | `wayshot` |
| 17 | `--stdout` |
| 28 | `maim` |
| 37 | `scrot` |
| 38 | `-z` |
| 38 | `-o` |
| 38 | `-` |
| 49 | `import` |
| 50 | `-window` |
| 50 | `root` |
| 50 | `png:-` |
| 60 | `Failed to capture screenshot on Linux: xcap failed and all fallback tools (grim/wayshot/maim/scrot/import) unavailable.` |
| 66 | `Failed to list monitors: {e}` |
| 68 | `No monitor found` |
| 78 | `Monitor capture error: {e}` |
| 80 | `Failed to construct ImageBuffer from frame` |
| 85 | `linux` |
| 88 | `[Perception] xcap failed ({}). Using Linux capture fallback...` |
| 99 | `Failed to list windows: {e}` |
| 106 | `Window capture error: {e}` |
| 108 | `Failed to construct ImageBuffer from window frame` |

---

## `engine/perception/mod.rs`

- **Path**: `src/engine/perception/mod.rs`
- **Comments**: 19
- **Strings**: 34

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 20 | `//` | "button", "input", "text" |
| 21 | `//` | [x, y, w, h] |
| 22 | `//` | [cx, cy] |
| 35 | `///` | Lightweight PerceptionEngine |
| 36 | `///` | Prioritizes OS-native OCR (Windows Media OCR / Apple Vision) without loading extra RAM |
| 37 | `///` | with heavy models. Features fast fallback to Tesseract OCR (streaming via stdin/stdout pipe). |
| 47 | `///` | Runs visual OCR scan on image |
| 59 | `//` | 1. Eksekusi OCR with prioritas Native OS -> Fallback Tesseract OCR |
| 63 | `//` | Coba native Windows Media OCR |
| 77 | `//` | Coba native Apple Vision OCR |
| 91 | `//` | Linux: Use Tesseract OCR memory-streaming |
| 183 | `///` | Initialize Vision Worker in thread background terpisah. |
| 184 | `///` | This thread is 100% idle (0% CPU, 0 MB model load) when there are no scan requests. |
| 188 | `//` | Already running |
| 213 | `//` | Get active window info for assistant voice status report |
| 268 | `///` | Helper sync for meminta scan from vision worker |
| 290 | `///` | Asynchronous fire-and-forget helper to trigger vision scan in background |
| 306 | `///` | Synchronizes perception results to \`active_window_app.json\` |
| 358 | `///` | On-demand visual perception scan |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 2 | `macos` |
| 5 | `windows` |
| 61 | `windows` |
| 68 | `[Perception] Windows Native OCR failed ({err}), attempting Tesseract OCR fallback...` |
| 75 | `macos` |
| 82 | `[Perception] Apple Vision OCR failed ({err}), attempting Tesseract OCR fallback...` |
| 89 | `windows` |
| 89 | `macos` |
| 118 | `button` |
| 120 | `text` |
| 135 | `[Perception] Failed to process visual OCR: {err}` |
| 144 | `input` |
| 151 | `{} ... ({} texts detected)` |
| 152 | ` \| ` |
| 156 | ` \| ` |
| 195 | `ragasukma-vision-worker` |
| 205 | `[Vision Worker] Failed to capture screen: {e}` |
| 220 | `desktop screen` |
| 223 | `desktop screen` |
| 228 | `[SYSTEM STATUS: Screen scan for window '{}' complete. Data is ready. Inform the user briefly that the screen has been...` |
| 233 | `[SYSTEM STATUS: Screen scan for window '{}' complete, but no text or buttons were detected. Inform the user briefly a...` |
| 249 | `[Vision Worker] Failed to capture window '{title}': {e}` |
| 265 | `Failed to spawn vision worker thread` |
| 309 | `vision_perception` |
| 365 | `[Perception] Failed to initialize Perception Engine: {e}` |
| 374 | `[Perception] Failed to capture screen: {e}` |
| 403 | `At least one monitor must be detected` |
| 409 | `windows` |
| 412 | `Perception elements count: {}` |
| 413 | `Perception texts count: {}` |
| 415 | `Sample text: {:?}` |
| 419 | `Snapshot source: {}` |
| 420 | `Snapshot buttons: {}` |
| 421 | `Snapshot texts: {}` |

---

## `engine/perception/ocr_macos.rs`

- **Path**: `src/engine/perception/ocr_macos.rs`
- **Comments**: 2
- **Strings**: 11

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 12 | `///` | Native OCR bawaan macOS using Apple Vision Framework (\`VNRecognizeTextRequest\`). |
| 13 | `///` | Leverages native macOS Apple Neural Engine / CoreML (0 MB model files, 0 MB extra RAM). |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 14 | `macos` |
| 18 | `ragasukma_vision_{}_{}.png` |
| 29 | `Failed to save temporary screenshot for macOS Vision: {e}` |
| 31 | ` import Foundation import Vision import Cocoa  guard CommandLine.arguments.count > 1 else {     print("{\"error\":\"N...` |
| 102 | `swift` |
| 103 | `-e` |
| 108 | `Failed to call swift Apple Vision: {e}` |
| 113 | `Apple Vision error: {err}` |
| 120 | `Apple Vision output did not match expected JSON format` |
| 123 | `macos` |
| 125 | `Apple Vision OCR is only supported on macOS.` |

---

## `engine/perception/ocr_tesseract.rs`

- **Path**: `src/engine/perception/ocr_tesseract.rs`
- **Comments**: 13
- **Strings**: 22

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 6 | `///` | Cache of languages supported by tesseract on system (e.g. "eng+ind" or "eng") |
| 49 | `///` | Universal fallback OCR using Tesseract OCR via streaming pipe (stdin -> stdout TSV). |
| 50 | `///` | Bekerja in Linux (X11/Wayland), Windows, and macOS without file sementara in disk. |
| 54 | `//` | 1. Encode image to memory buffer PNG without menulis to disk |
| 62 | `//` | 2. Spawn tesseract process with input stdin and output stdout TSV |
| 79 | `//` | 3. Tulis PNG bytes to child stdin |
| 86 | `//` | 4. Tunggu eksekusi and baca stdout TSV |
| 102 | `///` | Parses TSV output from Tesseract and groups adjacent words into UI phrases |
| 106 | `//` | Format: level, page_num, block_num, par_num, line_num, word_num, left, top, width, height, conf, text |
| 114 | `//` | Level 5 = Word level |
| 125 | `//` | Noise filter: ignore very low confidence (< 30.0%) or empty text |
| 149 | `//` | Group adjacent words on the same line into coherent UI phrases/elements |
| 179 | `//` | Save previous phrase |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 11 | `tesseract` |
| 11 | `--list-langs` |
| 18 | `eng` |
| 20 | `ind` |
| 25 | `eng+ind` |
| 27 | `ind` |
| 29 | `eng` |
| 32 | `eng` |
| 58 | `Failed to encode image to PNG buffer: {e}` |
| 63 | `tesseract` |
| 64 | `stdin` |
| 65 | `stdout` |
| 66 | `-l` |
| 68 | `tsv` |
| 75 | `Executable 'tesseract' not found on system: {e}. Please install tesseract (e.g. 'sudo dnf install tesseract tesseract...` |
| 83 | `Failed to send image buffer to tesseract stdin: {e}` |
| 89 | `Failed to read tesseract output: {e}` |
| 93 | `Tesseract exited with failure status code: {:?}` |
| 115 | `5` |
| 214 | `\ level\tpage_num\tblock_num\tpar_num\tline_num\tword_num\tleft\ttop\twidth\theight\tconf\ttext 1\t1\t0\t0\t0\t0\t0\t...` |
| 222 | `Sign In` |
| 224 | `Cancel` |

---

## `engine/perception/ocr_windows.rs`

- **Path**: `src/engine/perception/ocr_windows.rs`
- **Comments**: 0
- **Strings**: 21

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 3 | `windows` |
| 17 | `PNG encode error: {e}` |
| 20 | `InMemoryRandomAccessStream error: {e}` |
| 22 | `DataWriter error: {e}` |
| 25 | `WriteBytes error: {e}` |
| 28 | `StoreAsync error: {e}` |
| 30 | `StoreAsync get error: {e}` |
| 33 | `DetachStream error: {e}` |
| 34 | `Seek error: {e}` |
| 37 | `BitmapDecoder error: {e}` |
| 39 | `BitmapDecoder get error: {e}` |
| 43 | `GetSoftwareBitmap error: {e}` |
| 45 | `GetSoftwareBitmap get error: {e}` |
| 48 | `OcrEngine TryCreateFromUserProfileLanguages error: {e}` |
| 52 | `RecognizeAsync error: {e}` |
| 54 | `RecognizeAsync get error: {e}` |
| 58 | `Lines error: {e}` |
| 89 | `windows` |
| 93 | `Windows Native OCR is only supported on Windows 10/11 operating systems.` |
| 99 | `windows` |
| 105 | `Windows Native OcrEngine must be initializable` |

---

## `engine/perception/utils.rs`

- **Path**: `src/engine/perception/utils.rs`
- **Comments**: 39
- **Strings**: 105

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 4 | `///` | Gets coordinate scale divisor so visual scan results |
| 5 | `///` | sync with tool klik mouse in masing-masing sistem operasi. |
| 10 | `//` | On macOS Retina, captured images are in Physical Pixels (e.g. 2880x1800), |
| 11 | `//` | while click utilities (cliclick & AppleScript) take inputs in Points (1440x900). |
| 24 | `//` | Standar Retina display fallback |
| 29 | `//` | Windows in Per-Monitor V2 DPI Aware mode operates purely in 1:1 Physical Pixels |
| 43 | `//` | 1. Check Sway if session using Sway compositor: take value scale direct |
| 67 | `//` | 2. Check Hyprland if session using Hyprland compositor: take value scale direct |
| 91 | `//` | 3. Check GNOME Wayland via gsettings |
| 121 | `//` | 4. Check KDE Plasma Wayland via kreadconfig6 / kreadconfig5 |
| 138 | `//` | 5. Check monitor xcap: in XWayland, xcap reports logical desktop rect |
| 159 | `//` | 6. Check environment variables (GDK_SCALE, QT_SCALE_FACTOR) |
| 175 | `//` | Mode X11 (i3, bspwm, XFCE, Openbox, dll): check xrdb for DPI scaling |
| 197 | `//` | Linux X11 or 100% scale |
| 214 | `//` | 1. Calculator & numeric keypad digits (0-9, 00, etc.) |
| 219 | `//` | 2. Calculator math & function symbols |
| 232 | `//` | 3. Very short action or navigation tokens (1-2 characters) |
| 241 | `//` | 4. Universal UI button heuristics (language-agnostic): |
| 242 | `//` | - Buttons are concise labels (typically 1 to 4 words, length <= 25 chars) |
| 243 | `//` | - Cannot contain multiline breaks or paragraph punctuation (e.g. '.', ';', '?', '!') |
| 244 | `//` | - Cannot be a long descriptive sentence |
| 262 | `///` | Calculates similarity score between two texts (0.0 to 1.0) |
| 263 | `///` | using kombinasi token normalization, substring containment, Levenshtein distance, |
| 264 | `///` | and token-level similarity for mendukung frasa with typo / variasi word. |
| 287 | `//` | 1. Direct whole-string Levenshtein similarity |
| 292 | `//` | 2. Direct clean substring containment |
| 305 | `//` | 3. Token-level similarity for frasa sub-query (for example: "tagian listrik" vs "Bayar tagihan listrik PLN") |
| 343 | `//` | If source only 1 word short (<= 2 huruf, for example "in", "to"), |
| 344 | `//` | do not match against long targets containing many words. |
| 363 | `//` | For very short tokens (<= 2 chars), only allow exact match |
| 431 | `//` | Japanese: Submit |
| 432 | `//` | Spanish: Save |
| 443 | `//` | Real-world cases faced by users: |
| 444 | `//` | "skill kode.rd" vs "skillcode.md" |
| 451 | `//` | Music case: "tumhiana" vs "Tum Hi Aana" |
| 458 | `//` | Not similar: "Save" vs "Cancel" |
| 465 | `//` | Sub-query with typo vs long to-do title |
| 472 | `//` | Partial exact sub-query |
| 479 | `//` | Local music: song title with typo vs full audio filename |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 1 | `macos` |
| 1 | `linux` |
| 7 | `macos` |
| 27 | `windows` |
| 34 | `linux` |
| 37 | `WAYLAND_DISPLAY` |
| 38 | `XDG_SESSION_TYPE` |
| 39 | `wayland` |
| 44 | `swaymsg` |
| 45 | `-t` |
| 45 | `get_outputs` |
| 45 | `--raw` |
| 53 | `focused` |
| 56 | `scale` |
| 68 | `hyprctl` |
| 69 | `monitors` |
| 69 | `-j` |
| 77 | `focused` |
| 80 | `scale` |
| 92 | `gsettings` |
| 93 | `get` |
| 93 | `org.gnome.desktop.interface` |
| 93 | `scaling-factor` |
| 107 | `gsettings` |
| 108 | `get` |
| 108 | `org.gnome.desktop.interface` |
| 108 | `text-scaling-factor` |
| 122 | `kreadconfig6` |
| 122 | `kreadconfig5` |
| 124 | `--group` |
| 124 | `KScreen` |
| 124 | `--key` |
| 124 | `ScaleFactor` |
| 160 | `GDK_SCALE` |
| 167 | `QT_SCALE_FACTOR` |
| 176 | `xrdb` |
| 176 | `-query` |
| 180 | `xft.dpi:` |
| 201 | `windows` |
| 201 | `macos` |
| 201 | `linux` |
| 221 | `+` |
| 221 | `-` |
| 221 | `*` |
| 221 | `/` |
| 221 | `x` |
| 221 | `×` |
| 221 | `÷` |
| 221 | `=` |
| 221 | `%` |
| 221 | `±` |
| 221 | `+/-` |
| 221 | `.` |
| 221 | `,` |
| 221 | `c` |
| 221 | `ce` |
| 221 | `del` |
| 221 | `mc` |
| 222 | `mr` |
| 222 | `m+` |
| 222 | `m-` |
| 222 | `ms` |
| 222 | `m` |
| 222 | `sqrt` |
| 222 | `√` |
| 222 | `x²` |
| 222 | `1/x` |
| 222 | `exp` |
| 222 | `mod` |
| 222 | `log` |
| 222 | `ln` |
| 222 | `(` |
| 222 | `)` |
| 223 | `pi` |
| 223 | `π` |
| 223 | `e` |
| 223 | `deg` |
| 223 | `rad` |
| 417 | `linux` |
| 428 | `OK` |
| 429 | `Save` |
| 430 | `Batal` |
| 431 | `送信` |
| 432 | `Guardar` |
| 433 | `7` |
| 434 | `+` |
| 436 | `This is a very long descriptive paragraph that should not be detected as a button.` |
| 438 | `Click here to proceed to the next step.` |
| 445 | `skill kode.rd` |
| 445 | `skillcode.md` |
| 448 | `Similarity 'skill kode.rd' vs 'skillcode.md' should be >= 70%, got: {sim}` |
| 452 | `tumhiana` |
| 452 | `Tum Hi Aana` |
| 455 | `Similarity 'tumhiana' vs 'Tum Hi Aana' should be >= 70%, got: {sim_music}` |
| 459 | `Simpan` |
| 459 | `Batal` |
| 466 | `tagian listrik` |
| 466 | `Bayar tagihan listrik PLN` |
| 469 | `Similarity 'tagian listrik' vs 'Bayar tagihan listrik PLN' should be >= 70%, got: {sim_todo}` |
| 473 | `bayar listrik` |
| 473 | `Bayar tagihan listrik PLN` |
| 476 | `Similarity 'bayar listrik' vs 'Bayar tagihan listrik PLN' should be >= 70%, got: {sim_bayar}` |
| 480 | `sepia` |
| 480 | `01 - Sheila On 7 - Sephia (Remastered).mp3` |
| 483 | `Similarity 'sepia' vs '01 - Sheila On 7 - Sephia (Remastered).mp3' should be >= 70%, got: {sim_music}` |

---

## `engine/remote_text.rs`

- **Path**: `src/engine/remote_text.rs`
- **Comments**: 2
- **Strings**: 1

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 10 | `//` | Flush any pending texts sent while the session was starting |
| 29 | `//` | If chat loop is not running yet, save to pending queue and trigger start_chat automatically |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 35 | `start_chat` |

---

## `engine/session.rs`

- **Path**: `src/engine/session.rs`
- **Comments**: 3
- **Strings**: 12

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 51 | `//` | Check whether audio / language / TTS configuration changed |
| 110 | `//` | Check configuration changes live while in standby |
| 116 | `//` | Standby mode: waiting for commands from Web Dashboard or Start button |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 24 | ` Character {} (Cute & Shy) ready to accompany {}.` |
| 30 | ` TTS Engine: Supertonic-3 Neural ONNX ready.` |
| 73 | `[Reactive Update] Voice interruption updated in realtime!` |
| 78 | `[Reactive Update] Conversation language switched to: {}` |
| 86 | `[Reactive Update] Voice / Gender ({}) updated.` |
| 94 | `[Reactive Update] LLM Provider / Model / Persona switched to: {} / {} ({})` |
| 105 | `Standby` |
| 119 | `start_chat` |
| 123 | `reset_session` |
| 126 | `[Reset Session] AI chat memory history successfully reset!` |
| 129 | `quit` |
| 132 | `\n Program finished. Goodbye!` |

---

## `engine/skills.rs`

- **Path**: `src/engine/skills.rs`
- **Comments**: 6
- **Strings**: 21

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 12 | `//` | Use kandidat standar (Lokal project assets/ -> beside exe -> Resources macOS -> AppData/Roaming) |
| 18 | `//` | Default target if none exists yet |
| 83 | `///` | Utility helper to configure subprocess to be isolated from AppImage runtime |
| 99 | `//` | If perintah is executable direct or file (for example calc.exe, notepad.exe, explorer.exe) |
| 100 | `//` | and not mengandung shell pipe/redirect, run direct without cmd /C |
| 125 | `//` | Using /S /C "<cmd>" so that parser cmd.exe not memotong tanda kutip ganda internal |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 13 | `skill-command.md` |
| 20 | `assets` |
| 21 | `skill-command.md` |
| 35 | `:---` |
| 36 | `Skill Name` |
| 37 | `Nama Skill` |
| 67 | `#  Skill Command Registry\n\n\ List of quick commands and system automations that can be triggered directly via voice...` |
| 72 | `\| {} \| {} \| \`{}\` \|\n` |
| 86 | `linux` |
| 88 | `PYTHONHOME` |
| 89 | `PYTHONPATH` |
| 90 | `LD_LIBRARY_PATH` |
| 91 | `LD_PRELOAD` |
| 96 | `windows` |
| 124 | `cmd` |
| 126 | `/S` |
| 126 | `/C` |
| 126 | `"{trimmed}"` |
| 130 | `windows` |
| 132 | `sh` |
| 133 | `-c` |

---

## `engine/tasks.rs`

- **Path**: `src/engine/tasks.rs`
- **Comments**: 10
- **Strings**: 31

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 24 | `///` | Initialize dedicated background worker thread for yt-dlp & other heavy asynchronous tasks |
| 31 | `//` | Already initialized |
| 37 | `//` | Dedicated worker thread with 16 MB stack allocation |
| 73 | `//` | 1. Play directly via desktop speakers (Native Audio Player) |
| 77 | `//` | 2. Sync to Web Dashboard |
| 89 | `//` | 3. Send a signal to the assistant so that it speaks to inform the user that the song has finished downloading & playing |
| 108 | `//` | Send a signal to assistant so that notifies the user that the download failed |
| 131 | `///` | Send download & play music tasks to Background Worker |
| 146 | `///` | Find out whether a background worker is currently busy processing media downloads |
| 157 | `///` | Gets current active task info (for AI context or telemetry) |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 39 | `ragasukma-task-worker` |
| 44 | `[Task Worker] Dedicated Background Task Engine ready.` |
| 51 | `ytdlp_download` |
| 52 | `Downloading song: {query}` |
| 55 | ` [Task Worker: yt-dlp] Downloading in background: '{}'...` |
| 57 | `task:started` |
| 59 | `type` |
| 59 | `ytdlp_download` |
| 60 | `query` |
| 61 | `status` |
| 61 | `downloading` |
| 69 | ` [Task Worker: yt-dlp] Done: '{}' -> Playing directly to speaker!` |
| 78 | `music:play` |
| 81 | `task:completed` |
| 83 | `type` |
| 83 | `ytdlp_download` |
| 84 | `title` |
| 85 | `status` |
| 85 | `success` |
| 91 | `[SYSTEM STATUS: Song '{}' has been downloaded and is now playing automatically. Inform the user briefly and warmly. D...` |
| 97 | ` [Task Worker: yt-dlp] Failed to download '{}': {e}` |
| 99 | `task:failed` |
| 101 | `type` |
| 101 | `ytdlp_download` |
| 102 | `query` |
| 103 | `error` |
| 104 | `status` |
| 104 | `failed` |
| 110 | `[SYSTEM STATUS: Failed to download song '{}'. Detail: {}. Inform the user briefly and warmly. DO NOT include any comm...` |
| 122 | `Failed to spawn background task worker thread` |
| 150 | `ytdlp_download` |

---

## `engine/todo.rs`

- **Path**: `src/engine/todo.rs`
- **Comments**: 1
- **Strings**: 40

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 64 | `//` | Fallback: Fuzzy Similarity Matching for to-do items |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 4 | `todo.md` |
| 8 | `todo.md` |
| 14 | `- [ ] {}` |
| 19 | `## ` |
| 20 | `in progress` |
| 20 | `to-do` |
| 20 | `todo` |
| 31 | `##  In Progress / To-Do` |
| 35 | `\n` |
| 54 | `- [ ]` |
| 55 | `- [ ]` |
| 69 | `- [ ]` |
| 70 | `- [ ]` |
| 84 | `- [x] {}` |
| 88 | `## ` |
| 88 | `done` |
| 98 | `##  Done` |
| 102 | `\n` |
| 113 | `Task text must not be empty` |
| 121 | `#  Tasks & Notes\n\n##  In Progress / To-Do\n\n##  Done\n` |
| 126 | `Task '{}' successfully saved to todo.md` |
| 132 | `Task keyword must not be empty` |
| 137 | `To-do list has not been created yet` |
| 145 | `Task '{}' successfully marked as done in todo.md` |
| 150 | `Task with keyword '{}' not found in the active list` |
| 162 | `#  Tasks\n\n##  In Progress / To-Do\n- [ ] Buy milk\n\n##  Done\n` |
| 163 | `Undangan nikahan Yusuf` |
| 164 | `- [ ] Undangan nikahan Yusuf` |
| 165 | `- [ ] Buy milk` |
| 168 | `nikahan` |
| 168 | `should find task` |
| 169 | `Undangan nikahan Yusuf` |
| 170 | `- [ ] Undangan nikahan Yusuf` |
| 171 | `- [x] Undangan nikahan Yusuf` |
| 172 | `- [ ] Buy milk` |
| 178 | `#  Tasks\n\n##  In Progress / To-Do\n- [ ] Pay electricity bill\n\n##  Done\n` |
| 179 | `electricity bill` |
| 180 | `should find task with fuzzy` |
| 181 | `Pay electricity bill` |
| 182 | `- [x] Pay electricity bill` |

---

## `engine/window_scanner/linux.rs`

- **Path**: `src/engine/window_scanner/linux.rs`
- **Comments**: 1
- **Strings**: 43

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 164 | `//` | Fallback: X11 xdotool |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 1 | `linux` |
| 4 | `linux` |
| 8 | `linux` |
| 25 | `[Desktop Scanner] AT-SPI D-Bus init error: {_e}` |
| 31 | `linux` |
| 40 | `linux` |
| 42 | `SWAYSOCK` |
| 46 | `swaymsg` |
| 47 | `-t` |
| 47 | `get_tree` |
| 58 | `name` |
| 60 | `app_id` |
| 63 | `window_properties` |
| 64 | `class` |
| 69 | `waybar` |
| 70 | `id` |
| 72 | `focused` |
| 85 | `nodes` |
| 93 | `floating_nodes` |
| 106 | `linux` |
| 108 | `SWAYSOCK` |
| 109 | `swaymsg` |
| 110 | `-t` |
| 110 | `get_tree` |
| 117 | `focused` |
| 122 | `name` |
| 127 | `app_id` |
| 130 | `window_properties` |
| 131 | `class` |
| 138 | `nodes` |
| 146 | `floating_nodes` |
| 165 | `xdotool` |
| 166 | `getwindowfocus` |
| 166 | `getwindowname` |
| 180 | `linux` |
| 206 | `sway:{}` |
| 213 | `sway:{}` |
| 261 | `waybar` |
| 262 | `desktop` |
| 341 | `{:?}` |
| 350 | `input` |
| 361 | `text` |
| 387 | `No application window is focused` |

---

## `engine/window_scanner/macos.rs`

- **Path**: `src/engine/window_scanner/macos.rs`
- **Comments**: 0
- **Strings**: 17

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 1 | `macos` |
| 4 | `macos` |
| 6 | `     tell application "System Events"         set frontProc to first process whose frontmost is true         set proc...` |
| 18 | `osascript` |
| 19 | `-e` |
| 24 | `\|\|\|` |
| 34 | `macos` |
| 36 | `     set jsonOut to "{\"app\":\"\",\"title\":\"\",\"windows\":[],\"buttons\":[]}"     try         tell application "S...` |
| 96 | `osascript` |
| 97 | `-e` |
| 99 | `Failed to call osascript: {e}` |
| 102 | `\|\|\|` |
| 113 | `macos:{}` |
| 119 | `app` |
| 124 | `title` |
| 128 | `active` |
| 135 | `macos:{}` |

---

## `engine/window_scanner/mod.rs`

- **Path**: `src/engine/window_scanner/mod.rs`
- **Comments**: 5
- **Strings**: 45

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 73 | `//` | Vision Protection: If previous snapshot is from vision_perception on jendela/aplikasi which sama, |
| 74 | `//` | preserve vision data (buttons with center/bbox coords, inputs, and OCR text) so it is not overwritten |
| 75 | `//` | oleh scan dangkal OS / accessibility tree! |
| 240 | `//` | Only scan if jendela/aplikasi active correct-correct berpindah. |
| 241 | `//` | No more force_periodic 30s overwriting visual vision_perception results. |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 1 | `linux` |
| 3 | `macos` |
| 5 | `windows` |
| 18 | `Option::is_none` |
| 20 | `Option::is_none` |
| 34 | `desktop_window` |
| 42 | `default_source` |
| 76 | `vision_perception` |
| 83 | `vision_perception` |
| 94 | `vision_perception` |
| 105 | `Failed to serialize snapshot: {e}` |
| 107 | `Failed to write to {path:?}: {e}` |
| 109 | `assets/active_window_app.json` |
| 123 | `File active_window_app.json does not exist yet` |
| 127 | `Failed to read {:?}: {e}` |
| 129 | `Failed to parse JSON: {e}` |
| 134 | `assets/active_window_app.json` |
| 135 | `Cargo.toml` |
| 143 | `assets` |
| 147 | `active_window_app.json` |
| 158 | `windows` |
| 163 | `linux` |
| 168 | `macos` |
| 173 | `windows` |
| 173 | `linux` |
| 173 | `macos` |
| 175 | `Unsupported operating system` |
| 180 | `windows` |
| 185 | `linux` |
| 190 | `macos` |
| 195 | `windows` |
| 195 | `linux` |
| 195 | `macos` |
| 197 | `Unsupported operating system` |
| 202 | `windows` |
| 207 | `linux` |
| 212 | `macos` |
| 217 | `windows` |
| 217 | `linux` |
| 217 | `macos` |
| 225 | `ragasukma-window-watcher` |
| 243 | `linux` |
| 250 | `windows` |
| 255 | `macos` |
| 266 | `Failed to spawn window watcher thread` |

---

## `engine/window_scanner/windows.rs`

- **Path**: `src/engine/window_scanner/windows.rs`
- **Comments**: 4
- **Strings**: 13

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 90 | `//` | 1. Dapatkan Foreground Window Info |
| 127 | `//` | 2. Enum all jendela which terlihat in Windows |
| 205 | `//` | 3. UI Automation for mengekstrak Buttons & Inputs from Foreground Window |
| 305 | `//` | Check if previous snapshot originated from perception/OCR and is for the same window |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 3 | `windows` |
| 59 | `windows` |
| 97 | `win:0x{:x}` |
| 134 | `system` |
| 182 | `win:0x{:x}` |
| 259 | `button` |
| 269 | `input` |
| 281 | `text` |
| 307 | `vision_perception` |
| 323 | `{} ... ({} texts detected)` |
| 324 | ` \| ` |
| 328 | ` \| ` |
| 333 | `No application window is focused` |

---

## `engine/ytdlp.rs`

- **Path**: `src/engine/ytdlp.rs`
- **Comments**: 33
- **Strings**: 138

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 18 | `///` | Name file relatif in folder music (e.g. "Song Title [abc123].opus") |
| 22 | `///` | Install yt-dlp automatically sesuai OS |
| 69 | `//` | fallback pip3 |
| 92 | `///` | Search path executable yt-dlp in Windows (termasuk WinGet Links & Packages, Scoop, Python Script) |
| 94 | `//` | 1. Check direct if can dipanggil in PATH |
| 103 | `//` | 2. Check WinGet standard directory: %LOCALAPPDATA%\Microsoft\WinGet\Links\yt-dlp.exe |
| 114 | `//` | WinGet Packages folder scan |
| 133 | `//` | Programs folder |
| 143 | `//` | 3. Check %USERPROFILE%\scoop\shims\yt-dlp.exe or %USERPROFILE%\.cargo\bin\yt-dlp.exe |
| 153 | `//` | Python Scripts |
| 168 | `//` | Homebrew on Apple Silicon (/opt/homebrew/bin) and Intel (/usr/local/bin) |
| 212 | `///` | Resolve command yt-dlp |
| 229 | `//` | Clean environment variables hijacked by Linux AppImage wrapper |
| 230 | `//` | so that yt-dlp / Python using dynamic library asli milik OS host |
| 238 | `///` | Check whether yt-dlp is installed and return its version |
| 256 | `///` | Download audio pakai yt-dlp to out_dir, return metadata track |
| 266 | `//` | If query is not a URL, wrap as ytsearch |
| 278 | `//` | Record existing audio files before download |
| 289 | `//` | Use -f "ba[ext=m4a]/ba" (best audio M4A/AAC or best audio) |
| 290 | `//` | Use --print "after_move:..." so that yt-dlp CORRECT-CORRECT mendownload file to disk |
| 291 | `//` | then print metadata after file is completely written |
| 309 | `//` | Limit timeout: 25 detik |
| 317 | `//` | Process finished before timeout |
| 333 | `//` | Clean leftover JSON files if any |
| 343 | `//` | 1. Coba parse metadata persis from stdout yt-dlp (after_move) |
| 385 | `//` | 2. Detect newly added files in folder (before vs after comparison) |
| 417 | `//` | 2. Fallback: Search for the most recently modified audio file (latest mtime) in folder |
| 436 | `//` | Sort by latest modification time |
| 472 | `///` | Clean *.json files in music directory so it cleanly contains only pure audio files |
| 486 | `///` | Scan folder music, parse judul & ID direct from name file, return list track |
| 539 | `///` | Parse "(title) [(id)].ext" from filename if no sidecar exists |
| 541 | `//` | Strip any extension (.webm, .opus, .mp3, etc.) |
| 547 | `//` | Coba ekstrak [id] from end |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 24 | `windows` |
| 26 | `winget` |
| 28 | `install` |
| 29 | `--id` |
| 30 | `yt-dlp.yt-dlp` |
| 31 | `-e` |
| 32 | `--accept-source-agreements` |
| 33 | `--accept-package-agreements` |
| 36 | `Failed to run winget: {}` |
| 38 | `yt-dlp successfully installed via winget` |
| 43 | `macos` |
| 45 | `brew` |
| 46 | `install` |
| 46 | `yt-dlp` |
| 48 | `Failed to run brew: {}` |
| 50 | `yt-dlp successfully installed via brew` |
| 55 | `linux` |
| 57 | `pip` |
| 58 | `install` |
| 58 | `-U` |
| 58 | `yt-dlp` |
| 59 | `PYTHONHOME` |
| 60 | `PYTHONPATH` |
| 61 | `LD_LIBRARY_PATH` |
| 62 | `LD_PRELOAD` |
| 65 | `Failed to run pip: {}` |
| 67 | `yt-dlp successfully installed via pip` |
| 70 | `pip3` |
| 71 | `install` |
| 71 | `-U` |
| 71 | `yt-dlp` |
| 72 | `PYTHONHOME` |
| 73 | `PYTHONPATH` |
| 74 | `LD_LIBRARY_PATH` |
| 75 | `LD_PRELOAD` |
| 78 | `Failed to run pip3: {}` |
| 80 | `yt-dlp successfully installed via pip3` |
| 86 | `windows` |
| 86 | `macos` |
| 86 | `linux` |
| 88 | `OS not supported for auto-install` |
| 95 | `yt-dlp` |
| 95 | `--version` |
| 97 | `yt-dlp` |
| 101 | `windows` |
| 104 | `LOCALAPPDATA` |
| 106 | `Microsoft` |
| 107 | `WinGet` |
| 108 | `Links` |
| 109 | `yt-dlp.exe` |
| 116 | `Microsoft` |
| 117 | `WinGet` |
| 118 | `Packages` |
| 124 | `yt-dlp.exe` |
| 135 | `Programs` |
| 136 | `yt-dlp` |
| 137 | `yt-dlp.exe` |
| 144 | `USERPROFILE` |
| 146 | `scoop` |
| 147 | `shims` |
| 148 | `yt-dlp.exe` |
| 155 | `AppData` |
| 156 | `Roaming` |
| 157 | `Python` |
| 158 | `Scripts` |
| 159 | `yt-dlp.exe` |
| 166 | `macos` |
| 170 | `/opt/homebrew/bin/yt-dlp` |
| 171 | `/usr/local/bin/yt-dlp` |
| 172 | `/usr/bin/yt-dlp` |
| 180 | `HOME` |
| 181 | `.local/bin/yt-dlp` |
| 188 | `linux` |
| 191 | `/usr/bin/yt-dlp` |
| 192 | `/usr/local/bin/yt-dlp` |
| 193 | `/snap/bin/yt-dlp` |
| 201 | `HOME` |
| 202 | `.local/bin/yt-dlp` |
| 217 | `windows` |
| 219 | `cmd` |
| 220 | `/C` |
| 220 | `yt-dlp` |
| 223 | `windows` |
| 225 | `yt-dlp` |
| 231 | `PYTHONHOME` |
| 232 | `PYTHONPATH` |
| 233 | `LD_LIBRARY_PATH` |
| 234 | `LD_PRELOAD` |
| 240 | `--version` |
| 267 | `http://` |
| 267 | `https://` |
| 270 | `ytsearch1:{}` |
| 274 | `[yt-dlp] Starting audio download: {} to {:?}` |
| 295 | `--no-playlist` |
| 296 | `-f` |
| 297 | `ba[ext=m4a]/ba` |
| 298 | `-o` |
| 299 | `%(title)s [%(id)s].%(ext)s` |
| 300 | `--print` |
| 301 | `after_move:%(id)s\t%(title)s\t%(uploader)s\t%(filename)s` |
| 302 | `--no-warnings` |
| 325 | `Download timed out (25 second limit). Slow internet connection.` |
| 338 | `yt-dlp download error: {}` |
| 360 | `https://i.ytimg.com/vi/{}/hqdefault.jpg` |
| 366 | `[yt-dlp] Audio file successfully saved & loaded: {} ({})` |
| 373 | `YouTube Audio` |
| 396 | `m4a` |
| 396 | `opus` |
| 396 | `mp3` |
| 396 | `ogg` |
| 396 | `wav` |
| 396 | `webm` |
| 396 | `flac` |
| 399 | `https://i.ytimg.com/vi/{}/hqdefault.jpg` |
| 403 | `[yt-dlp] New file detected: {} ({})` |
| 407 | `YouTube Audio` |
| 424 | `m4a` |
| 424 | `opus` |
| 424 | `mp3` |
| 424 | `ogg` |
| 424 | `wav` |
| 424 | `webm` |
| 424 | `flac` |
| 447 | `https://i.ytimg.com/vi/{}/hqdefault.jpg` |
| 452 | `[yt-dlp] Fallback to latest audio file: {} ({})` |
| 458 | `YouTube Audio` |
| 467 | `Download finished but audio file not found at {:?}` |
| 478 | `json` |
| 503 | `mp3` |
| 503 | `opus` |
| 503 | `ogg` |
| 503 | `wav` |
| 503 | `m4a` |
| 503 | `flac` |
| 503 | `webm` |
| 514 | `https://i.ytimg.com/vi/{}/hqdefault.jpg` |
| 519 | ` - ` |
| 522 | `YouTube Audio` |

---
