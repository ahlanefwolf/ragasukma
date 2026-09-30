import { Component, Show } from 'solid-js';
import { Send, Mic, MicOff, MessageSquare, X, Trash2, CheckCircle2 } from 'lucide-solid';

interface MobileInteractionDockProps {
  inputText: string;
  onInputText: (val: string) => void;
  onSend: () => void;
  isSending: boolean;
  isListening: boolean;
  onToggleMic: () => void;
  lastSentText: string;
  showInput: boolean;
  onToggleShowInput: () => void;
  onDismissLastSent?: () => void;
}

export const MobileInteractionDock: Component<MobileInteractionDockProps> = (props) => {
  let textareaRef: HTMLTextAreaElement | undefined;

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      props.onSend();
    }
  };

  return (
    <div class="relative z-30 w-full flex flex-col items-center gap-2.5 px-4 pb-4 pointer-events-auto">
      {/* 1. Status Aksi Terakhir (Clean Minimalist Pill) */}
      <Show when={props.lastSentText}>
        <div class="max-w-[92vw] px-3.5 py-1.5 bg-theme-surface border border-theme-border flex items-center gap-2 shadow-sm animate-in fade-in duration-200">
          <CheckCircle2 size={13} class="text-theme-cyan shrink-0" />
          <span class="text-[11px] font-mono text-theme-cyan font-bold truncate max-w-[260px]">
            "{props.lastSentText}"
          </span>
          <Show when={props.onDismissLastSent}>
            <button
              type="button"
              onClick={props.onDismissLastSent}
              class="text-theme-muted hover:text-theme-text ml-1"
              title="Tutup status"
            >
              <X size={12} />
            </button>
          </Show>
        </div>
      </Show>

      {/* 2. Floating Text Input Form (Muncul Rapi di Atas Tombol Saat Toggle Pesan Aktif) */}
      <Show when={props.showInput}>
        <div class="w-full max-w-sm bg-theme-surface border border-theme-border p-3 shadow-xl space-y-2 animate-in fade-in slide-in-from-bottom-2 duration-150">
          <div class="relative flex items-center">
            <textarea
              ref={textareaRef}
              value={props.inputText}
              onInput={(e) => props.onInputText(e.currentTarget.value)}
              onKeyDown={handleKeyDown}
              placeholder="Ketik pesan atau perintah untuk Sukma..."
              rows={2}
              disabled={props.isSending}
              class="w-full pl-3 pr-16 py-2 bg-theme-card border border-theme-border text-xs text-theme-text placeholder:text-theme-muted/60 focus:border-theme-cyan transition-colors resize-none leading-relaxed"
            />
            <div class="absolute right-2 top-2 flex items-center gap-1.5">
              <Show when={props.inputText.length > 0}>
                <button
                  type="button"
                  onClick={() => props.onInputText('')}
                  class="p-1 text-theme-muted hover:text-theme-text transition-colors"
                  title="Hapus teks"
                >
                  <Trash2 size={13} />
                </button>
              </Show>
              <button
                type="button"
                onClick={props.onToggleShowInput}
                class="p-1 text-theme-muted hover:text-theme-text transition-colors"
                title="Tutup input"
              >
                <X size={14} />
              </button>
            </div>
          </div>

          <div class="flex items-center justify-between gap-2">
            <span class="text-[10px] font-mono text-theme-muted">
              Tekan Enter untuk kirim
            </span>
            <button
              type="button"
              onClick={props.onSend}
              disabled={!props.inputText.trim() || props.isSending}
              class="px-4 py-1.5 bg-theme-cyan text-theme-bg font-bold text-xs flex items-center gap-1.5 disabled:opacity-40 disabled:cursor-not-allowed active:scale-95 transition-all shadow-sm"
            >
              <Send size={12} class={props.isSending ? 'animate-pulse' : ''} />
              <span>{props.isSending ? 'Mengirim...' : 'Kirim'}</span>
            </button>
          </div>
        </div>
      </Show>

      {/* 3. Bar Tombol Utama: Ukuran Sama Persis, Simetris, dan Nyaman Disentuh */}
      <div class="flex items-center justify-center gap-5">
        {/* Tombol Pesan / Keyboard (Ukuran 56x56 px, Simetris Sempurna dengan Tombol Mic) */}
        <button
          type="button"
          onClick={props.onToggleShowInput}
          style={{ "border-radius": "9999px" }}
          class={`w-14 h-14 border flex items-center justify-center transition-all shadow-md active:scale-90 select-none ${
            props.showInput
              ? 'bg-theme-cyan text-theme-bg border-theme-cyan shadow-lg'
              : 'bg-theme-surface/90 border-theme-border text-theme-cyan hover:border-theme-cyan'
          }`}
          title={props.showInput ? 'Tutup Input Teks' : 'Ketik Pesan'}
        >
          <Show when={props.showInput} fallback={<MessageSquare size={22} />}>
            <X size={22} />
          </Show>
        </button>

        {/* Tombol Mic PTT (Ukuran 56x56 px, Simetris Sempurna dengan Tombol Pesan) */}
        <button
          type="button"
          onClick={props.onToggleMic}
          style={{ "border-radius": "9999px" }}
          class={`w-14 h-14 border flex items-center justify-center transition-all shadow-md active:scale-90 select-none ${
            props.isListening
              ? 'bg-rose-500 text-white border-rose-300 animate-pulse shadow-lg scale-105'
              : 'bg-theme-cyan text-theme-bg border-theme-cyan hover:opacity-90 shadow-lg'
          }`}
          title={props.isListening ? 'Hentikan Mendengar' : 'Mulai Bicara (PTT)'}
        >
          <Show when={props.isListening} fallback={<Mic size={24} />}>
            <MicOff size={24} />
          </Show>
        </button>
      </div>
    </div>
  );
};
