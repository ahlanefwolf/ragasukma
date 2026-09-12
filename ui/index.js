import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { VRMLoaderPlugin } from '@pixiv/three-vrm';
import { updateInteractivePose, setPosesConfig, getActivePosesConfig } from './assets/vrm_pose.js';

async function invoke(cmd, args) {
  if (window.__TAURI__?.core?.invoke) {
    return await window.__TAURI__.core.invoke(cmd, args);
  }
  for (let i = 0; i < 25; i++) {
    await new Promise(r => setTimeout(r, 40));
    if (window.__TAURI__?.core?.invoke) {
      return await window.__TAURI__.core.invoke(cmd, args);
    }
  }
  console.warn('⚠️ [Tauri Bridge] invoke not ready for:', cmd);
  return null;
}

const root = document.getElementById('root');
const imgElement = document.getElementById('avatar-image');
const canvasContainer = document.getElementById('canvas-container');
const contextMenu = document.getElementById('context-menu');

const btnStart = document.getElementById('btn-start');
const btnTestVoice = document.getElementById('btn-test-voice');
const btnTestMic = document.getElementById('btn-test-mic');
const btnPause = document.getElementById('btn-pause');
const btnStopChat = document.getElementById('btn-stop-chat');
const btnResetSize = document.getElementById('btn-reset-size');
const btnQuit = document.getElementById('btn-quit');
const statusToast = document.getElementById('status-toast');
const cornerButtons = document.querySelectorAll('.corner-btn');

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
function fitCameraToVrm(vrm) {
  if (!vrm || !camera) return;

  // Cari posisi kepala (head bone) atau gunakan bounding box jika bone belum siap
  let headY = null;
  try {
    if (vrm.humanoid) {
      const headNode = vrm.humanoid.getNormalizedBoneNode('head');
      if (headNode) {
        const headWorldPos = new THREE.Vector3();
        headNode.getWorldPosition(headWorldPos);
        headY = headWorldPos.y;
      }
    }
  } catch(e) {}

  const bbox = new THREE.Box3().setFromObject(vrm.scene);
  const center = bbox.getCenter(new THREE.Vector3());
  const size = bbox.getSize(new THREE.Vector3());

  // Framing Full Body: ujung kepala sampai ujung kaki
  const targetY = center.y;
  const targetHeight = size.y * 1.05; // Seluruh tinggi karakter + sedikit margin atas/bawah
  const targetWidth = Math.max(size.x * 1.1, 0.50); // Lebar bahu & tangan dengan margin

  const aspect = window.innerWidth / Math.max(window.innerHeight, 1);
  const fovRad = camera.fov * (Math.PI / 180);
  
  // Hitung jarak kamera vertikal & horizontal agar full body masuk sempurna di 9:16
  const distanceV = (targetHeight / 2) / Math.tan(fovRad / 2);
  const distanceH = (targetWidth / 2) / (Math.tan(fovRad / 2) * aspect);
  const cameraZ = Math.max(distanceV, distanceH) * 1.05;

  camera.position.set(0, targetY, cameraZ);
  camera.lookAt(0, targetY, 0);
}

let isVrmAnimationRunning = false;
let vrmLoaderInstance = null;

