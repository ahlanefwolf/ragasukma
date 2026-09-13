import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { VRMLoaderPlugin } from '@pixiv/three-vrm';
import { updateInteractivePose, setPosesConfig, getActivePosesConfig } from './assets/vrm_pose.js';

function getTauriInvoke() {
  if (window.__TAURI__?.core?.invoke) return window.__TAURI__.core.invoke;
  if (window.__TAURI__?.invoke) return window.__TAURI__.invoke;
  if (window.__TAURI_INTERNALS__?.invoke) return window.__TAURI_INTERNALS__.invoke;
  return null;
}

async function invoke(cmd, args = {}) {
  let inv = getTauriInvoke();
  if (inv) {
    try {
      return await inv(cmd, args);
    } catch (e) {
      console.error('[Invoke Error]', cmd, e);
      return null;
    }
  }
  for (let i = 0; i < 30; i++) {
    await new Promise(r => setTimeout(r, 35));
    inv = getTauriInvoke();
    if (inv) {
      try {
        return await inv(cmd, args);
      } catch (e) {
        console.error('[Invoke Error]', cmd, e);
        return null;
      }
    }
  }
  console.warn('⚠️ [Tauri Bridge] invoke not ready for:', cmd);
  return null;
}

const root = document.getElementById('root');
const imgElement = document.getElementById('avatar-image');
const canvasContainer = document.getElementById('canvas-container');
const contextMenu = document.getElementById('context-menu');

const btnQuit = document.getElementById('btn-quit');
const statusToast = document.getElementById('status-toast');

let toastTimer = null;
function showToast(text, duration = 1200) {
  if (!statusToast) return;
  statusToast.textContent = text;
  statusToast.classList.add('show');
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => {
    statusToast.classList.remove('show');
  }, duration);
}

let avatarType = 'vrm';
let isVrm = true;

// Three.js VRM state
let currentVrm = null;
let currentState = 'idle';
let currentEmotion = 'neutral';
let clock = new THREE.Clock();
let renderer, scene, camera;

// Expression & Lip-Sync Management
const currentExpressions = {
  happy: 0.0,
  joy: 0.0,
  sad: 0.0,
  sorrow: 0.0,
  surprised: 0.0,
  angry: 0.0,
  relaxed: 0.0,
  blink: 0.0,
  blinkLeft: 0.0,
  blinkRight: 0.0,
  aa: 0.0,
  ih: 0.0,
  ou: 0.0,
  ee: 0.0,
  oh: 0.0
};
const targetExpressions = { ...currentExpressions };

function setTargetExpression(name, value) {
  if (targetExpressions[name] !== undefined) targetExpressions[name] = value;
  if (name === 'happy') targetExpressions['joy'] = value;
  if (name === 'joy') targetExpressions['happy'] = value;
  if (name === 'sad') targetExpressions['sorrow'] = value;
  if (name === 'sorrow') targetExpressions['sad'] = value;
  if (name.startsWith('viseme_')) {
    const short = name.replace('viseme_', '');
    if (targetExpressions[short] !== undefined) targetExpressions[short] = value;
  }
}

function applyLerpedExpressions(vrm, deltaTime) {
  if (!vrm || !vrm.expressionManager) return;
  // Transisi blendshape & emosi wajah organik dengan peluruhan eksponensial halus
  const lerpSpeed = 1.0 - Math.exp(-deltaTime * 7.5);

  for (const [name, targetVal] of Object.entries(targetExpressions)) {
    if (currentExpressions[name] === undefined) {
      currentExpressions[name] = 0.0;
    }
    currentExpressions[name] += (targetVal - currentExpressions[name]) * lerpSpeed;
    try {
      vrm.expressionManager.setValue(name, Math.max(0.0, Math.min(1.0, currentExpressions[name])));
    } catch (e) {}
  }
}

let nextBlinkTime = 2.0;
let blinkProgress = 0.0;
let isBlinking = false;
let visemeTimer = 0.0;
const visemes = ['aa', 'ih', 'ou', 'ee', 'oh'];
let currentVisemeIndex = 0;

