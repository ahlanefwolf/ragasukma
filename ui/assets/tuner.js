import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { VRMLoaderPlugin } from '@pixiv/three-vrm';

function getTauriInvoke() {
  if (window.__TAURI__?.core?.invoke) return window.__TAURI__.core.invoke;
  if (window.__TAURI__?.invoke) return window.__TAURI__.invoke;
  if (window.__TAURI_INTERNALS__?.invoke) return window.__TAURI_INTERNALS__.invoke;
  if (window.parent && window.parent !== window) {
    if (window.parent.__TAURI__?.core?.invoke) return window.parent.__TAURI__.core.invoke;
    if (window.parent.__TAURI__?.invoke) return window.parent.__TAURI__.invoke;
    if (window.parent.__TAURI_INTERNALS__?.invoke) return window.parent.__TAURI_INTERNALS__.invoke;
  }
  if (window.top && window.top !== window) {
    if (window.top.__TAURI__?.core?.invoke) return window.top.__TAURI__.core.invoke;
    if (window.top.__TAURI__?.invoke) return window.top.__TAURI__.invoke;
    if (window.top.__TAURI_INTERNALS__?.invoke) return window.top.__TAURI_INTERNALS__.invoke;
  }
  return null;
}

async function invoke(cmd, args = {}) {
  let inv = getTauriInvoke();
  if (inv) {
    try {
      return await inv(cmd, args);
    } catch (e) {
      console.error('[Invoke Error Tuner]', cmd, e);
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
        console.error('[Invoke Error Tuner]', cmd, e);
        return null;
      }
    }
  }
  console.warn('⚠️ [Tauri Bridge Tuner] invoke not ready for:', cmd);
  return null;
}

const BONES = [
  // Badan
  { id: 'head', label: 'Kepala (Head)' },
  { id: 'chest', label: 'Dada (Chest)' },
  { id: 'spine', label: 'Tulang Belakang (Spine)' },
  { id: 'hips', label: 'Panggul (Hips)' },

  // Lengan Kiri
  { id: 'leftShoulder', label: 'Bahu Kiri (L-Shoulder)' },
  { id: 'leftUpperArm', label: 'Lengan Atas Kiri (L-UpperArm)' },
  { id: 'leftLowerArm', label: 'Lengan Bawah Kiri (L-LowerArm)' },
  { id: 'leftHand', label: 'Tangan Kiri (L-Hand)' },

  // Jari Tangan Kiri (Simple 5 Jari Utama)
  { id: 'leftThumbProximal', label: 'Jempol Kiri (L-Thumb)' },
  { id: 'leftIndexProximal', label: 'Telunjuk Kiri (L-Index)' },
  { id: 'leftMiddleProximal', label: 'Jari Tengah Kiri (L-Middle)' },
  { id: 'leftRingProximal', label: 'Jari Manis Kiri (L-Ring)' },
  { id: 'leftLittleProximal', label: 'Kelingking Kiri (L-Pinky)' },

  // Lengan Kanan
  { id: 'rightShoulder', label: 'Bahu Kanan (R-Shoulder)' },
  { id: 'rightUpperArm', label: 'Lengan Atas Kanan (R-UpperArm)' },
  { id: 'rightLowerArm', label: 'Lengan Bawah Kanan (R-LowerArm)' },
  { id: 'rightHand', label: 'Tangan Kanan (R-Hand)' },

  // Jari Tangan Kanan (Simple 5 Jari Utama)
  { id: 'rightThumbProximal', label: 'Jempol Kanan (R-Thumb)' },
  { id: 'rightIndexProximal', label: 'Telunjuk Kanan (R-Index)' },
  { id: 'rightMiddleProximal', label: 'Jari Tengah Kanan (R-Middle)' },
  { id: 'rightRingProximal', label: 'Jari Manis Kanan (R-Ring)' },
  { id: 'rightLittleProximal', label: 'Kelingking Kanan (R-Pinky)' },

  // Kaki Kiri
  { id: 'leftUpperLeg', label: 'Paha Kiri (L-UpperLeg)' },
  { id: 'leftLowerLeg', label: 'Betis Kiri (L-LowerLeg)' },
  { id: 'leftFoot', label: 'Kaki Kiri (L-Foot)' },

  // Kaki Kanan
  { id: 'rightUpperLeg', label: 'Paha Kanan (R-UpperLeg)' },
  { id: 'rightLowerLeg', label: 'Betis Kanan (R-LowerLeg)' },
  { id: 'rightFoot', label: 'Kaki Kanan (R-Foot)' }
];

