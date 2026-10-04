import { Component, createSignal, onMount, For, Show } from 'solid-js';
import {
  MessageSquare,
  Calendar,
  Clock,
  Trash2,
  RefreshCw,
  User,
  Bot,
  ChevronRight,
  Download
} from 'lucide-solid';
import { api, ChatHistoryDay, ChatTurnItem } from '../api';

interface HistoryTabProps {
  showToast: (msg: string) => void;
}

export const HistoryTab: Component<HistoryTabProps> = (props) => {
  const [dates, setDates] = createSignal<string[]>([]);
  const [selectedDate, setSelectedDate] = createSignal<string>('');
  const [currentHistory, setCurrentHistory] = createSignal<ChatHistoryDay | null>(null);
  const [loadingDates, setLoadingDates] = createSignal(true);
  const [loadingTurns, setLoadingTurns] = createSignal(false);

  const loadDates = async (selectFirst = true) => {
    setLoadingDates(true);
    try {
      const data = await api.getHistoryDates();
      const list = Array.isArray(data) ? data : [];
      setDates(list);
      if (selectFirst && list.length > 0) {
        loadHistoryDay(list[0]);
      } else if (list.length === 0) {
        setSelectedDate('');
        setCurrentHistory(null);
      }
    } catch {
      setDates([]);
    } finally {
      setLoadingDates(false);
    }
  };

  const loadHistoryDay = async (date: string) => {
    setSelectedDate(date);
    setLoadingTurns(true);
    try {
      const data = await api.getHistoryByDate(date);
      setCurrentHistory(data);
    } catch {
      setCurrentHistory({ date, turns: [] });
    } finally {
      setLoadingTurns(false);
    }
  };

  onMount(() => {
    loadDates(true);
  });

  const handleDeleteDate = async (date: string) => {
    if (confirm(`Delete conversation history for ${date}?`)) {
      const res = await api.deleteHistory(date);
      if (res.success) {
        props.showToast(`History for ${date} deleted`);
        const remaining = dates().filter(d => d !== date);
        setDates(remaining);
        if (selectedDate() === date) {
          if (remaining.length > 0) {
            loadHistoryDay(remaining[0]);
          } else {
            setSelectedDate('');
            setCurrentHistory(null);
          }
        }
      } else {
        props.showToast(`Failed to delete history: ${res.error || 'Error'}`);
      }
    }
  };

  const handleExportMarkdown = () => {
    const history = currentHistory();
    if (!history || history.turns.length === 0) {
      props.showToast('No messages to export');
      return;
    }
    const lines = [
      `# Conversation History — ${history.date}`,
      `Generated from Ragasukma AI Voice Assistant`,
      '',
      '---',
      ''
    ];
    for (const t of history.turns) {
      lines.push(`### [${t.time}] ${t.speaker}`);
      lines.push(t.message);
      lines.push('');
      lines.push('---');
      lines.push('');
    }
    const blob = new Blob([lines.join('\n')], { type: 'text/markdown;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `chat-history-${history.date}.md`;
    a.click();
    URL.revokeObjectURL(url);
    props.showToast(`Exported history for ${history.date}`);
  };

  return (
    <div class="space-y-5 animate-fade-in text-theme-text select-none">
      {/* Header Card */}
      <div class="p-5 bg-theme-surface border border-theme-border shadow-sm flex flex-col sm:flex-row sm:items-center justify-between gap-4">
        <div class="flex items-center gap-3">
          <div class="p-3 bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/30">
            <MessageSquare size={22} />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="text-xl sm:text-2xl font-heading tracking-wide">
                Chat History
              </h2>
              <span class="text-xs font-mono px-2 py-0.5 bg-theme-card border border-theme-border text-theme-cyan font-bold">
                {dates().length} Days Recorded
              </span>
            </div>
            <p class="text-xs text-theme-text/70 mt-1">
              Review dialogue sessions, spoken instructions, and assistant replies
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2 self-end sm:self-center">
          <button
            onClick={() => loadDates(false)}
            disabled={loadingDates()}
            class="px-3.5 py-2 text-xs font-semibold bg-theme-bg hover:bg-theme-card border border-theme-border flex items-center gap-2 transition-all cursor-pointer"
            title="Refresh dates"
          >
            <RefreshCw size={14} class={loadingDates() ? 'animate-spin text-theme-cyan' : ''} />
            <span class="hidden sm:inline">Refresh</span>
          </button>

          <button
            onClick={handleExportMarkdown}
            disabled={!currentHistory() || (currentHistory()?.turns.length || 0) === 0}
            class="px-4 py-2 text-xs font-bold bg-theme-card hover:bg-theme-bg border border-theme-border text-theme-cyan flex items-center gap-2 transition-all cursor-pointer disabled:opacity-40"
            title="Export markdown file"
          >
            <Download size={14} />
            <span>Export Markdown</span>
          </button>
        </div>
      </div>

      {/* Main Two-Column Layout (Date Sidebar + Chat Log) */}
      <div class="grid grid-cols-1 md:grid-cols-4 gap-4">
        {/* Left Column: Dates List */}
        <div class="md:col-span-1 bg-theme-surface border border-theme-border p-3 space-y-2 h-fit">
          <div class="flex items-center justify-between pb-2 border-b border-theme-border">
            <span class="text-xs font-mono uppercase tracking-wider text-theme-text/70 font-bold flex items-center gap-1.5">
              <Calendar size={13} class="text-theme-cyan" />
              <span>Log Dates</span>
            </span>
            <span class="text-[11px] font-mono text-theme-text/50">
              {dates().length} files
            </span>
          </div>

          <Show when={!loadingDates()} fallback={
            <div class="p-6 text-center text-xs text-theme-text/50 font-mono">
              Loading dates...
            </div>
          }>
            <Show when={dates().length > 0} fallback={
              <div class="p-6 text-center text-xs text-theme-text/50 font-mono">
                No conversation history found.
              </div>
            }>
              <div class="space-y-1 max-h-[60vh] overflow-y-auto custom-scrollbar">
                <For each={dates()}>
                  {(date) => {
                    const isSelected = () => selectedDate() === date;
                    return (
                      <div
                        onClick={() => loadHistoryDay(date)}
                        class={`w-full text-left p-2.5 flex items-center justify-between border transition-all cursor-pointer ${
                          isSelected()
                            ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold shadow-sm'
                            : 'bg-theme-bg hover:bg-theme-card border-theme-border text-theme-text/80'
                        }`}
                      >
                        <div class="flex items-center gap-2 min-w-0">
                          <Clock size={13} class={isSelected() ? 'text-theme-card' : 'text-theme-cyan'} />
                          <span class="text-xs font-mono truncate">{date}</span>
                        </div>
                        <button
                          type="button"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleDeleteDate(date);
                          }}
                          class={`p-1 transition-colors ${
                            isSelected()
                              ? 'text-theme-card/70 hover:text-theme-card'
                              : 'text-theme-text/40 hover:text-theme-rose'
                          }`}
                          title={`Delete history for ${date}`}
                        >
                          <Trash2 size={13} />
                        </button>
                      </div>
                    );
                  }}
                </For>
              </div>
            </Show>
          </Show>
        </div>

        {/* Right Column: Chat Turns Stream */}
        <div class="md:col-span-3 bg-theme-surface border border-theme-border p-4 space-y-4 min-h-[400px]">
          <div class="flex items-center justify-between pb-3 border-b border-theme-border">
            <div class="flex items-center gap-2">
              <h3 class="text-base font-bold text-theme-text font-heading">
                {selectedDate() ? `Conversation on ${selectedDate()}` : 'Select a date'}
              </h3>
              <Show when={currentHistory()}>
                <span class="text-xs font-mono px-2 py-0.5 bg-theme-card border border-theme-border text-theme-cyan">
                  {currentHistory()!.turns.length} Turns
                </span>
              </Show>
            </div>
          </div>

          <Show when={!loadingTurns()} fallback={
            <div class="p-12 text-center text-xs font-mono text-theme-text/50">
              Loading chat turns...
            </div>
          }>
            <Show when={currentHistory() && currentHistory()!.turns.length > 0} fallback={
              <div class="p-12 text-center text-xs font-mono text-theme-text/50 space-y-2">
                <MessageSquare size={32} class="mx-auto text-theme-text/30" />
                <p>No messages recorded for this date.</p>
              </div>
            }>
              <div class="space-y-3.5 max-h-[70vh] overflow-y-auto pr-1 custom-scrollbar">
                <For each={currentHistory()!.turns}>
                  {(turn) => {
                    const isAssistant = () => turn.speaker.toLowerCase().includes('sukma') ||
                      turn.speaker.toLowerCase().includes('assistant') ||
                      turn.speaker.toLowerCase().includes('ai');

                    return (
                      <div class={`p-3.5 border transition-all ${
                        isAssistant()
                          ? 'bg-theme-bg border-theme-cyan/30 ml-2 sm:ml-6'
                          : 'bg-theme-card border-theme-border mr-2 sm:mr-6'
                      }`}>
                        <div class="flex items-center justify-between gap-2 mb-1.5 pb-1 border-b border-theme-border/40">
                          <div class="flex items-center gap-2">
                            {isAssistant() ? (
                              <span class="p-1 bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/30">
                                <Bot size={13} />
                              </span>
                            ) : (
                              <span class="p-1 bg-theme-border text-theme-text/80 border border-theme-border">
                                <User size={13} />
                              </span>
                            )}
                            <span class={`text-xs font-bold font-mono ${isAssistant() ? 'text-theme-cyan' : 'text-theme-text'}`}>
                              {turn.speaker}
                            </span>
                          </div>
                          <span class="text-[10.5px] font-mono text-theme-text/50 flex items-center gap-1.5">
                            <Clock size={11} />
                            <span>{turn.time.split('+')[0].split('-')[0]}</span>
                            {turn.time.includes('+') && (
                              <span class="px-1.5 py-0.2 bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/30 text-[9.5px] font-bold">
                                UTC+{turn.time.split('+')[1]}
                              </span>
                            )}
                            {turn.time.includes('-') && turn.time.split('-').length > 1 && !turn.time.startsWith('-') && (
                              <span class="px-1.5 py-0.2 bg-theme-card text-theme-text/70 border border-theme-border text-[9.5px] font-bold">
                                UTC-{turn.time.split('-')[1]}
                              </span>
                            )}
                          </span>
                        </div>
                        <p class="text-xs leading-relaxed text-theme-text/90 font-sans whitespace-pre-wrap select-text">
                          {turn.message}
                        </p>
                      </div>
                    );
                  }}
                </For>
              </div>
            </Show>
          </Show>
        </div>
      </div>
    </div>
  );
};
