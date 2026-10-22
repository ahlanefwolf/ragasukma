import { Component, createMemo, createSignal, For, Show } from 'solid-js';
import { diffLines } from 'diff';
import {
  FileCode,
  ChevronDown,
  ExternalLink,
  Bot,
  AlertCircle,
  Copy,
  Check
} from 'lucide-solid';
import { highlightLine } from './highlightUtils';

function cleanToolCallId(rawId?: string | null): string {
  if (!rawId) return 'Tool Call';
  const clean = rawId.split('__thought__')[0];
  return clean || rawId;
}

interface ParsedToolCallItem {
  id: string;
  name: string;
  argsObj: Record<string, any>;
  rawArgs: string;
}

function parseToolCalls(raw?: string | null): ParsedToolCallItem[] {
  if (!raw) return [];
  try {
    const parsed = JSON.parse(raw);
    if (!Array.isArray(parsed)) return [];
    return parsed.map((item: any) => {
      let argsObj: Record<string, any> = {};
      let rawArgs = '';
      if (item?.function?.arguments) {
        rawArgs = typeof item.function.arguments === 'string' 
          ? item.function.arguments 
          : JSON.stringify(item.function.arguments);
        try {
          argsObj = typeof item.function.arguments === 'string' 
            ? JSON.parse(item.function.arguments) 
            : item.function.arguments;
        } catch {
          argsObj = {};
        }
      }
      return {
        id: item.id || '',
        name: item.function?.name || 'unknown_tool',
        argsObj,
        rawArgs,
      };
    });
  } catch {
    return [];
  }
}

function formatToolCallHeader(tc: ParsedToolCallItem): { title: string; subtitle?: string } {
  const filePath = tc.argsObj.file_path || tc.argsObj.path || tc.argsObj.target;
  const fileName = filePath ? filePath.split(/[/\\]/).pop() : '';

  switch (tc.name) {
    case 'workspace_write_file':
      return {
        title: `Writing file: ${fileName || filePath}`,
        subtitle: tc.argsObj.content ? `${tc.argsObj.content.length} chars` : undefined,
      };
    case 'workspace_read_file':
      return {
        title: `Reading file: ${fileName || filePath}`,
        subtitle: tc.argsObj.start_line ? `Lines ${tc.argsObj.start_line}-${tc.argsObj.end_line || ''}` : undefined,
      };
    case 'workspace_list_dir':
      return {
        title: `Listing directory: ${filePath || '.'}`,
      };
    case 'workspace_grep':
      return {
        title: `Searching: "${tc.argsObj.query || ''}"`,
      };
    case 'workspace_run_command':
      return {
        title: `Terminal: ${tc.argsObj.command || ''}`,
        subtitle: tc.argsObj.cwd ? `in ${tc.argsObj.cwd}` : undefined,
      };
    default:
      return {
        title: `Tool: ${tc.name}`,
        subtitle: fileName || undefined,
      };
  }
}

interface FormattedToolOutput {
  summary: string;
  isError: boolean;
  codePreview?: string;
  filename?: string;
  diffData?: {
    oldContent: string;
    newContent: string;
    filename: string;
    isNewFile: boolean;
  };
}

