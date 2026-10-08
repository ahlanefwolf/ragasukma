import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { VRMLoaderPlugin, VRM, VRMUtils } from '@pixiv/three-vrm';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { DEFAULT_CAM_POS, DEFAULT_TARGET } from './constants';
import { AvatarState, FpsLimit, PosesConfig, RenderQuality } from './types';
import { VrmPoseAnimator } from './poseAnimator';
import { VrmExpressionAnimator } from './expressionAnimator';
import { WebXRArManager, ArStateListener } from './arManager';
import { API_BASE } from '../../services/base';

export interface VrmSceneCallbacks {
  onLoadStart?: () => void;
  onProgress?: (percent: number) => void;
  onLoaded?: () => void;
  onError?: (err: unknown) => void;
}

export class VrmSceneManager {
  private container: HTMLDivElement | null = null;
  private scene: THREE.Scene | null = null;
  private camera: THREE.PerspectiveCamera | null = null;
  private renderer: THREE.WebGLRenderer | null = null;
  private controls: OrbitControls | null = null;
  private currentVrm: VRM | null = null;
  private vrmAnchor = new THREE.Group();
  private contactShadow: THREE.Mesh | null = null;
  private clock: THREE.Clock | null = null;
  private resizeObserver: ResizeObserver | null = null;

  private poseAnimator = new VrmPoseAnimator();
  private expressionAnimator = new VrmExpressionAnimator();
  private arManager = new WebXRArManager();
  private currentQuality: RenderQuality = 'hd';
  private currentFpsLimit: FpsLimit = 60; // Default 60 FPS (adem & stabil untuk baterai smartphone)
  private lastRenderTimestamp = 0;

  constructor() {
    this.arManager.setOnEndCallback(() => {
      this.resetToNonAr();
    });
  }

  public getArManager(): WebXRArManager {
    return this.arManager;
  }

  public subscribeAr(listener: ArStateListener): () => void {
    return this.arManager.subscribe(listener);
  }

  private createRealShadowPlane(): THREE.Mesh {
    const geometry = new THREE.PlaneGeometry(3.0, 3.0);
    geometry.rotateX(-Math.PI / 2);

    const material = new THREE.ShadowMaterial({
      opacity: 0.3,
      depthWrite: false,
    });

    const mesh = new THREE.Mesh(geometry, material);
    mesh.position.y = 0.001;
    mesh.receiveShadow = true;
    return mesh;
  }

