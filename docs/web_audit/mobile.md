# Web Language Audit: `Mobile`

- **Category**: `web/src/mobile`
- **Total Files**: 10
- **Total Comments**: 132
- **Total Console Calls**: 13

## Table of Contents
- [src/mobile/AudioSyncManager.ts](#srcmobileaudiosyncmanagerts) (28 comments, 8 console logs)
- [src/mobile/AvatarViewer.tsx](#srcmobileavatarviewertsx) (40 comments, 2 console logs)
- [src/mobile/MobileApp.tsx](#srcmobilemobileapptsx) (37 comments, 3 console logs)
- [src/mobile/components/MobileAudioUnlockModal.tsx](#srcmobilecomponentsmobileaudiounlockmodaltsx) (0 comments, 0 console logs)
- [src/mobile/components/MobileFloatingMusic.tsx](#srcmobilecomponentsmobilefloatingmusictsx) (2 comments, 0 console logs)
- [src/mobile/components/MobileHeader.tsx](#srcmobilecomponentsmobileheadertsx) (5 comments, 0 console logs)
- [src/mobile/components/MobileInteractionDock.tsx](#srcmobilecomponentsmobileinteractiondocktsx) (5 comments, 0 console logs)
- [src/mobile/components/MobileLogsDrawer.tsx](#srcmobilecomponentsmobilelogsdrawertsx) (4 comments, 0 console logs)
- [src/mobile/components/MobileSettingsSheet.tsx](#srcmobilecomponentsmobilesettingssheettsx) (9 comments, 0 console logs)
- [src/mobile/components/MobileTopBar.tsx](#srcmobilecomponentsmobiletopbartsx) (2 comments, 0 console logs)

---

## `src/mobile/AudioSyncManager.ts`

- **Path**: `web/src/mobile/AudioSyncManager.ts`
- **Comments**: 28
- **Console Calls**: 8

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 20 | `/* */` | *  * MobileAudioSyncManager  * Manage audio unlock and direct streaming music playback on smartphone browser. |
| 48 | `//` | Use playsinline so iOS / Android routes directly to pure Media Speaker |
| 98 | `/* */` | *    * Unlock audio engine in mobile browser via user gesture (click / touch) |
| 109 | `//` | Play silent 10ms oscillator to release Chrome Android autoplay lock |
| 113 | `//` | nyaris senyap |
| 120 | `//` | Priming HTMLAudioElement for mobile browser |
| 158 | `//` | 1. Check Exact Match |
| 168 | `//` | 2. Check Substring |
| 176 | `//` | 3. Check Multi-word Token |
| 234 | `/* */` | *    * Temporarily lowers music volume while AI assistant speaks (audio ducking) |
| 245 | `/* */` | *    * Restores normal music volume after AI assistant finishes speaking |
| 278 | `//` | --------------------------------------------------------------------------- |
| 279 | `//` | Supertonic-3 Neural Voice Audio & Audio-Driven Lip Sync (A-I-U-E-O Visemes) |
| 280 | `//` | --------------------------------------------------------------------------- |
| 304 | `//` | Backend WAV normalized to 0.92 peak, optimal gain (1.05) applied for clarity without distortion |
| 332 | `//` | Clean up previous listeners |
| 381 | `//` | Reusable voiceAudio already attached |
| 385 | `//` | Reusable voiceAudio remains alive; simply stopping playback is sufficient |
| 400 | `//` | Lightweight sinusoidal fallback if Analyser not ready |
| 409 | `//` | 1. RMS Energy from time-domain waveform (128 is zero baseline) |
| 418 | `//` | Noise floor: if below 0.015, treat as silent / breath pause between words |
| 425 | `//` | 2. Formant Energy Bands from FFT Frequency Data: |
| 426 | `//` | (sampleRate ~44100-48000 Hz, fftSize 512 => 256 bins, width bin ~86-94 Hz) |
| 427 | `//` | Low band (~200 - 650 Hz): bins 2..7 (vokal U & O) |
| 428 | `//` | Mid band (~700 - 1500 Hz): bins 8..17 (vokal A) |
| 429 | `//` | Mid-High band (~1500 - 2400 Hz): bins 18..27 (vokal E) |
| 430 | `//` | High band (~2400 - 3800 Hz): bins 28..44 (vokal I) |
| 446 | `//` | AIUEO vocal viseme weights (anime-proportioned, anti-overextended mouth) |


### Console Logs (`console.*`)

| Line | Call | Arguments |
|:---:|:---:|:---|
| 73 | `console.warn` | `'[MobileAudio] Error audio stream:', e` |
| 130 | `console.log` | `'[MobileAudio] Audio gesture lock successfully unlocked'` |
| 133 | `console.warn` | `'[MobileAudio] Failed to unlock audio context:', err` |
| 145 | `console.warn` | `'[MobileAudio] Music library empty on server'` |
| 196 | `console.warn` | `'[MobileAudio] Autoplay prevented by mobile browser (touch/tap required):', err` |
| 201 | `console.error` | `'[MobileAudio] Failed to play music:', err` |
| 215 | `console.warn` | `'[MobileAudio] Resume prevented:', err` |
| 313 | `console.warn` | `'[MobileAudio] Web Audio Analyser init note:', e` |

---

## `src/mobile/AvatarViewer.tsx`

- **Path**: `web/src/mobile/AvatarViewer.tsx`
- **Comments**: 40
- **Console Calls**: 2

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 16 | `//` | Mapping dictionary of body joints from poses.json to Three.js VRM Humanoid Bone Standard |
| 37 | `//` | Finger Hand (VRM Humanoid Normalized Bones 1:1 Engine WGPU Desktop) |
| 69 | `//` | Quaternion tracking for smooth slerp transition between states (1:1 identical to desktop pose_system.rs) |
| 76 | `//` | Smoothing transisi expression face & bibir |
| 79 | `//` | Default camera position: Minimized / zoomed-out framing full-body character proportionally from head to toe |
| 157 | `//` | Helper resolusi node bone in Three-VRM |
| 176 | `//` | Snapshot current rotation of all bones & fingers as start point for smooth slerp transition |
| 187 | `//` | Timer transisi Slerp (durasi 0.45 detik identik with WGPU desktop) |
| 192 | `//` | S-Curve cosine smoothing: |
| 204 | `//` | Natural Breathing ritme santai |
| 241 | `//` | Multi-keyframes cycle Catmull-Rom C1 continuity |
| 331 | `//` | Default camera position facing character from the front |
| 357 | `//` | Attach OrbitControls for pinch-to-zoom and pan/rotate gestures on mobile |
| 362 | `//` | zoom in freely near face |
| 363 | `//` | zoom out freely to view character from a distance |
| 367 | `//` | Can pan up and down freely on mobile screen |
| 401 | `//` | Three.js VRM model faces forward by default (+Z camera direction) |
| 431 | `//` | 1. Calculate & apply rotation of all humanoid bones FIRST |
| 434 | `//` | 2. Facial expressions & vocal lip-sync 100% consistent with Three.js VRM 1.0 standard & desktop engine |
| 438 | `//` | Synchronized with desktop app.rs: when Thinking state, expression automatically overrides to thinking |
| 453 | `//` | 2a. Main Emotion Layer (100% identical to resolve_emotion_layers in expressions.rs) |
| 512 | `//` | 2b. Relaxed natural eye blinking (Identical to expressions.rs) |
| 516 | `//` | Durasi kedip ~240ms |
| 526 | `//` | 2c. Audio-Driven Lip Sync Vokal Vowel (A, I, U, E, O) when Speaking |
| 528 | `//` | When speaking, damp emotions that open jaw/mouth (surprised & happy) |
| 529 | `//` | to avoid exaggerated double-mouth opening with vocal Lip Sync |
| 539 | `//` | Active audio from phone speaker: apply precise AIUEO formant weights |
| 546 | `//` | Natural vocal articulation fallback animation when audio is played on remote desktop PC: |
| 558 | `//` | Smooth interpolation damping (~100ms) for expressions & mouth to prevent jitter |
| 559 | `//` | Notes: 'blink' is applied directly following a smooth sine curve without lag dampening, ensuring 100% perfect eye clo... |
| 577 | `//` | 3. THEN sync normalized bones to raw mesh model & spring bones |
| 590 | `//` | 1. Directly initialize 3D canvas without waiting for network fetch |
| 595 | `//` | 2. Load poses.json in parallel in background |
| 603 | `//` | 3. Check avatar type from config in parallel |
| 614 | `//` | Sync fullscreen status with browser events |
| 630 | `{/* */}` | Vertical Camera Control Toolbar aligned with Settings |
| 649 | `{/* */}` | Fullscreen button right above camera reset |
| 671 | `{/* */}` | Loading Overlay When VRM Model is Downloading |
| 681 | `{/* */}` | 3D WebGL Canvas Container (Full Screen) |
| 688 | `{/* */}` | 2D Sprite fallback if 2D mode or on fatal VRM load error |


### Console Logs (`console.*`)

| Line | Call | Arguments |
|:---:|:---:|:---|
| 391 | `console.warn` | `'VRM user data not found on gltf'` |
| 416 | `console.warn` | `'Failed to load VRM model on mobile:', err` |

---

## `src/mobile/MobileApp.tsx`

- **Path**: `web/src/mobile/MobileApp.tsx`
- **Comments**: 37
- **Console Calls**: 3

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 29 | `//` | Status Karakter Realtime |
| 33 | `//` | Audio & Mic Mode Status on Mobile (Remote Desktop vs Full Mobile) |
| 56 | `//` | Live Console Logs |
| 76 | `//` | ignore |
| 94 | `//` | Stop previous TTS voice if any (do not stop music!) |
| 103 | `//` | Lock speaking status immediately so mobile avatar responds without premature idle |
| 108 | `//` | Audio Ducking: lower music volume so AI voice is clearly audible |
| 111 | `//` | Notify backend that speaking mode is active |
| 198 | `//` | Handler Terpusat All Aksi (SSE + Polling Fallback) with Deduplikasi ID |
| 207 | `//` | 1. Voice respon Supertonic-3 Neural Audio (WAV stream) |
| 228 | `//` | 2. Streaming music routed to phone or controlled on PC |
| 295 | `//` | Release STT mic immediately so Android exits Voice Telephony mode back to Media Hi-Fi |
| 367 | `//` | Refresh page to cleanly reinitialize AudioContext and SSE |
| 396 | `//` | Sync active mode to backend (if Full HP, send mode_full_mobile; if Remote PC, send mode_remote_desktop) |
| 404 | `//` | Sync STT language dynamically from server config |
| 418 | `//` | Monitor virtual keyboard on mobile via VisualViewport so layout is not obscured |
| 434 | `//` | Auto-unlock audio HP on sentuhan/interaksi pertama user |
| 443 | `//` | Polling Terpadu Super Fast & Ringan (Murni Polling HTTP GET - 100% Kebal Blokir SSE) |
| 449 | `//` | 1. Actions polling (assistant:speak & music:play) - Every 750ms |
| 466 | `//` | 2. State polling (idle, speaking, listening, thinking, emotion) - Every 750ms |
| 473 | `//` | IF HP currently playing voice TTS (activeVoiceAudio != null \|\| isVoiceSpeakingOnMobile), |
| 474 | `//` | LOCK speaking status! Prevent premature idle overwrite until audio ends. |
| 493 | `//` | 3. Logs polling (if log drawer open, poll every ~2.2s; if closed poll every ~6s) |
| 500 | `//` | 4. Remote active heartbeat to host every ~10s |
| 506 | `//` | Session recovery helper: restore full mic & voice to desktop on close |
| 562 | `//` | Synchronize browser tab title on mobile (Default: "Ragasukma", When music: "▶ Title - Ragasukma") |
| 585 | `//` | Ensure mobile STT mic is released so phone speaker is not trapped in telephony filter |
| 598 | `//` | Stop assistant voice if speaking (user interruption), do not stop music! |
| 658 | `{/* */}` | 1. Modal Unlock Audio HP (1x Interaksi) |
| 664 | `{/* */}` | 2. Floating Top Controls (Ultra Clean: Left Floating Status Pill & Right Settings Button) |
| 671 | `{/* */}` | Sheet Companion Settings |
| 683 | `{/* */}` | 3. Karakter 3D VRM 100% Fullscreen Full Without Terpotong Padding |
| 692 | `{/* */}` | Flexible center spacer for floating bottom controls |
| 695 | `{/* */}` | 4. Bottom Controls: Floating Music + Circular Interaction Dock |
| 697 | `{/* */}` | Floating Music Player Mini Card (When Music Is Playing) |
| 729 | `{/* */}` | Floating Circular Interaction Dock (Circular Mic PTT Button & Message Input Toggle) |
| 744 | `{/* */}` | 5. Slide-Up Bottom Sheet Console Logs |


### Console Logs (`console.*`)

| Line | Call | Arguments |
|:---:|:---:|:---|
| 153 | `console.warn` | `'[Mobile Supertonic Audio] Playback error/blocked:', err` |
| 190 | `console.warn` | `'[Mobile Supertonic Audio] Synthesize fetch error:', err` |
| 334 | `console.warn` | `'Mic start error:', e` |

---

## `src/mobile/components/MobileAudioUnlockModal.tsx`

- **Path**: `web/src/mobile/components/MobileAudioUnlockModal.tsx`
- **Comments**: 0
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

_No comments found._


### Console Logs (`console.*`)

_No console logs found._

---

## `src/mobile/components/MobileFloatingMusic.tsx`

- **Path**: `web/src/mobile/components/MobileFloatingMusic.tsx`
- **Comments**: 2
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 16 | `{/* */}` | Track Artwork & Titles |
| 42 | `{/* */}` | Player Controls |


### Console Logs (`console.*`)

_No console logs found._

---

## `src/mobile/components/MobileHeader.tsx`

- **Path**: `web/src/mobile/components/MobileHeader.tsx`
- **Comments**: 5
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 47 | `{/* */}` | Left Side: Brand + Live Status Badge |
| 56 | `{/* */}` | Unified Live Status Badge in Header (Always visible & unobstructed) |
| 69 | `{/* */}` | Right Side: Audio Mode Switcher + Live Logs |
| 71 | `{/* */}` | Mode Switcher PC / HP |
| 101 | `{/* */}` | Logs Drawer Toggle Button |


### Console Logs (`console.*`)

_No console logs found._

---

## `src/mobile/components/MobileInteractionDock.tsx`

- **Path**: `web/src/mobile/components/MobileInteractionDock.tsx`
- **Comments**: 5
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 29 | `{/* */}` | 1. Status Aksi Terakhir (Clean Minimalist Pill) |
| 49 | `{/* */}` | 2. Floating Text Input Form (Appears neatly above buttons when message toggle is active) |
| 102 | `{/* */}` | 3. Primary Button Bar: Equal sizing, symmetrical, touch-friendly |
| 104 | `{/* */}` | Message / Keyboard Button (56x56 px, symmetric with Mic button) |
| 121 | `{/* */}` | PTT Mic Button (56x56 px, symmetric with Message button) |


### Console Logs (`console.*`)

_No console logs found._

---

## `src/mobile/components/MobileLogsDrawer.tsx`

- **Path**: `web/src/mobile/components/MobileLogsDrawer.tsx`
- **Comments**: 4
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 20 | `{/* */}` | Backdrop Lembut & Not Gelap Gulita |
| 26 | `{/* */}` | Slide-Up Bottom Sheet flush to bottom (no gap/margin) |
| 28 | `{/* */}` | Drawer Header |
| 69 | `{/* */}` | Scrollable Logs Area Without Excessive Bottom Padding |


### Console Logs (`console.*`)

_No console logs found._

---

## `src/mobile/components/MobileSettingsSheet.tsx`

- **Path**: `web/src/mobile/components/MobileSettingsSheet.tsx`
- **Comments**: 9
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 19 | `{/* */}` | Backdrop Transparan Smooth (Not Gelap Pekat) |
| 25 | `{/* */}` | Floating Bottom Sheet Sesuai Tema Cerah & Clean |
| 28 | `{/* */}` | Header Without Kotak-kotak (Icon Direct Berwarna) |
| 51 | `{/* */}` | Section: Mode Audio & Input |
| 57 | `{/* */}` | Remote PC Button |
| 79 | `{/* */}` | Phone Only Button |
| 103 | `{/* */}` | Section: Live Console Logs |
| 126 | `{/* */}` | Section: Audio Sync Reset |
| 140 | `{/* */}` | Section: Putuskan Session Remote |


### Console Logs (`console.*`)

_No console logs found._

---

## `src/mobile/components/MobileTopBar.tsx`

- **Path**: `web/src/mobile/components/MobileTopBar.tsx`
- **Comments**: 2
- **Console Calls**: 0

### Comments (`//`, `/* */`, `{/* */}`)

| Line | Type | Content |
|:---:|:---:|:---|
| 42 | `{/* */}` | Left Side: Clean Floating Status (Icon + Direct colored label without heavy box) |
| 56 | `{/* */}` | Right Side: Theme-colored Settings Icon Button (Without box frame) |


### Console Logs (`console.*`)

_No console logs found._

---
