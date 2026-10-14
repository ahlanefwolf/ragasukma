import { Component, createSignal, onMount, onCleanup, createEffect, For, Show } from 'solid-js';
import { api, LogEntry } from '../api';
import { 
  Terminal, 
  Trash2, 
  Copy, 
  Search, 
  ArrowDown, 
  Check, 
  RefreshCw 
} from 'lucide-solid';

interface LogsTabProps {
  showToast: (msg: string) => void;
}

// Module-level persistent cache log
const [cachedLogs, setCachedLogs] = createSignal<LogEntry[]>([]);

export const LogsTab: Component<LogsTabProps> = (props) => {
  const [logs, setLogs] = createSignal<LogEntry[]>(cachedLogs());
  const [searchQuery, setSearchQuery] = createSignal('');
  const [levelFilter, setLevelFilter] = createSignal<'all' | 'info' | 'error'>('all');
  const [autoScroll, setAutoScroll] = createSignal(true);
  const [copied, setCopied] = createSignal(false);
  const [isRefreshing, setIsRefreshing] = createSignal(false);

  let logsContainerRef: HTMLDivElement | undefined;
  let isFetching = false;
  let scrollRafId: number | null = null;

  const fetchLogs = async () => {
    if (isFetching || document.hidden) return;
    isFetching = true;
    try {
      const data = await api.getLogs();
      if (data && Array.isArray(data)) {
        // Keep at most the last 250 logs so browser remains super responsive without DOM overhead
        const trimmed = data.length > 250 ? data.slice(data.length - 250) : data;
        setLogs(trimmed);
        setCachedLogs(trimmed);
      }
    } catch {
      // ignore network errors during background polling
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
      props.showToast('Activity logs cleared successfully');
    }
  };

  const handleCopyLogs = () => {
    const rawText = filteredLogs()
      .map((l) => `[${l.timestamp}] [${l.level.toUpperCase()}] ${l.message}`)
      .join('\n');
    navigator.clipboard.writeText(rawText);
    setCopied(true);
    props.showToast('All logs copied to clipboard');
    setTimeout(() => setCopied(false), 2000);
  };

  onMount(() => {
    fetchLogs();

    // Subscribe to live log stream via SSE
    const unsubLogs = api.subscribeLogs((newEntry) => {
      if (!newEntry || !newEntry.id) return;
      setLogs((prev) => {
        if (prev.some((l) => l.id === newEntry.id)) return prev;
        const updated = [...prev, newEntry];
        const trimmed = updated.length > 250 ? updated.slice(updated.length - 250) : updated;
        setCachedLogs(trimmed);
        return trimmed;
      });
    });

    onCleanup(() => {
      unsubLogs();
      if (scrollRafId) cancelAnimationFrame(scrollRafId);
    });
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
    // Smooth auto-scroll when new logs arrive using requestAnimationFrame
    filteredLogs();
    if (autoScroll() && logsContainerRef) {
      if (scrollRafId) cancelAnimationFrame(scrollRafId);
      scrollRafId = requestAnimationFrame(() => {
        if (logsContainerRef) {
          logsContainerRef.scrollTop = logsContainerRef.scrollHeight;
        }
      });
    }
  });

  const sanitizeLogText = (text: string) => {
    return text.replace(/[\u{1F000}-\u{1FAFF}\u{2600}-\u{27BF}\u{2300}-\u{23FF}\u{FE00}-\u{FE0F}\u{200D}]/gu, '');
  };

  const formatMessageStyle = (msg: string, level: string) => {
    if (level === 'error' || msg.includes('[ERROR]') || msg.includes('[STT Error]') || msg.includes('FAIL')) {
      return 'text-gi-status-danger font-semibold';
    }
    if (msg.includes('[User]')) {
      return 'text-theme-cyan font-medium';
    }
    if (msg.includes('[LLM]') || msg.includes('responding')) {
      return 'text-theme-rose font-medium';
    }
    if (msg.includes('[SKILL SUCCESS') || msg.includes('[PROCESSING SKILL]')) {
      return 'text-theme-cyan font-medium';
    }
    if (msg.includes('[RESUMED]') || msg.includes('[Music Controller]')) {
      return 'text-theme-cyan';
    }
    if (msg.includes('[PAUSED]') || msg.includes('[FINISHED]')) {
      return 'text-theme-rose';
    }
    return 'text-theme-text/90';
  };

  return (
    <div class="space-y-6">
      {/* Top Header Card */}
      <div class="p-4 sm:p-6 bg-gradient-to-r from-theme-card via-theme-surface to-theme-bg border border-theme-border space-y-4 shadow-sm">
        {/* 1. Header Banner */}
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
          <div class="flex items-center gap-3">
            <div class="p-2.5 bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/30 shrink-0">
              <Terminal size={22} />
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h2 class="font-heading tracking-tight text-theme-text leading-none">
                  Live Console & System Logs
                </h2>
                <span class="text-xs px-2 py-0.5 font-mono font-bold bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/30">
                  {filteredLogs().length} lines
                </span>
              </div>
              <p class="text-xs text-theme-text/60 mt-0.5">
                AI dialogue log, STT transcriptions, skill invocations, and realtime system events
              </p>
            </div>
          </div>

          {/* Realtime Stream Indicator Badge */}
          <div class="flex items-center gap-2">
            <span class="text-xs font-mono px-3 py-1 bg-theme-bg text-theme-cyan border border-theme-border flex items-center gap-2">
              <span class="w-2 h-2 bg-theme-cyan animate-pulse" />
              <span>LIVE LOG STREAM</span>
            </span>
          </div>
        </div>

        {/* 2. Structured Action & Filter Toolbar */}
        <div class="p-2.5 bg-theme-bg border border-theme-border flex flex-col lg:flex-row lg:items-center justify-between gap-3">
          {/* Left: Search Input & Level Filter Pills */}
          <div class="flex flex-col sm:flex-row sm:items-center gap-2.5 flex-1 max-w-full lg:max-w-xl">
            {/* Search Box */}
            <div class="relative flex-1">
              <Search size={14} class="absolute left-3 top-1/2 -translate-y-1/2 text-theme-text/40" />
              <input
                type="text"
                placeholder="Search log keywords..."
                value={searchQuery()}
                onInput={(e) => setSearchQuery(e.currentTarget.value)}
                class="w-full pl-8 pr-3 py-1.5 bg-theme-card border border-theme-border text-xs text-theme-text focus:border-theme-cyan focus:outline-none transition-all placeholder:text-theme-text/30 font-mono"
              />
            </div>

            {/* Level Filter Segmented Control */}
            <div class="flex items-center bg-theme-card p-1 border border-theme-border shrink-0">
              <button
                onClick={() => setLevelFilter('all')}
                class={`px-2.5 py-1 text-xs font-semibold transition-all ${
                  levelFilter() === 'all'
                    ? 'bg-theme-cyan text-theme-card font-bold shadow'
                    : 'text-theme-text/70 hover:text-theme-text'
                }`}
              >
                All
              </button>
              <button
                onClick={() => setLevelFilter('info')}
                class={`px-2.5 py-1 text-xs font-semibold transition-all ${
                  levelFilter() === 'info'
                    ? 'bg-theme-cyan text-theme-card font-bold shadow'
                    : 'text-theme-text/70 hover:text-theme-text'
                }`}
              >
                Info
              </button>
              <button
                onClick={() => setLevelFilter('error')}
                class={`px-2.5 py-1 text-xs font-semibold transition-all ${
                  levelFilter() === 'error'
                    ? 'bg-gi-status-danger text-white font-bold shadow'
                    : 'text-gi-status-danger hover:opacity-80'
                }`}
              >
                Error
              </button>
            </div>
          </div>

          {/* Right: Action Buttons Toolbar */}
          <div class="flex items-center flex-wrap gap-2 justify-end shrink-0">
            {/* Refresh Button */}
            <button
              onClick={handleManualRefresh}
              class="p-2 bg-theme-card text-theme-text/80 hover:text-theme-cyan border border-theme-border hover:border-theme-cyan/50 transition-all"
              title="Refresh Logs"
            >
              <RefreshCw size={14} class={isRefreshing() ? 'animate-spin text-theme-cyan' : ''} />
            </button>

            {/* Auto-Scroll Toggle */}
            <button
              onClick={() => setAutoScroll(!autoScroll())}
              class={`flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold border transition-all ${
                autoScroll()
                  ? 'bg-theme-cyan/20 text-theme-cyan border-theme-cyan/40 shadow-sm font-bold'
                  : 'bg-theme-card text-theme-text/60 border-theme-border'
              }`}
            >
              <ArrowDown size={13} class={autoScroll() ? 'animate-bounce' : ''} />
              <span>Auto-Scroll</span>
            </button>

            {/* Copy Logs */}
            <button
              onClick={handleCopyLogs}
              class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold bg-theme-card text-theme-text/80 hover:text-theme-cyan border border-theme-border hover:border-theme-cyan/50 transition-all"
            >
              {copied() ? <Check size={13} class="text-theme-cyan" /> : <Copy size={13} />}
              <span>{copied() ? 'Copied' : 'Copy All'}</span>
            </button>

            {/* Clear Logs */}
            <button
              onClick={handleClearLogs}
              class="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold bg-theme-card hover:bg-gi-status-danger/20 text-gi-status-danger border border-gi-status-danger/30 transition-all"
            >
              <Trash2 size={13} />
              <span>Clear</span>
            </button>
          </div>
        </div>
      </div>

      {/* Terminal Display Screen */}
      <div class="bg-theme-bg border border-theme-border shadow-2xl overflow-hidden flex flex-col h-[480px] sm:h-[540px]">
        {/* Terminal Window Header Bar */}
        <div class="px-4 py-3 bg-theme-surface border-b border-theme-border flex items-center justify-between select-none">
          <div class="flex items-center gap-2">
            <Terminal size={14} class="text-theme-cyan" />
            <span class="text-xs font-mono text-theme-text/80 font-semibold">ragasukma-stdout.log</span>
          </div>

          <div class="flex items-center gap-2">
            <span class="w-2 h-2 bg-theme-cyan shadow-[0_0_6px_var(--cyan)] animate-pulse" />
            <span class="text-xs font-mono text-theme-cyan font-semibold">STREAM ACTIVE</span>
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
              <div class="h-full flex flex-col items-center justify-center text-theme-text/40 py-16">
                <Terminal size={40} class="mb-3 stroke-1 text-theme-text/20" />
                <p class="text-sm font-semibold">No matching activity logs found.</p>
                <p class="text-xs text-theme-text/30 mt-1">Start speaking to the avatar or trigger a skill to inspect logs.</p>
              </div>
            }
          >
            <For each={filteredLogs()}>
              {(entry) => (
                <div class="flex items-start gap-3 py-0.5 hover:bg-theme-card/40 px-2 -mx-2 transition-colors">
                  {/* Timestamp */}
                  <span class="text-theme-text/40 select-none shrink-0 text-xs">
                    [{entry.timestamp}]
                  </span>

                  {/* Level Badge */}
                  <span
                    class={`text-xs uppercase font-bold px-1.5 py-0.5 select-none shrink-0 ${
                      entry.level === 'error'
                        ? 'bg-gi-status-danger/20 text-gi-status-danger border border-gi-status-danger/40'
                        : 'bg-theme-cyan/10 text-theme-cyan'
                    }`}
                  >
                    {entry.level}
                  </span>

                  {/* Message Content */}
                  <span class={`flex-1 break-words whitespace-pre-wrap ${formatMessageStyle(entry.message, entry.level)}`}>
                    {sanitizeLogText(entry.message)}
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
