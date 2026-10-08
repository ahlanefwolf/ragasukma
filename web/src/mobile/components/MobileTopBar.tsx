import { Component, Show } from 'solid-js';
import { Settings, Volume2, Mic, Brain, Sparkles, Activity } from 'lucide-solid';

interface MobileTopBarProps {
  visualState: 'idle' | 'listening' | 'speaking' | 'thinking';
  onOpenSettings: () => void;
  logsCount: number;
  pingMs?: number | null;
}

export const MobileTopBar: Component<MobileTopBarProps> = (props) => {
  const getStatusBadge = () => {
    switch (props.visualState) {
      case 'speaking':
        return {
          icon: Volume2,
          label: 'speaking',
          textColor: 'text-rose-500',
        };
      case 'listening':
        return {
          icon: Mic,
          label: 'listening',
          textColor: 'text-sky-500',
        };
      case 'thinking':
        return {
          icon: Brain,
          label: 'thinking',
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

  const getPingBadge = () => {
    if (props.pingMs === undefined || props.pingMs === null) {
      return {
        color: 'text-zinc-500',
        dotColor: 'bg-zinc-500',
        label: '-- ms',
      };
    }
    if (props.pingMs < 100) {
      return {
        color: 'text-emerald-400',
        dotColor: 'bg-emerald-400',
        label: `${props.pingMs}ms`,
      };
    }
    if (props.pingMs < 300) {
      return {
        color: 'text-amber-400',
        dotColor: 'bg-amber-400',
        label: `${props.pingMs}ms`,
      };
    }
    return {
      color: 'text-rose-500',
      dotColor: 'bg-rose-500',
      label: `${props.pingMs}ms`,
    };
  };

  return (
    <div class="absolute top-0 left-0 right-0 z-30 p-4 pointer-events-none flex items-center justify-between">
      {/* Left Side: Clean Floating Status (Icon + Direct colored label) & Ping Indicator */}
      <div class="pointer-events-auto select-none flex items-center gap-3">
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

        {/* Network Ping Indicator (Green <100ms, Yellow <300ms, Red >=300ms) */}
        {(() => {
          const ping = getPingBadge();
          return (
            <div
              class={`flex items-center gap-1 px-1.5 py-0.5 rounded-full font-mono text-[10px] font-bold drop-shadow-md border border-white/5 bg-black/20 backdrop-blur-xs ${ping.color}`}
              title={`Network Latency: ${ping.label}`}
            >
              <span class={`w-1.5 h-1.5 rounded-full ${ping.dotColor} ${props.pingMs !== null ? 'animate-pulse' : ''}`} />
              <span>{ping.label}</span>
            </div>
          );
        })()}
      </div>

      {/* Right Side: Theme-colored Settings Icon Button (Without box frame) */}
      <div class="pointer-events-auto">
        <button
          type="button"
          onClick={props.onOpenSettings}
          class="p-2 text-theme-cyan hover:opacity-80 active:scale-90 transition-transform drop-shadow-[0_0_8px_rgba(89,107,69,0.35)] flex items-center justify-center cursor-pointer"
          title="Settings"
        >
          <Settings size={24} class="stroke-[2.2]" />
        </button>
      </div>
    </div>
  );
};
