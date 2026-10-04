# Web Language Audit: `Services`

- **Category**: `web/src/services`
- **Total Files**: 15
- **Total Comments**: 116
- **Total Console Calls**: 40

## Table of Contents
- [src/services/antigravityModels.ts](#srcservicesantigravitymodelsts) (6 comments, 0 console logs)
- [src/services/antigravityService.ts](#srcservicesantigravityservicets) (0 comments, 0 console logs)
- [src/services/assetsService.ts](#srcservicesassetsservicets) (2 comments, 0 console logs)
- [src/services/base.ts](#srcservicesbasets) (2 comments, 0 console logs)
- [src/services/configService.ts](#srcservicesconfigservicets) (0 comments, 0 console logs)
- [src/services/eventsService.ts](#srcserviceseventsservicets) (11 comments, 4 console logs)
- [src/services/index.ts](#srcservicesindexts) (0 comments, 0 console logs)
- [src/services/logsService.ts](#srcserviceslogsservicets) (0 comments, 0 console logs)
- [src/services/menuService.ts](#srcservicesmenuservicets) (0 comments, 0 console logs)
- [src/services/musicService.ts](#srcservicesmusicservicets) (1 comments, 0 console logs)
- [src/services/pipService.ts](#srcservicespipservicets) (44 comments, 1 console logs)
- [src/services/posesService.ts](#srcservicesposesservicets) (2 comments, 0 console logs)
- [src/services/remoteService.ts](#srcservicesremoteservicets) (0 comments, 5 console logs)
- [src/services/skillsService.ts](#srcservicesskillsservicets) (0 comments, 0 console logs)
- [src/services/voiceSession.ts](#srcservicesvoicesessionts) (48 comments, 30 console logs)

---

## `src/services/antigravityModels.ts`

- **Path**: `web/src/services/antigravityModels.ts`
- **Comments**: 6
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 2 | `//` | e.g. 'ag/gemini-3.8-flash-high' |
| 10 | `//` | Gemini 3.8 Series |
| 40 | `//` | Gemini 3.7 Series |
| 63 | `//` | Claude Series |
| 93 | `//` | Gemini Legacy & Pro |
| 151 | `//` | Specialist & Open Weights |


### Console Logs (`console.*`)

_No console logs found._

---

## `src/services/antigravityService.ts`

- **Path**: `web/src/services/antigravityService.ts`
- **Comments**: 0
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

_No comments found._


### Console Logs (`console.*`)

_No console logs found._

---

## `src/services/assetsService.ts`

- **Path**: `web/src/services/assetsService.ts`
- **Comments**: 2
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 51 | `//` | Invalidate cache so data refreshes |
| 72 | `//` | Invalidate cache |


### Console Logs (`console.*`)

_No console logs found._

---

## `src/services/base.ts`

- **Path**: `web/src/services/base.ts`
- **Comments**: 2
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 3 | `//` | In smartphone / remote tunnel, window.location.origin is https://xxx.trycloudflare.com |
| 4 | `//` | Do not hardcode http://127.0.0.1:42128 because mobile devices cannot directly access laptop localhost! |


### Console Logs (`console.*`)

_No console logs found._

---

## `src/services/configService.ts`

- **Path**: `web/src/services/configService.ts`
- **Comments**: 0
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

_No comments found._


### Console Logs (`console.*`)

_No console logs found._

---

## `src/services/eventsService.ts`

- **Path**: `web/src/services/eventsService.ts`
- **Comments**: 11
- **Console Calls**: 4

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 3 | `//` | --------------------------------------------------------------------------- |
| 4 | `//` | 1. APP STATE SSE STREAM (/api/events/state) |
| 5 | `//` | Using event 'state' for visual AI and event 'action' for trigger skill |
| 6 | `//` | --------------------------------------------------------------------------- |
| 46 | `//` | Event: 'state' -> Murni status visual AI and token |
| 58 | `//` | Event: 'action' -> Murni aksi perintah which dipicu AI (example: music:play) |
| 68 | `//` | Event: 'log' -> Real-time console activity log stream (Zero Polling) |
| 135 | `//` | Single Client Session Claim (1x when tab opened) |
| 149 | `//` | Notify server of active tab change |
| 183 | `//` | 2. Subscribe Action Events (Perintah aksi AI murni, like music, notifikasi) |
| 194 | `//` | 3. Subscribe Live Logs via SSE (Realtime zero-polling console stream) |


### Console Logs (`console.*`)

| Line | Call | Arguments |
|:---:|:---:|:---|
| 54 | `console.error` | `'Failed to parse SSE state:', err` |
| 64 | `console.error` | `'Failed to parse SSE action:', err` |
| 74 | `console.error` | `'Failed to parse SSE log:', err` |
| 94 | `console.error` | `'Failed to init State EventSource:', err` |

---

## `src/services/index.ts`

- **Path**: `web/src/services/index.ts`
- **Comments**: 0
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

_No comments found._


### Console Logs (`console.*`)

_No console logs found._

---

## `src/services/logsService.ts`

- **Path**: `web/src/services/logsService.ts`
- **Comments**: 0
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

_No comments found._


### Console Logs (`console.*`)

_No console logs found._

---

## `src/services/menuService.ts`

- **Path**: `web/src/services/menuService.ts`
- **Comments**: 0
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

_No comments found._


### Console Logs (`console.*`)

_No console logs found._

---

## `src/services/musicService.ts`

- **Path**: `web/src/services/musicService.ts`
- **Comments**: 1
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 12 | `//` | detik |


### Console Logs (`console.*`)

_No console logs found._

---

## `src/services/pipService.ts`

- **Path**: `web/src/services/pipService.ts`
- **Comments**: 44
- **Console Calls**: 1

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 30 | `//` | Render on buffer 2x HiDPI / Retina so that font, border, and image super tajam (HD) |
| 40 | `//` | Attach video to DOM invisibly so the browser does not throttle it |
| 49 | `//` | Synchronize PiP status when user manually closes the PiP window |
| 55 | `//` | Synchronize when user clicks Play/Pause in floating PiP window |
| 74 | `//` | Registrasi MediaSession API bawaan browser OS |
| 98 | `//` | Dipanggil direct from playerStore when audio play |
| 109 | `//` | Dipanggil direct from playerStore when audio pause |
| 183 | `//` | ── 1. BACKGROUND & AMBIENT GLOW ── |
| 186 | `//` | Warm Cream / Light Earth Tone Base Gradient |
| 194 | `//` | Album Artwork / Thumbnail Loader |
| 199 | `//` | Dynamic Ambient Glow from Album |
| 214 | `//` | Crisp Window Frame Border |
| 221 | `//` | ── 2. LEFT COLUMN: ALBUM ARTWORK CARD (Zero Radius Square) ── |
| 226 | `//` | Drop shadow behind cover |
| 236 | `//` | Draw Thumbnail inside sharp rect |
| 255 | `//` | Clean vector music note |
| 275 | `//` | Sheen |
| 282 | `//` | Live Equalizer Overlay inside Album (Bottom-Right) |
| 294 | `//` | 4 Animated Waveform Bars |
| 305 | `//` | Crisp Border Stroke on Cover |
| 312 | `//` | ── 3. RIGHT COLUMN: METADATA & CONTROLS ── |
| 316 | `//` | Top Header: Status Badge & Brand Watermark |
| 329 | `//` | Badge Card Background & Clean Border |
| 336 | `//` | Indicator Dot (Centered Vertically) |
| 342 | `//` | Status Text (Crisp Math Baseline) |
| 348 | `//` | Brand Watermark on Top-Right |
| 356 | `//` | Track Metadata (Title & Artist) |
| 360 | `//` | Title |
| 371 | `//` | Artist |
| 379 | `//` | ── 4. TIMELINE PROGRESS BAR ── |
| 388 | `//` | Background Bar |
| 396 | `//` | Filled Gradient Bar |
| 406 | `//` | Square Knob |
| 414 | `//` | Time Indicators |
| 427 | `//` | Subtle Bottom Footer Hint |
| 436 | `//` | Restore HiDPI scale |
| 447 | `//` | Render frame pertama seketika |
| 450 | `//` | ── Web Worker Timer ── |
| 451 | `//` | Chromium aggressively freezes requestAnimationFrame and throttles standard setInterval |
| 452 | `//` | when web tab is minimized or in the background. Web Worker timer runs in a separate thread |
| 453 | `//` | so PiP render loop maintains a constant real-time 30 FPS without freezing! |
| 468 | `//` | Fallback standard interval if Web Worker is not permitted |
| 504 | `//` | Stream canvas to video element |
| 534 | `//` | Automatically synchronize when playback status changes |


### Console Logs (`console.*`)

| Line | Call | Arguments |
|:---:|:---:|:---|
| 518 | `console.error` | `'[PiP Error]:', err` |

---

## `src/services/posesService.ts`

- **Path**: `web/src/services/posesService.ts`
- **Comments**: 2
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 64 | `//` | Cancel previous in-flight slider request if still running |
| 80 | `//` | Normal when cancelled oleh slider input new |


### Console Logs (`console.*`)

_No console logs found._

---

## `src/services/remoteService.ts`

- **Path**: `web/src/services/remoteService.ts`
- **Comments**: 0
- **Console Calls**: 5

### Comments (`//`, `/* */`, `{/* */}`)

_No comments found._


### Console Logs (`console.*`)

| Line | Call | Arguments |
|:---:|:---:|:---|
| 31 | `console.error` | `'[RemoteService] getTunnelStatus error:', e` |
| 43 | `console.error` | `'[RemoteService] startTunnel error:', e` |
| 55 | `console.error` | `'[RemoteService] stopTunnel error:', e` |
| 67 | `console.error` | `'[RemoteService] checkCloudflared error:', e` |
| 79 | `console.error` | `'[RemoteService] installCloudflared error:', e` |

---

## `src/services/skillsService.ts`

- **Path**: `web/src/services/skillsService.ts`
- **Comments**: 0
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

_No comments found._


### Console Logs (`console.*`)

_No console logs found._

---

## `src/services/voiceSession.ts`

- **Path**: `web/src/services/voiceSession.ts`
- **Comments**: 48
- **Console Calls**: 30

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 14 | `//` | Runtime State Flags |
| 30 | `//` | VAD Audio Nodes |
| 35 | `//` | Echo Guard Buffer |
| 38 | `//` | TTS Deduplication & Session Isolation |
| 49 | `//` | Synchronously initialize SpeechRecognition from start |
| 57 | `//` | 1. Resume Web Audio context if sempat suspended oleh autoplay policy |
| 62 | `//` | 2. If speech recognition was held back by browser permissions (not-allowed / gesture required) |
| 76 | `//` | Recovery automatically if session assistant active but recognition not yet jalan |
| 117 | `//` | If assistant is active (e.g. after F5 refresh during active session), start STT immediately! |
| 125 | `//` | --------------------------------------------------------------------------- |
| 126 | `//` | 1. Anti-Echolalia Echo Guard |
| 127 | `//` | --------------------------------------------------------------------------- |
| 128 | `//` | 1. Anti-Echolalia Echo Guard (Acoustic Speaker Feedback Protection) |
| 129 | `//` | --------------------------------------------------------------------------- |
| 144 | `//` | Speaker acoustic echo into microphone is monitored up to 15 seconds after voice plays |
| 156 | `//` | Inspect acoustic echo up to the last 15 seconds |
| 163 | `//` | 1. Check exact match or substring lengkap |
| 178 | `//` | 2. Check Trigram (3 consecutive words) identical to assistant text |
| 191 | `//` | 3. Check Token Overlap Ratio (> 50%) |
| 218 | `//` | --------------------------------------------------------------------------- |
| 219 | `//` | 2. Speech Recognition (STT) Setup |
| 220 | `//` | --------------------------------------------------------------------------- |
| 240 | `//` | Safety watchdog: only triggers if browser hangs or never emits onspeechend (15s) |
| 254 | `//` | User finished speaking; provide 1.2s pause for STT to produce transcript |
| 255 | `//` | If no words were generated (only breathing/coughing), automatically reset to idle |
| 284 | `//` | Turn off local active listening status without sending 'idle' to backend! |
| 285 | `//` | Avoids race conditions where an 'idle' request overrides the 'Thinking' status in backend |
| 316 | `//` | 'aborted' and 'no-speech' is event normal lifecycle Web Speech API: |
| 317 | `//` | - 'aborted': triggered when app deliberately calls recognition.abort() (e.g. when TTS starts speaking or mic mute). |
| 318 | `//` | - 'no-speech': user remained silent before timeout. |
| 354 | `//` | Assistant currently speaking via TTS; do not reset state to idle and do not restart listening |
| 359 | `//` | Do not restart if currently awaiting user gesture (not-allowed) |
| 386 | `//` | If running but recognition not active yet (or state changed), start recognition! |
| 484 | `//` | --------------------------------------------------------------------------- |
| 485 | `//` | 3. VAD Barge-In Monitoring |
| 486 | `//` | --------------------------------------------------------------------------- |
| 592 | `//` | --------------------------------------------------------------------------- |
| 593 | `//` | 4. Speech Synthesis (Supertonic-3 Neural ONNX Audio Stream) |
| 594 | `//` | --------------------------------------------------------------------------- |
| 634 | `//` | Backend plays audio directly to PC hardware speakers |
| 643 | `//` | Mute microphone while voice plays on PC speakers, adding 400ms pause to eliminate echo |
| 658 | `//` | Stop previous Supertonic audio playback if any |
| 682 | `//` | Notify backend that voice is actually playing from speakers (not still loading) |
| 683 | `//` | Ensures AIUEO lip movement is 100% synced with voice waveforms |
| 736 | `//` | If audio_url is not present yet, request backend to synthesize directly via Supertonic |
| 761 | `//` | --------------------------------------------------------------------------- |
| 762 | `//` | 5. External Event Synchronizers |
| 763 | `//` | --------------------------------------------------------------------------- |


### Console Logs (`console.*`)

| Line | Call | Arguments |
|:---:|:---:|:---|
| 65 | `console.log` | `'[Chrome Web STT] User interaction detected, restoring microphone session...'` |
| 119 | `console.log` | `'[Voice Session] Auto-starting recognition on session init...'` |
| 166 | `console.warn` | `'[Echo Guard] Detected assistant voice echo substring:', transLower` |
| 184 | `console.warn` | `'[Echo Guard] Detected assistant speaker trigram echo:', trigram` |
| 208 | `console.warn` | `\`[Echo Guard] Detected high overlap (${Math.round(matchRatio * 100)}%) speaker echo:\`,         transcript` |
| 224 | `console.warn` | `'[Chrome Web STT] Web Speech API is not supported in this browser.'` |
| 268 | `console.log` | `'[Chrome Web STT] Result ignored due to mic mute / TTS active / assistant standby'` |
| 279 | `console.warn` | `'[Echo Guard] Self-echo speaker reflection ignored:', transcript` |
| 287 | `console.log` | `'[Chrome Web STT] Sending transcript:', transcript` |
| 292 | `console.error` | `'[Chrome Web STT] Failed to send transcript:', err` |
| 306 | `console.log` | `'[Chrome Web STT] Speech recognition started & active'` |
| 324 | `console.warn` | `'[Chrome Web STT] Google STT network disturbance detected, restoring connection...'` |
| 333 | `console.warn` | `'[Chrome Web STT] Error:', e.error` |
| 339 | `console.warn` | `'[Chrome Web STT] Microphone access blocked by browser. Awaiting user interaction (click/tap)...'` |
| 377 | `console.warn` | `'[Chrome Web STT] Init failed:', err` |
| 388 | `console.log` | `'[Voice Session] Assistant running -> starting recognition...'` |
| 394 | `console.log` | `'[Voice Session] Assistant stopped -> stopping recognition...'` |
| 429 | `console.log` | `\`[Chrome Web STT] resetToIdle ignored because TTS is actively speaking (${reason})\`` |
| 432 | `console.log` | `\`[Chrome Web STT] Reset listening to idle (${reason})\`` |
| 479 | `console.warn` | `'[Chrome Web STT] Start failed:', e` |
| 547 | `console.log` | `\`[VAD Barge-in] User voice (${Math.round(rms * 100)}%) exceeded threshold (${Math.round(                 threshold * ...` |
| 576 | `console.warn` | `'[VAD Barge-in] Inisialisasi analyser audio:', err?.message \|\| err` |
| 584 | `console.log` | `'[Chrome Web STT] Audio permission & VAD analyser successfully active'` |
| 587 | `console.warn` | `'[Chrome Web STT] getUserMedia permission check:', err?.message \|\| err` |
| 598 | `console.log` | `'[TTS] Ignored on desktop because desktop audio is muted (Phone Only active)'` |
| 622 | `console.warn` | `'[TTS] Duplicate speak action suppressed:', textToSpeak` |
| 720 | `console.warn` | `'[Supertonic Audio] Playback error:', e` |
| 725 | `console.warn` | `'[Supertonic Audio] Autoplay blocked/interrupted:', err` |
| 756 | `console.warn` | `'[Supertonic Audio] Synthesize fetch error:', err` |
| 814 | `console.log` | `'[Chrome Web STT] Reaktif ganti bahasa STT ke:', targetStt` |

---
