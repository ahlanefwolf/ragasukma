import * as THREE from 'three';
import { GLTFLoader } from 'three/examples/jsm/loaders/GLTFLoader.js';
import { VRMLoaderPlugin, VRM, VRMUtils } from '@pixiv/three-vrm';
import { OrbitControls } from 'three/examples/jsm/controls/OrbitControls.js';
import { DEFAULT_CAM_POS, DEFAULT_TARGET } from './constants';
import { AvatarState, PosesConfig, RenderQuality } from './types';
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

  private createContactShadow(): THREE.Mesh {
    const canvas = document.createElement('canvas');
    canvas.width = 128;
    canvas.height = 128;
    const ctx = canvas.getContext('2d');
    if (ctx) {
      const gradient = ctx.createRadialGradient(64, 64, 0, 64, 64, 64);
      gradient.addColorStop(0, 'rgba(0, 0, 0, 0.70)');
      gradient.addColorStop(0.35, 'rgba(0, 0, 0, 0.45)');
      gradient.addColorStop(0.70, 'rgba(0, 0, 0, 0.15)');
      gradient.addColorStop(1.0, 'rgba(0, 0, 0, 0.0)');
      ctx.fillStyle = gradient;
      ctx.fillRect(0, 0, 128, 128);
    }

    const texture = new THREE.CanvasTexture(canvas);
    const geometry = new THREE.PlaneGeometry(0.85, 0.85);
    geometry.rotateX(-Math.PI / 2);

    const material = new THREE.MeshBasicMaterial({
      map: texture,
      transparent: true,
      opacity: 0.65,
      depthWrite: false,
    });

    const mesh = new THREE.Mesh(geometry, material);
    mesh.position.y = 0.005; // Slightly above ground to prevent z-fighting
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
      antialias: true,
      powerPreference: 'high-performance',
    });
    this.renderer.setSize(width, height);
    this.renderer.setPixelRatio(this.getPixelRatioForQuality(this.currentQuality));
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;

    // Linear tone mapping bawaan standar VRM agar saturasi dan rona warna kulit/baju tetap hidup & tidak pucat
    this.renderer.toneMapping = THREE.LinearToneMapping;
    this.renderer.toneMappingExposure = 1.0;

    container.innerHTML = '';
    container.appendChild(this.renderer.domElement);

    this.vrmAnchor.position.set(0, 0, 0);
    this.vrmAnchor.rotation.set(0, 0, 0);
    this.vrmAnchor.visible = true;

    // Add soft contact shadow directly beneath avatar feet
    this.contactShadow = this.createContactShadow();
    this.vrmAnchor.add(this.contactShadow);

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

    // Pencahayaan standar VRM MToon yang cerah dan berwarna alami:
    const ambientLight = new THREE.AmbientLight(0xffffff, 1.25);
    this.scene.add(ambientLight);

    const dirLight = new THREE.DirectionalLight(0xffffff, 1.15);
    dirLight.position.set(0.5, 2.5, 1.8);
    this.scene.add(dirLight);

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

        if (vrm.springBoneManager) {
          try {
            vrm.springBoneManager.reset();
          } catch (_) {}
          (vrm as any).springBoneManager = null;
        }

        // VRM 0.X compatibility: Rotate VRM0 models 180 degrees so they face the camera (+Z)
        VRMUtils.rotateVRM0(vrm);

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

    this.renderer.setAnimationLoop((_timestamp, frame) => {
      const isXr = this.renderer?.xr.isPresenting;
      const rawDelta = this.clock?.getDelta() || 0.016;
      // Safety clamp terhadap lonjakan lag/tab backgrounding tanpa membatasi 60Hz/90Hz/120Hz refresh rate
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

  public setRenderQuality(quality: RenderQuality): void {
    this.currentQuality = quality;
    if (!this.renderer || this.renderer.xr.isPresenting) return;
    this.renderer.setPixelRatio(this.getPixelRatioForQuality(quality));
  }

  private getPixelRatioForQuality(quality: RenderQuality): number {
    const dpr = typeof window !== 'undefined' ? window.devicePixelRatio || 1 : 1;
    switch (quality) {
      case 'sd':
        return 1.0;
      case 'ultra':
        return Math.min(dpr, 2.75);
      case 'hd':
      default:
        return Math.min(dpr, 1.6);
    }
  }

  private getArScaleFactorForQuality(quality: RenderQuality): number {
    switch (quality) {
      case 'sd':
        return 0.45;
      case 'hd':
        return 0.65;
      case 'ultra':
      default:
        return 0.85;
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
