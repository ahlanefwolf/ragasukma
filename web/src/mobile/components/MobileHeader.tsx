import { Component, Show } from 'solid-js';
import { Monitor, Smartphone, Terminal, Volume2, Mic, Brain, Sparkles } from 'lucide-solid';

export type MobileAudioMode = 'remote_desktop' | 'full_mobile';

interface MobileHeaderProps {
  audioMode: MobileAudioMode;
  onSwitchMode: (mode: MobileAudioMode) => void;
  logsCount: number;
  showLogs: boolean;
  onToggleLogs: () => void;
  visualState: 'idle' | 'listening' | 'speaking' | 'thinking';
}

export const MobileHeader: Component<MobileHeaderProps> = (props) => {
  const getStatusBadge = () => {
    switch (props.visualState) {
      case 'speaking':
        return {
          icon: Volume2,
          label: 'Speaking',
          badgeClass: 'bg-rose-500/20 text-rose-300 border-rose-500/40 shadow-[0_0_10px_rgba(244,63,94,0.3)]',
        };
      case 'listening':
        return {
          icon: Mic,
          label: 'Listening',
          badgeClass: 'bg-sky-500/20 text-sky-300 border-sky-500/40 shadow-[0_0_10px_rgba(14,165,233,0.3)]',
        };
      case 'thinking':
        return {
          icon: Brain,
          label: 'Thinking',
          badgeClass: 'bg-amber-500/20 text-amber-300 border-amber-500/40 shadow-[0_0_10px_rgba(245,158,11,0.3)]',
        };
      default:
        return {
          icon: Sparkles,
          label: 'Standby',
          badgeClass: 'bg-theme-cyan/20 text-theme-cyan border-theme-cyan/40 shadow-[0_0_8px_rgba(34,211,238,0.25)]',
        };
    }
  };

  return (
    <header class="absolute top-0 left-0 right-0 z-30 h-13 px-3 py-2 bg-theme-bg/60 backdrop-blur-xl border-b border-theme-border/40 flex items-center justify-between shrink-0 select-none shadow-md">
      {/* Left Side: Brand + Live Status Badge */}
      <div class="flex items-center gap-2 min-w-0">
        <div class="flex items-center gap-1.5 shrink-0">
          <span class="w-2.5 h-2.5 rounded-full bg-theme-cyan shadow-[0_0_8px_var(--cyan)] animate-pulse" />
          <h1 class="text-xs sm:text-sm font-heading font-bold text-theme-text tracking-wide hidden xs:inline-block">
            Ragasukma
          </h1>
        </div>

        {/* Unified Live Status Badge in Header (Always visible & unobstructed) */}
        {(() => {
          const st = getStatusBadge();
          const Icon = st.icon;
          return (
            <div class={`px-2.5 py-1 rounded-full text-[11px] font-mono font-bold border backdrop-blur-md flex items-center gap-1.5 transition-all ${st.badgeClass}`}>
              <Icon size={12} class={props.visualState === 'speaking' || props.visualState === 'listening' ? 'animate-pulse' : ''} />
              <span>{st.label}</span>
            </div>
          );
        })()}
      </div>

      {/* Right Side: Audio Mode Switcher + Live Logs */}
      <div class="flex items-center gap-1.5 shrink-0">
        {/* Mode Switcher PC / HP */}
        <div class="flex items-center bg-theme-card/80 p-0.5 rounded-xl border border-theme-border/60 text-[10px] font-mono shadow-inner">
          <button
            type="button"
            onClick={() => props.onSwitchMode('remote_desktop')}
            class={`flex items-center gap-1 px-2 py-1 rounded-lg transition-all ${
              props.audioMode === 'remote_desktop'
                ? 'bg-theme-cyan text-theme-card font-bold shadow-sm'
                : 'text-theme-text/60 hover:text-theme-text'
            }`}
            title="Remote Desktop: Voice & Mic on PC"
          >
            <Monitor size={12} />
            <span>PC</span>
          </button>
          <button
            type="button"
            onClick={() => props.onSwitchMode('full_mobile')}
            class={`flex items-center gap-1 px-2 py-1 rounded-lg transition-all ${
              props.audioMode === 'full_mobile'
                ? 'bg-emerald-500 text-white font-bold shadow-sm'
                : 'text-theme-text/60 hover:text-theme-text'
            }`}
            title="Full Mobile: Voice & Mic on Phone"
          >
            <Smartphone size={12} />
            <span>HP</span>
          </button>
        </div>

        {/* Logs Drawer Toggle Button */}
        <button
          type="button"
          onClick={props.onToggleLogs}
          class={`relative p-2 rounded-xl border transition-all active:scale-95 flex items-center justify-center ${
            props.showLogs
              ? 'bg-theme-cyan/20 border-theme-cyan text-theme-cyan shadow-[0_0_10px_rgba(34,211,238,0.3)]'
              : 'bg-theme-card/80 border-theme-border/60 text-theme-text/70 hover:text-theme-text'
          }`}
          title="Open Console Logs"
        >
          <Terminal size={14} />
          <Show when={props.logsCount > 0}>
            <span class="absolute -top-1 -right-1 px-1 min-w-[15px] h-[15px] text-[8.5px] font-mono font-bold rounded-full bg-theme-cyan text-theme-card flex items-center justify-center">
              {props.logsCount > 99 ? '99+' : props.logsCount}
            </span>
          </Show>
        </button>
      </div>
    </header>
  );
};
