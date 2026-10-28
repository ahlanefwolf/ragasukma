import * as THREE from 'three';
import { Volume2, Mic, Brain, Sparkles } from 'lucide-solid';
import { AvatarState } from './types';

// Mapping dictionary of body joints from poses.json to Three.js VRM Humanoid Bone Standard
export const BONE_NAME_FALLBACKS: Record<string, string[]> = {
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
  // Finger Hand (VRM Humanoid Normalized Bones 1:1 Engine WGPU Desktop)
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

// Default camera position: Karakter tepat 3/4 (75%) tinggi canvas
export const DEFAULT_CAM_POS = new THREE.Vector3(0.0, 0.75, 4.33);
export const DEFAULT_TARGET = new THREE.Vector3(0.0, 0.75, 0.0);

export const AVATAR_IMAGES: Record<AvatarState, string> = {
  idle: '/api/assets/file/char-avatar-idle.png',
  listening: '/api/assets/file/char-avatar-listening.png',
  speaking: '/api/assets/file/char-avatar-speaking.png',
  thinking: '/api/assets/file/char-avatar-thinking.png',
};

export const ALL_KNOWN_EXPRESSION_KEYS = [
  'aa', 'ih', 'ou', 'ee', 'oh',
  'blink', 'blinkLeft', 'blinkRight',
  'happy', 'angry', 'sad', 'surprised', 'relaxed',
] as const;

export const getStatusBadge = (state: AvatarState) => {
  switch (state) {
    case 'speaking':
      return {
        icon: Volume2,
        label: 'Speaking',
        color: 'bg-gi-status-danger/20 text-gi-status-danger border-gi-status-danger/30',
        glow: 'drop-shadow-[0_0_20px_rgba(var(--gi-status-danger-rgb),0.45)]',
      };
    case 'listening':
      return {
        icon: Mic,
        label: 'Listening',
        color: 'bg-gi-status-info/20 text-gi-status-info border-gi-status-info/30',
        glow: 'drop-shadow-[0_0_20px_rgba(var(--gi-status-info-rgb),0.45)]',
      };
    case 'thinking':
      return {
        icon: Brain,
        label: 'Thinking',
        color: 'bg-gi-status-warning/20 text-gi-status-warning border-gi-status-warning/30',
        glow: 'drop-shadow-[0_0_20px_rgba(var(--gi-status-warning-rgb),0.45)]',
      };
    case 'idle':
    default:
      return {
        icon: Sparkles,
        label: 'Standby',
        color: 'bg-gi-primary/20 text-gi-primary border-gi-primary/30',
        glow: 'drop-shadow-[0_0_12px_rgba(var(--gi-primary-rgb),0.35)]',
      };
  }
};
