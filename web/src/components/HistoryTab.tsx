import { Component, createSignal, onMount, For, Show, createMemo } from 'solid-js';
import {
  MessageSquare,
  Calendar,
  Clock,
  Trash2,
  RefreshCw,
  User,
  Bot,
  ChevronRight,
  ChevronLeft,
  ChevronsRight,
  ChevronsLeft,
  Download,
  FolderGit2,
  X,
  Loader2,
  Search,
  Copy,
  Check,
  ArrowUpDown,
  Filter
} from 'lucide-solid';
import { api, ChatHistoryDay, ChatTurnItem } from '../api';
import { agentService, Project } from '../services/agentService';
import { showConfirm } from './ConfirmModal';

interface HistoryTabProps {
  showToast: (msg: string) => void;
}

export const HistoryTab: Component<HistoryTabProps> = (props) => {
  const [dates, setDates] = createSignal<string[]>([]);
  const [selectedDate, setSelectedDate] = createSignal<string>('');
  const [currentHistory, setCurrentHistory] = createSignal<ChatHistoryDay | null>(null);
  const [loadingDates, setLoadingDates] = createSignal(true);
  const [loadingTurns, setLoadingTurns] = createSignal(false);

  // Pagination & Sorting Signals
  const [page, setPage] = createSignal(1);
  const [limit, setLimit] = createSignal(25);
  const [order, setOrder] = createSignal<'desc' | 'asc'>('desc');

  // Search & Filter Signals
  const [dateSearch, setDateSearch] = createSignal('');
  const [messageSearch, setMessageSearch] = createSignal('');
  const [copiedTurnKey, setCopiedTurnKey] = createSignal('');

  // Bridge to Agent Workspace modal state
  const [showProjectModal, setShowProjectModal] = createSignal(false);
  const [availableProjects, setAvailableProjects] = createSignal<Project[]>([]);
  const [selectedBridgeProjectId, setSelectedBridgeProjectId] = createSignal<string>('');
  const [bridgeChatId, setBridgeChatId] = createSignal<string>('');
  const [bridgeTitle, setBridgeTitle] = createSignal<string>('');
  const [creatingTask, setCreatingTask] = createSignal(false);

  const handleSendToProject = async (chatId: string, messagePreview: string) => {
    try {
      const pList = await agentService.getProjects();
      if (pList.length === 0) {
        props.showToast('No workspace projects found. Create a project first in the Agent Workspace tab.');
        return;
      }
      setAvailableProjects(pList);
      setSelectedBridgeProjectId(pList[0].id);
      setBridgeChatId(chatId);

      const firstLine = messagePreview.split('\n')[0].trim();
      const title = firstLine.length > 50 ? firstLine.slice(0, 50) + '...' : firstLine;
      setBridgeTitle(title || 'Task from Main Chat');
      setShowProjectModal(true);
    } catch {
      props.showToast('Failed to load project list');
    }
  };

  const handleConfirmCreateTask = async () => {
    const pId = selectedBridgeProjectId();
    const cId = bridgeChatId();
    if (!pId || !cId || creatingTask()) return;

    setCreatingTask(true);
    try {
      const res = await agentService.createSessionFromChat({
        project_id: pId,
        chat_id: cId,
        title: bridgeTitle().trim(),
      });

      if (res.ok && res.session) {
        props.showToast(`Task "${res.session.title}" created successfully in Workspace! Redirecting...`);
        setShowProjectModal(false);
        window.location.hash = '#agent';
      } else {
        props.showToast(res.error || 'Failed to create task session');
      }
    } catch (e: any) {
      props.showToast(e.message || 'Failed to create task');
    } finally {
      setCreatingTask(false);
    }
  };

  const loadDates = async (selectFirst = true) => {
    setLoadingDates(true);
    try {
      const data = await api.getHistoryDates();
      const list = Array.isArray(data) ? data : [];
      setDates(list);
      if (selectFirst && list.length > 0) {
        setPage(1);
        loadHistoryDay(list[0], 1);
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

  const loadHistoryDay = async (date: string, targetPage = page()) => {
    setSelectedDate(date);
    setLoadingTurns(true);
    try {
      const data = await api.getHistoryByDate(date, targetPage, limit(), order());
      setCurrentHistory(data);
      if (data.page) setPage(data.page);
    } catch {
      setCurrentHistory({ date, turns: [] });
    } finally {
      setLoadingTurns(false);
    }
  };

  const handlePageChange = (newPage: number) => {
    const totalPages = currentHistory()?.total_pages || 1;
    const clamped = Math.max(1, Math.min(newPage, totalPages));
    if (clamped !== page() && !loadingTurns() && selectedDate()) {
      setPage(clamped);
      loadHistoryDay(selectedDate(), clamped);
    }
  };

  const toggleSortOrder = () => {
    const nextOrder = order() === 'desc' ? 'asc' : 'desc';
    setOrder(nextOrder);
    setPage(1);
    if (selectedDate()) {
      loadHistoryDay(selectedDate(), 1);
    }
  };

  const handleLimitChange = (newLimit: number) => {
    setLimit(newLimit);
    setPage(1);
    if (selectedDate()) {
      loadHistoryDay(selectedDate(), 1);
    }
  };

  onMount(() => {
    loadDates(true);
  });

  const handleDeleteDate = async (date: string) => {
    const ok = await showConfirm({
      title: 'Delete History Record',
      message: `Are you sure you want to permanently delete all conversation history for ${date}?`,
      confirmText: 'Delete',
      isDanger: true,
    });
    if (!ok) return;

    const res = await api.deleteHistory(date);
    if (res.success) {
      props.showToast(`History for ${date} deleted`);
      const remaining = dates().filter(d => d !== date);
      setDates(remaining);
      if (selectedDate() === date) {
        if (remaining.length > 0) {
          setPage(1);
          loadHistoryDay(remaining[0], 1);
        } else {
          setSelectedDate('');
          setCurrentHistory(null);
        }
      }
    } else {
      props.showToast(`Failed to delete history: ${res.error || 'Error'}`);
    }
  };

  const handleCopyMessage = async (key: string, text: string) => {
    try {
      await navigator.clipboard.writeText(text);
      setCopiedTurnKey(key);
      props.showToast('Copied to clipboard');
      setTimeout(() => {
        setCopiedTurnKey('');
      }, 2000);
    } catch {
      props.showToast('Failed to copy text');
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
      `Order: ${order() === 'desc' ? 'Newest First' : 'Oldest First'} | Page: ${page()} of ${history.total_pages || 1}`,
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
    a.download = `chat-history-${history.date}-p${page()}.md`;
    a.click();
    URL.revokeObjectURL(url);
    props.showToast(`Exported history for ${history.date}`);
  };

  const formatDateLabel = (dateStr: string) => {
    const today = new Date();
    const yyyy = today.getFullYear();
    const mm = String(today.getMonth() + 1).padStart(2, '0');
    const dd = String(today.getDate()).padStart(2, '0');
    const todayStr = `${yyyy}-${mm}-${dd}`;

    const yesterday = new Date(today);
    yesterday.setDate(yesterday.getDate() - 1);
    const y_yyyy = yesterday.getFullYear();
    const y_mm = String(yesterday.getMonth() + 1).padStart(2, '0');
    const y_dd = String(yesterday.getDate()).padStart(2, '0');
    const yesterdayStr = `${y_yyyy}-${y_mm}-${y_dd}`;

    if (dateStr === todayStr) return 'Today';
    if (dateStr === yesterdayStr) return 'Yesterday';
    return null;
  };

  // Filtered dates based on search
  const filteredDates = createMemo(() => {
    const q = dateSearch().trim().toLowerCase();
    if (!q) return dates();
    return dates().filter(d => d.toLowerCase().includes(q));
  });

  // Filtered turns based on search in current day
  const filteredTurns = createMemo(() => {
    const turns = currentHistory()?.turns || [];
    const q = messageSearch().trim().toLowerCase();
    if (!q) return turns;
    return turns.filter(t => 
      t.message.toLowerCase().includes(q) || 
      t.speaker.toLowerCase().includes(q)
    );
  });

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
              <h1 class="text-xl font-heading font-bold text-theme-text tracking-wide uppercase">
                Conversation History
              </h1>
              <span class="text-2xs font-mono px-2 py-0.5 bg-theme-cyan/10 text-theme-cyan border border-theme-cyan/30 uppercase font-semibold">
                SQLite Storage
              </span>
            </div>
            <p class="text-xs text-theme-text/60 font-mono mt-0.5">
              Review, filter, inspect and export interaction logs across all sessions
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2 flex-wrap">
          <button
            onClick={() => loadDates(false)}
            disabled={loadingDates()}
            class="px-3 py-1.5 text-xs font-semibold bg-theme-card border border-theme-border hover:border-theme-cyan text-theme-text transition-all flex items-center gap-1.5"
            title="Refresh dates list"
          >
            <RefreshCw size={13} class={loadingDates() ? 'animate-spin' : ''} />
            <span>Refresh</span>
          </button>

          <Show when={selectedDate() && currentHistory() && currentHistory()!.turns.length > 0}>
            <button
              onClick={handleExportMarkdown}
              class="px-3 py-1.5 text-xs bg-theme-cyan text-theme-card border border-theme-cyan shadow-sm hover:brightness-110 transition-all flex items-center gap-1.5 font-bold"
              title="Download chat history as Markdown file"
            >
              <Download size={13} />
              <span>Export Page MD</span>
            </button>
          </Show>
        </div>
      </div>

      {/* Main Grid: Date Selector + Turn Viewer */}
      <div class="grid grid-cols-1 md:grid-cols-4 gap-5">
        {/* Left Column: Dates list */}
        <div class="md:col-span-1 bg-theme-surface border border-theme-border p-3 space-y-2.5 flex flex-col h-full min-h-[500px]">
          <div class="flex items-center justify-between px-1 py-1 text-xs font-mono text-theme-text/70 border-b border-theme-border pb-2">
            <div class="flex items-center gap-1.5">
              <Calendar size={14} class="text-theme-cyan" />
              <span class="font-bold uppercase tracking-wider">Dates Logged</span>
            </div>
            <span class="text-2xs bg-theme-card px-1.5 py-0.5 border border-theme-border font-bold text-theme-cyan">
              {dates().length}
            </span>
          </div>

          {/* Date Search Input */}
          <Show when={dates().length > 3}>
            <div class="relative">
              <Search size={13} class="absolute left-2.5 top-1/2 -translate-y-1/2 text-theme-text/40 pointer-events-none" />
              <input
                type="text"
                value={dateSearch()}
                onInput={(e) => setDateSearch(e.currentTarget.value)}
                placeholder="Filter date (YYYY-MM)..."
                class="w-full pl-8 pr-2.5 py-1 text-xs font-mono bg-theme-card border border-theme-border text-theme-text placeholder:text-theme-text/40 focus:outline-none focus:border-theme-cyan"
              />
              <Show when={dateSearch()}>
                <button
                  onClick={() => setDateSearch('')}
                  class="absolute right-2 top-1/2 -translate-y-1/2 text-theme-text/40 hover:text-theme-text"
                >
                  <X size={12} />
                </button>
              </Show>
            </div>
          </Show>

          <Show when={!loadingDates()} fallback={
            <div class="p-8 text-center text-xs font-mono text-theme-text/50 flex flex-col items-center gap-2">
              <Loader2 size={16} class="animate-spin text-theme-cyan" />
              <span>Loading dates...</span>
            </div>
          }>
            <Show when={filteredDates().length > 0} fallback={
              <div class="p-8 text-center text-xs font-mono text-theme-text/50 space-y-1">
                <Calendar size={24} class="mx-auto text-theme-text/40 mb-2" />
                <p>{dateSearch() ? 'No matching dates found' : 'No history records yet'}</p>
              </div>
            }>
              <div class="space-y-1 max-h-[62vh] overflow-y-auto pr-1 custom-scrollbar flex-1">
                <For each={filteredDates()}>
                  {(date) => {
                    const isSelected = () => selectedDate() === date;
                    const dateBadge = formatDateLabel(date);
                    return (
                      <div
                        onClick={() => {
                          if (selectedDate() !== date) {
                            setPage(1);
                            loadHistoryDay(date, 1);
                          }
                        }}
                        class={`w-full group flex items-center justify-between px-3 py-2.5 text-xs font-mono cursor-pointer border transition-all ${
                          isSelected()
                            ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold shadow-sm'
                            : 'bg-theme-card/60 text-theme-text/80 border-theme-border hover:bg-theme-card hover:border-theme-cyan/50 hover:text-theme-text'
                        }`}
                      >
                        <div class="flex items-center gap-2 truncate">
                          <Clock size={13} class={isSelected() ? 'text-theme-card' : 'text-theme-cyan'} />
                          <span class="truncate">{date}</span>
                          <Show when={dateBadge}>
                            <span class={`text-3xs px-1.5 py-0.2 uppercase font-bold tracking-wider border ${
                              isSelected()
                                ? 'bg-theme-card text-theme-text border-theme-card'
                                : 'bg-theme-cyan/15 text-theme-cyan border-theme-cyan/30'
                            }`}>
                              {dateBadge}
                            </span>
                          </Show>
                        </div>
                        <button
                          type="button"
                          onClick={(e) => {
                            e.stopPropagation();
                            handleDeleteDate(date);
                          }}
                          class={`p-1 transition-colors opacity-60 group-hover:opacity-100 ${
                            isSelected()
                              ? 'text-theme-card/80 hover:text-theme-card hover:bg-black/10'
                              : 'text-theme-text/60 hover:text-theme-rose hover:bg-theme-rose/10'
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
        <div class="md:col-span-3 bg-theme-surface border border-theme-border p-4 space-y-4 min-h-[500px] flex flex-col justify-between">
          <div>
            {/* Top Toolbar inside right column */}
            <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-3 border-b border-theme-border">
              <div class="flex items-center gap-2.5 flex-wrap">
                <h3 class="text-theme-text font-heading text-sm font-bold flex items-center gap-1.5">
                  <MessageSquare size={15} class="text-theme-cyan" />
                  <span>{selectedDate() ? `Conversation on ${selectedDate()}` : 'Select a date'}</span>
                </h3>
                <Show when={currentHistory()}>
                  <span class="text-xs font-mono px-2 py-0.5 bg-theme-card border border-theme-border text-theme-cyan font-bold">
                    {currentHistory()!.total_turns ?? currentHistory()!.turns.length} Turns
                  </span>
                </Show>
                <Show when={formatDateLabel(selectedDate())}>
                  <span class="text-2xs font-mono px-1.5 py-0.5 bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/30 font-bold uppercase">
                    {formatDateLabel(selectedDate())}
                  </span>
                </Show>
              </div>

              {/* Controls: Search, Sort Order & Page Size */}
              <div class="flex items-center gap-2 flex-wrap">
                {/* Search Message Filter */}
                <div class="relative">
                  <Search size={12} class="absolute left-2.5 top-1/2 -translate-y-1/2 text-theme-text/40 pointer-events-none" />
                  <input
                    type="text"
                    value={messageSearch()}
                    onInput={(e) => setMessageSearch(e.currentTarget.value)}
                    placeholder="Search in day..."
                    class="pl-7 pr-2.5 py-1 text-xs font-mono bg-theme-card border border-theme-border text-theme-text placeholder:text-theme-text/40 focus:outline-none focus:border-theme-cyan w-36 sm:w-44"
                  />
                  <Show when={messageSearch()}>
                    <button
                      onClick={() => setMessageSearch('')}
                      class="absolute right-2 top-1/2 -translate-y-1/2 text-theme-text/40 hover:text-theme-text"
                    >
                      <X size={12} />
                    </button>
                  </Show>
                </div>

                {/* Sort Order Toggle */}
                <button
                  onClick={toggleSortOrder}
                  disabled={loadingTurns() || !selectedDate()}
                  class="px-2.5 py-1 text-xs font-mono bg-theme-card border border-theme-border hover:border-theme-cyan text-theme-text transition-all flex items-center gap-1 disabled:opacity-40"
                  title={order() === 'desc' ? 'Current: Newest first (click for Oldest first)' : 'Current: Oldest first (click for Newest first)'}
                >
                  <ArrowUpDown size={12} class="text-theme-cyan" />
                  <span>{order() === 'desc' ? 'Newest' : 'Oldest'}</span>
                </button>

                {/* Limit Selector */}
                <div class="flex items-center border border-theme-border bg-theme-card text-xs font-mono">
                  <span class="px-1.5 text-theme-text/50 text-2xs">Limit</span>
                  <select
                    value={limit()}
                    onChange={(e) => handleLimitChange(Number(e.currentTarget.value))}
                    disabled={loadingTurns() || !selectedDate()}
                    class="bg-transparent py-1 pr-1 text-theme-text focus:outline-none cursor-pointer"
                  >
                    <option value={20}>20</option>
                    <option value={25}>25</option>
                    <option value={50}>50</option>
                    <option value={100}>100</option>
                  </select>
                </div>
              </div>
            </div>

            {/* Turns Viewer Area */}
            <Show when={!loadingTurns()} fallback={
              <div class="p-16 text-center text-xs font-mono text-theme-text/50 flex flex-col items-center gap-2.5">
                <Loader2 size={24} class="animate-spin text-theme-cyan" />
                <span>Loading conversation turns...</span>
              </div>
            }>
              <Show when={currentHistory() && currentHistory()!.turns.length > 0} fallback={
                <div class="p-16 text-center text-xs font-mono text-theme-text/50 space-y-2">
                  <MessageSquare size={36} class="mx-auto text-theme-text/40 mb-1" />
                  <p class="font-bold text-theme-text/70">No conversation turns recorded</p>
                  <p class="text-2xs text-theme-text/40">Interactions on this date will be logged automatically.</p>
                </div>
              }>
                <Show when={filteredTurns().length > 0} fallback={
                  <div class="p-12 text-center text-xs font-mono text-theme-text/50 space-y-1">
                    <Filter size={24} class="mx-auto text-theme-text/40 mb-2" />
                    <p>No messages match filter "{messageSearch()}"</p>
                    <button
                      onClick={() => setMessageSearch('')}
                      class="text-2xs text-theme-cyan underline mt-1"
                    >
                      Clear filter
                    </button>
                  </div>
                }>
                  <div class="space-y-3.5 max-h-[60vh] overflow-y-auto pr-1.5 custom-scrollbar pt-3">
                    <For each={filteredTurns()}>
                      {(turn, idx) => {
                        const isAssistant = () => 
                          turn.speaker.toLowerCase().includes('sukma') ||
                          turn.speaker.toLowerCase().includes('assistant') ||
                          turn.speaker.toLowerCase().includes('ai');

                        const turnKey = turn.id || `${turn.time}-${turn.speaker}-${idx()}`;
                        const isCopied = () => copiedTurnKey() === turnKey;

                        return (
                          <div class={`p-3.5 border transition-all ${
                            isAssistant()
                              ? 'bg-theme-bg/80 border-theme-cyan/35 ml-2 sm:ml-8 shadow-sm'
                              : 'bg-theme-card/80 border-theme-border mr-2 sm:mr-8'
                          }`}>
                            {/* Turn Header */}
                            <div class="flex items-center justify-between gap-2 mb-2 pb-1.5 border-b border-theme-border/50">
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

                              <div class="flex items-center gap-2">
                                <span class="text-2xs font-mono text-theme-text/50 flex items-center gap-1.5">
                                  <Clock size={11} />
                                  <span>{turn.time.split('+')[0].split('-')[0]}</span>
                                  {turn.time.includes('+') && (
                                    <span class="px-1.5 py-0.2 bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/30 text-3xs font-bold font-mono">
                                      UTC+{turn.time.split('+')[1]}
                                    </span>
                                  )}
                                  {turn.time.includes('-') && turn.time.split('-').length > 1 && !turn.time.startsWith('-') && (
                                    <span class="px-1.5 py-0.2 bg-theme-card text-theme-text/70 border border-theme-border text-3xs font-bold font-mono">
                                      UTC-{turn.time.split('-')[1]}
                                    </span>
                                  )}
                                </span>
                              </div>
                            </div>

                            {/* Message Body */}
                            <p class="text-xs leading-relaxed text-theme-text/90 font-sans whitespace-pre-wrap select-text pl-0.5">
                              {turn.message}
                            </p>

                            {/* Turn Action Bar */}
                            <div class="mt-2.5 pt-2 border-t border-theme-border/30 flex items-center justify-end gap-2">
                              {/* Copy Button */}
                              <button
                                onClick={() => handleCopyMessage(turnKey, turn.message)}
                                class={`px-2 py-0.5 text-2xs font-mono border transition-all flex items-center gap-1 ${
                                  isCopied()
                                    ? 'bg-theme-cyan/20 border-theme-cyan text-theme-cyan font-bold'
                                    : 'bg-theme-card border-theme-border text-theme-text/60 hover:text-theme-text hover:border-theme-border/80'
                                }`}
                                title="Copy message to clipboard"
                              >
                                <Show when={isCopied()} fallback={<Copy size={11} />}>
                                  <Check size={11} class="text-theme-cyan" />
                                </Show>
                                <span>{isCopied() ? 'Copied' : 'Copy'}</span>
                              </button>

                              {/* On-Demand Bridge to Project Workspace Button */}
                              <Show when={turn.id}>
                                <button
                                  onClick={() => handleSendToProject(turn.id!, turn.message)}
                                  class="px-2 py-0.5 text-2xs font-mono bg-theme-cyan/10 border border-theme-cyan/30 text-theme-cyan hover:bg-theme-cyan/20 transition-all flex items-center gap-1 font-semibold"
                                  title="Send this interaction as a task to Project Workspace"
                                >
                                  <FolderGit2 size={11} />
                                  <span>Send Task</span>
                                </button>
                              </Show>
                            </div>
                          </div>
                        );
                      }}
                    </For>
                  </div>
                </Show>
              </Show>
            </Show>
          </div>

          {/* Pagination Controls */}
          <Show when={currentHistory() && (currentHistory()!.total_pages || 1) > 1}>
            <div class="pt-3 border-t border-theme-border flex flex-col sm:flex-row items-center justify-between gap-2">
              <span class="text-xs font-mono text-theme-text/60">
                Page <span class="text-theme-text font-bold">{page()}</span> of{' '}
                <span class="text-theme-text font-bold">{currentHistory()!.total_pages}</span>
                <span class="ml-2 text-theme-text/40">
                  ({currentHistory()!.total_turns ?? currentHistory()!.turns.length} total turns)
                </span>
              </span>

              <div class="flex items-center gap-1.5">
                <button
                  onClick={() => handlePageChange(1)}
                  disabled={page() <= 1 || loadingTurns()}
                  class="p-1.5 text-xs bg-theme-card border border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan transition-all disabled:opacity-40 disabled:cursor-not-allowed"
                  title="First page"
                >
                  <ChevronsLeft size={13} />
                </button>

                <button
                  onClick={() => handlePageChange(page() - 1)}
                  disabled={page() <= 1 || loadingTurns()}
                  class="px-2.5 py-1 text-xs font-semibold bg-theme-card border border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan transition-all flex items-center gap-1 disabled:opacity-40 disabled:cursor-not-allowed"
                >
                  <ChevronLeft size={13} />
                  <span>Prev</span>
                </button>

                <span class="px-2 py-1 text-xs font-mono font-bold bg-theme-card border border-theme-cyan/40 text-theme-cyan">
                  {page()}
                </span>

                <button
                  onClick={() => handlePageChange(page() + 1)}
                  disabled={page() >= (currentHistory()?.total_pages || 1) || loadingTurns()}
                  class="px-2.5 py-1 text-xs font-semibold bg-theme-card border border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan transition-all flex items-center gap-1 disabled:opacity-40 disabled:cursor-not-allowed"
                >
                  <span>Next</span>
                  <ChevronRight size={13} />
                </button>

                <button
                  onClick={() => handlePageChange(currentHistory()?.total_pages || 1)}
                  disabled={page() >= (currentHistory()?.total_pages || 1) || loadingTurns()}
                  class="p-1.5 text-xs bg-theme-card border border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan transition-all disabled:opacity-40 disabled:cursor-not-allowed"
                  title="Last page"
                >
                  <ChevronsRight size={13} />
                </button>
              </div>
            </div>
          </Show>
        </div>
      </div>

      {/* ── Modal: Send to Project Workspace ───────────────────────────── */}
      <Show when={showProjectModal()}>
        <div class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
          <div class="bg-theme-surface border border-theme-border max-w-md w-full p-5 space-y-4 shadow-2xl">
            <div class="flex items-center justify-between border-b border-theme-border pb-3">
              <div class="flex items-center gap-2">
                <FolderGit2 class="w-4 h-4 text-theme-cyan" />
                <h3 class="text-sm font-bold text-theme-text font-heading">Send Task to Project Workspace</h3>
              </div>
              <button
                onClick={() => setShowProjectModal(false)}
                class="text-theme-text/50 hover:text-theme-text"
              >
                <X class="w-4 h-4" />
              </button>
            </div>

            <div class="space-y-3 text-xs">
              <div>
                <label class="block text-theme-text/70 mb-1 font-mono">Target Workspace Project:</label>
                <select
                  value={selectedBridgeProjectId()}
                  onChange={(e) => setSelectedBridgeProjectId(e.currentTarget.value)}
                  class="w-full px-3 py-2 bg-theme-card border border-theme-border text-theme-text font-mono focus:outline-none focus:border-theme-cyan"
                >
                  <For each={availableProjects()}>
                    {(proj) => (
                      <option value={proj.id}>{proj.name}</option>
                    )}
                  </For>
                </select>
              </div>

              <div>
                <label class="block text-theme-text/70 mb-1 font-mono">Task Title / Objective:</label>
                <input
                  type="text"
                  value={bridgeTitle()}
                  onInput={(e) => setBridgeTitle(e.currentTarget.value)}
                  class="w-full px-3 py-2 bg-theme-card border border-theme-border text-theme-text font-mono focus:outline-none focus:border-theme-cyan"
                />
              </div>

              <div class="p-2.5 bg-theme-card/60 border border-theme-border text-xs text-theme-text/60 leading-relaxed">
                A new task session will be created in the selected project, transferring this conversation context into an isolated agent session.
              </div>
            </div>

            <div class="flex items-center justify-end gap-2 border-t border-theme-border pt-3">
              <button
                onClick={() => setShowProjectModal(false)}
                class="px-3.5 py-1.5 text-xs text-theme-text/60 hover:text-theme-text"
              >
                Cancel
              </button>
              <button
                onClick={handleConfirmCreateTask}
                disabled={creatingTask() || !selectedBridgeProjectId()}
                class="px-4 py-1.5 text-xs font-semibold bg-theme-cyan text-theme-bg hover:bg-theme-cyan/90 transition-all flex items-center gap-1.5 disabled:opacity-40"
              >
                <Show when={creatingTask()} fallback={<FolderGit2 size={12} />}>
                  <Loader2 size={12} class="animate-spin" />
                </Show>
                <span>Create Task Session</span>
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};
