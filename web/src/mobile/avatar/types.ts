export type AvatarState = 'idle' | 'listening' | 'speaking' | 'thinking';

export interface AvatarViewerProps {
  state: AvatarState;
  emotion?: string;
  isAudioStreaming?: boolean;
}

export interface PoseKeyframe {
  bones?: Record<string, [number, number, number] | number[]>;
}

export interface PoseStateConfig {
  speed?: number;
  poses?: PoseKeyframe[];
}

export type PosesConfig = Record<string, PoseStateConfig>;
