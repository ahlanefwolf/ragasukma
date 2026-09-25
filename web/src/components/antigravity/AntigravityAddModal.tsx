import { Component, createSignal, Show } from 'solid-js';
import { Rocket, KeyRound, Mail, FolderGit2, X, Plus, RefreshCw } from 'lucide-solid';

interface AntigravityAddModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (refreshToken: string, email?: string, projectId?: string) => Promise<boolean>;
}

export const AntigravityAddModal: Component<AntigravityAddModalProps> = (props) => {
  const [refreshToken, setRefreshToken] = createSignal('');
  const [email, setEmail] = createSignal('');
  const [projectId, setProjectId] = createSignal('');
  const [isSubmitting, setIsSubmitting] = createSignal(false);

  const handleSubmit = async (e: Event) => {
    e.preventDefault();
    if (!refreshToken().trim()) return;

    setIsSubmitting(true);
    const ok = await props.onSubmit(
      refreshToken().trim(),
      email().trim() || undefined,
      projectId().trim() || undefined
    );
    setIsSubmitting(false);

    if (ok) {
      setRefreshToken('');
      setEmail('');
      setProjectId('');
      props.onClose();
    }
  };

  return (
    <Show when={props.isOpen}>
      <div class="fixed inset-0 z-[100] !mt-0 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4 animate-fade-in select-none">
        <div class="max-w-md w-full p-6 bg-theme-surface border border-theme-border shadow-2xl space-y-5 animate-in fade-in zoom-in-95 duration-200">
          <div class="flex items-center justify-between pb-3 border-b border-theme-border">
            <div class="flex items-center gap-2.5">
              <div class="p-2 bg-theme-rose/15 text-theme-rose border border-theme-rose/30">
                <Rocket size={18} />
              </div>
              <div>
                <h3 class="text-base font-bold text-theme-text m-0 leading-tight">Input Token Antigravity Manual</h3>
                <p class="text-xs text-theme-text/50">Google OAuth Refresh Token</p>
              </div>
            </div>
            <button
              onClick={props.onClose}
              class="p-1.5 text-theme-text/50 hover:text-theme-text hover:bg-theme-card transition-colors"
            >
              <X size={18} />
            </button>
          </div>

          <form onSubmit={handleSubmit} class="space-y-4 text-xs">
            <div class="space-y-1.5">
              <label class="font-bold text-theme-text flex items-center gap-1.5">
                <KeyRound size={13} class="text-theme-cyan" />
                Refresh Token Google OAuth <span class="text-theme-rose">*</span>
              </label>
              <textarea
                value={refreshToken()}
                onInput={(e) => setRefreshToken(e.currentTarget.value)}
                placeholder="Contoh: 1//04..."
                rows={3}
                required
                class="w-full px-3.5 py-2.5 bg-theme-bg border border-theme-border focus:border-theme-cyan outline-none text-theme-text font-mono text-xs transition-colors select-text"
              />
            </div>

            <div class="space-y-1.5">
              <label class="font-bold text-theme-text flex items-center gap-1.5">
                <Mail size={13} class="text-theme-cyan" />
                Email Akun (Opsional)
              </label>
              <input
                type="email"
                value={email()}
                onInput={(e) => setEmail(e.currentTarget.value)}
                placeholder="user@gmail.com"
                class="w-full px-3.5 py-2.5 bg-theme-bg border border-theme-border focus:border-theme-cyan outline-none text-theme-text text-xs transition-colors select-text"
              />
            </div>

            <div class="space-y-1.5">
              <label class="font-bold text-theme-text flex items-center gap-1.5">
                <FolderGit2 size={13} class="text-theme-cyan" />
                Project ID Google Cloud (Opsional)
              </label>
              <input
                type="text"
                value={projectId()}
                onInput={(e) => setProjectId(e.currentTarget.value)}
                placeholder="Biarkan kosong untuk auto-discovery"
                class="w-full px-3.5 py-2.5 bg-theme-bg border border-theme-border focus:border-theme-cyan outline-none text-theme-text font-mono text-xs transition-colors select-text"
              />
            </div>

            <div class="flex items-center justify-end gap-2.5 pt-2">
              <button
                type="button"
                onClick={props.onClose}
                class="px-4 py-2.5 bg-theme-card hover:bg-theme-border text-theme-text/80 font-bold transition-all"
              >
                Batal
              </button>
              <button
                type="submit"
                disabled={isSubmitting()}
                class="px-5 py-2.5 bg-theme-cyan text-theme-card font-bold shadow-md anime-shadow-green hover:opacity-95 active:scale-95 transition-all flex items-center gap-2"
              >
                <Show when={isSubmitting()} fallback={<Plus size={16} />}>
                  <RefreshCw size={16} class="animate-spin" />
                </Show>
                <span>{isSubmitting() ? 'Menyimpan...' : 'Simpan Token'}</span>
              </button>
            </div>
          </form>
        </div>
      </div>
    </Show>
  );
};