function updateStateAnimations(deltaTime, elapsedTime) {
  if (!currentVrm) return;

  // Spring Bone Physics dimatikan saat idle untuk menghemat iGPU secara drastis
  if (currentState === 'speaking' && currentVrm.springBoneManager && currentVrm.springBoneManager.externalForce) {
    currentVrm.springBoneManager.externalForce.set(
      Math.sin(elapsedTime * 2.8) * 0.04 + 0.02,
      Math.cos(elapsedTime * 3.2) * 0.01 - 0.02,
      Math.sin(elapsedTime * 2.0) * 0.03
    );
  }

  // Auto Blink
  if (!isBlinking && elapsedTime > nextBlinkTime) {
    isBlinking = true;
    blinkProgress = 0.0;
    nextBlinkTime = elapsedTime + 2.5 + Math.random() * 3.0;
  }

  if (isBlinking) {
    blinkProgress += deltaTime * 14.0;
    const blinkVal = Math.sin(Math.min(blinkProgress, Math.PI));
    setTargetExpression('blink', blinkVal);
    setTargetExpression('blinkLeft', blinkVal);
    setTargetExpression('blinkRight', blinkVal);
    if (blinkProgress >= Math.PI) {
      isBlinking = false;
      setTargetExpression('blink', 0.0);
      setTargetExpression('blinkLeft', 0.0);
      setTargetExpression('blinkRight', 0.0);
    }
  }

  // Blendshapes Emosi
  setTargetExpression('happy', 0.0);
  setTargetExpression('joy', 0.0);
  setTargetExpression('sad', 0.0);
  setTargetExpression('sorrow', 0.0);
  setTargetExpression('surprised', 0.0);
  setTargetExpression('angry', 0.0);
  setTargetExpression('relaxed', 0.0);

  // Ambil konfigurasi emosi per-state dari poses.json jika ada
  const posesCfg = (typeof getActivePosesConfig === 'function') ? getActivePosesConfig() : null;
  const stateDefaultEmotion = posesCfg?.[currentState]?.emotion || (currentState === 'thinking' ? 'thinking' : (currentState === 'listening' ? 'attentive' : 'shy'));

  if (currentState === 'speaking') {
    // Saat speaking: utamakan emosi responsif dari respon AI (currentEmotion), fallback ke poses.json
    const activeEmo = (currentEmotion && currentEmotion !== 'neutral') ? currentEmotion : stateDefaultEmotion;

    if (activeEmo === 'shy' || activeEmo === 'malu') {
      // 1. Shy / Malu: senyum malu-malu lembut, mata sedikit menyipit
      setTargetExpression('relaxed', 0.8);
      setTargetExpression('happy', 0.4);
    } else if (activeEmo === 'happy' || activeEmo === 'senang') {
      // 2. Happy / Senang: senyum gembira lebar
      setTargetExpression('happy', 0.95);
      setTargetExpression('joy', 0.95);
    } else if (activeEmo === 'excited' || activeEmo === 'semangat') {
      // 3. Excited / Semangat: mata membesar berbinar gembira
      setTargetExpression('happy', 0.85);
      setTargetExpression('joy', 0.85);
      setTargetExpression('surprised', 0.4);
    } else if (activeEmo === 'thinking') {
      // 4. Thinking: merenung sambil bicara
      setTargetExpression('relaxed', 0.5);
      setTargetExpression('surprised', 0.25);
    } else if (activeEmo === 'surprised' || activeEmo === 'kaget') {
      // 5. Surprised / Kaget: mata terbelalak lebar
      setTargetExpression('surprised', 0.95);
    } else if (activeEmo === 'confused' || activeEmo === 'bingung') {
      // 6. Confused / Bingung: alis sedikit mengerut heran
      setTargetExpression('surprised', 0.4);
      setTargetExpression('sad', 0.35);
    } else if (activeEmo === 'sad' || activeEmo === 'sedih') {
      // 7. Sad / Sedih: mata sayu bersedih
      setTargetExpression('sad', 0.9);
      setTargetExpression('sorrow', 0.9);
    } else {
      // 8. Neutral / Attentive default speaking
      setTargetExpression('happy', 0.4);
      setTargetExpression('relaxed', 0.3);
    }

    visemeTimer += deltaTime * 16.0;
    const mouthOpen = Math.abs(Math.sin(visemeTimer));
    if (visemeTimer > Math.PI) {
      visemeTimer = 0;
      currentVisemeIndex = Math.floor(Math.random() * visemes.length);
    }

    const activeViseme = visemes[currentVisemeIndex];
    visemes.forEach((v) => {
      const val = (v === activeViseme) ? mouthOpen * 0.9 : 0.0;
      setTargetExpression(`viseme_${v}`, val);
      setTargetExpression(v, val);
    });
  } else if (currentState === 'thinking') {
    // Mode Berpikir: tatapan melirik bingung/mikir sesuai poses.json
    if (stateDefaultEmotion === 'confused') {
      setTargetExpression('surprised', 0.4);
      setTargetExpression('sad', 0.35);
    } else {
      setTargetExpression('relaxed', 0.6);
      setTargetExpression('surprised', 0.3);
    }
    visemes.forEach((v) => {
      setTargetExpression(`viseme_${v}`, 0.0);
      setTargetExpression(v, 0.0);
    });
  } else if (currentState === 'listening') {
    // Mode Menyimak: mengikuti emotion poses.json (misal 'attentive' / 'surprised')
    setTargetExpression('surprised', 0.35);
    setTargetExpression('happy', 0.45);
    visemes.forEach((v) => {
      setTargetExpression(`viseme_${v}`, 0.0);
      setTargetExpression(v, 0.0);
    });
  } else {
    // Mode Idle: ikuti emotion dari poses.json ("idle": { "emotion": "shy" })
    if (stateDefaultEmotion === 'shy' || stateDefaultEmotion === 'malu') {
      setTargetExpression('relaxed', 0.6);
      setTargetExpression('happy', 0.35);
    } else if (stateDefaultEmotion === 'happy') {
      setTargetExpression('happy', 0.6);
      setTargetExpression('joy', 0.4);
    } else {
      setTargetExpression('relaxed', 0.3);
    }
    visemes.forEach((v) => {
      setTargetExpression(`viseme_${v}`, 0.0);
      setTargetExpression(v, 0.0);
    });
  }

  applyLerpedExpressions(currentVrm, deltaTime);

  try {
    if (typeof updateInteractivePose === 'function') {
      updateInteractivePose(currentVrm, currentState, currentEmotion, deltaTime, elapsedTime, camera);
    }
    currentVrm.update(deltaTime);
  } catch (e) {}
}

