export type AvatarState = 'idle' | 'listening' | 'speaking' | 'thinking';

export type RenderQuality = 'sd' | 'hd' | 'ultra';

export type FpsLimit = 30 | 60 | 0; // 0 = Unlimited / Max (120Hz)

export interface AvatarViewerProps {
  state: AvatarState;
  emotion?: string;
  isAudioStreaming?: boolean;
  paused?: boolean;
}

interface PoseKeyframe {
  bones?: Record<string, [number, number, number] | number[]>;
}

interface PoseStateConfig {
  speed?: number;
  poses?: PoseKeyframe[];
}

export type PosesConfig = Record<string, PoseStateConfig>;