let posesConfig = {};
let currentState = 'idle';
let currentPoseKey = 'pose_a';
let isPreviewing = false;
let previewTime = 0;

const notifEl = document.getElementById('notification');
const loaderEl = document.getElementById('tuner-loader');

function hideLoader() {
  if (loaderEl) {
    loaderEl.classList.add('hidden');
    setTimeout(() => {
      loaderEl.style.display = 'none';
    }, 400);
  }
}

function showNotification(msg) {
  notifEl.textContent = msg;
  notifEl.classList.add('show');
  setTimeout(() => notifEl.classList.remove('show'), 2200);
}

// Three.js Scene Setup
const container = document.getElementById('viewport-container');
const scene = new THREE.Scene();

const initWidth = container.clientWidth || (window.innerWidth * 0.52) || 450;
const initHeight = container.clientHeight || (window.innerHeight - 45) || 600;

const camera = new THREE.PerspectiveCamera(28, initWidth / Math.max(initHeight, 1), 0.1, 50.0);
camera.position.set(0, 0.9, 2.6);

const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true, powerPreference: 'high-performance' });
renderer.setSize(initWidth, initHeight);
renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
renderer.outputColorSpace = THREE.SRGBColorSpace;
container.appendChild(renderer.domElement);

const controls = new OrbitControls(camera, renderer.domElement);
controls.enableDamping = true;
controls.dampingFactor = 0.08;
controls.enablePan = true;
controls.panSpeed = 1.0;
controls.screenSpacePanning = true;
controls.mouseButtons = {
  LEFT: THREE.MOUSE.ROTATE,
  MIDDLE: THREE.MOUSE.DOLLY,
  RIGHT: THREE.MOUSE.PAN
};
controls.minDistance = 0.3;
controls.maxDistance = 8.0;
controls.target.set(0, 0.85, 0);

function resizeRenderer() {
  const w = container.clientWidth || (window.innerWidth * 0.52) || 450;
  const h = container.clientHeight || (window.innerHeight - 45) || 600;
  if (w <= 0 || h <= 0) return;
  camera.aspect = w / h;
  camera.updateProjectionMatrix();
  renderer.setSize(w, h);
  renderer.setPixelRatio(Math.min(window.devicePixelRatio || 1, 2));
}

const resizeObserver = new ResizeObserver(() => {
  resizeRenderer();
  if (currentVrm) {
    renderer.render(scene, camera);
  }
});
resizeObserver.observe(container);

// Pencahayaan natural
const frontLight = new THREE.DirectionalLight(0xfff5ee, 1.1);
frontLight.position.set(0.0, 1.2, 2.5).normalize();
scene.add(frontLight);

const fillLight = new THREE.DirectionalLight(0xe8f4ff, 0.6);
fillLight.position.set(-1.5, 0.8, 1.0).normalize();
scene.add(fillLight);

const ambient = new THREE.AmbientLight(0xffffff, 0.65);
scene.add(ambient);

let currentVrm = null;

function fitCameraToVrm(vrm) {
  if (!vrm || !vrm.scene) return;
  resizeRenderer();

  const bbox = new THREE.Box3().setFromObject(vrm.scene);
  const center = bbox.getCenter(new THREE.Vector3());
  const size = bbox.getSize(new THREE.Vector3());

  const targetY = center.y || 0.85;
  const targetHeight = Math.max(size.y * 1.15, 1.25);
  const targetWidth = Math.max(size.x * 1.25, 0.65);

  const w = container.clientWidth || (window.innerWidth * 0.45) || 400;
  const h = container.clientHeight || (window.innerHeight - 45) || 600;
  const aspect = w / Math.max(h, 1);
  const fovRad = camera.fov * (Math.PI / 180);

  const distanceV = (targetHeight / 2) / Math.tan(fovRad / 2);
  const distanceH = (targetWidth / 2) / (Math.tan(fovRad / 2) * aspect);
  const cameraZ = Math.max(distanceV, distanceH) * 1.08;

  camera.position.set(0, targetY, cameraZ);
  controls.target.set(0, targetY, 0);
  camera.lookAt(0, targetY, 0);
  controls.update();
}

