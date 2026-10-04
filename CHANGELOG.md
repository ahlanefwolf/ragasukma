# 📋 Ragasukma Changelog

All notable updates, new features, and bug fixes for the **Ragasukma AI Voice Assistant** application are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/) and adheres to [Semantic Versioning](https://semver.org/).

---

## [v0.2.0] - 2026-09-20

### 🚀 Major Features & Enhancements
- **Supertonic-3 Native Rust TTS Runtime (Pure ONNX, Zero Python)**:
  - Integrated Supertonic-3 voice synthesis natively in Rust using `ort v2.0.0-rc.13` (ONNX Runtime) without Python sub-processes or runtimes.
  - Loaded 10 studio voice profiles (5 Male: M1..M5, 5 Female: F1..F5) with memory preloading for instant latency.
  - Provided 24kHz streaming WAV endpoints (`/api/tts/audio` and `/api/tts/synthesize`) piped directly to the web dashboard and smartphone clients.
  - Completely removed (100%) Google Chrome TTS (`window.speechSynthesis` & `SpeechSynthesisUtterance`), eliminating Linux PipeWire audio deadlocks and Chromium V8 garbage collection issues.
- **Chromium Native Web STT & Unified Language Synchronization**:
  - Maintained high-accuracy Google Chrome Web Speech API (`webkitSpeechRecognition`) on the client side with synchronized language options via a single `lang` property (`id-ID`, `en-US`, `ja-JP`, etc.).
  - Instant VAD Barge-in mechanism cutting WAV audio playback the moment the user speaks, complete with an integrated anti-echolalia filter.
- **Three-VRM Spring Bone Physics & 3D Facial Expressions**:
  - Full implementation of Verlet Spring Bone physics in Rust (`src/ui/spring_bone.rs`) matching 1:1 with the Three-VRM specification.
  - Evaluated dynamic sphere collider collisions, gravity, stiffness, and damping directly within the WGPU rendering pipeline.
  - Aligned canonical 8-emotion parsers (`*happy*`, `*shy*`, `*excited*`, `*surprised*`, `*thinking*`, `*sad*`, `*angry*`, `*neutral*`) with dynamic auto-blinking and AIUEO vocal lip-sync.
- **Native OS OCR & Modular Perception Engine**:
  - Completely removed the Python PaddleOCR module (`ocr_paddle.rs`).
  - Adopted native OS OCR engines: Apple Vision OCR on macOS, Windows Media OCR on Windows, and in-memory Tesseract TSV pipe streaming on Linux.
- **Antigravity Pool & Local OpenAI-Compatible API Router (`:42128/v1`)**:
  - Supported multi-account Google Multi-OAuth with automated pooling, thought signature caching, and intelligent quota failover upon hitting rate limits.
  - Universal standard OpenAI-compatible endpoints (`http://127.0.0.1:42128/v1/chat/completions` & `/v1/models`) compatible directly with Cursor, VS Code, Cline, OpenWebUI, and third-party tools.
- **Multiplatform Desktop Precision Click & Fallback Visual OCR Engine**:
  - Desktop interface element detection based on accessibility trees (UI Automation on Windows, AT-SPI D-Bus on Linux, System Events on macOS) now extracts button and text elements in a unified structure.
  - Automatic fallback to Visual OCR perception scan (`request_vision_scan_sync`) whenever accessibility trees lack absolute coordinates or targets reside in non-native UI canvases.
  - High-precision execution of `[CMD:desktop:click_text|...]` and `[CMD:desktop:click_button|...]` with actual monitor pixel coordinates and typo tolerance via fuzzy string similarity.
- **Defensive LLM Tool Call Guard & Multi-Language System Prompt**:
  - Prevented XML tool call hallucinations (`<dots_function_call>`, `<invoke>`, `<parameter>`) from Claude/Anthropic models in `prompt.rs`.
  - Defensive `RE_XML_TOOL_CALL` filter in `cleaner.rs` cleaning unexpected XML tags to keep TTS audio free from code syntax.
  - Standardized 100% of internal system prompts into universal English, dynamically compatible with 30+ world languages during user conversations.
  - Synchronized `assets/todo.md` structure in English (`## ⏳ In Progress / To-Do` & `## ✅ Done`) matching 1:1 with the Rust engine parser.
- **HiDPI Retina Picture-in-Picture (PiP) Widescreen Mini-Player**:
  - Horizontal widescreen floating mini-player (680 × 210 px) rendered at **2x HiDPI / Retina (1360 × 420 px)** resolution for crisp text and artwork.
  - Integrated **Dedicated Web Worker Timer Loop (30 FPS)** immune to Chromium background tab throttling.
- **Zero-Border-Radius Studio Design Overhaul & Premium Typography**:
  - Transformed the web dashboard UI with a *Zero Radius (Crisp Edges)* design language and delicate `#2F3B46` hairline borders.
  - Migrated all system fonts to the **IBM Plex Sans** and **IBM Plex Mono** families.
- **Comprehensive Code Quality, Verification & Audits (100% Clean Linters)**:
  - Quality verification standards documented in [`docs/quality.md`](docs/quality.md).
  - Modern flat ESLint 9+ configuration (`web/eslint.config.js`) and dead code analysis (`web/knip.json`).
  - 100% clean check: `cargo clippy && cargo fmt --check && (cd web && npx eslint . && npx tsc --noEmit && npx knip && npx jscpd .)`.
- **Complete Codebase Language Audit & English Standardization (Zero Hybrid Comments)**:
  - 100% standardization of all internal code comments (`//`, `///`, `/* */`, `{/* */}`) across backend Rust (`src/`) and frontend SolidJS/TypeScript (`web/src/`) to pure technical English.
  - Eradicated all mixed/loan words in logic comments (such as *artikulasi*, *dialihkan*, *menumpuk*, *melebihi*, *kumpulkan*, *tulang*, *ruas*, *pegas*, *normalisasi*, *kontinuitas*, etc.).
  - Standardized all frontend console messages (`console.log`, `console.warn`, `console.error`) and backend runtime logs to English.
  - Comprehensive line-by-line audit documentation structured in [`docs/language_audit/`](docs/language_audit/README.md) (91 backend Rust files, 921 comment lines, 4,461 string literals) and [`docs/web_audit/`](docs/web_audit/README.md) (63 frontend files, 507 comment lines, 71 console calls).

### 🐛 Bug Fixes & Performance Optimizations
- **Legacy Audio Cleanup**: Completely removed obsolete Python audio subprocesses, legacy browser speech synthesis workers, and duplicate assets.
- **Zero Race Condition Audio Pipeline**: Eliminated dual-backend audio decode dependencies in favor of Supertonic streaming WAV single-source-of-truth directly to web and mobile clients.
- **Decorative Clutter Removal**: Replaced thick 2px focus outlines and heavy shadows with sleek, responsive 1px highlight borders.
- **Minimal Resource Footprint**: PiP render loop and Web Worker consume < 0.5% CPU and ~3 MB RAM, featuring automated teardown (`terminate()`) upon closing.
- **Clean Diagnostics & Clippy Lints**: Zero errors and zero warnings across the Rust backend (`cargo clippy` clean) and TypeScript frontend.

---

## [v0.1.4] - 2026-09-16

### ✨ New Features & Enhancements
- **Native yt-dlp Music Engine & Offline Playback**:
  - Automated high-quality audio downloads (m4a/aac) via integrated `yt-dlp`.
  - Smart caching and local collection detection in `assets/music/` (bypasses re-downloading if files exist).
  - Removed browser companion extension dependencies for audio playback.
- **Embedded Web Music Player & Floating Bottom Bar**:
  - New **Music Player** tab on the web dashboard featuring track listings, artwork, duration, and delete actions.
  - Persistent **Floating Bottom Mini-Player** across all dashboard tabs (Play, Pause, Seek, Volume, Loop mode).
- **Strict Single-Tab Session Takeover**:
  - Instant session takeover mechanism when a new tab opens.
  - Obsolete tabs automatically disconnect quietly without intrusive modal dialogs.
- **Modern UI & Typographic Refinement**:
  - Adopted **Space Grotesk** typography and modern CSS color schemes.
  - Cleaned Cargo dependencies (`rodio` optimized exclusively for `symphonia-mp3`).

### 🐛 Bug Fixes & Optimizations
- **Zero Race Condition Audio Pipeline**: Coordinated microphone VAD and audio/TTS playback with atomic flags and instant buffer draining.
- **Safe Teardown**: Cleared all intervals, subscriptions, and abort controllers via SolidJS `onCleanup`.
- **Git Ignore Safeguard**: Isolated `assets/music/` and downloaded audio files completely from Git tracking.

---

## [v0.1.3] - 2026-09-16

### ✨ New Features & Enhancements
- **Modular Architecture Overhaul**:
  - Refactored modular Rust backend: cleanly segregated `src/server/` (`assets`, `handlers`, `state`) and `src/engine/` (`skills`, `system_actions`).
  - Streamlined integrations to focus on YouTube, YouTube Music, web navigation, and OS application launchers.
- **Precision 3D Camera Controls**:
  - **Right-Click Drag**: 3D orbit rotation.
  - **Ctrl + Right-Click Drag**: 3D pan / position shift.
  - **Scroll Wheel**: Zoom In / Zoom Out.
  - **Middle-Click**: Reset camera to default view.
- **Universal Browser Companion**:
  - 1-Click web companion installation script for direct AI voice control of YouTube & YouTube Music.
- **Atomic Single-Client Session & Instant Revoke**:
  - Dual-tab protection using Server-Sent Events (SSE) token broadcasts (<50ms instant lock).
  - Race-condition-free session takeover action.
- **In-Flight Lock & Throttled Pose Tuner**:
  - Real-time pose slider synchronization throttled (max 20 req/s) with trailing-edge queue, eliminating Firefox `NS_BINDING_ABORT` errors.

### 🐛 Bug Fixes & Optimizations
- Resolved race conditions between Live Pose Tuner and chat loop activation.
- Dynamic audio configuration updates on `AudioRecorder` without breaking background microphone threads.
- Added HTTP payload limits (100MB max) and WebSocket limits (2MB max) to prevent OOM errors.
- Optimized RAM allocations and audio buffer handling.

---

## [v0.1.2] - 2026-09-14

### ✨ New Features & Enhancements
- **Auto-Launch Default Browser**: Application automatically launches the web dashboard in the system default browser upon startup.
- **Isolated Dashboard Port (`42128`)**: Migrated local server to port `42128` to avoid conflicts with common dev services.
- **Integrated Update Checker**: *Check for Updates* button queries GitHub Releases API to verify versions and display changelogs.
- **Natural Pose Interpolation & Speed**: Synchronized WGPU bone interpolation and A ⇄ B pose cycles according to the `speed` parameter in `poses.json`.
- **Dynamic Emotion Expressions**: Synchronized facial blendshapes/morph targets directly from LLM response emotion tags (`[EMO:emotion]`).
- **Live Mic Audio Monitor**: Browser-side microphone audio monitoring with DSP noise gate filtering.

### 🐛 Bug Fixes & Optimizations
- Resolved dashboard status control button glitches by decoupling WGPU visual states from recorder loop states.
- Fixed `MeshPrimitiveData` and `VrmModelData` compilation in the VRM loader.
- Stabilized RAM consumption around ~200 MB.

---

## [v0.1.1] - 2026-09-10

### ✨ Initial Features
- Native WGPU 3D Cel-shaded VRM 0.x / 1.0 Overlay Engine.
- Continuous Microphone VAD (Voice Activity Detection) with Noise Reduction DSP & High-Pass Filtering.
- Integrated STT (Google Speech Recognition & Groq Cloud Whisper).
- LLM Streaming Client (OpenAI-compatible & Groq).
- Integrated TTS (Google Translate TTS & ElevenLabs Multilingual v2).
- Skill Command Registry System (`assets/skill-command.md`).
