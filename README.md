<div align="center">

  <img src="icons/128x128@2x.png" alt="Ragasukma Logo" width="120" height="120" />

  # Ragasukma
  **Native Desktop AI Voice Assistant & 3D Avatar Companion**

  [![Release](https://img.shields.io/github/v/release/ahlanefwolf/ragasukma?style=flat-square&color=7BDFF2)](https://github.com/ahlanefwolf/ragasukma/releases)
  [![Platform](https://img.shields.io/badge/Platform-Linux_%7C_Windows_%7C_macOS-2F3B46?style=flat-square)](https://github.com/ahlanefwolf/ragasukma/releases)
  [![License](https://img.shields.io/badge/License-Proprietary-F2B5D4?style=flat-square)](https://github.com/ahlanefwolf/ragasukma)

  <p align="center">
    Multilingual desktop AI voice assistant with transparent 3D/2D avatar overlay, in-page browser automation, local music player, and local OpenAI-compatible LLM router.
  </p>

</div>

---

## Features & System Capabilities

### 1. Voice Interaction & 3D Visuals
- **Transparent 3D VRM Overlay & Spring Bone Physics**:
  - Native WGPU rendering engine with 1:1 MToon shader and full **Three-VRM Spring Bone Verlet Physics** synchronization (avatar hair & clothing with dynamic colliders, drag force, and gravity).
  - Facial expression blendshape/morph target synchronization across 8 canonical emotions (*happy*, *shy*, *excited*, *surprised*, *thinking*, *sad*, *angry*, *neutral*) with dynamic auto-blinking.
- **Unified Audio Engine (Native Supertonic-3 TTS & Chromium STT)**:
  - **Supertonic-3 Native Rust TTS Runtime (Pure ONNX, Zero Python)**: High-speed voice inference via `ort v2.0.0-rc.13` directly embedded within the Rust binary, offering 10 studio voice profiles (5 Male: M1..M5, 5 Female: F1..F5) with memory preloading, numeric preservation, and streaming 24kHz WAV to web dashboards and mobile clients. 100% free of browser `window.speechSynthesis` limitations.
  - **Chromium Native Web STT (`webkitSpeechRecognition`)**: High-accuracy speech recognition with unified language synchronization (`id-ID`, `en-US`, `ja-JP`, `ar-SA`, etc.) via a single `lang` property.
  - **Zero Mic Locks & VAD Barge-in**: Instant audio ducking and barge-in interruption cutting TTS playback the moment the user resumes speaking.
  - **60 FPS Spectrum Visualizer & VU Meter**: Processed in real-time via the browser Web Audio API.
- **Strict Single-Tab Ownership & Clean Teardown**:
  - Total elimination of duplicate audio conflicts: obsolete tabs immediately cut audio, cancel pending requests, and close SSE streams (`forceCloseSSE`).

### 2. Browser DOM Automation & Click Engine
- **Active Tab DOM Ground-of-Truth**: Real-time visual analysis of interactive elements on the active tab for hallucination-free command execution.
- **In-Page Navigation**:
  - `browser:open` — Open or switch to a target website tab.
  - `browser:search` — Search on a target site or search in-page content.
  - `browser:type` & `browser:enter` — Type text into active forms/inputs and execute Enter.
  - `browser:click_text` & `browser:click_button` — Locate and click text, links, or buttons.
  - `browser:switch` & `browser:close` — Active tab management in the browser.

### 3. Music Player & HiDPI PiP
- **Automated YouTube Download & Playback**: Automatic background downloading and local playback via `yt-dlp` (`media:play`, `media:pause`, `media:resume`, `media:next`, `media:prev`, `media:stop`).
- **Retina Picture-in-Picture (PiP)**: Widescreen floating mini-player (680x210 @ 2x HiDPI) powered by a Dedicated Web Worker timer (30 FPS) that continues smooth playback when the browser tab is minimized.

### 4. OS System Control, Desktop Precision & Utilities
- **Multiplatform Desktop Precision Click**: Click buttons or text on screen monitors (`desktop:click_text`, `desktop:click_button`, `desktop:click_at`) with native OS accessibility tree integration (UI Automation on Windows, AT-SPI on Linux, System Events on macOS) and automatic visual OCR perception fallbacks for non-native UI canvases.
- **Application Launcher**: Automatically detect and launch installed desktop applications (`app:open`).
- **Audio & OS Controls**: System volume control (`os:volume_up`, `os:volume_down`, `os:mute`), screenshots (`os:screenshot`), screen locking (`os:lock`), calculator, terminal, and file manager shortcuts.
- **Note Management**: Integrated to-do list tracking (`todo:add`, `todo:done`).

### 5. Antigravity Pool & Local OpenAI Router
- **Multi-Account OAuth Pooling**: Dual Google OAuth account management with automated quota failover.
- **OpenAI-Compatible Local Endpoint (`http://127.0.0.1:42128/v1`)**: Drop-in compatibility with Cursor, VS Code, Cline, and third-party developer tools.

---

## System Requirements

| Platform | Minimum Requirements |
| :--- | :--- |
| **Linux** | Kernel 5.4+, X11 / Wayland (Vulkan/OpenGL enabled), `libasound2` (ALSA/PulseAudio/PipeWire) |
| **Windows** | Windows 10 (64-bit) 1909+ / Windows 11, DirectX 12 / Vulkan support |
| **macOS** | macOS 12 Monterey or later, Apple Silicon (M1/M2/M3/M4) with Metal Engine |

---

## Quick Start Guide

### 1. Download & Run
Download installer binaries from [GitHub Releases](https://github.com/ahlanefwolf/ragasukma/releases):
- **Linux**: `.AppImage`
- **Windows**: `.exe`
- **macOS**: `.dmg`

Once launched, the floating 3D avatar appears on your desktop and the web control dashboard opens at `http://127.0.0.1:42128`.

### 2. Start Conversation
1. Navigate to the **Main Menu** tab on the dashboard.
2. Click **Start Conversation (Enable AI)**.
3. Speak directly into your microphone to converse, play music, or execute system automation commands.

---

## Code Quality, Verification & Language Audits

The full codebase pipeline is verified via a single command:
```bash
cargo clippy && cargo fmt --check && (cd web && npx eslint . && npx tsc --noEmit && npx knip && npx jscpd .)
```
For comprehensive quality guidelines and audit reports, refer to:
- [Code Quality & Linting Documentation](docs/quality.md)
- [Backend Rust Language & String Audit (`docs/language_audit/`)](docs/language_audit/README.md)
- [Frontend Web & Console Audit (`docs/web_audit/`)](docs/web_audit/README.md)

---

## Copyright & License

Created by **Ahlan Saputra** ([@ahlanefwolf](https://github.com/ahlanefwolf)).  
Copyright © 2026 Ahlan Saputra. All rights reserved (*Proprietary*).
