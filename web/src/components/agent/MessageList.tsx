import { Component, For, Show, createEffect, createSignal } from 'solid-js';
import {
  Bot,
  Sparkles,
  Plus,
  ChevronUp,
  ChevronDown,
  History,
  Brain,
  Loader2,
  ArrowDown,
  Wrench,
} from 'lucide-solid';
import { AgentMessage, AgentSession } from '../../services';
import { MarkdownView } from './MarkdownView';
import { ToolCallCard, ToolResultCard } from './ToolCards';
import { countMessageLines } from './highlightUtils';
import { ThoughtBubble, UserMessageBody } from './MessageBubbles';

export interface MessageListProps {
  currentSession: AgentSession | null | undefined;
  visibleMessages: AgentMessage[];
  loadingMessages: boolean;
  hasMoreMessages?: boolean;
  loadingEarlier?: boolean;
  onLoadOlderMessages?: () => Promise<boolean>;
  onLoadAllMessages?: () => Promise<boolean>;
  isMessageCollapsed: (id: string) => boolean;
  toggleCollapseMessage: (id: string) => void;
  streamingText?: string;
  streamingThought?: string;
  isCurrentSessionBusy: boolean;
  currentStepInfo: string | null;
  onQuickCreateSession: () => void;
  onOpenFileInCodePanel?: (filePath: string) => void;
  setMessagesContainerRef: (el: HTMLDivElement) => void;
  onMessagesScroll: () => void;
}

