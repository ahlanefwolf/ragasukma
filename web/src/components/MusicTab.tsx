import { Component, createSignal, onMount, For, Show } from 'solid-js';
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
} from 'lucide-solid';
import { api, YtDlpInfo, YtTrackInfo } from '../api';
import {
  globalStatus,
  currentTime,
  duration,
  formatDuration,
  playTrack,
  pauseMusic,
  resumeMusic,
  stopMusic,
  seekMusic,
} from '../playerStore';

interface MusicTabProps {
  showToast: (msg: string) => void;
}

export const MusicTab: Component<MusicTabProps> = (props) => {
  const [ytDlpInfo, setYtDlpInfo] = createSignal<YtDlpInfo | null>(null);
  const [checkingYtDlp, setCheckingYtDlp] = createSignal(true);

  const [searchQuery, setSearchQuery] = createSignal('');
  const [downloading, setDownloading] = createSignal(false);
  const [downloadResult, setDownloadResult] = createSignal<{ success: boolean; error?: string } | null>(null);

  const [library, setLibrary] = createSignal<YtTrackInfo[]>([]);
  const [loadingLibrary, setLoadingLibrary] = createSignal(false);

  const [copiedCmd, setCopiedCmd] = createSignal<string | null>(null);
  const [installing, setInstalling] = createSignal(false);
  const [installMsg, setInstallMsg] = createSignal<string | null>(null);

  const checkYtDlp = async () => {
    setCheckingYtDlp(true);
    const info = await api.checkYtDlp();
    setYtDlpInfo(info);
    setCheckingYtDlp(false);
  };

  const loadLibrary = async () => {
    setLoadingLibrary(true);
    const tracks = await api.listMusic();
    setLibrary(tracks);
    setLoadingLibrary(false);
  };

  onMount(async () => {
    await checkYtDlp();
    await loadLibrary();
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

  const handlePlay = async (track: YtTrackInfo) => {
    await playTrack(track);
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
      if (res.info) setYtDlpInfo(res.info);
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
    { os: 'Linux', cmd: 'pip install yt-dlp', color: '#A8D8A8' },
  ];

  return (
    <div class="space-y-6">
      {/* Header Banner */}
      <div class="rounded-2xl bg-gradient-to-r from-theme-card via-theme-surface to-theme-bg p-4 sm:p-6 border border-theme-border flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div>
          <div class="flex items-center gap-2 flex-wrap">
            <span class="px-2.5 py-1 rounded-full text-xs font-bold uppercase tracking-wider bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/30">
              yt-dlp Powered
            </span>
            <span class="px-2 py-0.5 rounded text-[11px] font-mono bg-theme-bg text-theme-rose border border-theme-border">
              music-player
            </span>
          </div>
          <h2 class="text-2xl sm:text-3xl font-carter tracking-wide text-theme-text mt-2">
            MUSIC PLAYER
          </h2>
          <p class="text-xs sm:text-sm text-theme-text/70 mt-1 max-w-xl">
            Download dan putar musik dari YouTube langsung di Ragasukma menggunakan yt-dlp.
          </p>
        </div>
        <div class="shrink-0 w-14 h-14 rounded-2xl bg-theme-cyan/10 border border-theme-cyan/30 flex items-center justify-center">
          <Music size={28} class="text-theme-cyan" />
        </div>
      </div>

      {/* yt-dlp Status Bar */}
      <div class="rounded-xl border border-theme-border bg-theme-surface overflow-hidden">
        <div class="px-4 py-3 border-b border-theme-border flex items-center justify-between">
          <span class="text-xs font-bold text-theme-text/60 uppercase tracking-wider">Status yt-dlp</span>
          <Show when={!checkingYtDlp()}>
            <button
              onClick={checkYtDlp}
              class="text-[11px] px-2.5 py-1 rounded bg-theme-card hover:bg-theme-border text-theme-text/70 hover:text-theme-cyan border border-theme-border transition-all flex items-center gap-1.5"
            >
              <RefreshCw size={12} />
              <span>Cek Ulang</span>
            </button>
          </Show>
        </div>

        <div class="p-4">
          <Show when={checkingYtDlp()}>
            <div class="flex items-center gap-2 text-xs text-theme-text/60">
              <Loader size={14} class="animate-spin" />
              <span>Memeriksa yt-dlp...</span>
            </div>
          </Show>

          <Show when={!checkingYtDlp() && ytDlpInfo()?.available}>
            <div class="flex items-center gap-2">
              <span class="px-3 py-1.5 rounded-full text-xs font-bold bg-green-500/20 text-green-400 border border-green-500/30">
                ✓ yt-dlp v{ytDlpInfo()!.version}
              </span>
              <span class="text-xs text-theme-text/50">Siap digunakan</span>
            </div>
          </Show>

          <Show when={!checkingYtDlp() && !ytDlpInfo()?.available}>
            <div class="space-y-3">
              <div class="flex items-center gap-2 text-xs text-theme-rose">
                <AlertCircle size={14} />
                <span class="font-semibold">yt-dlp tidak ditemukan.</span>
              </div>
              {/* Tombol Auto-Install */}
              <button
                onClick={handleInstall}
                disabled={installing()}
                class="w-full flex items-center justify-center gap-2 px-4 py-3 rounded-xl bg-theme-cyan/20 hover:bg-theme-cyan/30 border border-theme-cyan/40 text-theme-cyan font-semibold text-sm transition-all disabled:opacity-50"
              >
                <Show when={installing()} fallback={<Download size={16} />}>
                  <Loader size={16} class="animate-spin" />
                </Show>
                <span>{installing() ? 'Menginstall yt-dlp...' : 'Install Otomatis'}</span>
              </button>
              <Show when={installMsg()}>
                <p class={`text-xs px-3 py-2 rounded-lg border ${
                  installMsg()!.startsWith('Gagal')
                    ? 'bg-red-500/10 border-red-500/30 text-red-400'
                    : 'bg-green-500/10 border-green-500/30 text-green-400'
                }`}>{installMsg()}</p>
              </Show>
              {/* Fallback manual */}
              <details class="group">
                <summary class="text-[11px] text-theme-text/40 hover:text-theme-text/70 cursor-pointer select-none">Install manual (klik untuk lihat perintah)</summary>
                <div class="mt-2 grid grid-cols-1 sm:grid-cols-3 gap-2">
                  <For each={installCommands}>
                    {(item) => (
                      <div class="p-3 rounded-xl bg-theme-bg border border-theme-border space-y-2">
                        <span class="text-[11px] font-bold text-theme-text/60 uppercase tracking-wide">{item.os}</span>
                        <div class="flex items-center gap-1.5 bg-theme-card rounded-lg px-2.5 py-2 border border-theme-border">
                          <code class="text-[11px] font-mono text-theme-cyan flex-1 break-all">{item.cmd}</code>
                          <button
                            onClick={() => handleCopyCmd(item.cmd)}
                            class="shrink-0 p-1 rounded hover:bg-theme-border transition-colors text-theme-text/50 hover:text-theme-cyan"
                            title="Copy"
                          >
                            <Show when={copiedCmd() === item.cmd} fallback={<Copy size={12} />}>
                              <Check size={12} class="text-green-400" />
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
        <div class="rounded-xl border border-theme-border bg-theme-surface overflow-hidden">
          <div class="px-4 py-3 border-b border-theme-border">
            <span class="text-xs font-bold text-theme-text/60 uppercase tracking-wider">Search & Download</span>
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
                class="flex-1 px-3.5 py-2.5 rounded-xl bg-theme-bg border border-theme-border text-sm text-theme-text placeholder-theme-text/30 focus:outline-none focus:border-theme-cyan/60 transition-colors disabled:opacity-50"
              />
              <button
                onClick={handleDownload}
                disabled={downloading() || !searchQuery().trim()}
                class="px-4 py-2.5 rounded-xl bg-theme-cyan hover:bg-theme-cyan/90 disabled:opacity-40 text-theme-bg text-sm font-bold transition-all flex items-center gap-2 shrink-0"
              >
                <Show when={downloading()} fallback={<Download size={16} />}>
                  <Loader size={16} class="animate-spin" />
                </Show>
                <span class="hidden sm:inline">{downloading() ? 'Mendownload...' : 'Download'}</span>
              </button>
            </div>

            <Show when={downloading()}>
              <div class="flex items-center gap-2.5 px-3.5 py-2.5 rounded-xl bg-theme-cyan/10 border border-theme-cyan/20">
                <Loader size={14} class="animate-spin text-theme-cyan shrink-0" />
                <span class="text-xs text-theme-cyan">Mendownload audio dari YouTube, harap tunggu...</span>
              </div>
            </Show>

            <Show when={downloadResult() !== null && !downloading()}>
              <div class={`flex items-center gap-2 px-3.5 py-2.5 rounded-xl text-xs font-semibold ${
                downloadResult()!.success
                  ? 'bg-green-500/10 border border-green-500/20 text-green-400'
                  : 'bg-theme-rose/10 border border-theme-rose/20 text-theme-rose'
              }`}>
                <Show when={downloadResult()!.success} fallback={<AlertCircle size={14} />}>
                  <Check size={14} />
                </Show>
                <span>
                  {downloadResult()!.success ? 'Download berhasil! Lagu ditambahkan ke library.' : (downloadResult()!.error || 'Gagal mendownload.')}
                </span>
              </div>
            </Show>
          </div>
        </div>
      </Show>

      {/* Now Playing with Interactive Seekbar */}
      <Show when={(globalStatus().is_playing || globalStatus().is_paused) && globalStatus().current_track}>
        <div class="rounded-xl border border-theme-cyan/30 bg-gradient-to-r from-theme-cyan/5 to-theme-surface overflow-hidden">
          <div class="px-4 py-3 border-b border-theme-cyan/20 flex items-center justify-between">
            <div class="flex items-center gap-2">
              <span class="w-2 h-2 rounded-full bg-theme-cyan animate-pulse" />
              <span class="text-xs font-bold text-theme-cyan uppercase tracking-wider">
                {globalStatus().is_paused ? 'Dijeda' : 'Now Playing'}
              </span>
            </div>
            <div class="flex items-center gap-2 text-xs font-mono text-theme-text/60">
              <span>{formatDuration(Math.floor(currentTime()))}</span>
              <span>/</span>
              <span>{formatDuration(Math.floor(duration() || globalStatus().current_track!.duration))}</span>
            </div>
          </div>
          <div class="p-4 space-y-3">
            <div class="flex items-center gap-4">
              <Show when={globalStatus().current_track!.thumbnail}>
                <img
                  src={globalStatus().current_track!.thumbnail}
                  alt="thumbnail"
                  class="w-14 h-14 rounded-xl object-cover shrink-0 border border-theme-border"
                />
              </Show>
              <Show when={!globalStatus().current_track!.thumbnail}>
                <div class="w-14 h-14 rounded-xl bg-theme-card border border-theme-border flex items-center justify-center shrink-0">
                  <Music size={22} class="text-theme-cyan/50" />
                </div>
              </Show>

              <div class="flex-1 min-w-0">
                <p class="text-sm font-bold text-theme-text truncate">{globalStatus().current_track!.title}</p>
                <p class="text-xs text-theme-text/60 truncate">{globalStatus().current_track!.uploader}</p>
              </div>

              <div class="flex items-center gap-2 shrink-0">
                <Show when={globalStatus().is_playing}>
                  <button
                    onClick={pauseMusic}
                    class="p-2.5 rounded-xl bg-theme-rose/20 hover:bg-theme-rose/30 text-theme-rose border border-theme-rose/30 transition-all"
                    title="Jeda"
                  >
                    <Pause size={18} />
                  </button>
                </Show>
                <Show when={globalStatus().is_paused}>
                  <button
                    onClick={resumeMusic}
                    class="p-2.5 rounded-xl bg-theme-cyan/20 hover:bg-theme-cyan/30 text-theme-cyan border border-theme-cyan/30 transition-all"
                    title="Lanjutkan"
                  >
                    <Play size={18} />
                  </button>
                </Show>
                <button
                  onClick={stopMusic}
                  class="p-2.5 rounded-xl bg-theme-card hover:bg-theme-border text-theme-text/70 hover:text-theme-rose border border-theme-border transition-all"
                  title="Stop"
                >
                  <Square size={18} />
                </button>
              </div>
            </div>

            {/* Interactive Timeline Progress Bar */}
            <div class="flex items-center gap-3 pt-1">
              <input
                type="range"
                min="0"
                max={duration() || globalStatus().current_track!.duration || 100}
                value={currentTime()}
                onInput={handleSeek}
                class="w-full h-1.5 bg-theme-card rounded-lg appearance-none cursor-pointer accent-theme-cyan"
              />
            </div>
          </div>
        </div>
      </Show>

      {/* Library */}
      <div class="rounded-xl border border-theme-border bg-theme-surface overflow-hidden">
        <div class="px-4 py-3 border-b border-theme-border flex items-center justify-between">
          <div class="flex items-center gap-2">
            <ListMusic size={14} class="text-theme-cyan" />
            <span class="text-xs font-bold text-theme-text/60 uppercase tracking-wider">Library</span>
            <Show when={library().length > 0}>
              <span class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-theme-cyan/20 text-theme-cyan">
                {library().length}
              </span>
            </Show>
          </div>
          <button
            onClick={loadLibrary}
            disabled={loadingLibrary()}
            class="text-[11px] px-2.5 py-1 rounded bg-theme-card hover:bg-theme-border text-theme-text/70 hover:text-theme-cyan border border-theme-border transition-all flex items-center gap-1.5 disabled:opacity-40"
          >
            <RefreshCw size={12} class={loadingLibrary() ? 'animate-spin' : ''} />
            <span>Refresh</span>
          </button>
        </div>

        <div class="divide-y divide-theme-border/60">
          <Show when={loadingLibrary()}>
            <div class="p-6 flex items-center justify-center gap-2 text-xs text-theme-text/40">
              <Loader size={14} class="animate-spin" />
              <span>Memuat library...</span>
            </div>
          </Show>

          <Show when={!loadingLibrary() && library().length === 0}>
            <div class="p-8 text-center text-xs text-theme-text/40 space-y-2">
              <Music size={32} class="mx-auto text-theme-text/20" />
              <p>Library kosong. Download lagu pertamamu!</p>
            </div>
          </Show>

          <For each={library()}>
            {(track) => {
              const isCurrentPlaying = () =>
                globalStatus().current_track?.filename === track.filename &&
                (globalStatus().is_playing || globalStatus().is_paused);

              return (
                <div class={`flex items-center gap-3 px-4 py-3 transition-colors ${
                  isCurrentPlaying() ? 'bg-theme-cyan/5' : 'hover:bg-theme-card/50'
                }`}>
                  <Show when={track.thumbnail}>
                    <img
                      src={track.thumbnail}
                      alt=""
                      class="w-10 h-10 rounded-lg object-cover shrink-0 border border-theme-border"
                    />
                  </Show>
                  <Show when={!track.thumbnail}>
                    <div class="w-10 h-10 rounded-lg bg-theme-card border border-theme-border flex items-center justify-center shrink-0">
                      <Music size={16} class="text-theme-cyan/40" />
                    </div>
                  </Show>

                  <div class="flex-1 min-w-0">
                    <p class={`text-sm font-semibold truncate ${isCurrentPlaying() ? 'text-theme-cyan' : 'text-theme-text'}`}>
                      {track.title}
                    </p>
                    <p class="text-xs text-theme-text/50 truncate">{track.uploader}</p>
                  </div>

                  <div class="flex items-center gap-2 shrink-0">
                    <span class="text-xs font-mono text-theme-text/40 hidden sm:inline">
                      {formatDuration(track.duration)}
                    </span>
                    <button
                      onClick={() => handlePlay(track)}
                      class={`p-2 rounded-xl transition-all border ${
                        isCurrentPlaying()
                          ? 'bg-theme-cyan/20 text-theme-cyan border-theme-cyan/30'
                          : 'bg-theme-card hover:bg-theme-cyan/20 text-theme-text/60 hover:text-theme-cyan border-theme-border hover:border-theme-cyan/30'
                      }`}
                      title="Putar"
                    >
                      <Play size={15} />
                    </button>
                    <button
                      onClick={() => handleDelete(track.filename, track.title)}
                      class="p-2 rounded-xl bg-theme-card hover:bg-red-500/20 text-theme-text/40 hover:text-red-400 border border-theme-border hover:border-red-500/30 transition-all"
                      title="Hapus Lagu"
                    >
                      <Trash2 size={15} />
                    </button>
                  </div>
                </div>
              );
            }}
          </For>
        </div>
      </div>
    </div>
  );
};
