import * as THREE from 'three';

interface ArState {
  isSupported: boolean;
  isActive: boolean;
  hasFoundSurface: boolean;
  isAvatarLocked: boolean;
}

export type ArStateListener = (state: ArState) => void;

export class WebXRArManager {
  private reticle: THREE.Group;
  private ringMaterial: THREE.MeshBasicMaterial;
  private dotMaterial: THREE.MeshBasicMaterial;
  private hitTestSource: any = null;
  private hitTestSourceRequested = false;
  private isAvatarLocked = false;
  private hasFoundSurface = false;
  private isActive = false;
  private isStarting = false;
  private listeners: Set<ArStateListener> = new Set();
  private onEndCallback: (() => void) | null = null;
  private sessionStartTime = 0;

  // Tracking interpolation targets (prevents jitter & abrupt jumps)
  private targetPosition = new THREE.Vector3();
  private targetQuaternion = new THREE.Quaternion();
  private targetScale = new THREE.Vector3();
  private hasFirstHit = false;

  constructor() {
    const { group, ringMat, dotMat } = this.createSmartReticle();
    this.reticle = group;
    this.ringMaterial = ringMat;
    this.dotMaterial = dotMat;
  }

  public getReticle(): THREE.Group {
    return this.reticle;
  }

  public isLocked(): boolean {
    return this.isAvatarLocked;
  }

  public isSessionActive(): boolean {
    return this.isActive;
  }

  public setOnEndCallback(cb: () => void): void {
    this.onEndCallback = cb;
  }

  public subscribe(listener: ArStateListener): () => void {
    this.listeners.add(listener);
    this.notify();
    return () => this.listeners.delete(listener);
  }

  private notify(): void {
    const state: ArState = {
      isSupported: true,
      isActive: this.isActive,
      hasFoundSurface: this.hasFoundSurface,
      isAvatarLocked: this.isAvatarLocked,
    };
    for (const l of this.listeners) {
      l(state);
    }
  }

  private createSmartReticle(): {
    group: THREE.Group;
    ringMat: THREE.MeshBasicMaterial;
    dotMat: THREE.MeshBasicMaterial;
  } {
    const group = new THREE.Group();
    group.visible = false;

    // 1. Outer Ring - Wish 5-Star Gold (0xe5b85c)
    // Geometry: radius 0.11 to 0.155 (4.5 cm thick band, 31 cm total diameter)
    const ringGeo = new THREE.RingGeometry(0.11, 0.155, 64);
    ringGeo.rotateX(-Math.PI / 2);
    const ringMat = new THREE.MeshBasicMaterial({
      color: 0xe5b85c,
      opacity: 0.9,
      transparent: true,
      side: THREE.DoubleSide,
      depthTest: false,
      depthWrite: false,
    });
    const ringMesh = new THREE.Mesh(ringGeo, ringMat);
    ringMesh.renderOrder = 9999;
    ringMesh.frustumCulled = false;
    group.add(ringMesh);

    // 2. Inner ambient ring - Warm Deep Gold (0xc79a3e)
    // Geometry: radius 0.05 to 0.065
    const innerRingGeo = new THREE.RingGeometry(0.05, 0.065, 48);
    innerRingGeo.rotateX(-Math.PI / 2);
    const innerRingMat = new THREE.MeshBasicMaterial({
      color: 0xc79a3e,
      opacity: 0.65,
      transparent: true,
      side: THREE.DoubleSide,
      depthTest: false,
      depthWrite: false,
    });
    const innerRingMesh = new THREE.Mesh(innerRingGeo, innerRingMat);
    innerRingMesh.renderOrder = 9999;
    innerRingMesh.frustumCulled = false;
    group.add(innerRingMesh);

    // 3. Center focal dot - Pure Warm White (0xfff9e6)
    const dotGeo = new THREE.CircleGeometry(0.022, 32);
    dotGeo.rotateX(-Math.PI / 2);
    const dotMat = new THREE.MeshBasicMaterial({
      color: 0xfff9e6,
      opacity: 0.95,
      transparent: true,
      side: THREE.DoubleSide,
      depthTest: false,
      depthWrite: false,
    });
    const dotMesh = new THREE.Mesh(dotGeo, dotMat);
    dotMesh.renderOrder = 9999;
    dotMesh.frustumCulled = false;
    group.add(dotMesh);

    // 4. Four alignment crosshair ticks (Wish Gold)
    const tickGeo = new THREE.PlaneGeometry(0.012, 0.04);
    tickGeo.rotateX(-Math.PI / 2);
    const tickMat = new THREE.MeshBasicMaterial({
      color: 0xe5b85c,
      opacity: 0.9,
      transparent: true,
      side: THREE.DoubleSide,
      depthTest: false,
      depthWrite: false,
    });
    for (let i = 0; i < 4; i++) {
      const angle = (i * Math.PI) / 2;
      const tick = new THREE.Mesh(tickGeo, tickMat);
      tick.renderOrder = 9999;
      tick.frustumCulled = false;
      tick.position.set(Math.cos(angle) * 0.175, 0, Math.sin(angle) * 0.175);
      tick.rotation.y = -angle;
      group.add(tick);
    }

    return { group, ringMat, dotMat };
  }

