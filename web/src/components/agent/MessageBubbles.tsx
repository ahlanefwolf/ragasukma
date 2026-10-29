import { Component, For, Show, createMemo, createSignal } from 'solid-js';
import { Brain, ChevronDown, FileCode } from 'lucide-solid';

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
    const text = props.content || '';
    if (!text) return [];
    // Updated regex to support unicode characters and spaces in file paths
    const regex = /(?:^|\s)(@[\p{L}\p{N}\s_\-\./\\]+(?::\d+(?:-\d+)?)?)/gu;
    const result: { text: string; isTag: boolean; filePath?: string }[] = [];
    let lastIndex = 0;
    let match: RegExpExecArray | null;

    while ((match = regex.exec(text)) !== null) {
      const fullMatch = match[0];
      const tag = match[1];
      const tagIndex = match.index + fullMatch.indexOf(tag);

      if (tagIndex > lastIndex) {
        result.push({ text: text.substring(lastIndex, tagIndex), isTag: false });
      }

      const rawPath = tag.startsWith('@') ? tag.slice(1) : tag;
      const filePath = rawPath.replace(/:\d+(?:-\d+)?$/, '');
      result.push({ text: tag, isTag: true, filePath });
      lastIndex = tagIndex + tag.length;
    }

    if (lastIndex < text.length) {
      result.push({ text: text.substring(lastIndex), isTag: false });
    }

    return result;
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
              class={`inline-flex items-center gap-1 font-mono text-cyan-400 bg-cyan-500/15 border border-cyan-500/30 px-1.5 py-0.5 text-[11px] font-medium align-baseline mx-0.5 select-none ${
                props.onOpenFile ? 'hover:bg-cyan-500/25 hover:border-cyan-400/50 cursor-pointer transition-all' : ''
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