function handleLoadedVrm(gltf) {
  const vrm = gltf.userData.vrm;
  if (!vrm) {
    console.error('userData.vrm tidak ditemukan pada gltf');
    showNotification('Format VRM tidak sesuai');
    return;
  }
  currentVrm = vrm;
  scene.add(vrm.scene);
  vrm.scene.rotation.y = 0;

  vrm.scene.traverse((obj) => {
    if (obj.isMesh && obj.material) {
      const mats = Array.isArray(obj.material) ? obj.material : [obj.material];
      mats.forEach((m) => {
        m.side = THREE.DoubleSide;
        if (m.isMeshStandardMaterial) {
          m.roughness = Math.min(m.roughness, 0.8);
          m.metalness = 0.0;
        }
      });
    }
  });

  fitCameraToVrm(vrm);
  applyCurrentPose();
  hideLoader();
  showNotification('Model VRM siap');
}

async function loadVrmModel() {
  const loader = new GLTFLoader();
  loader.register((parser) => new VRMLoaderPlugin(parser));

  try {
    let bytes = null;
    if (window.parent && typeof window.parent.getCachedVrmBytes === 'function') {
      bytes = window.parent.getCachedVrmBytes();
    }
    if (!bytes || bytes.length === 0) {
      bytes = await invoke('get_external_asset_bytes', { filename: 'char.vrm' });
    }

    if (bytes && bytes.length > 0) {
      const u8 = (bytes instanceof Uint8Array) ? bytes : new Uint8Array(bytes);
      const arrayBuffer = u8.buffer.slice(u8.byteOffset, u8.byteOffset + u8.byteLength);

      loader.parse(
        arrayBuffer,
        '',
        (gltf) => {
          handleLoadedVrm(gltf);
        },
        (err) => {
          console.error('Error parsing VRM buffer in tuner:', err);
          showNotification('Gagal parse buffer VRM');
        }
      );
      return;
    }

    // Fallback data URI jika byte array gagal
    const dataUri = await invoke('get_external_asset', { filename: 'char.vrm' });
    if (dataUri) {
      loader.load(
        dataUri,
        (gltf) => {
          handleLoadedVrm(gltf);
        },
        undefined,
        (err) => {
          console.error('Error loading VRM via Data URI:', err);
          showNotification('Gagal memuat VRM dari Data URI');
        }
      );
      return;
    }
  } catch (e) {
    console.warn('Gagal memuat model VRM di Tuner:', e);
  }

  hideLoader();
  showNotification('File char.vrm tidak ditemukan');
}

let currentBoneCategory = 'body';

const BONE_CATEGORIES = [
  { id: 'body', label: 'Badan' },
  { id: 'leftArm', label: 'Lengan Kiri' },
  { id: 'leftFingers', label: 'Jari Kiri' },
  { id: 'rightArm', label: 'Lengan Kanan' },
  { id: 'rightFingers', label: 'Jari Kanan' },
  { id: 'legs', label: 'Kaki' },
];

const BONE_CATEGORY_MAP = {
  body: ['head', 'chest', 'spine', 'hips'],
  leftArm: ['leftShoulder', 'leftUpperArm', 'leftLowerArm', 'leftHand'],
  leftFingers: [
    'leftThumbProximal',
    'leftIndexProximal',
    'leftMiddleProximal',
    'leftRingProximal',
    'leftLittleProximal'
  ],
  rightArm: ['rightShoulder', 'rightUpperArm', 'rightLowerArm', 'rightHand'],
  rightFingers: [
    'rightThumbProximal',
    'rightIndexProximal',
    'rightMiddleProximal',
    'rightRingProximal',
    'rightLittleProximal'
  ],
  legs: [
    'leftUpperLeg', 'leftLowerLeg', 'leftFoot',
    'rightUpperLeg', 'rightLowerLeg', 'rightFoot'
  ]
};

