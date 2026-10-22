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
      <aside class="h-full flex flex-col bg-[#0b0f17] border border-white/10 rounded-xl shadow-2xl overflow-hidden min-h-0 select-text">
        {/* Editor Tab Header */}
        <div class="px-3 py-2 bg-white/[0.03] border-b border-white/10 flex items-center justify-between gap-2 shrink-0 select-none">
          {/* File Tab Info */}
          <div class="flex items-center gap-2 min-w-0 flex-1">
            <div class="p-1 rounded bg-indigo-500/10 text-indigo-400 border border-indigo-500/20 shrink-0">
              <FileCode class="w-4 h-4" />
            </div>
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <span class="text-xs font-semibold text-white truncate" title={props.file?.name}>
                  {props.file?.name}
                </span>
                <span class="text-[10px] font-mono px-1.5 py-0.5 rounded bg-white/5 text-white/50 shrink-0">
                  {props.highlightedCodeLines.length} lines
                </span>
              </div>
              <p class="text-[11px] font-mono text-white/40 truncate" title={props.file?.relative_path}>
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
                class="px-2 py-1 text-[11px] font-mono bg-indigo-500/15 hover:bg-indigo-500/25 border border-indigo-500/30 text-indigo-300 rounded transition-all flex items-center gap-1 active:scale-95"
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
                class="px-2.5 py-1 text-[11px] bg-indigo-600 hover:bg-indigo-500 text-white font-medium rounded transition-all flex items-center gap-1 shadow-sm shadow-indigo-600/30 active:scale-95"
                title="Insert tag into chat input"
              >
                <Plus class="w-3 h-3" />
                <span>Insert</span>
              </button>

              <button
                type="button"
                onClick={props.onResetSelection}
                class="p-1 text-white/40 hover:text-white/80 rounded hover:bg-white/5 transition-colors"
                title="Reset selection (reference whole file)"
              >
                <RotateCcw class="w-3.5 h-3.5" />
              </button>
            </Show>

            {/* Close Docked Panel Button */}
            <button
              type="button"
              onClick={props.onClose}
              class="p-1 text-white/40 hover:text-white rounded hover:bg-white/10 transition-colors ml-1"
              title="Close code viewer panel"
              aria-label="Close Code Viewer"
            >
              <X class="w-4 h-4" />
            </button>
          </div>
        </div>

        {/* Selected Range Status Bar */}
        <div class="px-3 py-1 bg-white/[0.015] border-b border-white/5 flex items-center justify-between text-[11px] font-mono text-white/50 select-none shrink-0">
          <div class="flex items-center gap-1.5 truncate">
            <AtSign class="w-3 h-3 text-indigo-400 shrink-0" />
            <Show
              when={props.tagStartLine}
              fallback={
                <span class="text-white/40">Whole file reference (Click or drag line numbers to select range)</span>
              }
            >
              <span class="text-indigo-300 font-medium">
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
              class="text-[10px] text-indigo-400/80 hover:text-indigo-300 underline underline-offset-2 ml-2 shrink-0"
            >
              Select All
            </button>
          </Show>
        </div>

        {/* Code Content Area */}
        <div
          class="flex-1 min-h-0 overflow-auto bg-[#070a0f] p-1 font-mono text-xs select-text focus:outline-none"
          onMouseUp={() => {
            props.onGutterMouseUp();
            props.onCodeTextMouseUp();
          }}
          tabIndex={0}
        >
          {/* Loading State */}
          <Show when={props.loadingFilePreview}>
            <div class="flex flex-col items-center justify-center h-full py-16 text-white/40 gap-2 select-none">
              <Loader2 class="w-6 h-6 animate-spin text-indigo-400" />
              <span>Loading file & highlighting syntax...</span>
            </div>
          </Show>

          {/* Empty State */}
          <Show when={!props.loadingFilePreview && props.highlightedCodeLines.length === 0}>
            <div class="text-center py-16 text-white/30 italic select-none">
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
                          ? 'bg-indigo-500/20 border-l-2 border-indigo-400'
                          : 'border-l-2 border-transparent'
                      }`}
                    >
                      {/* Gutter Line Number */}
                      <div
                        onMouseDown={() => props.onGutterMouseDown(lineNum)}
                        onMouseEnter={() => props.onGutterMouseEnter(lineNum)}
                        onClick={(e) => props.onLineClick(lineNum, e.shiftKey)}
                        class={`w-12 select-none text-right pr-3 font-mono text-[11px] cursor-pointer transition-colors shrink-0 ${
                          selected()
                            ? 'text-indigo-300 font-bold bg-indigo-500/30'
                            : 'text-white/30 hover:text-white/70'
                        }`}
                        title={`Line ${lineNum} (Click or drag to select)`}
                      >
                        {lineNum}
                      </div>

                      {/* Code Content */}
                      <div
                        class="pl-3 pr-4 flex-1 whitespace-pre overflow-x-visible font-mono text-[12px] text-white/90"
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
        <div class="px-3 py-2 bg-white/[0.02] border-t border-white/10 flex items-center justify-between shrink-0 select-none">
          <span class="text-[11px] text-white/40 truncate">
            Drag gutter to pick range • Click line to toggle
          </span>

          <button
            type="button"
            onClick={() => {
              if (props.file) {
                props.onInsertTag(props.file.relative_path, props.tagStartLine, props.tagEndLine);
              }
            }}
            class="px-3 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded font-medium flex items-center gap-1.5 transition-colors shadow-sm active:scale-95 ml-2 shrink-0"
          >
            <Sparkles class="w-3.5 h-3.5" />
            <span>Insert Tag</span>
          </button>
        </div>
      </aside>
    </Show>
  );
};