  public static async isSupported(): Promise<boolean> {
    if (typeof navigator === 'undefined' || !(navigator as any).xr) return false;
    try {
      return await (navigator as any).xr.isSessionSupported('immersive-ar');
    } catch (_) {
      return false;
    }
  }

  public async startArSession(
    renderer: THREE.WebGLRenderer,
    domOverlayElement?: HTMLElement
  ): Promise<any> {
    if (this.isActive || this.isStarting) return null;
    this.isStarting = true;

    const xr = (navigator as any).xr;
    if (!xr) {
      this.isStarting = false;
      throw new Error('WebXR not available');
    }

    try {
      renderer.xr.enabled = true;
      try {
        renderer.xr.setReferenceSpaceType('local-floor');
      } catch (_) {
        try {
          renderer.xr.setReferenceSpaceType('local');
        } catch (_) {}
      }

      let session: any;
      try {
        session = await xr.requestSession('immersive-ar', {
          requiredFeatures: ['hit-test'],
          optionalFeatures: ['local-floor', 'local', 'dom-overlay', 'light-estimation'],
          ...(domOverlayElement ? { domOverlay: { root: domOverlayElement } } : {}),
        });
      } catch (err) {
        console.warn('[WebXR AR] Request with required hit-test rejected, trying optional:', err);
        session = await xr.requestSession('immersive-ar', {
          optionalFeatures: ['hit-test', 'local-floor', 'local', 'dom-overlay', 'light-estimation'],
          ...(domOverlayElement ? { domOverlay: { root: domOverlayElement } } : {}),
        });
      }

      await renderer.xr.setSession(session);

      this.isActive = true;
      this.isAvatarLocked = false;
      this.hasFoundSurface = false;
      this.hitTestSource = null;
      this.hitTestSourceRequested = false;
      this.hasFirstHit = false;
      this.reticle.visible = false;
      this.sessionStartTime = performance.now();

      // Acquire viewer reference space and hit test source
      try {
        const viewerSpace = await session.requestReferenceSpace('viewer');
        if (this.isActive) {
          this.hitTestSource = await session.requestHitTestSource({ space: viewerSpace });
          this.hitTestSourceRequested = true;
          console.log('[WebXR AR] HitTestSource acquired successfully');
        }
      } catch (err) {
        console.warn('[WebXR AR] Immediate hit-test source acquisition fallback:', err);
      }

      this.notify();

      session.addEventListener(
        'end',
        () => {
          this.onSessionEnded(renderer);
        },
        { once: true }
      );

      return session;
    } finally {
      this.isStarting = false;
    }
  }

  public update(frame: any, renderer: THREE.WebGLRenderer, camera: THREE.Camera): void {
    if (!this.isActive || this.isAvatarLocked) return;

    const session: any = renderer.xr.getSession();
    if (!session) return;

    // 1. Request hit-test source lazily if initial async call was deferred
    if (!this.hitTestSource && !this.hitTestSourceRequested) {
      this.hitTestSourceRequested = true;
      if (typeof session.requestHitTestSource === 'function') {
        session
          .requestReferenceSpace('viewer')
          .then((viewerSpace: any) => {
            session
              .requestHitTestSource({ space: viewerSpace })
              .then((source: any) => {
                this.hitTestSource = source;
                console.log('[WebXR AR] HitTestSource acquired in frame loop');
              })
              .catch((err: any) => {
                console.warn('[WebXR AR] requestHitTestSource failed:', err);
              });
          })
          .catch((err: any) => {
            console.warn('[WebXR AR] requestReferenceSpace viewer failed:', err);
          });
      }
    }

    let hitFound = false;

    // 2. Query WebXR SLAM Hit-Test against real-world detected floor/table plane
    if (this.hitTestSource && frame) {
      try {
        const hitTestResults = frame.getHitTestResults(this.hitTestSource);
        if (hitTestResults.length > 0) {
          const hit = hitTestResults[0];
          const referenceSpace = renderer.xr.getReferenceSpace();
          if (referenceSpace) {
            const pose = hit.getPose(referenceSpace);
            if (pose && pose.transform) {
              const m = new THREE.Matrix4().fromArray(pose.transform.matrix);
              m.decompose(this.targetPosition, this.targetQuaternion, this.targetScale);
              hitFound = true;
            }
          }
        }
      } catch (_) {}
    }

    if (hitFound) {
      // 3. Smooth interpolation towards physical surface hit
      if (!this.hasFirstHit) {
        this.reticle.position.copy(this.targetPosition);
        this.reticle.quaternion.copy(this.targetQuaternion);
        this.hasFirstHit = true;
      } else {
        this.reticle.position.lerp(this.targetPosition, 0.3);
        this.reticle.quaternion.slerp(this.targetQuaternion, 0.3);
      }

      // Natural subtle pulsing animation
      const pulse = 1.0 + Math.sin(performance.now() * 0.005) * 0.05;
      this.reticle.scale.setScalar(pulse);
      this.ringMaterial.opacity = 0.85 + Math.sin(performance.now() * 0.006) * 0.12;

      this.reticle.visible = true;
    } else {
      // Hide reticle if no physical plane is currently detected
      this.reticle.visible = false;
    }

    if (hitFound !== this.hasFoundSurface) {
      this.hasFoundSurface = hitFound;
      this.notify();
    }
  }