// -------------------------------------------------------------
// VRM Setup & Auto-Fit Camera
// -------------------------------------------------------------
let baseVrmTargetY = 0.85;
let baseVrmTargetHeight = 1.35;
let baseVrmTargetWidth = 0.65;
let hasMeasuredVrm = false;

function setupVrmFraming(vrm) {
  if (!vrm || !camera) return;

  const bbox = new THREE.Box3().setFromObject(vrm.scene);
  const center = bbox.getCenter(new THREE.Vector3());
  const size = bbox.getSize(new THREE.Vector3());

  // Framing Full Body: simpan referensi ukuran dasar sekali saja
  baseVrmTargetY = center.y || 0.85;
  baseVrmTargetHeight = Math.max(size.y * 1.12, 1.25);
  baseVrmTargetWidth = Math.max(size.x * 1.2, 0.6);
  hasMeasuredVrm = true;

  updateCameraProjection();
}

function updateCameraProjection() {
  if (!camera) return;
  const aspect = window.innerWidth / Math.max(window.innerHeight, 1);
  camera.aspect = aspect;
  camera.updateProjectionMatrix();

  const fovRad = camera.fov * (Math.PI / 180);
  // Fokus framing vertikal penuh yang kokoh dan konstan
  const distanceV = (baseVrmTargetHeight / 2) / Math.tan(fovRad / 2);
  const cameraZ = distanceV * 1.05;

  camera.position.set(0, baseVrmTargetY, cameraZ);
  camera.lookAt(0, baseVrmTargetY, 0);
}

function fitCameraToVrm(vrm) {
  if (!hasMeasuredVrm) {
    setupVrmFraming(vrm);
  } else {
    updateCameraProjection();
  }
}

let isVrmAnimationRunning = false;
let vrmLoaderInstance = null;

