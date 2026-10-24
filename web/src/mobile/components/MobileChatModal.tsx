import { Component, For, Show, createSignal, createEffect, onMount } from 'solid-js';
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
  RefreshCw
} from 'lucide-solid';
import { historyService, ChatTurnItem } from '../../services/historyService';

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

  const loadHistoryPage = async (page: number, dateStr?: string, preserveScroll = false) => {
    if (isLoadingMore()) return;
    setIsLoadingMore(true);

    const prevScrollHeight = scrollContainerRef ? scrollContainerRef.scrollHeight : 0;
    const prevScrollTop = scrollContainerRef ? scrollContainerRef.scrollTop : 0;

    try {
      const dayData = await historyService.getHistoryByDate(dateStr, page, 25);
      const incomingTurns = dayData.turns || [];

      if (incomingTurns.length > 0) {
        if (page === 1 && !preserveScroll) {
          setMessages(incomingTurns);
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
      const targetDate = dates.length > 0 ? dates[0] : undefined;
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

  createEffect(() => {
    if (props.isOpen) {
      initHistory();
      setTimeout(() => {
        if (inputRef) inputRef.focus();
      }, 250);
    }
  });

  const handleSend = () => {
    const text = props.inputText.trim();
    if (!text || props.isSending) return;

    // Optimistically push the sent user message
    const now = new Date();
    const timeStr = `${String(now.getHours()).padStart(2, '0')}:${String(now.getMinutes()).padStart(2, '0')}`;
    const userTurn: ChatTurnItem = {
      time: timeStr,
      speaker: props.callsign || 'You',
      message: text,
      timestamp: Math.floor(Date.now() / 1000)
    };

    setMessages(prev => [...prev, userTurn]);
    props.onSend(text);

    // Auto-scroll to bottom
    setTimeout(() => {
      if (scrollContainerRef) {
        scrollContainerRef.scrollTo({
          top: scrollContainerRef.scrollHeight,
          behavior: 'smooth'
        });
      }
    }, 50);
  };

  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  return (
    <Show when={props.isOpen}>
      <div 
        class="fixed inset-0 z-50 flex flex-col bg-theme-bg/95 backdrop-blur-md text-theme-text animate-in fade-in zoom-in-95 duration-200 select-none overflow-hidden"
        style={{ "padding-top": "env(safe-area-inset-top, 0px)", "padding-bottom": "env(safe-area-inset-bottom, 0px)" }}
      >
        {/* Top Header - WhatsApp Styled Bar adhering to App Theme */}
        <header class="h-14 border-b border-theme-border bg-theme-surface/90 px-3 flex items-center justify-between shadow-sm shrink-0">
          <div class="flex items-center gap-2.5 min-w-0">
            <button
              type="button"
              onClick={props.onClose}
              class="p-2 -ml-1 text-theme-muted hover:text-theme-text active:scale-90 transition-transform rounded-full"
              title="Back"
            >
              <ArrowLeft size={20} />
            </button>
            <div class="relative shrink-0">
              <div class="w-9 h-9 rounded-full bg-theme-cyan/20 border border-theme-cyan/50 flex items-center justify-center text-theme-cyan shadow-sm">
                <Bot size={18} />
              </div>
              <span class="absolute bottom-0 right-0 w-2.5 h-2.5 rounded-full bg-emerald-500 border-2 border-theme-surface animate-pulse" />
            </div>
            <div class="flex flex-col min-w-0">
              <span class="text-sm font-bold text-theme-text truncate leading-tight">
                {props.assistantName || 'Assistant'}
              </span>
              <span class="text-2xs text-theme-cyan font-mono truncate">
                {props.assistantStatus || 'online'}
              </span>
            </div>
          </div>

          <div class="flex items-center gap-1 text-theme-muted">
            <button
              type="button"
              onClick={() => initHistory()}
              class="p-2 hover:text-theme-cyan transition-colors rounded-full"
              title="Refresh History"
            >
              <RefreshCw size={16} class={isLoadingMore() ? 'animate-spin' : ''} />
            </button>
          </div>
        </header>

        {/* Chat History Viewport (Reverse Scroll & WhatsApp Wallpaper Feel) */}
        <div 
          ref={scrollContainerRef}
          onScroll={handleScroll}
          class="flex-1 overflow-y-auto p-4 space-y-3.5 overscroll-contain bg-gradient-to-b from-theme-bg/50 via-theme-card/30 to-theme-bg/60"
        >
          {/* Lazy Loading Indicator at Top */}
          <Show when={isLoadingMore() || isInitialLoading()}>
            <div class="flex items-center justify-center py-2">
              <div class="flex items-center gap-2 px-3 py-1 rounded-full bg-theme-surface border border-theme-border text-2xs text-theme-muted font-mono shadow-sm">
                <Loader2 size={12} class="animate-spin text-theme-cyan" />
                <span>Loading messages...</span>
              </div>
            </div>
          </Show>

          <Show when={!isLoadingMore() && !hasMore() && messages().length > 0}>
            <div class="flex items-center justify-center py-2">
              <span class="px-3 py-1 rounded-full bg-theme-surface/50 border border-theme-border/50 text-2xs text-theme-muted/70 font-mono">
                Beginning of message history
              </span>
            </div>
          </Show>

          {/* Empty State */}
          <Show when={!isInitialLoading() && messages().length === 0}>
            <div class="h-full flex flex-col items-center justify-center text-center p-6 space-y-3">
              <div class="w-14 h-14 rounded-full bg-theme-card border border-theme-border flex items-center justify-center text-theme-muted">
                <Sparkles size={24} class="text-theme-cyan/80" />
              </div>
              <div class="space-y-1">
                <p class="text-sm font-semibold text-theme-text">No conversation yet</p>
                <p class="text-xs text-theme-muted max-w-xs">
                  Send a message to initiate conversation with {props.assistantName}.
                </p>
              </div>
            </div>
          </Show>

          {/* WhatsApp Bubbles Stream */}
          <For each={messages()}>
            {(turn) => {
              const isUser = turn.speaker.toLowerCase() === (props.callsign || 'user').toLowerCase() || 
                             turn.speaker.toLowerCase() === 'user' || 
                             turn.speaker.toLowerCase() === 'you';

              return (
                <div class={`flex w-full ${isUser ? 'justify-end' : 'justify-start'} animate-in fade-in duration-150`}>
                  <div
                    class={`relative max-w-[82%] sm:max-w-[70%] px-3.5 py-2 text-xs shadow-md select-text ${
                      isUser
                        ? 'bg-theme-cyan text-theme-bg font-medium rounded-2xl rounded-tr-xs'
                        : 'bg-theme-card border border-theme-border text-theme-text rounded-2xl rounded-tl-xs'
                    }`}
                  >
                    {/* Speaker Header on Group/Bot */}
                    <Show when={!isUser}>
                      <span class="block text-2xs font-bold text-theme-cyan mb-0.5">
                        {turn.speaker}
                      </span>
                    </Show>

                    {/* Message Body Content */}
                    <div class="whitespace-pre-wrap break-words leading-relaxed">
                      {turn.message}
                    </div>

                    {/* Message Footer (Timestamp & WhatsApp Ticks) */}
                    <div class={`flex items-center justify-end gap-1 mt-1 text-2xs font-mono select-none ${
                      isUser ? 'text-theme-bg/75' : 'text-theme-muted'
                    }`}>
                      <span>{turn.time}</span>
                      <Show when={isUser}>
                        <CheckCheck size={13} class="text-theme-bg/90 inline-block" />
                      </Show>
                    </div>
                  </div>
                </div>
              );
            }}
          </For>
        </div>

        {/* Bottom Input Area - WhatsApp Pill Styled with Send Button */}
        <footer class="p-2.5 bg-theme-surface/95 border-t border-theme-border shrink-0 flex items-end gap-2">
          <div class="flex-1 min-h-[42px] max-h-[120px] bg-theme-card border border-theme-border rounded-2xl px-3 py-2 flex items-center gap-2 focus-within:border-theme-cyan transition-colors shadow-inner">
            <textarea
              ref={inputRef}
              value={props.inputText}
              onInput={(e) => props.onInputText(e.currentTarget.value)}
              onKeyDown={handleKeyDown}
              placeholder="Type a message..."
              rows={1}
              disabled={props.isSending}
              class="w-full bg-transparent text-xs text-theme-text placeholder:text-theme-muted/60 resize-none outline-hidden focus:outline-none max-h-24 leading-relaxed"
            />
            <Show when={props.inputText.length > 0}>
              <button
                type="button"
                onClick={() => props.onInputText('')}
                class="p-1 text-theme-muted hover:text-theme-text active:scale-90 transition-transform"
                title="Clear input"
              >
                <Trash2 size={14} />
              </button>
            </Show>
          </div>

          <button
            type="button"
            onClick={handleSend}
            disabled={!props.inputText.trim() || props.isSending}
            class="w-11 h-11 rounded-full bg-theme-cyan text-theme-bg flex items-center justify-center shrink-0 shadow-md active:scale-90 transition-all disabled:opacity-40 disabled:scale-100"
            title="Send Message"
          >
            <Show when={props.isSending} fallback={<Send size={16} class="translate-x-0.5" />}>
              <Loader2 size={16} class="animate-spin" />
            </Show>
          </button>
        </footer>
      </div>
    </Show>
  );
};
