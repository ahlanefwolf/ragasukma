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
  ChevronLeft,
  Download,
  FolderGit2,
  X,
  Layers,
  Loader2
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

  // Pagination Signals
  const [page, setPage] = createSignal(1);
  const [limit] = createSignal(20);

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
        // Switch to Agent Workspace tab
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
      const data = await api.getHistoryByDate(date, targetPage, limit());
      setCurrentHistory(data);
      if (data.page) setPage(data.page);
    } catch {
      setCurrentHistory({ date, turns: [] });
    } finally {
      setLoadingTurns(false);
    }
  };

  const handlePrevPage = () => {
    if (page() > 1 && !loadingTurns() && selectedDate()) {
      const prev = page() - 1;
      setPage(prev);
      loadHistoryDay(selectedDate(), prev);
    }
  };

  const handleNextPage = () => {
    const totalPages = currentHistory()?.total_pages || 1;
    if (page() < totalPages && !loadingTurns() && selectedDate()) {
      const next = page() + 1;
      setPage(next);
      loadHistoryDay(selectedDate(), next);
    }
  };

  onMount(() => {
    loadDates(true);
  });

  const handleDeleteDate = async (date: string) => {
    const ok = await showConfirm({
      title: 'Delete History Item',
      message: 'Are you sure you want to delete this item from history?',
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
            <h1 class="text-xl font-heading font-bold text-theme-text tracking-wide uppercase">
              Conversation History
            </h1>
            <p class="text-xs text-theme-text/60 font-mono mt-0.5">
              Review, search and export daily voice interactions stored in SQLite
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <button
            onClick={() => loadDates(false)}
            disabled={loadingDates()}
            class="px-3 py-1.5 text-xs font-semibold bg-theme-card border border-theme-border hover:border-theme-cyan text-theme-text transition-all flex items-center gap-1.5"
            title="Refresh date list"
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
              <span>Export MD</span>
            </button>
          </Show>
        </div>
      </div>

      {/* Main Grid: Date Selector + Turn Viewer */}
      <div class="grid grid-cols-1 md:grid-cols-4 gap-5">
        {/* Left Column: Dates list */}
        <div class="md:col-span-1 bg-theme-surface border border-theme-border p-3 space-y-2">
          <div class="flex items-center gap-2 px-2 py-1 text-xs font-mono text-theme-text/60 border-b border-theme-border pb-2">
            <Calendar size={14} />
            <span class="font-bold uppercase tracking-wider">Dates Logged</span>
            <span class="ml-auto text-2xs bg-theme-card px-1.5 py-0.5 border border-theme-border">
              {dates().length}
            </span>
          </div>

          <Show when={!loadingDates()} fallback={
            <div class="p-6 text-center text-xs font-mono text-theme-text/50">
              Loading dates...
            </div>
          }>
            <Show when={dates().length > 0} fallback={
              <div class="p-6 text-center text-xs font-mono text-theme-text/50">
                No history records yet.
              </div>
            }>
              <div class="space-y-1 max-h-[60vh] overflow-y-auto pr-1 custom-scrollbar">
                <For each={dates()}>
                  {(date) => {
                    const isSelected = () => selectedDate() === date;
                    return (
                      <div
                        onClick={() => {
                          setPage(1);
                          loadHistoryDay(date, 1);
                        }}
                        class={`w-full flex items-center justify-between px-3 py-2 text-xs font-mono cursor-pointer border transition-all ${
                          isSelected()
                            ? 'bg-theme-cyan text-theme-card border-theme-cyan font-bold shadow-sm'
                            : 'bg-theme-card/60 text-theme-text/80 border-theme-border hover:bg-theme-card hover:text-theme-text'
                        }`}
                      >
                        <div class="flex items-center gap-2">
                          <Clock size={13} class={isSelected() ? 'text-theme-card' : 'text-theme-cyan'} />
                          <span>{date}</span>
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
                              : 'text-theme-text/70 hover:text-theme-rose'
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
        <div class="md:col-span-3 bg-theme-surface border border-theme-border p-4 space-y-4 min-h-[400px] flex flex-col justify-between">
          <div>
            <div class="flex items-center justify-between pb-3 border-b border-theme-border">
              <div class="flex items-center gap-2">
                <h3 class="text-theme-text font-heading text-sm font-bold">
                  {selectedDate() ? `Conversation on ${selectedDate()}` : 'Select a date'}
                </h3>
                <Show when={currentHistory()}>
                  <span class="text-xs font-mono px-2 py-0.5 bg-theme-card border border-theme-border text-theme-cyan">
                    {currentHistory()!.total_turns ?? currentHistory()!.turns.length} Turns
                  </span>
                </Show>
              </div>

              <Show when={currentHistory() && (currentHistory()!.total_pages || 1) > 1}>
                <span class="text-xs font-mono text-theme-text/50">
                  Page {page()} of {currentHistory()!.total_pages}
                </span>
              </Show>
            </div>

            <Show when={!loadingTurns()} fallback={
              <div class="p-12 text-center text-xs font-mono text-theme-text/50">
                Loading chat turns...
              </div>
            }>
              <Show when={currentHistory() && currentHistory()!.turns.length > 0} fallback={
                <div class="p-12 text-center text-xs font-mono text-theme-text/50 space-y-2">
                  <MessageSquare size={32} class="mx-auto text-theme-text/60" />
                  <p>No messages recorded for this date.</p>
                </div>
              }>
                <div class="space-y-3.5 max-h-[60vh] overflow-y-auto pr-1 custom-scrollbar pt-2">
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
                            <span class="text-2xs font-mono text-theme-text/50 flex items-center gap-1.5">
                              <Clock size={11} />
                              <span>{turn.time.split('+')[0].split('-')[0]}</span>
                              {turn.time.includes('+') && (
                                <span class="px-1.5 py-0.2 bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/30 text-2xs font-bold">
                                  UTC+{turn.time.split('+')[1]}
                                </span>
                              )}
                              {turn.time.includes('-') && turn.time.split('-').length > 1 && !turn.time.startsWith('-') && (
                                <span class="px-1.5 py-0.2 bg-theme-card text-theme-text/70 border border-theme-border text-2xs font-bold">
                                  UTC-{turn.time.split('-')[1]}
                                </span>
                              )}
                            </span>
                          </div>
                          <p class="text-xs leading-relaxed text-theme-text/90 font-sans whitespace-pre-wrap select-text">
                            {turn.message}
                          </p>

                          {/* On-Demand Bridge to Project Workspace Button */}
                          <Show when={turn.id}>
                            <div class="mt-2.5 pt-2 border-t border-theme-border/30 flex justify-end">
                              <button
                                onClick={() => handleSendToProject(turn.id!, turn.message)}
                                class="text-xs font-mono text-theme-cyan/80 hover:text-theme-cyan hover:bg-theme-cyan/10 px-2 py-1 rounded transition-colors flex items-center gap-1.5"
                                title="Send this conversation as a task session to Project Workspace"
                              >
                                <FolderGit2 size={12} />
                                <span>Send Task to Project</span>
                              </button>
                            </div>
                          </Show>
                        </div>
                      );
                    }}
                  </For>
                </div>
              </Show>
            </Show>
          </div>

          {/* Pagination Controls */}
          <Show when={currentHistory() && (currentHistory()!.total_pages || 1) > 1}>
            <div class="pt-3 border-t border-theme-border flex items-center justify-between">
              <span class="text-xs font-mono text-theme-text/60">
                Showing page <span class="text-theme-text font-bold">{page()}</span> of{' '}
                <span class="text-theme-text font-bold">{currentHistory()!.total_pages}</span>
              </span>

              <div class="flex items-center gap-2">
                <button
                  onClick={handlePrevPage}
                  disabled={page() <= 1 || loadingTurns()}
                  class="px-3 py-1.5 text-xs font-semibold bg-theme-card border border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan transition-all flex items-center gap-1 disabled:opacity-40 disabled:cursor-not-allowed"
                >
                  <ChevronLeft size={14} />
                  <span>Previous</span>
                </button>

                <button
                  onClick={handleNextPage}
                  disabled={page() >= (currentHistory()?.total_pages || 1) || loadingTurns()}
                  class="px-3 py-1.5 text-xs font-semibold bg-theme-card border border-theme-border text-theme-text/80 hover:text-theme-text hover:border-theme-cyan transition-all flex items-center gap-1 disabled:opacity-40 disabled:cursor-not-allowed"
                >
                  <span>Next</span>
                  <ChevronRight size={14} />
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