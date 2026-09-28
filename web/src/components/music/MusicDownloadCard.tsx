import { Component, Show } from 'solid-js';
import { Download, Loader, AlertCircle, Check } from 'lucide-solid';

interface MusicDownloadCardProps {
  available: boolean;
  searchQuery: () => string;
  onSearchInput: (q: string) => void;
  downloading: () => boolean;
  downloadResult: () => { success: boolean; error?: string } | null;
  onDownload: () => void;
}

export const MusicDownloadCard: Component<MusicDownloadCardProps> = (props) => {
  return (
    <Show when={props.available}>
      <div class="border border-theme-border bg-theme-surface overflow-hidden">
        <div class="px-4 py-3 border-b border-theme-border">
          <h3 class="text-sm font-bold text-theme-text/80">Search & Download</h3>
        </div>
        <div class="p-4 space-y-3">
          <div class="flex gap-2">
            <input
              type="text"
              placeholder="Cari lagu / paste URL YouTube..."
              value={props.searchQuery()}
              onInput={(e) => props.onSearchInput(e.currentTarget.value)}
              onKeyDown={(e) =>
                e.key === 'Enter' && !props.downloading() && props.onDownload()
              }
              disabled={props.downloading()}
              class="flex-1 px-3.5 py-2.5 bg-theme-bg border border-theme-border text-sm text-theme-text placeholder-theme-text/40 focus:outline-none focus:border-theme-cyan/60 transition-colors disabled:opacity-50"
            />
            <button
              onClick={props.onDownload}
              disabled={props.downloading() || !props.searchQuery().trim()}
              class="px-5 py-2.5 bg-theme-cyan hover:bg-theme-cyan/90 disabled:opacity-40 text-theme-bg text-sm font-bold transition-all flex items-center gap-2 shrink-0 border border-theme-cyan"
            >
              <Show when={props.downloading()} fallback={<Download size={16} />}>
                <Loader size={16} class="animate-spin" />
              </Show>
              <span class="hidden sm:inline">
                {props.downloading() ? 'Mendownload...' : 'Download'}
              </span>
            </button>
          </div>

          <Show when={props.downloading()}>
            <div class="flex items-center gap-2.5 px-3.5 py-2.5 bg-theme-cyan/10 border border-theme-cyan/30">
              <Loader size={15} class="animate-spin text-theme-cyan shrink-0" />
              <span class="text-xs text-theme-cyan font-medium">
                Mendownload audio dari YouTube, harap tunggu...
              </span>
            </div>
          </Show>

          <Show when={props.downloadResult() !== null && !props.downloading()}>
            <div
              class={`flex items-center gap-2.5 px-3.5 py-2.5 text-xs font-semibold border ${
                props.downloadResult()!.success
                  ? 'bg-theme-cyan/15 border-theme-cyan/35 text-[#29483A]'
                  : 'bg-[#A84B4B]/10 border-[#A84B4B]/30 text-[#A84B4B]'
              }`}
            >
              <Show
                when={props.downloadResult()!.success}
                fallback={<AlertCircle size={15} />}
              >
                <Check size={15} />
              </Show>
              <span>
                {props.downloadResult()!.success
                  ? 'Download berhasil! Lagu ditambahkan ke library.'
                  : props.downloadResult()!.error || 'Gagal mendownload.'}
              </span>
            </div>
          </Show>
        </div>
      </div>
    </Show>
  );
};
