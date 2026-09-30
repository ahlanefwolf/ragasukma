import { Component, Show, createSignal, onMount, onCleanup } from 'solid-js';
import { Volume2, Mic, Brain, Sparkles, Loader2, RotateCcw, ZoomIn, ZoomOut, Maximize, Minimize } from 'lucide-solid';
import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { VRMLoaderPlugin, VRM, VRMUtils } from '@pixiv/three-vrm';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { API_BASE } from '../services/base';

interface AvatarViewerProps {
  state: 'idle' | 'listening' | 'speaking' | 'thinking';
  emotion?: string;
  isAudioStreaming?: boolean;
}

// Kamus pemetaan nama sendi badan dari poses.json ke VRM Humanoid Bone Standard Three.js
const BONE_NAME_FALLBACKS: Record<string, string[]> = {
  head: ['head'],
  neck: ['neck'],
  chest: ['chest', 'upperChest'],
  spine: ['spine'],
  hips: ['hips'],
  leftShoulder: ['leftShoulder'],
  leftUpperArm: ['leftUpperArm'],
  leftLowerArm: ['leftLowerArm'],
  leftHand: ['leftHand'],
  rightShoulder: ['rightShoulder'],
  rightUpperArm: ['rightUpperArm'],
  rightLowerArm: ['rightLowerArm'],
  rightHand: ['rightHand'],
  leftUpperLeg: ['leftUpperLeg'],
  leftLowerLeg: ['leftLowerLeg'],
  leftFoot: ['leftFoot'],
  rightUpperLeg: ['rightUpperLeg'],
  rightLowerLeg: ['rightLowerLeg'],
  rightFoot: ['rightFoot'],
  // Jari Tangan (VRM Humanoid Normalized Bones 1:1 Engine WGPU Desktop)
  leftThumb: ['leftThumbProximal', 'leftThumbMetacarpal'],
  leftIndex: ['leftIndexProximal'],
  leftMiddle: ['leftMiddleProximal'],
  leftRing: ['leftRingProximal'],
  leftLittle: ['leftLittleProximal'],
  rightThumb: ['rightThumbProximal', 'rightThumbMetacarpal'],
  rightIndex: ['rightIndexProximal'],
  rightMiddle: ['rightMiddleProximal'],
  rightRing: ['rightRingProximal'],
  rightLittle: ['rightLittleProximal'],
};

