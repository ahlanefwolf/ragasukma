# Language Audit: `root` Module

- **Directory**: `src/` (root files)
- **Total Files**: 5
- **Total Comments**: 66
- **Total String Literals**: 361

## Table of Contents
- [app_state.rs](#app-staters) (12 comments, 5 strings)
- [assets.rs](#assetsrs) (4 comments, 60 strings)
- [config.rs](#configrs) (8 comments, 38 strings)
- [main.rs](#mainrs) (19 comments, 46 strings)
- [server.rs](#serverrs) (23 comments, 212 strings)

---

## `app_state.rs`

- **Path**: `src/app_state.rs`
- **Comments**: 12
- **Strings**: 5

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 44 | `///` | Live in-memory pose preview without overwriting poses.json on disk |
| 178 | `//` | ----------------------------------------------------------------------------- |
| 179 | `//` | Anti-Echolalia Speaker Echo Guard (Acoustic Loopback Prevention) |
| 180 | `//` | ----------------------------------------------------------------------------- |
| 197 | `//` | Speaker voice reflection protection for up to 12 seconds after it is spoken |
| 211 | `//` | 1. Check exact match or substring |
| 216 | `//` | 2. Check the trigram (3 consecutive words) |
| 226 | `//` | 3. Check significant word overlap (> 50% words exist in assistant sentence) |
| 320 | `//` | --------------------------------------------------------------------------- |
| 321 | `//` | Client Action Trigger Queue (Music actions, UI commands, dll) |
| 322 | `//` | Thread-safe ring buffer with sequence ID per client for multi-client SSE pub-sub |
| 323 | `//` | --------------------------------------------------------------------------- |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 11 | `Option::is_none` |
| 17 | `Option::is_none` |
| 138 | `neutral` |
| 219 | `{} {} {}` |
| 329 | `Option::is_none` |

---

## `assets.rs`

- **Path**: `src/assets.rs`
- **Comments**: 4
- **Strings**: 60

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 134 | `///` | Ensures all default asset files (char.vrm, poses.json, avatar PNG, skill-command.md, config-ai.json, todo.md, etc.) |
| 135 | `///` | automatically extracted to the user config directory (~/.config/ragasukma/assets) when first installed/run. |
| 140 | `//` | Dynamically extract all asset files embedded in the binary |
| 245 | `//` | If the URL is internal chrome://, open it directly in the Chrome/Brave binary as a new tab |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 10 | `assets/` |
| 16 | `assets` |
| 20 | `../Resources/assets` |
| 27 | `assets` |
| 28 | `Cargo.toml` |
| 32 | `../assets` |
| 36 | `assets` |
| 47 | `assets` |
| 49 | `../Resources/assets` |
| 50 | `../Resources` |
| 55 | `assets/{}` |
| 56 | `../assets/{}` |
| 57 | `../{}` |
| 61 | `assets` |
| 72 | `windows` |
| 73 | `APPDATA` |
| 75 | `ragasukma` |
| 75 | `assets` |
| 79 | `ragasukma` |
| 91 | `extension` |
| 95 | `extension` |
| 96 | `../Resources/extension` |
| 99 | `extension` |
| 106 | `extension` |
| 118 | `poses.json` |
| 128 | `poses.json` |
| 137 | `assets` |
| 152 | ` Failed to extract default asset '{}': {:?}` |
| 157 | `[Assets] Successfully extracted default asset: {:?}` |
| 165 | `linux` |
| 167 | `HOME` |
| 167 | `.` |
| 168 | `{}/.local/share/applications` |
| 170 | `ragasukma.desktop` |
| 172 | `ragasukma` |
| 174 | `avatar.png` |
| 177 | `[Desktop Entry]\n\ Version=1.0\n\ Name=Ragasukma\n\ Comment=AI Voice Assistant & 3D/2D Avatar Assistant\n\ Exec="{}"\...` |
| 196 | `[Desktop Integration] Application shortcut created at ~/.local/share/applications/ragasukma.desktop` |
| 207 | `windows` |
| 210 | `chrome.exe` |
| 210 | `msedge.exe` |
| 210 | `brave.exe` |
| 221 | `cmd` |
| 222 | `/C` |
| 222 | `start` |
| 227 | `macos` |
| 229 | `open` |
| 232 | `linux` |
| 235 | `google-chrome-stable` |
| 236 | `google-chrome` |
| 237 | `brave-browser` |
| 238 | `chromium-browser` |
| 239 | `chromium` |
| 240 | `microsoft-edge-stable` |
| 249 | `LD_LIBRARY_PATH` |
| 250 | `LD_PRELOAD` |
| 253 | `[Browser Tab] Opening new tab via {}` |
| 260 | `xdg-open` |
| 262 | `LD_LIBRARY_PATH` |
| 263 | `LD_PRELOAD` |

---

## `config.rs`

- **Path**: `src/config.rs`
- **Comments**: 8
- **Strings**: 38

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 167 | `//` | 1. If there is a local folder assets/music (development mode / portable), use it |
| 178 | `//` | 2. If it doesn't exist, use the AppData / .config user folder so that it is permanent and not lost during the update |
| 206 | `//` | 1. Check via standard candidate (Local project assets/ -> beside exe -> Resources macOS -> AppData/Roaming) |
| 212 | `//` | 2. Default target if it doesn't already exist: always in the user's assets folder |
| 227 | `//` | Try reading up to 3 times with short pauses to overcome temporary file lock in Windows |
| 249 | `//` | Default configuration ONLY if the config-ai.json file has never existed on disk |
| 267 | `//` | Atomic write uses temporary files + retry rename to be safe in the Windows filesystem |
| 281 | `//` | Fallback: direct write if rename is blocked by file lock in Windows |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 27 | `{:02}:{:02}:{:02}` |
| 29 | `00:00:00` |
| 77 | `{}` |
| 78 | `info` |
| 86 | `{}` |
| 87 | `error` |
| 100 | `id` |
| 104 | `F4` |
| 125 | `default_lang` |
| 127 | `default_voice` |
| 137 | `default_vad_threshold` |
| 139 | `default_true` |
| 154 | `id` |
| 155 | `F4` |
| 157 | `2d` |
| 168 | `assets` |
| 168 | `music` |
| 173 | `../assets` |
| 173 | `music` |
| 179 | `assets` |
| 179 | `music` |
| 185 | `windows` |
| 187 | `APPDATA` |
| 188 | `ragasukma` |
| 194 | `HOME` |
| 194 | `USERPROFILE` |
| 195 | `.config` |
| 195 | `ragasukma` |
| 199 | `.config` |
| 199 | `ragasukma` |
| 207 | `config-ai.json` |
| 213 | `assets` |
| 213 | `config-ai.json` |
| 232 | `id` |
| 234 | `Google` |
| 236 | `F4` |
| 244 | `[Config Load] Failed to read config-ai.json (locked/unreadable), using in-memory fallback.` |
| 268 | `tmp` |

---

## `main.rs`

- **Path**: `src/main.rs`
- **Comments**: 19
- **Strings**: 46

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 1 | `//` | Prevents additional console window on Windows in release |
| 27 | `//` | Configure process to Per-Monitor V2 DPI Aware so coordinates |
| 28 | `//` | mouse (SetCursorPos), UI Automation (CurrentBoundingRectangle), |
| 29 | `//` | and monitor screenshots remain 1:1 synchronized without display scaling offset (125%, 150%, etc). |
| 98 | `//` | Single Instance Check: Verify port and application identity handshake |
| 103 | `//` | Check whether port 42128 is indeed the Ragasukma instance (Handshake verification) |
| 140 | `//` | Retain socket listener without drop to prevent TOCTOU (Time-of-Check to Time-of-Use) race conditions |
| 143 | `//` | Automatically copy all default assets to user config folder (~/.config/ragasukma/assets) |
| 146 | `//` | Clean reset browser session snapshot folder from previous sessions |
| 149 | `//` | Initialize dedicated BrowserSessionWorker for non-blocking active_tab_browser.json synchronization |
| 152 | `//` | Initialize reactive native OS active window watcher for active_window_app.json |
| 155 | `//` | Initialize standby dedicated vision worker thread (0% CPU idle) |
| 158 | `//` | Initialize dedicated background task worker thread |
| 161 | `//` | Initialize Supertonic-3 Neural TTS engine in background thread |
| 174 | `//` | Expand session thread stack size (16 MB) for safety on AppImage/portable Linux |
| 175 | `//` | Equipped with auto-recovery loop so internal panics (e.g. audio driver) do not kill the session |
| 204 | `//` | Run 3D VRM Native Overlay on OS Main Thread (Thread 0) |
| 205 | `//` | Following Cocoa/macOS, Windows, and Linux X11/Wayland industry standards |
| 208 | `//` | Send quit command to session thread for graceful shutdown |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 2 | `windows` |
| 22 | `windows` |
| 37 | `--scan-vision` |
| 39 | ` Running local perception scan via dedicated Vision Worker (PP-OCR + OmniParser Icon Detection)...` |
| 42 | ` Vision Scan Successful!` |
| 43 | `  Total Elements : {} items` |
| 44 | `  Buttons / Icons: {} items` |
| 47 | `    - [{}] "{}" at center {:?}` |
| 51 | `  Input Fields   : {} items` |
| 54 | `    - [{}] "{}" at center {:?}` |
| 58 | `  Detected Texts : {} items` |
| 60 | `    - "{}"` |
| 62 | ` File saved to assets/active_window_app.json (source: vision_perception)` |
| 65 | `--scan-window` |
| 66 | ` Running native OS accessibility scan...` |
| 69 | ` Scan Successful!` |
| 70 | `  App Name     : {}` |
| 71 | `  Window Title : {}` |
| 72 | `  Window ID    : {}` |
| 73 | `  Buttons        : {} items` |
| 75 | `    - [Button #{}] {}` |
| 77 | `  Input Forms    : {} items` |
| 79 | `    - [Input #{}] {}` |
| 81 | `  Text Labels    : {} items` |
| 83 | `    - {}` |
| 86 | ` File saved to {}` |
| 88 | ` File saved to assets/active_window_app.json` |
| 92 | ` Error during scan: {e}` |
| 100 | `127.0.0.1` |
| 110 | `http://127.0.0.1:{}/api/ping` |
| 115 | `"ragasukma"` |
| 119 | `[Single Instance] Ragasukma is already running in background!` |
| 121 | ` Opening active Dashboard at http://127.0.0.1:{} ...` |
| 124 | `http://127.0.0.1:{}` |
| 128 | `[Port Conflict] Port {} is in use by another application!` |
| 132 | ` Please close the process using port {}.` |
| 135 | `Port conflict: port {} already bound by another application` |
| 167 | `====================================================` |
| 169 | `  Ragasukma - Personal Assistant: {} (Anime ID)` |
| 172 | `====================================================` |
| 177 | `ragasukma-session` |
| 185 | `[Session Error]: {:?}` |
| 189 | `[AppSession Init Error]: {:?}` |
| 195 | `[Session Auto-Recover] Session thread recovered from unexpected error, restarting session loop...` |
| 202 | `Failed to create session thread` |
| 210 | `quit` |

---

## `server.rs`

- **Path**: `src/server.rs`
- **Comments**: 23
- **Strings**: 212

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 61 | `//` | Check Range header: bytes=start-end (Required for HTML5 <audio> m4a/mp4 container seeking) |
| 112 | `//` | Seek to start position |
| 115 | `//` | Streams are in 64KB chunks |
| 146 | `//` | Ping & Handshake |
| 155 | `//` | OpenAI-Compatible /v1 API Endpoints |
| 176 | `//` | Assets Service |
| 183 | `//` | Browser Sessions & DOM Snapshot Service |
| 201 | `//` | Config Service |
| 205 | `//` | Poses & Tuner Service |
| 211 | `//` | Skills & Applications Service |
| 219 | `//` | Cloudflared Remote Tunnel & Mobile Voice |
| 243 | `//` | Session & Single-Tab Control |
| 244 | `//` | Session Claim Service (1x when tab is opened) |
| 248 | `//` | Logs Service |
| 252 | `//` | LLM Providers - Antigravity |
| 282 | `//` | Menu Service |
| 287 | `//` | Events Service (SSE Stream & Actions Polling) |
| 293 | `//` | Music Storage & Downloader Service (Local Static Audio) |
| 304 | `//` | Assets & Avatar Image Static Serving (Dedicated API prefix to avoid collision with web/dist/assets) |
| 357 | `//` | Static Web Dashboard Files |
| 477 | `//` | Prevent endless hanging sockets when browser refreshes rapidly |
| 501 | `//` | If this is an SSE endpoint or OpenAI API /v1, remove read/write timeouts so that long-lived streams don't get interru... |
| 507 | `//` | CORS preflight OPTIONS |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 12 | `web/dist` |
| 17 | `OK` |
| 18 | `Bad Request` |
| 19 | `Not Found` |
| 20 | `Internal Server Error` |
| 21 | `OK` |
| 24 | `HTTP/1.1 {} {}\r\n\          Content-Type: application/json\r\n\          Content-Length: {}\r\n\          Connection...` |
| 44 | `{"error":"File not found"}` |
| 54 | `HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: 0\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nAc...` |
| 64 | `range:` |
| 72 | `bytes=` |
| 100 | `HTTP/1.1 206 Partial Content\r\nContent-Type: {}\r\nContent-Range: bytes {}-{}/{}\r\nContent-Length: {}\r\nConnection...` |
| 105 | `HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nA...` |
| 147 | `GET` |
| 147 | `/api/ping` |
| 151 | `{"app":"ragasukma","status":"running"}` |
| 156 | `OPTIONS` |
| 156 | `/v1` |
| 156 | `/api/v1` |
| 159 | `GET` |
| 159 | `/v1` |
| 159 | `GET` |
| 159 | `/v1/` |
| 159 | `GET` |
| 159 | `/api/v1` |
| 159 | `GET` |
| 159 | `/api/v1/` |
| 163 | `{"status":"ok","service":"ragasukma-openai-v1","version":"0.2.0"}` |
| 166 | `GET` |
| 166 | `/v1/models` |
| 166 | `GET` |
| 166 | `/api/v1/models` |
| 169 | `POST` |
| 169 | `/v1/chat/completions` |
| 169 | `POST` |
| 169 | `/api/v1/chat/completions` |
| 172 | `POST` |
| 172 | `/v1/completions` |
| 172 | `POST` |
| 172 | `/api/v1/completions` |
| 177 | `GET` |
| 177 | `/api/assets/info` |
| 178 | `POST` |
| 178 | `/api/upload/vrm` |
| 179 | `POST` |
| 179 | `/api/upload/avatar` |
| 184 | `POST` |
| 184 | `/api/browser/snapshot` |
| 188 | `POST` |
| 188 | `/api/browser/tabs` |
| 192 | `GET` |
| 192 | `/api/browser/tabs` |
| 196 | `GET` |
| 196 | `/api/browser/command/next` |
| 202 | `GET` |
| 202 | `/api/config` |
| 203 | `POST` |
| 203 | `/api/config` |
| 206 | `GET` |
| 206 | `/api/poses` |
| 207 | `POST` |
| 207 | `/api/poses` |
| 208 | `POST` |
| 208 | `/api/tuner/preview` |
| 209 | `POST` |
| 209 | `/api/tuner/live` |
| 212 | `GET` |
| 212 | `/api/system/apps` |
| 213 | `GET` |
| 213 | `/api/system/extension-path` |
| 214 | `GET` |
| 214 | `/api/skills` |
| 215 | `POST` |
| 215 | `/api/skills` |
| 216 | `POST` |
| 216 | `/api/skills/delete` |
| 217 | `POST` |
| 217 | `/api/skills/run` |
| 220 | `GET` |
| 220 | `/api/remote/tunnel/status` |
| 221 | `GET` |
| 221 | `/api/remote/tunnel/check` |
| 222 | `POST` |
| 222 | `/api/remote/tunnel/install` |
| 223 | `POST` |
| 223 | `/api/remote/tunnel/start` |
| 224 | `POST` |
| 224 | `/api/remote/tunnel/stop` |
| 225 | `POST` |
| 225 | `/api/remote/active` |
| 230 | `{"success":true,"message":"Remote heartbeat updated"}` |
| 233 | `POST` |
| 233 | `/api/stt/transcript` |
| 234 | `POST` |
| 234 | `/api/tts/state` |
| 235 | `POST` |
| 235 | `/api/tts/synthesize` |
| 236 | `GET` |
| 236 | `/api/tts/audio` |
| 239 | `POST` |
| 239 | `/api/remote/text` |
| 245 | `POST` |
| 245 | `/api/session/claim` |
| 246 | `POST` |
| 246 | `/api/session/tab` |
| 249 | `GET` |
| 249 | `/api/logs` |
| 250 | `POST` |
| 250 | `/api/logs/clear` |
| 253 | `GET` |
| 253 | `/api/llm/antigravity/accounts` |
| 254 | `POST` |
| 254 | `/api/llm/antigravity/accounts` |
| 257 | `POST` |
| 257 | `/api/llm/antigravity/accounts/delete` |
| 260 | `POST` |
| 260 | `/api/llm/antigravity/accounts/refresh` |
| 263 | `POST` |
| 263 | `/api/llm/antigravity/accounts/toggle` |
| 266 | `POST` |
| 266 | `/api/llm/antigravity/accounts/quota` |
| 269 | `GET` |
| 269 | `/api/llm/antigravity/accounts/quota` |
| 272 | `GET` |
| 272 | `/api/llm/antigravity/oauth/url` |
| 275 | `GET` |
| 275 | `/api/llm/antigravity/oauth/callback` |
| 278 | `POST` |
| 278 | `/api/llm/antigravity/oauth/exchange` |
| 283 | `POST` |
| 283 | `/api/cmd` |
| 284 | `GET` |
| 284 | `/api/state` |
| 285 | `POST` |
| 285 | `/api/state` |
| 288 | `GET` |
| 288 | `/api/events` |
| 288 | `GET` |
| 288 | `/api/events/state` |
| 289 | `GET` |
| 289 | `/api/actions/since` |
| 294 | `GET` |
| 294 | `/api/music/check` |
| 295 | `POST` |
| 295 | `/api/music/install-ytdlp` |
| 296 | `GET` |
| 296 | `/api/music/list` |
| 297 | `GET` |
| 297 | `/music/` |
| 297 | `/api/music/stream` |
| 300 | `POST` |
| 300 | `/api/music/control` |
| 301 | `POST` |
| 301 | `/api/music/download` |
| 302 | `POST` |
| 302 | `/api/music/delete` |
| 305 | `GET` |
| 305 | `/api/assets/file/` |
| 306 | `/api/assets/file/` |
| 323 | `png` |
| 323 | `image/png` |
| 324 | `jpg` |
| 324 | `jpeg` |
| 324 | `image/jpeg` |
| 325 | `webp` |
| 325 | `image/webp` |
| 326 | `vrm` |
| 326 | `glb` |
| 326 | `model/gltf-binary` |
| 327 | `json` |
| 327 | `application/json` |
| 328 | `application/octet-stream` |
| 331 | `HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nC...` |
| 338 | `500 Internal Server Error` |
| 340 | `HTTP/1.1 500 Internal Server Error\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}` |
| 347 | `404 Asset Not Found` |
| 349 | `HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}` |
| 358 | `GET` |
| 361 | `index.html` |
| 368 | `html` |
| 368 | `text/html; charset=utf-8` |
| 369 | `js` |
| 369 | `application/javascript; charset=utf-8` |
| 370 | `css` |
| 370 | `text/css; charset=utf-8` |
| 371 | `svg` |
| 371 | `image/svg+xml` |
| 372 | `png` |
| 372 | `image/png` |
| 373 | `json` |
| 373 | `application/json` |
| 374 | `application/octet-stream` |
| 378 | `HTTP/1.1 200 OK\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\nC...` |
| 384 | `index.html` |
| 386 | `HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Contro...` |
| 392 | `404 Not Found` |
| 394 | `HTTP/1.1 404 Not Found\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow...` |
| 402 | `405 Method Not Allowed` |
| 404 | `HTTP/1.1 405 Method Not Allowed\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control-Allow-Origin: *\r\n\r\n{}` |
| 424 | `\r\n\r\n` |
| 438 | `Incomplete HTTP headers` |
| 447 | `content-length:` |
| 457 | `HTTP Content-Length exceeds 100MB limit` |
| 502 | `/api/events` |
| 502 | `/v1` |
| 502 | `/api/v1` |
| 508 | `OPTIONS` |
| 509 | `HTTP/1.1 204 No Content\r\n\                     Access-Control-Allow-Origin: *\r\n\                     Access-Contr...` |
| 546 | `127.0.0.1:42128` |
| 548 | ` [Web Dashboard & API Server] Running at http://{}` |
| 553 | ` [Web Dashboard & API Server] Running at http://{}` |
| 557 | `[Web Server] Failed to bind to {}: {}` |

---
