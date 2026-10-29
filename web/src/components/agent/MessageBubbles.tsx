import { Component, For, Show, createMemo, createSignal } from 'solid-js';
import { Brain, ChevronDown, FileCode } from 'lucide-solid';
import { parseFileTags } from './fileTags';

// Compact & Collapsible Reasoning / Chain of Thought Component
export const ThoughtBubble: Component<{ thought: string }> = (props) => {
  const [isExpanded, setIsExpanded] = createSignal(false);
  const charCount = () => props.thought?.length ?? 0;

  return (
    <div class="w-full max-w-full mb-2 bg-gi-status-warning/10 border border-gi-status-warning/20 text-xs text-theme-text/90 font-mono overflow-hidden transition-all">
      <button
        type="button"
        onClick={() => setIsExpanded(!isExpanded())}
        class="w-full flex items-center justify-between px-2.5 py-1.5 hover:bg-gi-status-warning/15 cursor-pointer text-left transition-colors select-none"
      >
        <div class="flex items-center gap-1.5 font-semibold text-2xs text-gi-status-warning uppercase tracking-wider">
          <Brain class="w-3.5 h-3.5 text-gi-status-warning shrink-0" />
          <span>Reasoning / Thought Process</span>
          <span class="text-2xs lowercase font-normal opacity-70">({charCount()} chars)</span>
        </div>
        <div class="flex items-center gap-1 text-2xs text-gi-status-warning/80">
          <span>{isExpanded() ? 'Hide' : 'Show'}</span>
          <ChevronDown class={`w-3.5 h-3.5 transition-transform duration-200 ${isExpanded() ? 'rotate-180' : ''}`} />
        </div>
      </button>

      <Show when={isExpanded()}>
        <div class="p-2.5 pt-1.5 border-t border-gi-status-warning/15 whitespace-pre-wrap text-xs leading-relaxed opacity-90 max-h-80 overflow-y-auto custom-scrollbar">
          {props.thought}
        </div>
      </Show>
    </div>
  );
};

// Render user message content with color highlighting for tagged @files
export const UserMessageBody: Component<{
  content: string;
  onOpenFile?: (path: string) => void;
}> = (props) => {
  const parsed = createMemo(() => {
    // Single source of truth for the `@file` grammar (see ./fileTags.ts).
    // Using the shared parser guarantees the renderer, the autocomplete and
    // the inserter can never disagree — an earlier mismatch caused the whole
    // rest of the line to be highlighted as one giant tag.
    return parseFileTags(props.content || '');
  });

  return (
    <div class="select-text whitespace-pre-wrap break-words">
      <For each={parsed()}>
        {(token) => (
          <Show
            when={token.isTag}
            fallback={<span>{token.text}</span>}
          >
            <span
              class={`inline-flex items-center gap-1 font-mono text-blue-400 bg-blue-500/15 border border-blue-400/30 px-1.5 py-0.5 text-[11px] font-medium align-baseline mx-0.5 select-text rounded-sm ${
                props.onOpenFile
                  ? 'hover:bg-blue-500/25 hover:border-blue-300/60 cursor-pointer transition-colors'
                  : ''
              }`}
              onClick={(e) => {
                if (props.onOpenFile && token.filePath) {
                  e.stopPropagation();
                  props.onOpenFile(token.filePath);
                }
              }}
              title={props.onOpenFile ? `Open ${token.filePath}` : undefined}
            >
              <FileCode class="w-3 h-3 opacity-80 shrink-0" />
              <span>{token.text}</span>
            </span>
          </Show>
        )}
      </For>
    </div>
  );
};
