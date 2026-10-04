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

// Default camera position: Minimized / zoomed-out framing full-body character proportionally from head to toe
export const DEFAULT_CAM_POS = new THREE.Vector3(0.0, 0.85, 3.4);
export const DEFAULT_TARGET = new THREE.Vector3(0.0, 0.85, 0.0);

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
        color: 'bg-pink-500/20 text-pink-400 border-pink-500/30',
        glow: 'drop-shadow-[0_0_20px_rgba(236,72,153,0.45)]',
      };
    case 'listening':
      return {
        icon: Mic,
        label: 'Listening',
        color: 'bg-sky-500/20 text-sky-400 border-sky-500/30',
        glow: 'drop-shadow-[0_0_20px_rgba(14,165,233,0.45)]',
      };
    case 'thinking':
      return {
        icon: Brain,
        label: 'Thinking',
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
