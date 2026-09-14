import { Component, createSignal, onMount, onCleanup, createEffect, For, Show } from 'solid-js';
import { api, LogEntry } from '../api';
import { 
  Terminal, 
  Trash2, 
  Copy, 
  Search, 
  ArrowDown, 
  Filter, 
  Check, 
  RefreshCw,
  Sliders
} from 'lucide-solid';

interface LogsTabProps {
  showToast: (msg: string) => void;
}

export const LogsTab: Component<LogsTabProps> = (props) => {
  const [logs, setLogs] = createSignal<LogEntry[]>([]);
  const [searchQuery, setSearchQuery] = createSignal('');
  const [levelFilter, setLevelFilter] = createSignal<'all' | 'info' | 'error'>('all');
  const [autoScroll, setAutoScroll] = createSignal(true);
  const [copied, setCopied] = createSignal(false);
  const [isRefreshing, setIsRefreshing] = createSignal(false);

  let logsContainerRef: HTMLDivElement | undefined;
  let isFetching = false;

  const fetchLogs = async () => {
    if (isFetching) return;
    isFetching = true;
    try {
      const data = await api.getLogs();
      setLogs(data);
    } catch {
      // ignore network errors during tab unmount/background polling
    } finally {
      isFetching = false;
    }
  };

  const handleManualRefresh = async () => {
    setIsRefreshing(true);
    await fetchLogs();
    setTimeout(() => setIsRefreshing(false), 300);
  };

  const handleClearLogs = async () => {
    const ok = await api.clearLogs();
    if (ok) {
      setLogs([]);
      props.showToast('Log aktivitas berhasil dibersihkan');
    }
  };

  const handleCopyLogs = () => {
    const rawText = filteredLogs()
      .map((l) => `[${l.timestamp}] [${l.level.toUpperCase()}] ${l.message}`)
      .join('\n');
    navigator.clipboard.writeText(rawText);
    setCopied(true);
    props.showToast('Seluruh log berhasil disalin ke clipboard');
    setTimeout(() => setCopied(false), 2000);
  };

  onMount(() => {
    fetchLogs();
    const interval = setInterval(fetchLogs, 800);
    onCleanup(() => clearInterval(interval));
  });

  const filteredLogs = () => {
    const query = searchQuery().toLowerCase();
    const filter = levelFilter();

    return logs().filter((entry) => {
      const matchesLevel = filter === 'all' || entry.level === filter;
      const matchesQuery = !query || entry.message.toLowerCase().includes(query) || entry.timestamp.includes(query);
      return matchesLevel && matchesQuery;
    });
  };

  createEffect(() => {
    // Auto-scroll when new logs arrive
    filteredLogs();
    if (autoScroll() && logsContainerRef) {
      setTimeout(() => {
        if (logsContainerRef) {
          logsContainerRef.scrollTop = logsContainerRef.scrollHeight;
        }
      }, 50);
    }
  });

  const formatMessageStyle = (msg: string, level: string) => {
    if (level === 'error' || msg.includes('[ERROR]') || msg.includes('[STT Error]') || msg.includes('GAGAL')) {
      return 'text-[#FF7B7B]';
    }
    if (msg.includes('[User]')) {
      return 'text-[#7BDFF2] font-medium';
    }
    if (msg.includes('[LLM]') || msg.includes('merespons')) {
      return 'text-[#F2B5D4] font-medium';
    }
    if (msg.includes('[SKILL BERHASIL') || msg.includes('[MEMPROSES SKILL]')) {
      return 'text-[#FFE66D] font-medium';
    }
    if (msg.includes('[DILANJUTKAN]') || msg.includes('✨') || msg.includes('▶️')) {
      return 'text-[#7BDFF2]';
    }
    if (msg.includes('[DIJEDA]') || msg.includes('[SELESAI]')) {
      return 'text-[#FFD166]';
    }
    return 'text-[#EFF7F6]/90';
  };

  return (
    <div class="space-y-6">
      {/* Top Header Card */}
      <div class="p-4 sm:p-6 rounded-2xl bg-[#1D252E] border border-[#2F3B46] space-y-4 shadow-xl">
        {/* 1. Header Banner */}
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
          <div class="flex items-center gap-3">
            <div class="p-2.5 rounded-xl bg-[#7BDFF2]/20 text-[#7BDFF2] border border-[#7BDFF2]/30 shrink-0">
              <Terminal size={22} />
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h2 class="text-xl sm:text-2xl font-bangers tracking-wide text-[#EFF7F6] leading-none">
                  LIVE CONSOLE & SYSTEM LOGS
                </h2>
                <span class="text-xs px-2 py-0.5 rounded-full font-mono font-bold bg-[#7BDFF2]/20 text-[#7BDFF2] border border-[#7BDFF2]/30">
                  {filteredLogs().length} baris
                </span>
              </div>
              <p class="text-xs text-[#EFF7F6]/60 mt-0.5">
                Log percakapan AI, transkrip STT, pemanggilan skill, dan aktivitas sistem real-time
              </p>
            </div>
          </div>

          {/* Realtime Stream Indicator Badge */}
          <div class="flex items-center gap-2">
            <span class="text-xs font-mono px-3 py-1 rounded-full bg-[#161D24] text-[#7BDFF2] border border-[#2F3B46] flex items-center gap-2">
              <span class="w-2 h-2 rounded-full bg-[#7BDFF2] animate-pulse" />
              <span>LIVE LOG STREAM</span>
            </span>
          </div>
        </div>

        {/* 2. Structured Action & Filter Toolbar (Between: Kiri = Search & Filter, Kanan = Actions) */}
        <div class="p-2.5 rounded-xl bg-[#161D24] border border-[#2F3B46] flex flex-col lg:flex-row lg:items-center justify-between gap-3">
          {/* Sisi Kiri: Search Input & Level Filter Pills */}
          <div class="flex flex-col sm:flex-row sm:items-center gap-2.5 flex-1 max-w-full lg:max-w-xl">
            {/* Search Box */}
            <div class="relative flex-1">
              <Search size={14} class="absolute left-3 top-1/2 -translate-y-1/2 text-[#EFF7F6]/40" />
              <input
                type="text"
                placeholder="Cari kata kunci log..."
                value={searchQuery()}
                onInput={(e) => setSearchQuery(e.currentTarget.value)}
                class="w-full pl-8 pr-3 py-1.5 rounded-lg bg-[#232D36] border border-[#2F3B46] text-xs text-[#EFF7F6] focus:border-[#7BDFF2] focus:outline-none transition-all placeholder:text-[#EFF7F6]/30 font-mono"
              />
            </div>

            {/* Level Filter Segmented Control */}
            <div class="flex items-center bg-[#232D36] p-1 rounded-lg border border-[#2F3B46] shrink-0">
              <button
                onClick={() => setLevelFilter('all')}
                class={`px-2.5 py-1 rounded-md text-xs font-semibold transition-all ${
                  levelFilter() === 'all'
                    ? 'bg-[#7BDFF2] text-[#232D36] font-bold shadow'
                    : 'text-[#EFF7F6]/70 hover:text-[#EFF7F6]'
                }`}
              >
                Semua
              </button>
              <button
                onClick={() => setLevelFilter('info')}
                class={`px-2.5 py-1 rounded-md text-xs font-semibold transition-all ${
                  levelFilter() === 'info'
                    ? 'bg-[#7BDFF2] text-[#232D36] font-bold shadow'
                    : 'text-[#EFF7F6]/70 hover:text-[#EFF7F6]'
                }`}
              >
                Info
              </button>
              <button
                onClick={() => setLevelFilter('error')}
                class={`px-2.5 py-1 rounded-md text-xs font-semibold transition-all ${
                  levelFilter() === 'error'
                    ? 'bg-[#FF7B7B] text-[#232D36] font-bold shadow'
                    : 'text-[#FF7B7B]/80 hover:text-[#FF7B7B]'
                }`}
              >
                Error
              </button>
            </div>
          </div>

          {/* Sisi Kanan: Action Buttons Toolbar */}
          <div class="flex items-center flex-wrap gap-2 justify-end shrink-0">
            {/* Refresh Button */}
            <button
              onClick={handleManualRefresh}
              class="p-2 rounded-lg bg-[#232D36] text-[#EFF7F6]/80 hover:text-[#7BDFF2] border border-[#2F3B46] hover:border-[#7BDFF2]/50 transition-all"
              title="Refresh Log"
            >
              <RefreshCw size={14} class={isRefreshing() ? 'animate-spin text-[#7BDFF2]' : ''} />
            </button>

            {/* Auto-Scroll Toggle */}
            <button
              onClick={() => setAutoScroll(!autoScroll())}
              class={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold border transition-all ${
                autoScroll()
                  ? 'bg-[#7BDFF2]/20 text-[#7BDFF2] border-[#7BDFF2]/40 shadow-sm font-bold'
                  : 'bg-[#232D36] text-[#EFF7F6]/60 border-[#2F3B46]'
              }`}
            >
              <ArrowDown size={13} class={autoScroll() ? 'animate-bounce' : ''} />
              <span>Auto-Scroll</span>
            </button>

            {/* Copy Logs */}
            <button
              onClick={handleCopyLogs}
              class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold bg-[#232D36] text-[#EFF7F6]/80 hover:text-[#7BDFF2] border border-[#2F3B46] hover:border-[#7BDFF2]/50 transition-all"
            >
              {copied() ? <Check size={13} class="text-[#7BDFF2]" /> : <Copy size={13} />}
              <span>{copied() ? 'Tersalin' : 'Salin Semua'}</span>
            </button>

            {/* Clear Logs */}
            <button
              onClick={handleClearLogs}
              class="flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold bg-[#232D36] hover:bg-[#FF7B7B]/20 text-[#FF7B7B] border border-[#FF7B7B]/30 transition-all"
            >
              <Trash2 size={13} />
              <span>Bersihkan</span>
            </button>
          </div>
        </div>
      </div>

      {/* Terminal Display Screen */}
      <div class="rounded-2xl bg-[#161D24] border border-[#2F3B46] shadow-2xl overflow-hidden flex flex-col h-[480px] sm:h-[540px]">
        {/* Terminal Window Header Bar */}
        <div class="px-4 py-3 bg-[#1D252E] border-b border-[#2F3B46] flex items-center justify-between select-none">
          <div class="flex items-center gap-2">
            <span class="w-3 h-3 rounded-full bg-[#FF5F56] inline-block" />
            <span class="w-3 h-3 rounded-full bg-[#FFBD2E] inline-block" />
            <span class="w-3 h-3 rounded-full bg-[#27C93F] inline-block" />
            <span class="text-xs font-mono text-[#EFF7F6]/60 ml-2 font-semibold">ragasukma-stdout.log</span>
          </div>

          <div class="flex items-center gap-2">
            <span class="w-2 h-2 rounded-full bg-[#7BDFF2] shadow-[0_0_6px_#7BDFF2] animate-pulse" />
            <span class="text-[11px] font-mono text-[#7BDFF2] font-semibold">STREAM ACTIVE</span>
          </div>
        </div>

        {/* Terminal Content Log Body */}
        <div
          ref={logsContainerRef}
          class="flex-1 p-4 sm:p-5 overflow-y-auto font-mono text-xs leading-relaxed space-y-1 select-text scroll-smooth"
        >
          <Show
            when={filteredLogs().length > 0}
            fallback={
              <div class="h-full flex flex-col items-center justify-center text-[#EFF7F6]/40 py-16">
                <Terminal size={40} class="mb-3 stroke-1 text-[#EFF7F6]/20" />
                <p class="text-sm font-semibold">Belum ada log aktivitas yang cocok.</p>
                <p class="text-xs text-[#EFF7F6]/30 mt-1">Mulai berbicara dengan avatar atau jalankan skill untuk melihat log.</p>
              </div>
            }
          >
            <For each={filteredLogs()}>
              {(entry) => (
                <div class="flex items-start gap-3 py-0.5 hover:bg-[#232D36]/40 px-2 rounded -mx-2 transition-colors">
                  {/* Timestamp */}
                  <span class="text-[#EFF7F6]/40 select-none shrink-0 text-[11px]">
                    [{entry.timestamp}]
                  </span>

                  {/* Level Badge */}
                  <span
                    class={`text-[10px] uppercase font-bold px-1.5 py-0.2 rounded select-none shrink-0 ${
                      entry.level === 'error'
                        ? 'bg-[#FF7B7B]/20 text-[#FF7B7B] border border-[#FF7B7B]/30'
                        : 'bg-[#7BDFF2]/10 text-[#7BDFF2]'
                    }`}
                  >
                    {entry.level}
                  </span>

                  {/* Message Content */}
                  <span class={`flex-1 break-words whitespace-pre-wrap ${formatMessageStyle(entry.message, entry.level)}`}>
                    {entry.message}
                  </span>
                </div>
              )}
            </For>
          </Show>
        </div>
      </div>
    </div>
  );
};