function formatToolOutput(content?: string | null): FormattedToolOutput {
  if (!content) return { summary: 'No output', isError: false };

  const trimmed = content.trim();

  // 1. Check for standard error string
  if (trimmed.startsWith('Tool execution error:') || trimmed.startsWith('Error eksekusi tool:') || trimmed.startsWith('Error:')) {
    return { summary: trimmed, isError: true, codePreview: trimmed };
  }

  // 2. Parse JSON output
  try {
    const parsed = JSON.parse(trimmed);

    // 2.1 File Write with Git Diff
    if (parsed.status === 'ok' && parsed.file_path && parsed.new_content !== undefined) {
      const fileName = parsed.file_path.split(/[/\\]/).pop() || parsed.file_path;
      const isNew = !!parsed.is_new_file;
      const label = isNew ? 'Created file' : 'Updated file';

      return {
        summary: `${label}: ${fileName} (${parsed.bytes_written || 0} bytes)`,
        isError: false,
        filename: fileName,
        diffData: {
          oldContent: parsed.old_content || '',
          newContent: parsed.new_content,
          filename: fileName,
          isNewFile: isNew,
        },
      };
    }

    // 2.2 List Directory
    if (parsed.status === 'ok' && Array.isArray(parsed.entries)) {
      const count = parsed.entries.length;
      const path = parsed.path || '.';
      return {
        summary: `Folder "${path}" contains ${count} items`,
        isError: false,
        codePreview: parsed.entries.map((e: any) => `${e.is_dir ? '[DIR]' : '[FILE]'} ${e.name}`).join('\n'),
      };
    }

    // 2.3 Grep Results
    if (parsed && (Array.isArray(parsed.results) || Array.isArray(parsed.matches))) {
      const list = parsed.results || parsed.matches;
      const count = parsed.matches_found || list.length;
      return {
        summary: `Found ${count} matches for "${parsed.query || ''}"`,
        isError: false,
        codePreview: list.map((r: any) => `${r.file}:${r.line}  ${r.content}`).join('\n'),
      };
    }

    // 2.4 Shell / Command Output
    if (parsed && parsed.command !== undefined && (parsed.stdout !== undefined || parsed.stderr !== undefined)) {
      const isSuccess = parsed.success !== false && (parsed.exit_code === 0 || parsed.exit_code === undefined);
      const combinedOutput = [parsed.stdout, parsed.stderr].filter(Boolean).join('\n').trim();
      return {
        summary: `${isSuccess ? 'Command succeeded' : 'Command failed'}: \`${parsed.command}\` (exit: ${parsed.exit_code ?? 0})`,
        isError: !isSuccess,
        codePreview: combinedOutput || '(No output)',
        filename: 'terminal.sh',
      };
    }

    return {
      summary: 'Structured output received',
      isError: false,
      codePreview: JSON.stringify(parsed, null, 2),
      filename: 'data.json',
    };
  } catch {
    // 3. Plain text / raw file content
    if (trimmed.startsWith('File ') && (trimmed.includes('written') || trimmed.includes('created'))) {
      return { summary: trimmed, isError: false };
    }

    // It is raw file content (e.g. HTML, TS, Rust from read_file)
    const lines = trimmed.split('\n');
    return {
      summary: `Read file content (${lines.length} lines, ${trimmed.length} chars)`,
      isError: false,
      codePreview: trimmed,
    };
  }
}

const GitDiffViewer: Component<{
  oldContent?: string;
  newContent: string;
  filename?: string;
  isNewFile?: boolean;
}> = (props) => {
  const diffParts = createMemo(() => {
    return diffLines(props.oldContent || '', props.newContent);
  });

  const stats = createMemo(() => {
    let added = 0;
    let removed = 0;
    for (const part of diffParts()) {
      const lineCount = part.count || part.value.split('\n').filter(Boolean).length;
      if (part.added) added += lineCount;
      if (part.removed) removed += lineCount;
    }
    return { added, removed };
  });

  return (
    <div class="rounded-xl border border-white/10 overflow-hidden bg-[#0d1117] font-mono text-xs my-1 shadow-md w-full max-w-full min-w-0">
      {/* Diff Header */}
      <div class="flex items-center justify-between px-3 py-1.5 bg-white/[0.04] border-b border-white/10 text-[11px]">
        <div class="flex items-center gap-2">
          <FileCode class="w-3.5 h-3.5 text-indigo-400 shrink-0" />
          <span class="text-white/90 font-semibold">{props.filename || 'Diff Changes'}</span>
          <Show when={props.isNewFile}>
            <span class="text-[9px] px-1.5 py-0.2 rounded bg-emerald-500/20 text-emerald-300 font-bold">
              NEW FILE
            </span>
          </Show>
        </div>
        <div class="flex items-center gap-1.5 text-[10px]">
          <span class="text-emerald-400 font-bold bg-emerald-500/10 px-1.5 py-0.5 rounded border border-emerald-500/20 font-mono">
            +{stats().added}
          </span>
          <Show when={!props.isNewFile}>
            <span class="text-rose-400 font-bold bg-rose-500/10 px-1.5 py-0.5 rounded border border-rose-500/20 font-mono">
              -{stats().removed}
            </span>
          </Show>
        </div>
      </div>

      {/* Diff Lines Body */}
      <div class="overflow-x-auto max-h-72 custom-scrollbar select-text py-1 leading-relaxed w-full max-w-full min-w-0">
        <For each={diffParts()}>
          {(part) => {
            const rawLines = part.value.endsWith('\n') ? part.value.slice(0, -1).split('\n') : part.value.split('\n');
            const isAdded = part.added;
            const isRemoved = part.removed;

            return (
              <For each={rawLines}>
                {(line) => (
                  <div
                    class={`flex items-start px-3 py-0.5 font-mono text-[11px] ${
                      isAdded
                        ? 'bg-emerald-500/15 text-emerald-200 border-l-2 border-emerald-400'
                        : isRemoved
                        ? 'bg-rose-500/15 text-rose-200 border-l-2 border-rose-400 line-through opacity-80'
                        : 'text-white/60 hover:bg-white/[0.02] border-l-2 border-transparent'
                    }`}
                  >
                    <span
                      class={`w-5 select-none font-bold shrink-0 ${
                        isAdded ? 'text-emerald-400' : isRemoved ? 'text-rose-400' : 'text-white/20'
                      }`}
                    >
                      {isAdded ? '+' : isRemoved ? '-' : ' '}
                    </span>
                    <span class="whitespace-pre overflow-x-visible font-mono" innerHTML={highlightLine(line, props.filename)} />
                  </div>
                )}
              </For>
            );
          }}
        </For>
      </div>
    </div>
  );
};