  public init(
    container: HTMLDivElement,
    getState: () => { state: AvatarState; emotion?: string; poses: PosesConfig | null },
    callbacks: VrmSceneCallbacks = {}
  ): void {
    this.cleanup();
    this.container = container;

    const width = container.clientWidth || 280;
    const height = container.clientHeight || 240;

    this.scene = new THREE.Scene();

    this.camera = new THREE.PerspectiveCamera(26, width / height, 0.1, 20.0);
    this.camera.position.copy(DEFAULT_CAM_POS);
    this.camera.lookAt(DEFAULT_TARGET);

    this.renderer = new THREE.WebGLRenderer({
      alpha: true,
      antialias: false,
      powerPreference: 'default',
      stencil: false,
      depth: true,
    });
    this.renderer.shadowMap.enabled = false;
    this.renderer.setSize(width, height);
    this.renderer.setPixelRatio(this.getPixelRatioForQuality(this.currentQuality));
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;

    // Linear tone mapping bawaan standar VRM agar saturasi dan rona warna kulit/baju tetap bersih, konsisten & tidak berbayang kotor
    this.renderer.toneMapping = THREE.LinearToneMapping;
    this.renderer.toneMappingExposure = 1.0;

    container.innerHTML = '';
    container.appendChild(this.renderer.domElement);

    this.vrmAnchor.position.set(0, 0, 0);
    this.vrmAnchor.rotation.set(0, 0, 0);
    this.vrmAnchor.visible = true;

    this.scene.add(this.vrmAnchor);
    this.scene.add(this.arManager.getReticle());

    if (typeof ResizeObserver !== 'undefined') {
      this.resizeObserver = new ResizeObserver((entries) => {
        for (const entry of entries) {
          const w = Math.floor(entry.contentRect.width);
          const h = Math.floor(entry.contentRect.height);
          if (w > 0 && h > 0 && this.camera && this.renderer) {
            this.camera.aspect = w / h;
            this.camera.updateProjectionMatrix();
            this.renderer.setSize(w, h);
          }
        }
      });
      this.resizeObserver.observe(container);
    }

    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.target.copy(DEFAULT_TARGET);
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.08;
    this.controls.minDistance = 0.35;
    this.controls.maxDistance = 6.5;
    this.controls.enableRotate = true;
    this.controls.enableZoom = true;
    this.controls.enablePan = true;
    this.controls.screenSpacePanning = true;
    this.controls.touches = {
      ONE: THREE.TOUCH.ROTATE,
      TWO: THREE.TOUCH.DOLLY_PAN,
    };

    // Lighting Setup Standar MToon / VRM (1:1 Identik Bersih dengan Native UI):
    // 1. Ambient Light yang cukup terang agar base color MToon merata di seluruh sudut
    const ambientLight = new THREE.AmbientLight(0xffffff, 1.3);
    this.scene.add(ambientLight);

    // 2. Main Key Light dipasang langsung sebagai anak kamera (Camera Child Light)
    // Dengan begitu arah cahaya 100% terkunci dari depan kamera ke model tanpa pernah bergeser sudutnya!
    const light = new THREE.DirectionalLight(0xffffff, 0.9);
    light.position.set(0.0, 0.2, 1.0);
    this.camera.add(light);
    this.scene.add(this.camera);

    this.clock = new THREE.Clock();

    const loader = new GLTFLoader();
    loader.register((parser) => new VRMLoaderPlugin(parser));

    callbacks.onLoadStart?.();

    loader.load(
      `${API_BASE}/api/assets/file/char.vrm`,
      (gltf) => {
        const vrm = gltf.userData.vrm as VRM;
        if (!vrm) {
          console.warn('VRM user data not found on gltf');
          callbacks.onError?.(new Error('VRM user data not found'));
          return;
        }
        try {
          VRMUtils.removeUnnecessaryVertices(gltf.scene);
          VRMUtils.removeUnnecessaryJoints(gltf.scene);
        } catch (_) {}

        // VRM Spring Bone (Fisika dinamis rambut, rok, dan aksesoris)
        if (vrm.springBoneManager) {
          try {
            vrm.springBoneManager.reset();
          } catch (_) {}
        }

        // VRM 0.X compatibility: Rotate VRM0 models 180 degrees so they face the camera (+Z)
        VRMUtils.rotateVRM0(vrm);

        // Matikan total shading gelap / belang MToon pada seluruh avatar (1:1 Identik Bersih dengan Native UI):
        // Di Native UI, shade_color_factor = 1.0 dan shading_shift = -1.0 sehingga wajah 100% mulus anime tanpa bayangan kotor
        const maxAnisotropy = this.renderer ? this.renderer.capabilities.getMaxAnisotropy() : 1;

        vrm.scene.traverse((obj) => {
          if ((obj as THREE.Mesh).isMesh) {
            const mesh = obj as THREE.Mesh;
            mesh.frustumCulled = false;
            mesh.castShadow = false;
            mesh.receiveShadow = false;

            const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
            for (const mat of materials) {
              if (!mat) continue;

              // 1. Samakan warna shade dengan warna dasar (100% putih / bersih)
              if ('shadeColorFactor' in mat && (mat as any).shadeColorFactor) {
                (mat as any).shadeColorFactor.set(1.0, 1.0, 1.0);
              }

              // 2. Samakan tekstur bayangan dengan tekstur diffuse utama
              if ('shadeMultiplyTexture' in mat && 'map' in mat) {
                (mat as any).shadeMultiplyTexture = (mat as any).map || null;
              }

              // 3. Dorong batas shading keluar (-1.0 = 100% full lit tanpa cel-shading cutoff kotor)
              if ('shadingShiftFactor' in mat) {
                (mat as any).shadingShiftFactor = -1.0;
              }

              if ('shadingToonyFactor' in mat) {
                (mat as any).shadingToonyFactor = 0.0;
              }

              // 4. Samakan pencahayaan ambient di segala sudut normal
              if ('giEqualizationFactor' in mat) {
                (mat as any).giEqualizationFactor = 1.0;
              }

              if ('lightColorAttenuationFactor' in mat) {
                (mat as any).lightColorAttenuationFactor = 0.0;
              }

              // 5. Parametric Rim Lighting Halus (1:1 Identik dengan shader.wgsl di @ui: fresnel power 4.5, intensitas lembut)
              if ('rimFactor' in mat && (mat as any).rimFactor) {
                (mat as any).rimFactor.set(0.25, 0.25, 0.25);
              }

              if ('parametricRimFresnelPowerFactor' in mat) {
                (mat as any).parametricRimFresnelPowerFactor = 4.5;
              }

              if ('parametricRimLiftFactor' in mat) {
                (mat as any).parametricRimLiftFactor = 0.0;
              }

              if ('rimLightingMixFactor' in mat) {
                (mat as any).rimLightingMixFactor = 1.0;
              }

              // 6. Anisotropy filtering untuk ketajaman tekstur saat zoom
              if ('map' in mat && (mat as any).map) {
                const tex = (mat as any).map as THREE.Texture;
                tex.anisotropy = Math.min(maxAnisotropy, 4);
              }
            }
          }
        });

        this.vrmAnchor.add(vrm.scene);
        this.currentVrm = vrm;
        callbacks.onLoaded?.();
      },
      (progress) => {
        if (progress.total > 0) {
          const percent = Math.round((progress.loaded / progress.total) * 100);
          callbacks.onProgress?.(percent);
        }
      },
      (err) => {
        console.warn('Failed to load VRM model on mobile:', err);
        callbacks.onError?.(err);
      }
    );

    this.renderer.setAnimationLoop((timestamp, frame) => {
      // Pause total render loop jika tab browser di HP sedang diminimize/backgrounded
      if (typeof document !== 'undefined' && document.hidden) {
        return;
      }

      const isXr = this.renderer?.xr.isPresenting;

      // Ambil delta waktu SEBELUM throttling agar accumulator internal Clock Three.js tetap akurat
      const rawDelta = this.clock?.getDelta() || 0.016;

      // FPS Throttler:
      // Hanya aktifkan throttling frame di mode 3D standar.
      // Di mode WebXR AR, frame loop WAJIB mengikuti display XR native refresh rate tanpa dropped frame
      // agar kamera AR dan pose model tidak flickering/berkedip.
      if (!isXr && this.currentFpsLimit > 0) {
        const minFrameInterval = 1000 / this.currentFpsLimit;
        if (timestamp - this.lastRenderTimestamp < minFrameInterval - 1.5) {
          return;
        }
      }
      this.lastRenderTimestamp = timestamp;

      // Safety clamp terhadap lonjakan lag/tab backgrounding
      const delta = Math.min(rawDelta, 0.1);

      if (isXr) {
        if (this.renderer && this.camera) {
          this.arManager.update(frame, this.renderer, this.camera);
        }

        if (!this.arManager.isLocked() || !this.vrmAnchor.visible) {
          if (this.renderer && this.scene && this.camera) {
            this.renderer.render(this.scene, this.camera);
          }
          return;
        }

        if (this.currentVrm) {
          const { state, emotion, poses } = getState();
          this.poseAnimator.update(this.currentVrm, delta, state, poses);
          this.expressionAnimator.update(
            this.currentVrm,
            delta,
            state,
            emotion,
            this.poseAnimator.getElapsedTime()
          );
          this.currentVrm.update(delta);
        }
      } else {
        if (this.controls && this.controls.enabled) {
          this.controls.update();
        }

        if (this.currentVrm && this.vrmAnchor.visible) {
          const { state, emotion, poses } = getState();
          this.poseAnimator.update(this.currentVrm, delta, state, poses);
          this.expressionAnimator.update(
            this.currentVrm,
            delta,
            state,
            emotion,
            this.poseAnimator.getElapsedTime()
          );
          this.currentVrm.update(delta);
        }
      }

      if (this.renderer && this.scene && this.camera) {
        this.renderer.render(this.scene, this.camera);
      }
    });
  }

