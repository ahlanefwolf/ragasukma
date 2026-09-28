import { Component, Show, For } from 'solid-js';
import {
  Download,
  RefreshCw,
  Copy,
  Check,
  AlertCircle,
  Loader,
} from 'lucide-solid';
import { YtDlpInfo } from '../../api';

interface YtDlpStatusCardProps {
  checking: () => boolean;
  info: () => YtDlpInfo | null;
  onRefresh: () => void;
  onInstall: () => void;
  installing: () => boolean;
  installMsg: () => string | null;
  copiedCmd: () => string | null;
  onCopyCmd: (cmd: string) => void;
}

export const YtDlpStatusCard: Component<YtDlpStatusCardProps> = (props) => {
  const installCommands = [
    { os: 'Windows', cmd: 'winget install yt-dlp.yt-dlp', color: 'var(--cyan)' },
    { os: 'macOS', cmd: 'brew install yt-dlp', color: 'var(--rose)' },
    { os: 'Linux', cmd: 'pip install yt-dlp', color: 'var(--text)' },
  ];

  return (
    <div class="border border-theme-border bg-theme-surface overflow-hidden">
      <div class="px-4 py-3 border-b border-theme-border flex items-center justify-between">
        <h3 class="text-sm font-bold text-theme-text/80">Status yt-dlp</h3>
        <Show when={!props.checking()}>
          <button
            onClick={props.onRefresh}
            class="text-xs px-3 py-1 bg-theme-card hover:bg-theme-border text-theme-text/80 hover:text-theme-cyan border border-theme-border transition-all flex items-center gap-1.5"
          >
            <RefreshCw size={13} />
            <span>Cek Ulang</span>
          </button>
        </Show>
      </div>

      <div class="p-4">
        <Show when={props.checking()}>
          <div class="flex items-center gap-2 text-xs text-theme-text/70">
            <Loader size={14} class="animate-spin" />
            <span>Memeriksa yt-dlp...</span>
          </div>
        </Show>

        <Show when={!props.checking() && props.info()?.available}>
          <div class="flex items-center gap-2">
            <span class="px-3 py-1 text-xs font-bold bg-theme-cyan/20 text-[#29483A] border border-theme-cyan/40">
              ✓ yt-dlp v{props.info()!.version}
            </span>
            <span class="text-xs text-theme-text/70">Siap digunakan</span>
          </div>
        </Show>

        <Show when={!props.checking() && !props.info()?.available}>
          <div class="space-y-3">
            <div class="flex items-center gap-2 text-xs text-[#A84B4B]">
              <AlertCircle size={14} />
              <span class="font-semibold">yt-dlp tidak ditemukan.</span>
            </div>
            {/* Tombol Auto-Install */}
            <button
              onClick={props.onInstall}
              disabled={props.installing()}
              class="w-full flex items-center justify-center gap-2 px-4 py-3 bg-theme-cyan/20 hover:bg-theme-cyan/30 border border-theme-cyan/40 text-[#29483A] font-semibold text-sm transition-all disabled:opacity-50"
            >
              <Show when={props.installing()} fallback={<Download size={16} />}>
                <Loader size={16} class="animate-spin" />
              </Show>
              <span>{props.installing() ? 'Menginstall yt-dlp...' : 'Install Otomatis'}</span>
            </button>
            <Show when={props.installMsg()}>
              <p
                class={`text-xs px-3 py-2 border font-medium ${
                  props.installMsg()!.startsWith('Gagal')
                    ? 'bg-[#A84B4B]/10 border-[#A84B4B]/30 text-[#A84B4B]'
                    : 'bg-theme-cyan/15 border-theme-cyan/35 text-[#29483A]'
                }`}
              >
                {props.installMsg()}
              </p>
            </Show>
            {/* Fallback manual */}
            <details class="group">
              <summary class="text-xs text-theme-text/60 hover:text-theme-text cursor-pointer select-none">
                Install manual (klik untuk lihat perintah)
              </summary>
              <div class="mt-2 grid grid-cols-1 sm:grid-cols-3 gap-2">
                <For each={installCommands}>
                  {(item) => (
                    <div class="p-3 bg-theme-bg border border-theme-border space-y-2">
                      <span class="text-xs font-bold text-theme-text/70 uppercase tracking-wide">
                        {item.os}
                      </span>
                      <div class="flex items-center gap-1.5 bg-theme-card px-2.5 py-2 border border-theme-border">
                        <code class="text-xs font-mono text-theme-cyan flex-1 break-all">
                          {item.cmd}
                        </code>
                        <button
                          onClick={() => props.onCopyCmd(item.cmd)}
                          class="shrink-0 p-1 hover:bg-theme-border transition-colors text-theme-text/60 hover:text-theme-cyan border border-transparent hover:border-theme-border"
                          title="Copy"
                        >
                          <Show
                            when={props.copiedCmd() === item.cmd}
                            fallback={<Copy size={13} />}
                          >
                            <Check size={13} class="text-theme-cyan" />
                          </Show>
                        </button>
                      </div>
                    </div>
                  )}
                </For>
              </div>
            </details>
          </div>
        </Show>
      </div>
    </div>
  );
};