const SHORT_LABEL = {
  head: 'Kepala (Head)', chest: 'Dada (Chest)', spine: 'Punggung (Spine)', hips: 'Pinggul (Hips)',
  leftShoulder: 'Bahu Kiri (L-Shoulder)', leftUpperArm: 'Lengan Atas Kiri (L-UpperArm)', leftLowerArm: 'Lengan Bawah Kiri (L-LowerArm)', leftHand: 'Telapak Tangan Kiri (L-Hand)',
  leftThumbProximal: 'Jempol Kiri (L-Thumb)',
  leftIndexProximal: 'Telunjuk Kiri (L-Index)',
  leftMiddleProximal: 'Jari Tengah Kiri (L-Middle)',
  leftRingProximal: 'Jari Manis Kiri (L-Ring)',
  leftLittleProximal: 'Kelingking Kiri (L-Pinky)',

  rightShoulder: 'Bahu Kanan (R-Shoulder)', rightUpperArm: 'Lengan Atas Kanan (R-UpperArm)', rightLowerArm: 'Lengan Bawah Kanan (R-LowerArm)', rightHand: 'Telapak Tangan Kanan (R-Hand)',
  rightThumbProximal: 'Jempol Kanan (R-Thumb)',
  rightIndexProximal: 'Telunjuk Kanan (R-Index)',
  rightMiddleProximal: 'Jari Tengah Kanan (R-Middle)',
  rightRingProximal: 'Jari Manis Kanan (R-Ring)',
  rightLittleProximal: 'Kelingking Kanan (R-Pinky)',

  leftUpperLeg: 'Paha Kiri (L-UpperLeg)', leftLowerLeg: 'Betis Kiri (L-LowerLeg)', leftFoot: 'Kaki Kiri (L-Foot)',
  rightUpperLeg: 'Paha Kanan (R-UpperLeg)', rightLowerLeg: 'Betis Kanan (R-LowerLeg)', rightFoot: 'Kaki Kanan (R-Foot)',
};

function renderCategoryTabs() {
  const catBar = document.getElementById('bone-cat-bar');
  if (!catBar) return;
  catBar.innerHTML = '';

  BONE_CATEGORIES.forEach(cat => {
    const btn = document.createElement('button');
    btn.className = `cat-tab ${cat.id === currentBoneCategory ? 'active' : ''}`;
    btn.textContent = cat.label;
    btn.addEventListener('click', () => {
      currentBoneCategory = cat.id;
      renderCategoryTabs();
      generateBoneSliders();
    });
    catBar.appendChild(btn);
  });
}