  public setFpsLimit(fps: FpsLimit): void {
    this.currentFpsLimit = fps;
  }

  public setRenderQuality(quality: RenderQuality): void {
    this.currentQuality = quality;
    if (!this.renderer) return;

    if (this.renderer.xr.isPresenting) {
      try {
        const arScale = this.getArScaleFactorForQuality(quality);
        this.renderer.xr.setFramebufferScaleFactor(arScale);
      } catch (_) {}
    } else {
      this.renderer.setPixelRatio(this.getPixelRatioForQuality(quality));
    }

    // Sesuaikan ketajaman bayangan dan filtering tekstur sesuai profil kualitas
    if (this.currentVrm) {
      const maxAnisotropy = this.renderer.capabilities.getMaxAnisotropy();
      const targetAnisotropy = quality === 'sd' ? 1 : quality === 'hd' ? Math.min(maxAnisotropy, 4) : Math.min(maxAnisotropy, 8);
      
      this.currentVrm.scene.traverse((obj) => {
        if ((obj as THREE.Mesh).isMesh) {
          const mesh = obj as THREE.Mesh;
          const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
          for (const mat of materials) {
            if (mat && 'map' in mat && (mat as { map?: THREE.Texture }).map) {
              const tex = (mat as { map: THREE.Texture }).map;
              tex.anisotropy = targetAnisotropy;
              tex.needsUpdate = true;
            }
          }
        }
      });
    }
  }

