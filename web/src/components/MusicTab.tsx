import { Component, createSignal, onMount, onCleanup, For, Show } from 'solid-js';
import {
  Music,
  Download,
  Play,
  Pause,
  Square,
  RefreshCw,
  Copy,
  Check,
  AlertCircle,
  Loader,
  ListMusic,
  Trash2,
  Folder,
  SkipBack,
  SkipForward,
  Search,
  Volume2,
  VolumeX,
  PictureInPicture2,
  ChevronLeft,
  ChevronRight,
} from 'lucide-solid';
import { pipManager } from '../services/pipService';
import { api, YtDlpInfo, YtTrackInfo } from '../api';
import {
  globalStatus,
  currentTime,
  duration,
  volume,
  formatDuration,
  playTrack,
  pauseMusic,
  resumeMusic,
  stopMusic,
  nextTrack,
  prevTrack,
  seekMusic,
  changeVolume,
} from '../playerStore';

interface MusicTabProps {
  showToast: (msg: string) => void;
}

// Module-level persistent cache agar tidak re-fetch / re-load saat user berpindah tab
const [cachedYtDlpInfo, setCachedYtDlpInfo] = createSignal<YtDlpInfo | null>(null);
const [cachedLibrary, setCachedLibrary] = createSignal<YtTrackInfo[]>([]);
const [cachedMusicDir, setCachedMusicDir] = createSignal<string>('');
let isFirstMusicTabMount = true;

