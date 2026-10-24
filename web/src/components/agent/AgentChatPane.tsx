import { Component, For, Show, createMemo, createSignal, createEffect } from 'solid-js';
import { 
  MessageSquare, 
  Cpu, 
  RefreshCw, 
  Bot, 
  Plus, 
  Sparkles, 
  Wrench, 
  Brain, 
  ChevronDown, 
  ChevronUp, 
  AtSign, 
  FileCode, 
  Folder, 
  Send, 
  Loader2,
  Square,
  History,
  ArrowDown
} from 'lucide-solid';
import { Project, AgentSession, AgentMessage, WorkspaceFileItem } from '../../services';
import { MarkdownView } from './MarkdownView';
import { ToolCallCard, ToolResultCard } from './ToolCards';
import { countMessageLines } from './highlightUtils';
import { SearchableSelect } from '../SearchableSelect';

// Compact & Collapsible Reasoning / Chain of Thought Component
const ThoughtBubble: Component<{ thought: string }> = (props) => {
  const [isExpanded, setIsExpanded] = createSignal(false);
  const charCount = () => props.thought?.length ?? 0;

  return (
    <div class="w-full max-w-full mb-2 rounded-lg bg-amber-500/10 border border-amber-500/20 text-xs text-amber-200/90 font-mono overflow-hidden transition-all">
      <button
        type="button"
        onClick={() => setIsExpanded(!isExpanded())}
        class="w-full flex items-center justify-between px-2.5 py-1.5 hover:bg-amber-500/15 cursor-pointer text-left transition-colors select-none"
      >
        <div class="flex items-center gap-1.5 font-semibold text-2xs text-amber-400 uppercase tracking-wider">
          <Brain class="w-3.5 h-3.5 text-amber-400 shrink-0" />
          <span>Reasoning / Thought Process</span>
          <span class="text-2xs lowercase font-normal opacity-70">({charCount()} chars)</span>
        </div>
        <div class="flex items-center gap-1 text-2xs text-amber-400/80">
          <span>{isExpanded() ? 'Hide' : 'Show'}</span>
          <ChevronDown class={`w-3.5 h-3.5 transition-transform duration-200 ${isExpanded() ? 'rotate-180' : ''}`} />
        </div>
      </button>

      <Show when={isExpanded()}>
        <div class="p-2.5 pt-1.5 border-t border-amber-500/15 whitespace-pre-wrap text-xs leading-relaxed opacity-90 max-h-80 overflow-y-auto custom-scrollbar">
          {props.thought}
        </div>
      </Show>
    </div>
  );
};

