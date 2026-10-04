# Language Audit: `services` Module

- **Directory**: `src/services`
- **Total Files**: 17
- **Total Comments**: 56
- **Total String Literals**: 391

## Table of Contents
- [services/assets_service.rs](#servicesassets-servicers) (0 comments, 26 strings)
- [services/browser_session_service.rs](#servicesbrowser-session-servicers) (21 comments, 62 strings)
- [services/config_service.rs](#servicesconfig-servicers) (0 comments, 3 strings)
- [services/events_service.rs](#servicesevents-servicers) (17 comments, 13 strings)
- [services/llm_providers/antigravity_service.rs](#servicesllm-providersantigravity-servicers) (1 comments, 46 strings)
- [services/llm_providers/mod.rs](#servicesllm-providersmodrs) (0 comments, 0 strings)
- [services/logs_service.rs](#serviceslogs-servicers) (0 comments, 1 strings)
- [services/menu_service.rs](#servicesmenu-servicers) (3 comments, 23 strings)
- [services/mod.rs](#servicesmodrs) (0 comments, 0 strings)
- [services/music_service.rs](#servicesmusic-servicers) (2 comments, 40 strings)
- [services/openai_v1_service.rs](#servicesopenai-v1-servicers) (3 comments, 54 strings)
- [services/poses_service.rs](#servicesposes-servicers) (0 comments, 13 strings)
- [services/remote_service.rs](#servicesremote-servicers) (4 comments, 53 strings)
- [services/session_service.rs](#servicessession-servicers) (3 comments, 6 strings)
- [services/skills_service.rs](#servicesskills-servicers) (2 comments, 15 strings)
- [services/tray_service.rs](#servicestray-servicers) (0 comments, 21 strings)
- [services/tts_service.rs](#servicestts-servicers) (0 comments, 15 strings)

---

## `services/assets_service.rs`

- **Path**: `src/services/assets_service.rs`
- **Comments**: 0
- **Strings**: 26

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 11 | `char.vrm` |
| 25 | `svg` |
| 25 | `png` |
| 25 | `jpg` |
| 25 | `jpeg` |
| 25 | `webp` |
| 61 | `{"error":"Empty file payload"}` |
| 67 | `char.vrm` |
| 72 | `[Asset Manager] VRM model successfully uploaded ({:.2} MB) to {:?}` |
| 82 | `{"success":true,"message":"VRM model uploaded and reloaded successfully"}` |
| 86 | `[Asset Manager] Failed to save VRM file: {}` |
| 87 | `{{"error":"{}"}}` |
| 99 | `{"error":"Empty image payload"}` |
| 110 | `slot` |
| 117 | `idle` |
| 120 | `listening` |
| 120 | `char-avatar-listening.png` |
| 121 | `speaking` |
| 121 | `char-avatar-speaking.png` |
| 122 | `thinking` |
| 122 | `char-avatar-thinking.png` |
| 123 | `char-avatar-idle.png` |
| 133 | `[Asset Manager] Avatar image for slot '{}' saved to {:?}` |
| 140 | `{"success":true}` |
| 143 | `[Asset Manager] Failed to save avatar: {}` |
| 144 | `{{"error":"{}"}}` |

---

## `services/browser_session_service.rs`

- **Path**: `src/services/browser_session_service.rs`
- **Comments**: 21
- **Strings**: 62

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 22 | `//` | "SWITCH_OR_NAVIGATE", "CLOSE_TAB", "MEDIA_PLAY", "MEDIA_PAUSE", "MEDIA_TOGGLE", "MEDIA_NEXT", "MEDIA_PREV", "CLICK_EL... |
| 98 | `///` | Initializes dedicated background worker thread to synchronize \`active_tab_browser.json\` |
| 140 | `//` | Tunggu event with timeout 500ms so that remain responsif and konsisten in disk |
| 145 | `//` | Abaikan if berasal from dashboard internal |
| 157 | `//` | Use latest tabs list if content script snapshot does not carry tabs |
| 164 | `//` | Single Active Tab normalization: tab with matching URL / Title is marked active: true, others false |
| 190 | `//` | Save to in-memory cache global |
| 198 | `//` | Tulis to file disk |
| 235 | `//` | Prioritize tab reported active by browser |
| 252 | `//` | If active tab changed, update URL & base Title and clean old tab DOM elements |
| 265 | `//` | Clear old DOM elements to prevent mixing with new page before new snapshot arrives |
| 289 | `//` | Ensure disk file remains present and populated |
| 316 | `//` | Fallback synchronous if worker not yet initialized |
| 323 | `//` | Abaikan snapshot if berasal from internal dashboard Ragasukma sendiri |
| 337 | `//` | Fallback synchronous if worker not yet active |
| 355 | `//` | Limit queue to prevent build-up |
| 371 | `///` | Smart Browser Tab Navigator: |
| 372 | `///` | Check if tab already exists in browser. If it exists, switch & navigate to it without opening a new window/tab. |
| 422 | `//` | No matching tab found or extension not yet synced; open new tab via OS |
| 479 | `//` | Also delete legacy browser_sessions directory if it exists |
| 501 | `//` | Fallback: baca direct file assets/active_tab_browser.json from disk |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 23 | `Option::is_none` |
| 25 | `Option::is_none` |
| 27 | `Option::is_none` |
| 29 | `Option::is_none` |
| 31 | `Option::is_none` |
| 33 | `Option::is_none` |
| 35 | `Option::is_none` |
| 37 | `Option::is_none` |
| 44 | `Option::is_none` |
| 46 | `Option::is_none` |
| 48 | `Option::is_none` |
| 50 | `Option::is_none` |
| 52 | `Option::is_none` |
| 54 | `Option::is_none` |
| 56 | `Option::is_none` |
| 58 | `Vec::is_empty` |
| 60 | `Option::is_none` |
| 110 | `ragasukma-browser-worker` |
| 112 | `[Browser Worker] Dedicated BrowserSessionWorker active for active_tab_browser.json` |
| 125 | `assets/active_tab_browser.json` |
| 146 | `127.0.0.1:42128` |
| 146 | `localhost:42128` |
| 258 | `://` |
| 301 | `[Browser Worker] Channel disconnected, worker exiting.` |
| 324 | `127.0.0.1:42128` |
| 324 | `localhost:42128` |
| 325 | `Ignored internal dashboard snapshot` |
| 376 | `://` |
| 382 | `www.` |
| 403 | `[Smart Tab Reuse] Found existing open tab (ID: {}, Title: '{}'), activating & navigating without opening a new tab.` |
| 407 | `SWITCH_OR_NAVIGATE` |
| 434 | `\n###  OPEN BROWSER TABS:` |
| 436 | ` \`[ACTIVE_TAB]\`` |
| 438 | `{}. **{}**{} (URL: \`{}\`) [Tab ID: \`{}\`]` |
| 441 | `Untitled` |
| 450 | `\n` |
| 474 | `assets/active_tab_browser.json` |
| 480 | `assets/browser_sessions` |
| 484 | `[Active Tab] Initializing 'assets/active_tab_browser.json' for new session.` |
| 488 | `assets/active_tab_browser.json` |
| 489 | `Cargo.toml` |
| 492 | `active_tab_browser.json` |
| 517 | `{{"status":"ok","path":"{}"}}` |
| 517 | `/` |
| 518 | `application/json` |
| 521 | `{{"status":"error","message":"{}"}}` |
| 522 | `application/json` |
| 527 | `{{"status":"error","message":"Invalid snapshot JSON: {}"}}` |
| 530 | `application/json` |
| 546 | `{{"status":"ok","synced_tabs":{}}}` |
| 547 | `application/json` |
| 552 | `{{"status":"error","message":"Invalid tabs payload: {}"}}` |
| 555 | `application/json` |
| 563 | `application/json` |
| 565 | `{{"status":"error","message":"{}"}}` |
| 566 | `application/json` |
| 575 | `{{"has_command":true,"command":{}}}` |
| 576 | `application/json` |
| 578 | `{"has_command":false}` |
| 578 | `application/json` |
| 581 | `{"has_command":false}` |
| 581 | `application/json` |

---

## `services/config_service.rs`

- **Path**: `src/services/config_service.rs`
- **Comments**: 0
- **Strings**: 3

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 21 | `{{"error":"{}"}}` |
| 26 | `{"success":true}` |
| 29 | `{"error":"Invalid config format"}` |

---

## `services/events_service.rs`

- **Path**: `src/services/events_service.rs`
- **Comments**: 17
- **Strings**: 13

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 16 | `//` | Send header SSE lengkap + initial comment byte ': connected\n\n' |
| 17 | `//` | Crucial for Cloudflare Tunnel to avoid buffering or blocking the SSE handshake |
| 25 | `//` | --------------------------------------------------------------------------- |
| 26 | `//` | 1. GLOBAL APP STATE & ACTION SSE STREAM (/api/events/state) |
| 27 | `//` | Murni status visual AI and trigger skill perintah (Event-driven) |
| 28 | `//` | --------------------------------------------------------------------------- |
| 44 | `//` | Active SSE connection automatically indicates a remote client is currently connected |
| 60 | `//` | 1. Check for AI Visual State changes (event: state) |
| 116 | `//` | 2. Check for new action command from AI (event: action) - Multi-Client Pub-Sub |
| 130 | `//` | 3. Stream real-time activity logs to web console (event: log) - Zero Polling |
| 147 | `//` | Heartbeat ping every ~3 seconds (30 * 100ms) |
| 150 | `//` | Jaga status remote client remain active selama koneksi SSE terbuka |
| 163 | `//` | --------------------------------------------------------------------------- |
| 164 | `//` | 2. HTTP ACTIONS POLLING (/api/actions/since?last_id=...) |
| 165 | `//` | Fallback resilient for Mobile Tunnel / Cloudflare which membuffer SSE |
| 166 | `//` | --------------------------------------------------------------------------- |
| 184 | `//` | First connection / refresh: only take ID terbaru without sending repeat aksi history old! |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 18 | `HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache, no-transform\r\nConnection: keep-alive...` |
| 63 | `shutdown` |
| 66 | `idle` |
| 67 | `listening` |
| 68 | `speaking` |
| 69 | `thinking` |
| 109 | `event: state\ndata: {}\n\n` |
| 122 | `event: action\ndata: {}\n\n` |
| 135 | `event: log\ndata: {}\n\n` |
| 152 | `: ping\n\n` |
| 174 | `last_id` |
| 190 | `actions` |
| 191 | `last_id` |

---

## `services/llm_providers/antigravity_service.rs`

- **Path**: `src/services/llm_providers/antigravity_service.rs`
- **Comments**: 1
- **Strings**: 46

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 55 | `//` | Parse query string for ?code=... |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 8 | `http://localhost:42128/api/llm/antigravity/oauth/callback` |
| 12 | `refreshToken` |
| 12 | `token` |
| 16 | `projectId` |
| 34 | `ragasukma_{}` |
| 42 | `open=1` |
| 47 | `status` |
| 47 | `success` |
| 48 | `auth_url` |
| 49 | `redirect_uri` |
| 61 | `code` |
| 63 | `error` |
| 70 | `<!DOCTYPE html> <html> <head><title>OAuth Failed</title><meta charset="utf-8"></head> <body style="font-family: sans-...` |
| 90 | `<!DOCTYPE html> <html> <head><title>Invalid Code</title><meta charset="utf-8"></head> <body style="font-family: sans-...` |
| 117 | `<!DOCTYPE html> <html> <head>   <title>Google Antigravity Login Succeeded</title>   <meta charset="utf-8">   <meta na...` |
| 182 | `<!DOCTYPE html> <html> <head><title>OAuth Failed</title><meta charset="utf-8"></head> <body style="font-family: sans-...` |
| 207 | `{{"error":"Invalid request payload: {e}"}}` |
| 223 | `{{"status":"success","id":"{id}"}}` |
| 230 | `{{"error":"Failed to exchange code: {e}"}}` |
| 238 | `OK` |
| 239 | `Bad Request` |
| 240 | `Internal Server Error` |
| 241 | `OK` |
| 244 | `HTTP/1.1 {} {}\r\n\          Content-Type: text/html; charset=utf-8\r\n\          Content-Length: {}\r\n\          Co...` |
| 263 | `{{"error":"Failed to serialize accounts: {e}"}}` |
| 275 | `{{"error":"Invalid request payload: {e}"}}` |
| 287 | `{{"status":"success","id":"{id}"}}` |
| 294 | `{{"error":"Failed to add account: {e}"}}` |
| 307 | `{{"error":"Invalid request payload: {e}"}}` |
| 317 | `{"status":"success"}` |
| 319 | `{"error":"Account not found"}` |
| 330 | `{{"error":"Invalid request payload: {e}"}}` |
| 342 | `{{"status":"success","project_id":"{pid}"}}` |
| 349 | `{{"error":"Token refresh failed: {e}"}}` |
| 362 | `{{"error":"Invalid request payload: {e}"}}` |
| 374 | `{{"status":"success","is_active":{active}}}` |
| 378 | `{{"error":"{e}"}}` |
| 393 | `id` |
| 406 | `{"status":"error","error":"Account ID is required"}` |
| 421 | `{"status":"error","error":"Serialization error"}` |
| 427 | `status` |
| 427 | `error` |
| 428 | `account_id` |
| 429 | `error` |
| 430 | `models` |
| 431 | `weekly` |

---

## `services/llm_providers/mod.rs`

- **Path**: `src/services/llm_providers/mod.rs`
- **Comments**: 0
- **Strings**: 0

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

_No string literals found._

---

## `services/logs_service.rs`

- **Path**: `src/services/logs_service.rs`
- **Comments**: 0
- **Strings**: 1

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 13 | `{"success":true}` |

---

## `services/menu_service.rs`

- **Path**: `src/services/menu_service.rs`
- **Comments**: 3
- **Strings**: 23

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 63 | `//` | Mode Remote PC: TTS in PC (voice_muted = false), Mic in HP (mic PC dimute: mic_muted = true), Music in PC |
| 73 | `//` | Mode Full HP: TTS in HP, Mic in HP, Music in HP (all audio PC dimute) |
| 83 | `//` | Mode Full PC: TTS in PC, Mic in PC, Music in PC (all active normal in PC) |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 20 | `start_chat` |
| 30 | `stop_chat` |
| 36 | `toggle_pause` |
| 41 | `mute_voice` |
| 46 | `unmute_voice` |
| 51 | `mute_mic` |
| 56 | `unmute_mic` |
| 61 | `mode_remote_desktop` |
| 71 | `mode_full_mobile` |
| 81 | `mode_reset_desktop` |
| 81 | `reset_desktop_mode` |
| 81 | `mode_desktop` |
| 94 | `{"success":true}` |
| 96 | `{"error":"Invalid cmd payload"}` |
| 103 | `idle` |
| 104 | `listening` |
| 105 | `speaking` |
| 106 | `thinking` |
| 149 | `start_chat` |
| 152 | `stop_chat` |
| 154 | `toggle_pause` |
| 160 | `{"success":true}` |
| 162 | `{"error":"Invalid state payload"}` |

---

## `services/mod.rs`

- **Path**: `src/services/mod.rs`
- **Comments**: 0
- **Strings**: 0

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

_No string literals found._

---

## `services/music_service.rs`

- **Path**: `src/services/music_service.rs`
- **Comments**: 2
- **Strings**: 40

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 103 | `//` | 1. Coba take name file from path direct /music/<filename> |
| 107 | `//` | 2. Fallback query param ?filename=... or ?file=... |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 22 | `success` |
| 23 | `title` |
| 24 | `description` |
| 35 | `{"success":false,"error":"Invalid control action or payload"}` |
| 50 | `success` |
| 51 | `message` |
| 52 | `info` |
| 59 | `success` |
| 60 | `message` |
| 72 | `music_dir` |
| 73 | `tracks` |
| 104 | `/music/` |
| 110 | `filename` |
| 110 | `file` |
| 116 | `{"error":"Missing filename param"}` |
| 130 | `mp3` |
| 130 | `audio/mpeg` |
| 131 | `webm` |
| 131 | `audio/webm` |
| 132 | `opus` |
| 132 | `audio/opus` |
| 133 | `ogg` |
| 133 | `audio/ogg` |
| 134 | `m4a` |
| 134 | `mp4` |
| 134 | `aac` |
| 134 | `audio/mp4` |
| 135 | `wav` |
| 135 | `audio/wav` |
| 136 | `flac` |
| 136 | `audio/flac` |
| 137 | `application/octet-stream` |
| 142 | `{"error":"File not found"}` |
| 144 | `HTTP/1.1 404 Not Found\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\nAccess-Control...` |
| 174 | `{{"success":false,"error":"{}"}}` |
| 182 | `{"error":"Invalid payload, expected {query}"}` |
| 197 | `info.json` |
| 198 | `{}.info.json` |
| 202 | `{"success":true}` |
| 207 | `{"error":"Invalid payload, expected {filename}"}` |

---

## `services/openai_v1_service.rs`

- **Path**: `src/services/openai_v1_service.rs`
- **Comments**: 3
- **Strings**: 54

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 151 | `//` | Send initial chunk with role: "assistant" |
| 234 | `//` | Send SSE comment to keep TCP connection alive during thinking phases |
| 286 | `//` | Non-streaming response |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 35 | `Option::is_none` |
| 80 | `HTTP/1.1 200 OK\r\n\         Access-Control-Allow-Origin: *\r\n\         Access-Control-Allow-Methods: GET, POST, PUT...` |
| 87 | `ag/claude-3-5-sonnet` |
| 88 | `ag/claude-3-7-sonnet` |
| 89 | `ag/claude-sonnet-4-6` |
| 90 | `ag/claude-opus-4-6-thinking` |
| 91 | `ag/gemini-3.8-flash-high` |
| 92 | `ag/gemini-3.8-pro-high` |
| 93 | `ag/gemini-3.7-flash-high` |
| 94 | `ag/gemini-3.5-flash-high` |
| 95 | `ag/gemini-2.5-pro` |
| 96 | `ag/gemini-2.5-flash` |
| 97 | `ag/gpt-4o` |
| 98 | `ag/gpt-4o-mini` |
| 99 | `ag/gpt-oss-120b-medium` |
| 106 | `model` |
| 108 | `antigravity` |
| 113 | `list` |
| 120 | `{"error":"Failed to serialize models"}` |
| 129 | `{{"error":{{"message":"Invalid JSON request: {}"}}}}` |
| 139 | `chatcmpl-{}` |
| 145 | `HTTP/1.1 200 OK\r\n\             Content-Type: text/event-stream\r\n\             Cache-Control: no-cache\r\n\       ...` |
| 154 | `chat.completion.chunk` |
| 159 | `role` |
| 159 | `assistant` |
| 164 | `data: {}\n\n` |
| 172 | `stop` |
| 181 | `chat.completion.chunk` |
| 186 | `content` |
| 191 | `data: {}\n\n` |
| 203 | `index` |
| 204 | `type` |
| 204 | `function` |
| 205 | `function` |
| 206 | `arguments` |
| 210 | `id` |
| 213 | `function` |
| 213 | `name` |
| 218 | `chat.completion.chunk` |
| 223 | `tool_calls` |
| 228 | `data: {}\n\n` |
| 235 | `:\n\n` |
| 257 | `chat.completion.chunk` |
| 267 | `data: {}\n\n` |
| 270 | `data: [DONE]\n\n` |
| 275 | `error` |
| 276 | `message` |
| 276 | `Stream error: {e}` |
| 279 | `data: {}\n\n` |
| 281 | `data: [DONE]\n\n` |
| 299 | `chat.completion` |
| 305 | `assistant` |
| 321 | `{"error":"Failed to serialize response"}` |
| 325 | `{{"error":{{"message":"{}"}}}}` |

---

## `services/poses_service.rs`

- **Path**: `src/services/poses_service.rs`
- **Comments**: 0
- **Strings**: 13

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 23 | `{{"error":"{}"}}` |
| 29 | `{"success":true}` |
| 32 | `{"error":"Invalid poses format"}` |
| 55 | `listening` |
| 56 | `speaking` |
| 57 | `thinking` |
| 65 | `{"success":true}` |
| 67 | `{"error":"Invalid preview payload"}` |
| 91 | `listening` |
| 92 | `speaking` |
| 93 | `thinking` |
| 113 | `{"success":true}` |
| 115 | `{"error":"Invalid live tuner payload"}` |

---

## `services/remote_service.rs`

- **Path**: `src/services/remote_service.rs`
- **Comments**: 4
- **Strings**: 53

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 163 | `//` | 1. Anti-Echolalia Protection: Reject transcript if it matches the assistant speaker echo |
| 181 | `//` | If AI is currently speaking and voice interruption is disabled by user, |
| 182 | `//` | reject transcript outright to prevent overlapping speech. |
| 277 | `//` | Guard: Do not allow entering listening mode if voice assistant is not yet enabled (standby) |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 14 | `status` |
| 15 | `url` |
| 16 | `auth_key` |
| 17 | `cloudflared` |
| 34 | `success` |
| 35 | `message` |
| 36 | `info` |
| 43 | `success` |
| 44 | `message` |
| 56 | `starting` |
| 58 | `running` |
| 61 | `status` |
| 62 | `url` |
| 63 | `auth_key` |
| 70 | `error` |
| 81 | `status` |
| 81 | `stopped` |
| 101 | `key` |
| 107 | `{"error":"Unauthorized: invalid remote key"}` |
| 120 | `{"error":"Empty text string"}` |
| 126 | `[Mobile Remote] Received text from phone: "{}"` |
| 130 | `{"success":true,"message":"Text sent to LLM"}` |
| 133 | `[Mobile Web STT] Chat loop is not active!` |
| 137 | `{"error":"Voice chat loop is not currently active"}` |
| 144 | `{"error":"Invalid payload, expected {text: string}"}` |
| 152 | `transcript` |
| 159 | `{"error":"Empty transcript string"}` |
| 166 | `[Anti-Echolalia] Echo transcript from assistant speaker ignored: "{}"` |
| 172 | `{"success":false,"ignored":true,"message":"Echo of assistant speech ignored"}` |
| 185 | `[Chrome Web STT] Transcript ignored because assistant is speaking (interruption disabled): "{}"` |
| 191 | `{"success":false,"ignored":true,"message":"AI is currently speaking and voice interruption is disabled"}` |
| 200 | `[Chrome Web STT] Voice interruption cuts assistant speech: "{}"` |
| 205 | `[Chrome Web STT] Transcript received: "{}"` |
| 210 | `{asst_name} Thinking...` |
| 215 | `{"success":true,"message":"Transcript injected to LLM"}` |
| 218 | `[Chrome Web STT] Chat loop is not active!` |
| 222 | `{"error":"Voice chat loop is not currently active"}` |
| 229 | `{"error":"Invalid payload, expected {text: string} or {transcript: string}"}` |
| 251 | `neutral` |
| 255 | `speaking` |
| 257 | `listening` |
| 259 | `idle` |
| 262 | `speaking` |
| 268 | `{asst_name} Speaking` |
| 276 | `listening` |
| 279 | `{"success":true,"ignored":true}` |
| 287 | `{asst_name} Listening...` |
| 295 | `thinking` |
| 300 | `{asst_name} Thinking...` |
| 313 | `Standby` |
| 314 | `neutral` |
| 321 | `{"success":true}` |
| 323 | `{"error":"Invalid TTS state payload"}` |

---

## `services/session_service.rs`

- **Path**: `src/services/session_service.rs`
- **Comments**: 3
- **Strings**: 6

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 8 | `///` | Claim ownership of single web dashboard session (called once when tab opens or refreshes) |
| 32 | `//` | Send event session claim to antrean so that all SSE client terupdate instan |
| 45 | `///` | Update active tab when berpindah tab in dashboard |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 21 | `menu` |
| 33 | `session:claimed` |
| 35 | `{"success":true}` |
| 40 | `{"error":"Invalid payload, expected {token}"}` |
| 58 | `{"success":true}` |
| 60 | `{"error":"Invalid tab payload"}` |

---

## `services/skills_service.rs`

- **Path**: `src/services/skills_service.rs`
- **Comments**: 2
- **Strings**: 15

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 99 | `//` | 1. Check whether this is a built-in system action, internal skill, or application command |
| 116 | `//` | 2. Fallback to eksekusi command shell OS |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 18 | `ragasukma-companion.zip` |
| 23 | `folder` |
| 24 | `zip` |
| 62 | `{{"error":"{}"}}` |
| 64 | `{"success":true}` |
| 67 | `{"error":"Invalid skill format"}` |
| 81 | `{{"error":"{}"}}` |
| 83 | `{"success":true}` |
| 86 | `{"error":"Invalid delete payload"}` |
| 102 | `[Skill Run UI] Executing internal skill '{}' -> {}` |
| 109 | `{{"success":true,"message":"{}"}}` |
| 110 | `\"` |
| 118 | `{{"error":"{}"}}` |
| 120 | `{"success":true}` |
| 123 | `{"error":"Invalid run payload"}` |

---

## `services/tray_service.rs`

- **Path**: `src/services/tray_service.rs`
- **Comments**: 0
- **Strings**: 21

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 24 | `emotion` |
| 28 | `neutral` |
| 34 | `idle` |
| 35 | `listening` |
| 36 | `thinking` |
| 37 | `speaking` |
| 40 | `{}: {}` |
| 43 | `text` |
| 44 | `alt` |
| 45 | `tooltip` |
| 46 | `class` |
| 47 | `emotion` |
| 48 | `description` |
| 67 | `class` |
| 67 | `idle` |
| 69 | `emotion` |
| 71 | `neutral` |
| 74 | `listening` |
| 75 | `speaking` |
| 76 | `thinking` |
| 84 | `neutral` |

---

## `services/tts_service.rs`

- **Path**: `src/services/tts_service.rs`
- **Comments**: 0
- **Strings**: 15

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 24 | `error` |
| 24 | `Invalid JSON request: {e}` |
| 31 | `error` |
| 31 | `Text cannot be empty` |
| 42 | `error` |
| 42 | `Failed to initialize Supertonic engine: {e}` |
| 56 | `F4` |
| 64 | `id` |
| 75 | `{:x}` |
| 79 | `HTTP/1.1 200 OK\r\n\                  Content-Type: audio/wav\r\n\                  Content-Length: {}\r\n\          ...` |
| 87 | `error` |
| 87 | `Synthesis failed: {e}` |
| 98 | `id` |
| 108 | `HTTP/1.1 200 OK\r\n\              Content-Type: audio/wav\r\n\              Content-Length: {}\r\n\              Conn...` |
| 114 | `{"error":"No audio available"}` |

---