  private getPixelRatioForQuality(quality: RenderQuality): number {
    const dpr = typeof window !== 'undefined' ? window.devicePixelRatio || 1 : 1;
    switch (quality) {
      case 'sd':
        // SD: 1.0x native resolution (sangat hemat baterai / GPU entri level)
        return Math.min(dpr, 1.0);
      case 'ultra':
        // Ultra / 4K Crisp: Full Native Screen DPI (hingga 3.0x di layar OLED flagship)
        return Math.min(dpr, 3.0);
      case 'hd':
      default:
        // HD: 1.5x balanced (tajam tanpa membebani GPU & baterai)
        return Math.min(dpr, 1.5);
    }
  }

  private getArScaleFactorForQuality(quality: RenderQuality): number {
    switch (quality) {
      case 'sd':
        return 0.5;
      case 'hd':
        return 0.75;
      case 'ultra':
      default:
        return 1.0;
    }
  }

  public async startAr(domOverlayElement?: HTMLElement): Promise<void> {
    if (!this.renderer) return;

    if (this.controls) {
      this.controls.enabled = false;
    }

    const arScaleFactor = this.getArScaleFactorForQuality(this.currentQuality);
    try {
      this.renderer.xr.setFramebufferScaleFactor(arScaleFactor);
    } catch (_) {}

    // Matikan Shadow Map saat masuk AR agar kamera AR 60 FPS mulus tanpa drop frame
    if (this.renderer) {
      this.renderer.shadowMap.enabled = false;
    }
    this.renderer.setPixelRatio(1.0);
    this.vrmAnchor.visible = false;

    // Hide normal non-AR DOM canvas container during AR session to avoid double rendering
    if (this.container) {
      this.container.style.visibility = 'hidden';
    }

    if (this.clock) {
      this.clock.getDelta();
    }

    await this.arManager.startArSession(this.renderer, domOverlayElement);
  }

  public stopAr(): void {
    if (!this.renderer) return;
    this.arManager.stopArSession(this.renderer);
  }

  public lockAvatarInRealWorld(): boolean {
    if (!this.camera) return false;
    return this.arManager.lockAvatar(this.vrmAnchor, this.camera);
  }

  public unlockAvatarInRealWorld(): void {
    const session = this.renderer?.xr.getSession();
    this.arManager.unlockAvatar(this.vrmAnchor, session);
  }

  private resetToNonAr(): void {
    if (this.controls) {
      this.controls.enabled = true;
    }

    if (this.renderer) {
      this.renderer.shadowMap.enabled = true;
    }

    if (this.renderer && this.container) {
      // Restore normal non-AR DOM container visibility
      this.container.style.visibility = 'visible';

      const w = this.container.clientWidth || 280;
      const h = this.container.clientHeight || 240;
      this.renderer.setPixelRatio(this.getPixelRatioForQuality(this.currentQuality));
      this.renderer.setSize(w, h);

      if (this.camera) {
        this.camera.aspect = w / h;
        this.camera.updateProjectionMatrix();
      }
    }

    this.vrmAnchor.position.set(0, 0, 0);
    this.vrmAnchor.rotation.set(0, 0, 0);
    this.vrmAnchor.visible = true;
    this.resetCameraView();

    if (this.clock) {
      this.clock.getDelta();
    }
  }

  public resetCameraView(): void {
    if (this.camera && this.controls) {
      this.camera.position.copy(DEFAULT_CAM_POS);
      this.controls.target.copy(DEFAULT_TARGET);
      this.controls.update();
    }
  }

  public zoomIn(): void {
    if (this.camera && this.controls) {
      const dir = new THREE.Vector3().subVectors(this.controls.target, this.camera.position).normalize();
      this.camera.position.addScaledVector(dir, 0.2);
      this.controls.update();
    }
  }

  public zoomOut(): void {
    if (this.camera && this.controls) {
      const dir = new THREE.Vector3().subVectors(this.camera.position, this.controls.target).normalize();
      this.camera.position.addScaledVector(dir, 0.25);
      this.controls.update();
    }
  }

  public cleanup(): void {
    if (this.renderer) {
      this.stopAr();
      this.renderer.setAnimationLoop(null);
    }
    if (this.resizeObserver) {
      this.resizeObserver.disconnect();
      this.resizeObserver = null;
    }
    if (this.controls) {
      this.controls.dispose();
      this.controls = null;
    }
    if (this.currentVrm) {
      try {
        VRMUtils.deepDispose(this.currentVrm.scene);
      } catch (_) {}
      this.currentVrm = null;
    }
    if (this.scene) {
      this.scene.clear();
      this.scene = null;
    }
    if (this.renderer) {
      try {
        this.renderer.dispose();
        this.renderer.forceContextLoss();
        if (this.renderer.domElement && this.renderer.domElement.parentNode) {
          this.renderer.domElement.parentNode.removeChild(this.renderer.domElement);
        }
      } catch (_) {}
      this.renderer = null;
    }

    this.poseAnimator.reset();
    this.expressionAnimator.reset();
  }
}