let cachedVrmBytes = null;
window.getCachedVrmBytes = () => cachedVrmBytes;

async function loadVrmModelToScene() {
  if (!scene) return;
  
  const loadingEl = document.getElementById('loading-indicator');
  if (loadingEl) loadingEl.style.display = 'flex';

  // 1. Hapus model VRM lama dari scene jika ada
  if (currentVrm && currentVrm.scene) {
    scene.remove(currentVrm.scene);
    currentVrm.scene.traverse((obj) => {
      if (obj.geometry) obj.geometry.dispose();
      if (obj.material) {
        const mats = Array.isArray(obj.material) ? obj.material : [obj.material];
        mats.forEach(m => m.dispose());
      }
    });
    hasMeasuredVrm = false;
    currentVrm = null;
  }

  if (!vrmLoaderInstance) {
    vrmLoaderInstance = new GLTFLoader();
    vrmLoaderInstance.register((parser) => new VRMLoaderPlugin(parser));
  }

  try {
    let bytes = cachedVrmBytes;
    if (!bytes) {
      console.log('📦 [VRM Loader] Meminta file char.vrm dari backend...');
      bytes = await invoke('get_external_asset_bytes', { filename: 'char.vrm' });
      if (bytes && bytes.length > 0) {
        cachedVrmBytes = bytes;
      }
    }

    if (bytes && bytes.length > 0) {
      console.log(`📦 [VRM Loader] Parsing ${bytes.length} bytes via Three.js...`);
      const u8 = (bytes instanceof Uint8Array) ? bytes : new Uint8Array(bytes);
      
      vrmLoaderInstance.parse(
        u8.buffer,
        '',
        (gltf) => {
          console.log('✅ [VRM Loader] GLTF parse sukses, menyiapkan VRM avatar...');
          const vrm = gltf.userData.vrm;
          if (!vrm) {
            console.error('⚠️ [VRM Loader] gltf.userData.vrm kosong!');
            if (loadingEl) loadingEl.style.display = 'none';
            return;
          }
          currentVrm = vrm;
          scene.add(vrm.scene);
          vrm.scene.rotation.y = 0;

          fitCameraToVrm(vrm);

          if (gltf.parser) {
            delete gltf.parser.json;
            delete gltf.parser.associations;
          }
          delete gltf.scenes;
          delete gltf.animations;

          if (!isVrmAnimationRunning) {
            isVrmAnimationRunning = true;
            console.log('🎬 [VRM Loader] Memulai loop render VRM!');
            animateVrm();
          }
          if (loadingEl) loadingEl.style.display = 'none';
        },
        (err) => {
          console.error('❌ [VRM Loader] Error parsing VRM buffer:', err);
          if (loadingEl) loadingEl.style.display = 'none';
        }
      );
    } else {
      if (loadingEl) loadingEl.style.display = 'none';
    }
  } catch(e) {
    console.error('❌ [VRM Loader] Exception get_external_asset_bytes:', e);
    if (loadingEl) loadingEl.style.display = 'none';
  }
}

function initVrmViewer() {
  if (renderer && scene) {
    loadVrmModelToScene();
    return;
  }

  const width = window.innerWidth;
  const height = window.innerHeight;

  scene = new THREE.Scene();

  camera = new THREE.PerspectiveCamera(28.0, width / Math.max(height, 1), 0.1, 20.0);
  camera.position.set(0.0, 1.35, 1.45);

  renderer = new THREE.WebGLRenderer({
    alpha: true,
    antialias: true,
    powerPreference: 'low-power'
  });
  renderer.setSize(width, height);
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1.0, 1.5));
  renderer.outputColorSpace = THREE.SRGBColorSpace;
  renderer.autoClear = false;

  canvasContainer.appendChild(renderer.domElement);

  const frontLight = new THREE.DirectionalLight(0xfff5ea, 1.4);
  frontLight.position.set(0.8, 1.8, 1.5).normalize();
  scene.add(frontLight);

  const fillLight = new THREE.DirectionalLight(0xe8f4ff, 0.75);
  fillLight.position.set(-1.5, 0.8, 1.0).normalize();
  scene.add(fillLight);

  const ambient = new THREE.AmbientLight(0xffffff, 0.95);
  scene.add(ambient);

  loadVrmModelToScene();

  let resizeTimeout = null;
  window.addEventListener('resize', () => {
    if (!renderer || !camera) return;
    if (resizeTimeout) clearTimeout(resizeTimeout);
    resizeTimeout = setTimeout(() => {
      try {
        updateCameraProjection();
        renderer.setSize(window.innerWidth, window.innerHeight);
      } catch (err) {
        console.warn('⚠️ WebGL resize error handled:', err);
      }
    }, 16);
  });
}

