import * as THREE from 'three';

/**
 * Modul Pose Interaktif VRM Dinamis berbasis `poses.json` Eksternal.
 * Mendukung 6 state (12 pose keyframe: A & B) dengan interpolasi slerp yang sangat halus.
 */

const targetQuats = {};
const currentQuats = {};
const tempEuler = new THREE.Euler();
const tempQuatA = new THREE.Quaternion();
const tempQuatB = new THREE.Quaternion();

let activePosesConfig = null;
let hasLoggedPoses = false;

export function setPosesConfig(config) {
  if (config && typeof config === 'object' && Object.keys(config).length > 0) {
    activePosesConfig = config;
    if (!hasLoggedPoses) {
      console.log('🎭 [VRM Pose] Loaded external poses.json configuration!');
      hasLoggedPoses = true;
    }
  }
}

export function getActivePosesConfig() {
  return activePosesConfig;
}

function setBoneTargetEuler(boneName, x, y, z) {
  tempEuler.set(x, y, z, 'XYZ');
  if (!targetQuats[boneName]) {
    targetQuats[boneName] = new THREE.Quaternion();
    currentQuats[boneName] = new THREE.Quaternion(0, 0, 0, 1);
  }
  targetQuats[boneName].setFromEuler(tempEuler);
}

// Helper untuk menekuk ruas jari tangan secara natural (hanya jika belum di-custom di poses.json)
function setFingersNaturalCurl(side, curlAmount, microWave = 0.0, configuredBones = null) {
  const fingers = ['Index', 'Middle', 'Ring', 'Little'];
  const joints = ['Proximal', 'Intermediate', 'Distal'];

  // Jempol (Thumb)
  const thumbMeta = `${side}ThumbMetacarpal`;
  const thumbProx = `${side}ThumbProximal`;
  const thumbDist = `${side}ThumbDistal`;

  if (!configuredBones || !configuredBones.has(thumbMeta)) {
    setBoneTargetEuler(thumbMeta, 0.12 * curlAmount, 0.12 * curlAmount + microWave * 0.5, -0.18 * curlAmount);
  }
  if (!configuredBones || !configuredBones.has(thumbProx)) {
    setBoneTargetEuler(thumbProx, 0.18 * curlAmount, 0.0, -0.22 * curlAmount);
  }
  if (!configuredBones || !configuredBones.has(thumbDist)) {
    setBoneTargetEuler(thumbDist, 0.22 * curlAmount, 0.0, -0.25 * curlAmount);
  }

  // 4 Jari lainnya
  fingers.forEach((finger, fIndex) => {
    const fingerDelay = microWave * (1.0 + fIndex * 0.3);
    const fFactor = 1.0 + (fIndex * 0.1);
    joints.forEach((joint, jIndex) => {
      const boneName = `${side}${finger}${joint}`;
      if (!configuredBones || !configuredBones.has(boneName)) {
        const zRot = -0.35 * (curlAmount + fingerDelay) * fFactor * (jIndex === 0 ? 1.0 : 1.25);
        setBoneTargetEuler(boneName, 0.0, 0.0, (side === 'left' ? zRot : -zRot));
      }
    });
  });
}

/**
 * Menerapkan pose dinamis dari poses.json dengan looping A ⇄ B halus
 */
function applyConfiguredStatePose(stateName, elapsedTime) {
  if (!activePosesConfig || !activePosesConfig[stateName]) {
    return false;
  }

  const stateData = activePosesConfig[stateName];
  const poseA = stateData.pose_a || {};
  const poseB = stateData.pose_b || {};
  const speed = stateData.speed || 1.2;

  // Nilai interpolasi sinusoidal 0.0 s/d 1.0
  const t = (Math.sin(elapsedTime * speed * 2.2) + 1.0) / 2.0;

  // Dapatkan seluruh bone dari kedua pose
  const allBones = new Set([...Object.keys(poseA), ...Object.keys(poseB)]);

  allBones.forEach(boneName => {
    const rotA = poseA[boneName] || [0, 0, 0];
    const rotB = poseB[boneName] || [0, 0, 0];

    // Ayunan mikronafas lembut (organic breathing micro-movement)
    let breathOffset = 0.0;
    if (stateName === 'idle') {
      if (boneName === 'chest' || boneName === 'spine') {
        breathOffset = Math.sin(elapsedTime * 2.4) * 0.015;
      } else if (boneName === 'leftShoulder' || boneName === 'rightShoulder') {
        breathOffset = Math.sin(elapsedTime * 2.4) * 0.01;
      }
    }

    tempEuler.set(rotA[0] + breathOffset, rotA[1], rotA[2], 'XYZ');
    tempQuatA.setFromEuler(tempEuler);

    tempEuler.set(rotB[0] + breathOffset, rotB[1], rotB[2], 'XYZ');
    tempQuatB.setFromEuler(tempEuler);

    if (!targetQuats[boneName]) {
      targetQuats[boneName] = new THREE.Quaternion();
      currentQuats[boneName] = new THREE.Quaternion(0, 0, 0, 1);
    }

    targetQuats[boneName].copy(tempQuatA).slerp(tempQuatB, t);

    // Otomatis cascade tekukan ruas jari tengah & distal jika jari proksimal di-custom di poses.json
    if (boneName.includes('Proximal')) {
      const eulerCurrent = new THREE.Euler().setFromQuaternion(targetQuats[boneName], 'XYZ');
      const side = boneName.startsWith('left') ? 'left' : 'right';
      const finger = boneName.replace(side, '').replace('Proximal', '');

      const midBone = `${side}${finger}Intermediate`;
      const distBone = `${side}${finger}Distal`;

      if (!allBones.has(midBone)) {
        setBoneTargetEuler(midBone, eulerCurrent.x * 0.7, eulerCurrent.y * 0.7, eulerCurrent.z * 0.85);
      }
      if (!allBones.has(distBone)) {
        setBoneTargetEuler(distBone, eulerCurrent.x * 0.5, eulerCurrent.y * 0.5, eulerCurrent.z * 0.7);
      }
    } else if (boneName.includes('ThumbProximal')) {
      const eulerCurrent = new THREE.Euler().setFromQuaternion(targetQuats[boneName], 'XYZ');
      const side = boneName.startsWith('left') ? 'left' : 'right';
      const distThumb = `${side}ThumbDistal`;
      if (!allBones.has(distThumb)) {
        setBoneTargetEuler(distThumb, eulerCurrent.x * 0.75, eulerCurrent.y * 0.75, eulerCurrent.z * 0.85);
      }
    }
  });

  // Tambahkan gestur jari hidup (hanya jika jari tersebut belum di-custom di poses.json)
  const fingerPulse = Math.sin(elapsedTime * 3.5) * 0.05;
  if (stateName === 'speaking') {
    setFingersNaturalCurl('left', 0.45 + fingerPulse, 0.0, allBones);
    setFingersNaturalCurl('right', 0.45 - fingerPulse, 0.0, allBones);
  } else if (stateName === 'listening') {
    setFingersNaturalCurl('left', 0.35, 0.0, allBones);
    setFingersNaturalCurl('right', 0.35, 0.0, allBones);
  } else if (stateName === 'thinking') {
    setFingersNaturalCurl('left', 0.2, 0.0, allBones);
    setFingersNaturalCurl('right', 0.6, 0.0, allBones);
  } else {
    setFingersNaturalCurl('left', 0.3 + fingerPulse * 0.5, 0.0, allBones);
    setFingersNaturalCurl('right', 0.3 - fingerPulse * 0.5, 0.0, allBones);
  }

  return true;
}