export const AvatarViewer: Component<AvatarViewerProps> = (props) => {
  let canvasContainerRef: HTMLDivElement | undefined;
  
  const [avatarType, setAvatarType] = createSignal<'vrm' | '2d'>('vrm');
  const [isLoading, setIsLoading] = createSignal(true);
  const [loadingProgress, setLoadingProgress] = createSignal<number>(0);
  const [isVrmLoaded, setIsVrmLoaded] = createSignal(false);
  const [vrmLoadError, setVrmLoadError] = createSignal(false);
  const [posesConfig, setPosesConfig] = createSignal<any>(null);
  const [isFullscreen, setIsFullscreen] = createSignal(false);

  let scene: THREE.Scene | null = null;
  let camera: THREE.PerspectiveCamera | null = null;
  let renderer: THREE.WebGLRenderer | null = null;
  let controls: OrbitControls | null = null;
  let currentVrm: VRM | null = null;
  let clock: THREE.Clock | null = null;
  let animFrameId: number | null = null;

  // Tracking Quaternions untuk Slerp Transisi Halus antar-State (1:1 persis pose_system.rs WGPU Desktop)
  const currentBoneQuats = new Map<string, THREE.Quaternion>();
  const stateFromQuats = new Map<string, THREE.Quaternion>();
  let stateTransitionT = 1.0;
  let lastAnimState = props.state || 'idle';
  let stateElapsedTime = 0;

  // Smoothing transisi ekspresi wajah & bibir
  const currentExpressionWeights: Record<string, number> = {};

  // Posisi kamera default: Paling minimize / zoom-out menampung full-body karakter dari kepala sampai kaki secara proporsional
  const defaultCamPos = new THREE.Vector3(0.0, 0.85, 3.4);
  const defaultTarget = new THREE.Vector3(0.0, 0.85, 0.0);

  const avatarImages: Record<string, string> = {
    idle: '/api/assets/file/char-avatar-idle.png',
    listening: '/api/assets/file/char-avatar-listening.png',
    speaking: '/api/assets/file/char-avatar-speaking.png',
    thinking: '/api/assets/file/char-avatar-thinking.png',
  };

  const getStatusBadge = () => {
    switch (props.state) {
      case 'speaking':
        return {
          icon: Volume2,
          label: 'Bicara',
          color: 'bg-pink-500/20 text-pink-400 border-pink-500/30',
          glow: 'drop-shadow-[0_0_20px_rgba(236,72,153,0.45)]',
        };
      case 'listening':
        return {
          icon: Mic,
          label: 'Mendengar',
          color: 'bg-sky-500/20 text-sky-400 border-sky-500/30',
          glow: 'drop-shadow-[0_0_20px_rgba(14,165,233,0.45)]',
        };
      case 'thinking':
        return {
          icon: Brain,
          label: 'Mikir',
          color: 'bg-amber-500/20 text-amber-400 border-amber-500/30',
          glow: 'drop-shadow-[0_0_20px_rgba(245,158,11,0.45)]',
        };
      default:
        return {
          icon: Sparkles,
          label: 'Standby',
          color: 'bg-theme-cyan/20 text-theme-cyan border-theme-cyan/30',
          glow: 'drop-shadow-[0_0_12px_rgba(34,211,238,0.25)]',
        };
    }
  };

  const resetCameraView = () => {
    if (camera && controls) {
      camera.position.copy(defaultCamPos);
      controls.target.copy(defaultTarget);
      controls.update();
    }
  };

  const zoomIn = () => {
    if (camera && controls) {
      const dir = new THREE.Vector3().subVectors(controls.target, camera.position).normalize();
      camera.position.addScaledVector(dir, 0.2);
      controls.update();
    }
  };

  const zoomOut = () => {
    if (camera && controls) {
      const dir = new THREE.Vector3().subVectors(camera.position, controls.target).normalize();
      camera.position.addScaledVector(dir, 0.25);
      controls.update();
    }
  };

  const toggleFullscreen = () => {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen?.().catch(() => {});
      setIsFullscreen(true);
    } else {
      document.exitFullscreen?.().catch(() => {});
      setIsFullscreen(false);
    }
  };

  // Helper resolusi node bone di Three-VRM
  const getNormalizedBone = (vrm: VRM, boneKey: string) => {
    if (!vrm.humanoid) return null;
    const candidates = BONE_NAME_FALLBACKS[boneKey] || [boneKey];
    for (const name of candidates) {
      const node = vrm.humanoid.getNormalizedBoneNode(name as any);
      if (node) return node;
    }
    return null;
  };


  const updateVrmBonePoses = (vrm: VRM, delta: number) => {
    if (!vrm.humanoid) return;
    const poses = posesConfig();
    if (!poses) return;

    const currentState = props.state || 'idle';
    if (currentState !== lastAnimState) {
      // Snapshot rotasi seluruh tulang & jari saat ini sebagai titik awal transisi slerp halus
      stateFromQuats.clear();
      for (const [k, q] of currentBoneQuats.entries()) {
        stateFromQuats.set(k, q.clone());
      }
      stateTransitionT = 0.0;
      stateElapsedTime = 0;
      lastAnimState = currentState;
    }
    stateElapsedTime += delta;

    // Timer transisi Slerp (durasi 0.45 detik identik dengan WGPU desktop)
    const transitionDuration = 0.45;
    if (stateTransitionT < 1.0) {
      stateTransitionT = Math.min(1.0, stateTransitionT + delta / transitionDuration);
    }
    // S-Curve cosine smoothing:
    const blendT = stateTransitionT < 1.0
      ? (1.0 - Math.cos(stateTransitionT * Math.PI)) / 2.0
      : 1.0;

    const stateObj = poses[currentState] || poses.idle;
    const keyframes = stateObj?.poses || [];
    const speed = Math.max(0.1, stateObj?.speed || 1.0);
    const count = keyframes.length;

    if (count === 0) return;

    // Natural Breathing ritme santai
    const breathSine = Math.sin(stateElapsedTime * 1.35);
    const breathChest = breathSine * 0.012;
    const breathSpine = breathSine * 0.006;
    const breathShoulder = breathSine * 0.006;

    const applyBoneRotation = (boneName: string, rx: number, ry: number, rz: number) => {
      const node = getNormalizedBone(vrm, boneName);
      if (!node) return;

      let offY = 0;
      if (boneName === 'chest') offY = breathChest;
      else if (boneName === 'spine') offY = breathSpine;
      else if (boneName === 'leftShoulder' || boneName === 'rightShoulder') offY = breathShoulder;

      const targetEuler = new THREE.Euler(rx + offY, ry, rz, 'XYZ');
      const targetQ = new THREE.Quaternion().setFromEuler(targetEuler);

      let finalQ: THREE.Quaternion;
      if (blendT < 1.0) {
        const fromQ = stateFromQuats.get(boneName) || targetQ;
        finalQ = fromQ.clone().slerp(targetQ, blendT);
      } else {
        finalQ = targetQ;
      }

      currentBoneQuats.set(boneName, finalQ.clone());
      node.quaternion.copy(finalQ);
    };

    if (count === 1) {
      const singleBones = keyframes[0].bones || {};
      for (const [boneName, rot] of Object.entries(singleBones)) {
        if (!Array.isArray(rot) || rot.length < 3) continue;
        applyBoneRotation(boneName, rot[0] as number, rot[1] as number, rot[2] as number);
      }
    } else {
      // Multi-keyframes cycle Catmull-Rom kontinuitas C1
      const cycleStep = 2.5 / speed;
      const totalCycleTime = count * cycleStep;
      const loopTime = stateElapsedTime % totalCycleTime;
      const curIdx = Math.floor(loopTime / cycleStep) % count;
      const nextIdx = (curIdx + 1) % count;
      const prevIdx = curIdx === 0 ? count - 1 : curIdx - 1;
      const next2Idx = (curIdx + 2) % count;
      const t = (loopTime % cycleStep) / cycleStep;

      const p0 = keyframes[prevIdx]?.bones || {};
      const p1 = keyframes[curIdx]?.bones || {};
      const p2 = keyframes[nextIdx]?.bones || {};
      const p3 = keyframes[next2Idx]?.bones || {};

      const allBoneKeys = new Set([...Object.keys(p1), ...Object.keys(p2)]);
      
      const catmullRom = (v0: number, v1: number, v2: number, v3: number, tVal: number) => {
        const t2 = tVal * tVal;
        const t3 = t2 * tVal;
        return 0.5 * (
          2.0 * v1 +
          (-v0 + v2) * tVal +
          (2.0 * v0 - 5.0 * v1 + 4.0 * v2 - v3) * t2 +
          (-v0 + 3.0 * v1 - 3.0 * v2 + v3) * t3
        );
      };

      for (const boneName of allBoneKeys) {
        const r0 = p0[boneName] || [0, 0, 0];
        const r1 = p1[boneName] || [0, 0, 0];
        const r2 = p2[boneName] || [0, 0, 0];
        const r3 = p3[boneName] || [0, 0, 0];

        const rx = catmullRom(r0[0], r1[0], r2[0], r3[0], t);
        const ry = catmullRom(r0[1], r1[1], r2[1], r3[1], t);
        const rz = catmullRom(r0[2], r1[2], r2[2], r3[2], t);

        applyBoneRotation(boneName, rx, ry, rz);
      }
    }
  };

  let resizeObserver: ResizeObserver | null = null;

  const cleanupThree = () => {
    if (animFrameId !== null) {
      cancelAnimationFrame(animFrameId);
      animFrameId = null;
    }
    if (resizeObserver) {
      resizeObserver.disconnect();
      resizeObserver = null;
    }
    if (controls) {
      controls.dispose();
      controls = null;
    }
    if (currentVrm) {
      try {
        VRMUtils.deepDispose(currentVrm.scene);
      } catch (_) {}
      currentVrm = null;
    }
    if (scene) {
      scene.clear();
      scene = null;
    }
    if (renderer) {
      try {
        renderer.dispose();
        renderer.forceContextLoss();
        if (renderer.domElement && renderer.domElement.parentNode) {
          renderer.domElement.parentNode.removeChild(renderer.domElement);
        }
      } catch (_) {}
      renderer = null;
    }
  };

  const initThreeVrm = () => {
    if (!canvasContainerRef) return;
    cleanupThree();

    const width = canvasContainerRef.clientWidth || 280;
    const height = canvasContainerRef.clientHeight || 240;

    scene = new THREE.Scene();

    camera = new THREE.PerspectiveCamera(26, width / height, 0.1, 20.0);
    // Posisi kamera default menatap wajah karakter dari arah depan
    camera.position.copy(defaultCamPos);
    camera.lookAt(defaultTarget);

    renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true, powerPreference: 'low-power' });
    renderer.setSize(width, height);
    renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    renderer.outputColorSpace = THREE.SRGBColorSpace;
    canvasContainerRef.innerHTML = '';
    canvasContainerRef.appendChild(renderer.domElement);

    if (typeof ResizeObserver !== 'undefined' && canvasContainerRef) {
      resizeObserver = new ResizeObserver((entries) => {
        for (const entry of entries) {
          const w = Math.floor(entry.contentRect.width);
          const h = Math.floor(entry.contentRect.height);
          if (w > 0 && h > 0 && camera && renderer) {
            camera.aspect = w / h;
            camera.updateProjectionMatrix();
            renderer.setSize(w, h);
          }
        }
      });
      resizeObserver.observe(canvasContainerRef);
    }

    // Pasang OrbitControls untuk gesture Pinch-to-Zoom dan Pan/Rotate di HP
    controls = new OrbitControls(camera, renderer.domElement);
    controls.target.copy(defaultTarget);
    controls.enableDamping = true;
    controls.dampingFactor = 0.08;
    controls.minDistance = 0.35; // zoom in leluasa dekat wajah
    controls.maxDistance = 6.5;  // zoom out jauh leluasa melihat karakter dari kejauhan
    controls.enableRotate = true;
    controls.enableZoom = true;
    controls.enablePan = true;
    controls.screenSpacePanning = true; // Bisa digeser naik-turun leluasa di layar HP
    controls.touches = {
      ONE: THREE.TOUCH.ROTATE,
      TWO: THREE.TOUCH.DOLLY_PAN,
    };

    const ambientLight = new THREE.AmbientLight(0xffffff, 1.8);
    scene.add(ambientLight);
    const dirLight = new THREE.DirectionalLight(0xffffff, 1.5);
    dirLight.position.set(0.0, 2.0, 2.0);
    scene.add(dirLight);

    clock = new THREE.Clock();

    const loader = new GLTFLoader();
    loader.register((parser) => new VRMLoaderPlugin(parser));

    setIsLoading(true);

    loader.load(
      `${API_BASE}/api/assets/file/char.vrm`,
      (gltf) => {
        const vrm = gltf.userData.vrm as VRM;
        if (!vrm) {
          console.warn('VRM user data not found on gltf');
          setVrmLoadError(true);
          setIsLoading(false);
          return;
        }
        try {
          VRMUtils.removeUnnecessaryVertices(gltf.scene);
          VRMUtils.removeUnnecessaryJoints(gltf.scene);
        } catch (_) {}
        
        // Model VRM Three.js default menghadap ke depan (+Z arah kamera)
        vrm.scene.rotation.y = 0.0;

        scene?.add(vrm.scene);
        currentVrm = vrm;
        setIsVrmLoaded(true);
        setIsLoading(false);
      },
      (progress) => {
        if (progress.total > 0) {
          const percent = Math.round((progress.loaded / progress.total) * 100);
          setLoadingProgress(percent);
        }
      },
      (err) => {
        console.warn('Gagal memuat model VRM di HP:', err);
        setVrmLoadError(true);
        setIsLoading(false);
      }
    );

    const animate = () => {
      animFrameId = requestAnimationFrame(animate);
      const delta = clock?.getDelta() || 0.016;

      if (controls) {
        controls.update();
      }

      if (currentVrm) {
        // 1. Hitung & aplikasikan rotasi seluruh tulang humanoid TERLEBIH DAHULU
        updateVrmBonePoses(currentVrm, delta);

        // 2. Ekspresi wajah & lip-sync vokal 100% konsisten dengan standar VRM 1.0 Three.js & engine desktop
        const expressions = currentVrm.expressionManager;
        if (expressions) {
          const currentState = props.state || 'idle';
          // Sinkronisasi dengan app.rs desktop: saat state Thinking, ekspresi otomatis override ke thinking
          const currentEmotion = currentState === 'thinking' ? 'thinking' : (props.emotion || 'neutral').toLowerCase();

          const allKnownKeys = [
            'aa', 'ih', 'ou', 'ee', 'oh',
            'blink', 'blinkLeft', 'blinkRight',
            'happy', 'angry', 'sad', 'surprised', 'relaxed'
          ];
          const targetWeights: Record<string, number> = {};
          for (const k of allKnownKeys) {
            targetWeights[k] = 0.0;
          }

          let hasStrongEmotion = false;

          // 2a. Layer Emosi Utama (Identik 100% dengan resolve_emotion_layers di expressions.rs)
          switch (currentEmotion) {
            case 'happy':
            case 'senang':
              targetWeights['happy'] = 0.85;
              hasStrongEmotion = true;
              break;

            case 'shy':
            case 'malu':
              targetWeights['happy'] = 0.40;
              targetWeights['blink'] = 0.30;
              hasStrongEmotion = true;
              break;

            case 'excited':
            case 'semangat':
              targetWeights['happy'] = 0.85;
              targetWeights['surprised'] = 0.25;
              hasStrongEmotion = true;
              break;

            case 'surprised':
            case 'kaget':
              targetWeights['surprised'] = 0.85;
              hasStrongEmotion = true;
              break;

            case 'thinking':
            case 'mikir':
              targetWeights['ou'] = 0.35;
              targetWeights['sad'] = 0.20;
              hasStrongEmotion = true;
              break;

            case 'sad':
            case 'sedih':
              targetWeights['sad'] = 0.85;
              hasStrongEmotion = true;
              break;

            case 'angry':
            case 'marah':
              targetWeights['angry'] = 0.85;
              hasStrongEmotion = true;
              break;

            case 'neutral':
            case 'netral':
              break;

            default:
              if (allKnownKeys.includes(currentEmotion)) {
                targetWeights[currentEmotion] = 0.75;
                hasStrongEmotion = true;
              }
              break;
          }

          // 2b. Natural Eye Blinking santai (Identik dengan expressions.rs)
          let naturalBlinkWeight = 0.0;
          const blinkPeriod = currentState === 'speaking' ? 6.2 : 5.2;
          const blinkPhase = stateElapsedTime % blinkPeriod;
          const blinkDuration = 0.24; // Durasi kedip ~240ms
          if (blinkPhase < blinkDuration) {
            const t = blinkPhase / blinkDuration;
            const blinkDamp = hasStrongEmotion ? 0.6 : 1.0;
            naturalBlinkWeight = Math.pow(Math.sin(t * Math.PI), 1.5) * blinkDamp;
          }
          if (naturalBlinkWeight > 0.01) {
            targetWeights['blink'] = Math.min(1.0, (targetWeights['blink'] || 0) + naturalBlinkWeight);
          }

          // 2c. Lip Sync Vokal Vowel (A, I, U, E, O) saat Speaking (Identik dengan expressions.rs)
          if (currentState === 'speaking') {
            const vowels = ['aa', 'ih', 'ou', 'ee', 'oh'];
            const vowelIdx = Math.floor(stateElapsedTime * 3.0) % vowels.length;
            const targetVowel = vowels[vowelIdx];

            const vowelMouthOpen = Math.pow(Math.abs(Math.sin(stateElapsedTime * 5.5)), 1.4) * 0.60;
            if (vowelMouthOpen > 0.04) {
              targetWeights[targetVowel] = vowelMouthOpen;
            }
          }

          // Damping interpolasi halus (~100ms) untuk ekspresi & mulut agar tidak patah-patah
          // Catatan: 'blink' diterapkan langsung mengikuti kurva sinus mulus tanpa lag dampening, agar mata menutup 100% sempurna persis seperti desktop
          const keysToUpdate = new Set([...allKnownKeys, ...Object.keys(currentExpressionWeights)]);
          for (const k of keysToUpdate) {
            if (k === 'blink') {
              currentExpressionWeights[k] = targetWeights[k] || 0.0;
            } else {
              const curVal = currentExpressionWeights[k] || 0.0;
              const tgtVal = targetWeights[k] || 0.0;
              const newVal = THREE.MathUtils.damp(curVal, tgtVal, 14.0, delta);
              currentExpressionWeights[k] = Math.abs(newVal) < 0.001 ? 0.0 : newVal;
            }
            expressions.setValue(k, currentExpressionWeights[k]);
          }

          expressions.update();
        }

        // 3. BARU sinkronkan normalized bones ke raw mesh model & spring bones
        currentVrm.update(delta);
      }

      if (renderer && scene && camera) {
        renderer.render(scene, camera);
      }
    };

    animate();
  };

  onMount(() => {
    // 1. Langsung inisialisasi 3D canvas tanpa menunggu network fetch
    setTimeout(() => {
      initThreeVrm();
    }, 50);

    // 2. Muat poses.json secara paralel di latar belakang
    fetch(`${API_BASE}/api/poses`)
      .then((res) => (res.ok ? res.json() : null))
      .then((p) => {
        if (p) setPosesConfig(p);
      })
      .catch(() => {});

    // 3. Cek tipe avatar dari config secara paralel
    fetch(`${API_BASE}/api/config`)
      .then((res) => (res.ok ? res.json() : null))
      .then((cfg) => {
        if (cfg && cfg.avatar_type === '2d') {
          setAvatarType('2d');
          setIsLoading(false);
        }
      })
      .catch(() => {});

    // Sinkronkan status fullscreen dengan event browser
    const onFullscreenChange = () => {
      setIsFullscreen(!!document.fullscreenElement);
    };
    document.addEventListener('fullscreenchange', onFullscreenChange);
    onCleanup(() => {
      document.removeEventListener('fullscreenchange', onFullscreenChange);
    });
  });

  onCleanup(() => {
    cleanupThree();
  });

  return (
    <div class="relative w-full h-full flex items-center justify-center overflow-hidden select-none touch-none">
      {/* Toolbar Kontrol Kamera Vertikal Sejajar dengan Pengaturan (Tanpa Box, Warna Coklat Tema) */}
      <Show when={avatarType() === 'vrm' && isVrmLoaded()}>
        <div class="absolute top-14 right-4 z-20 flex flex-col items-center gap-0.5 pointer-events-auto select-none">
          <button
            type="button"
            onClick={zoomIn}
            class="p-2 text-theme-muted hover:text-theme-text active:scale-85 transition-transform drop-shadow-sm cursor-pointer"
            title="Perbesar"
          >
            <ZoomIn size={22} class="stroke-[2.2]" />
          </button>
          <button
            type="button"
            onClick={zoomOut}
            class="p-2 text-theme-muted hover:text-theme-text active:scale-85 transition-transform drop-shadow-sm cursor-pointer"
            title="Perkecil"
          >
            <ZoomOut size={22} class="stroke-[2.2]" />
          </button>
          {/* Tombol Fullscreen Tepat di Atas Reset Kamera */}
          <button
            type="button"
            onClick={toggleFullscreen}
            class="p-2 text-theme-muted hover:text-theme-text active:scale-85 transition-transform drop-shadow-sm cursor-pointer"
            title={isFullscreen() ? 'Keluar Fullscreen' : 'Layar Penuh'}
          >
            <Show when={isFullscreen()} fallback={<Maximize size={20} class="stroke-[2.2]" />}>
              <Minimize size={20} class="stroke-[2.2]" />
            </Show>
          </button>
          <button
            type="button"
            onClick={resetCameraView}
            class="p-2 text-theme-muted hover:text-theme-text active:scale-85 transition-transform drop-shadow-sm cursor-pointer"
            title="Reset Sudut Kamera"
          >
            <RotateCcw size={20} class="stroke-[2.2]" />
          </button>
        </div>
      </Show>

      {/* Loading Overlay Saat Model VRM Sedang Diunduh */}
      <Show when={isLoading() && avatarType() === 'vrm'}>
        <div class="absolute inset-0 z-30 bg-theme-bg/85 backdrop-blur-md flex flex-col items-center justify-center gap-3 text-theme-cyan">
          <Loader2 size={28} class="animate-spin text-theme-cyan" />
          <span class="text-xs font-mono font-semibold tracking-wide">
            {loadingProgress() > 0 ? `Memuat Karakter 3D (${loadingProgress()}%)` : 'Menyiapkan 3D Engine...'}
          </span>
        </div>
      </Show>

      {/* Kontainer 3D Canvas WebGL (Mengisi Penuh Layar) */}
      <div
        ref={canvasContainerRef}
        class="w-full h-full relative z-10 flex items-center justify-center touch-none"
        style={{ display: avatarType() === 'vrm' && !vrmLoadError() ? 'flex' : 'none' }}
      />

      {/* Fallback 2D Sprite jika mode 2D atau jika ada error fatal loading VRM */}
      <Show when={avatarType() === '2d' || vrmLoadError()}>
        <div class="relative w-full h-full flex flex-col items-center justify-center p-6">
          <img
            src={avatarImages[props.state] || avatarImages.idle}
            alt="Avatar"
            class={`max-h-[70vh] max-w-full object-contain transition-all duration-300 ${getStatusBadge().glow}`}
          />
          <Show when={vrmLoadError()}>
            <button
              type="button"
              onClick={() => {
                setVrmLoadError(false);
                setIsLoading(true);
                initThreeVrm();
              }}
              class="mt-3 px-4 py-2 bg-theme-cyan/20 border border-theme-cyan/40 text-theme-cyan text-xs font-mono rounded-xl flex items-center gap-1.5 shadow-md active:scale-95"
            >
              <RotateCcw size={12} />
              <span>Coba Muat Ulang 3D</span>
            </button>
          </Show>
        </div>
      </Show>
    </div>
  );
};