function generateBoneSliders() {
  const grid = document.getElementById('bone-grid');
  if (!grid) return;
  grid.innerHTML = '';

  let activeBoneIds = BONE_CATEGORY_MAP[currentBoneCategory] || BONE_CATEGORY_MAP.body;

  activeBoneIds.forEach(boneId => {
    const card = document.createElement('div');
    card.className = 'bone-card';
    card.innerHTML = `
      <div class="bone-card-header">
        <span class="bone-card-label">${SHORT_LABEL[boneId] || boneId}</span>
        <button class="btn-reset-bone" data-bone="${boneId}" title="Reset bone ke 0">
          <svg viewBox="0 0 24 24" class="icon-tiny"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>
          <span>Reset</span>
        </button>
      </div>
      <div class="axis-row">
        <span class="axis-label axis-x" data-bone="${boneId}" data-axis="0" title="Klik untuk Reset X ke 0">X</span>
        <input type="range" class="slider" data-bone="${boneId}" data-axis="0" min="-3.14" max="3.14" step="0.01" value="0">
        <input type="number" class="axis-val-input" id="val-${boneId}-0" data-bone="${boneId}" data-axis="0" min="-3.14" max="3.14" step="0.01" value="0.00">
        <button class="btn-reset-axis" data-bone="${boneId}" data-axis="0" title="Reset X ke 0">
          <svg viewBox="0 0 24 24" class="icon-tiny"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>
        </button>
      </div>
      <div class="axis-row">
        <span class="axis-label axis-y" data-bone="${boneId}" data-axis="1" title="Klik untuk Reset Y ke 0">Y</span>
        <input type="range" class="slider" data-bone="${boneId}" data-axis="1" min="-3.14" max="3.14" step="0.01" value="0">
        <input type="number" class="axis-val-input" id="val-${boneId}-1" data-bone="${boneId}" data-axis="1" min="-3.14" max="3.14" step="0.01" value="0.00">
        <button class="btn-reset-axis" data-bone="${boneId}" data-axis="1" title="Reset Y ke 0">
          <svg viewBox="0 0 24 24" class="icon-tiny"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>
        </button>
      </div>
      <div class="axis-row">
        <span class="axis-label axis-z" data-bone="${boneId}" data-axis="2" title="Klik untuk Reset Z ke 0">Z</span>
        <input type="range" class="slider" data-bone="${boneId}" data-axis="2" min="-3.14" max="3.14" step="0.01" value="0">
        <input type="number" class="axis-val-input" id="val-${boneId}-2" data-bone="${boneId}" data-axis="2" min="-3.14" max="3.14" step="0.01" value="0.00">
        <button class="btn-reset-axis" data-bone="${boneId}" data-axis="2" title="Reset Z ke 0">
          <svg viewBox="0 0 24 24" class="icon-tiny"><path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/></svg>
        </button>
      </div>
    `;

    card.querySelectorAll('.slider').forEach(slider => {
      slider.addEventListener('input', (e) => {
        const bId = e.target.dataset.bone;
        const axisIdx = parseInt(e.target.dataset.axis, 10);
        const val = parseFloat(e.target.value) || 0;
        const numInput = document.getElementById(`val-${bId}-${axisIdx}`);
        if (numInput) numInput.value = val.toFixed(2);
        ensureBoneData(bId);
        posesConfig[currentState][currentPoseKey][bId][axisIdx] = val;
        if (!isPreviewing) applyCurrentPose();
      });
    });

    card.querySelectorAll('.axis-val-input').forEach(input => {
      input.addEventListener('input', (e) => {
        const bId = e.target.dataset.bone;
        const axisIdx = parseInt(e.target.dataset.axis, 10);
        let val = parseFloat(e.target.value);
        if (isNaN(val)) return;
        val = Math.max(-3.14, Math.min(3.14, val));
        const slider = card.querySelector(`.slider[data-bone="${bId}"][data-axis="${axisIdx}"]`);
        if (slider) slider.value = val;
        ensureBoneData(bId);
        posesConfig[currentState][currentPoseKey][bId][axisIdx] = val;
        if (!isPreviewing) applyCurrentPose();
      });
      input.addEventListener('blur', (e) => {
        let val = parseFloat(e.target.value);
        if (isNaN(val)) val = 0;
        val = Math.max(-3.14, Math.min(3.14, val));
        e.target.value = val.toFixed(2);
      });
    });

    const resetAxisHandler = (targetEl) => {
      const bId = targetEl.dataset.bone;
      const axisIdx = parseInt(targetEl.dataset.axis, 10);
      ensureBoneData(bId);
      posesConfig[currentState][currentPoseKey][bId][axisIdx] = 0;
      const slider = card.querySelector(`.slider[data-bone="${bId}"][data-axis="${axisIdx}"]`);
      if (slider) slider.value = 0;
      const numInput = document.getElementById(`val-${bId}-${axisIdx}`);
      if (numInput) numInput.value = '0.00';
      if (!isPreviewing) applyCurrentPose();
    };

    card.querySelectorAll('.btn-reset-axis').forEach(btn => {
      btn.addEventListener('click', () => resetAxisHandler(btn));
    });

    card.querySelectorAll('.axis-label').forEach(lbl => {
      lbl.addEventListener('click', () => resetAxisHandler(lbl));
    });

    card.querySelector('.btn-reset-bone').addEventListener('click', () => {
      ensureBoneData(boneId);
      posesConfig[currentState][currentPoseKey][boneId] = [0, 0, 0];
      updateSlidersUI();
      if (!isPreviewing) applyCurrentPose();
    });

    grid.appendChild(card);
  });

  updateSlidersUI();
}

function ensureBoneData(boneId) {
  if (!posesConfig[currentState]) {
    posesConfig[currentState] = { speed: 1.2, emotion: 'neutral', pose_a: {}, pose_b: {} };
  }
  if (!posesConfig[currentState][currentPoseKey]) {
    posesConfig[currentState][currentPoseKey] = {};
  }
  if (boneId && !posesConfig[currentState][currentPoseKey][boneId]) {
    posesConfig[currentState][currentPoseKey][boneId] = [0, 0, 0];
  }
}

function updateSlidersUI() {
  const stateData = posesConfig[currentState] || {};
  const poseData = stateData[currentPoseKey] || {};

  document.getElementById('state-select').value = currentState;
  document.getElementById('emotion-select').value = stateData.emotion || 'neutral';
  document.getElementById('speed-slider').value = stateData.speed || 1.2;
  document.getElementById('speed-val').textContent = `${(stateData.speed || 1.2).toFixed(1)}x`;

  BONES.forEach(bone => {
    const vals = poseData[bone.id] || [0, 0, 0];
    for (let i = 0; i < 3; i++) {
      const slider = document.querySelector(`.slider[data-bone="${bone.id}"][data-axis="${i}"]`);
      const label = document.getElementById(`val-${bone.id}-${i}`);
      if (slider && label) {
        slider.value = vals[i];
        if ('value' in label) {
          label.value = parseFloat(vals[i] || 0).toFixed(2);
        } else {
          label.textContent = parseFloat(vals[i] || 0).toFixed(2);
        }
      }
    }
  });
}

