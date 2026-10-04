import { Component, Show, For, createSignal } from 'solid-js';
import {
  Music,
  Play,
  RefreshCw,
  Copy,
  Check,
  Loader,
  ListMusic,
  Trash2,
  Folder,
  Search,
  ChevronLeft,
  ChevronRight,
} from 'lucide-solid';
import { YtTrackInfo } from '../../api';
import { globalStatus, formatDuration } from '../../playerStore';

interface MusicTrackTableProps {
  library: () => YtTrackInfo[];
  musicDir: () => string;
  loading: () => boolean;
  filterQuery: () => string;
  onFilterChange: (q: string) => void;
  currentPage: () => number;
  onPageChange: (p: number) => void;
  pageSize: () => number;
  onPageSizeChange: (s: number) => void;
  filteredTracks: () => YtTrackInfo[];
  paginatedTracks: () => YtTrackInfo[];
  totalPages: () => number;
  onRefresh: () => void;
  onPlay: (t: YtTrackInfo) => void;
  onDelete: (filename: string, title: string) => void;
  showToast: (msg: string) => void;
}

export const MusicTrackTable: Component<MusicTrackTableProps> = (props) => {
  const [copiedDir, setCopiedDir] = createSignal(false);

  return (
    <div class="border border-theme-border bg-theme-surface overflow-hidden shadow-lg">
      {/* Library Header & Local Search */}
      <div class="px-4 py-3.5 border-b border-theme-border flex flex-col sm:flex-row sm:items-center justify-between gap-3 bg-theme-card/40">
        <div class="flex items-center gap-2.5">
          <div class="w-8 h-8 bg-theme-cyan/15 flex items-center justify-center border border-theme-cyan/30">
            <ListMusic size={16} class="text-theme-cyan" />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h3 class="text-sm font-bold text-theme-text">Library Playlist</h3>
              <Show when={props.library().length > 0}>
                <span class="px-2 py-0.5 text-xs font-bold bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/30">
                  {props.library().length} Tracks
                </span>
              </Show>
            </div>
            <Show when={props.musicDir()}>
              <div
                class="flex items-center gap-1.5 mt-0.5 text-xs text-theme-text/60 hover:text-theme-cyan cursor-pointer transition-colors max-w-[260px] sm:max-w-md truncate"
                title="Click to copy music folder path"
                onClick={() => {
                  navigator.clipboard.writeText(props.musicDir());
                  setCopiedDir(true);
                  setTimeout(() => setCopiedDir(false), 2000);
                  props.showToast('Music folder path copied to clipboard!');
                }}
              >
                <Folder size={12} class="shrink-0 text-theme-cyan/80" />
                <span class="truncate font-mono">{props.musicDir()}</span>
                <Show
                  when={copiedDir()}
                  fallback={<Copy size={11} class="shrink-0 opacity-70" />}
                >
                  <Check size={11} class="shrink-0 text-emerald-400" />
                </Show>
              </div>
            </Show>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <div class="relative flex-1 sm:w-52">
            <Search
              size={14}
              class="absolute left-2.5 top-1/2 -translate-y-1/2 text-theme-text/40"
            />
            <input
              type="text"
              placeholder="Search tracks in library..."
              value={props.filterQuery()}
              onInput={(e) => {
                props.onFilterChange(e.currentTarget.value);
                props.onPageChange(1);
              }}
              class="w-full pl-8 pr-3 py-1.5 bg-theme-bg border border-theme-border text-xs text-theme-text placeholder-theme-text/40 focus:outline-none focus:border-theme-cyan/60 transition-colors"
            />
          </div>

          <button
            onClick={props.onRefresh}
            disabled={props.loading()}
            class="px-3.5 py-1.5 bg-theme-card hover:bg-theme-border text-theme-text/80 hover:text-theme-cyan border border-theme-border transition-all flex items-center gap-1.5 disabled:opacity-40 text-xs font-semibold"
          >
            <RefreshCw
              size={13}
              class={props.loading() ? 'animate-spin' : ''}
            />
            <span class="hidden xs:inline">Refresh</span>
          </button>
        </div>
      </div>

      {/* Tracks List */}
      <div class="divide-y divide-theme-border/60">
        <Show when={props.loading()}>
          <div class="p-8 flex items-center justify-center gap-2 text-xs text-theme-text/60">
            <Loader size={16} class="animate-spin text-theme-cyan" />
            <span>Loading track list...</span>
          </div>
        </Show>

        <Show when={!props.loading() && props.library().length === 0}>
          <div class="p-10 text-center text-xs text-theme-text/50 space-y-3">
            <div class="w-14 h-14 bg-theme-card border border-theme-border flex items-center justify-center mx-auto">
              <Music size={26} class="text-theme-text/40" />
            </div>
            <p class="font-medium text-sm text-theme-text/70">
              Music library is empty
            </p>
            <p class="text-xs text-theme-text/50">
              Use Search & Download above to save tracks from
              YouTube.
            </p>
          </div>
        </Show>

        <For each={props.paginatedTracks()}>
          {(track, idx) => {
            const globalIndex = () =>
              (props.currentPage() - 1) * props.pageSize() + idx() + 1;
            const isCurrentPlaying = () =>
              globalStatus().current_track?.filename === track.filename &&
              (globalStatus().is_playing || globalStatus().is_paused);

            return (
              <div
                class={`flex items-center gap-3 px-4 py-3 transition-all duration-150 group ${
                  isCurrentPlaying()
                    ? 'bg-theme-cyan/10 border-l-4 border-l-theme-cyan'
                    : 'hover:bg-theme-card/60'
                }`}
              >
                {/* Track Number / Soundwave */}
                <div class="w-6 text-center shrink-0">
                  <Show
                    when={isCurrentPlaying() && globalStatus().is_playing}
                    fallback={
                      <span class="text-xs font-mono text-theme-text/50 group-hover:text-theme-cyan transition-colors">
                        {globalIndex()}
                      </span>
                    }
                  >
                    <div class="flex items-end justify-center gap-0.5 h-3.5">
                      <span class="w-1 bg-theme-cyan animate-[pulse_0.6s_ease-in-out_infinite] h-2" />
                      <span class="w-1 bg-theme-rose animate-[pulse_0.9s_ease-in-out_infinite] h-3.5" />
                      <span class="w-1 bg-theme-cyan animate-[pulse_0.75s_ease-in-out_infinite] h-2.5" />
                    </div>
                  </Show>
                </div>

                {/* Thumbnail */}
                <div class="relative shrink-0 w-11 h-11 overflow-hidden bg-theme-card border border-theme-border">
                  <Show
                    when={track.thumbnail}
                    fallback={
                      <div class="w-full h-full flex items-center justify-center">
                        <Music size={18} class="text-theme-cyan/40" />
                      </div>
                    }
                  >
                    <img
                      src={track.thumbnail}
                      alt=""
                      class="w-full h-full object-cover"
                    />
                  </Show>
                </div>

                {/* Track Title & Artist */}
                <div class="flex-1 min-w-0">
                  <p
                    class={`text-sm font-bold truncate transition-colors ${
                      isCurrentPlaying()
                        ? 'text-theme-cyan'
                        : 'text-theme-text group-hover:text-theme-cyan'
                    }`}
                  >
                    {track.title}
                  </p>
                  <p class="text-xs text-theme-text/60 truncate mt-0.5 font-medium">
                    {track.uploader || 'Local Library'}
                  </p>
                </div>

                {/* Track Duration & Actions */}
                <div class="flex items-center gap-2 shrink-0">
                  <span class="text-xs font-mono text-theme-text/50 hidden sm:inline px-2">
                    {formatDuration(track.duration)}
                  </span>

                  <button
                    onClick={() => props.onPlay(track)}
                    class={`p-2.5 transition-all border active:scale-95 ${
                      isCurrentPlaying()
                        ? 'bg-theme-cyan text-theme-bg border-theme-cyan font-bold shadow-md shadow-theme-cyan/20'
                        : 'bg-theme-card hover:bg-theme-cyan/20 text-theme-text/80 hover:text-theme-cyan border-theme-border hover:border-theme-cyan/40'
                    }`}
                    title="Play This Track"
                  >
                    <Play
                      size={15}
                      class={isCurrentPlaying() ? 'fill-current' : ''}
                    />
                  </button>

                  <button
                    onClick={() => props.onDelete(track.filename, track.title)}
                    class="p-2.5 bg-theme-card hover:bg-theme-rose/20 text-theme-text/50 hover:text-theme-rose border border-theme-border hover:border-theme-rose/40 transition-all active:scale-95"
                    title="Delete from Library"
                  >
                    <Trash2 size={15} />
                  </button>
                </div>
              </div>
            );
          }}
        </For>
      </div>

      {/* Pagination Bar */}
      <Show when={props.filteredTracks().length > 0}>
        <div class="px-4 py-3 bg-theme-card/30 border-t border-theme-border flex flex-col sm:flex-row items-center justify-between gap-3 text-xs">
          {/* Info Range */}
          <div class="text-theme-text/60 text-xs">
            Showing{' '}
            <span class="font-semibold text-theme-text font-mono">
              {(props.currentPage() - 1) * props.pageSize() + 1}
            </span>
            {' - '}
            <span class="font-semibold text-theme-text font-mono">
              {Math.min(
                props.currentPage() * props.pageSize(),
                props.filteredTracks().length
              )}
            </span>
            {' of '}
            <span class="font-semibold text-theme-cyan font-mono">
              {props.filteredTracks().length}
            </span>{' '}
            tracks
          </div>

          {/* Pagination Controls */}
          <div class="flex items-center gap-2">
            {/* Page Size Selector */}
            <div class="flex items-center gap-1.5 text-xs text-theme-text/60 mr-2">
              <span>Per page:</span>
              <select
                value={props.pageSize()}
                onChange={(e) => {
                  props.onPageSizeChange(parseInt(e.currentTarget.value, 10));
                  props.onPageChange(1);
                }}
                class="bg-theme-bg border border-theme-border px-2 py-1 text-theme-text text-xs focus:outline-none focus:border-theme-cyan/50 cursor-pointer"
              >
                <option value={10}>10</option>
                <option value={20}>20</option>
                <option value={50}>50</option>
              </select>
            </div>

            {/* Prev Button */}
            <button
              onClick={() => props.onPageChange(Math.max(1, props.currentPage() - 1))}
              disabled={props.currentPage() <= 1}
              class="p-1.5 bg-theme-card hover:bg-theme-border border border-theme-border text-theme-text/80 hover:text-theme-cyan disabled:opacity-30 disabled:pointer-events-none transition-all"
              title="Halaman Sebelumnya"
            >
              <ChevronLeft size={15} />
            </button>

            {/* Page Indicator */}
            <div class="px-3 py-1 bg-theme-bg border border-theme-border font-mono text-xs text-theme-text font-semibold">
              <span>{props.currentPage()}</span>
              <span class="text-theme-text/40 mx-1">/</span>
              <span>{props.totalPages()}</span>
            </div>

            {/* Next Button */}
            <button
              onClick={() =>
                props.onPageChange(Math.min(props.totalPages(), props.currentPage() + 1))
              }
              disabled={props.currentPage() >= props.totalPages()}
              class="p-1.5 bg-theme-card hover:bg-theme-border border border-theme-border text-theme-text/80 hover:text-theme-cyan disabled:opacity-30 disabled:pointer-events-none transition-all"
              title="Halaman Berikutnya"
            >
              <ChevronRight size={15} />
            </button>
          </div>
        </div>
      </Show>
    </div>
  );
};