let lastRenderTime = 0;

function animateVrm(timestamp) {
  requestAnimationFrame(animateVrm);
  if (document.hidden) return;

  // Smooth 60/30 FPS adaptive loop:
  // idle/listening: 30 FPS (halus dan sangat hemat resource di iGPU)
  // thinking/speaking: 60 FPS (responsif & mulus saat berekspresi)
  let targetFps = (currentState === 'speaking' || currentState === 'thinking') ? 60 : 30;
  const interval = 1000 / targetFps;
  const elapsedSinceLast = timestamp - lastRenderTime;

  if (elapsedSinceLast < interval) {
    return;
  }
  lastRenderTime = timestamp - (elapsedSinceLast % interval);

  const delta = Math.min(clock.getDelta(), 0.1);
  const elapsedTime = clock.getElapsedTime();

  updateStateAnimations(delta, elapsedTime);

  if (renderer && scene && camera) {
    renderer.clear();
    renderer.render(scene, camera);
  }
}

// -------------------------------------------------------------
// Mouse, Dragging & Gestures
// -------------------------------------------------------------

function setupInteractions() {
  // Context Menu Hide on click outside
  window.addEventListener('pointerdown', (e) => {
    if (contextMenu.style.display !== 'none' && !contextMenu.contains(e.target)) {
      contextMenu.style.display = 'none';
    }
  });

  // Tombol Escape untuk menutup popup
  window.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && contextMenu && contextMenu.style.display !== 'none') {
      contextMenu.style.display = 'none';
    }
  });

  // Right Click: Tampilkan Popup Menu Pilihan
  window.addEventListener('contextmenu', (e) => {
    e.preventDefault();
    e.stopPropagation();
    if (contextMenu.style.display === 'flex') {
      contextMenu.style.display = 'none';
    } else {
      contextMenu.style.display = 'flex';
    }
  });

  // Scroll Wheel Resize (Perbesar / Perkecil Overlay saat kursor di atas avatar)
  let isResizing = false;
  window.addEventListener('wheel', (e) => {
    // Abaikan jika sedang membuka popup menu atau panel iframe
    if (contextMenu && contextMenu.style.display !== 'none') return;
    if (panelContainer && panelContainer.style.display !== 'none') return;

    e.preventDefault();
    e.stopPropagation();

    if (isResizing) return;
    isResizing = true;

    const delta = e.deltaY < 0 ? 30 : -30;
    invoke('change_size_delta', { delta })
      .catch((err) => {
        console.warn('[wheel resize error]:', err);
      })
      .finally(() => {
        setTimeout(() => {
          isResizing = false;
        }, 120);
      });
  }, { passive: false });

  // Native Smooth Dragging
  root.addEventListener('mousedown', (e) => {
    if (e.button !== 0) return; // Only Left Click
    if (contextMenu && contextMenu.contains(e.target)) return;
    if (panelContainer && panelContainer.style.display !== 'none') return;

    // Aktifkan native OS hardware accelerated window dragging
    invoke('start_drag');
  });
}

// -------------------------------------------------------------
// Panel & Iframe Single-Window Management
// -------------------------------------------------------------
const panelContainer = document.getElementById('panel-container');
const panelTitleText = document.getElementById('panel-title-text');
const btnBackOverlay = document.getElementById('btn-back-overlay');
const contentFrame = document.getElementById('content-frame');
const topbarNavBtns = document.querySelectorAll('.topbar-nav-btn');

let currentActiveUrl = '';

function updateNavActiveState(url) {
  topbarNavBtns.forEach((btn) => {
    if (btn.getAttribute('data-url') === url) {
      btn.classList.add('active');
    } else {
      btn.classList.remove('active');
    }
  });
}