export const MessageList: Component<MessageListProps> = (props) => {
  const [isScrolledUp, setIsScrolledUp] = createSignal(false);
  let messagesContainerEl: HTMLDivElement | undefined;
  let pinnedSessionId: string | null = null;
  let lastSeenBottomMsgId: string | null = null;
  let isLoadingEarlier = false;

  const pinToBottom = () => {
    if (!messagesContainerEl) return;
    messagesContainerEl.scrollTop = messagesContainerEl.scrollHeight;
    setIsScrolledUp(false);
  };

  // Ensure scroll is pinned to the bottom ONLY on initial load / session switch
  createEffect(() => {
    const sId = props.currentSession?.id;
    const loading = props.loadingMessages;
    const count = props.visibleMessages?.length ?? 0;

    if (sId && !loading && count > 0) {
      if (pinnedSessionId !== sId) {
        pinnedSessionId = sId;
        const msgs = props.visibleMessages || [];
        lastSeenBottomMsgId = msgs[msgs.length - 1]?.id ?? null;

        pinToBottom();
        requestAnimationFrame(() => {
          pinToBottom();
          requestAnimationFrame(pinToBottom);
        });
        setTimeout(pinToBottom, 50);
        setTimeout(pinToBottom, 150);
        setTimeout(pinToBottom, 350);
      }
    } else if (!sId) {
      pinnedSessionId = null;
      lastSeenBottomMsgId = null;
    }
  });

  // Auto-scroll only when a NEW message arrives at the bottom (not when older messages are prepended)
  createEffect(() => {
    const msgs = props.visibleMessages || [];
    if (msgs.length === 0) return;
    const latestId = msgs[msgs.length - 1]?.id;
    if (latestId && pinnedSessionId === props.currentSession?.id) {
      if (lastSeenBottomMsgId && latestId !== lastSeenBottomMsgId) {
        lastSeenBottomMsgId = latestId;
        if (!isScrolledUp()) {
          scrollToBottom();
        }
      } else if (!lastSeenBottomMsgId) {
        lastSeenBottomMsgId = latestId;
      }
    }
  });

  // Auto-scroll as streaming text/thought flows in (unless user intentionally scrolled up)
  createEffect(() => {
    const text = props.streamingText;
    const thought = props.streamingThought;
    if ((text || thought) && !isScrolledUp()) {
      if (messagesContainerEl) {
        messagesContainerEl.scrollTop = messagesContainerEl.scrollHeight;
      }
    }
  });

  const handleLoadEarlier = async () => {
    if (isLoadingEarlier || !props.onLoadOlderMessages || props.loadingEarlier || !props.hasMoreMessages) return;
    if (!messagesContainerEl) {
      await props.onLoadOlderMessages();
      return;
    }

    isLoadingEarlier = true;
    const prevScrollHeight = messagesContainerEl.scrollHeight;
    const prevScrollTop = messagesContainerEl.scrollTop;

    try {
      const ok = await props.onLoadOlderMessages();
      if (ok) {
        requestAnimationFrame(() => {
          requestAnimationFrame(() => {
            if (messagesContainerEl) {
              const heightDiff = messagesContainerEl.scrollHeight - prevScrollHeight;
              messagesContainerEl.scrollTop = prevScrollTop + heightDiff;
              setIsScrolledUp(true);
            }
            isLoadingEarlier = false;
          });
        });
      } else {
        isLoadingEarlier = false;
      }
    } catch {
      isLoadingEarlier = false;
    }
  };

  const handleLoadAll = async () => {
    if (isLoadingEarlier || !props.onLoadAllMessages || props.loadingEarlier) return;
    if (!messagesContainerEl) {
      await props.onLoadAllMessages();
      return;
    }

    isLoadingEarlier = true;
    const prevScrollHeight = messagesContainerEl.scrollHeight;
    const prevScrollTop = messagesContainerEl.scrollTop;

    try {
      const ok = await props.onLoadAllMessages();
      if (ok) {
        requestAnimationFrame(() => {
          requestAnimationFrame(() => {
            if (messagesContainerEl) {
              const heightDiff = messagesContainerEl.scrollHeight - prevScrollHeight;
              messagesContainerEl.scrollTop = prevScrollTop + heightDiff;
              setIsScrolledUp(true);
            }
            isLoadingEarlier = false;
          });
        });
      } else {
        isLoadingEarlier = false;
      }
    } catch {
      isLoadingEarlier = false;
    }
  };

  const handleScroll = (e: Event) => {
    const el = e.currentTarget as HTMLDivElement;
    const distanceToBottom = el.scrollHeight - (el.scrollTop + el.clientHeight);
    setIsScrolledUp(distanceToBottom > 150);
    props.onMessagesScroll();

    // Auto lazy-load earlier messages when scrolled near top
    if (el.scrollTop <= 80 && props.hasMoreMessages && !props.loadingEarlier) {
      handleLoadEarlier();
    }
  };

  const scrollToBottom = (smooth = true) => {
    if (messagesContainerEl) {
      if (smooth) {
        messagesContainerEl.scrollTo({ top: messagesContainerEl.scrollHeight, behavior: 'smooth' });
      } else {
        messagesContainerEl.scrollTop = messagesContainerEl.scrollHeight;
      }
    }
  };

  return (
    <>
      <div
        ref={(el) => {
          messagesContainerEl = el;
          props.setMessagesContainerRef(el);
        }}
        onScroll={handleScroll}
        class="flex-1 overflow-y-auto overflow-x-hidden p-4 space-y-3 custom-scrollbar min-w-0 w-full"
      >
        {/* Upward Lazy Load Banner for Earlier Messages */}
        <Show when={props.currentSession && props.hasMoreMessages}>
          <div class="flex items-center justify-center gap-2 py-2 mb-1 border-b border-white/[0.06] bg-white/[0.02] select-none">
            <button
              type="button"
              onClick={handleLoadEarlier}
              disabled={props.loadingEarlier}
              class="px-3 py-1.5 bg-white/5 hover:bg-white/10 active:scale-95 text-white/80 hover:text-white text-xs flex items-center gap-1.5 border border-white/10 transition-colors shadow-sm font-sans disabled:opacity-50"
            >
              <History class={`w-3.5 h-3.5 text-gi-primary ${props.loadingEarlier ? 'animate-spin' : ''}`} />
              <span>{props.loadingEarlier ? 'Loading earlier messages...' : 'Load 25 Earlier Messages'}</span>
            </button>
            <Show when={props.onLoadAllMessages}>
              <button
                type="button"
                onClick={handleLoadAll}
                disabled={props.loadingEarlier}
                class="px-2.5 py-1.5 bg-white/5 hover:bg-white/10 active:scale-95 text-white/50 hover:text-white/80 text-xs transition-colors font-sans disabled:opacity-50"
                title="Load all earlier messages in this session"
              >
                Show All
              </button>
            </Show>
          </div>
        </Show>

        <Show when={!props.currentSession}>
          <div class="h-full flex flex-col items-center justify-center text-center text-white/70 py-12">
            <Bot class="w-10 h-10 mb-2 text-white/20" />
            <p class="text-xs">Select a task session on the left or create a new session to start the agent.</p>
            <button
              onClick={props.onQuickCreateSession}
              class="mt-3 px-3.5 py-1.5 text-xs bg-gi-primary hover:bg-gi-primary-hover text-gi-primary-foreground font-semibold flex items-center gap-1.5 transition-colors"
            >
              <Plus class="w-3.5 h-3.5" />
              New Task
            </button>
          </div>
        </Show>

        <Show when={props.currentSession && props.visibleMessages.length === 0 && !props.loadingMessages}>
          <div class="h-full flex flex-col items-center justify-center text-center text-white/70 py-12">
            <Sparkles class="w-8 h-8 mb-2 text-gi-primary/40" />
            <p class="text-xs font-medium text-white/60">This task session is empty</p>
            <p class="text-xs text-white/60 max-w-xs mt-1">
              Type instructions or goals below to run the autonomous agent on the workspace folders.
            </p>
          </div>
        </Show>

        <For each={props.visibleMessages || []}>
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
                    <div class="flex items-center gap-1 text-gi-status-success">
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
                        class="flex items-center gap-1 px-2 py-0.5 bg-theme-bg/60 hover:bg-theme-card border border-theme-border/60 hover:text-theme-cyan transition-colors text-2xs"
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
                      class="cursor-pointer w-full p-2.5 bg-theme-cyan/10 border border-theme-cyan/20 text-xs text-theme-cyan/90 hover:bg-theme-cyan/15 transition-all flex items-center justify-between"
                    >
                      <span class="truncate italic text-xs">Generative output collapsed ({countMessageLines(msg.content)} {countMessageLines(msg.content) === 1 ? 'line' : 'lines'})</span>
                      <span class="text-2xs font-semibold text-theme-cyan underline ml-2 shrink-0">Expand</span>
                    </div>
                  </Show>

                  <Show when={!props.isMessageCollapsed(msg.id)}>
                    <div
                      class={`w-fit max-w-full px-4 py-3 text-xs leading-relaxed ${
                        msg.role === 'user'
                          ? 'bg-theme-card border border-theme-border text-theme-text whitespace-pre-wrap shadow-sm'
                          : 'bg-theme-bg/80 border border-theme-border text-theme-text/90 shadow-sm'
                      }`}
                    >
                      <Show
                        when={msg.role === 'assistant'}
                        fallback={
                          <UserMessageBody
                            content={msg.content || ''}
                            onOpenFile={props.onOpenFileInCodePanel}
                          />
                        }
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
            <div class="text-2xs text-gi-status-warning mb-1 px-1 flex items-center gap-1.5 font-semibold uppercase tracking-wider">
              <Brain class="w-3.5 h-3.5 text-gi-status-warning shrink-0 animate-pulse" />
              <span>Reasoning (Live...)</span>
            </div>
            <div class="w-full max-w-full px-4 py-3 text-xs leading-relaxed bg-gi-status-warning/10 border border-gi-status-warning/25 text-theme-text/90 font-mono whitespace-pre-wrap max-h-56 overflow-y-auto custom-scrollbar shadow-sm">
              {props.streamingThought}
              <span class="inline-block w-1.5 h-3.5 bg-gi-status-warning animate-pulse ml-1 align-middle" />
            </div>
          </div>
        </Show>

        {/* Live Streaming Assistant Output */}
        <Show when={props.streamingText && props.streamingText.trim().length > 0}>
          <div class="flex flex-col items-start w-full max-w-full">
            <div class="text-2xs text-white/70 mb-1 px-1 flex items-center gap-1.5">
              <div class="flex items-center gap-1 text-gi-primary">
                <Bot class="w-3 h-3 animate-pulse" />
                <span class="font-medium">Agent (Generating response...)</span>
              </div>
            </div>
            <div class="w-fit max-w-full px-4 py-3 text-xs leading-relaxed bg-white/[0.04] border border-gi-primary/30 text-white/90 shadow-md">
              <MarkdownView content={props.streamingText || ''} />
              <span class="inline-block w-1.5 h-3.5 bg-gi-primary animate-pulse ml-1 align-middle" />
            </div>
          </div>
        </Show>

        {/* Active Execution Spinner */}
        <Show when={props.isCurrentSessionBusy}>
          <div class="flex items-center gap-2.5 p-3 bg-gi-primary/10 border border-gi-primary/20 text-xs text-gi-primary w-fit animate-pulse">
            <Loader2 class="w-4 h-4 animate-spin text-gi-status-warning shrink-0" />
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
          onClick={() => scrollToBottom(true)}
          class="absolute right-6 bottom-24 z-30 flex items-center gap-1.5 px-3 py-1.5 bg-gi-primary hover:bg-gi-primary-hover active:scale-95 text-gi-primary-foreground text-xs font-semibold shadow-xl border border-gi-primary-hover/40 backdrop-blur-md transition-all hover:scale-105 select-none"
          title="Jump to latest message"
        >
          <ArrowDown class="w-3.5 h-3.5" />
          <span>Jump to bottom</span>
        </button>
      </Show>
    </>
  );
};
