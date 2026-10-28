import { Component, For, Show, createSignal, createEffect, onMount, onCleanup } from 'solid-js';
import { 
  ArrowLeft, 
  Send, 
  Trash2, 
  Check, 
  CheckCheck, 
  Loader2, 
  Bot, 
  User, 
  Sparkles,
  RefreshCw,
  X,
  Mic,
  MicOff
} from 'lucide-solid';
import { historyService, ChatTurnItem } from '../../services/historyService';
import { MarkdownView } from '../../components/agent/MarkdownView';

interface IncomingChatEvent {
  id?: string;
  sender: 'assistant' | 'user';
  text: string;
  emotion?: string;
  time?: string;
}

export interface MobileChatModalProps {
  isOpen: boolean;
  onClose: () => void;
  assistantName: string;
  callsign: string;
  isSending: boolean;
  inputText: string;
  onInputText: (val: string) => void;
  onSend: (text?: string) => Promise<void> | void;
  assistantStatus?: string;
  currentEmotion?: string;
  incomingMessage?: IncomingChatEvent | null;
  isListening?: boolean;
  onToggleMic?: () => void;
}

export const MobileChatModal: Component<MobileChatModalProps> = (props) => {
  let scrollContainerRef: HTMLDivElement | undefined;
  let inputRef: HTMLTextAreaElement | undefined;

  const [messages, setMessages] = createSignal<ChatTurnItem[]>([]);
  const [currentPage, setCurrentPage] = createSignal<number>(1);
  const [hasMore, setHasMore] = createSignal<boolean>(true);
  const [isLoadingMore, setIsLoadingMore] = createSignal<boolean>(false);
  const [isInitialLoading, setIsInitialLoading] = createSignal<boolean>(false);
  const [historyDates, setHistoryDates] = createSignal<string[]>([]);
  const [currentDateIndex, setCurrentDateIndex] = createSignal<number>(0);
  const [viewportHeight, setViewportHeight] = createSignal<string>('100dvh');

  const updateViewport = () => {
    if (window.visualViewport) {
      setViewportHeight(`${window.visualViewport.height}px`);
    } else {
      setViewportHeight(`${window.innerHeight}px`);
    }
  };

  onMount(() => {
    updateViewport();
    if (window.visualViewport) {
      window.visualViewport.addEventListener('resize', updateViewport);
      window.visualViewport.addEventListener('scroll', updateViewport);
    } else {
      window.addEventListener('resize', updateViewport);
    }
    document.addEventListener('fullscreenchange', updateViewport);

    onCleanup(() => {
      if (window.visualViewport) {
        window.visualViewport.removeEventListener('resize', updateViewport);
        window.visualViewport.removeEventListener('scroll', updateViewport);
      } else {
        window.removeEventListener('resize', updateViewport);
      }
      document.removeEventListener('fullscreenchange', updateViewport);
    });
  });

  const scrollToBottom = (smooth = false) => {
    if (!scrollContainerRef) return;
    if (smooth) {
      scrollContainerRef.scrollTo({
        top: scrollContainerRef.scrollHeight,
        behavior: 'smooth',
      });
    } else {
      scrollContainerRef.scrollTop = scrollContainerRef.scrollHeight;
    }
  };

  const loadHistoryPage = async (page: number, dateStr?: string, preserveScroll = false) => {
    if (isLoadingMore()) return;
    setIsLoadingMore(true);

    const prevScrollHeight = scrollContainerRef ? scrollContainerRef.scrollHeight : 0;
    const prevScrollTop = scrollContainerRef ? scrollContainerRef.scrollTop : 0;

    try {
      const dayData = await historyService.getHistoryByDate(dateStr, page, 25, 'desc');
      const incomingTurns = dayData.turns || [];

      if (incomingTurns.length > 0) {
        if (page === 1 && !preserveScroll) {
          setMessages((prev) => {
            if (prev.length === 0) return incomingTurns;
            // Retain recent optimistic local messages (sent in last 60s) that have not yet been stored in DB
            const incomingTextMap = new Set(incomingTurns.map((m) => m.message.trim()));
            const pendingLocal = prev.filter((m) => {
              const isRecent = m.timestamp && (Math.floor(Date.now() / 1000) - m.timestamp) < 60;
              return isRecent && !incomingTextMap.has(m.message.trim());
            });
            return [...incomingTurns, ...pendingLocal];
          });
        } else {
          // Prepend older history to the top without duplicates
          setMessages(prev => {
            const existingIds = new Set(prev.map(m => m.id || `${m.time}-${m.speaker}-${m.message.slice(0, 15)}`));
            const filteredIncoming = incomingTurns.filter(m => !existingIds.has(m.id || `${m.time}-${m.speaker}-${m.message.slice(0, 15)}`));
            return [...filteredIncoming, ...prev];
          });
        }
      }

      setHasMore(dayData.has_more ?? (dayData.page ? dayData.page < (dayData.total_pages || 1) : false));
      setCurrentPage(page);

      if (preserveScroll && scrollContainerRef) {
        requestAnimationFrame(() => {
          if (scrollContainerRef) {
            const heightDiff = scrollContainerRef.scrollHeight - prevScrollHeight;
            scrollContainerRef.scrollTop = prevScrollTop + heightDiff;
          }
        });
      } else if (!preserveScroll && page === 1 && scrollContainerRef) {
        requestAnimationFrame(() => {
          if (scrollContainerRef) {
            scrollContainerRef.scrollTop = scrollContainerRef.scrollHeight;
          }
        });
      }
    } catch (err) {
      console.error('Failed to load history:', err);
    } finally {
      setIsLoadingMore(false);
      setIsInitialLoading(false);
    }
  };

  const initHistory = async () => {
    setIsInitialLoading(true);
    try {
      const dates = await historyService.getHistoryDates();
      setHistoryDates(dates);
      setCurrentDateIndex(0);
      // Use latest date from history, or default to today's local date (YYYY-MM-DD)
      const now = new Date();
      const todayStr = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
      const targetDate = dates.length > 0 ? dates[0] : todayStr;
      await loadHistoryPage(1, targetDate, false);
    } catch {
      setIsInitialLoading(false);
    }
  };

  const handleScroll = () => {
    if (!scrollContainerRef || isLoadingMore()) return;

    // Trigger lazy loading when user scrolls within 60px of top
    if (scrollContainerRef.scrollTop <= 60) {
      if (hasMore()) {
        const dates = historyDates();
        const curDate = dates[currentDateIndex()];
        loadHistoryPage(currentPage() + 1, curDate, true);
      } else if (currentDateIndex() + 1 < historyDates().length) {
        // Move to the previous date in history
        const nextIndex = currentDateIndex() + 1;
        setCurrentDateIndex(nextIndex);
        const prevDate = historyDates()[nextIndex];
        setCurrentPage(1);
        loadHistoryPage(1, prevDate, true);
      }
    }
  };

  // Helper to match user speaker aliases (You, user, me, callsign)
  const isUserSpeaker = (speaker: string) => {
    const s = (speaker || '').toLowerCase().trim();
    const callsign = (props.callsign || '').toLowerCase().trim();
    return s === 'you' || s === 'user' || s === 'me' || (callsign !== '' && s === callsign);
  };

  const isSameSpeaker = (s1: string, s2: string) => {
    if (s1.toLowerCase().trim() === s2.toLowerCase().trim()) return true;
    if (isUserSpeaker(s1) && isUserSpeaker(s2)) return true;
    if (!isUserSpeaker(s1) && !isUserSpeaker(s2)) return true;
    return false;
  };

  createEffect(() => {
    if (props.isOpen) {
      initHistory();
      setTimeout(() => {
        if (inputRef) inputRef.focus();
        scrollToBottom(false);
      }, 250);

      // Connect to native Axum Tokio SSE stream for real-time chat turns
      const unsubscribeSSE = historyService.subscribeChatEvents((item) => {
        setMessages((prev) => {
          // Find matching message by exact ID or by speaker + text similarity / timestamp proximity
          const existingIndex = prev.findIndex((m) => {
            if (item.id && m.id && item.id === m.id) return true;
            if (isSameSpeaker(m.speaker, item.speaker)) {
              if (m.message.trim() === item.message.trim()) return true;
              // If local message was a TTS chunk or substring of the complete server turn
              if (item.message.includes(m.message.trim()) && Math.abs((m.timestamp || 0) - (item.timestamp || 0)) <= 25) {
                return true;
              }
              // Proximity match within 15 seconds
              if (Math.abs((m.timestamp || 0) - (item.timestamp || 0)) <= 15) {
                return true;
              }
            }
            return false;
          });

          if (existingIndex !== -1) {
            // Replace optimistic / partial message with verified server record
            const updated = [...prev];
            updated[existingIndex] = { ...updated[existingIndex], ...item };
            return updated;
          }

          return [...prev, item];
        });

        if (scrollContainerRef) {
          const isNearBottom = scrollContainerRef.scrollHeight - scrollContainerRef.scrollTop - scrollContainerRef.clientHeight < 120;
          if (isNearBottom) {
            requestAnimationFrame(() => scrollToBottom(false));
          }
        }
      });

      const handleVisibilityChange = () => {
        if (document.visibilityState === 'visible') {
          initHistory();
        }
      };
      document.addEventListener('visibilitychange', handleVisibilityChange);

      onCleanup(() => {
        unsubscribeSSE();
        document.removeEventListener('visibilitychange', handleVisibilityChange);
      });
    }
  });

  // Listen to incoming real-time messages from action stream (e.g. WebSocket speech actions)
  createEffect(() => {
    const inc = props.incomingMessage;
    if (!inc || !inc.text) return;

    const now = new Date();
    const timeStr = inc.time || `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
    const speakerName = inc.sender === 'user' ? (props.callsign || 'You') : (props.assistantName || 'Assistant');
    const isAssistant = inc.sender === 'assistant';

    setMessages((prev) => {
      // 1. Check if exact ID was already appended
      if (inc.id && prev.some((m) => m.id === inc.id)) {
        return prev;
      }

      // 2. If this is an assistant speech chunk from an ongoing speech stream, merge or append
      if (isAssistant && prev.length > 0) {
        const last = prev[prev.length - 1];
        if (!isUserSpeaker(last.speaker) && Math.abs((last.timestamp || 0) - Math.floor(Date.now() / 1000)) <= 5) {
          // If the last message already contains or equals this chunk, skip
          if (last.message.includes(inc.text.trim())) {
            return prev;
          }
          // Concatenate chunk to the ongoing assistant bubble
          const updated = [...prev];
          updated[updated.length - 1] = {
            ...last,
            message: `${last.message} ${inc.text.trim()}`.trim(),
            emotion: inc.emotion || last.emotion,
          };
          return updated;
        }
      }

      // 3. Duplicate check within 4s for same speaker and text
      const isDupe = prev.some((m) => 
        isSameSpeaker(m.speaker, speakerName) &&
        m.message.trim() === inc.text.trim() &&
        Math.abs((m.timestamp || 0) - Math.floor(Date.now() / 1000)) <= 4
      );
      if (isDupe) return prev;

      return [
        ...prev,
        {
          id: inc.id || `msg_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`,
          time: timeStr,
          speaker: speakerName,
          message: inc.text,
          emotion: inc.emotion,
          timestamp: Math.floor(Date.now() / 1000),
        },
      ];
    });

    if (scrollContainerRef) {
      const isNearBottom = scrollContainerRef.scrollHeight - scrollContainerRef.scrollTop - scrollContainerRef.clientHeight < 120;
      if (isNearBottom) {
        requestAnimationFrame(() => scrollToBottom(false));
      }
    }
  });

  const handleSend = () => {
    const text = props.inputText.trim();
    if (!text || props.isSending) return;

    // Optimistically push the sent user message with unique ID
    const now = new Date();
    const timeStr = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
    const localId = `user_${Date.now()}_${Math.random().toString(36).substring(2, 7)}`;
    const userTurn: ChatTurnItem = {
      id: localId,
      time: timeStr,
      speaker: props.callsign || 'You',
      message: text,
      timestamp: Math.floor(Date.now() / 1000)
    };

    setMessages(prev => [...prev, userTurn]);
    
    // Immediately clear input for snappy UI feedback
    props.onInputText('');
    if (inputRef) {
      inputRef.style.height = 'auto';
    }
    
    props.onSend(text);

    // Smooth auto-scroll to bottom on user send
    requestAnimationFrame(() => {
      scrollToBottom(true);
    });
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    // Send message on Enter (without Shift), Shift+Enter creates a newline
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
      return;
    }

    // Shift+Enter: allow newline (pindah baris) and expand textarea smoothly
    if (e.key === 'Enter') {
      requestAnimationFrame(() => {
        if (inputRef) adjustInputHeight(inputRef);
      });
    }
  };

  const adjustInputHeight = (el?: HTMLTextAreaElement | null) => {
    if (!el) return;
    el.style.height = 'auto';
    const next = Math.min(Math.max(el.scrollHeight, 38), 140);
    el.style.height = `${next}px`;
  };

  createEffect(() => {
    if (!props.inputText && inputRef) {
      inputRef.style.height = 'auto';
    }
  });

  return (
    <Show when={props.isOpen}>
      <div 
        class="mobile-modal-shell fixed inset-0 z-50 m-0 box-border w-screen h-[100dvh] max-w-[100vw] pointer-events-auto bg-gi-canvas overflow-hidden flex flex-col select-none animate-in fade-in duration-200"
        style={{
          height: viewportHeight(),
          "max-height": viewportHeight(),
          "box-sizing": "border-box",
        }}
      >
        {/* Top Header - WhatsApp Styled Bar adhering to App Theme */}
        <header class="h-14 border-b border-theme-border bg-theme-surface px-3 flex items-center justify-between shadow-sm shrink-0">
          <div class="flex items-center gap-2.5 min-w-0">
            <button
              type="button"
              onClick={props.onClose}
              class="p-2 -ml-1 text-theme-muted hover:text-theme-text active:scale-90 transition-transform"
              title="Back"
            >
              <ArrowLeft size={20} />
            </button>
            <div class="relative shrink-0">
              <div class="w-9 h-9 bg-gi-primary/15 border border-gi-primary/40 flex items-center justify-center text-gi-primary shadow-sm">
                <Bot size={18} />
              </div>
              <span class="absolute bottom-0 right-0 w-2.5 h-2.5 bg-gi-status-success border-2 border-theme-surface animate-pulse" />
            </div>
            <div class="flex flex-col min-w-0">
              <span class="text-sm font-bold text-theme-text truncate leading-tight">
                {props.assistantName || 'Assistant'}
              </span>
              <span class="text-2xs text-gi-primary font-mono truncate">
                {props.assistantStatus || 'online'}
              </span>
            </div>
          </div>

          <div class="flex items-center gap-1 text-theme-muted">
            <button
              type="button"
              onClick={() => initHistory()}
              class="p-2 hover:text-gi-primary-hover transition-colors"
              title="Refresh History"
            >
              <RefreshCw size={16} class={isLoadingMore() ? 'animate-spin' : ''} />
            </button>
          </div>
        </header>

        {/* Chat History Viewport */}
        <div 
          ref={scrollContainerRef}
          onScroll={handleScroll}
          class="flex-1 min-h-0 overflow-y-auto p-4 space-y-3.5 overscroll-contain bg-gradient-to-b from-theme-bg/50 via-theme-card/30 to-theme-bg/60"
        >
          {/* Lazy Loading Indicator at Top */}
          <Show when={isLoadingMore() || isInitialLoading()}>
            <div class="flex items-center justify-center py-2">
              <div class="flex items-center gap-2 px-3 py-1 bg-theme-surface border border-theme-border text-2xs text-theme-muted font-mono shadow-sm">
                <Loader2 size={12} class="animate-spin text-gi-primary" />
                <span>Loading messages...</span>
              </div>
            </div>
          </Show>

          {/* Empty State */}
          <Show when={!isInitialLoading() && messages().length === 0}>
            <div class="h-full flex flex-col items-center justify-center text-center p-6 space-y-3">
              <div class="w-14 h-14 bg-theme-card border border-theme-border flex items-center justify-center text-theme-muted">
                <Sparkles size={24} class="text-gi-primary/80" />
              </div>
              <div class="space-y-1">
                <p class="text-sm font-semibold text-theme-text">No conversation yet</p>
                <p class="text-xs text-theme-muted max-w-xs">
                  Send a message to initiate conversation with {props.assistantName}.
                </p>
              </div>
            </div>
          </Show>

          {/* Chat Messages */}
          <For each={messages()}>
            {(turn: any) => {
              const speaker = turn.speaker || turn.sender || 'Assistant';
              const textContent = turn.message || turn.text || '';
              const isUser = speaker.toLowerCase() === (props.callsign || 'user').toLowerCase() || 
                             speaker.toLowerCase() === 'user' || 
                             speaker.toLowerCase() === 'you';

              return (
                <div class={`flex w-full ${isUser ? 'justify-end' : 'justify-start'} animate-in fade-in duration-150`}>
                  <div
                    class={`relative max-w-[85%] sm:max-w-[75%] px-3.5 py-2.5 text-xs shadow-md select-text ${
                      isUser
                        ? 'bg-gi-primary text-gi-primary-foreground font-medium'
                        : 'bg-theme-card border border-theme-border/80 text-theme-text'
                    }`}
                  >
                    {/* Speaker Header on Group/Bot */}
                    <Show when={!isUser}>
                      <span class="block text-2xs font-bold text-gi-primary mb-1">
                        {speaker}
                      </span>
                    </Show>

                    {/* Message Body Content */}
                    <div class="break-words leading-relaxed">
                      <Show when={isUser} fallback={<MarkdownView content={textContent} />}>
                        <div class="whitespace-pre-wrap break-words leading-relaxed">{textContent}</div>
                      </Show>
                    </div>

                    {/* Message Footer (Timestamp & WhatsApp Ticks) */}
                    <div class={`flex items-center justify-end gap-1 mt-1 text-2xs font-mono select-none ${
                      isUser ? 'text-gi-primary-foreground/75' : 'text-theme-muted'
                    }`}>
                      <span>{turn.time}</span>
                      <Show when={isUser}>
                        <CheckCheck size={13} class="text-gi-primary-foreground/90 inline-block" />
                      </Show>
                    </div>
                  </div>
                </div>
              );
            }}
          </For>
        </div>

        {/* Bottom Input Area - Precision Engineered Modern Capsule */}
        <footer 
          class="p-2.5 sm:p-3 pb-2.5 sm:pb-3 bg-gi-surface/95 backdrop-blur-md border-t border-theme-border/60 shrink-0 flex items-end w-full"
        >
          <div class="relative w-full max-w-full bg-gi-surface-elevated/90 border border-theme-border/80 focus-within:border-gi-primary focus-within:ring-2 focus-within:ring-gi-primary/20 shadow-sm transition-all flex flex-col justify-between overflow-hidden">
            <textarea
              ref={inputRef}
              value={props.inputText}
              onFocus={() => {
                setTimeout(scrollToBottom, 200);
              }}
              onInput={(e) => {
                props.onInputText(e.currentTarget.value);
                adjustInputHeight(e.currentTarget);
              }}
              onKeyDown={handleKeyDown}
              placeholder="Type a message..."
              rows={1}
              disabled={props.isSending}
              class="w-full min-w-0 px-3.5 pt-2.5 pb-1.5 bg-transparent text-[14px] sm:text-xs text-theme-text placeholder:text-theme-muted/50 resize-none outline-none leading-relaxed min-h-[38px] max-h-[140px] custom-scrollbar disabled:opacity-60 font-sans"
            />

            {/* Bottom Micro Action Bar */}
            <div class="flex items-center justify-between px-3 pb-2 pt-1 select-none border-t border-theme-border/20">
              <div class="flex items-center gap-1.5 min-w-0">
                <Show when={props.inputText.length > 0}>
                  <button
                    type="button"
                    onClick={() => {
                      props.onInputText('');
                      if (inputRef) adjustInputHeight(inputRef);
                    }}
                    class="w-5 h-5 bg-theme-surface/80 border border-theme-border/60 text-theme-muted hover:text-theme-text flex items-center justify-center active:scale-90 transition-transform cursor-pointer"
                    title="Clear message"
                  >
                    <X size={11} />
                  </button>
                  <Show when={props.inputText.length > 50}>
                    <span class="text-[11px] font-mono text-theme-muted/70 bg-theme-bg/40 px-1.5 py-0.5">
                      {props.inputText.length} chars
                    </span>
                  </Show>
                </Show>
                <Show when={props.inputText.length === 0}>
                  <span class="text-[11px] font-mono text-theme-muted/40 hidden sm:inline">
                    ↵ send • shift+↵ line
                  </span>
                </Show>
              </div>

              <div class="flex items-center gap-1.5 shrink-0 ml-2">
                <Show when={props.onToggleMic}>
                  <button
                    type="button"
                    onClick={props.onToggleMic}
                    class={`w-8 h-8 flex items-center justify-center transition-all active:scale-90 shadow-sm cursor-pointer ${
                      props.isListening
                        ? 'bg-gi-status-danger text-white animate-pulse shadow-md shadow-gi-status-danger/40 ring-2 ring-gi-status-danger/30'
                        : 'bg-theme-surface/80 border border-theme-border text-theme-muted hover:text-gi-primary-hover hover:border-gi-primary/50'
                    }`}
                    title={props.isListening ? 'Stop Listening' : 'Voice Input (PTT)'}
                  >
                    <Show when={props.isListening} fallback={<Mic size={15} />}>
                      <MicOff size={15} />
                    </Show>
                  </button>
                </Show>

                <button
                  type="button"
                  onClick={handleSend}
                  disabled={!props.inputText.trim() || props.isSending}
                  class="w-8 h-8 bg-gi-primary hover:bg-gi-primary-hover text-gi-primary-foreground flex items-center justify-center disabled:opacity-25 disabled:cursor-not-allowed active:scale-90 transition-all shadow-md shadow-gi-primary/25 cursor-pointer"
                  title="Send Message"
                >
                  <Show when={props.isSending} fallback={<Send size={14} class="translate-x-px" />}>
                    <Loader2 size={14} class="animate-spin" />
                  </Show>
                </button>
              </div>
            </div>
          </div>
        </footer>
      </div>
    </Show>
  );
};
