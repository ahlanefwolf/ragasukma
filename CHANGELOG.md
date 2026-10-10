# 📋 Ragasukma Changelog

All notable updates, new features, and bug fixes for the **Ragasukma AI Voice Assistant** application are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and adheres to [Semantic Versioning](https://semver.org/).

---

## [v0.2.0] - 2026-10-10

### 🚀 Major Features & Enhancements
- **Supertonic 3 & OpenVoice V2 Native Pipeline (Zero-Shot Voice Cloning)**:
  - High-speed neural voice synthesis powered by Supertonic 3 in pure Rust with ONNX Runtime (`ort`).
  - Integrated OpenVoice V2 Tone Color Converter: effortlessly clone any reference audio sample (`assets/refvoice.*`) uploaded directly via the web dashboard.
  - Automatic fallback to native Supertonic voices when reference audio is empty.
  - Slerp-based geometric voice morphing and context-aware prosody calculation (dynamic tempo, punctuation pauses, and sentence density adaptation).
  - Multi-language coverage across 31 world languages (with automatic Hangul Jamo decomposition and full STT synchronization).
  - Automated background model downloader fetching model checkpoints seamlessly from HuggingFace.
- **Three-VRM Spring Bone Physics & 3D Visual Companion**:
  - Transparent desktop overlay window with real-time 3D VRM MToon cel-shaded rendering.
  - Full Verlet Spring Bone physics (gravity, stiffness, drag, and sphere colliders for hair and clothing).
  - Real-time vocal lip-sync and dynamic expressions across 8 canonical emotions (*happy*, *shy*, *excited*, *surprised*, *thinking*, *sad*, *angry*, *neutral*).
- **In-Page Browser DOM Automation & Precision Action Engine**:
  - Live DOM element analysis for hallucination-free website interaction.
  - Voice-driven web navigation: tab management, searching, form filling, text clicking, and button pressing.
- **Retina HiDPI Picture-in-Picture (PiP) Music Player**:
  - High-quality local and YouTube audio streaming and background caching.
  - 2x HiDPI floating mini-player (680×210 px) with persistent dedicated Web Worker timing for stutter-free playback.
- **Desktop Automation & Operating System Controls**:
  - Multiplatform precision desktop clicking with native accessibility tree integration and OCR fallback.
  - Voice-controlled application launcher, volume management, screenshots, and lock screen commands.
  - Integrated to-do list manager with voice task tracking.
- **Antigravity Pool & Local OpenAI API Router (`:42128/v1`)**:
  - Multi-account OAuth pooling with automatic failover.
  - Standard OpenAI-compatible local endpoints (`/v1/chat/completions`) for drop-in use with Cursor, VS Code, and Cline.
- **Memory & Resource Efficiency**:
  - Disabled memory pattern cache on ONNX models and enabled safe OS working-set trimming (`malloc_trim`).
  - Thread safety and resource-friendly CPU budgeting preventing OOM issues alongside heavy apps like OBS.

---

## [v0.1.4] - 2026-09-16

### ✨ New Features & Enhancements
- **Native yt-dlp Music Engine & Offline Playback**:
  - Automated high-quality audio downloads (m4a/aac) via integrated `yt-dlp`.
  - Smart caching and local collection detection in `assets/music/`.
- **Embedded Web Music Player & Floating Bottom Bar**:
  - Dedicated Music Player tab with track listings, artwork, and full playback controls.
- **Strict Single-Tab Session Takeover**:
  - Instant session takeover mechanism when a new browser tab opens.

---

## [v0.1.3] - 2026-09-16

### ✨ New Features & Enhancements
- **Modular Architecture Overhaul**: Clean segregation of server, assets, and engine modules.
- **Precision 3D Camera Controls**: Orbit rotation, panning, zoom, and middle-click reset.
- **Universal Browser Companion**: 1-Click web companion integration for YouTube voice control.

---

## [v0.1.2] - 2026-09-14

### ✨ New Features & Enhancements
- Auto-launch default browser dashboard upon application startup.
- Isolated default dashboard port `42128`.
- Integrated update checker querying GitHub Releases API.
- Dynamic facial emotion expressions synchronized with LLM responses.

---

## [v0.1.1] - 2026-09-10

### ✨ Initial Features
- Native WGPU 3D VRM Overlay Engine.
- Continuous Microphone VAD with DSP noise reduction.
- Integrated STT and streaming LLM client.
- Skill command registry system.
