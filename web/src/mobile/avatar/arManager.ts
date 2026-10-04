import * as THREE from 'three';

export interface ArState {
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
  private isAvatarLocked = false;
  private hasFoundSurface = false;
  private isActive = false;
  private isStarting = false;
  private listeners: Set<ArStateListener> = new Set();
  private onEndCallback: (() => void) | null = null;
  private sessionStartTime = 0;

  // Smart tracking interpolation targets (mencegah jitter & lompatan kaku)
  private targetPosition = new THREE.Vector3();
  private targetQuaternion = new THREE.Quaternion();
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

  private createSmartReticle(): { group: THREE.Group; ringMat: THREE.MeshBasicMaterial; dotMat: THREE.MeshBasicMaterial } {
    const group = new THREE.Group();
    group.visible = false;

    // 1. Cincin Luar (Outer Ring)
    const ringGeo = new THREE.RingGeometry(0.12, 0.145, 48);
    const ringMat = new THREE.MeshBasicMaterial({
      color: 0x22d3ee,
      opacity: 0.85,
      transparent: true,
      side: THREE.DoubleSide,
      depthTest: false,
    });
    const ringMesh = new THREE.Mesh(ringGeo, ringMat);
    group.add(ringMesh);

    // 2. Inner ambient ring
    const innerRingGeo = new THREE.RingGeometry(0.045, 0.055, 32);
    const innerRingMat = new THREE.MeshBasicMaterial({
      color: 0x06b6d4,
      opacity: 0.45,
      transparent: true,
      side: THREE.DoubleSide,
      depthTest: false,
    });
    const innerRingMesh = new THREE.Mesh(innerRingGeo, innerRingMat);
    group.add(innerRingMesh);

    // 3. Center focal dot
    const dotGeo = new THREE.CircleGeometry(0.018, 24);
    const dotMat = new THREE.MeshBasicMaterial({
      color: 0x38bdf8,
      opacity: 0.95,
      transparent: true,
      side: THREE.DoubleSide,
      depthTest: false,
    });
    const dotMesh = new THREE.Mesh(dotGeo, dotMat);
    group.add(dotMesh);

    // Lock initial orientation flat on horizontal floor plane
    group.quaternion.setFromAxisAngle(new THREE.Vector3(1, 0, 0), -Math.PI / 2);

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
        try { renderer.xr.setReferenceSpaceType('local'); } catch (_) {}
      }

      const sessionInit: any = {
        requiredFeatures: ['hit-test'],
        optionalFeatures: ['local-floor', 'dom-overlay', 'light-estimation'],
      };

      if (domOverlayElement) {
        sessionInit.domOverlay = { root: domOverlayElement };
      }

      const session = await xr.requestSession('immersive-ar', sessionInit);
      await renderer.xr.setSession(session);

      this.isActive = true;
      this.isAvatarLocked = false;
      this.hasFoundSurface = false;
      this.hitTestSource = null;
      this.hasFirstHit = false;
      this.reticle.visible = false;
      this.sessionStartTime = performance.now();

      try {
        const viewerSpace = await session.requestReferenceSpace('viewer');
        if (this.isActive) {
          this.hitTestSource = await session.requestHitTestSource({ space: viewerSpace });
        }
      } catch (err) {
        console.warn('AR hit-test init fallback:', err);
      }

      this.notify();

      session.addEventListener('end', () => {
        this.onSessionEnded(renderer);
      }, { once: true });

      return session;
    } finally {
      this.isStarting = false;
    }
  }

  public update(frame: any, renderer: THREE.WebGLRenderer, camera: THREE.Camera): void {
    if (!this.isActive || this.isAvatarLocked) return;

    let hitFound = false;

    if (this.hitTestSource && frame) {
      try {
        const hitTestResults = frame.getHitTestResults(this.hitTestSource);
        if (hitTestResults.length > 0) {
          const hit = hitTestResults[0];
          const referenceSpace = renderer.xr.getReferenceSpace();
          if (referenceSpace) {
            const pose = hit.getPose(referenceSpace);
            if (pose) {
              this.targetPosition.set(
                pose.transform.position.x,
                pose.transform.position.y,
                pose.transform.position.z
              );

              // 1. SMART SURFACE-NORMAL:
              // Align with detected real-world surface normal (floors/tables)
              // Rotate -90 deg along X-axis so 2D planar ring lays flush on the surface
              const hitQuat = new THREE.Quaternion(
                pose.transform.orientation.x,
                pose.transform.orientation.y,
                pose.transform.orientation.z,
                pose.transform.orientation.w
              );
              const alignPlane = new THREE.Quaternion().setFromAxisAngle(new THREE.Vector3(1, 0, 0), -Math.PI / 2);
              this.targetQuaternion.copy(hitQuat).multiply(alignPlane);

              hitFound = true;
            }
          }
        }
      } catch (_) {}
    }

    // Smart Fallback if surface hit is temporarily undetected (dark/low-contrast floor)
    if (!hitFound) {
      const elapsed = performance.now() - this.sessionStartTime;
      if (elapsed > 600) {
        const forward = new THREE.Vector3(0, -0.4, -1.3).applyQuaternion(camera.quaternion);
        this.targetPosition.copy(camera.position).add(forward);
        this.targetQuaternion.setFromAxisAngle(new THREE.Vector3(1, 0, 0), -Math.PI / 2);
        hitFound = true;
      }
    }

    if (hitFound) {
      // 2. SMOOTH LERP INTERPOLATION (Gerakan meluncur halus, tidak lompat-lompat kaku)
      if (!this.hasFirstHit) {
        this.reticle.position.copy(this.targetPosition);
        this.reticle.quaternion.copy(this.targetQuaternion);
        this.hasFirstHit = true;
      } else {
        this.reticle.position.lerp(this.targetPosition, 0.28);
        this.reticle.quaternion.slerp(this.targetQuaternion, 0.28);
      }

      // 3. DISTANCE AUTO-SCALE & PULSE RING ANIMATION:
      // Menyesuaikan ukuran cincin secara proporsional sesuai jarak kamera pengguna
      const dist = camera.position.distanceTo(this.reticle.position);
      const baseScale = THREE.MathUtils.clamp(dist * 0.55, 0.65, 1.8);
      const pulse = 1.0 + Math.sin(performance.now() * 0.005) * 0.06;
      this.reticle.scale.setScalar(baseScale * pulse);

      // Denyut halus opacity
      this.ringMaterial.opacity = 0.75 + Math.sin(performance.now() * 0.006) * 0.15;

      this.reticle.visible = true;
    } else {
      this.reticle.visible = false;
    }

    if (hitFound !== this.hasFoundSurface) {
      this.hasFoundSurface = hitFound;
      this.notify();
    }
  }

  public lockAvatar(vrmAnchor: THREE.Group, camera: THREE.Camera): boolean {
    if (!this.isActive || !this.hasFoundSurface) return false;

    vrmAnchor.position.copy(this.reticle.position);

    // Hadapkan avatar tegak lurus ke arah pengguna di dunia nyata
    const camPos = new THREE.Vector3();
    camera.getWorldPosition(camPos);
    const lookTarget = new THREE.Vector3(camPos.x, this.reticle.position.y, camPos.z);
    vrmAnchor.lookAt(lookTarget);

    vrmAnchor.visible = true;
    this.reticle.visible = false;
    this.isAvatarLocked = true;
    this.notify();
    return true;
  }

  public unlockAvatar(vrmAnchor: THREE.Group): void {
    if (!this.isActive) return;
    this.isAvatarLocked = false;
    this.hasFirstHit = false;
    this.reticle.visible = true;
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