export const ToolCallCard: Component<{ toolCalls: string }> = (props) => {
  const [isCardCollapsed, setIsCardCollapsed] = createSignal(true);
  const [showRaw, setShowRaw] = createSignal(false);
  const [expandedArgs, setExpandedArgs] = createSignal<Set<string>>(new Set());

  const parsedCalls = createMemo(() => parseToolCalls(props.toolCalls));

  const toggleArgExpand = (id: string) => {
    const next = new Set(expandedArgs());
    if (next.has(id)) next.delete(id);
    else next.add(id);
    setExpandedArgs(next);
  };

  const headerSummary = createMemo(() => {
    const calls = parsedCalls();
    if (calls.length === 0) return 'Calling System Tool';
    if (calls.length === 1) {
      const h = formatToolCallHeader(calls[0]);
      return h.subtitle ? `${h.title} (${h.subtitle})` : h.title;
    }
    return `Running ${calls.length} Tools: ${calls.map((c) => c.name).join(', ')}`;
  });

  return (
    <div class="my-1.5 rounded-xl border border-indigo-500/20 bg-indigo-950/20 overflow-hidden shadow-sm transition-all text-xs">
      {/* Header bar: Klik untuk collapse/expand */}
      <div
        onClick={() => setIsCardCollapsed(!isCardCollapsed())}
        class="flex items-center justify-between px-3 py-2 bg-indigo-900/30 hover:bg-indigo-900/40 cursor-pointer transition-colors border-b border-indigo-500/10 select-none"
      >
        <div class="flex items-center gap-2 min-w-0">
          <Bot class="w-3.5 h-3.5 text-indigo-400 shrink-0" />
          <span class="font-medium text-indigo-200 truncate">{headerSummary()}</span>
        </div>
        <div class="flex items-center gap-1.5 text-[10px] text-indigo-300/60 shrink-0">
          <span>{isCardCollapsed() ? 'Details' : 'Collapse'}</span>
          <ChevronDown class={`w-3.5 h-3.5 transition-transform ${isCardCollapsed() ? '' : 'rotate-180'}`} />
        </div>
      </div>

      {/* Body: Visible when not collapsed */}
      <Show when={!isCardCollapsed()}>
        <div class="p-3 space-y-2.5">
          <For each={parsedCalls()}>
            {(tc) => (
              <div class="bg-black/30 rounded-lg p-2.5 border border-white/5 space-y-1.5">
                <div class="flex items-center justify-between font-mono text-[11px]">
                  <span class="text-indigo-300 font-semibold flex items-center gap-1.5">
                    <span class="w-1.5 h-1.5 rounded-full bg-indigo-400"></span>
                    {tc.name}
                  </span>
                  <span class="text-white/30 text-[10px]">{cleanToolCallId(tc.id)}</span>
                </div>

                {/* Structured arguments preview */}
                <div class="space-y-1 pt-1 font-mono text-[11px]">
                  <For each={Object.entries(tc.argsObj)}>
                    {([key, val]) => {
                      const strVal = typeof val === 'string' ? val : JSON.stringify(val, null, 2);
                      const isLong = strVal.length > 100 || strVal.includes('\n');
                      const isArgExpanded = () => expandedArgs().has(`${tc.id}_${key}`);

                      return (
                        <div class="bg-white/[0.02] p-1.5 rounded border border-white/5">
                          <div class="flex items-center justify-between text-white/50 text-[10px] mb-0.5">
                            <span class="text-indigo-400/80 font-bold">{key}:</span>
                            <Show when={isLong}>
                              <button
                                type="button"
                                onClick={() => toggleArgExpand(`${tc.id}_${key}`)}
                                class="text-indigo-300 hover:underline"
                              >
                                {isArgExpanded() ? 'Show less' : 'Show more'}
                              </button>
                            </Show>
                          </div>
                          <div
                            class={`text-white/80 whitespace-pre-wrap break-all ${
                              isLong && !isArgExpanded() ? 'max-h-16 overflow-hidden relative' : ''
                            }`}
                          >
                            {strVal}
                            <Show when={isLong && !isArgExpanded()}>
                              <div class="absolute inset-x-0 bottom-0 h-6 bg-gradient-to-t from-black/80 to-transparent pointer-events-none" />
                            </Show>
                          </div>
                        </div>
                      );
                    }}
                  </For>
                </div>
              </div>
            )}
          </For>

          <div class="flex items-center justify-between pt-1 text-[10px]">
            <button
              type="button"
              onClick={() => setShowRaw(!showRaw())}
              class="text-blue-400 hover:text-blue-200 underline"
            >
              {showRaw() ? 'Hide Raw Payload' : 'View Raw Payload JSON'}
            </button>
          </div>

          <Show when={showRaw()}>
            <pre class="text-[10px] font-mono text-white/60 bg-black/50 p-2.5 rounded overflow-x-auto max-h-40 custom-scrollbar select-all">
              {props.toolCalls}
            </pre>
          </Show>
        </div>
      </Show>
    </div>
  );
};

