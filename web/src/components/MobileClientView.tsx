import { Component, createSignal, onMount, onCleanup, createEffect, For, Show } from 'solid-js';
import { 
  Terminal, 
  RefreshCw, 
  Send,
  Smartphone,
  Trash2,
  CheckCircle2
} from 'lucide-solid';
import { api, LogEntry } from '../api';

interface MobileClientViewProps {
  authKey: string;
  showToast: (msg: string) => void;
}

export const MobileClientView: Component<MobileClientViewProps> = (props) => {
  const [inputText, setInputText] = createSignal('');
  const [isSending, setIsSending] = createSignal(false);
  const [lastSentText, setLastSentText] = createSignal('');

  // Live Console Logs state
  const [logs, setLogs] = createSignal<LogEntry[]>([]);
  const [isRefreshingLogs, setIsRefreshingLogs] = createSignal(false);

  let logsContainerRef: HTMLDivElement | undefined;
  let isFetchingLogs = false;

  const fetchLogs = async () => {
    if (isFetchingLogs || document.hidden) return;
    isFetchingLogs = true;
    try {
      const data = await api.getLogs();
      if (data && Array.isArray(data)) {
        const trimmed = data.length > 60 ? data.slice(data.length - 60) : data;
        setLogs(trimmed);
      }
    } catch {
      // ignore
    } finally {
      isFetchingLogs = false;
    }
  };

  onMount(() => {
    fetchLogs();
    const interval = setInterval(fetchLogs, 2500);

    onCleanup(() => {
      clearInterval(interval);
    });
  });

  createEffect(() => {
    logs();
    if (logsContainerRef) {
      logsContainerRef.scrollTop = logsContainerRef.scrollHeight;
    }
  });

  const sendTextToHost = async (e?: Event) => {
    if (e) e.preventDefault();
    const textToSend = inputText().trim();
    if (!textToSend || isSending()) return;

    setIsSending(true);
    try {
      const res = await api.sendRemoteText(textToSend, props.authKey);
      if (res.success) {
        setLastSentText(textToSend);
        setInputText('');
        props.showToast(`✅ Terkirim: "${textToSend}"`);
        setTimeout(fetchLogs, 400);
      } else {
        props.showToast(`⚠️ ${res.error || 'Gagal mengirim pesan'}`);
      }
    } catch (err: any) {
      props.showToast(`Error: ${err.message || err}`);
    } finally {
      setIsSending(false);
    }
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendTextToHost();
    }
  };

  const formatMessageStyle = (msg: string, level: string) => {
    if (level === 'error' || msg.includes('[ERROR]') || msg.includes('GAGAL')) {
      return 'text-red-400 font-semibold';
    }
    if (msg.includes('[User]')) {
      return 'text-theme-cyan font-bold';
    }
    if (msg.includes('[LLM]') || msg.includes('merespons')) {
      return 'text-theme-rose font-medium';
    }
    if (msg.includes('✨') || msg.includes('▶️')) {
      return 'text-theme-cyan';
    }
    return 'text-theme-text/80';
  };

  return (
    <div class="flex flex-col h-full w-full bg-theme-bg select-none">
      {/* 1. Header Minimalis HP */}
      <header class="h-12 px-4 bg-theme-surface border-b border-theme-border flex items-center justify-between shrink-0 shadow-sm">
        <div class="flex items-center gap-2">
          <span class="w-2.5 h-2.5 rounded-full bg-theme-cyan shadow-[0_0_8px_var(--cyan)] animate-pulse" />
          <h1 class="text-sm font-heading font-bold text-theme-text tracking-wide">
            Ragasukma Remote
          </h1>
        </div>
        <div class="flex items-center gap-2">
          <span class="text-[10px] font-mono px-2 py-0.5 bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/30 rounded font-semibold uppercase flex items-center gap-1">
            <Smartphone size={11} />
            <span>Connected</span>
          </span>
        </div>
      </header>

      {/* 2. Top Half: Clean Text Input & Send */}
      <div class="p-4 bg-gradient-to-b from-theme-surface to-theme-bg border-b border-theme-border flex flex-col shrink-0 space-y-3">
        {/* Status Bubble / Last Sent Feedback */}
        <Show when={lastSentText()}>
          <div class="w-full px-3 py-2 bg-theme-card border border-theme-border rounded-lg flex items-center gap-2 shadow-inner">
            <CheckCircle2 size={14} class="text-theme-cyan shrink-0" />
            <div class="min-w-0 flex-1">
              <span class="text-[10px] text-theme-text/50 font-mono block">Pesan Terakhir:</span>
              <p class="text-xs font-semibold text-theme-cyan truncate">
                "{lastSentText()}"
              </p>
            </div>
          </div>
        </Show>

        {/* Text Input Area */}
        <div class="relative w-full">
          <textarea
            value={inputText()}
            onInput={(e) => setInputText(e.currentTarget.value)}
            onKeyDown={handleKeyDown}
            placeholder="Ketik perintah atau gunakan Voice Typing keyboard HP (ikon mic di keyboard)..."
            rows={3}
            disabled={isSending()}
            class="w-full p-3 bg-theme-card border border-theme-border rounded-xl text-sm text-theme-text placeholder:text-theme-text/40 focus:outline-none focus:border-theme-cyan transition-colors resize-none shadow-inner"
          />

          <Show when={inputText().length > 0}>
            <button
              type="button"
              onClick={() => setInputText('')}
              class="absolute top-2 right-2 p-1.5 text-theme-text/40 hover:text-theme-text transition-colors rounded-md"
              title="Hapus teks"
            >
              <Trash2 size={14} />
            </button>
          </Show>
        </div>

        {/* Actions Row */}
        <div class="flex items-center justify-between gap-3">
          <span class="text-[10px] text-theme-text/50 font-mono">
            💡 Tekan Enter / tombol Kirim
          </span>
          <button
            type="button"
            onClick={() => sendTextToHost()}
            disabled={!inputText().trim() || isSending()}
            class="px-5 py-2.5 bg-theme-cyan text-theme-card rounded-lg text-xs font-bold flex items-center gap-2 disabled:opacity-40 disabled:cursor-not-allowed hover:opacity-90 active:scale-95 transition-all shadow-[0_0_12px_var(--cyan)]"
          >
            <Send size={14} class={isSending() ? 'animate-pulse' : ''} />
            <span>{isSending() ? 'Mengirim...' : 'Kirim Perintah'}</span>
          </button>
        </div>
      </div>

      {/* 3. Bottom Half: Live Logs Console */}
      <div class="flex-1 flex flex-col min-h-0 bg-theme-surface">
        <div class="h-8 px-3 bg-theme-surface border-b border-theme-border flex items-center justify-between shrink-0">
          <div class="flex items-center gap-2 text-xs font-mono font-bold text-theme-text/80">
            <Terminal size={13} class="text-theme-cyan" />
            <span>CONSOLE LOGS</span>
            <span class="text-[10px] text-theme-text/50">({logs().length})</span>
          </div>

          <button
            onClick={() => {
              setIsRefreshingLogs(true);
              fetchLogs();
              setTimeout(() => setIsRefreshingLogs(false), 300);
            }}
            class="p-1 text-theme-text/60 hover:text-theme-text transition-colors"
            title="Refresh Logs"
          >
            <RefreshCw size={12} class={isRefreshingLogs() ? 'animate-spin' : ''} />
          </button>
        </div>

        {/* Scrollable Logs Window */}
        <div
          ref={logsContainerRef}
          class="flex-1 overflow-y-auto p-3 font-mono text-[11px] leading-relaxed space-y-1.5 bg-theme-card/50 text-theme-text select-text"
        >
          <Show
            when={logs().length > 0}
            fallback={
              <div class="h-full flex items-center justify-center text-xs text-theme-text/40">
                Belum ada aktivitas suara / log...
              </div>
            }
          >
            <For each={logs()}>
              {(log) => (
                <div class="break-words border-b border-theme-border/30 pb-1">
                  <span class="text-[10px] text-theme-text/50 mr-1.5 font-mono">
                    {log.timestamp ? log.timestamp.split(' ')[1] || log.timestamp : ''}
                  </span>
                  <span class={formatMessageStyle(log.message, log.level)}>
                    {log.message}
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
