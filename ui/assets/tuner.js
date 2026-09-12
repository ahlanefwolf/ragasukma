import * as THREE from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import { OrbitControls } from 'three/addons/controls/OrbitControls.js';
import { VRMLoaderPlugin } from '@pixiv/three-vrm';

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
  console.warn('⚠️ [Tauri Bridge Tuner] invoke not ready for:', cmd);
  return null;
}

const BONES = [
  { id: 'head', label: 'Kepala (Head)' },
  { id: 'chest', label: 'Dada (Chest)' },
  { id: 'spine', label: 'Tulang Belakang (Spine)' },
  { id: 'hips', label: 'Panggul (Hips)' },
  { id: 'leftShoulder', label: 'Bahu Kiri (L-Shoulder)' },
  { id: 'leftUpperArm', label: 'Lengan Atas Kiri (L-UpperArm)' },
  { id: 'leftLowerArm', label: 'Lengan Bawah Kiri (L-LowerArm)' },
  { id: 'leftHand', label: 'Tangan Kiri (L-Hand)' },
  { id: 'rightShoulder', label: 'Bahu Kanan (R-Shoulder)' },
  { id: 'rightUpperArm', label: 'Lengan Atas Kanan (R-UpperArm)' },
  { id: 'rightLowerArm', label: 'Lengan Bawah Kanan (R-LowerArm)' },
  { id: 'rightHand', label: 'Tangan Kanan (R-Hand)' },
  { id: 'leftUpperLeg', label: 'Paha Kiri (L-UpperLeg)' },
  { id: 'leftLowerLeg', label: 'Betis Kiri (L-LowerLeg)' },
  { id: 'leftFoot', label: 'Kaki Kiri (L-Foot)' },
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
function showNotification(msg) {
  notifEl.textContent = msg;
  notifEl.classList.add('show');
  setTimeout(() => notifEl.classList.remove('show'), 2200);
}

// Three.js Scene Setup
const container = document.getElementById('viewport-container');
const scene = new THREE.Scene();
const camera = new THREE.PerspectiveCamera(28, container.clientWidth / container.clientHeight, 0.1, 50.0);
camera.position.set(0, 0.9, 2.6);

const renderer = new THREE.WebGLRenderer({ antialias: true, alpha: true, powerPreference: 'high-performance' });
renderer.setSize(container.clientWidth, container.clientHeight);
renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
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

async function loadVrmModel() {
  const loader = new GLTFLoader();
  loader.register((parser) => new VRMLoaderPlugin(parser));

  try {
    const bytes = await invoke('get_external_asset_bytes', { filename: 'char.vrm' });
    if (bytes && bytes.length > 0) {
      const u8 = (bytes instanceof Uint8Array) ? bytes : new Uint8Array(bytes);

      loader.parse(
        u8.buffer,
        '',
        (gltf) => {
          const vrm = gltf.userData.vrm;
          if (!vrm) return;
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

          const bbox = new THREE.Box3().setFromObject(vrm.scene);
          const center = bbox.getCenter(new THREE.Vector3());
          controls.target.copy(center);

          applyCurrentPose();
          showNotification('✅ Model VRM siap');
        },
        (err) => {
          console.error('Error parsing VRM buffer:', err);
          showNotification('⚠️ Gagal parse VRM buffer');
        }
      );
      return;
    }
  } catch(e) {
    console.warn('Gagal get_external_asset_bytes:', e);
  }

  loader.load(
    'assets/char.vrm',
    (gltf) => {
      const vrm = gltf.userData.vrm;
      if (!vrm) return;
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

      const bbox = new THREE.Box3().setFromObject(vrm.scene);
      const center = bbox.getCenter(new THREE.Vector3());
      controls.target.copy(center);

      applyCurrentPose();
      showNotification('✅ Model VRM siap');
    },
    undefined,
    (err) => {
      console.error('Error loading VRM:', err);
      showNotification('⚠️ Gagal load VRM, cek folder assets/char.vrm');
    }
  );
}

function generateBoneSliders() {
  const grid = document.getElementById('bone-grid');
  grid.innerHTML = '';

  const GROUPS = [
    { label: '🗣 Badan', bones: ['head', 'chest', 'spine', 'hips'] },
    { label: '💪 Lengan Kiri', bones: ['leftShoulder', 'leftUpperArm', 'leftLowerArm', 'leftHand'] },
    { label: '💪 Lengan Kanan', bones: ['rightShoulder', 'rightUpperArm', 'rightLowerArm', 'rightHand'] },
    { label: '🦵 Kaki Kiri', bones: ['leftUpperLeg', 'leftLowerLeg', 'leftFoot'] },
    { label: '🦵 Kaki Kanan', bones: ['rightUpperLeg', 'rightLowerLeg', 'rightFoot'] },
  ];

  const SHORT_LABEL = {
    head: 'Head', chest: 'Chest', spine: 'Spine', hips: 'Hips',
    leftShoulder: 'L-Shoulder', leftUpperArm: 'L-UpperArm', leftLowerArm: 'L-LowerArm', leftHand: 'L-Hand',
    rightShoulder: 'R-Shoulder', rightUpperArm: 'R-UpperArm', rightLowerArm: 'R-LowerArm', rightHand: 'R-Hand',
    leftUpperLeg: 'L-UpperLeg', leftLowerLeg: 'L-LowerLeg', leftFoot: 'L-Foot',
    rightUpperLeg: 'R-UpperLeg', rightLowerLeg: 'R-LowerLeg', rightFoot: 'R-Foot',
  };

  GROUPS.forEach(group => {
    const col = document.createElement('div');
    col.className = 'bone-col';

    const title = document.createElement('div');
    title.className = 'bone-col-title';
    title.textContent = group.label;
    col.appendChild(title);

    group.bones.forEach(boneId => {
      const card = document.createElement('div');
      card.className = 'bone-card';
      card.innerHTML = `
        <div class="bone-card-header">
          <span class="bone-card-label">${SHORT_LABEL[boneId] || boneId}</span>
          <button class="btn-reset-bone" data-bone="${boneId}" title="Reset semua axis ke 0">✕ reset all</button>
        </div>
        <div class="axis-row">
          <span class="axis-label axis-x">X</span>
          <input type="range" class="slider" data-bone="${boneId}" data-axis="0" min="-3.14" max="3.14" step="0.01" value="0">
          <span class="axis-val" id="val-${boneId}-0">0.00</span>
          <button class="btn-reset-axis" data-bone="${boneId}" data-axis="0" title="Reset X ke 0">↺</button>
        </div>
        <div class="axis-row">
          <span class="axis-label axis-y">Y</span>
          <input type="range" class="slider" data-bone="${boneId}" data-axis="1" min="-3.14" max="3.14" step="0.01" value="0">
          <span class="axis-val" id="val-${boneId}-1">0.00</span>
          <button class="btn-reset-axis" data-bone="${boneId}" data-axis="1" title="Reset Y ke 0">↺</button>
        </div>
        <div class="axis-row">
          <span class="axis-label axis-z">Z</span>
          <input type="range" class="slider" data-bone="${boneId}" data-axis="2" min="-3.14" max="3.14" step="0.01" value="0">
          <span class="axis-val" id="val-${boneId}-2">0.00</span>
          <button class="btn-reset-axis" data-bone="${boneId}" data-axis="2" title="Reset Z ke 0">↺</button>
        </div>
      `;

      card.querySelectorAll('.slider').forEach(slider => {
        slider.addEventListener('input', (e) => {
          const bId = e.target.dataset.bone;
          const axisIdx = parseInt(e.target.dataset.axis, 10);
          const val = parseFloat(e.target.value);
          document.getElementById(`val-${bId}-${axisIdx}`).textContent = val.toFixed(2);
          ensureBoneData(bId);
          posesConfig[currentState][currentPoseKey][bId][axisIdx] = val;
          if (!isPreviewing) applyCurrentPose();
        });
      });

      card.querySelectorAll('.btn-reset-axis').forEach(btn => {
        btn.addEventListener('click', () => {
          const bId = btn.dataset.bone;
          const axisIdx = parseInt(btn.dataset.axis, 10);
          ensureBoneData(bId);
          posesConfig[currentState][currentPoseKey][bId][axisIdx] = 0;
          const slider = card.querySelector(`.slider[data-bone="${bId}"][data-axis="${axisIdx}"]`);
          if (slider) slider.value = 0;
          document.getElementById(`val-${bId}-${axisIdx}`).textContent = '0.00';
          if (!isPreviewing) applyCurrentPose();
        });
      });

      card.querySelector('.btn-reset-bone').addEventListener('click', () => {
        ensureBoneData(boneId);
        posesConfig[currentState][currentPoseKey][boneId] = [0, 0, 0];
        updateSlidersUI();
        if (!isPreviewing) applyCurrentPose();
      });

      col.appendChild(card);
    });

    grid.appendChild(col);
  });
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
        label.textContent = parseFloat(vals[i]).toFixed(2);
      }
    }
  });
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
      showNotification('📂 poses.json berhasil dimuat');
    }
  } catch (e) {
    console.error('Failed to load poses.json:', e);
  }
}

async function savePosesToFile() {
  try {
    const jsonStr = JSON.stringify(posesConfig, null, 2);
    await invoke('save_poses_config', { configJson: jsonStr });
    showNotification('💾 Berhasil disimpan ke poses.json!');
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
    showNotification('📋 Pose A berhasil di-copy ke Pose B!');
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
    showNotification('📋 Pose B berhasil di-copy ke Pose A!');
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
    showNotification('🪞 Reverse A ➔ B berhasil diterapkan!');
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
    showNotification('🪞 Reverse B ➔ A berhasil diterapkan!');
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
    showNotification('✖ Semua bone di-reset ke 0');
  });

  window.addEventListener('resize', () => {
    camera.aspect = container.clientWidth / container.clientHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(container.clientWidth, container.clientHeight);
  });
}

const clock = new THREE.Clock();
function animate() {
  requestAnimationFrame(animate);
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

generateBoneSliders();
setupUIEvents();
loadVrmModel();
loadPosesFromFile();
animate();
