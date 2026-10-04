# Language Audit: `ui` Module

- **Directory**: `src/ui`
- **Total Files**: 12
- **Total Comments**: 136
- **Total String Literals**: 506

## Table of Contents
- [ui/app.rs](#uiapprs) (32 comments, 34 strings)
- [ui/mod.rs](#uimodrs) (2 comments, 8 strings)
- [ui/pose_system.rs](#uipose-systemrs) (16 comments, 20 strings)
- [ui/renderer/camera.rs](#uirenderercamerars) (6 comments, 0 strings)
- [ui/renderer/expressions.rs](#uirendererexpressionsrs) (25 comments, 164 strings)
- [ui/renderer/mod.rs](#uirenderermodrs) (12 comments, 12 strings)
- [ui/renderer/overlay.rs](#uirendereroverlayrs) (0 comments, 8 strings)
- [ui/renderer/pipeline_2d.rs](#uirendererpipeline-2drs) (2 comments, 33 strings)
- [ui/renderer/pipeline_3d.rs](#uirendererpipeline-3drs) (6 comments, 23 strings)
- [ui/renderer/types.rs](#uirenderertypesrs) (3 comments, 0 strings)
- [ui/spring_bone.rs](#uispring-boners) (14 comments, 24 strings)
- [ui/vrm_loader.rs](#uivrm-loaderrs) (18 comments, 180 strings)

---

## `ui/app.rs`

- **Path**: `src/ui/app.rs`
- **Comments**: 32
- **Strings**: 34

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 60 | `//` | Load icon window from icons/icon.png |
| 72 | `//` | Compact portrait window size (270x480) in bottom-right corner of desktop |
| 100 | `//` | Asynchronously lazy load via background thread if avatar_type is VRM |
| 201 | `//` | Pause 200ms to allow browser to receive shutdown SSE event and clean up resources/player |
| 236 | `//` | 1. Check Resize Border on all 4 sides & 4 window corners |
| 264 | `//` | Drag window borders/corners: Resize window natively via OS |
| 269 | `//` | 2. Check click on top Header Bar (y <= 28.0) |
| 271 | `//` | Close button () |
| 276 | `//` | Pause 200ms to allow browser to receive shutdown SSE event and clean up resources/player |
| 281 | `//` | Maximize / Fullscreen button () |
| 287 | `//` | Minimize button (—) |
| 293 | `//` | Click on other header bar areas -> Drag window |
| 299 | `//` | Left-click and drag on avatar body: Move avatar window position natively via OS |
| 310 | `//` | Released |
| 314 | `//` | Middle-Click: Reset avatar camera rotation & zoom |
| 326 | `//` | Update Icon Kursor sesuai position tepi border |
| 361 | `//` | Ctrl + Right-Click Drag: Pan / Shift 3D Character Position |
| 364 | `//` | Right-Click Drag: Orbit rotation 3D camera |
| 380 | `//` | Scroll Mouse: POV Camera Zoom In / Out smooth |
| 392 | `//` | Check background loader channel for asynchronous loading status |
| 398 | `//` | Open standalone dashboard only once after the 3D avatar/model is first loaded to GPU! |
| 405 | `//` | If mode 2D (without perlu tunggu VRM loader), direct open dashboard sekali when start start |
| 410 | `//` | Sync state & live poses reload from Web HTTP Dashboard |
| 419 | `//` | In mode edit Tuner: snap seketika without pause/slowmotion slerp |
| 425 | `//` | Freeze cyclic animation and directly apply live bone rotation from tuner |
| 429 | `//` | Use preview poses in-memory without overwrite poses.json in disk |
| 432 | `//` | Reload 100% from file poses.json resmi in disk (Source of Truth) |
| 542 | `//` | Watchdog: if berada in state listening > 4.5s or session assistant not active, paksa return to Idle |
| 565 | `//` | Emotion Hierarchy (updated BEFORE update_camera_and_bones so it is applied on current frame): |
| 566 | `//` | 1. If currently in Pose Tuner (Tuning edit mode or Preview loop), use emotion realtime which selected user in UI |
| 567 | `//` | 2. While Thinking, display thinking expression |
| 568 | `//` | 3. While Speaking, Idle, or Listening, retain last emotion (e.g. remain blushing/happy/pouting while idle)! |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 62 | `../../icons/icon.png` |
| 79 | `Ragasukma Avatar Overlay (Native)` |
| 101 | `vrm` |
| 107 | `char.vrm` |
| 108 | `char.glb` |
| 109 | `assets/char.vrm` |
| 110 | `assets/char.glb` |
| 115 | `[Async Asset Loader] Loading avatar model in background: {:?}` |
| 135 | `neutral` |
| 198 | ` Closing Ragasukma (Window Close)...` |
| 200 | `quit` |
| 273 | ` Closing Ragasukma (Header Close)...` |
| 275 | `quit` |
| 401 | `http://127.0.0.1:42128` |
| 407 | `http://127.0.0.1:42128` |
| 433 | `poses.json` |
| 443 | `char.vrm` |
| 444 | `assets/char.vrm` |
| 445 | `../assets/char.vrm` |
| 446 | `assets/char.glb` |
| 447 | `../assets/char.glb` |
| 452 | `[Async Asset Loader] Reloading avatar model in background: {:?}` |
| 492 | `2d` |
| 494 | `vrm` |
| 501 | `char.vrm` |
| 502 | `assets/char.vrm` |
| 503 | `../assets/char.vrm` |
| 504 | `assets/char.glb` |
| 505 | `../assets/char.glb` |
| 510 | `[Async Asset Loader] Loading avatar model in background: {:?}` |
| 553 | `Standby` |
| 554 | `neutral` |
| 574 | `thinking` |
| 586 | `Render error: {:?}` |

---

## `ui/mod.rs`

- **Path**: `src/ui/mod.rs`
- **Comments**: 2
- **Strings**: 8

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 19 | `//` | Coba bangun event loop standar (Wayland if tersedia), if Wayland NoCompositor, fallback automatically to X11 |
| 45 | `//` | ~60 FPS |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 10 | `linux` |
| 14 | ` Starting Ragasukma 3D Native Overlay (Ultra Lightweight WGPU)...` |
| 17 | `linux` |
| 23 | `[Wayland] Not detected or connection refused, falling back to X11 backend...` |
| 27 | `Failed to create winit event loop (Wayland & X11 fallback)` |
| 29 | `linux` |
| 33 | `Failed to create winit event loop` |
| 40 | ` Failed to initialize Native App: {:?}` |

---

## `ui/pose_system.rs`

- **Path**: `src/ui/pose_system.rs`
- **Comments**: 16
- **Strings**: 20

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 57 | `//` | 0.0 -> 1.0 (smooth interpolation on state change) |
| 58 | `//` | Snapshot joint rotations immediately before state change |
| 60 | `//` | Current state active elapsed time (reset to 0 on state change, identical to Three.js) |
| 105 | `///` | Update pose 1:1 persis Three.js / VRM standard with multi-pose dynamic cycle & natural breathing |
| 110 | `//` | Check state transition changes (e.g. Idle -> Listening -> Thinking -> Speaking) |
| 112 | `//` | Snapshot joint rotations as starting point for smooth slerp transition |
| 119 | `//` | Run timer interpolasi transisi antar state (~0.45 detik smooth slerp) |
| 138 | `//` | Engine Natural Breathing: Ritme nafas hidup in chest, punggung, pundak, and mikro-gerak head |
| 139 | `//` | ~13 tarikan nafas per menit (ritme nafas santai) |
| 146 | `//` | No registered poses |
| 148 | `//` | Only 1 pose: diam in pose tersebut with efek nafas alami |
| 169 | `//` | Multi-pose (2 pose or more): Catmull-Rom Spline interpolation (C1 continuous) |
| 170 | `//` | Ensures continuous motion speed across keyframes without abrupt pauses (ease-in/out stall). |
| 193 | `//` | Catmull-Rom spline evaluator: q(t) = 0.5 * ((2*P1) + (-P0 + P2)*t + (2*P0 - 5*P1 + 4*P2 - P3)*t^2 + (-P0 + 3*P1 - 3*P... |
| 209 | `//` | Handle hip yaw angular phase unwrapping so rotation does not reverse on wrap-around |
| 254 | `//` | Apply pose to joints: If transitioning between states, perform S-Curve Slerp from previous pose to new pose |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 9 | `snake_case` |
| 26 | `default_speed` |
| 28 | `Vec::is_empty` |
| 79 | `poses.json` |
| 95 | `[PoseManager] Successfully loaded {} states from {:?}` |
| 127 | `idle` |
| 128 | `listening` |
| 129 | `speaking` |
| 130 | `thinking` |
| 151 | `chest` |
| 153 | `spine` |
| 155 | `leftShoulder` |
| 155 | `rightShoulder` |
| 157 | `head` |
| 210 | `hips` |
| 228 | `chest` |
| 230 | `spine` |
| 232 | `leftShoulder` |
| 232 | `rightShoulder` |
| 234 | `head` |

---

## `ui/renderer/camera.rs`

- **Path**: `src/ui/renderer/camera.rs`
- **Comments**: 6
- **Strings**: 0

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 93 | `//` | 1. The actual center point and height of the model |
| 103 | `//` | 2. Standard RH Orbit Camera Transformation (Yaw & Pitch) |
| 129 | `//` | Bone Matrix Update (Three.js Pose Hierarchy) - 0 Allocation |
| 147 | `//` | VRM SpringBone Verlet Physics (Hair, Bangs, Dynamic swaying skirt) |
| 185 | `//` | Three-VRM Normalized Humanoid Rig standard (@pixiv/three-vrm) |
| 186 | `//` | boneNode.quaternion = inv(P) * rigBoneNode.quaternion * P * boneRotation |


### String Literals (`"..."`)

_No string literals found._

---

## `ui/renderer/expressions.rs`

- **Path**: `src/ui/renderer/expressions.rs`
- **Comments**: 25
- **Strings**: 164

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 6 | `///` | VRM expression finder based on name/alias without conflicts |
| 16 | `//` | 1. Exact match case-insensitive |
| 23 | `//` | 2. Substring match if name contains prefixes like "fcl_all_happy", "vrm_joy", "preset_smile" |
| 34 | `///` | Search expression pertama which cocok from list prioritas alias |
| 47 | `///` | Structure representing active expression weights on the current frame |
| 53 | `///` | Calculates blend shape weights for 8 canonical emotions (VRM 1.0 Three.js standard). |
| 97 | `//` | Proportional weighting so surprised expressions remain clear without distorting the jaw shape |
| 126 | `//` | Netral: 100% base mesh without morph |
| 129 | `//` | Fallback if name emotion direct cocok with name morph |
| 158 | `///` | Ekstrak emotion from tag eksplisit standar AI (*happy*, *shy*, *excited*, *surprised*, *thinking*, *sad*, *angry*, *n... |
| 169 | `///` | Extracts emotion with a default fallback to "neutral" if undetected |
| 174 | `///` | Realtime Facial Expressions, Natural Eye Blinking & Procedural Lip Sync |
| 188 | `//` | 1. Calculate layer emotion utama |
| 193 | `//` | When berbicara (Speaking), redam emotion which opening jaw/mouth (surprised) |
| 194 | `//` | Prevents double mouth-opening effects alongside vocal Lip Sync |
| 213 | `//` | 2. Natural Eye Blinking (Natural relaxed eye blinks) |
| 221 | `//` | Durasi kedip ~240ms |
| 237 | `//` | 3. Audio-Driven Lip Sync Vokal Vowel (A, I, U, E, O) when Speaking |
| 242 | `//` | Voice active from speaker hardware: use analisis frekuensi formant audio nyata |
| 259 | `//` | Natural vocal articulation fallback animation when audio is redirected to remote web client: |
| 282 | `//` | 4. Update Position Vertex Morph Targets on Mesh Face in GPU Buffer |
| 291 | `//` | Always reset to position dasar (base_vertices) |
| 296 | `//` | Accumulate total weights per target_index so morphs do not stack beyond safe limits (anti-jaw distortion) |
| 316 | `//` | Apply morph target with aesthetic limit clamping (max 0.85) to avoid over-stretching |
| 367 | `//` | Fallback or without tag |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 62 | `happy` |
| 63 | `happy` |
| 67 | `shy` |
| 68 | `happy` |
| 74 | `blink` |
| 81 | `excited` |
| 82 | `happy` |
| 88 | `surprised` |
| 95 | `surprised` |
| 96 | `surprised` |
| 101 | `thinking` |
| 102 | `ou` |
| 102 | `u` |
| 108 | `sad` |
| 115 | `sad` |
| 116 | `sad` |
| 120 | `angry` |
| 121 | `angry` |
| 125 | `neutral` |
| 140 | `*happy*` |
| 140 | `happy` |
| 141 | `[happy]` |
| 141 | `happy` |
| 142 | `*shy*` |
| 142 | `shy` |
| 143 | `[shy]` |
| 143 | `shy` |
| 144 | `*excited*` |
| 144 | `excited` |
| 145 | `[excited]` |
| 145 | `excited` |
| 146 | `*surprised*` |
| 146 | `surprised` |
| 147 | `[surprised]` |
| 147 | `surprised` |
| 148 | `*thinking*` |
| 148 | `thinking` |
| 149 | `[thinking]` |
| 149 | `thinking` |
| 150 | `*sad*` |
| 150 | `sad` |
| 151 | `[sad]` |
| 151 | `sad` |
| 152 | `*angry*` |
| 152 | `angry` |
| 153 | `[angry]` |
| 153 | `angry` |
| 154 | `*neutral*` |
| 154 | `neutral` |
| 155 | `[neutral]` |
| 155 | `neutral` |
| 171 | `neutral` |
| 197 | `surprised` |
| 199 | `happy` |
| 199 | `joy` |
| 229 | `blink` |
| 229 | `eyeblink` |
| 229 | `blink_both` |
| 229 | `fcl_eye_close` |
| 244 | `aa` |
| 244 | `a` |
| 244 | `viseme_a` |
| 244 | `preset_a` |
| 244 | `fcl_mth_a` |
| 245 | `ih` |
| 245 | `i` |
| 245 | `viseme_i` |
| 245 | `preset_i` |
| 245 | `fcl_mth_i` |
| 246 | `ou` |
| 246 | `u` |
| 246 | `viseme_u` |
| 246 | `preset_u` |
| 246 | `fcl_mth_u` |
| 247 | `ee` |
| 247 | `e` |
| 247 | `viseme_e` |
| 247 | `preset_e` |
| 247 | `fcl_mth_e` |
| 248 | `oh` |
| 248 | `o` |
| 248 | `viseme_o` |
| 248 | `preset_o` |
| 248 | `fcl_mth_o` |
| 260 | `aa` |
| 260 | `ih` |
| 260 | `ou` |
| 260 | `ee` |
| 260 | `oh` |
| 265 | `aa` |
| 265 | `aa` |
| 265 | `a` |
| 265 | `viseme_a` |
| 265 | `preset_a` |
| 265 | `fcl_mth_a` |
| 266 | `ih` |
| 266 | `ih` |
| 266 | `i` |
| 266 | `viseme_i` |
| 266 | `preset_i` |
| 266 | `fcl_mth_i` |
| 267 | `ou` |
| 267 | `ou` |
| 267 | `u` |
| 267 | `viseme_u` |
| 267 | `preset_u` |
| 267 | `fcl_mth_u` |
| 268 | `ee` |
| 268 | `ee` |
| 268 | `e` |
| 268 | `viseme_e` |
| 268 | `preset_e` |
| 268 | `fcl_mth_e` |
| 269 | `oh` |
| 269 | `oh` |
| 269 | `o` |
| 269 | `viseme_o` |
| 269 | `preset_o` |
| 269 | `fcl_mth_o` |
| 348 | `*happy* hello!` |
| 348 | `happy` |
| 349 | `[happy] hello!` |
| 349 | `happy` |
| 350 | `*shy* oh don't say that...` |
| 350 | `shy` |
| 351 | `*excited* let's go!` |
| 351 | `excited` |
| 352 | `*surprised* really?` |
| 352 | `surprised` |
| 354 | `*thinking* hmm give me a second...` |
| 355 | `thinking` |
| 357 | `*sad* sorry about that...` |
| 357 | `sad` |
| 358 | `*angry* stop that!` |
| 358 | `angry` |
| 360 | `*neutral* here is the info.` |
| 361 | `neutral` |
| 363 | `*shy* hello...` |
| 363 | `shy` |
| 364 | `*happy* finally done!` |
| 364 | `happy` |
| 365 | `[excited] Wow!` |
| 365 | `excited` |
| 368 | `hello there, how are you?` |
| 369 | `hello` |
| 369 | `neutral` |
| 376 | `happy` |
| 378 | `happy` |
| 383 | `blink` |
| 385 | `blink` |
| 390 | `blinkleft` |
| 392 | `blinkleft` |
| 397 | `surprised` |
| 399 | `surprised` |
| 404 | `sad` |
| 406 | `sad` |
| 411 | `angry` |
| 413 | `angry` |
| 418 | `ou` |
| 420 | `ou` |
| 425 | `happy` |
| 429 | `shy` |
| 432 | `excited` |
| 435 | `neutral` |

---

## `ui/renderer/mod.rs`

- **Path**: `src/ui/renderer/mod.rs`
- **Comments**: 12
- **Strings**: 12

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 46 | `//` | Referensi bounding box model |
| 54 | `//` | Kontrol POV & Position Camera in Screen Fullscreen |
| 61 | `//` | Mode Avatar & Visual State |
| 76 | `//` | CSD Header & Controls |
| 166 | `//` | 1. Pipeline 3D VRM MToon |
| 169 | `//` | 2. Overlay Pipeline (Border & Header Buttons) |
| 172 | `//` | 3. Pipeline 2D Sprite Quad |
| 326 | `//` | Free old VRAM / GTT allocations immediately before creating new texture |
| 345 | `//` | Free GPU buffers (vertex & index) from previous VRM model if any |
| 452 | `//` | Background 100% transparan |
| 495 | `//` | For 2D Avatar when Speaking: create animasi mouth flapping (open-close mouth) |
| 496 | `//` | Synchronized with AIUEO vocal waves matching the 3D VRM (~5.5 Hz) |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 108 | `Failed to find a suitable GPU adapter` |
| 111 | `[WGPU Adapter] Using GPU: {}` |
| 120 | `Ragasukma Device` |
| 149 | `[WGPU Surface] Formats: {:?}, Alpha Modes: {:?}` |
| 152 | `[WGPU Surface] Active Alpha Mode: {:?}` |
| 210 | `vrm` |
| 212 | `neutral` |
| 274 | `MSAA Multisample Texture` |
| 296 | `Depth Texture` |
| 442 | `Render Encoder` |
| 447 | `Render Pass` |
| 492 | `2d` |

---

## `ui/renderer/overlay.rs`

- **Path**: `src/ui/renderer/overlay.rs`
- **Comments**: 0
- **Strings**: 8

### Comments (`//`, `///`, `/* */`)

_No comments found._


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 13 | `../overlay.wgsl` |
| 26 | `overlay_bind_group_layout` |
| 30 | `Overlay Uniform Buffer` |
| 42 | `overlay_bind_group` |
| 46 | `Overlay Pipeline Layout` |
| 52 | `Overlay Pipeline` |
| 56 | `vs_overlay` |
| 61 | `fs_overlay` |

---

## `ui/renderer/pipeline_2d.rs`

- **Path**: `src/ui/renderer/pipeline_2d.rs`
- **Comments**: 2
- **Strings**: 33

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 156 | `//` | 1. Coba baca from file candidates in disk |
| 175 | `//` | 2. Fallback: Embed icon direct from rust-embed or byte build |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 23 | `avatar2d.wgsl` |
| 24 | `../avatar2d.wgsl` |
| 38 | `avatar_2d_uniform_layout` |
| 42 | `Avatar 2D Uniform Buffer` |
| 54 | `avatar_2d_uniform_bind_group` |
| 76 | `avatar_2d_bind_group_layout` |
| 90 | `Avatar 2D Pipeline Layout` |
| 96 | `Avatar 2D Pipeline` |
| 100 | `vs_main` |
| 105 | `fs_main` |
| 158 | `icon.png` |
| 159 | `icons/icon.png` |
| 160 | `icons/128x128.png` |
| 161 | `assets/icon.png` |
| 177 | `icon.png` |
| 192 | `Loading_Icon_Texture` |
| 229 | `loading_icon_bind_group` |
| 294 | `char-avatar-idle.png` |
| 295 | `char-avatar-idle.jpg` |
| 296 | `char-avatar-idle.svg` |
| 302 | `char-avatar-listening.png` |
| 303 | `char-avatar-listening.jpg` |
| 304 | `char-avatar-listening.svg` |
| 310 | `char-avatar-speaking.png` |
| 311 | `char-avatar-speaking.jpg` |
| 312 | `char-avatar-speaking.svg` |
| 318 | `char-avatar-thinking.png` |
| 319 | `char-avatar-thinking.jpg` |
| 320 | `char-avatar-thinking.svg` |
| 331 | `Avatar2D_{:?}` |
| 340 | `char-avatar-idle.png` |
| 346 | `Avatar2D_Fallback_{:?}` |
| 364 | `avatar_2d_bind_group_{:?}` |

---

## `ui/renderer/pipeline_3d.rs`

- **Path**: `src/ui/renderer/pipeline_3d.rs`
- **Comments**: 6
- **Strings**: 23

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 56 | `//` | 0: MaterialUniform |
| 67 | `//` | 1: Sampler |
| 74 | `//` | 2: Base Color Texture |
| 85 | `//` | 3: Shade Texture |
| 96 | `//` | 4: Emissive Texture |
| 107 | `//` | 5: Matcap Texture |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 35 | `camera_bind_group_layout` |
| 50 | `bone_bind_group_layout` |
| 119 | `material_bind_group_layout` |
| 122 | `../shader.wgsl` |
| 125 | `Render Pipeline Layout` |
| 135 | `MToon Anime Pipeline` |
| 139 | `vs_main` |
| 174 | `fs_main` |
| 211 | `Camera Buffer` |
| 221 | `camera_bind_group` |
| 228 | `Bone Buffer` |
| 238 | `bone_bind_group` |
| 445 | `[Model Bounds] Y: [{:.3} .. {:.3}], Extent X: {:.3}` |
| 455 | `Shared_Dummy_White` |
| 459 | `Shared_Dummy_Black` |
| 505 | `Base_{}` |
| 516 | `Shade_{}` |
| 527 | `Emissive_{}` |
| 538 | `Matcap_{}` |
| 580 | `Material Uniform Buffer` |
| 613 | `material_bind_group` |
| 629 | `Vertex Buffer` |
| 635 | `Index Buffer` |

---

## `ui/renderer/types.rs`

- **Path**: `src/ui/renderer/types.rs`
- **Comments**: 3
- **Strings**: 0

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 21 | `//` | rgb + 0.0 |
| 22 | `//` | rgb + 0.0 |
| 23 | `//` | rgb + 0.0 |


### String Literals (`"..."`)

_No string literals found._

---

## `ui/spring_bone.rs`

- **Path**: `src/ui/spring_bone.rs`
- **Comments**: 14
- **Strings**: 24

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 40 | `//` | 1. Parse colliders |
| 71 | `//` | 2. Parse collider groups: group_idx -> Vec<collider_idx> |
| 87 | `//` | 3. Parse springs and joints |
| 91 | `//` | Collect all relevant colliders for this spring chain |
| 198 | `//` | Precompute position bola collider in world space |
| 222 | `//` | Rest world transform for this joint |
| 234 | `//` | Initialize start tail on frame pertama |
| 242 | `//` | 1. Verlet Integration (Inertia + Spring Stiffness + Gravity) - 100% Identical to Three-VRM Pixiv Standard |
| 248 | `//` | 2. Length Constraint (Consistent spring bone joint length in world space) |
| 257 | `//` | 3. Collision Resolution (Prevents hair/skirt clipping through head/chest colliders) |
| 267 | `//` | Recalibrate bone length after collider bounce |
| 277 | `//` | 4. Update history position |
| 281 | `//` | 5. Direct bone rotation to next_tail (Identical to Three-VRM: setFromUnitVectors premultiply initialLocalRotation) |
| 299 | `//` | Update entire child hierarchy of this bone so subsequent hair chain follows |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 38 | `extensions` |
| 38 | `VRMC_springBone` |
| 42 | `colliders` |
| 44 | `node` |
| 45 | `shape` |
| 45 | `sphere` |
| 47 | `offset` |
| 58 | `radius` |
| 73 | `colliderGroups` |
| 76 | `colliders` |
| 89 | `springs` |
| 93 | `colliderGroups` |
| 107 | `joints` |
| 112 | `node` |
| 116 | `node` |
| 136 | `hitRadius` |
| 140 | `stiffness` |
| 144 | `gravityPower` |
| 148 | `gravityDir` |
| 159 | `dragForce` |
| 187 | `[SpringBone] Initialized: {} dynamic spring joints, {} collision colliders` |
| 334 | `assets/char.vrm` |
| 344 | `dummy` |
| 358 | `SpringBoneManager should parse VRMC_springBone successfully` |

---

## `ui/vrm_loader.rs`

- **Path**: `src/ui/vrm_loader.rs`
- **Comments**: 18
- **Strings**: 180

### Comments (`//`, `///`, `/* */`)

| Line | Type | Content |
|:---:|:---:|:---|
| 38 | `//` | 0.0 - 1.0 |
| 132 | `//` | Memory optimization: Clamp texture resolution to 1024px |
| 166 | `//` | Execute VRM parsing and texture decoding in dedicated thread with 32MB stack to prevent stack overflow |
| 191 | `//` | Read raw JSON from GLB for accurate parsing of MToon and VRMC_vrm extensions |
| 206 | `//` | 1. Ekstrak Material, MToon & KHR_materials_unlit Extensions |
| 248 | `//` | Base Color Texture |
| 255 | `//` | Emissive Texture & Factor |
| 263 | `//` | MToon Parsing from raw JSON |
| 359 | `//` | 2. Ekstrak Node Hierarchy & Bone Mapping |
| 395 | `//` | Mapping Humanoid Bones from VRMC_vrm extension in raw JSON |
| 432 | `//` | 3. Ekstrak Mesh & Primitives with Relasi Node Mesh |
| 484 | `//` | Ekstrak Morph Targets (Blend Shapes for Facial Expression & Lip Sync) |
| 512 | `//` | 4. Ekstrak Skins (Joints & Inverse Bind Matrices) |
| 528 | `//` | 5. Ekstrak Blend Shapes & Expressions (VRM 0.X & VRM 1.0) |
| 531 | `//` | VRM 0.X BlendShapeMaster |
| 575 | `//` | VRM 1.0 Expressions (preset & custom) |
| 629 | `//` | Precompute node_to_bone reverse mapping & known_mesh_indices for O(1) direct lookup when render loop |
| 638 | `//` | Precompute parent indices and rest world rotations for Three-VRM Normalized Rig |


### String Literals (`"..."`)

| Line | Literal |
|:---:|:---|
| 168 | `vrm-loader-worker` |
| 175 | `VRM Loader thread panicked during model parsing` |
| 183 | `[VRM Loader] Reading Three.js 1:1 VRM model: {:?}` |
| 211 | `materials` |
| 215 | `Material` |
| 218 | `hair` |
| 220 | `face` |
| 221 | `body` |
| 222 | `skin` |
| 225 | `eye` |
| 225 | `iris` |
| 227 | `cloth` |
| 228 | `tops` |
| 229 | `bottoms` |
| 230 | `shoes` |
| 231 | `onepiece` |
| 273 | `extensions` |
| 274 | `VRMC_materials_mtoon` |
| 277 | `shadeColorFactor` |
| 289 | `shadingShiftFactor` |
| 295 | `shadingToonyFactor` |
| 300 | `matcapFactor` |
| 310 | `shadeMultiplyTexture` |
| 311 | `index` |
| 320 | `matcapTexture` |
| 321 | `index` |
| 333 | `   [Material] '{}' -> BaseTex: {}, ShadeTex: {}, EmissiveTex: {}, MatcapTex: {}` |
| 398 | `extensions` |
| 399 | `VRMC_vrm` |
| 402 | `humanoid` |
| 403 | `humanBones` |
| 407 | `node` |
| 409 | `leftThumbMetacarpal` |
| 409 | `leftThumbProximal` |
| 409 | `leftThumb` |
| 410 | `leftIndexProximal` |
| 410 | `leftIndex` |
| 411 | `leftMiddleProximal` |
| 411 | `leftMiddle` |
| 412 | `leftRingProximal` |
| 412 | `leftRing` |
| 413 | `leftLittleProximal` |
| 413 | `leftLittle` |
| 414 | `rightThumbMetacarpal` |
| 414 | `rightThumbProximal` |
| 414 | `rightThumb` |
| 415 | `rightIndexProximal` |
| 415 | `rightIndex` |
| 416 | `rightMiddleProximal` |
| 416 | `rightMiddle` |
| 417 | `rightRingProximal` |
| 417 | `rightRing` |
| 418 | `rightLittleProximal` |
| 418 | `rightLittle` |
| 534 | `extensions` |
| 535 | `VRM` |
| 538 | `blendShapeMaster` |
| 539 | `blendShapeGroups` |
| 544 | `name` |
| 546 | `presetName` |
| 554 | `binds` |
| 557 | `mesh` |
| 559 | `index` |
| 560 | `weight` |
| 578 | `extensions` |
| 579 | `VRMC_vrm` |
| 581 | `expressions` |
| 588 | `morphTargetBinds` |
| 592 | `node` |
| 600 | `index` |
| 603 | `weight` |
| 673 | `[VRM Loader] Successfully loaded {} mesh parts, {} material layers, {} blend expressions` |
| 697 | `Head` |
| 697 | `head` |
| 698 | `head` |
| 699 | `Chest` |
| 699 | `chest` |
| 700 | `chest` |
| 701 | `Spine` |
| 701 | `spine` |
| 702 | `spine` |
| 703 | `Hips` |
| 703 | `hips` |
| 704 | `hips` |
| 705 | `LeftUpperArm` |
| 706 | `L_UpperArm` |
| 707 | `leftUpperArm` |
| 709 | `leftUpperArm` |
| 710 | `LeftLowerArm` |
| 711 | `L_LowerArm` |
| 712 | `leftLowerArm` |
| 714 | `leftLowerArm` |
| 715 | `LeftHand` |
| 715 | `L_Hand` |
| 715 | `leftHand` |
| 717 | `leftHand` |
| 718 | `L_Thumb1` |
| 719 | `LeftThumbProximal` |
| 720 | `leftThumb` |
| 722 | `leftThumb` |
| 723 | `L_Index1` |
| 724 | `LeftIndexProximal` |
| 725 | `leftIndex` |
| 727 | `leftIndex` |
| 728 | `L_Middle1` |
| 729 | `LeftMiddleProximal` |
| 730 | `leftMiddle` |
| 732 | `leftMiddle` |
| 733 | `L_Ring1` |
| 734 | `LeftRingProximal` |
| 735 | `leftRing` |
| 737 | `leftRing` |
| 738 | `L_Little1` |
| 739 | `LeftLittleProximal` |
| 740 | `leftLittle` |
| 742 | `leftLittle` |
| 743 | `RightUpperArm` |
| 744 | `R_UpperArm` |
| 745 | `rightUpperArm` |
| 747 | `rightUpperArm` |
| 748 | `RightLowerArm` |
| 749 | `R_LowerArm` |
| 750 | `rightLowerArm` |
| 752 | `rightLowerArm` |
| 753 | `RightHand` |
| 754 | `R_Hand` |
| 755 | `rightHand` |
| 757 | `rightHand` |
| 758 | `R_Thumb1` |
| 759 | `RightThumbProximal` |
| 760 | `rightThumb` |
| 762 | `rightThumb` |
| 763 | `R_Index1` |
| 764 | `RightIndexProximal` |
| 765 | `rightIndex` |
| 767 | `rightIndex` |
| 768 | `R_Middle1` |
| 769 | `RightMiddleProximal` |
| 770 | `rightMiddle` |
| 772 | `rightMiddle` |
| 773 | `R_Ring1` |
| 774 | `RightRingProximal` |
| 775 | `rightRing` |
| 777 | `rightRing` |
| 778 | `R_Little1` |
| 779 | `RightLittleProximal` |
| 780 | `rightLittle` |
| 782 | `rightLittle` |
| 783 | `LeftShoulder` |
| 784 | `L_Shoulder` |
| 785 | `leftShoulder` |
| 787 | `leftShoulder` |
| 788 | `RightShoulder` |
| 789 | `R_Shoulder` |
| 790 | `rightShoulder` |
| 792 | `rightShoulder` |
| 793 | `LeftUpperLeg` |
| 794 | `L_UpperLeg` |
| 795 | `leftUpperLeg` |
| 797 | `leftUpperLeg` |
| 798 | `LeftLowerLeg` |
| 799 | `L_LowerLeg` |
| 800 | `leftLowerLeg` |
| 802 | `leftLowerLeg` |
| 803 | `LeftFoot` |
| 803 | `L_Foot` |
| 803 | `leftFoot` |
| 805 | `leftFoot` |
| 806 | `RightUpperLeg` |
| 807 | `R_UpperLeg` |
| 808 | `rightUpperLeg` |
| 810 | `rightUpperLeg` |
| 811 | `RightLowerLeg` |
| 812 | `R_LowerLeg` |
| 813 | `rightLowerLeg` |
| 815 | `rightLowerLeg` |
| 816 | `RightFoot` |
| 817 | `R_Foot` |
| 818 | `rightFoot` |
| 820 | `rightFoot` |

---