  public lockAvatar(vrmAnchor: THREE.Group, camera: THREE.Camera, renderer?: THREE.WebGLRenderer): boolean {
    if (!this.isActive || !this.hasFoundSurface) return false;

    // 1. Place avatar firmly at current reticle coordinates
    vrmAnchor.position.copy(this.reticle.position);

    // 2. Orient avatar directly facing the user in real-world space
    const camPos = new THREE.Vector3();
    let hasCamPose = false;

    if (renderer) {
      const xrCam = (renderer.xr.getCamera() as any)?.cameras?.[0] || renderer.xr.getCamera();
      if (xrCam) {
        xrCam.getWorldPosition(camPos);
        hasCamPose = camPos.lengthSq() > 0;
      }
    }

    if (!hasCamPose) {
      camera.getWorldPosition(camPos);
    }

    const dist = Math.hypot(camPos.x - this.reticle.position.x, camPos.z - this.reticle.position.z);
    if (dist > 0.05) {
      const lookTarget = new THREE.Vector3(camPos.x, this.reticle.position.y, camPos.z);
      vrmAnchor.lookAt(lookTarget);
    }

    vrmAnchor.visible = true;
    this.reticle.visible = false;
    this.isAvatarLocked = true;

    // Cancel active hit test sensor to eliminate SLAM compute overhead while anchored
    if (this.hitTestSource) {
      try {
        this.hitTestSource.cancel();
      } catch (_) {}
      this.hitTestSource = null;
    }
    this.hitTestSourceRequested = false;

    this.notify();
    return true;
  }

  public async unlockAvatar(vrmAnchor: THREE.Group, session?: any): Promise<void> {
    if (!this.isActive) return;
    this.isAvatarLocked = false;
    this.hasFirstHit = false;
    this.hasFoundSurface = false;
    this.reticle.visible = false;
    vrmAnchor.visible = false;

    // Re-enable hit-test sensor when unlocked to discover new surface placement
    if (session && !this.hitTestSource) {
      try {
        const viewerSpace = await session.requestReferenceSpace('viewer');
        if (this.isActive && !this.isAvatarLocked) {
          this.hitTestSource = await session.requestHitTestSource({ space: viewerSpace });
        }
      } catch (err) {
        console.warn('[WebXR AR] Re-enable hit-test source error:', err);
      }
    }

    this.notify();
  }

  public stopArSession(renderer: THREE.WebGLRenderer): void {
    const session = renderer.xr.getSession();
    if (session) {
      session.end().catch(() => {
        this.onSessionEnded(renderer);
      });
    } else {
      this.onSessionEnded(renderer);
    }
  }

  private onSessionEnded(renderer: THREE.WebGLRenderer): void {
    if (!this.isActive && !this.hitTestSource) return;

    this.isActive = false;
    this.isAvatarLocked = false;
    this.hasFoundSurface = false;
    this.hasFirstHit = false;

    if (this.hitTestSource) {
      try {
        this.hitTestSource.cancel();
      } catch (_) {}
      this.hitTestSource = null;
    }
    this.hitTestSourceRequested = false;
    this.reticle.visible = false;

    try {
      renderer.xr.enabled = false;
    } catch (_) {}

    this.notify();

    if (this.onEndCallback) {
      this.onEndCallback();
    }
  }
}
