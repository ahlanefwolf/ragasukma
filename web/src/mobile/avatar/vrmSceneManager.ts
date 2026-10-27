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
  private lastFrameTime = 0;
  private resizeObserver: ResizeObserver | null = null;

  private poseAnimator = new VrmPoseAnimator();
  private expressionAnimator = new VrmExpressionAnimator();
  private arManager = new WebXRArManager();
  private mainLight: THREE.DirectionalLight | null = null;
  private currentQuality: RenderQuality = 'hd';
  private currentFpsLimit: FpsLimit = 60; // Default 60 FPS (cool & stable for mobile battery)
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

    this.camera = new THREE.PerspectiveCamera(26, width / height, 0.1, 100.0);
    this.camera.position.copy(DEFAULT_CAM_POS);
    this.camera.lookAt(DEFAULT_TARGET);

    this.renderer = new THREE.WebGLRenderer({
      alpha: true,
      antialias: false,
      powerPreference: 'high-performance',
      stencil: false,
      depth: true,
    });
    this.renderer.shadowMap.enabled = false;
    this.renderer.setSize(width, height);
    this.renderer.setPixelRatio(this.getPixelRatioForQuality(this.currentQuality));
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;

    // VRM standard linear tone mapping to ensure clean, consistent skin and clothing saturation without muddy shadows
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

    // Standard MToon Directional Key Light:
    // Placed in scene to illuminate avatar from front-top with Math.PI intensity (unit irradiance).
    // Zero ambient light ensures authentic MToon cel-shading and texture saturation.
    this.mainLight = new THREE.DirectionalLight(0xffffff, Math.PI);
    this.mainLight.position.set(0.35, 2.0, 1.5);
    this.mainLight.target.position.set(0, 0, 0);
    this.scene.add(this.mainLight);
    this.scene.add(this.mainLight.target);

    this.lastFrameTime = performance.now();

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
          VRMUtils.combineSkeletons(gltf.scene);
        } catch (_) {}

        // VRM Spring Bone (dynamic physics for hair, skirt, and accessories)
        if (vrm.springBoneManager) {
          try {
            vrm.springBoneManager.reset();
          } catch (_) {}
        }

        // VRM 0.X compatibility: Rotate VRM0 models 180 degrees so they face the camera (+Z)
        VRMUtils.rotateVRM0(vrm);

        // MToon Material Setup (identical to WGPU pipeline in Native UI):
        // Preserves original material parameters (shade texture, shade color, toony factor),
        // completely disables outlines (inverted hull), and activates soft rim lighting matching shader.wgsl.
        const maxAnisotropy = this.renderer ? this.renderer.capabilities.getMaxAnisotropy() : 1;

        vrm.scene.traverse((obj) => {
          if ((obj as THREE.Mesh).isMesh) {
            const mesh = obj as THREE.Mesh;
            mesh.frustumCulled = false;
            mesh.castShadow = false;
            mesh.receiveShadow = false;

            // 1. Disable and remove outlines completely (matching desktop / native UI shader.wgsl)
            if (Array.isArray(mesh.material)) {
              const nonOutlineMaterials = mesh.material.filter(
                (m) => !(m as any).isOutline && !m.name?.includes('(Outline)')
              );
              const outlineMaterials = mesh.material.filter(
                (m) => (m as any).isOutline || m.name?.includes('(Outline)')
              );

              for (const outMat of outlineMaterials) {
                outMat.visible = false;
                if ('outlineWidthMode' in outMat) {
                  (outMat as any).outlineWidthMode = 'none';
                }
                if ('outlineWidthFactor' in outMat) {
                  (outMat as any).outlineWidthFactor = 0.0;
                }
              }

              // If there is only 1 original surface material and 1 outline material created by three-vrm,
              // revert mesh.material to the single surface material and clear geometry groups
              if (nonOutlineMaterials.length === 1 && mesh.material.length > 1) {
                mesh.material = nonOutlineMaterials[0];
                if (mesh.geometry.groups.length > 0) {
                  mesh.geometry.clearGroups();
                }
              }
            }

            const materials = Array.isArray(mesh.material) ? mesh.material : [mesh.material];
            const isFullyTransparent =
              materials.length > 0 && materials.every((m) => m && (m as THREE.Material).transparent);
            if (isFullyTransparent) {
              mesh.renderOrder = 25;
            }

            for (const mat of materials) {
              if (!mat) continue;

              // Ensure outline property on main material is also disabled
              if ((mat as any).isOutline || mat.name?.includes('(Outline)')) {
                mat.visible = false;
              }
              if ('outlineWidthMode' in mat) {
                (mat as any).outlineWidthMode = 'none';
              }
              if ('outlineWidthFactor' in mat) {
                (mat as any).outlineWidthFactor = 0.0;
              }

              // For transparent / blending materials only:
              // Ensure depthWrite = false to avoid punching holes in background geometry.
              if (mat.transparent) {
                mat.depthWrite = false;
                mat.depthTest = true;
                if (mat.alphaTest === 0) {
                  mat.alphaTest = 0.001;
                }
                // If blend material has 0 opacity (e.g. eye_touming helper quads),
                // hide it to save draw calls and prevent occluding eyeballs
                if (mat.opacity <= 0.001) {
                  mat.visible = false;
                }
              }

              // 2. Anisotropic filtering for crisp textures on close-up zoom
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
      // Pause render loop if mobile browser tab is minimized or backgrounded
      if (typeof document !== 'undefined' && document.hidden) {
        return;
      }

      const isXr = this.renderer?.xr.isPresenting;

      // Sample delta time BEFORE throttling using high-precision performance timer
      const now = performance.now();
      const rawDelta = this.lastFrameTime > 0 ? (now - this.lastFrameTime) / 1000 : 0.016;
      this.lastFrameTime = now;

      // FPS Throttler:
      // Only throttle frames in standard 3D mode.
      // In WebXR AR mode, the frame loop MUST follow native XR display refresh rate without dropped frames
      // to prevent camera and avatar pose flickering.
      if (!isXr && this.currentFpsLimit > 0) {
        const minFrameInterval = 1000 / this.currentFpsLimit;
        if (timestamp - this.lastRenderTimestamp < minFrameInterval - 1.5) {
          return;
        }
      }
      this.lastRenderTimestamp = timestamp;

      // Safety clamp against backgrounding lag spikes
      const delta = Math.min(rawDelta, 0.1);

      if (isXr) {
        if (this.renderer && this.scene && this.camera) {
          this.arManager.update(frame, this.renderer, this.camera);

          if (!this.arManager.isLocked() || !this.vrmAnchor.visible) {
            this.renderer.render(this.scene, this.camera);
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

          this.renderer.render(this.scene, this.camera);
        }
        return;
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

    // Adjust shadow sharpness and texture filtering according to quality profile
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
        // SD: 1.0x (power-saving / low-end mode)
        return Math.min(dpr, 1.0);
      case 'ultra':
      case 'hd':
      default:
        // Native screen pixel ratio (1:1 native screen sharpness)
        return dpr;
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

    // Disable shadow maps during AR to maintain smooth 60 FPS without frame drops
    if (this.renderer) {
      this.renderer.shadowMap.enabled = false;
    }
    this.renderer.setPixelRatio(1.0);
    this.vrmAnchor.visible = false;
    this.lastFrameTime = performance.now();

    await this.arManager.startArSession(this.renderer, domOverlayElement);
  }

  public stopAr(): void {
    if (!this.renderer) return;
    this.arManager.stopArSession(this.renderer);
  }

  public lockAvatarInRealWorld(): boolean {
    if (!this.camera || !this.renderer) return false;
    const locked = this.arManager.lockAvatar(this.vrmAnchor, this.camera, this.renderer);
    if (locked && this.mainLight) {
      this.mainLight.position.set(
        this.vrmAnchor.position.x + 0.35,
        this.vrmAnchor.position.y + 2.0,
        this.vrmAnchor.position.z + 1.5
      );
      this.mainLight.target.position.copy(this.vrmAnchor.position);
    }
    return locked;
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
    if (this.mainLight) {
      this.mainLight.position.set(0.35, 2.0, 1.5);
      this.mainLight.target.position.set(0, 0, 0);
    }
    this.resetCameraView();
    this.lastFrameTime = performance.now();
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