export const AgentChatPane: Component<{
  currentProject: Project | null | undefined;
  currentSession: AgentSession | null | undefined;
  activeSessionModel: string;
  setActiveSessionModel: (m: string) => void;
  defaultBrainModel: string;
  availableModels: Array<{ id: string; label: string }>;
  isRefreshingModels: boolean;
  onRefreshModels: () => void;
  setMessagesContainerRef: (el: HTMLDivElement) => void;
  onMessagesScroll: () => void;
  visibleMessages: AgentMessage[];
  loadingMessages: boolean;
  isMessageCollapsed: (id: string) => boolean;
  toggleCollapseMessage: (id: string) => void;
  isCurrentSessionBusy: boolean;
  currentStepInfo: string | null;
  streamingText?: string;
  streamingThought?: string;
  onQuickCreateSession: () => void;
  onOpenFilesTab: () => void;
  autocompleteOpen: boolean;
  autocompleteSuggestions: WorkspaceFileItem[];
  autocompleteIndex: number;
  onSelectAutocompleteFile: (file: WorkspaceFileItem) => void;
  setChatInputRef: (el: HTMLTextAreaElement) => void;
  userInput: string;
  onChatInput: (e: InputEvent & { currentTarget: HTMLTextAreaElement }) => void;
  onChatKeyDown: (e: KeyboardEvent & { currentTarget: HTMLTextAreaElement }) => void;
  onSendMessage: () => void;
  onStopSession: () => void;
}> = (props) => {
  let backdropRef: HTMLDivElement | undefined;

  const tokens = createMemo(() => {
    const text = props.userInput || '';
    if (!text) return [];
    const regex = /(?:^|\s)(@[a-zA-Z0-9_\-\./\\]*(?::\d+(?:-\d+)?)?)/g;
    const result: { text: string; isTag: boolean }[] = [];
    let lastIndex = 0;
    let match: RegExpExecArray | null;

    while ((match = regex.exec(text)) !== null) {
      const fullMatch = match[0];
      const tag = match[1];
      const tagIndex = match.index + fullMatch.indexOf(tag);

      if (tagIndex > lastIndex) {
        result.push({ text: text.substring(lastIndex, tagIndex), isTag: false });
      }

      result.push({ text: tag, isTag: true });
      lastIndex = tagIndex + tag.length;
    }

    if (lastIndex < text.length) {
      result.push({ text: text.substring(lastIndex), isTag: false });
    }

    return result;
  });

  const hasTags = createMemo(() => tokens().some((t) => t.isTag));

  // Upward Lazy Loading / Windowing for Long Conversation Tasks
  const INITIAL_VISIBLE_COUNT = 25;
  const [visibleWindowCount, setVisibleWindowCount] = createSignal(INITIAL_VISIBLE_COUNT);
  const [isScrolledUp, setIsScrolledUp] = createSignal(false);

  let messagesContainerEl: HTMLDivElement | undefined;
  let trackedSessionId: string | null = null;
  let lastKnownMsgCount = 0;

  // Track session change & initial loading accurately
  createEffect(() => {
    const sId = props.currentSession?.id ?? null;
    const isLoading = props.loadingMessages;
    const currentLen = props.visibleMessages?.length ?? 0;

    if (sId !== trackedSessionId) {
      // Switched session or initial session mount
      trackedSessionId = sId;
      setVisibleWindowCount(INITIAL_VISIBLE_COUNT);
      lastKnownMsgCount = currentLen;
      return;
    }

    if (isLoading) {
      // When backend is fetching messages for session, ensure window count is clamped to initial
      setVisibleWindowCount(INITIAL_VISIBLE_COUNT);
      return;
    }

    // When initial fetch finishes (transitioned from 0 messages to loaded list)
    if (lastKnownMsgCount === 0 && currentLen > 0) {
      lastKnownMsgCount = currentLen;
      setVisibleWindowCount(INITIAL_VISIBLE_COUNT);
      return;
    }

    // Live message additions during active chat session
    if (currentLen > lastKnownMsgCount) {
      const added = currentLen - lastKnownMsgCount;
      if (added <= 10) {
        setVisibleWindowCount((prev) => prev + added);
      }
      lastKnownMsgCount = currentLen;
    } else if (currentLen < lastKnownMsgCount) {
      lastKnownMsgCount = currentLen;
    }
  });

  const totalMessagesCount = () => props.visibleMessages?.length ?? 0;
  const hasEarlierMessages = () => totalMessagesCount() > visibleWindowCount();
  const hiddenEarlierCount = () => Math.max(0, totalMessagesCount() - visibleWindowCount());

  const renderedMessages = createMemo(() => {
    const msgs = props.visibleMessages || [];
    if (msgs.length <= visibleWindowCount()) {
      return msgs;
    }
    return msgs.slice(msgs.length - visibleWindowCount());
  });

  const loadEarlierMessages = () => {
    if (!messagesContainerEl) {
      setVisibleWindowCount((c) => c + 25);
      return;
    }
    const prevScrollHeight = messagesContainerEl.scrollHeight;
    const prevScrollTop = messagesContainerEl.scrollTop;

    setVisibleWindowCount((c) => c + 25);

    requestAnimationFrame(() => {
      if (messagesContainerEl) {
        const heightDiff = messagesContainerEl.scrollHeight - prevScrollHeight;
        messagesContainerEl.scrollTop = prevScrollTop + heightDiff;
      }
    });
  };

  const loadAllMessages = () => {
    if (!messagesContainerEl) {
      setVisibleWindowCount(totalMessagesCount());
      return;
    }
    const prevScrollHeight = messagesContainerEl.scrollHeight;
    const prevScrollTop = messagesContainerEl.scrollTop;

    setVisibleWindowCount(totalMessagesCount());

    requestAnimationFrame(() => {
      if (messagesContainerEl) {
        const heightDiff = messagesContainerEl.scrollHeight - prevScrollHeight;
        messagesContainerEl.scrollTop = prevScrollTop + heightDiff;
      }
    });
  };

  const handleScroll = (e: Event) => {
    const el = e.currentTarget as HTMLDivElement;
    const distanceToBottom = el.scrollHeight - (el.scrollTop + el.clientHeight);
    setIsScrolledUp(distanceToBottom > 150);
    props.onMessagesScroll();
  };

  const scrollToBottom = () => {
    if (messagesContainerEl) {
      messagesContainerEl.scrollTo({ top: messagesContainerEl.scrollHeight, behavior: 'smooth' });
    }
  };

  return (
    <div class="bg-white/[0.03] border border-white/10 rounded-xl flex-1 flex flex-col min-h-0 overflow-hidden shadow-lg relative">
      {/* Session Header */}
      <div class="px-4 py-2.5 border-b border-white/10 flex flex-wrap items-center justify-between gap-2 bg-white/[0.01]">
        <div class="flex items-center gap-2 min-w-0">
          <MessageSquare class="w-4 h-4 text-indigo-400 shrink-0" />
          <div class="min-w-0 flex items-center gap-2">
            <span class="text-xs font-semibold text-white truncate">
              {props.currentSession?.title || 'Select or Create a Task Session'}
            </span>
            <Show when={props.currentSession}>
              <span class="text-2xs uppercase px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-medium">
                {props.currentSession?.status}
              </span>
            </Show>
          </div>
        </div>

        {/* Dynamic Model Switcher */}
        <Show when={props.currentSession}>
          <div class="flex items-center gap-1.5 bg-white/5 border border-white/10 rounded-lg px-2 py-1 text-xs">
            <Cpu class="w-3.5 h-3.5 text-indigo-400 shrink-0" />
            <SearchableSelect
              value={props.activeSessionModel || props.currentSession?.model || props.defaultBrainModel}
              onChange={(val) => props.setActiveSessionModel(val)}
              options={[
                ...(props.defaultBrainModel
                  ? [
                      {
                        value: props.defaultBrainModel,
                        label: `⭐ ${props.defaultBrainModel}`,
                        badge: 'Brain',
                      },
                    ]
                  : []),
                ...props.availableModels
                  .filter((m) => m.id !== props.defaultBrainModel)
                  .map((m) => ({
                    value: m.id,
                    label: m.label || m.id,
                    badge: m.id.startsWith('ag/')
                      ? 'Antigravity'
                      : m.id.startsWith('cx/')
                      ? 'Codex'
                      : undefined,
                  })),
              ]}
              buttonClass="flex items-center justify-between gap-1.5 text-white font-mono text-xs max-w-[210px] truncate hover:text-indigo-300 transition-colors"
              dropdownClass="absolute right-0 mt-2 w-72 bg-[#12161f] border border-white/10 rounded-lg shadow-2xl z-50 overflow-hidden flex flex-col max-h-80"
              searchPlaceholder="Filter AI models..."
            />
            <button
              type="button"
              onClick={props.onRefreshModels}
              disabled={props.isRefreshingModels}
              class="text-white/70 hover:text-white disabled:opacity-50 transition-colors p-0.5 ml-0.5 shrink-0"
              title="Sync latest available models"
            >
              <RefreshCw class={`w-3 h-3 ${props.isRefreshingModels ? 'animate-spin' : ''}`} />
            </button>
          </div>
        </Show>
      </div>

      {/* Messages Stream */}
      <div 
        ref={(el) => {
          messagesContainerEl = el;
          props.setMessagesContainerRef(el);
        }}
        onScroll={handleScroll}
        class="flex-1 overflow-y-auto p-4 space-y-3 custom-scrollbar"
      >
        {/* Upward Lazy Load Banner for Earlier Messages */}
        <Show when={props.currentSession && hasEarlierMessages()}>
          <div class="flex items-center justify-center gap-2 py-2 mb-1 border-b border-white/[0.06] bg-white/[0.02] rounded-lg select-none">
            <button
              type="button"
              onClick={loadEarlierMessages}
              class="px-3 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 active:scale-95 text-white/80 hover:text-white text-xs flex items-center gap-1.5 border border-white/10 transition-colors shadow-sm font-sans"
            >
              <History class="w-3.5 h-3.5 text-indigo-400" />
              <span>Load 25 Earlier Messages ({hiddenEarlierCount()} hidden)</span>
            </button>
            <button
              type="button"
              onClick={loadAllMessages}
              class="px-2.5 py-1.5 rounded-lg bg-white/5 hover:bg-white/10 active:scale-95 text-white/50 hover:text-white/80 text-xs transition-colors font-sans"
              title="Load all earlier messages in this session"
            >
              Show All
            </button>
          </div>
        </Show>

        <Show when={!props.currentSession}>
          <div class="h-full flex flex-col items-center justify-center text-center text-white/70 py-12">
            <Bot class="w-10 h-10 mb-2 text-white/20" />
            <p class="text-xs">Select a task session on the left or create a new session to start the agent.</p>
            <button
              onClick={props.onQuickCreateSession}
              class="mt-3 px-3.5 py-1.5 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium flex items-center gap-1.5 transition-colors"
            >
              <Plus class="w-3.5 h-3.5" />
              New Task
            </button>
          </div>
        </Show>

        <Show when={props.currentSession && props.visibleMessages.length === 0 && !props.loadingMessages}>
          <div class="h-full flex flex-col items-center justify-center text-center text-white/70 py-12">
            <Sparkles class="w-8 h-8 mb-2 text-indigo-400/40" />
            <p class="text-xs font-medium text-white/60">This task session is empty</p>
            <p class="text-xs text-white/60 max-w-xs mt-1">
              Type instructions or goals below to run the autonomous agent on the workspace folders.
            </p>
          </div>
        </Show>

        <For each={renderedMessages()}>
          {(msg) => (
            <div class={`flex flex-col ${msg.role === 'user' ? 'items-end' : 'items-start'}`}>
              <div class="text-2xs text-theme-text/70 mb-1 px-1 flex items-center gap-1.5">
                <Show when={msg.role === 'user'} fallback={
                  <Show when={msg.role === 'tool'} fallback={
                    <div class="flex items-center gap-1 text-theme-cyan">
                      <Bot class="w-3 h-3" />
                      <span>Agent</span>
                    </div>
                  }>
                    <div class="flex items-center gap-1 text-emerald-400">
                      <Wrench class="w-3 h-3" />
                      <span>Tool Output</span>
                    </div>
                  </Show>
                }>
                  <span class="font-medium text-theme-text/70">You</span>
                </Show>
                <span>• #{msg.sequence_num}</span>
              </div>

              {/* Tool Calls badge if present on assistant */}
              <Show when={msg.tool_calls}>
                <div class="w-full max-w-full min-w-0 overflow-hidden">
                  <ToolCallCard toolCalls={msg.tool_calls!} />
                </div>
              </Show>

              {/* Reasoning Thought Collapse if present */}
              <Show when={msg.thought}>
                <ThoughtBubble thought={msg.thought!} />
              </Show>

              {/* Tool Result Bubble if role == 'tool' */}
              <Show when={msg.role === 'tool'}>
                <div class="w-full max-w-full min-w-0 overflow-hidden">
                  <ToolResultCard toolCallId={msg.tool_call_id} content={msg.content} />
                </div>
              </Show>

              {/* Standard Message Bubble */}
              <Show when={msg.role !== 'tool' && msg.content}>
                <div class={`flex flex-col space-y-1 w-full max-w-full ${msg.role === 'user' ? 'items-end' : 'items-start'}`}>
                  <Show when={msg.role === 'assistant'}>
                    <div class="flex items-center justify-between text-2xs text-theme-text/70 px-1">
                      <span class="font-mono flex items-center gap-1">
                        <Bot class="w-3 h-3 text-theme-cyan" />
                        Generative Assistant ({countMessageLines(msg.content)} {countMessageLines(msg.content) === 1 ? 'line' : 'lines'})
                      </span>
                      <button
                        type="button"
                        onClick={() => props.toggleCollapseMessage(msg.id)}
                        class="flex items-center gap-1 px-2 py-0.5 rounded bg-theme-bg/60 hover:bg-theme-card border border-theme-border/60 hover:text-theme-cyan transition-colors text-2xs"
                        title={props.isMessageCollapsed(msg.id) ? "Show full message" : "Collapse generative output"}
                      >
                        <Show when={props.isMessageCollapsed(msg.id)} fallback={<><ChevronUp class="w-3 h-3" /> Collapse</>}>
                          <ChevronDown class="w-3 h-3" /> Expand ({countMessageLines(msg.content)} {countMessageLines(msg.content) === 1 ? 'line' : 'lines'})
                        </Show>
                      </button>
                    </div>
                  </Show>

                  <Show when={props.isMessageCollapsed(msg.id)}>
                    <div
                      onClick={() => props.toggleCollapseMessage(msg.id)}
                      class="cursor-pointer w-full rounded-xl p-2.5 bg-theme-cyan/10 border border-theme-cyan/20 text-xs text-theme-cyan/90 hover:bg-theme-cyan/15 transition-all flex items-center justify-between"
                    >
                      <span class="truncate italic text-xs">Generative output collapsed ({countMessageLines(msg.content)} {countMessageLines(msg.content) === 1 ? 'line' : 'lines'})</span>
                      <span class="text-2xs font-semibold text-theme-cyan underline ml-2 shrink-0">Expand</span>
                    </div>
                  </Show>

                  <Show when={!props.isMessageCollapsed(msg.id)}>
                    <div
                      class={`w-fit max-w-full rounded-xl px-4 py-3 text-xs leading-relaxed ${
                        msg.role === 'user'
                          ? 'bg-theme-card border border-theme-border text-theme-text rounded-br-none whitespace-pre-wrap shadow-sm'
                          : 'bg-theme-bg/80 border border-theme-border text-theme-text/90 rounded-bl-none shadow-sm'
                      }`}
                    >
                      <Show
                        when={msg.role === 'assistant'}
                        fallback={<div class="select-text">{msg.content || ''}</div>}
                      >
                        <MarkdownView content={msg.content || ''} />
                      </Show>
                    </div>
                  </Show>
                </div>
              </Show>
            </div>
          )}
        </For>

        {/* Live Streaming Reasoning / Thought Output */}
        <Show when={props.streamingThought && props.streamingThought.trim().length > 0}>
          <div class="flex flex-col items-start w-full max-w-full mb-3">
            <div class="text-2xs text-amber-400 mb-1 px-1 flex items-center gap-1.5 font-semibold uppercase tracking-wider">
              <Brain class="w-3.5 h-3.5 text-amber-400 shrink-0 animate-pulse" />
              <span>Reasoning (Live...)</span>
            </div>
            <div class="w-full max-w-full rounded-xl px-4 py-3 text-xs leading-relaxed bg-amber-500/10 border border-amber-500/25 text-amber-200/90 font-mono whitespace-pre-wrap max-h-56 overflow-y-auto custom-scrollbar shadow-sm">
              {props.streamingThought}
              <span class="inline-block w-1.5 h-3.5 bg-amber-400 animate-pulse ml-1 align-middle rounded-xs" />
            </div>
          </div>
        </Show>

        {/* Live Streaming Assistant Output */}
        <Show when={props.streamingText && props.streamingText.trim().length > 0}>
          <div class="flex flex-col items-start w-full max-w-full">
            <div class="text-2xs text-white/70 mb-1 px-1 flex items-center gap-1.5">
              <div class="flex items-center gap-1 text-indigo-400">
                <Bot class="w-3 h-3 animate-pulse" />
                <span class="font-medium">Agent (Generating response...)</span>
              </div>
            </div>
            <div class="w-fit max-w-full rounded-xl px-4 py-3 text-xs leading-relaxed bg-white/[0.04] border border-indigo-500/30 text-white/90 rounded-bl-none shadow-md">
              <MarkdownView content={props.streamingText || ''} />
              <span class="inline-block w-1.5 h-3.5 bg-indigo-400 animate-pulse ml-1 align-middle rounded-sm" />
            </div>
          </div>
        </Show>

        {/* Active Execution Spinner */}
        <Show when={props.isCurrentSessionBusy}>
          <div class="flex items-center gap-2.5 p-3 bg-indigo-500/10 border border-indigo-500/20 rounded-xl text-xs text-indigo-300 w-fit animate-pulse">
            <Loader2 class="w-4 h-4 animate-spin text-amber-400 shrink-0" />
            <div>
              <div class="font-semibold text-white">
                {props.currentStepInfo || 'Agent is processing this task session...'}
              </div>
              <div class="text-2xs text-white/50">
                Task is active in background. You can safely switch tasks or projects.
              </div>
            </div>
          </div>
        </Show>
      </div>

      {/* Floating Jump to Bottom Button */}
      <Show when={isScrolledUp()}>
        <button
          type="button"
          onClick={scrollToBottom}
          class="absolute right-6 bottom-24 z-30 flex items-center gap-1.5 px-3 py-1.5 bg-indigo-600/90 hover:bg-indigo-600 active:scale-95 text-white text-xs font-medium rounded-full shadow-xl border border-indigo-400/40 backdrop-blur-md transition-all hover:scale-105 select-none"
          title="Jump to latest message"
        >
          <ArrowDown class="w-3.5 h-3.5" />
          <span>Jump to bottom</span>
        </button>
      </Show>

      {/* Chat Input Bar (Unified Modern Card) */}
      <Show when={props.currentSession}>
        <div class="p-3 border-t border-white/10 bg-white/[0.01]">
          <div class="relative bg-white/[0.03] hover:bg-white/[0.04] focus-within:bg-white/[0.05] border border-white/10 focus-within:border-indigo-500/60 rounded-2xl p-2.5 transition-all shadow-sm">
            {/* Autocomplete Dropdown Popup */}
            <Show when={props.autocompleteOpen && props.autocompleteSuggestions.length > 0}>
              <div class="absolute bottom-full mb-2 left-0 right-0 max-h-60 z-50 overflow-y-auto bg-slate-900/95 border border-indigo-500/30 rounded-xl shadow-2xl backdrop-blur-md p-1.5 custom-scrollbar">
                <div class="px-2 py-1 text-2xs font-semibold text-indigo-400 border-b border-white/10 flex items-center justify-between mb-1">
                  <span class="flex items-center gap-1"><AtSign class="w-3 h-3" /> Tag Workspace File</span>
                  <span class="text-white/70 font-normal">Use ↑ ↓ then Enter / Tab</span>
                </div>
                <For each={props.autocompleteSuggestions}>
                  {(file, idx) => (
                    <div
                      class={`px-2.5 py-1.5 rounded-lg text-xs font-mono flex items-center justify-between cursor-pointer transition-colors ${
                        idx() === props.autocompleteIndex
                          ? 'bg-indigo-600 text-white shadow-sm'
                          : 'hover:bg-white/10 text-white/80'
                      }`}
                      onClick={() => props.onSelectAutocompleteFile(file)}
                    >
                      <div class="flex items-center gap-2 min-w-0">
                        <Show when={file.is_dir} fallback={<FileCode class="w-3.5 h-3.5 shrink-0 opacity-70" />}>
                          <Folder class="w-3.5 h-3.5 text-amber-400 shrink-0" />
                        </Show>
                        <span class="font-semibold truncate">{file.name}{file.is_dir ? '/' : ''}</span>
                        <span class="text-2xs opacity-50 truncate">@{file.relative_path}</span>
                      </div>
                      <Show when={file.is_dir} fallback={
                        <Show when={file.line_count !== undefined && file.line_count !== null}>
                          <span class="text-2xs opacity-40 shrink-0 ml-2">{file.line_count}L</span>
                        </Show>
                      }>
                        <span class="text-2xs text-amber-400/80 shrink-0 ml-2 font-mono">folder</span>
                      </Show>
                    </div>
                  )}
                </For>
              </div>
            </Show>

            {/* Seamless Auto-Expanding Textarea with Tag Highlighting */}
            <div class="relative w-full">
              {/* Highlight Backdrop Mirror */}
              <div
                ref={(el) => (backdropRef = el)}
                aria-hidden="true"
                class={`absolute inset-0 pointer-events-none whitespace-pre-wrap break-words overflow-y-auto px-1 py-0.5 text-xs font-sans leading-relaxed select-none ${
                  hasTags() ? 'block' : 'hidden'
                }`}
                style={{
                  "word-break": "break-word",
                  "overflow-wrap": "break-word",
                  "scrollbar-width": "none",
                }}
              >
                <For each={tokens()}>
                  {(token) => (
                    <Show
                      when={token.isTag}
                      fallback={<span class="text-white">{token.text}</span>}
                    >
                      <span class="text-cyan-400 bg-cyan-500/20 rounded-[2px]">{token.text}</span>
                    </Show>
                  )}
                </For>
                {props.userInput.endsWith('\n') ? '\n' : ''}
              </div>

              {/* Seamless Auto-Expanding Textarea */}
              <textarea
                ref={(el) => {
                  props.setChatInputRef(el);
                  if (el) {
                    el.style.height = 'auto';
                    el.style.height = `${Math.min(el.scrollHeight, 200)}px`;
                  }
                }}
                rows={1}
                placeholder={`Send instructions to agent for workspace "${props.currentProject?.name}" (type @ to tag files)...`}
                value={props.userInput}
                onInput={(e) => {
                  const target = e.currentTarget;
                  target.style.height = 'auto';
                  target.style.height = `${Math.min(target.scrollHeight, 200)}px`;
                  if (backdropRef) {
                    backdropRef.scrollTop = target.scrollTop;
                  }
                  props.onChatInput(e);
                }}
                onScroll={(e) => {
                  if (backdropRef) {
                    backdropRef.scrollTop = e.currentTarget.scrollTop;
                  }
                }}
                onKeyDown={props.onChatKeyDown}
                class={`relative z-10 w-full bg-transparent border-0 px-1 py-0.5 text-xs focus:outline-none focus:ring-0 resize-none min-h-[38px] max-h-52 custom-scrollbar leading-relaxed font-sans ${
                  hasTags() ? 'text-transparent caret-cyan-400 selection:bg-cyan-500/30 selection:text-white' : 'text-white placeholder-white/35'
                }`}
                style={{
                  "word-break": "break-word",
                  "overflow-wrap": "break-word",
                }}
              />
            </div>

            {/* Action Bar inside the card */}
            <div class="flex items-center justify-between pt-2 border-t border-white/[0.06] mt-1">
              <div class="flex items-center gap-1.5">
                <button
                  type="button"
                  onClick={props.onOpenFilesTab}
                  class="px-2 py-1 bg-white/5 hover:bg-white/10 text-white/70 hover:text-white rounded-lg text-xs font-mono flex items-center gap-1 border border-white/5 transition-colors"
                  title="Mention project file (@file:lines)"
                >
                  <AtSign class="w-3 h-3 text-indigo-400" />
                  <span>Files</span>
                </button>
                <span class="text-2xs text-white/60 hidden sm:inline">
                  Type @ to tag files
                </span>
              </div>

              <div class="flex items-center gap-2">
                <span class="text-2xs text-white/25 hidden md:inline font-mono">
                  ↵ send • shift+↵ line
                </span>

                <Show
                  when={props.isCurrentSessionBusy}
                  fallback={
                    <button
                      type="button"
                      onClick={props.onSendMessage}
                      disabled={!props.userInput.trim()}
                      class="h-[30px] px-3.5 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-40 text-white rounded-lg text-xs font-medium flex items-center justify-center gap-1.5 transition-colors shrink-0 shadow-sm"
                    >
                      <Send class="w-3 h-3" />
                      <span>Send</span>
                    </button>
                  }
                >
                  <button
                    type="button"
                    onClick={props.onStopSession}
                    class="h-[30px] px-3.5 bg-rose-600 hover:bg-rose-500 text-white rounded-lg text-xs font-medium flex items-center justify-center gap-1.5 transition-colors shrink-0 shadow-md animate-pulse"
                    title="Stop active agent task"
                  >
                    <Square class="w-3 h-3 fill-current" />
                    <span>Stop</span>
                  </button>
                </Show>
              </div>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};
