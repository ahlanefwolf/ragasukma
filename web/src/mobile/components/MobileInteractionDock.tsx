import { Component, Show } from 'solid-js';
import { Mic, MicOff, MessageSquare, Bot } from 'lucide-solid';

interface MobileInteractionDockProps {
  inputText?: string;
  onInputText?: (val: string) => void;
  onSend?: () => void;
  isSending?: boolean;
  isListening: boolean;
  onToggleMic: () => void;
  lastSentText?: string;
  showInput?: boolean;
  onToggleShowInput: () => void;
  onDismissLastSent?: () => void;
  onOpenAgent?: () => void;
  isAgentRunning?: boolean;
}

export const MobileInteractionDock: Component<MobileInteractionDockProps> = (props) => {
  const baseBtn =
    'relative flex items-center justify-center w-11 h-11 border transition-colors active:bg-theme-card';

  return (
    <div class="relative z-30 w-full flex flex-col items-center gap-3 px-4 pb-4 pointer-events-auto select-none">
      {/* Dock: one shared frame, three equal-weight actions, mic carries the accent */}
      <div class="flex items-stretch gap-px bg-theme-border border border-theme-border">
        {/* 1. Autonomous Agent Monitor */}
        <Show when={props.onOpenAgent}>
          <button
            type="button"
            onClick={props.onOpenAgent}
            class={`${baseBtn} ${
              props.isAgentRunning
                ? 'bg-gi-primary/12 border-gi-primary/40 text-gi-primary'
                : 'bg-theme-surface border-transparent text-theme-muted'
            }`}
            title="Agent tasks"
            aria-label="Agent tasks"
          >
            <Bot size={19} class="stroke-[1.9]" />
            <Show when={props.isAgentRunning}>
              <span class="absolute top-1.5 right-1.5 w-1.5 h-1.5 bg-gi-status-warning" />
            </Show>
          </button>
        </Show>

        {/* 2. Chat */}
        <button
          type="button"
          onClick={props.onToggleShowInput}
          class={`${baseBtn} bg-theme-surface border-transparent text-theme-muted active:text-theme-text`}
          title="Open chat"
          aria-label="Open chat"
        >
          <MessageSquare size={19} class="stroke-[1.9]" />
        </button>

        {/* 3. Primary push-to-talk */}
        <button
          type="button"
          onClick={props.onToggleMic}
          class={`${baseBtn} w-14 ${
            props.isListening
              ? 'bg-gi-status-danger border-gi-status-danger text-gi-primary-foreground'
              : 'bg-theme-surface border-transparent text-gi-primary'
          }`}
          title={props.isListening ? 'Stop listening' : 'Push to talk'}
          aria-label={props.isListening ? 'Stop listening' : 'Push to talk'}
        >
          <Show when={props.isListening} fallback={<Mic size={21} class="stroke-[1.9]" />}>
            <MicOff size={21} class="stroke-[1.9]" />
          </Show>
        </button>
      </div>

      {/* Mic state is stated once, in words, instead of pulsing the icon forever */}
      <Show when={props.isListening}>
        <span class="text-2xs font-mono text-gi-status-danger">listening — tap to stop</span>
      </Show>
    </div>
  );
};