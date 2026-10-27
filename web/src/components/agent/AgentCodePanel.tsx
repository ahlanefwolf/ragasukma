import { Component, For, Show } from 'solid-js';
import {
  FileCode,
  X,
  Copy,
  Plus,
  Loader2,
  AtSign,
  RotateCcw,
  Sparkles,
} from 'lucide-solid';
import { WorkspaceFileItem } from '../../api';

export interface AgentCodePanelProps {
  file: WorkspaceFileItem | null;
  onClose: () => void;
  tagStartLine: string;
  tagEndLine: string;
  loadingFilePreview: boolean;
  highlightedCodeLines: string[];
  currentSelectedTag: string;
  onResetSelection: () => void;
  onLineClick: (lineNum: number, isShiftKey: boolean) => void;
  onGutterMouseDown: (lineNum: number) => void;
  onGutterMouseEnter: (lineNum: number) => void;
  onGutterMouseUp: () => void;
  onCodeTextMouseUp: () => void;
  isLineSelected: (lineNum: number) => boolean;
  onInsertTag: (relPath: string, start?: string, end?: string) => void;
  onCopyTag: () => void;
}

export const AgentCodePanel: Component<AgentCodePanelProps> = (props) => {
  return (
    <Show when={props.file}>
      <aside class="h-full flex flex-col bg-theme-surface border border-theme-border rounded-xl shadow-2xl overflow-hidden min-h-0 select-text">
        {/* Editor Tab Header */}
        <div class="px-3 py-2 bg-theme-card/50 border-b border-theme-border flex items-center justify-between gap-2 shrink-0 select-none">
          {/* File Tab Info */}
          <div class="flex items-center gap-2 min-w-0 flex-1">
            <div class="p-1 rounded bg-gi-primary/10 text-gi-primary border border-gi-primary/20 shrink-0">
              <FileCode class="w-4 h-4" />
            </div>
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <span class="text-xs font-semibold text-white truncate" title={props.file?.name}>
                  {props.file?.name}
                </span>
                <span class="text-2xs font-mono px-1.5 py-0.5 rounded bg-white/5 text-white/50 shrink-0">
                  {props.highlightedCodeLines.length} lines
                </span>
              </div>
              <p class="text-xs font-mono text-white/70 truncate" title={props.file?.relative_path}>
                {props.file?.relative_path}
              </p>
            </div>
          </div>

          {/* Quick Header Actions */}
          <div class="flex items-center gap-1.5 shrink-0">
            {/* Tag Badge / Copy */}
            <Show when={props.currentSelectedTag}>
              <button
                type="button"
                onClick={props.onCopyTag}
                class="px-2 py-1 text-xs font-mono bg-gi-primary/15 hover:bg-gi-primary/25 border border-gi-primary/30 text-gi-primary rounded transition-all flex items-center gap-1 active:scale-95"
                title="Copy tag reference to clipboard"
              >
                <Copy class="w-3 h-3" />
                <span class="hidden sm:inline">Copy Tag</span>
              </button>

              <button
                type="button"
                onClick={() => {
                  if (props.file) {
                    props.onInsertTag(props.file.relative_path, props.tagStartLine, props.tagEndLine);
                  }
                }}
                class="px-2.5 py-1 text-xs bg-gi-primary hover:bg-gi-primary-hover text-gi-primary-foreground font-semibold rounded transition-all flex items-center gap-1 shadow-sm active:scale-95"
                title="Insert tag into chat input"
              >
                <Plus class="w-3 h-3" />
                <span>Insert</span>
              </button>

              <button
                type="button"
                onClick={props.onResetSelection}
                class="p-1 text-white/70 hover:text-white/80 rounded hover:bg-white/5 transition-colors"
                title="Reset selection (reference whole file)"
              >
                <RotateCcw class="w-3.5 h-3.5" />
              </button>
            </Show>

            {/* Close Docked Panel Button */}
            <button
              type="button"
              onClick={props.onClose}
              class="p-1 text-white/70 hover:text-white rounded hover:bg-white/10 transition-colors ml-1"
              title="Close code viewer panel"
              aria-label="Close Code Viewer"
            >
              <X class="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Selected Range Status Bar */}
        <div class="px-3 py-1 bg-theme-bg/50 border-b border-theme-border/50 flex items-center justify-between text-xs font-mono text-white/50 select-none shrink-0">
          <div class="flex items-center gap-1.5 truncate">
            <AtSign class="w-3 h-3 text-gi-primary shrink-0" />
            <Show
              when={props.tagStartLine}
              fallback={
                <span class="text-white/70">Whole file reference (Click or drag line numbers to select range)</span>
              }
            >
              <span class="text-gi-primary font-semibold">
                @{props.file?.relative_path}:
                {props.tagStartLine}
                {props.tagEndLine && props.tagEndLine !== props.tagStartLine ? `-${props.tagEndLine}` : ''}
              </span>
            </Show>
          </div>

          <Show when={props.tagStartLine}>
            <button
              type="button"
              onClick={props.onResetSelection}
              class="text-2xs text-gi-primary/80 hover:text-gi-primary-hover underline underline-offset-2 ml-2 shrink-0"
            >
              Select All
            </button>
          </Show>
        </div>

        {/* Code Content Area */}
        <div
          class="flex-1 min-h-0 overflow-auto bg-theme-bg p-1 font-mono text-xs select-text focus:outline-none"
          onMouseUp={() => {
            props.onGutterMouseUp();
            props.onCodeTextMouseUp();
          }}
          tabIndex={0}
        >
          {/* Loading State */}
          <Show when={props.loadingFilePreview}>
            <div class="flex flex-col items-center justify-center h-full py-16 text-white/70 gap-2 select-none">
              <Loader2 class="w-6 h-6 animate-spin text-gi-primary" />
              <span>Loading file & highlighting syntax...</span>
            </div>
          </Show>

          {/* Empty State */}
          <Show when={!props.loadingFilePreview && props.highlightedCodeLines.length === 0}>
            <div class="text-center py-16 text-white/60 italic select-none">
              File is empty or contains no readable text content
            </div>
          </Show>

          {/* Render Code with Line Numbers */}
          <Show when={!props.loadingFilePreview && props.highlightedCodeLines.length > 0}>
            <div class="min-w-full inline-block py-2">
              <For each={props.highlightedCodeLines}>
                {(lineHtml, idx) => {
                  const lineNum = idx() + 1;
                  const selected = () => props.isLineSelected(lineNum);
                  return (
                    <div
                      data-line={lineNum}
                      class={`flex items-stretch hover:bg-white/[0.04] transition-colors leading-6 ${
                        selected()
                          ? 'bg-gi-primary/15 border-l-2 border-gi-primary'
                          : 'border-l-2 border-transparent'
                      }`}
                    >
                      {/* Gutter Line Number */}
                      <div
                        onMouseDown={() => props.onGutterMouseDown(lineNum)}
                        onMouseEnter={() => props.onGutterMouseEnter(lineNum)}
                        onClick={(e) => props.onLineClick(lineNum, e.shiftKey)}
                        class={`w-12 select-none text-right pr-3 font-mono text-xs cursor-pointer transition-colors shrink-0 ${
                          selected()
                            ? 'text-gi-primary font-bold bg-gi-primary/25'
                            : 'text-white/60 hover:text-white/70'
                        }`}
                        title={`Line ${lineNum} (Click or drag to select)`}
                      >
                        {lineNum}
                      </div>

                      {/* Code Content */}
                      <div
                        class="pl-3 pr-4 flex-1 whitespace-pre overflow-x-visible font-mono text-xs text-white/90"
                        innerHTML={lineHtml || ' '}
                      />
                    </div>
                  );
                }}
              </For>
            </div>
          </Show>
        </div>

        {/* Panel Footer */}
        <div class="px-3 py-2 bg-theme-card/30 border-t border-theme-border flex items-center justify-between shrink-0 select-none">
          <span class="text-xs text-white/70 truncate">
            Drag gutter to pick range • Click line to toggle
          </span>

          <button
            type="button"
            onClick={() => {
              if (props.file) {
                props.onInsertTag(props.file.relative_path, props.tagStartLine, props.tagEndLine);
              }
            }}
            class="px-3 py-1 text-xs bg-gi-primary hover:bg-gi-primary-hover text-gi-primary-foreground rounded font-semibold flex items-center gap-1.5 transition-colors shadow-sm active:scale-95 ml-2 shrink-0"
          >
            <Sparkles class="w-3.5 h-3.5" />
            <span>Insert Tag</span>
          </button>
        </div>
      </aside>
    </Show>
  );
};