async function loadVrmModelToScene() {
  if (!scene) return;
  
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
    currentVrm = null;
  }

  if (!vrmLoaderInstance) {
    vrmLoaderInstance = new GLTFLoader();
    vrmLoaderInstance.register((parser) => new VRMLoaderPlugin(parser));
  }

  try {
    console.log('📦 [VRM Loader] Meminta file char.vrm dari backend...');
    const bytes = await invoke('get_external_asset_bytes', { filename: 'char.vrm' });
    if (bytes && bytes.length > 0) {
      console.log(`📦 [VRM Loader] Menerima ${bytes.length} bytes, parsing via Three.js...`);
      const u8 = (bytes instanceof Uint8Array) ? bytes : new Uint8Array(bytes);
      
      vrmLoaderInstance.parse(
        u8.buffer,
        '',
        (gltf) => {
          console.log('✅ [VRM Loader] GLTF parse sukses, menyiapkan VRM avatar...');
          const vrm = gltf.userData.vrm;
          if (!vrm) {
            console.error('⚠️ [VRM Loader] gltf.userData.vrm kosong!');
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
        },
        (err) => {
          console.error('❌ [VRM Loader] Error parsing VRM buffer:', err);
        }
      );
    }
  } catch(e) {
    console.error('❌ [VRM Loader] Exception get_external_asset_bytes:', e);
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

  window.addEventListener('resize', () => {
    if (!renderer || !camera) return;
    camera.aspect = window.innerWidth / Math.max(window.innerHeight, 1);
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
    if (currentVrm) fitCameraToVrm(currentVrm);
  });
}

let lastRenderTime = 0;

function animateVrm(timestamp) {
  requestAnimationFrame(animateVrm);

  // Adaptive framerate iGPU:
  // idle/listening: 12 FPS
  // thinking: 18 FPS
  // speaking: 24 FPS
  let targetFps = 12;
  if (currentState === 'speaking') {
    targetFps = 24;
  } else if (currentState === 'thinking') {
    targetFps = 18;
  }
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
let isMouseDown = false;
let startX = 0;
let startY = 0;
let isDraggingNative = false;

function setupInteractions() {
  // Context Menu Hide on click outside
  window.addEventListener('click', (e) => {
    if (!contextMenu.contains(e.target)) {
      contextMenu.style.display = 'none';
    }
  });

  // Right Click: Popover Menu
  // Context Menu Trigger (Klik Kanan) -> Buka Menu Window Terpisah agar tidak terpotong
  window.addEventListener('contextmenu', (e) => {
    e.preventDefault();
    invoke('open_menu_window');
  });

  // Native Drag & Single Click
  root.addEventListener('mousedown', (e) => {
    if (e.button !== 0) return; // Only Left Click
    isMouseDown = true;
    isDraggingNative = false;
    startX = e.clientX;
    startY = e.clientY;
  });

  window.addEventListener('mousemove', (e) => {
    if (!isMouseDown) return;
    const dist = Math.hypot(e.clientX - startX, e.clientY - startY);
    if (dist > 5 && !isDraggingNative) {
      isDraggingNative = true;
      invoke('start_drag');
    }
  });

  window.addEventListener('mouseup', (e) => {
    if (!isMouseDown) return;
    isMouseDown = false;
    if (isDraggingNative) {
      isDraggingNative = false;
      invoke('save_current_position');
    }
    // Klik kiri murni khusus drag & interaksi canvas, tidak trigger aksi pause/apa pun
  });

  // Scroll Zoom Gesture
  let zoomTimer = null;
  window.addEventListener('wheel', (e) => {
    e.preventDefault();
    const delta = e.deltaY < 0 ? 15 : (e.deltaY > 0 ? -15 : 0);
    if (delta === 0) return;

    invoke('change_size_delta', { delta }).then(() => {
      if (currentVrm) fitCameraToVrm(currentVrm);
    });

    clearTimeout(zoomTimer);
    zoomTimer = setTimeout(() => {
      invoke('save_current_position');
    }, 300);
  }, { passive: false });
}

// -------------------------------------------------------------
// Menu Actions & IPC Polling
// -------------------------------------------------------------
const btnSkills = document.getElementById('btn-skills');
const skillsModal = document.getElementById('skills-modal');
const btnCloseSkills = document.getElementById('btn-close-skills');
const skillsList = document.getElementById('skills-list');
const btnAddSkill = document.getElementById('btn-add-skill');
const inputSkillId = document.getElementById('skill-input-id');
const inputSkillName = document.getElementById('skill-input-name');
const inputSkillDesc = document.getElementById('skill-input-desc');
const inputSkillCat = document.getElementById('skill-input-cat');
const inputSkillCmd = document.getElementById('skill-input-cmd');

let lastMode = '';

function setupMenuActions() {
  btnStart.addEventListener('click', () => {
    invoke('send_command', { cmd: 'start_chat' });
    contextMenu.style.display = 'none';
  });
  btnTestVoice.addEventListener('click', () => {
    invoke('send_command', { cmd: 'test_voice' });
    contextMenu.style.display = 'none';
  });
  btnTestMic.addEventListener('click', () => {
    invoke('send_command', { cmd: 'test_mic' });
    contextMenu.style.display = 'none';
  });
  btnPause.addEventListener('click', () => {
    invoke('send_command', { cmd: 'toggle_pause' });
    contextMenu.style.display = 'none';
  });
  btnStopChat.addEventListener('click', () => {
    invoke('send_command', { cmd: 'stop_chat' });
    contextMenu.style.display = 'none';
  });
  btnSkills.addEventListener('click', () => {
    contextMenu.style.display = 'none';
    invoke('open_skills_window');
  });

  btnCloseSkills.addEventListener('click', () => {
    skillsModal.style.display = 'none';
  });

  btnAddSkill.addEventListener('click', async () => {
    const id = inputSkillId.value.trim().toLowerCase().replace(/\s+/g, '_');
    const name = inputSkillName.value.trim();
    const desc = inputSkillDesc.value.trim();
    const cat = inputSkillCat.value.trim() || 'System';
    const cmd = inputSkillCmd.value.trim();

    if (!id || !name || !cmd) {
      alert('ID, Nama, dan Command wajib diisi!');
      return;
    }

    try {
      await invoke('save_skill', {
        skill: { id, name, description: desc, category: cat, command: cmd }
      });
      inputSkillId.value = '';
      inputSkillName.value = '';
      inputSkillDesc.value = '';
      inputSkillCmd.value = '';
      loadSkillsList();
      showToast('✅ Skill tersimpan!');
    } catch (e) {
      alert(`Gagal menyimpan skill: ${e}`);
    }
  });

  btnResetSize.addEventListener('click', () => {
    invoke('reset_size').then(() => {
      if (currentVrm) fitCameraToVrm(currentVrm);
    });
    contextMenu.style.display = 'none';
  });
  btnQuit.addEventListener('click', () => {
    invoke('send_command', { cmd: 'quit' });
    contextMenu.style.display = 'none';
  });
  cornerButtons.forEach((btn) => {
    btn.addEventListener('click', () => {
      const corner = btn.getAttribute('data-corner');
      invoke('set_corner', { corner }).then(() => {
        if (currentVrm) fitCameraToVrm(currentVrm);
      });
      contextMenu.style.display = 'none';
    });
  });
}

async function openSkillsModal() {
  skillsModal.style.display = 'flex';
  await loadSkillsList();
}

async function loadSkillsList() {
  try {
    const list = await invoke('get_skills');
    skillsList.innerHTML = '';
    if (!list || list.length === 0) {
      skillsList.innerHTML = '<div style="color: #888; font-size: 11px; text-align: center; padding: 20px;">Belum ada skill di skill-command.md</div>';
      return;
    }

    list.forEach((skill) => {
      const card = document.createElement('div');
      card.className = 'skill-card';
      card.innerHTML = `
        <div class="skill-info">
          <div class="skill-name">${escapeHtml(skill.name)} <span style="font-size: 9px; color: #f472b6; background: #2e2e42; padding: 1px 5px; border-radius: 3px; margin-left: 4px;">${escapeHtml(skill.category)}</span></div>
          <div class="skill-desc">${escapeHtml(skill.description)}</div>
          <div class="skill-cmd">${escapeHtml(skill.command)}</div>
        </div>
        <div class="skill-actions">
          <button class="action-btn run-btn" data-id="${escapeHtml(skill.id)}">▶ Run</button>
          <button class="action-btn del-btn" data-id="${escapeHtml(skill.id)}">🗑</button>
        </div>
      `;

      card.querySelector('.run-btn').addEventListener('click', async () => {
        try {
          await invoke('execute_skill', { id: skill.id });
          showToast(`⚡ ${skill.name} dijalankan!`);
        } catch (e) {
          alert(`Error: ${e}`);
        }
      });

      card.querySelector('.del-btn').addEventListener('click', async () => {
        if (confirm(`Hapus skill "${skill.name}"?`)) {
          await invoke('delete_skill', { id: skill.id });
          loadSkillsList();
          showToast(`🗑️ ${skill.name} dihapus`);
        }
      });

      skillsList.appendChild(card);
    });
  } catch (e) {
    console.error('Failed to load skills:', e);
  }
}

function escapeHtml(text) {
  const div = document.createElement('div');
  div.textContent = text || '';
  return div.innerHTML;
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
            const titleEl = document.querySelector('.menu-title');
            if (titleEl && titleEl.textContent !== `🎀 ${config.assistant_name} Assistant`) {
              titleEl.textContent = `🎀 ${config.assistant_name} Assistant`;
            }
            if (btnTestVoice && btnTestVoice.textContent !== `🔊 Tes Suara ${config.assistant_name}`) {
              btnTestVoice.textContent = `🔊 Tes Suara ${config.assistant_name}`;
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
        showToast('⏸️ Dijeda', 1200);
      } else {
        showToast('▶️ Dilanjutkan', 1200);
      }
    }
    window._lastPausedState = isPaused;

    const curMode = `${isInMainMenu}_${isPaused}`;
    if (lastMode !== curMode) {
      lastMode = curMode;
      if (isInMainMenu) {
        btnStart.style.display = 'block';
        btnTestVoice.style.display = 'block';
        btnTestMic.style.display = 'block';
        btnPause.style.display = 'none';
        btnStopChat.style.display = 'none';
        btnQuit.style.display = 'block';
      } else {
        btnStart.style.display = 'none';
        btnTestVoice.style.display = 'none';
        btnTestMic.style.display = 'none';
        btnPause.style.display = 'block';
        btnStopChat.style.display = 'block';
        btnQuit.style.display = 'none';

        btnPause.innerText = isPaused ? '▶️ Lanjutkan (Resume)' : '⏸️ Jeda (Pause)';
      }
    }
  } catch (err) {}
}

async function initApp() {
  let asstName = 'Niki';
  try {
    const config = await invoke('get_app_config');
    if (config && config.avatar_type) {
      avatarType = config.avatar_type.toLowerCase();
    }
    if (config && config.assistant_name) {
      asstName = config.assistant_name;
      const titleEl = document.querySelector('.menu-title');
      if (titleEl) titleEl.textContent = `🎀 ${asstName} Assistant`;
      if (btnTestVoice) btnTestVoice.textContent = `🔊 Tes Suara ${asstName}`;
    }
  } catch (e) {}

  isVrm = avatarType === 'vrm';

  if (isVrm) {
    if (imgElement) imgElement.style.display = 'none';
    if (canvasContainer) canvasContainer.style.display = 'flex';
    initVrmViewer();
  } else {
    if (imgElement) imgElement.style.display = 'block';
    if (canvasContainer) canvasContainer.style.display = 'none';
  }

  setupMenuActions();
  setupInteractions();
  await loadExternalPoses();
  // Dynamic visibility detection: skip rendering when window hidden
  document.addEventListener('visibilitychange', () => {
    if (!document.hidden) {
      lastRenderTime = performance.now();
    }
  });

  setInterval(pollState, 250);
}

initApp();
