import * as THREE from 'three';
import { VRM } from '@pixiv/three-vrm';
import { BONE_NAME_FALLBACKS } from './constants';
import { AvatarState, PosesConfig } from './types';

export class VrmPoseAnimator {
  private currentBoneQuats = new Map<string, THREE.Quaternion>();
  private stateFromQuats = new Map<string, THREE.Quaternion>();
  private stateTransitionT = 1.0;
  private lastAnimState: AvatarState = 'idle';
  private stateElapsedTime = 0;

  public getElapsedTime(): number {
    return this.stateElapsedTime;
  }

  public reset(): void {
    this.currentBoneQuats.clear();
    this.stateFromQuats.clear();
    this.stateTransitionT = 1.0;
    this.lastAnimState = 'idle';
    this.stateElapsedTime = 0;
  }

  private getNormalizedBone(vrm: VRM, boneKey: string): THREE.Object3D | null {
    if (!vrm.humanoid) return null;
    const candidates = BONE_NAME_FALLBACKS[boneKey] || [boneKey];
    for (const name of candidates) {
      const node = vrm.humanoid.getNormalizedBoneNode(name as any);
      if (node) return node;
    }
    return null;
  }

  private catmullRom(v0: number, v1: number, v2: number, v3: number, tVal: number): number {
    const t2 = tVal * tVal;
    const t3 = t2 * tVal;
    return 0.5 * (
      2.0 * v1 +
      (-v0 + v2) * tVal +
      (2.0 * v0 - 5.0 * v1 + 4.0 * v2 - v3) * t2 +
      (-v0 + 3.0 * v1 - 3.0 * v2 + v3) * t3
    );
  }

  public update(vrm: VRM, delta: number, currentState: AvatarState, poses: PosesConfig | null): void {
    if (!vrm.humanoid || !poses) return;

    if (currentState !== this.lastAnimState) {
      // Snapshot current rotation of all bones & fingers as start point for smooth slerp transition
      this.stateFromQuats.clear();
      for (const [k, q] of this.currentBoneQuats.entries()) {
        this.stateFromQuats.set(k, q.clone());
      }
      this.stateTransitionT = 0.0;
      this.stateElapsedTime = 0;
      this.lastAnimState = currentState;
    }
    this.stateElapsedTime += delta;

    // Timer transisi Slerp (durasi 0.45 detik identik with WGPU desktop)
    const transitionDuration = 0.45;
    if (this.stateTransitionT < 1.0) {
      this.stateTransitionT = Math.min(1.0, this.stateTransitionT + delta / transitionDuration);
    }
    // S-Curve cosine smoothing:
    const blendT = this.stateTransitionT < 1.0
      ? (1.0 - Math.cos(this.stateTransitionT * Math.PI)) / 2.0
      : 1.0;

    const stateObj = poses[currentState] || poses.idle;
    const keyframes = stateObj?.poses || [];
    const speed = Math.max(0.1, stateObj?.speed || 1.0);
    const count = keyframes.length;

    if (count === 0) return;

    // Natural Breathing ritme santai
    const breathSine = Math.sin(this.stateElapsedTime * 1.35);
    const breathChest = breathSine * 0.012;
    const breathSpine = breathSine * 0.006;
    const breathShoulder = breathSine * 0.006;

    // Check if model is VRM 0.X (metaVersion === "0")
    // In VRM 0.X under 180° Y flip (R_y(π)), Euler angles transform as (-rx, ry, -rz)
    const isVrm0 = (vrm.meta as any)?.metaVersion === '0';

    const applyBoneRotation = (boneName: string, rx: number, ry: number, rz: number) => {
      const node = this.getNormalizedBone(vrm, boneName);
      if (!node) return;

      let effRx = rx;
      let effRy = ry;
      let effRz = rz;
      if (isVrm0) {
        effRx = -rx;
        effRz = -rz;
      }

      let offY = 0;
      if (boneName === 'chest') offY = breathChest;
      else if (boneName === 'spine') offY = breathSpine;
      else if (boneName === 'leftShoulder' || boneName === 'rightShoulder') offY = breathShoulder;

      const targetEuler = new THREE.Euler(effRx + offY, effRy, effRz, 'XYZ');
      const targetQ = new THREE.Quaternion().setFromEuler(targetEuler);

      let finalQ: THREE.Quaternion;
      if (blendT < 1.0) {
        const fromQ = this.stateFromQuats.get(boneName) || targetQ;
        finalQ = fromQ.clone().slerp(targetQ, blendT);
      } else {
        finalQ = targetQ;
      }

      this.currentBoneQuats.set(boneName, finalQ.clone());
      node.quaternion.copy(finalQ);
    };

    if (count === 1) {
      const singleBones = keyframes[0].bones || {};
      for (const [boneName, rot] of Object.entries(singleBones)) {
        if (!Array.isArray(rot) || rot.length < 3) continue;
        applyBoneRotation(boneName, rot[0] as number, rot[1] as number, rot[2] as number);
      }
    } else {
      // Multi-keyframes cycle Catmull-Rom C1 continuity
      const cycleStep = 2.5 / speed;
      const totalCycleTime = count * cycleStep;
      const loopTime = this.stateElapsedTime % totalCycleTime;
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

      for (const boneName of allBoneKeys) {
        const r0 = p0[boneName] || [0, 0, 0];
        const r1 = p1[boneName] || [0, 0, 0];
        const r2 = p2[boneName] || [0, 0, 0];
        const r3 = p3[boneName] || [0, 0, 0];

        const rx = this.catmullRom(r0[0], r1[0], r2[0], r3[0], t);
        const ry = this.catmullRom(r0[1], r1[1], r2[1], r3[1], t);
        const rz = this.catmullRom(r0[2], r1[2], r2[2], r3[2], t);

        applyBoneRotation(boneName, rx, ry, rz);
      }
    }
  }
}
