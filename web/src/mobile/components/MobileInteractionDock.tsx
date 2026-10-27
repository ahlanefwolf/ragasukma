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
  return (
    <div class="relative z-30 w-full flex flex-col items-center gap-3 px-4 pb-4 pointer-events-auto select-none">
      {/* Primary Floating Action Bar: Symmetrical, pure borderless floating dock */}
      <div class="flex items-center justify-center gap-6">
        {/* 1. Autonomous Agent Monitor Button */}
        <Show when={props.onOpenAgent}>
          <button
            type="button"
            onClick={props.onOpenAgent}
            class={`relative p-3 transition-all active:scale-75 cursor-pointer select-none ${
              props.isAgentRunning
                ? 'text-gi-primary drop-shadow-[0_0_16px_rgba(var(--gi-primary-rgb),0.85)] animate-pulse'
                : 'text-gi-text-primary/80 hover:text-gi-primary-hover drop-shadow-[0_2px_10px_rgba(0,0,0,0.9)]'
            }`}
            title="Agent Tasks"
            aria-label="Agent Tasks"
          >
            <Bot size={24} class={`stroke-[2.2] ${props.isAgentRunning ? 'animate-bounce' : ''}`} />
            <Show when={props.isAgentRunning}>
              <span class="absolute top-1.5 right-1.5 flex h-2 w-2">
                <span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-gi-primary opacity-75"></span>
                <span class="relative inline-flex rounded-full h-2 w-2 bg-gi-primary"></span>
              </span>
            </Show>
          </button>
        </Show>

        {/* 2. Message / Chat Button */}
        <button
          type="button"
          onClick={props.onToggleShowInput}
          class="p-3 text-gi-text-primary/80 hover:text-gi-primary-hover active:scale-75 transition-all drop-shadow-[0_2px_10px_rgba(0,0,0,0.9)] cursor-pointer select-none"
          title="Open Chat"
          aria-label="Open Chat"
        >
          <MessageSquare size={24} class="stroke-[2.2]" />
        </button>

        {/* 3. Primary PTT Mic Button */}
        <button
          type="button"
          onClick={props.onToggleMic}
          class={`p-3 transition-all active:scale-75 cursor-pointer select-none ${
            props.isListening
              ? 'text-gi-status-danger animate-pulse drop-shadow-[0_0_24px_rgba(var(--gi-status-danger-rgb),0.9)]'
              : 'text-gi-primary hover:text-gi-primary-hover drop-shadow-[0_0_18px_rgba(var(--gi-primary-rgb),0.8)]'
          }`}
          title={props.isListening ? 'Stop Listening' : 'Push to Talk (PTT)'}
          aria-label={props.isListening ? 'Stop Listening' : 'Push to Talk (PTT)'}
        >
          <Show when={props.isListening} fallback={<Mic size={28} class="stroke-[2.2]" />}>
            <MicOff size={28} class="stroke-[2.2]" />
          </Show>
        </button>
      </div>
    </div>
  );
};