export const ToolResultCard: Component<{ toolCallId?: string | null; content?: string | null }> = (props) => {
  const info = createMemo(() => formatToolOutput(props.content));
  const cleanId = createMemo(() => cleanToolCallId(props.toolCallId));
  const [isCardCollapsed, setIsCardCollapsed] = createSignal(true);

  return (
    <div
      class={`my-1.5 rounded-xl border overflow-hidden shadow-sm transition-all text-xs w-full max-w-full min-w-0 ${
        info().isError
          ? 'border-rose-500/30 bg-rose-950/20'
          : 'border-emerald-500/20 bg-emerald-950/20'
      }`}
    >
      {/* Header bar */}
      <div
        onClick={() => setIsCardCollapsed(!isCardCollapsed())}
        class={`flex items-center justify-between px-3 py-2 cursor-pointer transition-colors border-b select-none ${
          info().isError
            ? 'bg-rose-900/30 hover:bg-rose-900/40 border-rose-500/20'
            : 'bg-emerald-900/30 hover:bg-emerald-900/40 border-emerald-500/20'
        }`}
      >
        <div class="flex items-center gap-2 min-w-0">
          <Show
            when={!info().isError}
            fallback={<AlertCircle class="w-3.5 h-3.5 text-rose-400 shrink-0" />}
          >
            <Check class="w-3.5 h-3.5 text-emerald-400 shrink-0" />
          </Show>
          <span
            class={`font-medium truncate ${
              info().isError ? 'text-rose-200' : 'text-emerald-200'
            }`}
          >
            {info().summary}
          </span>
        </div>
        <div class="flex items-center gap-1.5 text-[10px] opacity-60 shrink-0">
          <span class="font-mono text-[9px] mr-1">{cleanId()}</span>
          <span>{isCardCollapsed() ? 'Expand' : 'Collapse'}</span>
          <ChevronDown class={`w-3.5 h-3.5 transition-transform ${isCardCollapsed() ? '' : 'rotate-180'}`} />
        </div>
      </div>

      {/* Body: Show Diff / Code Preview when not collapsed */}
      <Show when={!isCardCollapsed()}>
        <div class="p-3 space-y-2">
          {/* 1. Git Diff View (for file write operations) */}
          <Show when={info().diffData}>
            <GitDiffViewer
              oldContent={info().diffData?.oldContent}
              newContent={info().diffData?.newContent || ''}
              filename={info().diffData?.filename}
              isNewFile={info().diffData?.isNewFile}
            />
          </Show>

          {/* 2. Text/Code Preview Standar */}
          <Show when={info().codePreview && !info().diffData}>
            <div class="rounded-lg bg-black/40 border border-white/5 overflow-hidden">
              <div class="flex items-center justify-between px-2.5 py-1 bg-white/[0.02] border-b border-white/5 text-[10px] text-white/40">
                <span>{info().filename || 'Output Data'}</span>
              </div>
              <div class="p-2.5 font-mono text-[11px] text-white/80 overflow-x-auto max-h-60 custom-scrollbar whitespace-pre leading-relaxed select-text">
                <For each={info().codePreview?.split('\n') || []}>
                  {(line) => (
                    <div class="leading-relaxed" innerHTML={highlightLine(line, info().filename)} />
                  )}
                </For>
              </div>
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
};