export const MusicTab: Component<MusicTabProps> = (props) => {
  const [checkingYtDlp, setCheckingYtDlp] = createSignal(cachedYtDlpInfo() === null);
  const [searchQuery, setSearchQuery] = createSignal('');
  const [downloading, setDownloading] = createSignal(false);
  const [downloadResult, setDownloadResult] = createSignal<{ success: boolean; error?: string } | null>(null);

  const [loadingLibrary, setLoadingLibrary] = createSignal(cachedLibrary().length === 0);
  const [filterQuery, setFilterQuery] = createSignal('');
  const [currentPage, setCurrentPage] = createSignal(1);
  const [pageSize, setPageSize] = createSignal(10);

  const [copiedCmd, setCopiedCmd] = createSignal<string | null>(null);
  const [installing, setInstalling] = createSignal(false);
  const [installMsg, setInstallMsg] = createSignal<string | null>(null);

  const [copiedDir, setCopiedDir] = createSignal(false);

  const ytDlpInfo = () => cachedYtDlpInfo();
  const library = () => cachedLibrary();
  const musicDir = () => cachedMusicDir();

  const filteredTracks = () =>
    library().filter(
      (t) =>
        !filterQuery().trim() ||
        t.title.toLowerCase().includes(filterQuery().toLowerCase()) ||
        (t.uploader && t.uploader.toLowerCase().includes(filterQuery().toLowerCase()))
    );

  const totalPages = () => Math.max(1, Math.ceil(filteredTracks().length / pageSize()));

  const paginatedTracks = () => {
    const page = Math.min(currentPage(), totalPages());
    const size = pageSize();
    const start = (page - 1) * size;
    return filteredTracks().slice(start, start + size);
  };

  const checkYtDlp = async (silent = false) => {
    if (!silent) setCheckingYtDlp(true);
    try {
      const info = await api.checkYtDlp();
      setCachedYtDlpInfo(info);
    } finally {
      setCheckingYtDlp(false);
    }
  };

  const loadLibrary = async (silent = false) => {
    if (!silent) setLoadingLibrary(true);
    try {
      const res = await api.listMusic();
      if (Array.isArray(res)) {
        setCachedLibrary(res);
      } else {
        setCachedLibrary(res?.tracks || []);
        if (res?.music_dir) {
          setCachedMusicDir(res.music_dir);
        }
      }
    } finally {
      setLoadingLibrary(false);
    }
  };

  let isComponentMounted = true;

  onMount(() => {
    isComponentMounted = true;
    if (isFirstMusicTabMount) {
      isFirstMusicTabMount = false;
      checkYtDlp(false);
      loadLibrary(false);
    } else {
      // Jika sudah pernah dimuat, lakukan silent background sync tanpa spinner yang mengganggu
      checkYtDlp(true);
      loadLibrary(true);
    }
  });

  onCleanup(() => {
    isComponentMounted = false;
  });

  const handleDownload = async () => {
    const q = searchQuery().trim();
    if (!q) return;
    setDownloading(true);
    setDownloadResult(null);
    const result = await api.downloadMusic(q);
    setDownloading(false);
    if (result.success) {
      props.showToast('Download selesai!');
      setSearchQuery('');
      await loadLibrary();
      if (result.track) {
        await playTrack(result.track);
      }
    } else {
      props.showToast(result.error || 'Gagal mendownload');
    }
  };

  const handlePlay = (track: YtTrackInfo) => {
    playTrack(track);
    props.showToast('Memutar lagu...');
  };

  const handleSeek = (e: Event) => {
    const val = parseFloat((e.target as HTMLInputElement).value);
    seekMusic(val);
  };

  const handleDelete = async (filename: string, title: string) => {
    if (!confirm(`Hapus lagu "${title}" dari library?`)) return;
    const ok = await api.deleteMusic(filename);
    if (ok) {
      props.showToast('Lagu berhasil dihapus');
      await loadLibrary();
    } else {
      props.showToast('Gagal menghapus lagu');
    }
  };

  const handleCopyCmd = (cmd: string) => {
    navigator.clipboard.writeText(cmd).then(() => {
      setCopiedCmd(cmd);
      setTimeout(() => setCopiedCmd(null), 2000);
    });
  };

  const handleInstall = async () => {
    setInstalling(true);
    setInstallMsg(null);
    const res = await api.installYtDlp();
    setInstalling(false);
    if (res.success) {
      setInstallMsg(res.message);
      if (res.info) setCachedYtDlpInfo(res.info);
      props.showToast('yt-dlp berhasil diinstall!');
      await loadLibrary();
    } else {
      setInstallMsg('Gagal: ' + res.message);
      props.showToast('Gagal install yt-dlp');
    }
  };

  const installCommands = [
    { os: 'Windows', cmd: 'winget install yt-dlp.yt-dlp', color: 'var(--cyan)' },
    { os: 'macOS', cmd: 'brew install yt-dlp', color: 'var(--rose)' },
    { os: 'Linux', cmd: 'pip install yt-dlp', color: 'var(--text)' },
  ];

  return (
    <div class="space-y-6">
      {/* Header Banner */}
      <div class="bg-gradient-to-r from-theme-card via-theme-surface to-theme-bg p-5 sm:p-6 border border-theme-border flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <div class="flex items-center gap-2 flex-wrap">
            <span class="px-2.5 py-1 text-xs font-bold uppercase tracking-wider bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/30">
              yt-dlp Powered
            </span>
            <span class="px-2 py-0.5 text-xs font-mono bg-theme-bg text-theme-rose border border-theme-border">
              music-player
            </span>
          </div>
          <h2 class="text-2xl sm:text-3xl font-heading tracking-tight text-theme-text mt-2">
            Music Player
          </h2>
          <p class="text-xs sm:text-sm text-theme-text/70 mt-1 max-w-xl">
            Download dan putar musik dari YouTube langsung di Ragasukma menggunakan yt-dlp.
          </p>
        </div>
        <div class="shrink-0 w-14 h-14 bg-theme-cyan/10 border border-theme-cyan/30 flex items-center justify-center">
          <Music size={28} class="text-theme-cyan" />
        </div>
      </div>

      {/* yt-dlp Status Bar */}
      <div class="border border-theme-border bg-theme-surface overflow-hidden">
        <div class="px-4 py-3 border-b border-theme-border flex items-center justify-between">
          <span class="text-xs font-bold text-theme-text/70 uppercase tracking-wider">Status yt-dlp</span>
          <Show when={!checkingYtDlp()}>
            <button
              onClick={() => checkYtDlp()}
              class="text-xs px-3 py-1 bg-theme-card hover:bg-theme-border text-theme-text/80 hover:text-theme-cyan border border-theme-border transition-all flex items-center gap-1.5"
            >
              <RefreshCw size={13} />
              <span>Cek Ulang</span>
            </button>
          </Show>
        </div>

        <div class="p-4">
          <Show when={checkingYtDlp()}>
            <div class="flex items-center gap-2 text-xs text-theme-text/70">
              <Loader size={14} class="animate-spin" />
              <span>Memeriksa yt-dlp...</span>
            </div>
          </Show>

          <Show when={!checkingYtDlp() && ytDlpInfo()?.available}>
            <div class="flex items-center gap-2">
              <span class="px-3 py-1 text-xs font-bold bg-theme-cyan/20 text-[#29483A] border border-theme-cyan/40">
                ✓ yt-dlp v{ytDlpInfo()!.version}
              </span>
              <span class="text-xs text-theme-text/70">Siap digunakan</span>
            </div>
          </Show>

          <Show when={!checkingYtDlp() && !ytDlpInfo()?.available}>
            <div class="space-y-3">
              <div class="flex items-center gap-2 text-xs text-[#A84B4B]">
                <AlertCircle size={14} />
                <span class="font-semibold">yt-dlp tidak ditemukan.</span>
              </div>
              {/* Tombol Auto-Install */}
              <button
                onClick={handleInstall}
                disabled={installing()}
                class="w-full flex items-center justify-center gap-2 px-4 py-3 bg-theme-cyan/20 hover:bg-theme-cyan/30 border border-theme-cyan/40 text-[#29483A] font-semibold text-sm transition-all disabled:opacity-50"
              >
                <Show when={installing()} fallback={<Download size={16} />}>
                  <Loader size={16} class="animate-spin" />
                </Show>
                <span>{installing() ? 'Menginstall yt-dlp...' : 'Install Otomatis'}</span>
              </button>
              <Show when={installMsg()}>
                <p class={`text-xs px-3 py-2 border font-medium ${
                  installMsg()!.startsWith('Gagal')
                    ? 'bg-[#A84B4B]/10 border-[#A84B4B]/30 text-[#A84B4B]'
                    : 'bg-theme-cyan/15 border-theme-cyan/35 text-[#29483A]'
                }`}>{installMsg()}</p>
              </Show>
              {/* Fallback manual */}
              <details class="group">
                <summary class="text-xs text-theme-text/60 hover:text-theme-text cursor-pointer select-none">Install manual (klik untuk lihat perintah)</summary>
                <div class="mt-2 grid grid-cols-1 sm:grid-cols-3 gap-2">
                  <For each={installCommands}>
                    {(item) => (
                      <div class="p-3 bg-theme-bg border border-theme-border space-y-2">
                        <span class="text-xs font-bold text-theme-text/70 uppercase tracking-wide">{item.os}</span>
                        <div class="flex items-center gap-1.5 bg-theme-card px-2.5 py-2 border border-theme-border">
                          <code class="text-xs font-mono text-theme-cyan flex-1 break-all">{item.cmd}</code>
                          <button
                            onClick={() => handleCopyCmd(item.cmd)}
                            class="shrink-0 p-1 hover:bg-theme-border transition-colors text-theme-text/60 hover:text-theme-cyan border border-transparent hover:border-theme-border"
                            title="Copy"
                          >
                            <Show when={copiedCmd() === item.cmd} fallback={<Copy size={13} />}>
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

      {/* Search & Download */}
      <Show when={ytDlpInfo()?.available}>
        <div class="border border-theme-border bg-theme-surface overflow-hidden">
          <div class="px-4 py-3 border-b border-theme-border">
            <span class="text-xs font-bold text-theme-text/70 uppercase tracking-wider">Search & Download</span>
          </div>
          <div class="p-4 space-y-3">
            <div class="flex gap-2">
              <input
                type="text"
                placeholder="Cari lagu / paste URL YouTube..."
                value={searchQuery()}
                onInput={(e) => setSearchQuery(e.currentTarget.value)}
                onKeyDown={(e) => e.key === 'Enter' && !downloading() && handleDownload()}
                disabled={downloading()}
                class="flex-1 px-3.5 py-2.5 bg-theme-bg border border-theme-border text-sm text-theme-text placeholder-theme-text/40 focus:outline-none focus:border-theme-cyan/60 transition-colors disabled:opacity-50"
              />
              <button
                onClick={handleDownload}
                disabled={downloading() || !searchQuery().trim()}
                class="px-5 py-2.5 bg-theme-cyan hover:bg-theme-cyan/90 disabled:opacity-40 text-theme-bg text-sm font-bold transition-all flex items-center gap-2 shrink-0 border border-theme-cyan"
              >
                <Show when={downloading()} fallback={<Download size={16} />}>
                  <Loader size={16} class="animate-spin" />
                </Show>
                <span class="hidden sm:inline">{downloading() ? 'Mendownload...' : 'Download'}</span>
              </button>
            </div>

            <Show when={downloading()}>
              <div class="flex items-center gap-2.5 px-3.5 py-2.5 bg-theme-cyan/10 border border-theme-cyan/30">
                <Loader size={15} class="animate-spin text-theme-cyan shrink-0" />
                <span class="text-xs text-theme-cyan font-medium">Mendownload audio dari YouTube, harap tunggu...</span>
              </div>
            </Show>

            <Show when={downloadResult() !== null && !downloading()}>
              <div class={`flex items-center gap-2.5 px-3.5 py-2.5 text-xs font-semibold border ${
                downloadResult()!.success
                  ? 'bg-theme-cyan/15 border-theme-cyan/35 text-[#29483A]'
                  : 'bg-[#A84B4B]/10 border-[#A84B4B]/30 text-[#A84B4B]'
              }`}>
                <Show when={downloadResult()!.success} fallback={<AlertCircle size={15} />}>
                  <Check size={15} />
                </Show>
                <span>
                  {downloadResult()!.success ? 'Download berhasil! Lagu ditambahkan ke library.' : (downloadResult()!.error || 'Gagal mendownload.')}
                </span>
              </div>
            </Show>
          </div>
        </div>
      </Show>

      {/* Now Playing Hero Card */}
      <Show when={(globalStatus().is_playing || globalStatus().is_paused) && globalStatus().current_track}>
        <div class="relative overflow-hidden border border-theme-border bg-theme-card/95 backdrop-blur-xl shadow-2xl p-4 sm:p-6">
          <div class="relative flex flex-col md:flex-row items-center gap-5 sm:gap-6">
            {/* Album Art Display */}
            <div class="relative shrink-0 w-24 h-24 sm:w-28 sm:h-28 overflow-hidden bg-theme-card border border-theme-border hover:border-theme-cyan/60 transition-all shadow-md group">
              <Show
                when={globalStatus().current_track!.thumbnail}
                fallback={
                  <div class="w-full h-full flex items-center justify-center">
                    <Music size={32} class="text-theme-cyan/70" />
                  </div>
                }
              >
                <img
                  src={globalStatus().current_track!.thumbnail}
                  alt="thumbnail"
                  class={`w-full h-full object-cover transition-transform duration-700 ${globalStatus().is_playing ? 'scale-105' : ''}`}
                />
              </Show>

              {/* Live Audio Equalizer Waveform Overlay */}
              <Show when={globalStatus().is_playing}>
                <div class="absolute inset-0 bg-black/40 backdrop-blur-[1px] flex items-end justify-center gap-0.5 p-1.5">
                  <span class="w-1 bg-theme-cyan animate-[pulse_0.6s_ease-in-out_infinite] h-4" />
                  <span class="w-1 bg-theme-rose animate-[pulse_0.9s_ease-in-out_infinite] h-6" />
                  <span class="w-1 bg-theme-cyan animate-[pulse_0.7s_ease-in-out_infinite] h-3" />
                  <span class="w-1 bg-theme-rose animate-[pulse_0.5s_ease-in-out_infinite] h-5" />
                </div>
              </Show>
            </div>

            {/* Track Info & Interactive Player Controls */}
            <div class="flex-1 min-w-0 w-full space-y-3.5 text-center md:text-left">
              <div class="flex flex-col md:flex-row md:items-center justify-between gap-2">
                <div class="min-w-0">
                  <div class="inline-flex items-center gap-2 px-2.5 py-0.5 bg-theme-card border border-theme-border text-xs font-bold uppercase tracking-wider mb-1">
                    <span class={`w-2 h-2 shrink-0 ${globalStatus().is_playing ? 'bg-theme-cyan shadow-[0_0_8px_var(--cyan)] animate-pulse' : 'bg-theme-rose'}`} />
                    <span class="text-theme-text/80">{globalStatus().is_paused ? 'Playback Dijeda' : 'Sedang Diputar'}</span>
                  </div>
                  <h3 class="text-base sm:text-lg font-bold text-theme-text truncate hover:text-theme-cyan transition-colors">
                    {globalStatus().current_track!.title}
                  </h3>
                  <p class="text-xs text-theme-text/60 truncate mt-0.5 font-medium">
                    {globalStatus().current_track!.uploader || 'Local Library'}
                  </p>
                </div>

                {/* Volume slider pill */}
                <div class="hidden sm:flex items-center justify-center md:justify-end gap-2 px-3 py-1.5 bg-theme-card border border-theme-border self-center md:self-start">
                  <button
                    onClick={() => changeVolume(volume() > 0 ? 0 : 1)}
                    class="text-theme-text/70 hover:text-theme-cyan p-0.5 transition-colors"
                    title={volume() > 0 ? 'Mute' : 'Unmute'}
                  >
                    <Show when={volume() > 0} fallback={<VolumeX size={15} />}>
                      <Volume2 size={15} />
                    </Show>
                  </button>
                  <input
                    type="range"
                    min="0"
                    max="1"
                    step="0.05"
                    value={volume()}
                    onInput={(e) => changeVolume(parseFloat(e.currentTarget.value))}
                    class="w-20 h-1.5 bg-theme-border appearance-none cursor-pointer accent-theme-cyan"
                    title={`Volume: ${Math.round(volume() * 100)}%`}
                  />
                  <span class="text-xs font-mono text-theme-text/60 w-8 text-right">
                    {Math.round(volume() * 100)}%
                  </span>
                </div>
              </div>

              {/* Progress Seekbar */}
              <div class="space-y-1">
                <div class="relative flex items-center group">
                  <input
                    type="range"
                    min="0"
                    max={duration() || globalStatus().current_track!.duration || 100}
                    value={currentTime()}
                    onInput={handleSeek}
                    class="w-full h-1.5 bg-theme-border/60 appearance-none cursor-pointer accent-theme-cyan transition-all"
                  />
                </div>
                <div class="flex justify-between text-xs font-mono text-theme-text/60">
                  <span>{formatDuration(Math.floor(currentTime()))}</span>
                  <span>{formatDuration(Math.floor(duration() || globalStatus().current_track!.duration || 0))}</span>
                </div>
              </div>

              {/* Playback Action Buttons */}
              <div class="flex items-center justify-center md:justify-start gap-2 pt-1 flex-wrap">
                {/* Prev Button */}
                <button
                  onClick={prevTrack}
                  class="p-2 bg-theme-card hover:bg-theme-border text-theme-text hover:text-theme-cyan border border-theme-border hover:border-theme-cyan/40 transition-all active:scale-95 shadow-sm"
                  title="Lagu Sebelumnya"
                  aria-label="Previous Track"
                >
                  <SkipBack size={16} />
                </button>

                {/* Play / Pause Toggle Button */}
                <Show when={globalStatus().is_playing}>
                  <button
                    onClick={pauseMusic}
                    class="h-9 px-4 bg-theme-rose/20 hover:bg-theme-rose/30 text-theme-rose border border-theme-rose/50 font-bold text-xs transition-all flex items-center gap-2 active:scale-95 shadow-sm"
                    title="Jeda"
                    aria-label="Pause"
                  >
                    <Pause size={16} />
                    <span>Jeda</span>
                  </button>
                </Show>

                <Show when={globalStatus().is_paused}>
                  <button
                    onClick={resumeMusic}
                    class="h-9 px-4 bg-theme-cyan hover:brightness-110 text-theme-bg font-bold text-xs transition-all flex items-center gap-2 active:scale-95 shadow-md shadow-theme-cyan/20 border border-theme-cyan"
                    title="Lanjutkan"
                    aria-label="Play"
                  >
                    <Play size={16} class="fill-current" />
                    <span>Lanjutkan</span>
                  </button>
                </Show>

                {/* Next Button */}
                <button
                  onClick={nextTrack}
                  class="p-2 bg-theme-card hover:bg-theme-border text-theme-text hover:text-theme-cyan border border-theme-border hover:border-theme-cyan/40 transition-all active:scale-95 shadow-sm"
                  title="Lagu Berikutnya"
                  aria-label="Next Track"
                >
                  <SkipForward size={16} />
                </button>

                {/* Stop Button */}
                <button
                  onClick={stopMusic}
                  class="p-2 bg-theme-card hover:bg-theme-rose/20 text-theme-text/70 hover:text-theme-rose border border-theme-border hover:border-theme-rose/40 transition-all active:scale-95 shadow-sm"
                  title="Hentikan Musik"
                  aria-label="Stop"
                >
                  <Square size={15} />
                </button>

                {/* PiP Button */}
                <button
                  onClick={() => pipManager.togglePip()}
                  class="px-3.5 py-1.5 bg-theme-card hover:bg-theme-border text-theme-text hover:text-theme-cyan border border-theme-border hover:border-theme-cyan/50 transition-all flex items-center gap-2 shadow-sm active:scale-95 text-xs font-semibold ml-auto"
                  title="Picture-in-Picture (Layar Melayang)"
                  aria-label="Picture-in-Picture"
                >
                  <PictureInPicture2 size={16} class="text-theme-cyan shrink-0" />
                  <span>Mode PiP</span>
                </button>
              </div>
            </div>
          </div>
        </div>
      </Show>

      {/* Library Section */}
      <div class="border border-theme-border bg-theme-surface overflow-hidden shadow-lg">
        {/* Library Header & Local Search */}
        <div class="px-4 py-3.5 border-b border-theme-border flex flex-col sm:flex-row sm:items-center justify-between gap-3 bg-theme-card/40">
          <div class="flex items-center gap-2.5">
            <div class="w-8 h-8 bg-theme-cyan/15 flex items-center justify-center border border-theme-cyan/30">
              <ListMusic size={16} class="text-theme-cyan" />
            </div>
            <div>
              <div class="flex items-center gap-2">
                <span class="text-xs font-bold text-theme-text uppercase tracking-wider">Library Playlist</span>
                <Show when={library().length > 0}>
                  <span class="px-2 py-0.5 text-xs font-bold bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/30">
                    {library().length} Lagu
                  </span>
                </Show>
              </div>
              <Show when={musicDir()}>
                <div
                  class="flex items-center gap-1.5 mt-0.5 text-xs text-theme-text/60 hover:text-theme-cyan cursor-pointer transition-colors max-w-[260px] sm:max-w-md truncate"
                  title="Klik untuk menyalin folder path musik"
                  onClick={() => {
                    navigator.clipboard.writeText(musicDir());
                    setCopiedDir(true);
                    setTimeout(() => setCopiedDir(false), 2000);
                    props.showToast('Path folder musik disalin ke clipboard!');
                  }}
                >
                  <Folder size={12} class="shrink-0 text-theme-cyan/80" />
                  <span class="truncate font-mono">{musicDir()}</span>
                  <Show when={copiedDir()} fallback={<Copy size={11} class="shrink-0 opacity-70" />}>
                    <Check size={11} class="shrink-0 text-emerald-400" />
                  </Show>
                </div>
              </Show>
            </div>
          </div>

          <div class="flex items-center gap-2">
            <div class="relative flex-1 sm:w-52">
              <Search size={14} class="absolute left-2.5 top-1/2 -translate-y-1/2 text-theme-text/40" />
              <input
                type="text"
                placeholder="Cari lagu di library..."
                value={filterQuery()}
                onInput={(e) => {
                  setFilterQuery(e.currentTarget.value);
                  setCurrentPage(1);
                }}
                class="w-full pl-8 pr-3 py-1.5 bg-theme-bg border border-theme-border text-xs text-theme-text placeholder-theme-text/40 focus:outline-none focus:border-theme-cyan/60 transition-colors"
              />
            </div>

            <button
              onClick={() => loadLibrary()}
              disabled={loadingLibrary()}
              class="px-3.5 py-1.5 bg-theme-card hover:bg-theme-border text-theme-text/80 hover:text-theme-cyan border border-theme-border transition-all flex items-center gap-1.5 disabled:opacity-40 text-xs font-semibold"
            >
              <RefreshCw size={13} class={loadingLibrary() ? 'animate-spin' : ''} />
              <span class="hidden xs:inline">Refresh</span>
            </button>
          </div>
        </div>

        {/* Tracks List */}
        <div class="divide-y divide-theme-border/60">
          <Show when={loadingLibrary()}>
            <div class="p-8 flex items-center justify-center gap-2 text-xs text-theme-text/60">
              <Loader size={16} class="animate-spin text-theme-cyan" />
              <span>Memuat daftar lagu...</span>
            </div>
          </Show>

          <Show when={!loadingLibrary() && library().length === 0}>
            <div class="p-10 text-center text-xs text-theme-text/50 space-y-3">
              <div class="w-14 h-14 bg-theme-card border border-theme-border flex items-center justify-center mx-auto">
                <Music size={26} class="text-theme-text/40" />
              </div>
              <p class="font-medium text-sm text-theme-text/70">Library lagu masih kosong</p>
              <p class="text-xs text-theme-text/50">Gunakan kolom Search & Download di atas untuk menyimpan lagu dari YouTube.</p>
            </div>
          </Show>

          <For each={paginatedTracks()}>
            {(track, idx) => {
              const globalIndex = () => (currentPage() - 1) * pageSize() + idx() + 1;
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
                      <img src={track.thumbnail} alt="" class="w-full h-full object-cover" />
                    </Show>
                  </div>

                  {/* Track Title & Artist */}
                  <div class="flex-1 min-w-0">
                    <p
                      class={`text-sm font-bold truncate transition-colors ${
                        isCurrentPlaying() ? 'text-theme-cyan' : 'text-theme-text group-hover:text-theme-cyan'
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
                      onClick={() => handlePlay(track)}
                      class={`p-2.5 transition-all border active:scale-95 ${
                        isCurrentPlaying()
                          ? 'bg-theme-cyan text-theme-bg border-theme-cyan font-bold shadow-md shadow-theme-cyan/20'
                          : 'bg-theme-card hover:bg-theme-cyan/20 text-theme-text/80 hover:text-theme-cyan border-theme-border hover:border-theme-cyan/40'
                      }`}
                      title="Putar Lagu Ini"
                    >
                      <Play size={15} class={isCurrentPlaying() ? 'fill-current' : ''} />
                    </button>

                    <button
                      onClick={() => handleDelete(track.filename, track.title)}
                      class="p-2.5 bg-theme-card hover:bg-theme-rose/20 text-theme-text/50 hover:text-theme-rose border border-theme-border hover:border-theme-rose/40 transition-all active:scale-95"
                      title="Hapus dari Library"
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
        <Show when={filteredTracks().length > 0}>
          <div class="px-4 py-3 bg-theme-card/30 border-t border-theme-border flex flex-col sm:flex-row items-center justify-between gap-3 text-xs">
            {/* Info Range */}
            <div class="text-theme-text/60 text-xs">
              Menampilkan{' '}
              <span class="font-semibold text-theme-text font-mono">
                {(currentPage() - 1) * pageSize() + 1}
              </span>
              {' - '}
              <span class="font-semibold text-theme-text font-mono">
                {Math.min(currentPage() * pageSize(), filteredTracks().length)}
              </span>
              {' dari '}
              <span class="font-semibold text-theme-cyan font-mono">
                {filteredTracks().length}
              </span>{' '}
              lagu
            </div>

            {/* Pagination Controls */}
            <div class="flex items-center gap-2">
              {/* Page Size Selector */}
              <div class="flex items-center gap-1.5 text-xs text-theme-text/60 mr-2">
                <span>Per hal:</span>
                <select
                  value={pageSize()}
                  onChange={(e) => {
                    setPageSize(parseInt(e.currentTarget.value, 10));
                    setCurrentPage(1);
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
                onClick={() => setCurrentPage((p) => Math.max(1, p - 1))}
                disabled={currentPage() <= 1}
                class="p-1.5 bg-theme-card hover:bg-theme-border border border-theme-border text-theme-text/80 hover:text-theme-cyan disabled:opacity-30 disabled:pointer-events-none transition-all"
                title="Halaman Sebelumnya"
              >
                <ChevronLeft size={15} />
              </button>

              {/* Page Indicator */}
              <div class="px-3 py-1 bg-theme-bg border border-theme-border font-mono text-xs text-theme-text font-semibold">
                <span>{currentPage()}</span>
                <span class="text-theme-text/40 mx-1">/</span>
                <span>{totalPages()}</span>
              </div>

              {/* Next Button */}
              <button
                onClick={() => setCurrentPage((p) => Math.min(totalPages(), p + 1))}
                disabled={currentPage() >= totalPages()}
                class="p-1.5 bg-theme-card hover:bg-theme-border border border-theme-border text-theme-text/80 hover:text-theme-cyan disabled:opacity-30 disabled:pointer-events-none transition-all"
                title="Halaman Berikutnya"
              >
                <ChevronRight size={15} />
              </button>
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
};
