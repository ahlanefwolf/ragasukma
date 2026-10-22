import { Component, createSignal, Show, onMount, onCleanup } from 'solid-js';
import { AlertTriangle, Trash2, HelpCircle, X } from 'lucide-solid';

export interface ConfirmOptions {
  title?: string;
  message: string;
  confirmText?: string;
  cancelText?: string;
  isDanger?: boolean;
}

interface ConfirmState extends ConfirmOptions {
  resolve: (value: boolean) => void;
}

const [confirmState, setConfirmState] = createSignal<ConfirmState | null>(null);

/**
 * Reusable promise-based confirm dialog matching Ragasukma cyberpunk aesthetic.
 * Can be called with `const ok = await showConfirm(...)`.
 */
export function showConfirm(options: ConfirmOptions | string): Promise<boolean> {
  const opts: ConfirmOptions = typeof options === 'string' ? { message: options } : options;
  return new Promise<boolean>((resolve) => {
    setConfirmState({
      ...opts,
      resolve,
    });
  });
}

export const ConfirmModal: Component = () => {
  const handleConfirm = () => {
    const s = confirmState();
    if (s) {
      s.resolve(true);
      setConfirmState(null);
    }
  };

  const handleCancel = () => {
    const s = confirmState();
    if (s) {
      s.resolve(false);
      setConfirmState(null);
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (!confirmState()) return;
    if (e.key === 'Escape') {
      e.preventDefault();
      handleCancel();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      handleConfirm();
    }
  };

  onMount(() => {
    window.addEventListener('keydown', handleKeyDown);
  });

  onCleanup(() => {
    window.removeEventListener('keydown', handleKeyDown);
  });

  return (
    <Show when={confirmState()}>
      {(state) => (
        <div class="fixed inset-0 z-[1000] flex items-center justify-center p-4 bg-black/75 backdrop-blur-sm animate-fade-in select-none">
          {/* Backdrop Click */}
          <div class="absolute inset-0" onClick={handleCancel} />

          {/* Modal Container */}
          <div
            class={`relative w-full max-w-md bg-theme-surface border ${
              state().isDanger ? 'border-theme-rose/60' : 'border-theme-cyan/60'
            } shadow-2xl p-6 space-y-5 animate-scale-in text-theme-text font-sans`}
            onClick={(e) => e.stopPropagation()}
          >
            {/* Header / Title */}
            <div class="flex items-start gap-4">
              <div
                class={`p-3 border shrink-0 ${
                  state().isDanger
                    ? 'bg-theme-rose/15 text-theme-rose border-theme-rose/40'
                    : 'bg-theme-cyan/15 text-theme-cyan border-theme-cyan/40'
                }`}
              >
                {state().isDanger ? <AlertTriangle size={24} /> : <HelpCircle size={24} />}
              </div>
              <div class="space-y-1 flex-1 min-w-0">
                <h3 class="text-base font-heading font-bold uppercase tracking-wider text-theme-text">
                  {state().title || (state().isDanger ? 'Konfirmasi Tindakan' : 'Konfirmasi')}
                </h3>
                <p class="text-xs text-theme-text/80 leading-relaxed font-sans break-words whitespace-pre-wrap">
                  {state().message}
                </p>
              </div>
              <button
                type="button"
                onClick={handleCancel}
                class="text-theme-text/50 hover:text-theme-text transition-colors p-1"
                title="Tutup (Esc)"
              >
                <X size={16} />
              </button>
            </div>

            {/* Actions */}
            <div class="flex items-center justify-end gap-3 pt-2 border-t border-theme-border/60">
              <button
                type="button"
                onClick={handleCancel}
                class="px-4 py-2 text-xs font-semibold uppercase tracking-wider bg-theme-card border border-theme-border hover:bg-theme-border text-theme-text transition-all"
              >
                {state().cancelText || 'Batal'}
              </button>
              <button
                type="button"
                onClick={handleConfirm}
                autofocus
                class={`px-4 py-2 text-xs font-bold uppercase tracking-wider transition-all flex items-center gap-1.5 shadow-sm ${
                  state().isDanger
                    ? 'bg-theme-rose text-white border border-theme-rose hover:brightness-110 shadow-theme-rose/20'
                    : 'bg-theme-cyan text-theme-card border border-theme-cyan hover:brightness-110'
                }`}
              >
                {state().isDanger && <Trash2 size={13} />}
                <span>{state().confirmText || (state().isDanger ? 'Hapus' : 'Lanjutkan')}</span>
              </button>
            </div>
          </div>
        </div>
      )}
    </Show>
  );
};
