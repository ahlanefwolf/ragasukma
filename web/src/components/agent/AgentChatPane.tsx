import { Component, For, Show, createMemo, createSignal, createEffect, onCleanup } from 'solid-js';
import {
  MessageSquare,
  Cpu,
  RefreshCw,
  ChevronDown,
  AtSign,
  FileCode,
  Folder,
  Send,
  Square,
  FolderGit2,
  ExternalLink,
  Download,
  Copy,
  Check,
  FileText,
  X,
  Zap,
  ArrowDownToLine,
  ArrowUpToLine,
} from 'lucide-solid';
import { Project, AgentSession, AgentMessage, WorkspaceFileItem } from '../../services';
import { MarkdownView } from './MarkdownView';
import { ToolCallCard, ToolResultCard, GitDiffViewer, parseToolCalls, formatToolOutput } from './ToolCards';
import { countMessageLines } from './highlightUtils';
import { SearchableSelect } from '../SearchableSelect';
import { MessageList } from './MessageList';

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
  hasMoreMessages?: boolean;
  loadingEarlier?: boolean;
  onLoadOlderMessages?: () => Promise<boolean>;
  onLoadAllMessages?: () => Promise<boolean>;
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
  /** Lazily-known line count for a path (undefined until the file is opened). */
  getLineCount: (path: string) => number | undefined;
  setChatInputRef: (el: HTMLTextAreaElement) => void;
  userInput: string;
  onChatInput: (e: InputEvent & { currentTarget: HTMLTextAreaElement }) => void;
  onChatKeyDown: (e: KeyboardEvent & { currentTarget: HTMLTextAreaElement }) => void;
  onSendMessage: () => void;
  onStopSession: () => void;
  sendShortcut?: 'enter' | 'ctrl_enter';
  onToggleSendShortcut?: () => void;
  showToast?: (message: string) => void;
  onOpenFileInCodePanel?: (filePath: string) => void;
}> = (props) => {
  // Session Export & Copying State
  const [exportMenuOpen, setExportMenuOpen] = createSignal(false);
  const [copiedTranscript, setCopiedTranscript] = createSignal(false);

  // Calculate total session token usage
  const sessionTokenStats = createMemo(() => {
    const msgs = props.visibleMessages || [];
    let runningIn = 0;
    let runningOut = 0;
    for (const m of msgs) {
      if (m.token_usage_input) runningIn += m.token_usage_input;
      if (m.token_usage_output) runningOut += m.token_usage_output;
    }
    const sessIn = props.currentSession?.total_tokens_input || 0;
    const sessOut = props.currentSession?.total_tokens_output || 0;
    const finalIn = Math.max(runningIn, sessIn);
    const finalOut = Math.max(runningOut, sessOut);
    return {
      totalIn: finalIn,
      totalOut: finalOut,
      total: finalIn + finalOut,
    };
  });

  createEffect(() => {
    if (!exportMenuOpen()) return;
    const close = () => setExportMenuOpen(false);
    window.addEventListener('click', close);
    onCleanup(() => window.removeEventListener('click', close));
  });

  const generateMarkdownTranscript = () => {
    const session = props.currentSession;
    const msgs = props.visibleMessages || [];
    const title = session?.title || 'Task Session';
    const model = session?.model || 'Unknown Model';
    const dateStr = session?.created_at ? new Date(session.created_at).toLocaleString() : 'Unknown Date';

    const lines: string[] = [];
    lines.push(`# ${title}`);
    lines.push('');
    lines.push(`**Model:** ${model}`);
    lines.push(`**Created:** ${dateStr}`);
    lines.push(`**Status:** ${session?.status || 'Unknown'}`);
    lines.push('');

    for (const msg of msgs) {
      const roleLabel = msg.role === 'user' ? '**User**' : msg.role === 'assistant' ? '**Agent**' : '**Tool**';
      lines.push(`${roleLabel}:`);

      if (msg.thought) {
        lines.push('> **Thought Process:**');
        lines.push(`> ${msg.thought.split('\n').join('\n> ')}`);
        lines.push('');
      }

      if (msg.tool_calls) {
        lines.push('> **Tool Calls:**');
        const calls = parseToolCalls(msg.tool_calls);
        for (const call of calls) {
          lines.push(`> - ${call.name}`);
        }
        lines.push('');
      }

      if (msg.content) {
        lines.push(msg.content);
      }

      lines.push('');
    }

    const calls = props.visibleMessages.filter((m) => m.tool_calls).length;
    const inTokens = sessionTokenStats().totalIn;
    const outTokens = sessionTokenStats().totalOut;

    lines.push('---');
    lines.push('');
    lines.push(`**Summary:**`);
    const formatted = `${(inTokens + outTokens).toLocaleString()} tokens (${inTokens.toLocaleString()} in, ${outTokens.toLocaleString()} out), ${calls} tool call(s)`;
    lines.push(formatted);

    return lines.join('\n');
  };

  const sanitizeFilename = (name: string) => name.replace(/[^a-z0-9_-]/gi, '_').toLowerCase();

  const handleCopyTranscript = () => {
    const md = generateMarkdownTranscript();
    navigator.clipboard.writeText(md).then(() => {
      setCopiedTranscript(true);
      setTimeout(() => setCopiedTranscript(false), 2000);
      props.showToast?.('Transcript copied to clipboard');
    });
  };

  const handleExportMarkdown = () => {
    const md = generateMarkdownTranscript();
    const blob = new Blob([md], { type: 'text/markdown;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    const baseName = sanitizeFilename(props.currentSession?.title || 'task');
    a.href = url;
    a.download = `${baseName}_transcript.md`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    props.showToast?.('Markdown file exported');
  };

  const handleExportJson = () => {
    const data = {
      project: props.currentProject?.name,
      session: {
        title: props.currentSession?.title,
        model: props.currentSession?.model,
        status: props.currentSession?.status,
      },
      messages: props.visibleMessages,
      exportedAt: new Date().toISOString(),
    };
    const jsonStr = JSON.stringify(data, null, 2);
    const blob = new Blob([jsonStr], { type: 'application/json;charset=utf-8' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    const baseName = sanitizeFilename(props.currentSession?.title || 'task');
    a.href = url;
    a.download = `${baseName}_session.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
    props.showToast?.('JSON file exported');
  };

  // Session Changes Review Panel State
  const [showChangesPanel, setShowChangesPanel] = createSignal(false);
  const [selectedDiffFile, setSelectedDiffFile] = createSignal<{
    filePath: string;
    fileName: string;
    status: 'created' | 'modified';
    diffData?: { oldContent: string; newContent: string; isNewFile: boolean };
  } | null>(null);

  const sessionModifiedFiles = createMemo(() => {
    const map = new Map<
      string,
      {
        filePath: string;
        fileName: string;
        status: 'created' | 'modified';
        diffData?: { oldContent: string; newContent: string; isNewFile: boolean };
      }
    >();

    const msgs = props.visibleMessages || [];
    for (const m of msgs) {
      const calls = parseToolCalls(m.tool_calls);
      for (const call of calls) {
        const fp = call.argsObj?.filepath || call.argsObj?.path;
        if (fp) {
          const fileName = fp.split('/').pop() || fp;
          if (!map.has(fp)) {
            map.set(fp, {
              filePath: fp,
              fileName,
              status: 'modified',
            });
          }
        }
      }
    }

    for (const m of msgs) {
      if (m.role !== 'assistant') continue;
      const out = formatToolOutput(m.content);
      if (out.diffData && out.filename) {
        for (const [key, val] of map.entries()) {
          if (val.fileName === out.filename || val.filePath.endsWith(out.filename)) {
            val.diffData = out.diffData;
            val.status = out.diffData.isNewFile ? 'created' : 'modified';
            break;
          }
        }
      }
    }

    return Array.from(map.values());
  });

  return (
    <div class="bg-white/[0.03] border border-white/10 flex-1 flex flex-col min-h-0 overflow-hidden shadow-lg relative">
      {/* Session Header */}
      <div class="px-4 py-2.5 border-b border-white/10 flex flex-wrap items-center justify-between gap-2 bg-white/[0.01]">
        <div class="flex items-center gap-2 min-w-0">
          <MessageSquare class="w-4 h-4 text-gi-primary shrink-0" />
          <div class="min-w-0 flex items-center gap-2">
            <span class="text-xs font-semibold text-white truncate">
              {props.currentSession?.title || 'Select or Create a Task Session'}
            </span>
            <Show when={props.currentSession}>
              <span class="text-2xs uppercase px-1.5 py-0.5 bg-gi-status-success/10 text-gi-status-success border border-gi-status-success/20 font-medium">
                {props.currentSession?.status}
              </span>
            </Show>
            <Show when={sessionTokenStats().total > 0}>
              <div
                class="flex items-center gap-1.5 px-2 py-0.5 text-2xs font-mono font-medium bg-white/5 border border-white/10 text-theme-text/80 select-none shadow-sm"
                title={`Task Token Telemetry:\n• Total: ${sessionTokenStats().total.toLocaleString()} tokens\n• Input: ${sessionTokenStats().totalIn.toLocaleString()}\n• Output: ${sessionTokenStats().totalOut.toLocaleString()}`}
              >
                <Zap class="w-3 h-3 text-gi-status-warning shrink-0" />
                <span class="text-gi-status-warning font-semibold">{sessionTokenStats().total.toLocaleString()}</span>
                <span class="text-theme-text/40">tokens</span>
                <span class="text-theme-text/25">•</span>
                <span class="text-gi-status-info/80 flex items-center gap-0.5">
                  <ArrowDownToLine class="w-2.5 h-2.5 shrink-0" />
                  <span>{sessionTokenStats().totalIn.toLocaleString()} in</span>
                </span>
                <span class="text-gi-status-success/80 flex items-center gap-0.5">
                  <ArrowUpToLine class="w-2.5 h-2.5 shrink-0" />
                  <span>{sessionTokenStats().totalOut.toLocaleString()} out</span>
                </span>
              </div>
            </Show>
            <Show when={sessionModifiedFiles().length > 0}>
              <button
                type="button"
                onClick={() => setShowChangesPanel(!showChangesPanel())}
                class={`flex items-center gap-1 px-2 py-0.5 text-2xs font-mono font-medium transition-colors border ${
                  showChangesPanel()
                    ? 'bg-gi-status-warning/20 text-gi-status-warning border-gi-status-warning/40 shadow-sm'
                    : 'bg-white/5 hover:bg-white/10 text-white/80 border-white/10'
                }`}
                title="View all files modified in this task session"
              >
                <FolderGit2 class="w-3 h-3 text-gi-status-warning" />
                <span>{sessionModifiedFiles().length} modified</span>
              </button>
            </Show>
          </div>
        </div>

        {/* Model Switcher & Session Controls */}
        <Show when={props.currentSession}>
          <div class="flex items-center gap-1.5 bg-white/5 border border-white/10 px-2 py-1 text-xs">
            <Cpu class="w-3.5 h-3.5 text-gi-primary shrink-0" />
            <SearchableSelect
              value={props.activeSessionModel || props.currentSession?.model || props.defaultBrainModel}
              placeholder={props.isRefreshingModels ? 'Loading models...' : 'No models available'}
              onChange={(val) => props.setActiveSessionModel(val)}
              options={[
                ...(props.isRefreshingModels && props.availableModels.length === 0
                  ? [{ value: '', label: 'Loading models...' }]
                  : props.availableModels.length === 0
                  ? [{ value: '', label: 'No models available' }]
                  : []),
                ...props.availableModels.map((m) => ({
                  value: m.id,
                  label: m.label || m.id,
                  sublabel: m.id.startsWith('claude-')
                    ? 'Anthropic'
                    : m.id.startsWith('gpt-')
                    ? 'OpenAI'
                    : m.id.startsWith('gemini-')
                    ? 'Google'
                    : m.id.startsWith('deepseek')
                    ? 'DeepSeek'
                    : m.id.startsWith('cx/')
                    ? 'Codex'
                    : undefined,
                })),
              ]}
              buttonClass="flex items-center justify-between gap-1.5 text-white font-mono text-xs max-w-[210px] truncate hover:text-gi-primary-hover transition-colors"
              dropdownClass="absolute right-0 mt-2 w-72 bg-theme-surface border border-theme-border shadow-2xl z-50 overflow-hidden flex flex-col max-h-80"
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

          {/* Session Export Menu */}
          <div class="relative">
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                setExportMenuOpen(!exportMenuOpen());
              }}
              class="flex items-center gap-1.5 bg-white/5 hover:bg-white/10 border border-white/10 px-2 py-1 text-xs text-white/80 hover:text-white transition-colors"
              title="Export or copy session transcript"
            >
              <Download class="w-3.5 h-3.5 text-gi-primary" />
              <span class="hidden sm:inline">Export</span>
              <ChevronDown class={`w-3 h-3 text-white/50 transition-transform ${exportMenuOpen() ? 'rotate-180' : ''}`} />
            </button>

            <Show when={exportMenuOpen()}>
              <div
                class="absolute right-0 mt-2 w-52 bg-theme-surface border border-theme-border shadow-2xl z-50 p-1.5 space-y-1 text-xs"
                onClick={(e) => e.stopPropagation()}
              >
                <button
                  type="button"
                  onClick={handleCopyTranscript}
                  class="w-full flex items-center gap-2 px-2.5 py-1.5 hover:bg-white/10 text-white/80 hover:text-white transition-colors text-left"
                >
                  <Show when={copiedTranscript()} fallback={<Copy class="w-3.5 h-3.5 text-gi-primary shrink-0" />}>
                    <Check class="w-3.5 h-3.5 text-gi-status-success shrink-0" />
                  </Show>
                  <div class="flex flex-col min-w-0">
                    <span class="font-medium text-white">{copiedTranscript() ? 'Copied!' : 'Copy Transcript'}</span>
                    <span class="text-2xs text-white/40">Markdown to clipboard</span>
                  </div>
                </button>

                <button
                  type="button"
                  onClick={handleExportMarkdown}
                  class="w-full flex items-center gap-2 px-2.5 py-1.5 hover:bg-white/10 text-white/80 hover:text-white transition-colors text-left"
                >
                  <FileText class="w-3.5 h-3.5 text-gi-status-warning shrink-0" />
                  <div class="flex flex-col min-w-0">
                    <span class="font-medium text-white">Export Markdown</span>
                    <span class="text-2xs text-white/40">Download .md file</span>
                  </div>
                </button>

                <button
                  type="button"
                  onClick={handleExportJson}
                  class="w-full flex items-center gap-2 px-2.5 py-1.5 hover:bg-white/10 text-white/80 hover:text-white transition-colors text-left"
                >
                  <FileCode class="w-3.5 h-3.5 text-gi-status-info shrink-0" />
                  <div class="flex flex-col min-w-0">
                    <span class="font-medium text-white">Export JSON</span>
                    <span class="text-2xs text-white/40">Download full session log</span>
                  </div>
                </button>
              </div>
            </Show>
          </div>
        </Show>
      </div>

      {/* Session Changes Review Panel (Collapsible Drawer) */}
      <Show when={showChangesPanel() && sessionModifiedFiles().length > 0}>
        <div class="border-b border-theme-border bg-theme-bg p-3 space-y-2.5 shadow-md">
          <div class="flex items-center justify-between">
            <div class="flex items-center gap-2">
              <FolderGit2 class="w-4 h-4 text-gi-status-warning" />
              <span class="text-xs font-semibold text-white">Session Changes Review</span>
              <span class="text-2xs text-white/50">({sessionModifiedFiles().length} files touched)</span>
            </div>
            <button
              onClick={() => {
                setShowChangesPanel(false);
                setSelectedDiffFile(null);
              }}
              class="text-white/50 hover:text-white p-0.5"
            >
              <X class="w-3.5 h-3.5" />
            </button>
          </div>

          {/* File list pills */}
          <div class="flex flex-wrap gap-1.5">
            <For each={sessionModifiedFiles()}>
              {(file) => {
                const isSelected = () => selectedDiffFile()?.filePath === file.filePath;
                return (
                  <div class="flex items-center gap-1 bg-white/5 border border-white/10 p-1 text-xs">
                    <button
                      type="button"
                      onClick={() => setSelectedDiffFile(isSelected() ? null : file)}
                      class={`px-2 py-0.5 text-xs font-mono flex items-center gap-1.5 transition-colors ${
                        isSelected() ? 'bg-gi-primary text-gi-primary-foreground font-bold' : 'text-white/80 hover:text-white hover:bg-white/10'
                      }`}
                    >
                      <FileCode class="w-3.5 h-3.5 text-gi-primary" />
                      <span>{file.fileName}</span>
                      <span class={`text-2xs px-1 font-sans uppercase font-bold ${
                        file.status === 'created' ? 'bg-gi-status-success/20 text-gi-status-success' : 'bg-gi-status-warning/20 text-gi-status-warning'
                      }`}>
                        {file.status}
                      </span>
                    </button>
                    <Show when={props.onOpenFileInCodePanel}>
                      <button
                        type="button"
                        onClick={() => props.onOpenFileInCodePanel?.(file.filePath)}
                        class="p-1 text-white/50 hover:text-gi-primary-hover"
                        title="Open in Right Docked Code Viewer"
                      >
                        <ExternalLink class="w-3 h-3" />
                      </button>
                    </Show>
                  </div>
                );
              }}
            </For>
          </div>

          {/* Inline Diff Viewer */}
          <Show when={selectedDiffFile() && selectedDiffFile()?.diffData}>
            <div class="mt-2 border-t border-white/10 pt-2">
              <GitDiffViewer
                oldContent={selectedDiffFile()!.diffData!.oldContent}
                newContent={selectedDiffFile()!.diffData!.newContent}
                filename={selectedDiffFile()!.fileName}
                isNewFile={selectedDiffFile()!.diffData!.isNewFile}
              />
            </div>
          </Show>
        </div>
      </Show>

      {/* Messages Stream - delegated to MessageList component */}
      <div class="relative flex-1 flex flex-col min-h-0">
        <MessageList
          currentSession={props.currentSession}
          visibleMessages={props.visibleMessages}
          loadingMessages={props.loadingMessages}
          hasMoreMessages={props.hasMoreMessages}
          loadingEarlier={props.loadingEarlier}
          onLoadOlderMessages={props.onLoadOlderMessages}
          onLoadAllMessages={props.onLoadAllMessages}
          isMessageCollapsed={props.isMessageCollapsed}
          toggleCollapseMessage={props.toggleCollapseMessage}
          streamingText={props.streamingText}
          streamingThought={props.streamingThought}
          isCurrentSessionBusy={props.isCurrentSessionBusy}
          currentStepInfo={props.currentStepInfo}
          onQuickCreateSession={props.onQuickCreateSession}
          onOpenFileInCodePanel={props.onOpenFileInCodePanel}
          setMessagesContainerRef={props.setMessagesContainerRef}
          onMessagesScroll={props.onMessagesScroll}
        />
      </div>

      {/* Chat Input Bar (Unified Modern Card) */}
      <Show when={props.currentSession}>
        <div class="p-3 border-t border-white/10 bg-white/[0.01]">
          <div class="relative bg-white/[0.03] hover:bg-white/[0.04] focus-within:bg-white/[0.05] border border-white/10 focus-within:border-gi-primary/60 p-2.5 transition-all shadow-sm">
            {/* Autocomplete Dropdown Popup */}
            <Show when={props.autocompleteOpen && props.autocompleteSuggestions.length > 0}>
              <div class="absolute bottom-full mb-2 left-0 right-0 max-h-60 z-50 overflow-y-auto bg-theme-surface/95 border border-theme-border shadow-2xl backdrop-blur-md p-1.5 custom-scrollbar">
                <div class="px-2 py-1 text-2xs font-semibold text-gi-primary border-b border-theme-border flex items-center justify-between mb-1">
                  <span class="flex items-center gap-1"><AtSign class="w-3 h-3" /> Tag Workspace File</span>
                  <span class="text-white/70 font-normal">Use ↑ ↓ then Enter / Tab</span>
                </div>
                <For each={props.autocompleteSuggestions}>
                  {(file, idx) => (
                    <div
                      class={`px-2.5 py-1.5 text-xs font-mono flex items-center justify-between cursor-pointer transition-colors ${
                        idx() === props.autocompleteIndex
                          ? 'bg-gi-primary text-gi-primary-foreground font-semibold shadow-sm'
                          : 'hover:bg-white/10 text-white/80'
                      }`}
                      onClick={() => props.onSelectAutocompleteFile(file)}
                    >
                      <div class="flex items-center gap-2 min-w-0">
                        <Show when={file.is_dir} fallback={<FileCode class="w-3.5 h-3.5 shrink-0 opacity-70" />}>
                          <Folder class="w-3.5 h-3.5 text-gi-status-warning shrink-0" />
                        </Show>
                        <span class="font-semibold truncate">{file.name}{file.is_dir ? '/' : ''}</span>
                        <span class="text-2xs opacity-50 truncate">@{file.relative_path}</span>
                      </div>
                      <Show when={file.is_dir} fallback={
                        <Show when={props.getLineCount(file.relative_path) !== undefined}>
                          <span class="text-2xs opacity-40 shrink-0 ml-2">
                            {props.getLineCount(file.relative_path)}L
                          </span>
                        </Show>
                      }>
                        <span class="text-2xs text-gi-status-warning/80 shrink-0 ml-2 font-mono">folder</span>
                      </Show>
                    </div>
                  )}
                </For>
              </div>
            </Show>

            {/* Auto-Expanding Textarea */}
            <div class="relative w-full">
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
                  props.onChatInput(e);
                }}
                onKeyDown={props.onChatKeyDown}
                class="relative z-10 w-full bg-transparent border-0 px-1 py-0.5 text-xs focus:outline-none focus:ring-0 resize-none min-h-[38px] max-h-52 custom-scrollbar leading-relaxed font-sans text-white caret-white placeholder-white/35 selection:bg-gi-primary/30 selection:text-white"
                style={{
                  "word-break": "break-word",
                  "overflow-wrap": "break-word",
                  "background-color": "transparent",
                  "-webkit-font-smoothing": "antialiased",
                  "-moz-osx-font-smoothing": "grayscale",
                  "font-variant-ligatures": "none",
                  "text-rendering": "optimizeLegibility",
                }}
              />
            </div>

            {/* Action Bar inside the card */}
            <div class="flex items-center justify-between pt-2 border-t border-white/[0.06] mt-1">
              <div class="flex items-center gap-1.5">
                <button
                  type="button"
                  onClick={props.onOpenFilesTab}
                  class="px-2 py-1 bg-white/5 hover:bg-white/10 text-white/70 hover:text-white text-xs font-mono flex items-center gap-1 border border-white/5 transition-colors"
                  title="Mention project file (@file:lines)"
                >
                  <AtSign class="w-3 h-3 text-gi-primary" />
                  <span>Files</span>
                </button>
                <span class="text-2xs text-white/60 hidden sm:inline">
                  Type @ to tag files
                </span>
              </div>

              <div class="flex items-center gap-2">
                <button
                  type="button"
                  onClick={props.onToggleSendShortcut}
                  class="text-2xs text-white/40 hover:text-white/80 transition-colors font-mono px-2 py-0.5 hover:bg-white/5 border border-transparent hover:border-white/10 hidden md:flex items-center gap-1 cursor-pointer select-none"
                  title="Click to toggle send shortcut (Enter vs Ctrl+Enter)"
                >
                  <span class="text-white/70 font-semibold">{props.sendShortcut === 'ctrl_enter' ? 'Ctrl+↵' : '↵'} send</span>
                  <span class="opacity-40">• {props.sendShortcut === 'ctrl_enter' ? '↵' : 'shift+↵'} line</span>
                </button>

                <Show
                  when={props.isCurrentSessionBusy}
                  fallback={
                    <button
                      type="button"
                      onClick={props.onSendMessage}
                      disabled={!props.userInput.trim()}
                      class="h-[30px] px-3.5 bg-gi-primary hover:bg-gi-primary-hover disabled:opacity-40 text-gi-primary-foreground text-xs font-semibold flex items-center justify-center gap-1.5 transition-colors shrink-0 shadow-sm"
                    >
                      <Send class="w-3 h-3" />
                      <span>Send</span>
                    </button>
                  }
                >
                  <button
                    type="button"
                    onClick={props.onStopSession}
                    class="h-[30px] px-3.5 bg-gi-status-danger hover:bg-gi-status-danger/90 text-white text-xs font-medium flex items-center justify-center gap-1.5 transition-colors shrink-0 shadow-md animate-pulse"
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
