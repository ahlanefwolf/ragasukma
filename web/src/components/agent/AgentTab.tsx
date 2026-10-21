import { Component, createSignal, onMount, For, Show, createMemo, createEffect, onCleanup } from 'solid-js';
import { 
  FolderGit2, 
  Layers, 
  MessageSquare, 
  Sparkles, 
  Server, 
  Plus, 
  Trash2,
  Folder,
  FolderPlus,
  X,
  Check,
  CheckCircle2,
  AlertCircle,
  ChevronRight,
  Terminal,
  Send,
  Loader2,
  Bot,
  Brain,
  Wrench,
  Settings,
  ArrowLeft,
  RefreshCw,
  ChevronDown,
  ChevronUp,
  Cpu,
  Files,
  FileCode,
  FileText,
  Copy,
  Search,
  Hash,
  AtSign,
  FolderOpen,
  ExternalLink,
  FoldVertical,
  UnfoldVertical
} from 'lucide-solid';
import { 
  agentService, 
  Project, 
  PromptSkill, 
  McpServerConfig, 
  AgentSession, 
  AgentMessage, 
  WorkspaceFileItem, 
  parseProjectFolders,
  antigravityService,
  codexService,
  CodexModelItem,
  configService,
  API_BASE
} from '../../services';

// Git Diff Computation
import { diffLines } from 'diff';

// Prism.js Syntax Highlighting
import Prism from 'prismjs';
import 'prismjs/themes/prism-tomorrow.css';
import 'prismjs/components/prism-typescript';
import 'prismjs/components/prism-jsx';
import 'prismjs/components/prism-tsx';
import 'prismjs/components/prism-rust';
import 'prismjs/components/prism-python';
import 'prismjs/components/prism-json';
import 'prismjs/components/prism-bash';
import 'prismjs/components/prism-toml';
import 'prismjs/components/prism-css';
import 'prismjs/components/prism-markdown';
import 'prismjs/components/prism-sql';
import 'prismjs/components/prism-yaml';

function escapeCodeHtml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function highlightCode(code: string, filename: string): string[] {
  const ext = filename.split('.').pop()?.toLowerCase() || '';
  const langMap: { [key: string]: string } = {
    ts: 'typescript',
    tsx: 'tsx',
    js: 'javascript',
    jsx: 'jsx',
    rs: 'rust',
    py: 'python',
    json: 'json',
    css: 'css',
    scss: 'css',
    html: 'html',
    htm: 'html',
    sh: 'bash',
    bash: 'bash',
    zsh: 'bash',
    md: 'markdown',
    toml: 'toml',
    yaml: 'yaml',
    yml: 'yaml',
    sql: 'sql',
  };
  const lang = langMap[ext] || 'javascript';
  const grammar = Prism.languages[lang] || Prism.languages.javascript || Prism.languages.markup;

  try {
    const highlighted = Prism.highlight(code, grammar, lang);
    return highlighted.split('\n');
  } catch {
    return code.split('\n').map((l) => escapeCodeHtml(l));
  }
}
import { MarkdownView } from './MarkdownView';

interface AgentTabProps {
  showToast: (msg: string) => void;
}

// ============================================================================
// Tool Calls & Output Formatting Helpers
// ============================================================================

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
        id: cleanToolCallId(item?.id),
        name: item?.function?.name || 'unknown_tool',
        argsObj,
        rawArgs,
      };
    });
  } catch {
    return [];
  }
}

