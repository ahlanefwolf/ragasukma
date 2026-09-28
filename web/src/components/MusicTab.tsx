import { Component, createSignal, onMount, onCleanup } from 'solid-js';
import { Music } from 'lucide-solid';
import { api, YtDlpInfo, YtTrackInfo } from '../api';
import { playTrack } from '../playerStore';
import { YtDlpStatusCard } from './music/YtDlpStatusCard';
import { MusicDownloadCard } from './music/MusicDownloadCard';
import { NowPlayingHeroCard } from './music/NowPlayingHeroCard';
import { MusicTrackTable } from './music/MusicTrackTable';

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
  const [downloadResult, setDownloadResult] = createSignal<{
    success: boolean;
    error?: string;
  } | null>(null);

  const [loadingLibrary, setLoadingLibrary] = createSignal(cachedLibrary().length === 0);
  const [filterQuery, setFilterQuery] = createSignal('');
  const [currentPage, setCurrentPage] = createSignal(1);
  const [pageSize, setPageSize] = createSignal(10);

  const [copiedCmd, setCopiedCmd] = createSignal<string | null>(null);
  const [installing, setInstalling] = createSignal(false);
  const [installMsg, setInstallMsg] = createSignal<string | null>(null);

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

  onMount(() => {
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

      {/* yt-dlp Status Card */}
      <YtDlpStatusCard
        checking={checkingYtDlp}
        info={ytDlpInfo}
        onRefresh={() => checkYtDlp()}
        onInstall={handleInstall}
        installing={installing}
        installMsg={installMsg}
        copiedCmd={copiedCmd}
        onCopyCmd={handleCopyCmd}
      />

      {/* Search & Download Card */}
      <MusicDownloadCard
        available={!!ytDlpInfo()?.available}
        searchQuery={searchQuery}
        onSearchInput={setSearchQuery}
        downloading={downloading}
        downloadResult={downloadResult}
        onDownload={handleDownload}
      />

      {/* Now Playing Hero Card */}
      <NowPlayingHeroCard />

      {/* Track Library Table & Pagination */}
      <MusicTrackTable
        library={library}
        musicDir={musicDir}
        loading={loadingLibrary}
        filterQuery={filterQuery}
        onFilterChange={setFilterQuery}
        currentPage={currentPage}
        onPageChange={setCurrentPage}
        pageSize={pageSize}
        onPageSizeChange={setPageSize}
        filteredTracks={filteredTracks}
        paginatedTracks={paginatedTracks}
        totalPages={totalPages}
        onRefresh={() => loadLibrary()}
        onPlay={handlePlay}
        onDelete={handleDelete}
        showToast={props.showToast}
      />
    </div>
  );
};