function applyFingerJointsCascade(humanoid, boneId, euler) {
  if (boneId.includes('Thumb')) {
    const side = boneId.startsWith('left') ? 'left' : 'right';
    const dist = humanoid.getNormalizedBoneNode(`${side}ThumbDistal`);
    if (dist) {
      const distEuler = new THREE.Euler(euler.x * 0.75, euler.y * 0.75, euler.z * 0.85, 'XYZ');
      dist.quaternion.setFromEuler(distEuler);
    }
  } else if (boneId.includes('Index') || boneId.includes('Middle') || boneId.includes('Ring') || boneId.includes('Little')) {
    const side = boneId.startsWith('left') ? 'left' : 'right';
    const fingerName = boneId.replace(side, '').replace('Proximal', '');
    const mid = humanoid.getNormalizedBoneNode(`${side}${fingerName}Intermediate`);
    const dist = humanoid.getNormalizedBoneNode(`${side}${fingerName}Distal`);
    if (mid) {
      const midEuler = new THREE.Euler(euler.x * 0.7, euler.y * 0.7, euler.z * 0.85, 'XYZ');
      mid.quaternion.setFromEuler(midEuler);
    }
    if (dist) {
      const distEuler = new THREE.Euler(euler.x * 0.5, euler.y * 0.5, euler.z * 0.7, 'XYZ');
      dist.quaternion.setFromEuler(distEuler);
    }
  }
}

function applyCurrentPose() {
  if (!currentVrm || !currentVrm.humanoid) return;

  const stateData = posesConfig[currentState] || {};
  const poseData = stateData[currentPoseKey] || {};

  const euler = new THREE.Euler();
  BONES.forEach(b => {
    const vals = poseData[b.id] || [0, 0, 0];
    euler.set(vals[0], vals[1], vals[2], 'XYZ');
    const node = currentVrm.humanoid.getNormalizedBoneNode(b.id);
    if (node) {
      node.quaternion.setFromEuler(euler);
      if (b.id.includes('Proximal')) {
        applyFingerJointsCascade(currentVrm.humanoid, b.id, euler);
      }
    }
  });

  if (currentVrm.expressionManager) {
    const emo = stateData.emotion || 'neutral';
    ['happy', 'joy', 'relaxed', 'surprised', 'sad', 'sorrow'].forEach(exp => {
      currentVrm.expressionManager.setValue(exp, 0.0);
    });
    if (emo === 'happy') {
      currentVrm.expressionManager.setValue('happy', 0.95);
      currentVrm.expressionManager.setValue('joy', 0.95);
    } else if (emo === 'shy') {
      currentVrm.expressionManager.setValue('relaxed', 0.8);
      currentVrm.expressionManager.setValue('happy', 0.4);
    } else if (emo === 'excited') {
      currentVrm.expressionManager.setValue('happy', 0.85);
      currentVrm.expressionManager.setValue('joy', 0.85);
      currentVrm.expressionManager.setValue('surprised', 0.4);
    } else if (emo === 'surprised') {
      currentVrm.expressionManager.setValue('surprised', 0.95);
    } else if (emo === 'thinking') {
      currentVrm.expressionManager.setValue('relaxed', 0.5);
      currentVrm.expressionManager.setValue('surprised', 0.25);
    } else if (emo === 'confused') {
      currentVrm.expressionManager.setValue('surprised', 0.4);
      currentVrm.expressionManager.setValue('sad', 0.35);
    } else if (emo === 'sad') {
      currentVrm.expressionManager.setValue('sad', 0.9);
      currentVrm.expressionManager.setValue('sorrow', 0.9);
    } else {
      currentVrm.expressionManager.setValue('relaxed', 0.25);
    }
  }
}

// Dynamic Slerp Preview A ⇄ B
const quatA = new THREE.Quaternion();
const quatB = new THREE.Quaternion();
const slerpQuat = new THREE.Quaternion();
const eulerTemp = new THREE.Euler();