export async function openPanel(url, title = '') {
  if (!url) return;
  currentActiveUrl = url;
  
  if (contextMenu) contextMenu.style.display = 'none';
  if (panelTitleText && title) panelTitleText.textContent = title;
  
  updateNavActiveState(url);

  if (panelContainer) {
    panelContainer.style.display = 'flex';
  }

  if (contentFrame && contentFrame.getAttribute('src') !== url) {
    contentFrame.src = url;
  }

  try {
    await invoke('set_window_mode', { mode: 'panel', page: url });
  } catch (e) {
    console.warn('[openPanel] Failed to set panel mode:', e);
  }
}

export async function closePanel() {
  currentActiveUrl = '';
  if (panelContainer) {
    panelContainer.style.display = 'none';
  }
  if (contentFrame) {
    contentFrame.src = 'about:blank';
  }
  try {
    await invoke('set_window_mode', { mode: 'overlay' });
  } catch (e) {
    console.warn('[closePanel] Failed to set overlay mode:', e);
  }
}

window.openPanel = openPanel;
window.closePanel = closePanel;

if (btnBackOverlay) {
  btnBackOverlay.addEventListener('click', () => {
    closePanel();
  });
}

topbarNavBtns.forEach((btn) => {
  btn.addEventListener('click', () => {
    const url = btn.getAttribute('data-url');
    const title = btn.getAttribute('data-title') || '';
    if (url) {
      openPanel(url, title);
    }
  });
});

window.addEventListener('message', (event) => {
  if (event.data?.type === 'navigate' && event.data?.url) {
    openPanel(event.data.url, event.data.title || '');
  } else if (event.data?.type === 'close_panel' || event.data?.type === 'back_to_overlay') {
    closePanel();
  }
});

function setupMenuActions() {
  const btnClosePopup = document.getElementById('btn-close-popup');
  const btnMenu = document.getElementById('btn-menu');
  const btnSettings = document.getElementById('btn-settings');
  const btnSkills = document.getElementById('btn-skills');
  const btnTuner = document.getElementById('btn-tuner');
  const btnQuit = document.getElementById('btn-quit');

  if (btnClosePopup) {
    btnClosePopup.addEventListener('click', (e) => {
      e.stopPropagation();
      if (contextMenu) contextMenu.style.display = 'none';
    });
  }

  if (btnMenu) {
    btnMenu.addEventListener('click', () => {
      openPanel('menu.html', 'Menu Utama');
    });
  }

  if (btnSettings) {
    btnSettings.addEventListener('click', () => {
      openPanel('settings.html', 'Pengaturan');
    });
  }

  if (btnSkills) {
    btnSkills.addEventListener('click', () => {
      openPanel('skills.html', 'Skill Commands');
    });
  }

  if (btnTuner) {
    btnTuner.addEventListener('click', () => {
      openPanel('tuner.html', 'Pose Tuner');
    });
  }

  if (btnQuit) {
    btnQuit.addEventListener('click', () => {
      if (contextMenu) contextMenu.style.display = 'none';
      invoke('send_command', { cmd: 'quit' });
    });
  }

  // Corner Position Buttons
  document.querySelectorAll('.context-corner-btn').forEach((btn) => {
    btn.addEventListener('click', () => {
      const corner = btn.getAttribute('data-corner');
      if (contextMenu) contextMenu.style.display = 'none';
      invoke('set_corner', { corner });
    });
  });
}

let cachedSvgUrls = {};

async function updateSvgState(state) {
  if (isVrm || !imgElement) return;

  const mapped = (state === 'speaking') ? 'talking'
    : (state === 'listening') ? 'listening'
    : (state === 'thinking' || state === 'processing') ? 'thinking'
    : 'idle';

  const filename = `char-avatar-${mapped}.svg`;
  if (!cachedSvgUrls[filename]) {
    try {
      const uri = await invoke('get_external_asset', { filename });
      if (uri) cachedSvgUrls[filename] = uri;
    } catch(e) {}
  }

  const src = cachedSvgUrls[filename] || `assets/${filename}`;
  if (imgElement.src !== src) {
    imgElement.src = src;
  }
}