export function updateInteractivePose(vrm, state, emotion, deltaTime, elapsedTime, camera) {
  if (!vrm) return;

  // 1. Gimik Angin Lembut (Hanya saat speaking agar hemat daya iGPU)
  if (state === 'speaking' && vrm.springBoneManager && vrm.springBoneManager.externalForce) {
    const windBreeze = Math.sin(elapsedTime * 1.8) * 0.05 + Math.cos(elapsedTime * 2.5) * 0.035;
    const windGust = Math.sin(elapsedTime * 0.7) > 0.35 ? 0.06 : 0.0;
    vrm.springBoneManager.externalForce.set(
      windBreeze + windGust,
      Math.cos(elapsedTime * 2.0) * 0.02 - 0.025,
      Math.sin(elapsedTime * 1.5) * 0.04
    );
  }

  if (!vrm.humanoid) return;

  // 2. Bernapas Halus
  const breath = Math.sin(elapsedTime * 2.4) * 0.025;

  // 3. Pose Interaktif: Cek poses.json eksternal terlebih dahulu
  const mappedState = (state === 'processing') ? 'thinking' : (state || 'idle');
  const applied = applyConfiguredStatePose(mappedState, elapsedTime);

  // Jika belum ada config untuk state ini, gunakan fallback default
  if (!applied) {
    setBoneTargetEuler('head', 0.0, 0.0, Math.sin(elapsedTime * 1.5) * 0.05);
    setBoneTargetEuler('leftUpperArm', 0.2, 0.1, -1.15);
    setBoneTargetEuler('rightUpperArm', 0.2, -0.1, 1.15);
    setBoneTargetEuler('leftLowerArm', -0.3, -0.5, 0.2);
    setBoneTargetEuler('rightLowerArm', -0.3, 0.5, -0.2);
    setFingersNaturalCurl('left', 0.3);
    setFingersNaturalCurl('right', 0.3);
  }

  // 4. Terapkan Pose ke VRM Humanoid via Slerp Interpolation Organik & Mulus
  // Kecepatan slerp adaptif: transisi state lebih luwes tanpa patah / hentakan kaku
  const slerpSpeed = 1.0 - Math.exp(-deltaTime * 4.8);

  for (const [boneName, targetQ] of Object.entries(targetQuats)) {
    if (!currentQuats[boneName]) {
      currentQuats[boneName] = new THREE.Quaternion(0, 0, 0, 1);
    }
    currentQuats[boneName].slerp(targetQ, slerpSpeed);
    const q = currentQuats[boneName];

    if (vrm.humanoid) {
      try {
        const normNode = vrm.humanoid.getNormalizedBoneNode ? vrm.humanoid.getNormalizedBoneNode(boneName) : null;
        const rawNode = vrm.humanoid.getRawBoneNode ? vrm.humanoid.getRawBoneNode(boneName) : (vrm.humanoid.getBoneNode ? vrm.humanoid.getBoneNode(boneName) : null);

        if (normNode && normNode.quaternion) normNode.quaternion.copy(q);
        if (rawNode && rawNode.quaternion) rawNode.quaternion.copy(q);
      } catch (e) {}
    }
  }

  // 5. Tatapan Mata & Kepala Menghadap Kamera (LookAt)
  if (mappedState !== 'thinking' && vrm.lookAt && camera) {
    try {
      vrm.lookAt.target = camera;
      vrm.lookAt.update(deltaTime);
    } catch (e) {}
  }
}