function updatePreviewAnimation(delta) {
  if (!isPreviewing || !currentVrm || !currentVrm.humanoid) return;

  const stateData = posesConfig[currentState] || {};
  const poseA = stateData.pose_a || {};
  const poseB = stateData.pose_b || {};
  const speed = stateData.speed || 1.2;

  previewTime += delta * speed * 2.2;
  const t = (Math.sin(previewTime) + 1.0) / 2.0;

  BONES.forEach(b => {
    const vA = poseA[b.id] || [0, 0, 0];
    const vB = poseB[b.id] || [0, 0, 0];

    eulerTemp.set(vA[0], vA[1], vA[2], 'XYZ');
    quatA.setFromEuler(eulerTemp);

    eulerTemp.set(vB[0], vB[1], vB[2], 'XYZ');
    quatB.setFromEuler(eulerTemp);

    slerpQuat.copy(quatA).slerp(quatB, t);

    const node = currentVrm.humanoid.getNormalizedBoneNode(b.id);
    if (node) {
      node.quaternion.copy(slerpQuat);
      if (b.id.includes('Proximal')) {
        const currentEuler = new THREE.Euler().setFromQuaternion(slerpQuat, 'XYZ');
        applyFingerJointsCascade(currentVrm.humanoid, b.id, currentEuler);
      }
    }
  });
}

async function loadPosesFromFile() {
  try {
    const data = await invoke('get_poses_config');
    if (data && Object.keys(data).length > 0) {
      posesConfig = data;
      updateSlidersUI();
      if (!isPreviewing) applyCurrentPose();
      showNotification('poses.json berhasil dimuat');
    }
  } catch (e) {
    console.error('Failed to load poses.json:', e);
  }
}

async function savePosesToFile() {
  try {
    const jsonStr = JSON.stringify(posesConfig, null, 2);
    await invoke('save_poses_config', { configJson: jsonStr });
    showNotification('Berhasil disimpan ke poses.json');
  } catch (e) {
    alert('Gagal menyimpan poses.json: ' + e);
  }
}

function mirrorPose(srcPose) {
  const mirrored = {};
  const mirrorMap = {
    'leftShoulder': 'rightShoulder',
    'rightShoulder': 'leftShoulder',
    'leftUpperArm': 'rightUpperArm',
    'rightUpperArm': 'leftUpperArm',
    'leftLowerArm': 'rightLowerArm',
    'rightLowerArm': 'leftLowerArm',
    'leftHand': 'rightHand',
    'rightHand': 'leftHand',

    'leftThumbProximal': 'rightThumbProximal',
    'rightThumbProximal': 'leftThumbProximal',
    'leftIndexProximal': 'rightIndexProximal',
    'rightIndexProximal': 'leftIndexProximal',
    'leftMiddleProximal': 'rightMiddleProximal',
    'rightMiddleProximal': 'leftMiddleProximal',
    'leftRingProximal': 'rightRingProximal',
    'rightRingProximal': 'leftRingProximal',
    'leftLittleProximal': 'rightLittleProximal',
    'rightLittleProximal': 'leftLittleProximal',

    'leftUpperLeg': 'rightUpperLeg',
    'rightUpperLeg': 'leftUpperLeg',
    'leftLowerLeg': 'rightLowerLeg',
    'rightLowerLeg': 'leftLowerLeg',
    'leftFoot': 'rightFoot',
    'rightFoot': 'leftFoot',
  };

  for (const [bone, rot] of Object.entries(srcPose)) {
    const targetBone = mirrorMap[bone] || bone;
    const [rx, ry, rz] = rot || [0, 0, 0];
    mirrored[targetBone] = [
      parseFloat(rx.toFixed(3)),
      parseFloat((-ry).toFixed(3)),
      parseFloat((-rz).toFixed(3))
    ];
  }
  return mirrored;
}