async function loadExternalPoses() {
  try {
    const poses = await invoke('get_poses_config');
    if (poses && typeof poses === 'object' && Object.keys(poses).length > 0) {
      setPosesConfig(poses);
    }
  } catch(e) {
    console.warn('Gagal memuat poses.json:', e);
  }
}
window.loadExternalPoses = loadExternalPoses;
window.reloadAvatar = function() {
  cachedSvgUrls = {};
  if (isVrm) {
    initVrmViewer();
  } else {
    updateSvgState(currentState);
  }
};

let _pollCounter = 0;
async function pollState() {
  try {
    _pollCounter++;
    if (_pollCounter % 20 === 0) {
      // Periksa konfigurasi nama asisten & tipe avatar secara reaktif
      try {
        const config = await invoke('get_app_config');
        if (config) {
          if (config.assistant_name) {
            const titleEl = document.querySelector('.menu-title-text');
            if (titleEl && titleEl.textContent !== `Ragasukma - ${config.assistant_name}`) {
              titleEl.textContent = `Ragasukma - ${config.assistant_name}`;
            }
          }
          if (config.avatar_type) {
            const newType = config.avatar_type.toLowerCase();
            if (newType !== avatarType) {
              avatarType = newType;
              isVrm = avatarType === 'vrm';
              if (isVrm) {
                if (imgElement) imgElement.style.display = 'none';
                if (canvasContainer) canvasContainer.style.display = 'flex';
                if (!currentVrm) initVrmViewer();
              } else {
                if (imgElement) imgElement.style.display = 'block';
                if (canvasContainer) canvasContainer.style.display = 'none';
              }
            }
          }
        }
      } catch (e) {}
    }

    const data = await invoke('get_app_state');
    const state = data?.class || 'idle';
    const emotion = data?.emotion || 'neutral';
    const desc = data?.description || '';

    if (['idle', 'speaking', 'listening', 'thinking', 'processing'].includes(state)) {
      currentState = (state === 'processing') ? 'thinking' : state;
    }
    if (emotion) {
      currentEmotion = emotion;
    }

    if (!isVrm) {
      updateSvgState(currentState);
    }

    const isInMainMenu = desc.includes('Menu Utama') || !desc;
    const isPaused = desc.includes('Dijeda') || desc.includes('Paused');

    if (window._lastPausedState !== undefined && window._lastPausedState !== isPaused && !isInMainMenu) {
      if (isPaused) {
        showToast('Dijeda', 1200);
      } else {
        showToast('Dilanjutkan', 1200);
      }
    }
    window._lastPausedState = isPaused;
  } catch (err) {}
}

