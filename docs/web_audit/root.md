# Web Language Audit: `Root`

- **Category**: `web/src/` (Root & App level)
- **Total Files**: 4
- **Total Comments**: 43
- **Total Console Calls**: 14

## Table of Contents
- [src/App.tsx](#srcapptsx) (18 comments, 11 console logs)
- [src/api.ts](#srcapits) (10 comments, 0 console logs)
- [src/index.tsx](#srcindextsx) (1 comments, 0 console logs)
- [src/playerStore.ts](#srcplayerstorets) (14 comments, 3 console logs)

---

## `src/App.tsx`

- **Path**: `web/src/App.tsx`
- **Comments**: 18
- **Console Calls**: 11

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 19 | `//` | Token unik per instance tab, created when module pertama kali loaded |
| 63 | `//` | Detect whether current tab is mobile remote client (opened via QR scan or key auth) |
| 70 | `//` | Only treat as mobile client if possessing auth key AND originating from cloudflare tunnel or mobile hash |
| 85 | `//` | 1. Check query parameter standar (?key=...) |
| 90 | `//` | 2. Check query parameter inside hash if URL is #remote?key=... |
| 113 | `//` | Synchronize desktop browser tab title when playing music |
| 136 | `//` | 1. Stop Chrome STT, TTS, & VAD Barge-in monitor |
| 139 | `//` | 2. Disconnect SSE stream and cancel listeners |
| 146 | `//` | 3. Close session, global audio, and BroadcastChannel |
| 157 | `//` | Smartphone Remote Client mode: bypass single-tab kill |
| 189 | `//` | CLAIM 1X |
| 197 | `//` | Start voice session manager (STT, VAD, Echo Guard) |
| 200 | `//` | SSE: UI state update & shutdown / claim detection |
| 303 | `/* */` | Tab Deactivated / Shutdown Modal |
| 346 | `{/* */}` | Top Header |
| 372 | `{/* */}` | Scrollable View Content |
| 415 | `{/* */}` | DISPLAY KHUSUS SMARTPHONE (ONLY MIC PTT + CONSOLE LOGS REAL-TIME) |
| 422 | `{/* */}` | Toast |


### Console Logs (`console.*`)

| Line | Call | Arguments |
|:---:|:---:|:---|
| 159 | `console.log` | `'[Session] Remote client detected, skipping single-tab exclusive claim'` |
| 163 | `console.log` | `'[Session] Tab mount, MY_TOKEN =', MY_TOKEN` |
| 170 | `console.log` | `'[Session] BroadcastChannel msg:', e.data, 'MY_TOKEN =', MY_TOKEN` |
| 172 | `console.log` | `'[Session] BroadcastChannel -> killThisTab'` |
| 176 | `console.log` | `'[Session] BroadcastChannel listener ready'` |
| 178 | `console.warn` | `'[Session] BroadcastChannel not supported:', err` |
| 183 | `console.log` | `'[Session] storage event -> killThisTab'` |
| 191 | `console.log` | `'[Session] Claiming ownership 1x...'` |
| 205 | `console.log` | `'[Session] Backend signaled SHUTDOWN'` |
| 227 | `console.log` | `'[Runtime Config] Updated after save:', cfg` |
| 238 | `console.log` | `'[Session] Backend signaled another tab claimed ownership -> killThisTab'` |

---

## `src/api.ts`

- **Path**: `web/src/api.ts`
- **Comments**: 10
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 17 | `//` | Config Service |
| 21 | `//` | Poses Service |
| 27 | `//` | Skills Service |
| 35 | `//` | Logs Service |
| 39 | `//` | Menu / App State & Command Service |
| 44 | `//` | Assets Service |
| 49 | `//` | Single Client Session & SSE Events Service |
| 59 | `//` | Music Storage & Downloader Service |
| 68 | `//` | LLM Providers - Antigravity |
| 77 | `//` | Remote Tunnel & Text Injection Service |


### Console Logs (`console.*`)

_No console logs found._

---

## `src/index.tsx`

- **Path**: `web/src/index.tsx`
- **Comments**: 1
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 1 | `/* */` | @refresh reload |


### Console Logs (`console.*`)

_No console logs found._

---

## `src/playerStore.ts`

- **Path**: `web/src/playerStore.ts`
- **Comments**: 14
- **Console Calls**: 3

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 13 | `//` | Bridge status for komponen BottomPlayer, PiP & UI |
| 87 | `//` | Initialize automatically audio element singleton in browser |
| 109 | `//` | Listen for mute status from server (if Phone Only active, mute music playback on PC browser) |
| 136 | `//` | --------------------------------------------------------------------------- |
| 137 | `//` | 1. LISTEN TO EVENTS FROM BACKEND SINGLE SOURCE OF TRUTH (SSE ACTION) |
| 138 | `//` | --------------------------------------------------------------------------- |
| 140 | `//` | If this tab is a mobile remote client, let mobile audioSync handle playback |
| 172 | `//` | 1. Check Exact |
| 182 | `//` | 2. Check Substring |
| 190 | `//` | 3. Check Multi-word Token |
| 224 | `//` | If server is in Phone Only mode (isDesktopVoiceMuted = true), PC browser must remain silent |
| 241 | `//` | --------------------------------------------------------------------------- |
| 242 | `//` | 2. REMOTE CONTROL ACTIONS (WEB DASHBOARD & PIP) |
| 243 | `//` | --------------------------------------------------------------------------- |


### Console Logs (`console.*`)

| Line | Call | Arguments |
|:---:|:---:|:---|
| 83 | `console.warn` | `'⚠️ [AudioPlayer] Error saat memutar audio stream:', e` |
| 155 | `console.log` | `'🎵 [playerStore] Menerima action SSE:', act` |
| 236 | `console.warn` | `'[AudioPlayer] Autoplay prevented:', err` |

---
