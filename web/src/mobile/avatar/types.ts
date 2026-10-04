export type AvatarState = 'idle' | 'listening' | 'speaking' | 'thinking';

export type RenderQuality = 'sd' | 'hd' | 'ultra';

export interface AvatarViewerProps {
  state: AvatarState;
  emotion?: string;
  isAudioStreaming?: boolean;
  // Callback status AR untuk sinkronisasi dock interaksi di MobileApp
  onArStateChange?: (isActive: boolean) => void;
  // Slot custom untuk meletakkan mic/chat dock di dalam DOM Overlay AR
  arBottomSlot?: () => any;
}

export interface PoseKeyframe {
  bones?: Record<string, [number, number, number] | number[]>;
}

export interface PoseStateConfig {
  speed?: number;
  poses?: PoseKeyframe[];
}

export type PosesConfig = Record<string, PoseStateConfig>;
