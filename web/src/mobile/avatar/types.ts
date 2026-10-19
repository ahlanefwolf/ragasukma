export type AvatarState = 'idle' | 'listening' | 'speaking' | 'thinking';

export type RenderQuality = 'sd' | 'hd' | 'ultra';

export type FpsLimit = 30 | 60 | 0; // 0 = Unlimited / Max (120Hz)

export interface AvatarViewerProps {
  state: AvatarState;
  emotion?: string;
  isAudioStreaming?: boolean;
  // Callback status AR untuk sinkronisasi dock interaksi di MobileApp
  onArStateChange?: (isActive: boolean) => void;
  // Slot custom untuk meletakkan mic/chat dock di dalam DOM Overlay AR
  arBottomSlot?: () => any;
}

interface PoseKeyframe {
  bones?: Record<string, [number, number, number] | number[]>;
}

interface PoseStateConfig {
  speed?: number;
  poses?: PoseKeyframe[];
}

export type PosesConfig = Record<string, PoseStateConfig>;