function formatToolCallHeader(tc: ParsedToolCallItem): { title: string; subtitle?: string } {
  const filePath = tc.argsObj.file_path || tc.argsObj.path || '';
  const fileName = filePath ? filePath.split('/').pop() : '';

  switch (tc.name) {
    case 'workspace_write_file':
      return {
        title: `Menulis file: ${fileName || filePath}`,
        subtitle: tc.argsObj.content ? `${tc.argsObj.content.length} karakter` : undefined,
      };
    case 'workspace_read_file':
      return {
        title: `Membaca file: ${fileName || filePath}`,
        subtitle: tc.argsObj.start_line ? `Baris ${tc.argsObj.start_line}-${tc.argsObj.end_line || ''}` : undefined,
      };
    case 'workspace_list_dir':
      return {
        title: `Memeriksa folder: ${filePath || '.'}`,
      };
    case 'workspace_grep':
      return {
        title: `Mencari teks: "${tc.argsObj.query || ''}"`,
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

function formatToolOutput(rawContent?: string | null): FormattedToolOutput {
  if (!rawContent) return { summary: 'Selesai tanpa output', isError: false };

  const trimmed = rawContent.trim();

  // 1. Error check
  if (trimmed.startsWith('Error') || trimmed.toLowerCase().includes('akses ditolak')) {
    return { summary: trimmed, isError: true };
  }

  // 2. Try JSON
  try {
    const parsed = JSON.parse(rawContent);

    if (parsed && parsed.error) {
      return { summary: `Gagal: ${parsed.error}`, isError: true };
    }

    // Git Diff for write_file
    if (parsed && (parsed.is_new_file !== undefined || parsed.new_content !== undefined)) {
      const fileName = parsed.file_path ? parsed.file_path.split('/').pop() : 'file';
      const isNew = !!parsed.is_new_file;
      const label = isNew ? 'File baru dibuat' : 'File dimodifikasi';
      return {
        summary: `✓ ${label}: ${fileName}`,
        isError: false,
        filename: fileName,
        diffData: {
          oldContent: parsed.old_content || '',
          newContent: parsed.new_content || '',
          filename: fileName,
          isNewFile: isNew,
        },
      };
    }

    if (parsed && Array.isArray(parsed.entries)) {
      const count = parsed.entries.length;
      const path = parsed.path || '';
      return {
        summary: count === 0 ? `Folder kosong di ${path}` : `Ditemukan ${count} item di ${path}`,
        isError: false,
        codePreview: parsed.entries.map((e: any) => `${e.is_dir ? '[dir] ' : '[file]'} ${e.name}`).join('\n'),
      };
    }

    if (parsed && (Array.isArray(parsed.results) || Array.isArray(parsed.matches))) {
      const list = parsed.results || parsed.matches;
      const count = parsed.matches_found || list.length;
      return {
        summary: `Ditemukan ${count} kecocokan untuk "${parsed.query || ''}"`,
        isError: false,
        codePreview: list.map((r: any) => `${r.file}:${r.line}  ${r.content}`).join('\n'),
      };
    }

    return {
      summary: 'Data output diterima',
      isError: false,
      codePreview: JSON.stringify(parsed, null, 2),
      filename: 'data.json',
    };
  } catch {
    // 3. Plain text / raw file content
    if (trimmed.startsWith('File ') && trimmed.includes('berhasil ditulis')) {
      return { summary: trimmed, isError: false };
    }

    // It is raw file content (e.g. HTML, TS, Rust from read_file)
    const lines = trimmed.split('\n');
    return {
      summary: `Berhasil membaca isi file (${lines.length} baris, ${trimmed.length} karakter)`,
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
    <div class="rounded-xl border border-white/10 overflow-hidden bg-[#0d1117] font-mono text-xs my-1 shadow-md">
      {/* Diff Header */}
      <div class="flex items-center justify-between px-3 py-1.5 bg-white/[0.04] border-b border-white/10 text-[11px]">
        <div class="flex items-center gap-2">
          <FileCode class="w-3.5 h-3.5 text-indigo-400 shrink-0" />
          <span class="text-white/90 font-semibold">{props.filename || 'Diff Perubahan'}</span>
          <Show when={props.isNewFile}>
            <span class="text-[9px] px-1.5 py-0.2 rounded bg-emerald-500/20 text-emerald-300 font-bold">
              FILE BARU
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
      <div class="overflow-x-auto max-h-72 custom-scrollbar select-text py-1 leading-relaxed">
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
                    <span class="whitespace-pre overflow-x-visible">{line || ' '}</span>
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

const ToolCallCard: Component<{ toolCalls: string }> = (props) => {
  const [isCardCollapsed, setIsCardCollapsed] = createSignal(true);
  const [showRaw, setShowRaw] = createSignal(false);
  const [expandedArgs, setExpandedArgs] = createSignal<Set<string>>(new Set());

  const parsedCalls = createMemo(() => parseToolCalls(props.toolCalls));

  const toggleArgExpand = (key: string) => {
    const next = new Set(expandedArgs());
    if (next.has(key)) next.delete(key);
    else next.add(key);
    setExpandedArgs(next);
  };

  const headerSummary = createMemo(() => {
    const calls = parsedCalls();
    if (calls.length === 0) return 'Memanggil Tool Workspace';
    if (calls.length === 1) {
      const h = formatToolCallHeader(calls[0]);
      return `${h.title}${h.subtitle ? ` (${h.subtitle})` : ''}`;
    }
    return `Memanggil ${calls.length} tool workspace (${calls.map(c => c.name).join(', ')})`;
  });

  return (
    <div class="w-full max-w-2xl mb-2 rounded-xl bg-blue-950/20 border border-blue-500/20 text-xs text-blue-200/90 overflow-hidden shadow-sm">
      <div 
        class="flex items-center justify-between px-3 py-2 bg-blue-500/10 border-b border-blue-500/15 cursor-pointer select-none hover:bg-blue-500/15 transition-colors"
        onClick={() => setIsCardCollapsed(!isCardCollapsed())}
      >
        <div class="flex items-center gap-2 font-semibold text-[11px] text-blue-300 min-w-0 flex-1">
          <Terminal class="w-3.5 h-3.5 text-blue-400 shrink-0" />
          <span class="truncate">{headerSummary()}</span>
        </div>
        <div class="flex items-center gap-1.5 ml-2 shrink-0" onClick={(e) => e.stopPropagation()}>
          <button
            type="button"
            onClick={() => setIsCardCollapsed(!isCardCollapsed())}
            class="text-[10px] text-blue-400 hover:text-blue-200 flex items-center gap-0.5 px-1.5 py-0.5 rounded hover:bg-blue-500/10 transition-colors"
          >
            <span>{isCardCollapsed() ? 'Detail' : 'Tutup'}</span>
            <ChevronDown class={`w-3 h-3 transition-transform ${isCardCollapsed() ? '' : 'rotate-180'}`} />
          </button>
        </div>
      </div>

      <Show when={!isCardCollapsed()}>
        <div class="p-3 space-y-2">
          <For each={parsedCalls()}>
            {(tc) => (
              <div class="bg-black/30 rounded-lg p-2.5 border border-white/5 space-y-1.5">
                <div class="flex items-center justify-between gap-2">
                  <div class="flex items-center gap-2">
                    <span class="font-mono font-semibold text-white text-[11px] px-2 py-0.5 rounded bg-blue-500/20 border border-blue-400/30">
                      {tc.name}
                    </span>
                    <span class="text-[10px] text-white/40 font-mono truncate max-w-xs">
                      {cleanToolCallId(tc.id)}
                    </span>
                  </div>
                </div>

                {/* Arguments breakdown with long text truncating */}
                <div class="text-[11px] text-white/70 font-mono space-y-1 pl-1">
                  <For each={Object.entries(tc.argsObj)}>
                    {([key, val]) => {
                      const strVal = typeof val === 'string' ? val : JSON.stringify(val, null, 2);
                      const isLong = strVal.length > 120;
                      const isArgExpanded = expandedArgs().has(`${tc.id}_${key}`);

                      return (
                        <div class="flex flex-col gap-0.5">
                          <div class="flex items-start gap-1.5">
                            <span class="text-blue-300 font-semibold">{key}:</span>
                            <span class="text-white/80 break-all select-all font-mono">
                              {!isLong || isArgExpanded
                                ? strVal
                                : `${strVal.slice(0, 120)}...`}
                            </span>
                          </div>
                          <Show when={isLong}>
                            <button
                              type="button"
                              onClick={() => toggleArgExpand(`${tc.id}_${key}`)}
                              class="text-[10px] text-blue-400 hover:text-blue-200 self-start pl-2 underline"
                            >
                              {isArgExpanded ? '« Ringkas parameter' : `» Lihat isi lengkap (${strVal.length} karakter)`}
                            </button>
                          </Show>
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
              {showRaw() ? 'Sembunyikan Raw Payload' : 'Lihat Raw Payload JSON'}
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

const ToolResultCard: Component<{ toolCallId?: string | null; content?: string | null }> = (props) => {
  const info = createMemo(() => formatToolOutput(props.content));
  const cleanId = createMemo(() => cleanToolCallId(props.toolCallId));
  // Default open if diff data is present so user sees git changes immediately
  const [isCardCollapsed, setIsCardCollapsed] = createSignal(!info().diffData);

  return (
    <div class={`w-full max-w-2xl mb-2 rounded-xl text-xs overflow-hidden shadow-sm border ${
      info().isError 
        ? 'bg-rose-950/20 border-rose-500/25 text-rose-200/90' 
        : 'bg-emerald-950/20 border-emerald-500/20 text-emerald-200/90'
    }`}>
      <div 
        class={`flex items-center justify-between px-3 py-2 border-b cursor-pointer select-none transition-colors ${
          info().isError 
            ? 'bg-rose-500/10 border-rose-500/20 hover:bg-rose-500/15' 
            : 'bg-emerald-500/10 border-emerald-500/15 hover:bg-emerald-500/15'
        }`}
        onClick={() => setIsCardCollapsed(!isCardCollapsed())}
      >
        <div class="flex items-center gap-2 font-semibold text-[11px] min-w-0 flex-1">
          <Show when={info().isError} fallback={<CheckCircle2 class="w-3.5 h-3.5 text-emerald-400 shrink-0" />}>
            <AlertCircle class="w-3.5 h-3.5 text-rose-400 shrink-0" />
          </Show>
          <span class="truncate">{info().summary}</span>
        </div>
        <div class="flex items-center gap-1.5 ml-2 shrink-0">
          <span class="text-[10px] px-1.5 py-0.2 rounded bg-black/30 font-mono text-white/40">
            {cleanId()}
          </span>
          <button
            type="button"
            class="text-[10px] flex items-center gap-0.5 px-1 py-0.5 rounded hover:bg-white/10 transition-colors"
          >
            <span>{isCardCollapsed() ? 'Detail' : 'Tutup'}</span>
            <ChevronDown class={`w-3 h-3 transition-transform ${isCardCollapsed() ? '' : 'rotate-180'}`} />
          </button>
        </div>
      </div>

      <Show when={!isCardCollapsed()}>
        <div class="p-3 space-y-2">
          {/* Git Diff Viewer for file modifications / creations */}
          <Show when={info().diffData}>
            <GitDiffViewer
              oldContent={info().diffData!.oldContent}
              newContent={info().diffData!.newContent}
              filename={info().diffData!.filename}
              isNewFile={info().diffData!.isNewFile}
            />
          </Show>

          {/* Standard Code / Details Output */}
          <Show when={!info().diffData && info().codePreview}>
            <div class="mt-2 pt-2 border-t border-white/10 space-y-1">
              <div class="flex items-center justify-between text-[10px] text-white/40">
                <span>Detail Output Data:</span>
                <button
                  type="button"
                  onClick={() => {
                    navigator.clipboard.writeText(info().codePreview || '');
                  }}
                  class="hover:underline text-indigo-400"
                >
                  Salin Konten
                </button>
              </div>
              <pre class="text-[11px] font-mono text-emerald-300 bg-black/60 p-3 rounded-lg overflow-x-auto max-h-56 custom-scrollbar select-all whitespace-pre leading-relaxed border border-white/5">
                {info().codePreview}
              </pre>
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
};

// ============================================================================
// Workspace File Tree Components & Types
// ============================================================================

interface FileTreeNode {
  name: string;
  path: string;
  is_dir: boolean;
  item?: WorkspaceFileItem;
  children: FileTreeNode[];
}

const FileTreeNodeRow: Component<{
  node: FileTreeNode;
  depth: number;
  isExpanded: (path: string) => boolean;
  onToggleDir: (path: string) => void;
  onSelectFile: (file: WorkspaceFileItem) => void;
  onOpenRange: (file: WorkspaceFileItem) => void;
  onCopyTag: (path: string) => void;
  onTagFolder: (path: string) => void;
}> = (props) => {
  return (
    <div class="select-none">
      <Show
        when={props.node.is_dir}
        fallback={
          <div
            class="group flex items-center justify-between py-1 px-1.5 rounded-lg hover:bg-white/5 cursor-pointer transition-colors text-xs font-mono"
            style={{ "padding-left": `${props.depth * 14 + 6}px` }}
            onClick={() => props.node.item && props.onSelectFile(props.node.item)}
            title={`Klik untuk tag @${props.node.path}`}
          >
            <div class="flex items-center gap-1.5 min-w-0 flex-1">
              <FileCode class="w-3.5 h-3.5 text-indigo-400 shrink-0" />
              <span class="truncate text-white/90">{props.node.name}</span>
              <Show when={props.node.item?.line_count !== undefined && props.node.item?.line_count !== null}>
                <span class="text-[9px] text-white/30 shrink-0">({props.node.item?.line_count}L)</span>
              </Show>
            </div>
            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity ml-1" onClick={(e) => e.stopPropagation()}>
              <button
                type="button"
                onClick={() => props.node.item && props.onOpenRange(props.node.item)}
                class="px-1 py-0.5 hover:bg-indigo-500/20 text-white/50 hover:text-indigo-300 rounded text-[9px] font-mono flex items-center gap-0.5 transition-colors border border-white/10"
                title="Pilih rentang baris (@file:start-end)"
              >
                <Hash class="w-2.5 h-2.5 text-indigo-400" />
              </button>
              <button
                type="button"
                onClick={() => props.onCopyTag(props.node.path)}
                class="p-0.5 hover:bg-white/10 text-white/40 hover:text-white rounded transition-colors"
                title="Salin tag @file"
              >
                <Copy class="w-3 h-3" />
              </button>
            </div>
          </div>
        }
      >
        <div
          class="group flex items-center justify-between py-1 px-1.5 rounded-lg hover:bg-white/5 cursor-pointer transition-colors text-xs font-mono"
          style={{ "padding-left": `${props.depth * 14 + 6}px` }}
          onClick={() => props.onToggleDir(props.node.path)}
        >
          <div class="flex items-center gap-1.5 min-w-0 flex-1">
            <Show when={props.isExpanded(props.node.path)} fallback={<ChevronRight class="w-3 h-3 text-white/40 shrink-0" />}>
              <ChevronDown class="w-3 h-3 text-indigo-400 shrink-0" />
            </Show>
            <Show when={props.isExpanded(props.node.path)} fallback={<Folder class="w-3.5 h-3.5 text-amber-400 shrink-0" />}>
              <FolderOpen class="w-3.5 h-3.5 text-amber-300 shrink-0" />
            </Show>
            <span class="truncate font-semibold text-white/90">{props.node.name}</span>
            <span class="text-[9px] text-white/30 shrink-0">({props.node.children.length})</span>
          </div>
          <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity ml-1" onClick={(e) => e.stopPropagation()}>
            <button
              type="button"
              onClick={() => props.onTagFolder(props.node.path)}
              class="px-1 py-0.5 hover:bg-indigo-500/20 text-white/50 hover:text-indigo-300 rounded text-[9px] font-mono flex items-center gap-0.5 transition-colors border border-white/10"
              title="Tag folder ini (@folder)"
            >
              <AtSign class="w-2.5 h-2.5 text-indigo-400" />
              <span>tag</span>
            </button>
            <button
              type="button"
              onClick={() => props.onCopyTag(props.node.path)}
              class="p-0.5 hover:bg-white/10 text-white/40 hover:text-white rounded transition-colors"
              title="Salin tag @folder"
            >
              <Copy class="w-3 h-3" />
            </button>
          </div>
        </div>

        <Show when={props.isExpanded(props.node.path)}>
          <div class="space-y-0.5">
            <For each={props.node.children}>
              {(child) => (
                <FileTreeNodeRow
                  node={child}
                  depth={props.depth + 1}
                  isExpanded={props.isExpanded}
                  onToggleDir={props.onToggleDir}
                  onSelectFile={props.onSelectFile}
                  onOpenRange={props.onOpenRange}
                  onCopyTag={props.onCopyTag}
                  onTagFolder={props.onTagFolder}
                />
              )}
            </For>
          </div>
        </Show>
      </Show>
    </div>
  );
};

export const AgentTab: Component<AgentTabProps> = (props) => {
  // Navigation & Data State
  const [activeSubTab, setActiveSubTab] = createSignal<'projects' | 'skills' | 'mcp'>('projects');
  const [loading, setLoading] = createSignal(false);
  const [projects, setProjects] = createSignal<Project[]>([]);
  const [skills, setSkills] = createSignal<PromptSkill[]>([]);
  const [mcpServers, setMcpServers] = createSignal<McpServerConfig[]>([]);

  // Selected State
  const [selectedProjectId, setSelectedProjectId] = createSignal<string | null>(null);
  const [sessions, setSessions] = createSignal<AgentSession[]>([]);
  const [selectedSessionId, setSelectedSessionId] = createSignal<string | null>(null);
  const [messages, setMessages] = createSignal<AgentMessage[]>([]);
  const [loadingMessages, setLoadingMessages] = createSignal(false);
  const [showWorkspaceSettings, setShowWorkspaceSettings] = createSignal(false);

  // New Message Input State
  const [userInput, setUserInput] = createSignal('');
  const [sendingMessage, setSendingMessage] = createSignal(false);

  // Message Collapse State (Feature 1: Hide / Sembunyikan generative output)
  const [collapsedMessages, setCollapsedMessages] = createSignal<Set<string>>(new Set());
  const toggleCollapseMessage = (id: string) => {
    const next = new Set(collapsedMessages());
    if (next.has(id)) {
      next.delete(id);
    } else {
      next.add(id);
    }
    setCollapsedMessages(next);
  };
  const isMessageCollapsed = (id: string) => collapsedMessages().has(id);
  const countMessageLines = (content?: string) => (content ? content.split('\n').length : 0);

  // Project Files & @tagging State (Feature 2: File Explorer & Tagging @file:baris)
  const [sidebarTab, setSidebarTab] = createSignal<'sessions' | 'files'>('sessions');
  const [workspaceFiles, setWorkspaceFiles] = createSignal<WorkspaceFileItem[]>([]);
  const [loadingFiles, setLoadingFiles] = createSignal(false);
  const [fileSearch, setFileSearch] = createSignal('');

  // Tree View Expand/Collapse state
  const [expandedDirs, setExpandedDirs] = createSignal<{ [path: string]: boolean }>({});

  const toggleDir = (dirPath: string) => {
    setExpandedDirs((prev) => ({
      ...prev,
      [dirPath]: prev[dirPath] !== undefined ? !prev[dirPath] : false,
    }));
  };

  const expandAllDirs = () => {
    const next: { [path: string]: boolean } = {};
    const walk = (nodes: FileTreeNode[]) => {
      for (const n of nodes) {
        if (n.is_dir) {
          next[n.path] = true;
          walk(n.children);
        }
      }
    };
    walk(fileTree());
    setExpandedDirs(next);
  };

  const collapseAllDirs = () => {
    const next: { [path: string]: boolean } = {};
    const walk = (nodes: FileTreeNode[]) => {
      for (const n of nodes) {
        if (n.is_dir) {
          next[n.path] = false;
          walk(n.children);
        }
      }
    };
    walk(fileTree());
    setExpandedDirs(next);
  };

  const isDirExpanded = (dirPath: string) => {
    // Only auto-expand during active search filtering
    if (fileSearch().trim()) return true;
    // Default initial state is completely collapsed
    return expandedDirs()[dirPath] === true;
  };

  // Autocomplete @tagging in chat input (both files & folders)
  const [autocompleteOpen, setAutocompleteOpen] = createSignal(false);
  const [autocompleteQuery, setAutocompleteQuery] = createSignal('');
  const [autocompleteIndex, setAutocompleteIndex] = createSignal(0);
  const [autocompleteCursorPos, setAutocompleteCursorPos] = createSignal(0);

  const autocompleteSuggestions = createMemo(() => {
    if (!autocompleteOpen()) return [];
    const q = autocompleteQuery().toLowerCase();
    const all = workspaceFiles();
    if (!q) {
      return all.slice(0, 10);
    }
    return all
      .filter(
        (f) =>
          f.relative_path.toLowerCase().includes(q) ||
          f.name.toLowerCase().includes(q)
      )
      .slice(0, 10);
  });

  const selectAutocompleteFile = (file: WorkspaceFileItem) => {
    const current = userInput();
    const cursorPos = autocompleteCursorPos();
    const textBefore = current.slice(0, cursorPos);
    const textAfter = current.slice(cursorPos);

    const updatedBefore = textBefore.replace(/@([a-zA-Z0-9_\-\./\\]*)$/, `@${file.relative_path} `);
    const newVal = updatedBefore + textAfter;

    setUserInput(newVal);
    setAutocompleteOpen(false);

    setTimeout(() => {
      if (chatInputRef) {
        chatInputRef.focus();
        const newPos = updatedBefore.length;
        chatInputRef.setSelectionRange(newPos, newPos);
      }
    }, 20);
  };

  const handleChatInput = (e: InputEvent & { currentTarget: HTMLInputElement }) => {
    const target = e.currentTarget;
    const val = target.value;
    setUserInput(val);

    const cursorPos = target.selectionStart || 0;
    setAutocompleteCursorPos(cursorPos);

    const textBefore = val.slice(0, cursorPos);
    const match = textBefore.match(/@([a-zA-Z0-9_\-\./\\]*)$/);

    if (match) {
      setAutocompleteQuery(match[1]);
      setAutocompleteIndex(0);
      setAutocompleteOpen(true);
    } else {
      setAutocompleteOpen(false);
    }
  };

  const handleChatKeyDown = (e: KeyboardEvent & { currentTarget: HTMLInputElement }) => {
    if (autocompleteOpen() && autocompleteSuggestions().length > 0) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setAutocompleteIndex((prev) => (prev + 1) % autocompleteSuggestions().length);
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        setAutocompleteIndex((prev) => (prev - 1 + autocompleteSuggestions().length) % autocompleteSuggestions().length);
        return;
      }
      if (e.key === 'Enter' || e.key === 'Tab') {
        e.preventDefault();
        const item = autocompleteSuggestions()[autocompleteIndex()];
        if (item) {
          selectAutocompleteFile(item);
        }
        return;
      }
      if (e.key === 'Escape') {
        e.preventDefault();
        setAutocompleteOpen(false);
        return;
      }
    }

    if (e.key === 'Enter') {
      handleSendMessage();
    }
  };

  // Line Range Picker Dialog State
  const [selectedFileForTag, setSelectedFileForTag] = createSignal<WorkspaceFileItem | null>(null);
  const [tagStartLine, setTagStartLine] = createSignal('');
  const [tagEndLine, setTagEndLine] = createSignal('');
  const [filePreview, setFilePreview] = createSignal<string | null>(null);
  const [loadingFilePreview, setLoadingFilePreview] = createSignal(false);

  // SSE Real-time Streaming State
  let activeEventSource: EventSource | null = null;
  const [currentStepInfo, setCurrentStepInfo] = createSignal<string | null>(null);

  // Modals
  const [showNewProjectModal, setShowNewProjectModal] = createSignal(false);
  const [showNewSkillModal, setShowNewSkillModal] = createSignal(false);

  // New Project Form
  const [projName, setProjName] = createSignal('');
  const [projDesc, setProjDesc] = createSignal('');
  const [projInstructions, setProjInstructions] = createSignal('');
  const [projFolders, setProjFolders] = createSignal<string[]>([]);
  const [folderInput, setFolderInput] = createSignal('');
  const [validatingFolder, setValidatingFolder] = createSignal(false);
  const [folderValidationMsg, setFolderValidationMsg] = createSignal<{ valid: boolean; text: string } | null>(null);

  // Dynamic Models & Active Model per turn/message
  const [activeSessionModel, setActiveSessionModel] = createSignal('');
  const [availableModels, setAvailableModels] = createSignal<Array<{ id: string; label: string }>>([]);
  const [defaultBrainModel, setDefaultBrainModel] = createSignal<string>('');
  const [isRefreshingModels, setIsRefreshingModels] = createSignal(false);

  // Element Refs for Auto-scroll & Focus
  let messagesContainerRef: HTMLDivElement | undefined;
  let chatInputRef: HTMLInputElement | undefined;
  let isUserScrolledUp = false;

  const handleMessagesScroll = () => {
    if (!messagesContainerRef) return;
    const { scrollTop, scrollHeight, clientHeight } = messagesContainerRef;
    isUserScrolledUp = scrollHeight - (scrollTop + clientHeight) > 120;
  };

  const scrollToBottom = (smooth = true, force = false) => {
    if (!messagesContainerRef) return;
    if (isUserScrolledUp && !force) return;

    messagesContainerRef.scrollTo({
      top: messagesContainerRef.scrollHeight,
      behavior: smooth ? 'smooth' : 'auto',
    });
  };

  createEffect(() => {
    // Automatically keep chat scrolled to the bottom on new messages or state changes
    messages();
    if (sendingMessage()) {
      isUserScrolledUp = false;
      setTimeout(() => scrollToBottom(true, true), 30);
    } else {
      setTimeout(() => scrollToBottom(true, false), 40);
    }
  });

  const refreshDynamicModels = async () => {
    setIsRefreshingModels(true);
    try {
      const [dynamicAg, codexModels, cfg] = await Promise.all([
        antigravityService.getDynamicModels().catch(() => [] as Array<{ model_id: string; display_name: string }>),
        codexService.getModels().catch(() => [] as CodexModelItem[]),
        configService.getConfig().catch(() => null),
      ]);

      const list: Array<{ id: string; label: string }> = [];

      if (dynamicAg && dynamicAg.length > 0) {
        dynamicAg.forEach((m: { model_id: string; display_name?: string }) => {
          const fullId = `ag/${m.model_id}`;
          list.push({
            id: fullId,
            label: m.display_name ? `${m.display_name} (${fullId})` : fullId,
          });
        });
      }

      if (codexModels && codexModels.length > 0) {
        codexModels.forEach((m: CodexModelItem) => {
          list.push({
            id: m.id,
            label: m.display_name ? `${m.display_name} (${m.id})` : m.id,
          });
        });
      }

      const activeBrain = cfg?.llm_model?.trim() || '';
      setDefaultBrainModel(activeBrain);

      if (activeBrain && !list.some((m) => m.id === activeBrain)) {
        list.unshift({ id: activeBrain, label: `${activeBrain} (Brain Persona)` });
      }

      if (list.length === 0) {
        list.push(
          { id: 'ag/gemini-2.5-flash', label: 'Gemini 2.5 Flash (ag/gemini-2.5-flash)' },
          { id: 'ag/claude-3-7-sonnet', label: 'Claude 3.7 Sonnet (ag/claude-3-7-sonnet)' },
          { id: 'cx/gpt-4o', label: 'GPT-4o (cx/gpt-4o)' }
        );
      }

      setAvailableModels(list);

      // Default active model if not manually chosen
      if (!activeSessionModel()) {
        setActiveSessionModel(activeBrain || (list.length > 0 ? list[0].id : ''));
      }
    } catch (e) {
      console.error('Failed to load dynamic models for agent workspace', e);
    } finally {
      setIsRefreshingModels(false);
    }
  };

  // New Prompt Skill Form
  const [skillName, setSkillName] = createSignal('');
  const [skillSlug, setSkillSlug] = createSignal('');
  const [skillDesc, setSkillDesc] = createSignal('');
  const [skillSystemPrompt, setSkillSystemPrompt] = createSignal('');
  const [skillTags, setSkillTags] = createSignal('');

  // Selected Project Helper
  const currentProject = createMemo(() => {
    return projects().find((p) => p.id === selectedProjectId()) || null;
  });

  // Selected Session Helper
  const currentSession = createMemo(() => {
    return sessions().find((s) => s.id === selectedSessionId()) || null;
  });

  // Per-Session Multi-task Tracking
  const [runningSessionIds, setRunningSessionIds] = createSignal<Set<string>>(new Set());

  const markSessionRunning = (sessionId: string, running: boolean) => {
    const next = new Set(runningSessionIds());
    if (running) {
      next.add(sessionId);
    } else {
      next.delete(sessionId);
    }
    setRunningSessionIds(next);
  };

  const isSessionRunning = (sessionId?: string | null) => {
    if (!sessionId) return false;
    if (runningSessionIds().has(sessionId)) return true;
    const sess = sessions().find((s) => s.id === sessionId);
    return sess?.status === 'running';
  };

  const isCurrentSessionBusy = () => {
    const currentId = selectedSessionId();
    if (!currentId) return false;
    return isSessionRunning(currentId);
  };

  // Background multi-task sync: if any session in current project is running, periodic sync keeps sidebar updated
  createEffect(() => {
    const hasRunning = sessions().some((s) => s.status === 'running') || runningSessionIds().size > 0;
    if (!hasRunning) return;

    const interval = setInterval(async () => {
      const projId = selectedProjectId();
      if (!projId) return;

      const updated = await agentService.getSessions(projId);
      setSessions(updated);

      // Clean up runningSessionIds that completed in background
      setRunningSessionIds((prev) => {
        const next = new Set(prev);
        let completedAny = false;
        for (const id of prev) {
          const match = updated.find((s) => s.id === id);
          if (match && match.status !== 'running') {
            next.delete(id);
            completedAny = true;
          }
        }
        if (completedAny && projId) {
          loadProjectFiles(projId);
        }
        return next;
      });
    }, 2500);

    onCleanup(() => clearInterval(interval));
  });

  const connectSessionSse = (sessionId: string) => {
    if (activeEventSource) {
      activeEventSource.close();
      activeEventSource = null;
    }

    try {
      const url = `${API_BASE}/api/agent/sessions/${sessionId}/events`;
      const es = new EventSource(url);
      activeEventSource = es;

      es.onmessage = (event) => {
        try {
          if (!event.data || event.data.trim() === '') return;
          const payload = JSON.parse(event.data);

          if (payload.type === 'message') {
            const newMsg: AgentMessage = payload.data;
            setMessages((prev) => {
              const idx = prev.findIndex((m) => m.id === newMsg.id);
              if (idx !== -1) {
                const copy = [...prev];
                copy[idx] = newMsg;
                return copy;
              }
              return [...prev, newMsg];
            });
            setTimeout(() => scrollToBottom(true, false), 40);

            // Auto-refresh file list when tool outputs modify files
            if (newMsg.role === 'tool') {
              const pId = selectedProjectId();
              if (pId) loadProjectFiles(pId);
            }
          } else if (payload.type === 'step') {
            setCurrentStepInfo(payload.data?.details || payload.data?.action || null);
            const pId = selectedProjectId();
            if (pId) loadProjectFiles(pId);
          } else if (payload.type === 'status') {
            const st = payload.data?.status;
            if (st) {
              setSessions((prev) =>
                prev.map((s) => (s.id === sessionId ? { ...s, status: st } : s))
              );
              if (st === 'idle' || st === 'completed' || st === 'failed') {
                markSessionRunning(sessionId, false);
                if (selectedSessionId() === sessionId) {
                  setCurrentStepInfo(null);
                }
                const pId = selectedProjectId();
                if (pId) loadProjectFiles(pId);
              } else if (st === 'running') {
                markSessionRunning(sessionId, true);
              }
            }
          } else if (payload.type === 'done') {
            markSessionRunning(sessionId, false);
            if (selectedSessionId() === sessionId) {
              setCurrentStepInfo(null);
            }
            const pId = selectedProjectId();
            if (pId) loadProjectFiles(pId);
          } else if (payload.type === 'error') {
            markSessionRunning(sessionId, false);
            if (selectedSessionId() === sessionId) {
              setCurrentStepInfo(null);
            }
            props.showToast(payload.data?.message || 'Terjadi kesalahan pada agent');
          }
        } catch (err) {
          console.warn('Error parsing SSE event:', err);
        }
      };

      es.onerror = (e) => {
        // EventSource will auto-reconnect if server closes connection
      };
    } catch (e) {
      console.warn('Failed to initialize EventSource:', e);
    }
  };

  onCleanup(() => {
    if (activeEventSource) {
      activeEventSource.close();
      activeEventSource = null;
    }
  });

  const loadData = async () => {
    setLoading(true);
    try {
      const [p, s, m] = await Promise.all([
        agentService.getProjects(),
        agentService.getPromptSkills(),
        agentService.getMcpServers(),
      ]);
      setProjects(p);
      setSkills(s);
      setMcpServers(m);

      if (p.length > 0) {
        const savedProj = localStorage.getItem('ragasukma_active_project_id');
        const targetProj = (savedProj && p.some(proj => proj.id === savedProj)) ? savedProj : p[0].id;
        setSelectedProjectId(targetProj);
        await loadSessions(targetProj);
        loadProjectFiles(targetProj);
      }
    } catch (e) {
      console.error(e);
      props.showToast('Gagal memuat data workspace');
    } finally {
      setLoading(false);
    }
  };

  const selectSession = async (sess: AgentSession) => {
    setSelectedSessionId(sess.id);
    localStorage.setItem('ragasukma_active_session_id', sess.id);
    if (sess.model) {
      setActiveSessionModel(sess.model);
    } else if (!activeSessionModel()) {
      setActiveSessionModel(defaultBrainModel() || (availableModels()[0]?.id ?? ''));
    }
    await loadMessages(sess.id);
    connectSessionSse(sess.id);
  };

  const loadProjectFiles = async (projectId: string) => {
    setLoadingFiles(true);
    try {
      const files = await agentService.getProjectFiles(projectId);
      setWorkspaceFiles(files);
    } catch (e) {
      console.error(e);
    } finally {
      setLoadingFiles(false);
    }
  };

  const filteredWorkspaceFiles = createMemo(() => {
    const q = fileSearch().trim().toLowerCase();
    if (!q) return workspaceFiles();
    return workspaceFiles().filter((f) =>
      f.name.toLowerCase().includes(q) || f.relative_path.toLowerCase().includes(q)
    );
  });

  const fileTree = createMemo<FileTreeNode[]>(() => {
    const files = filteredWorkspaceFiles();
    const rootNodes: FileTreeNode[] = [];
    const map: { [path: string]: FileTreeNode } = {};

    const sorted = [...files].sort((a, b) => a.relative_path.localeCompare(b.relative_path));

    for (const item of sorted) {
      const parts = item.relative_path.split('/').filter(Boolean);
      let currentPath = '';

      for (let i = 0; i < parts.length; i++) {
        const part = parts[i];
        const isLast = i === parts.length - 1;
        const prevPath = currentPath;
        currentPath = currentPath ? `${currentPath}/${part}` : part;

        if (!map[currentPath]) {
          const isDir = isLast ? item.is_dir : true;
          const node: FileTreeNode = {
            name: part,
            path: currentPath,
            is_dir: isDir,
            item: isLast ? item : undefined,
            children: [],
          };
          map[currentPath] = node;

          if (prevPath && map[prevPath]) {
            map[prevPath].children.push(node);
          } else if (i === 0) {
            rootNodes.push(node);
          }
        } else if (isLast) {
          map[currentPath].item = item;
          map[currentPath].is_dir = item.is_dir;
        }
      }
    }

    const sortNodes = (nodes: FileTreeNode[]) => {
      nodes.sort((a, b) => {
        if (a.is_dir === b.is_dir) {
          return a.name.localeCompare(b.name);
        }
        return a.is_dir ? -1 : 1;
      });
      for (const n of nodes) {
        if (n.children.length > 0) {
          sortNodes(n.children);
        }
      }
    };

    sortNodes(rootNodes);
    return rootNodes;
  });

  const insertFileTag = (relPath: string, start?: string, end?: string) => {
    let tag = `@${relPath}`;
    const s = start?.trim();
    const e = end?.trim();
    if (s && e) {
      tag = `@${relPath}:${s}-${e}`;
    } else if (s) {
      tag = `@${relPath}:${s}`;
    }

    const current = userInput().trim();
    if (!current) {
      setUserInput(`${tag} `);
    } else {
      setUserInput(`${current} ${tag} `);
    }
    props.showToast(`Tag ${tag} disisipkan ke input pesan!`);
    setSelectedFileForTag(null);
    setTimeout(() => chatInputRef?.focus(), 50);
  };

  const copyFileTag = (relPath: string, start?: string, end?: string) => {
    let tag = `@${relPath}`;
    const s = start?.trim();
    const e = end?.trim();
    if (s && e) {
      tag = `@${relPath}:${s}-${e}`;
    } else if (s) {
      tag = `@${relPath}:${s}`;
    }

    navigator.clipboard.writeText(tag);
    props.showToast(`Tag ${tag} disalin ke clipboard!`);
  };

  const openLineRangePicker = async (file: WorkspaceFileItem) => {
    setSelectedFileForTag(file);
    setTagStartLine('');
    setTagEndLine('');
    setFilePreview(null);

    const projId = selectedProjectId();
    if (projId) {
      setLoadingFilePreview(true);
      const res = await agentService.getFileContent(projId, file.relative_path);
      if (res) {
        setFilePreview(res.content);
      }
      setLoadingFilePreview(false);
    }
  };

  const highlightedCodeLines = createMemo(() => {
    const raw = filePreview();
    const file = selectedFileForTag();
    if (!raw || !file) return [];
    return highlightCode(raw, file.name);
  });

  let isDraggingGutter = false;
  let gutterDragStartLine: number | null = null;

  const handleLineClick = (lineNum: number, isShiftKey: boolean) => {
    if (isShiftKey && tagStartLine()) {
      const start = parseInt(tagStartLine(), 10);
      const minLine = Math.min(start, lineNum);
      const maxLine = Math.max(start, lineNum);
      setTagStartLine(String(minLine));
      setTagEndLine(String(maxLine));
    } else {
      setTagStartLine(String(lineNum));
      setTagEndLine(String(lineNum));
    }
  };

  const handleGutterMouseDown = (lineNum: number) => {
    isDraggingGutter = true;
    gutterDragStartLine = lineNum;
    setTagStartLine(String(lineNum));
    setTagEndLine(String(lineNum));
  };

  const handleGutterMouseEnter = (lineNum: number) => {
    if (!isDraggingGutter || gutterDragStartLine === null) return;
    const minLine = Math.min(gutterDragStartLine, lineNum);
    const maxLine = Math.max(gutterDragStartLine, lineNum);
    setTagStartLine(String(minLine));
    setTagEndLine(String(maxLine));
  };

  const handleGutterMouseUp = () => {
    isDraggingGutter = false;
    gutterDragStartLine = null;
  };

  const handleCodeTextMouseUp = () => {
    if (isDraggingGutter) {
      isDraggingGutter = false;
      gutterDragStartLine = null;
      return;
    }

    const selection = window.getSelection();
    if (!selection || selection.isCollapsed || selection.rangeCount === 0) return;

    const range = selection.getRangeAt(0);
    let startEl: HTMLElement | null = range.startContainer instanceof HTMLElement 
      ? range.startContainer 
      : range.startContainer.parentElement;
    let endEl: HTMLElement | null = range.endContainer instanceof HTMLElement 
      ? range.endContainer 
      : range.endContainer.parentElement;

    const startLineEl = startEl?.closest('[data-line]');
    const endLineEl = endEl?.closest('[data-line]');

    if (startLineEl && endLineEl) {
      const s = parseInt(startLineEl.getAttribute('data-line') || '0', 10);
      const e = parseInt(endLineEl.getAttribute('data-line') || '0', 10);
      if (s > 0 && e > 0) {
        const minLine = Math.min(s, e);
        const maxLine = Math.max(s, e);
        setTagStartLine(String(minLine));
        setTagEndLine(String(maxLine));
      }
    }
  };

  const isLineSelected = (lineNum: number) => {
    const s = parseInt(tagStartLine(), 10);
    const e = parseInt(tagEndLine(), 10);
    if (!s) return false;
    if (!e) return lineNum === s;
    return lineNum >= s && lineNum <= e;
  };

  const handleResetSelection = () => {
    setTagStartLine('');
    setTagEndLine('');
    window.getSelection()?.removeAllRanges();
  };

  const currentSelectedTag = () => {
    const file = selectedFileForTag();
    if (!file) return '';
    const s = tagStartLine();
    const e = tagEndLine();
    if (s && e && s !== e) {
      return `@${file.relative_path}:${s}-${e}`;
    } else if (s) {
      return `@${file.relative_path}:${s}`;
    }
    return `@${file.relative_path}`;
  };

  const loadSessions = async (projectId: string) => {
    try {
      const sess = await agentService.getSessions(projectId);
      setSessions(sess);
      const savedSess = localStorage.getItem('ragasukma_active_session_id');
      const targetSess = (savedSess && sess.some(s => s.id === savedSess)) 
        ? savedSess 
        : (sess.length > 0 ? sess[0].id : null);

      if (targetSess) {
        setSelectedSessionId(targetSess);
        const activeS = sess.find(s => s.id === targetSess);
        if (activeS?.model) {
          setActiveSessionModel(activeS.model);
        } else if (!activeSessionModel()) {
          setActiveSessionModel(defaultBrainModel() || (availableModels()[0]?.id ?? ''));
        }
        await loadMessages(targetSess);
        connectSessionSse(targetSess);
      } else if (sess.length === 0) {
        setSelectedSessionId(null);
        setMessages([]);
      }
    } catch (e) {
      console.error(e);
    }
  };

  const loadMessages = async (sessionId: string) => {
    setLoadingMessages(true);
    try {
      const msgs = await agentService.getMessages(sessionId);
      setMessages(msgs);
      setTimeout(() => scrollToBottom(false), 50);
    } catch (e) {
      console.error(e);
    } finally {
      setLoadingMessages(false);
    }
  };

  onMount(() => {
    loadData();
    refreshDynamicModels();
  });

  // ==========================================================================
  // Workspace Folder Handlers
  // ==========================================================================

  const [browsingFolder, setBrowsingFolder] = createSignal(false);

  const handleOpenFolderInFileManager = async (folderPath: string) => {
    try {
      const res = await agentService.openInFileManager(folderPath);
      if (res.ok) {
        props.showToast(`Membuka "${folderPath}" di File Manager sistem...`);
      } else {
        props.showToast(res.error || 'Gagal membuka file manager');
      }
    } catch (e: any) {
      props.showToast(e.message || 'Gagal membuka file manager');
    }
  };

  const handleBrowseFolderForModal = async () => {
    setBrowsingFolder(true);
    props.showToast('Membuka dialog file manager di layar utama desktop...');
    try {
      const res = await agentService.browseFolder(folderInput());
      if (res.ok && res.path) {
        if (projFolders().includes(res.path)) {
          setFolderValidationMsg({ valid: false, text: 'Folder sudah ada di daftar workspace ini!' });
        } else {
          setProjFolders([...projFolders(), res.path]);
          setFolderInput('');
          setFolderValidationMsg({ valid: true, text: `Folder dipilih: ${res.path}` });
        }
      } else if (res.error) {
        props.showToast(res.error);
      }
    } catch (e: any) {
      props.showToast(e.message || 'Gagal membuka file manager');
    } finally {
      setBrowsingFolder(false);
    }
  };

  const handleBrowseFolderForActiveProject = async () => {
    const proj = currentProject();
    if (!proj) return;

    setBrowsingFolder(true);
    props.showToast('Membuka dialog file manager di layar utama desktop...');
    try {
      const res = await agentService.browseFolder();
      if (res.ok && res.path) {
        const current = parseProjectFolders(proj.root_path);
        if (current.includes(res.path)) {
          props.showToast('Folder sudah terlampir di workspace ini!');
          return;
        }
        const updated = [...current, res.path];
        const updateRes = await agentService.updateProject(proj.id, {
          name: proj.name,
          description: proj.description,
          folders: updated,
          custom_instructions: proj.custom_instructions,
        });
        if (updateRes.ok) {
          props.showToast(`Folder "${res.path}" berhasil dilampirkan!`);
          const pList = await agentService.getProjects();
          setProjects(pList);
          loadProjectFiles(proj.id);
        } else {
          props.showToast(updateRes.error || 'Gagal melampirkan folder');
        }
      } else if (res.error) {
        props.showToast(res.error);
      }
    } catch (e: any) {
      props.showToast(e.message || 'Gagal membuka file manager');
    } finally {
      setBrowsingFolder(false);
    }
  };

  const handleValidateAndAddFolder = async () => {
    const raw = folderInput().trim();
    if (!raw) return;

    if (projFolders().includes(raw)) {
      setFolderValidationMsg({ valid: false, text: 'Folder sudah ada di daftar workspace ini!' });
      return;
    }

    setValidatingFolder(true);
    setFolderValidationMsg(null);
    try {
      const res = await agentService.validateFolder(raw);
      if (res.valid && res.path) {
        setProjFolders([...projFolders(), res.path]);
        setFolderInput('');
        setFolderValidationMsg({ valid: true, text: `Folder valid: ${res.path}` });
      } else {
        setFolderValidationMsg({ valid: false, text: res.error || 'Folder tidak ditemukan di disk!' });
      }
    } catch (e: any) {
      setFolderValidationMsg({ valid: false, text: e.message || 'Gagal memvalidasi folder' });
    } finally {
      setValidatingFolder(false);
    }
  };

  const handleRemoveFolder = (folderToRemove: string) => {
    setProjFolders(projFolders().filter((f) => f !== folderToRemove));
  };

  const handleAddFolderToActiveProject = async () => {
    const proj = currentProject();
    if (!proj) return;
    const raw = folderInput().trim();
    if (!raw) return;

    const currentFolders = parseProjectFolders(proj.root_path);
    if (currentFolders.includes(raw)) {
      props.showToast('Folder ini sudah ada di dalam workspace');
      return;
    }

    setValidatingFolder(true);
    try {
      const res = await agentService.validateFolder(raw);
      if (!res.valid || !res.path) {
        props.showToast(res.error || 'Folder tidak valid atau tidak ada di disk!');
        return;
      }

      const updated = [...currentFolders, res.path];
      const updateRes = await agentService.updateProject(proj.id, {
        name: proj.name,
        description: proj.description,
        folders: updated,
        custom_instructions: proj.custom_instructions,
      });

      if (updateRes.ok) {
        props.showToast('Folder berhasil ditambahkan ke workspace');
        setFolderInput('');
        const pList = await agentService.getProjects();
        setProjects(pList);
      } else {
        props.showToast(updateRes.error || 'Gagal mengupdate folder');
      }
    } finally {
      setValidatingFolder(false);
    }
  };

  const handleRemoveFolderFromActiveProject = async (folderToRemove: string) => {
    const proj = currentProject();
    if (!proj) return;
    const currentFolders = parseProjectFolders(proj.root_path);
    const updated = currentFolders.filter((f: string) => f !== folderToRemove);

    const updateRes = await agentService.updateProject(proj.id, {
      name: proj.name,
      description: proj.description,
      folders: updated,
      custom_instructions: proj.custom_instructions,
    });

    if (updateRes.ok) {
      props.showToast('Folder dihapus dari workspace');
      const pList = await agentService.getProjects();
      setProjects(pList);
    } else {
      props.showToast(updateRes.error || 'Gagal menghapus folder');
    }
  };

  // ==========================================================================
  // Project Actions
  // ==========================================================================

  const handleCreateProject = async () => {
    if (!projName().trim()) {
      props.showToast('Nama project tidak boleh kosong');
      return;
    }

    const res = await agentService.createProject({
      name: projName().trim(),
      description: projDesc().trim(),
      folders: projFolders(),
      custom_instructions: projInstructions().trim(),
    });

    if (res.ok && res.project) {
      props.showToast(`Project "${res.project.name}" berhasil dibuat!`);
      setShowNewProjectModal(false);
      setProjName('');
      setProjDesc('');
      setProjInstructions('');
      setProjFolders([]);
      setFolderInput('');
      setFolderValidationMsg(null);

      const pList = await agentService.getProjects();
      setProjects(pList);
      setSelectedProjectId(res.project.id);
      await loadSessions(res.project.id);
      loadProjectFiles(res.project.id);
    } else {
      props.showToast(res.error || 'Gagal membuat project');
    }
  };

  const handleDeleteProject = async (id: string, name: string) => {
    if (!confirm(`Hapus workspace "${name}" beserta seluruh sesi obrolannya?`)) return;
    const ok = await agentService.deleteProject(id);
    if (ok) {
      props.showToast(`Workspace "${name}" dihapus`);
      const pList = await agentService.getProjects();
      setProjects(pList);
      if (pList.length > 0) {
        setSelectedProjectId(pList[0].id);
        await loadSessions(pList[0].id);
      } else {
        setSelectedProjectId(null);
        setSessions([]);
        setSelectedSessionId(null);
        setMessages([]);
      }
    } else {
      props.showToast('Gagal menghapus project');
    }
  };

  // ==========================================================================
  // Session Actions (1-Click Instant Task Creation)
  // ==========================================================================

  const handleQuickCreateSession = async () => {
    const projId = selectedProjectId();
    if (!projId) {
      props.showToast('Pilih workspace project terlebih dahulu');
      return;
    }

    const nextNum = sessions().length + 1;
    const title = `Task #${nextNum}`;
    const modelToUse = activeSessionModel() || defaultBrainModel() || undefined;

    const res = await agentService.createSession(projId, {
      title,
      model: modelToUse,
    });

    if (res.ok && res.session) {
      props.showToast(`Sesi "${title}" dibuat!`);
      await loadSessions(projId);
      setSelectedSessionId(res.session.id);
      if (res.session.model) {
        setActiveSessionModel(res.session.model);
      }
      await loadMessages(res.session.id);
      setTimeout(() => {
        chatInputRef?.focus();
      }, 100);
    } else {
      props.showToast(res.error || 'Gagal membuat sesi');
    }
  };

  const handleDeleteSession = async (id: string, title: string) => {
    if (!confirm(`Hapus task sesi "${title}"?`)) return;
    const ok = await agentService.deleteSession(id);
    if (ok) {
      props.showToast(`Sesi "${title}" dihapus`);
      const projId = selectedProjectId();
      if (projId) await loadSessions(projId);
    } else {
      props.showToast('Gagal menghapus sesi');
    }
  };

  const handleSendMessage = async () => {
    const sessId = selectedSessionId();
    const text = userInput().trim();
    if (!sessId || !text || isSessionRunning(sessId)) return;

    setAutocompleteOpen(false);
    markSessionRunning(sessId, true);
    setCurrentStepInfo('Mengirim instruksi ke agent...');
    try {
      // 1. Create User message in session
      const res = await agentService.createMessage(sessId, {
        role: 'user',
        content: text,
      });

      if (res.ok && res.message) {
        setUserInput('');
        setMessages([...messages(), res.message]);
        setTimeout(() => scrollToBottom(true, true), 30);

        // Connect SSE stream to receive live step-by-step tool executions and messages
        connectSessionSse(sessId);

        // 2. Trigger Agent execution turn (runs async in Tokio runtime, broadcasting live SSE events)
        const currentModel = activeSessionModel() || undefined;
        const runRes = await agentService.runSession(sessId, currentModel);
        if (!runRes.ok) {
          props.showToast(runRes.error || 'Agent execution error');
          markSessionRunning(sessId, false);
          setCurrentStepInfo(null);
        }
      } else {
        props.showToast(res.error || 'Gagal mengirim pesan');
        markSessionRunning(sessId, false);
        setCurrentStepInfo(null);
      }
    } catch (e: any) {
      props.showToast(e.message || 'Gagal mengirim pesan');
      markSessionRunning(sessId, false);
      setCurrentStepInfo(null);
    } finally {
      const pId = selectedProjectId();
      if (pId) loadProjectFiles(pId);
    }
  };

  // ==========================================================================
  // Prompt Skills Actions
  // ==========================================================================

  const handleCreateSkill = async () => {
    if (!skillName().trim() || !skillSystemPrompt().trim()) {
      props.showToast('Nama dan System Prompt skill wajib diisi!');
      return;
    }
    const slug = skillSlug().trim() || skillName().trim().toLowerCase().replace(/[^a-z0-9]+/g, '-');
    const tagsArr = skillTags().split(',').map(t => t.trim()).filter(Boolean);

    const res = await agentService.createPromptSkill({
      name: skillName().trim(),
      slug,
      description: skillDesc().trim(),
      system_prompt: skillSystemPrompt().trim(),
      tags: tagsArr,
    });

    if (res.ok && res.skill) {
      props.showToast(`Prompt Skill "${res.skill.name}" ditambahkan!`);
      setShowNewSkillModal(false);
      setSkillName('');
      setSkillSlug('');
      setSkillDesc('');
      setSkillSystemPrompt('');
      setSkillTags('');
      const sList = await agentService.getPromptSkills();
      setSkills(sList);
    } else {
      props.showToast(res.error || 'Gagal menambahkan skill');
    }
  };

  const handleDeleteSkill = async (id: string, name: string) => {
    if (!confirm(`Hapus prompt skill "${name}"?`)) return;
    const ok = await agentService.deletePromptSkill(id);
    if (ok) {
      props.showToast(`Skill "${name}" dihapus`);
      const sList = await agentService.getPromptSkills();
      setSkills(sList);
    } else {
      props.showToast('Gagal menghapus skill');
    }
  };

  return (
    <div class="h-full flex flex-col space-y-2.5 overflow-hidden">
      {/* ── Top Header Bar ─────────────────────────────────────────────── */}
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-3 border-b border-white/10">
        <div class="flex items-center gap-2.5">
          <div class="p-2 rounded-lg bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
            <Layers class="w-5 h-5" />
          </div>
          <div>
            <h2 class="text-base font-semibold text-white">Agent Workspace</h2>
            <p class="text-xs text-white/50">Multi-folder Projects, Task Sessions, Prompt Skills & MCP</p>
          </div>
        </div>

        {/* Sub-tabs Toggle */}
        <div class="flex items-center gap-1 bg-white/5 p-1 rounded-lg border border-white/5 self-start sm:self-auto">
          <button
            onClick={() => setActiveSubTab('projects')}
            class={`px-3 py-1.5 text-xs font-medium rounded-md transition-colors flex items-center gap-1.5 ${
              activeSubTab() === 'projects'
                ? 'bg-indigo-600 text-white shadow-sm'
                : 'text-white/60 hover:text-white hover:bg-white/5'
            }`}
          >
            <FolderGit2 class="w-3.5 h-3.5" />
            Projects
          </button>
          <button
            onClick={() => setActiveSubTab('skills')}
            class={`px-3 py-1.5 text-xs font-medium rounded-md transition-colors flex items-center gap-1.5 ${
              activeSubTab() === 'skills'
                ? 'bg-indigo-600 text-white shadow-sm'
                : 'text-white/60 hover:text-white hover:bg-white/5'
            }`}
          >
            <Sparkles class="w-3.5 h-3.5" />
            Prompt Skills ({skills().length})
          </button>
          <button
            onClick={() => setActiveSubTab('mcp')}
            class={`px-3 py-1.5 text-xs font-medium rounded-md transition-colors flex items-center gap-1.5 ${
              activeSubTab() === 'mcp'
                ? 'bg-indigo-600 text-white shadow-sm'
                : 'text-white/60 hover:text-white hover:bg-white/5'
            }`}
          >
            <Server class="w-3.5 h-3.5" />
            MCP Servers ({mcpServers().length})
          </button>
        </div>
      </div>

      {/* ── SUB-TAB: PROJECTS & WORKSPACES ─────────────────────────────── */}
      <Show when={activeSubTab() === 'projects'}>
        <div class="flex-1 grid grid-cols-1 lg:grid-cols-12 gap-3 min-h-0 overflow-hidden">
          {/* Left Column: Projects List & Sessions (4 cols) */}
          <div class="lg:col-span-4 flex flex-col space-y-2.5 min-h-0 h-full overflow-hidden">
            {/* Projects Header & Selector */}
            <div class="bg-white/[0.03] border border-white/10 rounded-xl p-3 flex flex-col space-y-2 shrink-0">
              <div class="flex items-center justify-between">
                <span class="text-xs font-semibold text-white uppercase tracking-wider">Workspaces</span>
                <button
                  onClick={() => {
                    setProjFolders([]);
                    setFolderInput('');
                    setFolderValidationMsg(null);
                    setShowNewProjectModal(true);
                  }}
                  class="px-2 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium flex items-center gap-1 transition-colors"
                >
                  <Plus class="w-3.5 h-3.5" />
                  New Project
                </button>
              </div>

              {/* Projects List */}
              <div class="space-y-1 max-h-36 overflow-y-auto pr-1 custom-scrollbar">
                <Show when={projects().length === 0}>
                  <div class="text-xs text-white/40 py-4 text-center">
                    Belum ada project workspace
                  </div>
                </Show>
                <For each={projects()}>
                  {(proj) => {
                    const folders = parseProjectFolders(proj.root_path);
                    const isSelected = selectedProjectId() === proj.id;
                    return (
                      <div
                        onClick={() => {
                          setSelectedProjectId(proj.id);
                          localStorage.setItem('ragasukma_active_project_id', proj.id);
                          loadSessions(proj.id);
                        }}
                        class={`p-2.5 rounded-lg border text-left cursor-pointer transition-all flex items-center justify-between group ${
                          isSelected
                            ? 'bg-indigo-600/15 border-indigo-500/40 text-white shadow-sm'
                            : 'bg-white/[0.02] border-white/5 text-white/70 hover:bg-white/5 hover:text-white'
                        }`}
                      >
                        <div class="min-w-0 flex-1">
                          <div class="text-xs font-semibold truncate flex items-center gap-1.5">
                            <FolderGit2 class={`w-3.5 h-3.5 ${isSelected ? 'text-indigo-400' : 'text-white/40'}`} />
                            {proj.name}
                          </div>
                          <div class="text-[10px] text-white/40 truncate mt-0.5">
                            {folders.length} folder{folders.length > 1 ? 's' : ''} terlampir
                          </div>
                        </div>

                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            handleDeleteProject(proj.id, proj.name);
                          }}
                          class="opacity-0 group-hover:opacity-100 p-1 text-white/40 hover:text-rose-400 hover:bg-rose-500/10 rounded transition-all ml-2"
                          title="Hapus Workspace"
                        >
                          <Trash2 class="w-3.5 h-3.5" />
                        </button>
                      </div>
                    );
                  }}
                </For>
              </div>
            </div>

            {/* Sessions List & Workspace Files under Selected Project */}
            <Show when={currentProject()}>
              <div class="bg-white/[0.03] border border-white/10 rounded-xl p-3 flex-1 flex flex-col space-y-2 min-h-0 overflow-hidden">
                {/* Tab Switcher: Tasks vs Files */}
                <div class="flex items-center gap-1 bg-white/5 p-1 rounded-lg border border-white/10">
                  <button
                    onClick={() => setSidebarTab('sessions')}
                    class={`flex-1 py-1 px-2 text-xs font-semibold rounded transition-colors flex items-center justify-center gap-1.5 ${
                      sidebarTab() === 'sessions'
                        ? 'bg-indigo-600 text-white shadow-sm'
                        : 'text-white/60 hover:text-white'
                    }`}
                  >
                    <MessageSquare class="w-3.5 h-3.5" />
                    Tasks ({sessions().length})
                  </button>
                  <button
                    onClick={() => {
                      setSidebarTab('files');
                      const pId = selectedProjectId();
                      if (pId) loadProjectFiles(pId);
                    }}
                    class={`flex-1 py-1 px-2 text-xs font-semibold rounded transition-colors flex items-center justify-center gap-1.5 ${
                      sidebarTab() === 'files'
                        ? 'bg-indigo-600 text-white shadow-sm'
                        : 'text-white/60 hover:text-white'
                    }`}
                  >
                    <Files class="w-3.5 h-3.5" />
                    Files ({workspaceFiles().filter(f => !f.is_dir).length})
                  </button>
                </div>

                <Show when={sidebarTab() === 'sessions'}>
                  <div class="flex items-center justify-between">
                    <div>
                      <span class="text-xs font-semibold text-white uppercase tracking-wider">Task Sessions</span>
                      <span class="text-[10px] text-white/40 block">di bawah {currentProject()?.name}</span>
                    </div>
                    <button
                      onClick={handleQuickCreateSession}
                      class="px-2.5 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium flex items-center gap-1 transition-colors"
                    >
                      <Plus class="w-3.5 h-3.5" />
                      New Task
                    </button>
                  </div>

                  {/* Session Items */}
                  <div class="flex-1 overflow-y-auto space-y-1.5 pr-1 min-h-0 custom-scrollbar">
                    <Show when={sessions().length === 0}>
                      <div class="text-xs text-white/40 py-8 text-center flex flex-col items-center justify-center">
                        <Bot class="w-7 h-7 text-white/20 mb-2" />
                        <span>Belum ada sesi task di project ini</span>
                        <button
                          onClick={handleQuickCreateSession}
                          class="mt-2 text-xs text-indigo-400 hover:text-indigo-300 font-medium underline"
                        >
                          + Buat task pertama
                        </button>
                      </div>
                    </Show>
                    <For each={sessions()}>
                      {(sess) => {
                        const isSelected = selectedSessionId() === sess.id;
                        const isRunning = () => isSessionRunning(sess.id);
                        return (
                          <div
                            onClick={() => selectSession(sess)}
                            class={`p-2.5 rounded-lg border text-left cursor-pointer transition-all flex items-center justify-between group ${
                              isSelected
                                ? 'bg-indigo-600/20 border-indigo-500/50 text-white shadow-sm'
                                : 'bg-white/[0.02] border-white/5 text-white/70 hover:bg-white/5 hover:text-white'
                            }`}
                          >
                            <div class="min-w-0 flex-1">
                              <div class="text-xs font-medium truncate flex items-center gap-1.5">
                                <Show
                                  when={isRunning()}
                                  fallback={<MessageSquare class="w-3 h-3 text-indigo-400 shrink-0" />}
                                >
                                  <Loader2 class="w-3 h-3 text-amber-400 animate-spin shrink-0" />
                                </Show>
                                <span class={isRunning() ? 'text-amber-200 font-semibold truncate' : 'truncate'}>
                                  {sess.title}
                                </span>
                              </div>
                              <div class="flex items-center gap-2 mt-1">
                                <Show
                                  when={isRunning()}
                                  fallback={
                                    <span class="text-[9px] uppercase px-1.5 py-0.2 rounded bg-white/5 text-white/50">
                                      {sess.status}
                                    </span>
                                  }
                                >
                                  <span class="text-[9px] uppercase px-1.5 py-0.2 rounded bg-amber-500/15 text-amber-300 border border-amber-500/30 font-medium flex items-center gap-1 animate-pulse">
                                    <span class="w-1.5 h-1.5 rounded-full bg-amber-400 animate-ping"></span>
                                    running
                                  </span>
                                </Show>
                                <span class="text-[10px] text-white/30 truncate">
                                  {sess.model || 'default'}
                                </span>
                              </div>
                            </div>

                            <button
                              onClick={(e) => {
                                e.stopPropagation();
                                handleDeleteSession(sess.id, sess.title);
                              }}
                              class="opacity-0 group-hover:opacity-100 p-1 text-white/40 hover:text-rose-400 hover:bg-rose-500/10 rounded transition-all ml-2"
                              title="Hapus Sesi"
                            >
                              <Trash2 class="w-3.5 h-3.5" />
                            </button>
                          </div>
                        );
                      }}
                    </For>
                  </div>
                </Show>

                {/* Workspace Files List */}
                <Show when={sidebarTab() === 'files'}>
                  <div class="space-y-2">
                    <div class="flex items-center gap-1.5">
                      <div class="relative flex-1">
                        <Search class="w-3 h-3 text-white/40 absolute left-2 top-2" />
                        <input
                          type="text"
                          placeholder="Filter file..."
                          value={fileSearch()}
                          onInput={(e) => setFileSearch(e.currentTarget.value)}
                          class="w-full pl-6 pr-2 py-1 text-xs bg-white/5 border border-white/10 rounded-lg text-white placeholder-white/30 focus:outline-none focus:border-indigo-500"
                        />
                      </div>
                      <button
                        type="button"
                        onClick={expandAllDirs}
                        class="p-1.5 bg-white/5 hover:bg-white/10 text-white/60 hover:text-white rounded-lg transition-colors border border-white/10"
                        title="Buka / Expand Semua Folder"
                      >
                        <UnfoldVertical class="w-3.5 h-3.5" />
                      </button>
                      <button
                        type="button"
                        onClick={collapseAllDirs}
                        class="p-1.5 bg-white/5 hover:bg-white/10 text-white/60 hover:text-white rounded-lg transition-colors border border-white/10"
                        title="Tutup / Collapse Semua Folder"
                      >
                        <FoldVertical class="w-3.5 h-3.5" />
                      </button>
                      <button
                        onClick={() => {
                          const pId = selectedProjectId();
                          if (pId) loadProjectFiles(pId);
                        }}
                        class="p-1.5 bg-white/5 hover:bg-white/10 text-white/60 hover:text-white rounded-lg transition-colors border border-white/10"
                        title="Refresh file list"
                      >
                        <RefreshCw class={`w-3.5 h-3.5 ${loadingFiles() ? 'animate-spin' : ''}`} />
                      </button>
                      <button
                        type="button"
                        onClick={() => {
                          const proj = currentProject();
                          if (!proj) return;
                          const folders = parseProjectFolders(proj.root_path);
                          if (folders.length > 0) {
                            handleOpenFolderInFileManager(folders[0]);
                          } else {
                            props.showToast('Belum ada folder terlampir di workspace');
                          }
                        }}
                        class="p-1.5 bg-white/5 hover:bg-white/10 text-indigo-400 hover:text-indigo-300 rounded-lg transition-colors border border-white/10"
                        title="Buka folder workspace di File Manager sistem (OS)"
                      >
                        <ExternalLink class="w-3.5 h-3.5" />
                      </button>
                    </div>

                    <div class="flex items-center justify-between text-[10px] text-white/40 px-1">
                      <span>Klik baris untuk tag <code class="text-indigo-400">@file</code></span>
                      <span>{filteredWorkspaceFiles().filter(f => !f.is_dir).length} file</span>
                    </div>

                    <div class="flex-1 overflow-y-auto space-y-0.5 pr-1 min-h-0 custom-scrollbar">
                      <Show when={loadingFiles()}>
                        <div class="flex items-center justify-center py-8 text-xs text-white/40 gap-2">
                          <Loader2 class="w-4 h-4 animate-spin text-indigo-400" />
                          <span>Memindai file workspace...</span>
                        </div>
                      </Show>

                      <Show when={!loadingFiles() && fileTree().length === 0}>
                        <div class="text-xs text-white/40 py-8 text-center flex flex-col items-center justify-center">
                          <Files class="w-7 h-7 text-white/20 mb-2" />
                          <span>Tidak ada file di folder workspace</span>
                        </div>
                      </Show>

                      <For each={fileTree()}>
                        {(rootNode) => (
                          <FileTreeNodeRow
                            node={rootNode}
                            depth={0}
                            isExpanded={isDirExpanded}
                            onToggleDir={toggleDir}
                            onSelectFile={(f) => insertFileTag(f.relative_path)}
                            onOpenRange={openLineRangePicker}
                            onCopyTag={copyFileTag}
                            onTagFolder={(path) => insertFileTag(path)}
                          />
                        )}
                      </For>
                    </div>
                  </div>
                </Show>
              </div>
            </Show>
          </div>

          {/* Right Column: Project Workspace Details & Execution Pane (8 cols) */}
          <div class="lg:col-span-8 flex flex-col space-y-2.5 min-h-0 h-full overflow-hidden">
            <Show when={currentProject()} fallback={
              <div class="h-full bg-white/[0.02] border border-white/10 rounded-xl p-8 flex flex-col items-center justify-center text-center">
                <FolderGit2 class="w-12 h-12 text-white/20 mb-3" />
                <h3 class="text-sm font-semibold text-white">Pilih atau Buat Project</h3>
                <p class="text-xs text-white/40 max-w-sm mt-1">
                  Pilih workspace dari daftar di sebelah kiri atau buat project baru dengan melampirkan folder lokal yang sudah ada.
                </p>
              </div>
            }>
              {/* Workspace Context Bar (Compact & Collapsible) */}
              <div class="bg-white/[0.03] border border-white/10 rounded-xl px-3.5 py-2 space-y-2 shrink-0">
                <div class="flex items-center justify-between">
                  <div class="flex items-center gap-2 min-w-0">
                    <FolderGit2 class="w-4 h-4 text-indigo-400 shrink-0" />
                    <span class="text-xs font-semibold text-white truncate">{currentProject()?.name}</span>
                    <span class="text-[10px] text-white/40 shrink-0">
                      ({parseProjectFolders(currentProject()?.root_path).length} folder)
                    </span>
                  </div>
                  <button
                    type="button"
                    onClick={() => setShowWorkspaceSettings(!showWorkspaceSettings())}
                    class="text-[10px] text-white/50 hover:text-white flex items-center gap-1 transition-colors px-2 py-0.5 rounded hover:bg-white/5 shrink-0"
                  >
                    <span>{showWorkspaceSettings() ? 'Tutup Folder' : 'Kelola Folder'}</span>
                    <ChevronDown class={`w-3 h-3 transition-transform ${showWorkspaceSettings() ? 'rotate-180' : ''}`} />
                  </button>
                </div>

                <Show when={showWorkspaceSettings()}>
                  <div class="pt-2 border-t border-white/5 space-y-2">
                    <div class="flex flex-wrap gap-1.5 max-h-24 overflow-y-auto custom-scrollbar">
                      <For each={parseProjectFolders(currentProject()?.root_path)}>
                        {(folderPath) => (
                          <div class="flex items-center gap-1.5 px-2 py-0.5 bg-white/5 border border-white/10 rounded-md text-[11px] text-white font-mono group">
                            <Folder class="w-3 h-3 text-amber-400 shrink-0" />
                            <span class="truncate max-w-xs">{folderPath}</span>
                            <button
                              type="button"
                              onClick={() => handleOpenFolderInFileManager(folderPath)}
                              class="text-white/40 hover:text-indigo-400 p-0.5 rounded transition-colors"
                              title="Buka folder ini di File Manager sistem (OS)"
                            >
                              <ExternalLink class="w-3 h-3" />
                            </button>
                            <button
                              onClick={() => handleRemoveFolderFromActiveProject(folderPath)}
                              class="text-white/40 hover:text-rose-400 p-0.5 rounded transition-colors"
                              title="Hapus folder dari workspace"
                            >
                              <X class="w-3 h-3" />
                            </button>
                          </div>
                        )}
                      </For>
                      <Show when={parseProjectFolders(currentProject()?.root_path).length === 0}>
                        <span class="text-xs text-white/40 italic">Belum ada folder terlampir ke workspace ini</span>
                      </Show>
                    </div>

                    <div class="flex items-center gap-2 pt-1">
                      <input
                        type="text"
                        placeholder="Tambahkan folder lokal yang sudah ada..."
                        value={folderInput()}
                        onInput={(e) => setFolderInput(e.currentTarget.value)}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter') handleAddFolderToActiveProject();
                        }}
                        class="flex-1 px-2.5 py-1 text-xs bg-white/5 border border-white/10 rounded-lg text-white font-mono placeholder-white/30 focus:outline-none focus:border-indigo-500"
                      />
                      <button
                        onClick={handleAddFolderToActiveProject}
                        disabled={validatingFolder() || !folderInput().trim()}
                        class="px-2.5 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded-lg font-medium flex items-center gap-1 transition-colors shrink-0"
                      >
                        <Show when={validatingFolder()} fallback={<FolderPlus class="w-3.5 h-3.5" />}>
                          <Loader2 class="w-3.5 h-3.5 animate-spin" />
                        </Show>
                        Lampirkan
                      </button>
                      <button
                        type="button"
                        onClick={handleBrowseFolderForActiveProject}
                        disabled={browsingFolder()}
                        class="px-2.5 py-1 text-xs bg-white/10 hover:bg-white/15 disabled:opacity-50 text-white rounded-lg font-medium flex items-center gap-1.5 transition-colors shrink-0 border border-white/10"
                        title="Buka File Manager sistem untuk memilih folder langsung"
                      >
                        <Show when={browsingFolder()} fallback={<FolderOpen class="w-3.5 h-3.5 text-indigo-400" />}>
                          <Loader2 class="w-3.5 h-3.5 animate-spin" />
                        </Show>
                        Buka File Manager
                      </button>
                    </div>
                  </div>
                </Show>
              </div>

              {/* Active Session Chat / Execution Area (100% Flex Fitted, Only Messages Scroll) */}
              <div class="bg-white/[0.03] border border-white/10 rounded-xl flex-1 flex flex-col min-h-0 overflow-hidden shadow-lg">
                {/* Session Header */}
                <div class="px-4 py-2.5 border-b border-white/10 flex flex-wrap items-center justify-between gap-2 bg-white/[0.01]">
                  <div class="flex items-center gap-2 min-w-0">
                    <MessageSquare class="w-4 h-4 text-indigo-400 shrink-0" />
                    <div class="min-w-0 flex items-center gap-2">
                      <span class="text-xs font-semibold text-white truncate">
                        {currentSession()?.title || 'Pilih atau Buat Sesi Task'}
                      </span>
                      <Show when={currentSession()}>
                        <span class="text-[9px] uppercase px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-400 border border-emerald-500/20 font-medium">
                          {currentSession()?.status}
                        </span>
                      </Show>
                    </div>
                  </div>

                  {/* Model Switcher on-the-fly */}
                  <Show when={currentSession()}>
                    <div class="flex items-center gap-1.5 bg-white/5 border border-white/10 rounded-lg px-2.5 py-1 text-xs">
                      <Cpu class="w-3.5 h-3.5 text-indigo-400 shrink-0" />
                      <select
                        value={activeSessionModel() || currentSession()?.model || defaultBrainModel()}
                        onChange={(e) => setActiveSessionModel(e.currentTarget.value)}
                        class="bg-transparent text-white font-mono text-[11px] focus:outline-none cursor-pointer max-w-[220px] truncate"
                        title="Ganti Model LLM untuk giliran task ini secara dinamis"
                      >
                        <Show when={defaultBrainModel()}>
                          <option value={defaultBrainModel()} class="bg-[#181d26] text-white">
                            ⭐ {defaultBrainModel()} (Brain Persona)
                          </option>
                        </Show>
                        <For each={availableModels().filter((m) => m.id !== defaultBrainModel())}>
                          {(m) => (
                            <option value={m.id} class="bg-[#181d26] text-white">
                              {m.label}
                            </option>
                          )}
                        </For>
                      </select>
                      <button
                        type="button"
                        onClick={() => refreshDynamicModels()}
                        disabled={isRefreshingModels()}
                        class="text-white/40 hover:text-white disabled:opacity-50 transition-colors p-0.5 ml-0.5"
                        title="Sinkronkan daftar model terbaru"
                      >
                        <RefreshCw class={`w-3 h-3 ${isRefreshingModels() ? 'animate-spin' : ''}`} />
                      </button>
                    </div>
                  </Show>
                </div>

                {/* Messages Stream */}
                <div 
                  ref={messagesContainerRef}
                  onScroll={handleMessagesScroll}
                  class="flex-1 overflow-y-auto p-4 space-y-3 custom-scrollbar"
                >
                  <Show when={!currentSession()}>
                    <div class="h-full flex flex-col items-center justify-center text-center text-white/40 py-12">
                      <Bot class="w-10 h-10 mb-2 text-white/20" />
                      <p class="text-xs">Pilih salah satu task session di sebelah kiri atau buat sesi baru untuk memulai agent.</p>
                      <button
                        onClick={handleQuickCreateSession}
                        class="mt-3 px-3.5 py-1.5 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium flex items-center gap-1.5 transition-colors"
                      >
                        <Plus class="w-3.5 h-3.5" />
                        Buat Task Baru
                      </button>
                    </div>
                  </Show>

                  <Show when={currentSession() && messages().length === 0 && !loadingMessages()}>
                    <div class="h-full flex flex-col items-center justify-center text-center text-white/40 py-12">
                      <Sparkles class="w-8 h-8 mb-2 text-indigo-400/40" />
                      <p class="text-xs font-medium text-white/60">Sesi task ini masih kosong</p>
                      <p class="text-[11px] text-white/30 max-w-xs mt-1">
                        Ketik instruksi atau goal di bawah untuk memulai agent pada folder workspace yang terlampir.
                      </p>
                    </div>
                  </Show>

                  <For each={messages()}>
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
                            <span>Anda</span>
                          </Show>
                          <span>• #{msg.sequence_num}</span>
                        </div>

                        {/* Tool Calls badge if present on assistant */}
                        <Show when={msg.tool_calls}>
                          <ToolCallCard toolCalls={msg.tool_calls!} />
                        </Show>

                        {/* Reasoning Thought Collapse if present */}
                        <Show when={msg.thought}>
                          <div class="w-full max-w-xl mb-2 p-2.5 rounded-lg bg-amber-500/10 border border-amber-500/20 text-xs text-amber-200/90 font-mono">
                            <div class="flex items-center gap-1 font-semibold text-[10px] text-amber-400 uppercase tracking-wider mb-1">
                              <Brain class="w-3 h-3" />
                              Reasoning / Chain-of-Thought
                            </div>
                            <div class="whitespace-pre-wrap text-[11px] leading-relaxed opacity-80">{msg.thought}</div>
                          </div>
                        </Show>

                        {/* Tool Result Bubble if role == 'tool' */}
                        <Show when={msg.role === 'tool'}>
                          <ToolResultCard toolCallId={msg.tool_call_id} content={msg.content} />
                        </Show>

                        {/* Standard Message Bubble */}
                        <Show when={msg.role !== 'tool' && msg.content}>
                          <div class="flex flex-col space-y-1">
                            <Show when={msg.role === 'assistant'}>
                              <div class="flex items-center justify-between text-[10px] text-white/40 px-1">
                                <span class="font-mono flex items-center gap-1">
                                  <Bot class="w-3 h-3 text-indigo-400" />
                                  Assistant Generative ({countMessageLines(msg.content)} baris)
                                </span>
                                <button
                                  type="button"
                                  onClick={() => toggleCollapseMessage(msg.id)}
                                  class="flex items-center gap-1 px-2 py-0.5 rounded bg-white/5 hover:bg-white/10 hover:text-indigo-300 transition-colors text-[10px]"
                                  title={isMessageCollapsed(msg.id) ? "Tampilkan pesan lengkap" : "Sembunyikan hasil generative"}
                                >
                                  <Show when={isMessageCollapsed(msg.id)} fallback={<><ChevronUp class="w-3 h-3" /> Sembunyikan</>}>
                                    <ChevronDown class="w-3 h-3" /> Tampilkan ({countMessageLines(msg.content)} baris)
                                  </Show>
                                </button>
                              </div>
                            </Show>

                            <Show when={isMessageCollapsed(msg.id)}>
                              <div
                                onClick={() => toggleCollapseMessage(msg.id)}
                                class="cursor-pointer max-w-xl rounded-xl p-2.5 bg-indigo-500/10 border border-indigo-500/20 text-xs text-indigo-300/90 hover:bg-indigo-500/15 transition-all flex items-center justify-between"
                              >
                                <span class="truncate italic text-[11px]">Hasil generative disembunyikan ({countMessageLines(msg.content)} baris)</span>
                                <span class="text-[10px] font-semibold text-indigo-400 underline ml-2 shrink-0">Buka</span>
                              </div>
                            </Show>

                            <Show when={!isMessageCollapsed(msg.id)}>
                              <div
                                class={`max-w-2xl rounded-xl px-4 py-3 text-xs leading-relaxed ${
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

                  {/* Active Execution Spinner */}
                  <Show when={isCurrentSessionBusy()}>
                    <div class="flex items-center gap-2.5 p-3 bg-indigo-500/10 border border-indigo-500/20 rounded-xl text-xs text-indigo-300 w-fit animate-pulse">
                      <Loader2 class="w-4 h-4 animate-spin text-amber-400 shrink-0" />
                      <div>
                        <div class="font-semibold text-white">
                          {currentStepInfo() || 'Agent sedang memproses task sesi ini...'}
                        </div>
                        <div class="text-[10px] text-white/50">
                          Multi-task aktif di background. Anda bebas berpindah task atau project lain.
                        </div>
                      </div>
                    </div>
                  </Show>
                </div>

                {/* Chat Input Bar */}
                <Show when={currentSession()}>
                  <div class="p-3 border-t border-white/10 bg-white/[0.02]">
                    <div class="flex items-center gap-2">
                      <button
                        type="button"
                        onClick={() => {
                          setSidebarTab('files');
                          const pId = selectedProjectId();
                          if (pId) loadProjectFiles(pId);
                        }}
                        class="p-2 bg-white/5 hover:bg-white/10 text-white/70 hover:text-white rounded-lg text-xs font-mono flex items-center justify-center border border-white/10 transition-colors shrink-0"
                        title="Buka list file project untuk ditag (@file:baris)"
                      >
                        <AtSign class="w-3.5 h-3.5 text-indigo-400" />
                      </button>
                      <div class="relative flex-1 flex">
                        {/* Autocomplete Dropdown Popup */}
                        <Show when={autocompleteOpen() && autocompleteSuggestions().length > 0}>
                          <div class="absolute bottom-full mb-2 left-0 right-0 max-h-60 z-50 overflow-y-auto bg-slate-900/95 border border-indigo-500/30 rounded-xl shadow-2xl backdrop-blur-md p-1.5 custom-scrollbar">
                            <div class="px-2 py-1 text-[10px] font-semibold text-indigo-400 border-b border-white/10 flex items-center justify-between mb-1">
                              <span class="flex items-center gap-1"><AtSign class="w-3 h-3" /> Tag Workspace File</span>
                              <span class="text-white/40 font-normal">Gunakan ↑ ↓ lalu Enter / Tab</span>
                            </div>
                            <For each={autocompleteSuggestions()}>
                              {(file, idx) => (
                                <div
                                  class={`px-2.5 py-1.5 rounded-lg text-xs font-mono flex items-center justify-between cursor-pointer transition-colors ${
                                    idx() === autocompleteIndex()
                                      ? 'bg-indigo-600 text-white shadow-sm'
                                      : 'hover:bg-white/10 text-white/80'
                                  }`}
                                  onClick={() => selectAutocompleteFile(file)}
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

                        <input
                          ref={chatInputRef}
                          type="text"
                          placeholder={`Kirim prompt instruksi ke agent untuk workspace "${currentProject()?.name}" (ketik @ untuk mention file)...`}
                          value={userInput()}
                          onInput={handleChatInput}
                          onKeyDown={handleChatKeyDown}
                          disabled={isCurrentSessionBusy()}
                          class="w-full px-3 py-2 text-xs bg-white/5 border border-white/10 rounded-lg text-white placeholder-white/30 focus:outline-none focus:border-indigo-500"
                        />
                      </div>
                      <button
                        onClick={handleSendMessage}
                        disabled={isCurrentSessionBusy() || !userInput().trim()}
                        class="px-4 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded-lg text-xs font-medium flex items-center gap-1.5 transition-colors shrink-0"
                      >
                        <Show when={isCurrentSessionBusy()} fallback={<Send class="w-3.5 h-3.5" />}>
                          <Loader2 class="w-3.5 h-3.5 animate-spin" />
                        </Show>
                        Kirim
                      </button>
                    </div>
                  </div>
                </Show>
              </div>
            </Show>
          </div>
        </div>
      </Show>

      {/* ── SUB-TAB: PROMPT SKILLS LIBRARY ─────────────────────────────── */}
      <Show when={activeSubTab() === 'skills'}>
        <div class="space-y-4">
          <div class="flex items-center justify-between bg-white/[0.02] border border-white/5 p-4 rounded-xl">
            <div>
              <h3 class="text-sm font-semibold text-white">Pustaka Prompt Skills</h3>
              <p class="text-xs text-white/50 mt-0.5">
                Simpan persona khusus, aturan coding, atau sistem prompt yang bisa disematkan ke project mana pun.
              </p>
            </div>
            <button
              onClick={() => setShowNewSkillModal(true)}
              class="px-3 py-1.5 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium flex items-center gap-1.5 transition-colors"
            >
              <Plus class="w-3.5 h-3.5" />
              New Prompt Skill
            </button>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
            <For each={skills()}>
              {(skill) => (
                <div class="bg-white/[0.03] border border-white/10 rounded-xl p-4 flex flex-col justify-between space-y-3 group hover:border-indigo-500/40 transition-colors">
                  <div>
                    <div class="flex items-center justify-between mb-1.5">
                      <span class="text-xs font-semibold text-white flex items-center gap-1.5">
                        <Sparkles class="w-3.5 h-3.5 text-indigo-400" />
                        {skill.name}
                      </span>
                      <button
                        onClick={() => handleDeleteSkill(skill.id, skill.name)}
                        class="opacity-0 group-hover:opacity-100 p-1 text-white/40 hover:text-rose-400 rounded transition-all"
                        title="Hapus Skill"
                      >
                        <Trash2 class="w-3.5 h-3.5" />
                      </button>
                    </div>
                    <span class="text-[10px] font-mono bg-white/5 px-2 py-0.5 rounded text-indigo-300">
                      @{skill.slug}
                    </span>
                    <p class="text-xs text-white/60 mt-2 line-clamp-2">
                      {skill.description || 'Tidak ada deskripsi'}
                    </p>
                  </div>

                  <div class="pt-2 border-t border-white/5">
                    <div class="text-[10px] text-white/30 uppercase tracking-wider mb-1">System Prompt Preview:</div>
                    <p class="text-[11px] font-mono text-white/70 line-clamp-3 bg-black/20 p-2 rounded">
                      {skill.system_prompt}
                    </p>
                  </div>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>

      {/* ── SUB-TAB: MCP SERVERS REGISTRY ──────────────────────────────── */}
      <Show when={activeSubTab() === 'mcp'}>
        <div class="space-y-4">
          <div class="bg-white/[0.02] border border-white/5 p-4 rounded-xl flex items-center justify-between">
            <div>
              <h3 class="text-sm font-semibold text-white">Model Context Protocol (MCP) Servers</h3>
              <p class="text-xs text-white/50 mt-0.5">
                Standar terbuka menghubungkan agent dengan tools eksternal (Filesystem, Terminal, Postgres, GitHub, Brave Search).
              </p>
            </div>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
            <For each={mcpServers()}>
              {(server) => (
                <div class="bg-white/[0.03] border border-white/10 rounded-xl p-4 space-y-2">
                  <div class="flex items-center justify-between">
                    <span class="text-xs font-semibold text-white flex items-center gap-1.5">
                      <Server class="w-3.5 h-3.5 text-indigo-400" />
                      {server.name}
                    </span>
                    <span class="text-[10px] px-2 py-0.5 rounded uppercase font-semibold bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                      {server.transport}
                    </span>
                  </div>
                  <div class="text-xs font-mono text-white/60 bg-black/20 p-2 rounded">
                    Command: {server.command || '-'}
                  </div>
                </div>
              )}
            </For>
            <Show when={mcpServers().length === 0}>
              <div class="col-span-full py-12 text-center text-white/40 bg-white/[0.02] border border-white/5 rounded-xl">
                <Server class="w-10 h-10 mx-auto text-white/20 mb-2" />
                <p class="text-xs">Belum ada MCP server terdaftar.</p>
                <p class="text-[11px] text-white/30 mt-1">
                  Server MCP dapat didaftarkan untuk memperluas tool calling agent secara aman.
                </p>
              </div>
            </Show>
          </div>
        </div>
      </Show>

      {/* ── MODAL: CREATE NEW PROJECT / WORKSPACE ───────────────────────── */}
      <Show when={showNewProjectModal()}>
        <div class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
          <div class="bg-[#181d26] border border-white/10 rounded-xl max-w-lg w-full p-5 space-y-4 shadow-2xl">
            <div class="flex items-center justify-between border-b border-white/10 pb-3">
              <h3 class="text-sm font-semibold text-white flex items-center gap-2">
                <FolderPlus class="w-4 h-4 text-indigo-400" />
                Buat Project Workspace Baru
              </h3>
              <button
                onClick={() => setShowNewProjectModal(false)}
                class="text-white/40 hover:text-white"
              >
                <X class="w-4 h-4" />
              </button>
            </div>

            <div class="space-y-3 text-xs">
              <div>
                <label class="block text-white/70 mb-1 font-medium">Nama Project / Workspace *</label>
                <input
                  type="text"
                  placeholder="Contoh: AI Voice Engine, Web Frontend, Core Backend"
                  value={projName()}
                  onInput={(e) => setProjName(e.currentTarget.value)}
                  class="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label class="block text-white/70 mb-1 font-medium">Deskripsi Singkat</label>
                <input
                  type="text"
                  placeholder="Contoh: Workspace pengembangan pipeline audio dan MCP agent"
                  value={projDesc()}
                  onInput={(e) => setProjDesc(e.currentTarget.value)}
                  class="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              {/* Workspace Folder Inputs */}
              <div>
                <label class="block text-white/70 mb-1 font-medium">
                  Lampirkan Folder Workspace (Folder yang sudah ada di disk)
                </label>
                <p class="text-[11px] text-white/40 mb-2">
                  Masukkan absolute path folder di komputermu. Kamu bisa menambahkan berapapun folder ke dalam satu project ini.
                </p>

                <div class="flex items-center gap-2 mb-2">
                  <input
                    type="text"
                    placeholder="Contoh: /home/headway/MyWork/project/node/ai-voice"
                    value={folderInput()}
                    onInput={(e) => {
                      setFolderInput(e.currentTarget.value);
                      setFolderValidationMsg(null);
                    }}
                    onKeyDown={(e) => {
                      if (e.key === 'Enter') {
                        e.preventDefault();
                        handleValidateAndAddFolder();
                      }
                    }}
                    class="flex-1 px-3 py-1.5 bg-white/5 border border-white/10 rounded-lg text-white font-mono placeholder-white/30 focus:outline-none focus:border-indigo-500"
                  />
                  <button
                    onClick={handleValidateAndAddFolder}
                    disabled={validatingFolder() || !folderInput().trim()}
                    class="px-3 py-1.5 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded-lg font-medium flex items-center gap-1.5 shrink-0"
                  >
                    <Show when={validatingFolder()} fallback={<Check class="w-3.5 h-3.5" />}>
                      <Loader2 class="w-3.5 h-3.5 animate-spin" />
                    </Show>
                    Validasi & Tambah
                  </button>
                  <button
                    type="button"
                    onClick={handleBrowseFolderForModal}
                    disabled={browsingFolder()}
                    class="px-3 py-1.5 bg-white/10 hover:bg-white/15 disabled:opacity-50 text-white rounded-lg font-medium flex items-center gap-1.5 shrink-0 border border-white/10"
                    title="Buka File Manager sistem untuk memilih folder langsung"
                  >
                    <Show when={browsingFolder()} fallback={<FolderOpen class="w-3.5 h-3.5 text-indigo-400" />}>
                      <Loader2 class="w-3.5 h-3.5 animate-spin" />
                    </Show>
                    File Manager
                  </button>
                </div>

                {/* Validation Feedback */}
                <Show when={folderValidationMsg()}>
                  <div
                    class={`p-2 rounded-lg text-[11px] flex items-center gap-1.5 mb-2 ${
                      folderValidationMsg()?.valid
                        ? 'bg-emerald-500/10 text-emerald-300 border border-emerald-500/20'
                        : 'bg-rose-500/10 text-rose-300 border border-rose-500/20'
                    }`}
                  >
                    <Show when={folderValidationMsg()?.valid} fallback={<AlertCircle class="w-3.5 h-3.5 shrink-0" />}>
                      <CheckCircle2 class="w-3.5 h-3.5 shrink-0 text-emerald-400" />
                    </Show>
                    <span>{folderValidationMsg()?.text}</span>
                  </div>
                </Show>

                {/* List of staged folders */}
                <div class="space-y-1 max-h-32 overflow-y-auto">
                  <For each={projFolders()}>
                    {(f) => (
                      <div class="flex items-center justify-between p-2 bg-white/5 border border-white/5 rounded-lg font-mono text-[11px]">
                        <span class="truncate flex items-center gap-1.5 text-white/90">
                          <Folder class="w-3.5 h-3.5 text-amber-400 shrink-0" />
                          {f}
                        </span>
                        <button
                          onClick={() => handleRemoveFolder(f)}
                          class="text-white/40 hover:text-rose-400 p-0.5 rounded ml-2"
                        >
                          <X class="w-3.5 h-3.5" />
                        </button>
                      </div>
                    )}
                  </For>
                </div>
              </div>

              <div>
                <label class="block text-white/70 mb-1 font-medium">Custom System Instructions (Opsional)</label>
                <textarea
                  rows={2}
                  placeholder="Instruksi khusus yang akan selalu disertakan dalam prompt untuk project ini..."
                  value={projInstructions()}
                  onInput={(e) => setProjInstructions(e.currentTarget.value)}
                  class="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white focus:outline-none focus:border-indigo-500"
                />
              </div>
            </div>

            <div class="flex items-center justify-end gap-2 border-t border-white/10 pt-3">
              <button
                onClick={() => setShowNewProjectModal(false)}
                class="px-3.5 py-1.5 text-xs text-white/60 hover:text-white rounded-lg"
              >
                Batal
              </button>
              <button
                onClick={handleCreateProject}
                disabled={!projName().trim()}
                class="px-4 py-1.5 text-xs bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded-lg font-medium"
              >
                Buat Workspace
              </button>
            </div>
          </div>
        </div>
      </Show>



      {/* ── MODAL: CREATE PROMPT SKILL ──────────────────────────────────── */}
      <Show when={showNewSkillModal()}>
        <div class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
          <div class="bg-[#181d26] border border-white/10 rounded-xl max-w-lg w-full p-5 space-y-4 shadow-2xl">
            <div class="flex items-center justify-between border-b border-white/10 pb-3">
              <h3 class="text-sm font-semibold text-white flex items-center gap-2">
                <Sparkles class="w-4 h-4 text-indigo-400" />
                Tambah Prompt Skill Baru
              </h3>
              <button
                onClick={() => setShowNewSkillModal(false)}
                class="text-white/40 hover:text-white"
              >
                <X class="w-4 h-4" />
              </button>
            </div>

            <div class="space-y-3 text-xs">
              <div>
                <label class="block text-white/70 mb-1 font-medium">Nama Skill *</label>
                <input
                  type="text"
                  placeholder="Contoh: Rust Architecture Expert"
                  value={skillName()}
                  onInput={(e) => setSkillName(e.currentTarget.value)}
                  class="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label class="block text-white/70 mb-1 font-medium">Slug Identifikasi</label>
                <input
                  type="text"
                  placeholder="Contoh: rust-arch-expert"
                  value={skillSlug()}
                  onInput={(e) => setSkillSlug(e.currentTarget.value)}
                  class="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white font-mono focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label class="block text-white/70 mb-1 font-medium">Tags (Dipisahkan koma)</label>
                <input
                  type="text"
                  placeholder="coding, rust, review, architecture"
                  value={skillTags()}
                  onInput={(e) => setSkillTags(e.currentTarget.value)}
                  class="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label class="block text-white/70 mb-1 font-medium">Deskripsi Singkat</label>
                <input
                  type="text"
                  placeholder="Contoh: Ahli desain sistem dan idiom performa Rust"
                  value={skillDesc()}
                  onInput={(e) => setSkillDesc(e.currentTarget.value)}
                  class="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white focus:outline-none focus:border-indigo-500"
                />
              </div>

              <div>
                <label class="block text-white/70 mb-1 font-medium">System Prompt Instructions *</label>
                <textarea
                  rows={4}
                  placeholder="Instruksi sistem yang akan dibaca oleh model saat skill ini aktif..."
                  value={skillSystemPrompt()}
                  onInput={(e) => setSkillSystemPrompt(e.currentTarget.value)}
                  class="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white font-mono focus:outline-none focus:border-indigo-500"
                />
              </div>
            </div>

            <div class="flex items-center justify-end gap-2 border-t border-white/10 pt-3">
              <button
                onClick={() => setShowNewSkillModal(false)}
                class="px-3.5 py-1.5 text-xs text-white/60 hover:text-white rounded-lg"
              >
                Batal
              </button>
              <button
                onClick={handleCreateSkill}
                disabled={!skillName().trim() || !skillSystemPrompt().trim()}
                class="px-4 py-1.5 text-xs bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded-lg font-medium"
              >
                Simpan Skill
              </button>
            </div>
          </div>
        </div>
      </Show>

      {/* ── MODAL: LINE RANGE PICKER & TAG GENERATOR (@file:baris) ────────── */}
      <Show when={selectedFileForTag()}>
        <div 
          class="fixed inset-0 z-50 bg-black/80 backdrop-blur-md flex items-center justify-center p-3 sm:p-5"
          onMouseUp={handleGutterMouseUp}
        >
          <div class="bg-[#141820] border border-white/10 rounded-2xl max-w-5xl w-full h-[85vh] flex flex-col shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-150">
            {/* Modal Header */}
            <div class="px-5 py-3 border-b border-white/10 flex items-center justify-between bg-white/[0.02] shrink-0">
              <div class="flex items-center gap-2.5 min-w-0">
                <FileCode class="w-5 h-5 text-indigo-400 shrink-0" />
                <div class="min-w-0">
                  <div class="flex items-center gap-2">
                    <h3 class="text-sm font-semibold text-white font-mono truncate">
                      @{selectedFileForTag()?.relative_path}
                    </h3>
                    <span class="px-1.5 py-0.5 rounded bg-indigo-500/20 text-indigo-300 border border-indigo-500/30 text-[10px] uppercase font-mono font-bold shrink-0">
                      {selectedFileForTag()?.name.split('.').pop() || 'code'}
                    </span>
                  </div>
                  <p class="text-[11px] text-white/40 mt-0.5">
                    Total: {highlightedCodeLines().length || selectedFileForTag()?.line_count || 0} baris • Blok teks dengan kursor mouse atau klik nomor baris untuk memilih rentang
                  </p>
                </div>
              </div>

              <div class="flex items-center gap-2">
                <button
                  type="button"
                  onClick={() => {
                    const tag = currentSelectedTag();
                    if (tag) {
                      navigator.clipboard.writeText(tag);
                      props.showToast(`Tag ${tag} disalin ke clipboard!`);
                    }
                  }}
                  class="px-2.5 py-1 text-xs bg-white/5 hover:bg-white/10 text-white/70 hover:text-white rounded-lg flex items-center gap-1.5 transition-colors border border-white/10"
                  title="Salin tag @file:baris"
                >
                  <Copy class="w-3.5 h-3.5" />
                  Salin Tag
                </button>
                <button
                  onClick={() => setSelectedFileForTag(null)}
                  class="p-1.5 text-white/40 hover:text-white rounded-lg hover:bg-white/5 transition-colors"
                >
                  <X class="w-4 h-4" />
                </button>
              </div>
            </div>

            {/* Selection Status Banner */}
            <div class="px-5 py-2.5 bg-indigo-500/10 border-b border-indigo-500/20 flex flex-wrap items-center justify-between gap-2 shrink-0">
              <div class="flex items-center gap-2 min-w-0">
                <span class="text-indigo-400 text-xs font-semibold flex items-center gap-1 shrink-0">
                  <AtSign class="w-3.5 h-3.5" /> Tag yang akan disisipkan:
                </span>
                <span class="font-mono text-xs text-white bg-black/50 px-2.5 py-1 rounded-lg border border-indigo-500/30 font-bold truncate">
                  {currentSelectedTag()}
                </span>
                <Show when={tagStartLine()}>
                  <span class="text-[11px] text-indigo-300 font-medium">
                    ({tagEndLine() && tagEndLine() !== tagStartLine() 
                      ? `${parseInt(tagEndLine()) - parseInt(tagStartLine()) + 1} baris terpilih` 
                      : '1 baris terpilih'})
                  </span>
                </Show>
                <Show when={!tagStartLine()}>
                  <span class="text-[11px] text-white/40 italic">
                    (Semua isi file akan dirujuk)
                  </span>
                </Show>
              </div>

              <Show when={tagStartLine()}>
                <button
                  type="button"
                  onClick={handleResetSelection}
                  class="text-[11px] text-indigo-300 hover:text-white underline underline-offset-2 transition-colors"
                >
                  Reset pilihan (Pilih seluruh file)
                </button>
              </Show>
            </div>

            {/* Code Viewer Body with Line Numbers & Prism Highlighting */}
            <div 
              class="flex-1 overflow-auto bg-[#0d1117] font-mono text-xs custom-scrollbar select-text"
              onMouseUp={handleCodeTextMouseUp}
            >
              <Show when={loadingFilePreview()}>
                <div class="flex flex-col items-center justify-center h-full py-16 text-white/40 gap-2">
                  <Loader2 class="w-6 h-6 animate-spin text-indigo-400" />
                  <span>Memuat isi file dan mewarnai sintaks...</span>
                </div>
              </Show>

              <Show when={!loadingFilePreview() && highlightedCodeLines().length === 0}>
                <div class="text-center py-16 text-white/30 italic">
                  File kosong atau tidak ada konten teks yang dapat dibaca
                </div>
              </Show>

              <Show when={!loadingFilePreview() && highlightedCodeLines().length > 0}>
                <div class="min-w-full inline-block py-2">
                  <For each={highlightedCodeLines()}>
                    {(lineHtml, idx) => {
                      const lineNum = idx() + 1;
                      const selected = () => isLineSelected(lineNum);
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
                            onMouseDown={() => handleGutterMouseDown(lineNum)}
                            onMouseEnter={() => handleGutterMouseEnter(lineNum)}
                            onClick={(e) => handleLineClick(lineNum, e.shiftKey)}
                            class={`w-14 select-none text-right pr-3.5 font-mono text-[11px] cursor-pointer transition-colors shrink-0 ${
                              selected()
                                ? 'text-indigo-300 font-bold bg-indigo-500/30'
                                : 'text-white/30 hover:text-white/70'
                            }`}
                            title={`Baris ${lineNum} (Klik atau drag untuk pilih)`}
                          >
                            {lineNum}
                          </div>

                          {/* Code Line Content */}
                          <div
                            class="pl-4 pr-6 flex-1 whitespace-pre overflow-x-visible font-mono"
                            innerHTML={lineHtml || ' '}
                          />
                        </div>
                      );
                    }}
                  </For>
                </div>
              </Show>
            </div>

            {/* Modal Footer */}
            <div class="px-5 py-3 border-t border-white/10 flex items-center justify-between bg-white/[0.02] shrink-0">
              <span class="text-xs text-white/40 hidden sm:inline">
                Tip: Cukup blok teks kode atau klik-drag nomor baris untuk menandai rentang baris
              </span>

              <div class="flex items-center gap-2 ml-auto">
                <button
                  onClick={() => setSelectedFileForTag(null)}
                  class="px-3.5 py-1.5 text-xs text-white/60 hover:text-white rounded-lg hover:bg-white/5 transition-colors"
                >
                  Tutup
                </button>
                <button
                  onClick={() => {
                    const file = selectedFileForTag();
                    if (file) {
                      insertFileTag(file.relative_path, tagStartLine(), tagEndLine());
                    }
                  }}
                  class="px-4 py-1.5 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium flex items-center gap-1.5 transition-colors shadow-lg shadow-indigo-600/20"
                >
                  <Plus class="w-3.5 h-3.5" />
                  Sisipkan Tag ke Pesan
                </button>
              </div>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};
