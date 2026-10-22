import { Component, For, Show } from 'solid-js';
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
  Square 
} from 'lucide-solid';
import { Project, AgentSession, AgentMessage, WorkspaceFileItem } from '../../services';
import { MarkdownView } from './MarkdownView';
import { ToolCallCard, ToolResultCard } from './ToolCards';
import { countMessageLines } from './highlightUtils';
import { SearchableSelect } from '../SearchableSelect';

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
  return (
    <div class="bg-white/[0.03] border border-white/10 rounded-xl flex-1 flex flex-col min-h-0 overflow-hidden shadow-lg">
      {/* Session Header */}
      <div class="px-4 py-2.5 border-b border-white/10 flex flex-wrap items-center justify-between gap-2 bg-white/[0.01]">
        <div class="flex items-center gap-2 min-w-0">
          <MessageSquare class="w-4 h-4 text-indigo-400 shrink-0" />
          <div class="min-w-0 flex items-center gap-2">
            <span class="text-xs font-semibold text-white truncate">
              {props.currentSession?.title || 'Select or Create a Task Session'}
            </span>
            <Show when={props.currentSession}>
              <span class="text-[9px] uppercase px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-medium">
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
              buttonClass="flex items-center justify-between gap-1.5 text-white font-mono text-[11px] max-w-[210px] truncate hover:text-indigo-300 transition-colors"
              dropdownClass="absolute right-0 mt-2 w-72 bg-[#12161f] border border-white/10 rounded-lg shadow-2xl z-50 overflow-hidden flex flex-col max-h-80"
              searchPlaceholder="Filter AI models..."
            />
            <button
              type="button"
              onClick={props.onRefreshModels}
              disabled={props.isRefreshingModels}
              class="text-white/40 hover:text-white disabled:opacity-50 transition-colors p-0.5 ml-0.5 shrink-0"
              title="Sync latest available models"
            >
              <RefreshCw class={`w-3 h-3 ${props.isRefreshingModels ? 'animate-spin' : ''}`} />
            </button>
          </div>
        </Show>
      </div>

      {/* Messages Stream */}
      <div 
        ref={props.setMessagesContainerRef}
        onScroll={props.onMessagesScroll}
        class="flex-1 overflow-y-auto p-4 space-y-3 custom-scrollbar"
      >
        <Show when={!props.currentSession}>
          <div class="h-full flex flex-col items-center justify-center text-center text-white/40 py-12">
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
          <div class="h-full flex flex-col items-center justify-center text-center text-white/40 py-12">
            <Sparkles class="w-8 h-8 mb-2 text-indigo-400/40" />
            <p class="text-xs font-medium text-white/60">This task session is empty</p>
            <p class="text-[11px] text-white/30 max-w-xs mt-1">
              Type instructions or goals below to run the autonomous agent on the workspace folders.
            </p>
          </div>
        </Show>

        <For each={props.visibleMessages}>
          {(msg) => (
            <div class={`flex flex-col ${msg.role === 'user' ? 'items-end' : 'items-start'}`}>
              <div class="text-[10px] text-white/40 mb-1 px-1 flex items-center gap-1.5">
                <Show when={msg.role === 'user'} fallback={
                  <Show when={msg.role === 'tool'} fallback={
                    <div class="flex items-center gap-1 text-indigo-400">
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
                  <span>You</span>
                </Show>
                <span>• #{msg.sequence_num}</span>
              </div>

              {/* Tool Calls badge if present on assistant */}
              <Show when={msg.tool_calls}>
                <div class="w-full max-w-3xl min-w-0 overflow-hidden">
                  <ToolCallCard toolCalls={msg.tool_calls!} />
                </div>
              </Show>

              {/* Reasoning Thought Collapse if present */}
              <Show when={msg.thought}>
                <div class="w-full max-w-3xl mb-2 p-2.5 rounded-lg bg-amber-500/10 border border-amber-500/20 text-xs text-amber-200/90 font-mono">
                  <div class="flex items-center gap-1 font-semibold text-[10px] text-amber-400 uppercase tracking-wider mb-1">
                    <Brain class="w-3 h-3" />
                    Reasoning Process / Chain of Thought
                  </div>
                  <div class="whitespace-pre-wrap text-[11px] leading-relaxed opacity-80">{msg.thought}</div>
                </div>
              </Show>

              {/* Tool Result Bubble if role == 'tool' */}
              <Show when={msg.role === 'tool'}>
                <div class="w-full max-w-3xl min-w-0 overflow-hidden">
                  <ToolResultCard toolCallId={msg.tool_call_id} content={msg.content} />
                </div>
              </Show>

              {/* Standard Message Bubble */}
              <Show when={msg.role !== 'tool' && msg.content}>
                <div class={`flex flex-col space-y-1 w-full max-w-3xl ${msg.role === 'user' ? 'items-end' : 'items-start'}`}>
                  <Show when={msg.role === 'assistant'}>
                    <div class="flex items-center justify-between text-[10px] text-white/40 px-1">
                      <span class="font-mono flex items-center gap-1">
                        <Bot class="w-3 h-3 text-indigo-400" />
                        Generative Assistant ({countMessageLines(msg.content)} {countMessageLines(msg.content) === 1 ? 'line' : 'lines'})
                      </span>
                      <button
                        type="button"
                        onClick={() => props.toggleCollapseMessage(msg.id)}
                        class="flex items-center gap-1 px-2 py-0.5 rounded bg-white/5 hover:bg-white/10 hover:text-indigo-300 transition-colors text-[10px]"
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
                      class="cursor-pointer w-full rounded-xl p-2.5 bg-indigo-500/10 border border-indigo-500/20 text-xs text-indigo-300/90 hover:bg-indigo-500/15 transition-all flex items-center justify-between"
                    >
                      <span class="truncate italic text-[11px]">Generative output collapsed ({countMessageLines(msg.content)} {countMessageLines(msg.content) === 1 ? 'line' : 'lines'})</span>
                      <span class="text-[10px] font-semibold text-indigo-400 underline ml-2 shrink-0">Expand</span>
                    </div>
                  </Show>

                  <Show when={!props.isMessageCollapsed(msg.id)}>
                    <div
                      class={`w-fit max-w-full rounded-xl px-4 py-3 text-xs leading-relaxed ${
                        msg.role === 'user'
                          ? 'bg-indigo-600 text-white rounded-br-none whitespace-pre-wrap'
                          : 'bg-white/[0.04] border border-white/10 text-white/90 rounded-bl-none shadow-sm'
                      }`}
                    >
                      <Show
                        when={msg.role === 'assistant'}
                        fallback={<div>{msg.content || ''}</div>}
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

        {/* Live Streaming Assistant Output */}
        <Show when={props.streamingText && props.streamingText.trim().length > 0}>
          <div class="flex flex-col items-start w-full max-w-3xl">
            <div class="text-[10px] text-white/40 mb-1 px-1 flex items-center gap-1.5">
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
              <div class="text-[10px] text-white/50">
                Task is active in background. You can safely switch tasks or projects.
              </div>
            </div>
          </div>
        </Show>
      </div>

      {/* Chat Input Bar (Unified Modern Card) */}
      <Show when={props.currentSession}>
        <div class="p-3 border-t border-white/10 bg-white/[0.01]">
          <div class="relative bg-white/[0.03] hover:bg-white/[0.04] focus-within:bg-white/[0.05] border border-white/10 focus-within:border-indigo-500/60 rounded-2xl p-2.5 transition-all shadow-sm">
            {/* Autocomplete Dropdown Popup */}
            <Show when={props.autocompleteOpen && props.autocompleteSuggestions.length > 0}>
              <div class="absolute bottom-full mb-2 left-0 right-0 max-h-60 z-50 overflow-y-auto bg-slate-900/95 border border-indigo-500/30 rounded-xl shadow-2xl backdrop-blur-md p-1.5 custom-scrollbar">
                <div class="px-2 py-1 text-[10px] font-semibold text-indigo-400 border-b border-white/10 flex items-center justify-between mb-1">
                  <span class="flex items-center gap-1"><AtSign class="w-3 h-3" /> Tag Workspace File</span>
                  <span class="text-white/40 font-normal">Use ↑ ↓ then Enter / Tab</span>
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
                        <span class="text-[10px] opacity-50 truncate">@{file.relative_path}</span>
                      </div>
                      <Show when={file.is_dir} fallback={
                        <Show when={file.line_count !== undefined && file.line_count !== null}>
                          <span class="text-[10px] opacity-40 shrink-0 ml-2">{file.line_count}L</span>
                        </Show>
                      }>
                        <span class="text-[10px] text-amber-400/80 shrink-0 ml-2 font-mono">folder</span>
                      </Show>
                    </div>
                  )}
                </For>
              </div>
            </Show>

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
                props.onChatInput(e);
              }}
              onKeyDown={props.onChatKeyDown}
              class="w-full bg-transparent border-0 px-1 py-0.5 text-xs text-white placeholder-white/35 focus:outline-none focus:ring-0 resize-none min-h-[38px] max-h-52 custom-scrollbar leading-relaxed"
            />

            {/* Action Bar inside the card */}
            <div class="flex items-center justify-between pt-2 border-t border-white/[0.06] mt-1">
              <div class="flex items-center gap-1.5">
                <button
                  type="button"
                  onClick={props.onOpenFilesTab}
                  class="px-2 py-1 bg-white/5 hover:bg-white/10 text-white/70 hover:text-white rounded-lg text-[11px] font-mono flex items-center gap-1 border border-white/5 transition-colors"
                  title="Mention project file (@file:lines)"
                >
                  <AtSign class="w-3 h-3 text-indigo-400" />
                  <span>Files</span>
                </button>
                <span class="text-[10px] text-white/30 hidden sm:inline">
                  Type @ to tag files
                </span>
              </div>

              <div class="flex items-center gap-2">
                <span class="text-[10px] text-white/25 hidden md:inline font-mono">
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
