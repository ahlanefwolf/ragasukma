export type AvatarState = 'idle' | 'listening' | 'speaking' | 'thinking';

export type RenderQuality = 'sd' | 'hd' | 'ultra';

export type FpsLimit = 30 | 60 | 0; // 0 = Unlimited / Max (120Hz)

export interface AvatarViewerProps {
  state: AvatarState;
  emotion?: string;
  isAudioStreaming?: boolean;
  // AR status callback to synchronize interaction dock in MobileApp
  onArStateChange?: (isActive: boolean) => void;
  // Custom slot to mount mic/chat dock inside WebXR DOM Overlay
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