async function initApp() {
  setupMenuActions();
  setupInteractions();

  let asstName = 'Niki';
  try {
    const config = await invoke('get_app_config');
    if (config && config.avatar_type) {
      avatarType = config.avatar_type.toLowerCase();
    }
    if (config && config.assistant_name) {
      asstName = config.assistant_name;
      const titleEl = document.querySelector('.menu-title-text');
      if (titleEl) titleEl.textContent = `Ragasukma - ${asstName}`;
    }
  } catch (e) {}

  isVrm = avatarType === 'vrm';
  let isVrmInitialized = false;

  function ensureVrmLoaded() {
    if (!isVrm || isVrmInitialized) return;
    isVrmInitialized = true;
    if (imgElement) imgElement.style.display = 'none';
    if (canvasContainer) canvasContainer.style.display = 'flex';
    initVrmViewer();
    loadExternalPoses();
  }

  // Pemuatan VRM secara background (lazy) saat avatar window benar-benar visible
  if (isVrm) {
    if (imgElement) imgElement.style.display = 'none';
    if (canvasContainer) canvasContainer.style.display = 'flex';
    // Gunakan requestIdleCallback / setTimeout agar tidak memblokir render awal menu & UI
    setTimeout(() => {
      ensureVrmLoaded();
    }, 1000);
  } else {
    if (imgElement) imgElement.style.display = 'block';
    if (canvasContainer) canvasContainer.style.display = 'none';
  }

  // Dynamic visibility detection: Hanya load VRM & render saat window aktif
  document.addEventListener('visibilitychange', () => {
    if (!document.hidden) {
      if (isVrm && !isVrmInitialized) {
        setTimeout(ensureVrmLoaded, 300);
      }
      lastRenderTime = performance.now();
    }
  });

  window.addEventListener('focus', () => {
    if (isVrm && !isVrmInitialized) {
      setTimeout(ensureVrmLoaded, 300);
    }
  });

  function setupRealtimeEventListener() {
    const getListen = () => window.__TAURI__?.event?.listen || window.__TAURI_INTERNALS__?.event?.listen || null;
    let listenFn = getListen();

    const attach = (listen) => {
      // Forward SSE & Tauri Events ke Sub-Iframe Panel secara otomatis
      const forwardToFrame = (eventName, data) => {
        if (contentFrame && contentFrame.contentWindow) {
          try {
            contentFrame.contentWindow.postMessage({
              type: 'tauri-event',
              event: eventName,
              payload: data
            }, '*');
          } catch (e) {}
        }
      };

      listen('assistant:state', (event) => {
        const data = event.payload;
        forwardToFrame('assistant:state', data);
        if (!data) return;
        const state = data.class || 'idle';
        const emotion = data.emotion || 'neutral';
        if (['idle', 'speaking', 'listening', 'thinking', 'processing'].includes(state)) {
          currentState = (state === 'processing') ? 'thinking' : state;
        }
        if (emotion) {
          currentEmotion = emotion;
        }
        if (!isVrm) {
          updateSvgState(currentState);
        }
      });

      listen('assistant:stream-chunk', (event) => {
        const payload = event.payload;
        forwardToFrame('assistant:stream-chunk', payload);
        if (payload && payload.sentence) {
          currentState = 'speaking';
          if (currentVrm) {
            setTargetExpression('aa', 0.65 + Math.random() * 0.3);
            setTimeout(() => {
              if (currentState === 'speaking') {
                setTargetExpression('aa', 0.2);
              }
            }, 120);
          }
        }
      });

      listen('open_view', (event) => {
        const payload = event.payload;
        if (payload && payload.url) {
          openPanel(payload.url, payload.title || 'Menu Utama');
        }
      });

      listen('request_close_panel', () => {
        closePanel();
      });

      listen('assistant:response-done', (event) => {
        forwardToFrame('assistant:response-done', event.payload);
        setTargetExpression('aa', 0.0);
        setTargetExpression('oh', 0.0);
        setTargetExpression('ee', 0.0);
      });

      listen('assistant:transcript', (event) => {
        forwardToFrame('assistant:transcript', event.payload);
      });

      listen('assistant:skill', (event) => {
        forwardToFrame('assistant:skill', event.payload);
      });

      listen('assistant:poses-updated', (event) => {
        forwardToFrame('assistant:poses-updated', event.payload);
        loadExternalPoses();
      });

      listen('assistant:config-updated', (event) => {
        const cfg = event.payload;
        forwardToFrame('assistant:config-updated', cfg);
        if (!cfg) return;
        if (cfg.assistant_name) {
          const titleEl = document.querySelector('.menu-title-text');
          if (titleEl) titleEl.textContent = `Ragasukma - ${cfg.assistant_name}`;
        }
        if (cfg.avatar_type) {
          const newType = cfg.avatar_type.toLowerCase();
          if (newType !== avatarType) {
            avatarType = newType;
            isVrm = avatarType === 'vrm';
            if (isVrm) {
              if (imgElement) imgElement.style.display = 'none';
              if (canvasContainer) canvasContainer.style.display = 'flex';
              if (!currentVrm) initVrmViewer();
            } else {
              if (imgElement) imgElement.style.display = 'block';
              if (canvasContainer) canvasContainer.style.display = 'none';
              updateSvgState(currentState);
            }
          }
        }
      });
    };

    if (listenFn) {
      attach(listenFn);
    } else {
      let retries = 0;
      const t = setInterval(() => {
        retries++;
        listenFn = getListen();
        if (listenFn || retries > 30) {
          clearInterval(t);
          if (listenFn) attach(listenFn);
        }
      }, 50);
    }
  }

  setupRealtimeEventListener();
  // Panggil pollState() hanya 1x di awal untuk inisialisasi, selebihnya 100% didorong event SSE/Tauri dari Rust
  pollState();
}

initApp();
