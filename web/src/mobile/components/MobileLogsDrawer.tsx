import { Component, For, Show } from 'solid-js';
import { Terminal, Copy, RefreshCw, X } from 'lucide-solid';
import { LogEntry } from '../../api';

interface MobileLogsDrawerProps {
  isOpen: boolean;
  onClose: () => void;
  logs: LogEntry[];
  onRefresh: () => void;
  isRefreshing: boolean;
  onCopy: () => void;
  formatStyle: (msg: string, level: string) => string;
}

export const MobileLogsDrawer: Component<MobileLogsDrawerProps> = (props) => {
  let containerRef: HTMLDivElement | undefined;

  return (
    <Show when={props.isOpen}>
      {/* Backdrop Lembut & Not Gelap Gulita */}
      <div 
        onClick={props.onClose} 
        class="fixed inset-0 z-40 bg-black/25 backdrop-blur-[2px] animate-in fade-in duration-150" 
      />

      {/* Slide-Up Bottom Sheet flush to bottom (no gap/margin) */}
      <div class="fixed inset-x-0 bottom-0 z-50 h-[70dvh] max-h-[70dvh] bg-theme-surface border-t border-theme-border shadow-2xl flex flex-col overflow-hidden animate-in slide-in-from-bottom duration-250 select-none pb-0 mb-0">
        {/* Drawer Header */}
        <div class="px-4 py-2.5 bg-theme-card border-b border-theme-border flex items-center justify-between shrink-0 select-none">
          <div class="flex items-center gap-2 font-mono">
            <Terminal size={15} class="text-theme-cyan" />
            <h3 class="text-theme-text uppercase tracking-wider">
              Live Console Logs
            </h3>
            <span class="text-2xs px-1.5 py-0.2 bg-theme-surface text-theme-cyan border border-theme-border font-bold">
              {props.logs.length}
            </span>
          </div>

          <div class="flex items-center gap-1">
            <button
              type="button"
              onClick={props.onCopy}
              class="p-1.5 text-theme-muted hover:text-theme-cyan active:scale-90 transition-colors"
              title="Copy All Logs"
            >
              <Copy size={14} />
            </button>
            <button
              type="button"
              onClick={props.onRefresh}
              class="p-1.5 text-theme-muted hover:text-theme-cyan active:scale-90 transition-colors"
              title="Refresh Logs"
            >
              <RefreshCw size={14} class={props.isRefreshing ? 'animate-spin text-theme-cyan' : ''} />
            </button>
            <div class="w-px h-3.5 bg-theme-border mx-0.5" />
            <button
              type="button"
              onClick={props.onClose}
              class="p-1.5 text-theme-muted hover:text-gi-status-danger active:scale-90 transition-colors"
              title="Close"
            >
              <X size={16} />
            </button>
          </div>
        </div>

        {/* Scrollable Logs Area Without Excessive Bottom Padding */}
        <div
          ref={containerRef}
          class="flex-1 overflow-y-auto overscroll-contain px-3 py-2 font-mono text-2xs leading-relaxed bg-theme-bg/80 select-text"
          style={{ "overscroll-behavior-y": "contain" }}
        >
          <Show
            when={props.logs.length > 0}
            fallback={
              <div class="h-full flex flex-col items-center justify-center text-xs text-theme-muted py-8 gap-2">
                <Terminal size={24} class="opacity-40" />
                <span>No activity logs yet...</span>
              </div>
            }
          >
            <For each={props.logs}>
              {(log, idx) => (
                <div class="break-words py-0.5 border-b border-theme-border/20 last:border-b-0 hover:bg-theme-card/30 px-1 transition-colors">
                  <span class="text-2xs text-theme-muted/70 mr-1.5 font-mono select-none">
                    [{log.timestamp ? log.timestamp.split(' ')[1] || log.timestamp : ''}]
                  </span>
                  <span class={props.formatStyle(log.message, log.level)}>
                    {log.message.replace(/[\u{1F000}-\u{1FAFF}\u{2600}-\u{27BF}\u{2300}-\u{23FF}\u{FE00}-\u{FE0F}\u{200D}]/gu, '')}
                  </span>
                </div>
              )}
            </For>
          </Show>
        </div>
      </div>
    </Show>
  );
};
