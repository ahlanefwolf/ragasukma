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
  - **Custom Model Support & Hot-Reload**: Easily swap the default avatar with any `.vrm` 3D model (created via [VRoid Studio](https://vroid.com/studio), Blender, or downloaded from Booth / VRoid Hub) directly from the Web Dashboard or by replacing `assets/char.vrm`.
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

### 7. Autonomous Workspace & Software Engineering Agent (`src/agent`)
- **Self-Healing Code Modifications & Resilient Editor**:
  - Git-style 3-way merge LCS hunk decomposition, anchor-based fuzzy matching, and indentation-adaptive patching via `workspace_edit_file` and `workspace_write_file`.
  - Automatic rollback and safety reverts on failed modifications.
- **Strict Sequential Tool Execution (One Tool Per Turn)**:
  - Single-tool turn gating ensuring each action is grounded on actual prior observations, eliminating empty-argument failures and halving wasted turn latency.
- **KV-Cache Friendly Token Compaction & Outline Discovery**:
  - Automatic polyglot symbol outline generation for large source files (>250 lines), slicing reads with precision line boundaries (`workspace_read_file`).
  - Tail-compacted historical tool call arguments and head-tail output truncation to preserve prefix KV-cache across upstream providers.
- **Evidence Caching & Adaptive Verification Loop**:
  - Local hash/mtime evidence cache tracking source revisions without heavy vector databases.
  - Autonomous *Discovery -> Action -> Verification* lifecycle running workspace test commands (`cargo test`, `npm test`, etc.) to validate code changes before closing goals.
- **Model Context Protocol (MCP) Integration**:
  - First-class support for project-scoped stdio, SSE, and HTTP external MCP servers and tools.

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

### 3. Customizing Your 3D Avatar (.vrm)
You can replace the avatar with any 3D character model you like at any time:
- **Via Web Dashboard**: Open `http://127.0.0.1:42128` -> Go to **Settings** tab -> **Avatar & 3D Model** -> Click **Select VRM File (.vrm)**. The new avatar will be uploaded and hot-reloaded immediately on your desktop without restarting the app.
- **Via Local File**: Replace `assets/char.vrm` (or place your `.vrm` in your application data directory).
- **Create Your Own**: You can design and export your own free custom avatar in minutes using [VRoid Studio](https://vroid.com/studio).

---

## Acknowledgements & Credits

Special thanks to the following open-source projects, research teams, and creators:
- **[Supertonic 3](https://supertonic3.github.io/)**: Ultra-fast on-device neural Text-to-Speech architecture.
- **[OpenVoice (MyShell.ai)](https://github.com/myshell-ai/openvoice)**: Versatile instant zero-shot voice cloning and tone color conversion.
- **[9router](https://github.com/decolua/9router)**: Architectural inspiration and reference for multi-provider LLM routing.
- **[oh-my-pi](https://github.com/can1357/oh-my-pi)**: Reference implementation for the AI harness.
- **[VRoid Hub](https://hub.vroid.com/) & [miHoYo / HoYoverse](https://www.hoyoverse.com/)**: Default preview 3D model ("Navia" character design © miHoYo). Provided as a sample avatar for demonstration and testing purposes; users can freely replace it with their own custom VRM models.

---

## Copyright & License

Created by **Ahlan Saputra** ([@ahlanefwolf](https://github.com/ahlanefwolf)).  
Copyright © 2026 Ahlan Saputra. All rights reserved (*Proprietary*).
