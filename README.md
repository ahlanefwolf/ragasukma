<div align="center">

  <img src="icons/128x128@2x.png" alt="Ragasukma Logo" width="120" height="120" />

  # Ragasukma
  **Native Desktop AI Voice Assistant & 3D Avatar Companion**

  [![Release](https://img.shields.io/github/v/release/ahlanefwolf/ragasukma?style=flat-square&color=7BDFF2)](https://github.com/ahlanefwolf/ragasukma/releases)
  [![Platform](https://img.shields.io/badge/Platform-Linux_%7C_Windows_%7C_macOS-2F3B46?style=flat-square)](https://github.com/ahlanefwolf/ragasukma/releases)
  [![License](https://img.shields.io/badge/License-Proprietary-F2B5D4?style=flat-square)](https://github.com/ahlanefwolf/ragasukma)

  <p align="center">
    Multilingual desktop AI voice assistant with transparent 3D avatar overlay, zero-shot voice cloning, in-page browser automation, local music player, and local OpenAI-compatible LLM router.
  </p>

</div>

---

## Key Features & Highlights

### 1. Ultra-Fast Voice Engine & Zero-Shot Voice Cloning
- **Native Supertonic 3 Text-to-Speech (Pure ONNX)**:
  - Lightning-fast, on-device neural voice generation running directly on CPU without cloud dependency or Python runtimes.
  - 10 distinct studio voice presets (5 Male: `M1`–`M5`, 5 Female: `F1`–`F5`) with natural prosody and punctuation-aware breathing pauses.
  - Comprehensive support across **31 world languages**:

| Code | Language | Code | Language | Code | Language | Code | Language |
| :--- | :--- | :--- | :--- | :--- | :--- | :--- | :--- |
| `id` | Indonesian | `en` | English | `ja` | Japanese | `ko` | Korean |
| `ar` | Arabic | `bg` | Bulgarian | `cs` | Czech | `da` | Danish |
| `de` | German | `el` | Greek | `es` | Spanish | `et` | Estonian |
| `fi` | Finnish | `fr` | French | `hi` | Hindi | `hr` | Croatian |
| `hu` | Hungarian | `it` | Italian | `lt` | Lithuanian | `lv` | Latvian |
| `nl` | Dutch | `pl` | Polish | `pt` | Portuguese | `ro` | Romanian |
| `ru` | Russian | `sk` | Slovak | `sl` | Slovenian | `sv` | Swedish |
| `tr` | Turkish | `uk` | Ukrainian | `vi` | Vietnamese | | |
- **Zero-Shot Voice Cloning with OpenVoice V2**:
  - Upload any reference audio sample (`assets/refvoice.*`) from the web dashboard to instantly clone and morph the assistant's voice timbre into your target character.
  - Seamless automatic fallback to native Supertonic voices whenever no custom reference voice is provided.
  - Slerp-based voice morphing to blend multiple voice personalities seamlessly.
- **Natural Conversation & Barge-in Interruption**:
  - Instant Voice Activity Detection (VAD) that cuts off playback immediately the moment you start speaking, mimicking natural human conversation.
  - Real-time 60 FPS spectrum visualizer and microphone VU meter.

### 2. Transparent 3D Avatar & Physics
- **Desktop 3D VRM Companion**:
  - Lightweight transparent 3D window overlay with MToon cel-shaded rendering.
  - Full Verlet Spring Bone physics synchronization for realistic hair and clothing movement, gravity, and body colliders.
- **Emotion & Lip-Sync Synchronization**:
  - Real-time AIUEO lip-sync matched to speech audio playback.
  - Dynamic facial expressions and automatic eye-blinking reflecting 8 canonical emotions (*happy*, *shy*, *excited*, *surprised*, *thinking*, *sad*, *angry*, *neutral*).

### 3. In-Page Browser DOM Automation
- **Smart Web Navigation**:
  - Open, switch, scroll, or close browser tabs automatically.
  - Locate, read, and click buttons, links, and search bars on active websites based on real-time visual DOM layout.
  - Automatic web searches, typing, and form completion across websites.

### 4. Built-in Music Player & Retina Picture-in-Picture (PiP)
- **Local & YouTube Music Playback**:
  - Automated background streaming and audio caching with full playback controls (play, pause, skip, seek, loop).
- **Floating HiDPI PiP Player**:
  - Retina mini-player (680×210 px) that stays visible and runs smoothly even when your browser tab is minimized.

### 5. Desktop OS Controls & Productivity Utilities
- **Desktop Precision Automation**:
  - Click UI elements, open installed applications, manage system volume, take screenshots, or lock your computer via voice commands.
- **Integrated To-Do & Task Management**:
  - Voice-activated task tracking to add, view, and complete daily reminders.

### 6. Multi-Provider OAuth Routing (Antigravity & OpenAI Codex)
- **Universal Local API (`http://127.0.0.1:42128/v1`)**:
  - Drop-in standard OpenAI-compatible API endpoint allowing direct connection with Cursor, Zed, VS Code, Cline, and third-party AI tools.
- **Multi-Account OAuth Pooling (Antigravity & OpenAI Codex)**:
  - 1-click seamless browser OAuth integration for Google Antigravity (`ag/`) and OpenAI Codex (`cx/`).
  - Automated account rotation, token refresh, and quota failover for uninterrupted conversation flow.
  - Live quota tracking with clean `model_id` display, eliminating redundant labels.

---

## System Requirements

| Platform | Minimum Requirements |
| :--- | :--- |
| **Linux** | Kernel 5.4+, X11 / Wayland (Vulkan/OpenGL enabled), ALSA / PulseAudio / PipeWire |
| **Windows** | Windows 10 (64-bit) 1909+ / Windows 11, DirectX 12 / Vulkan |
| **macOS** | macOS 12 Monterey or later, Apple Silicon (M1/M2/M3/M4) with Metal |

---

## Quick Start Guide

### 1. Download & Run
Download installer binaries from [GitHub Releases](https://github.com/ahlanefwolf/ragasukma/releases):
- **Linux**: `.AppImage`
- **Windows**: `.exe`
- **macOS**: `.dmg`

Once launched, the floating 3D avatar appears on your desktop and the web control dashboard opens at `http://127.0.0.1:42128`.

### 2. Start Conversation
1. Navigate to the **Main Menu** on the dashboard.
2. Click **Start Conversation (Enable AI)**.
3. Speak directly into your microphone to chat, trigger browser actions, play music, or clone your favorite voice character!

---

## Copyright & License

Created by **Ahlan Saputra** ([@ahlanefwolf](https://github.com/ahlanefwolf)).  
Copyright © 2026 Ahlan Saputra. All rights reserved (*Proprietary*).