function setupUIEvents() {
  document.getElementById('state-select').addEventListener('change', (e) => {
    currentState = e.target.value;
    updateSlidersUI();
    if (!isPreviewing) applyCurrentPose();
  });

  document.getElementById('emotion-select').addEventListener('change', (e) => {
    if (!posesConfig[currentState]) posesConfig[currentState] = {};
    posesConfig[currentState].emotion = e.target.value;
    if (!isPreviewing) applyCurrentPose();
  });

  document.getElementById('speed-slider').addEventListener('input', (e) => {
    const val = parseFloat(e.target.value);
    if (!posesConfig[currentState]) posesConfig[currentState] = {};
    posesConfig[currentState].speed = val;
    document.getElementById('speed-val').textContent = `${val.toFixed(1)}x`;
  });

  const tabA = document.getElementById('tab-pose-a');
  const tabB = document.getElementById('tab-pose-b');
  tabA.addEventListener('click', () => {
    currentPoseKey = 'pose_a';
    tabA.classList.add('active');
    tabB.classList.remove('active');
    updateSlidersUI();
    if (!isPreviewing) applyCurrentPose();
  });
  tabB.addEventListener('click', () => {
    currentPoseKey = 'pose_b';
    tabB.classList.add('active');
    tabA.classList.remove('active');
    updateSlidersUI();
    if (!isPreviewing) applyCurrentPose();
  });

  document.getElementById('btn-copy-a-to-b').addEventListener('click', () => {
    ensureBoneData();
    const stateData = posesConfig[currentState];
    stateData.pose_b = JSON.parse(JSON.stringify(stateData.pose_a || {}));
    currentPoseKey = 'pose_b';
    tabB.classList.add('active');
    tabA.classList.remove('active');
    updateSlidersUI();
    if (!isPreviewing) applyCurrentPose();
    showNotification('Pose A berhasil di-copy ke Pose B');
  });

  document.getElementById('btn-copy-b-to-a').addEventListener('click', () => {
    ensureBoneData();
    const stateData = posesConfig[currentState];
    stateData.pose_a = JSON.parse(JSON.stringify(stateData.pose_b || {}));
    currentPoseKey = 'pose_a';
    tabA.classList.add('active');
    tabB.classList.remove('active');
    updateSlidersUI();
    if (!isPreviewing) applyCurrentPose();
    showNotification('Pose B berhasil di-copy ke Pose A');
  });

  document.getElementById('btn-reverse-a-to-b').addEventListener('click', () => {
    ensureBoneData();
    const stateData = posesConfig[currentState];
    stateData.pose_b = mirrorPose(stateData.pose_a || {});
    currentPoseKey = 'pose_b';
    tabB.classList.add('active');
    tabA.classList.remove('active');
    updateSlidersUI();
    if (!isPreviewing) applyCurrentPose();
    showNotification('Mirror A ke B berhasil diterapkan');
  });

  document.getElementById('btn-reverse-b-to-a').addEventListener('click', () => {
    ensureBoneData();
    const stateData = posesConfig[currentState];
    stateData.pose_a = mirrorPose(stateData.pose_b || {});
    currentPoseKey = 'pose_a';
    tabA.classList.add('active');
    tabB.classList.remove('active');
    updateSlidersUI();
    if (!isPreviewing) applyCurrentPose();
    showNotification('Mirror B ke A berhasil diterapkan');
  });

  const btnPreview = document.getElementById('btn-preview');
  btnPreview.addEventListener('click', () => {
    isPreviewing = !isPreviewing;
    const span = btnPreview.querySelector('span');
    if (isPreviewing) {
      if (span) span.textContent = 'Stop';
      btnPreview.classList.add('active-green');
      previewTime = 0;
    } else {
      if (span) span.textContent = 'Preview';
      btnPreview.classList.remove('active-green');
      applyCurrentPose();
    }
  });

  document.getElementById('btn-reload').addEventListener('click', loadPosesFromFile);
  document.getElementById('btn-save').addEventListener('click', savePosesToFile);

  document.getElementById('btn-reset-all').addEventListener('click', () => {
    ensureBoneData();
    const pose = posesConfig[currentState][currentPoseKey];
    BONES.forEach(b => { pose[b.id] = [0, 0, 0]; });
    updateSlidersUI();
    if (!isPreviewing) applyCurrentPose();
    showNotification('Semua bone di-reset ke 0');
  });

  window.addEventListener('resize', () => {
    resizeRenderer();
  });
}

const clock = new THREE.Clock();
function animate() {
  requestAnimationFrame(animate);
  if (document.hidden) return; // Skip rendering WebGL jika window tuner sedang tidak ditampilkan
  const delta = clock.getDelta();
  controls.update();

  if (currentVrm) {
    currentVrm.update(delta);
  }

  if (isPreviewing) {
    updatePreviewAnimation(delta);
  }

  renderer.render(scene, camera);
}

renderCategoryTabs();
generateBoneSliders();
setupUIEvents();

let isTunerVrmLoaded = false;
function ensureTunerLoaded() {
  if (isTunerVrmLoaded) return;
  isTunerVrmLoaded = true;
  loadVrmModel();
  loadPosesFromFile();
}

if (!document.hidden) {
  ensureTunerLoaded();
}

document.addEventListener('visibilitychange', () => {
  if (!document.hidden) {
    ensureTunerLoaded();
  }
});

window.addEventListener('focus', () => {
  if (!document.hidden) {
    ensureTunerLoaded();
  }
});

animate();
