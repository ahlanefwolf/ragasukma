import { Component } from 'solid-js';
import { Settings, Volume2, Mic, Brain, Sparkles } from 'lucide-solid';

interface MobileTopBarProps {
  visualState: 'idle' | 'listening' | 'speaking' | 'thinking';
  onOpenSettings: () => void;
  logsCount: number;
}

export const MobileTopBar: Component<MobileTopBarProps> = (props) => {
  const getStatusBadge = () => {
    switch (props.visualState) {
      case 'speaking':
        return {
          icon: Volume2,
          label: 'bicara',
          textColor: 'text-rose-500',
        };
      case 'listening':
        return {
          icon: Mic,
          label: 'mendengar',
          textColor: 'text-sky-500',
        };
      case 'thinking':
        return {
          icon: Brain,
          label: 'mikir',
          textColor: 'text-amber-500',
        };
      default:
        return {
          icon: Sparkles,
          label: 'standby',
          textColor: 'text-theme-cyan',
        };
    }
  };

  return (
    <div class="absolute top-0 left-0 right-0 z-30 p-4 pointer-events-none flex items-center justify-between">
      {/* Sisi Kiri: Clean Floating Status (Icon + Label langsung berwarna tanpa box tebal) */}
      <div class="pointer-events-auto select-none">
        {(() => {
          const st = getStatusBadge();
          const Icon = st.icon;
          return (
            <div class={`flex items-center gap-1.5 font-mono text-xs font-bold drop-shadow-md ${st.textColor}`}>
              <Icon size={15} class={props.visualState === 'speaking' || props.visualState === 'listening' ? 'animate-pulse' : ''} />
              <span class="tracking-wide">{st.label}</span>
            </div>
          );
        })()}
      </div>

      {/* Sisi Kanan: Block Icon Pengaturan Langsung Berwarna Hijau Tema (Tanpa Box / Kotak-kotak) */}
      <div class="pointer-events-auto">
        <button
          type="button"
          onClick={props.onOpenSettings}
          class="p-2 text-theme-cyan hover:opacity-80 active:scale-90 transition-transform drop-shadow-[0_0_8px_rgba(89,107,69,0.35)] flex items-center justify-center cursor-pointer"
          title="Pengaturan"
        >
          <Settings size={24} class="stroke-[2.2]" />
        </button>
      </div>
    </div>
  );
};
