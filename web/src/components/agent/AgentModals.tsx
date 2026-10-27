import { Component, For, Show, createSignal, createEffect } from 'solid-js';
import {
  FileCode,
  Sparkles,
  FolderPlus,
  Folder,
  FolderOpen,
  Plus,
  Loader2,
  X,
  Copy,
  AtSign,
  CheckCircle2,
  AlertCircle,
  Check,
  Server,
  Terminal,
  Globe,
  Wrench
} from 'lucide-solid';
import { WorkspaceFileItem, SaveMcpServerPayload, McpServerConfig } from '../../services';

export const CreateProjectModal: Component<{
  isOpen: boolean;
  onClose: () => void;
  projName: string;
  setProjName: (v: string) => void;
  projDesc: string;
  setProjDesc: (v: string) => void;
  projInstructions: string;
  setProjInstructions: (v: string) => void;
  projFolders: string[];
  folderInput: string;
  setFolderInput: (v: string) => void;
  validatingFolder: boolean;
  browsingFolder: boolean;
  folderValidationMsg: { valid: boolean; text: string } | null;
  onBrowseFolder: () => void;
  onValidateAndAddFolder: () => void;
  onRemoveFolder: (f: string) => void;
  onCreateProject: () => void;
}> = (props) => {
  return (
    <Show when={props.isOpen}>
      <div class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
        <div class="bg-[#181d26] border border-white/10 rounded-xl max-w-lg w-full p-5 space-y-4 shadow-2xl">
          <div class="flex items-center justify-between border-b border-white/10 pb-3">
            <h3 class="text-sm font-semibold text-white flex items-center gap-2">
              <FolderPlus class="w-4 h-4 text-indigo-400" />
              Create New Workspace Project
            </h3>
            <button
              onClick={props.onClose}
              class="text-white/70 hover:text-white"
            >
              <X class="w-4 h-4" />
            </button>
          </div>

          <div class="space-y-3 text-xs">
            <div>
              <label class="block text-white/70 mb-1 font-medium">Project Name *</label>
              <input
                type="text"
                placeholder="e.g. Backend API, Frontend App"
                value={props.projName}
                onInput={(e) => props.setProjName(e.currentTarget.value)}
                class="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white focus:outline-none focus:border-indigo-500"
              />
            </div>

            <div>
              <label class="block text-white/70 mb-1 font-medium">Description</label>
              <input
                type="text"
                placeholder="Brief workspace description..."
                value={props.projDesc}
                onInput={(e) => props.setProjDesc(e.currentTarget.value)}
                class="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white focus:outline-none focus:border-indigo-500"
              />
            </div>

            <div>
              <label class="block text-white/70 mb-1 font-medium">Workspace Folders (Can attach multiple)</label>
              <div class="flex items-center gap-2 mb-2">
                <input
                  type="text"
                  placeholder="Absolute or relative folder path..."
                  value={props.folderInput}
                  onInput={(e) => props.setFolderInput(e.currentTarget.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') props.onValidateAndAddFolder();
                  }}
                  class="flex-1 px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white font-mono placeholder-white/30 focus:outline-none focus:border-indigo-500"
                />
                <button
                  onClick={props.onValidateAndAddFolder}
                  disabled={props.validatingFolder || !props.folderInput.trim()}
                  class="px-3 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded-lg font-medium flex items-center gap-1 transition-colors shrink-0"
                >
                  <Show when={props.validatingFolder} fallback={<Plus class="w-3.5 h-3.5" />}>
                    <Loader2 class="w-3.5 h-3.5 animate-spin" />
                  </Show>
                  Add
                </button>
                <button
                  type="button"
                  onClick={props.onBrowseFolder}
                  disabled={props.browsingFolder}
                  class="px-3 py-2 bg-white/10 hover:bg-white/15 disabled:opacity-50 text-white rounded-lg font-medium flex items-center gap-1.5 transition-colors shrink-0 border border-white/10"
                  title="Open OS file manager to choose a folder directly"
                >
                  <Show when={props.browsingFolder} fallback={<FolderOpen class="w-3.5 h-3.5 text-indigo-400" />}>
                    <Loader2 class="w-3.5 h-3.5 animate-spin" />
                  </Show>
                  Browse Folder
                </button>
              </div>

              {/* Validation message badge */}
              <Show when={props.folderValidationMsg}>
                <div class={`text-xs mb-2 px-2.5 py-1 rounded flex items-center gap-1.5 ${
                  props.folderValidationMsg?.valid
                    ? 'bg-emerald-500/10 text-emerald-300 border border-emerald-500/20'
                    : 'bg-rose-500/10 text-rose-300 border border-rose-500/20'
                }`}>
                  <span>{props.folderValidationMsg?.text}</span>
                </div>
              </Show>

              {/* List of staged folders */}
              <div class="space-y-1 max-h-32 overflow-y-auto">
                <For each={props.projFolders}>
                  {(f) => (
                    <div class="flex items-center justify-between p-2 bg-white/5 border border-white/5 rounded-lg font-mono text-xs">
                      <span class="truncate flex items-center gap-1.5 text-white/90">
                        <Folder class="w-3.5 h-3.5 text-amber-400 shrink-0" />
                        {f}
                      </span>
                      <button
                        onClick={() => props.onRemoveFolder(f)}
                        class="text-white/70 hover:text-rose-400 p-0.5 rounded ml-2"
                      >
                        <X class="w-3.5 h-3.5" />
                      </button>
                    </div>
                  )}
                </For>
              </div>
            </div>

            <div>
              <label class="block text-white/70 mb-1 font-medium">Custom System Instructions (Optional)</label>
              <textarea
                rows={2}
                placeholder="Specific instructions that will always be included in the system prompt for this project..."
                value={props.projInstructions}
                onInput={(e) => props.setProjInstructions(e.currentTarget.value)}
                class="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white focus:outline-none focus:border-indigo-500"
              />
            </div>
          </div>

          <div class="flex items-center justify-end gap-2 border-t border-white/10 pt-3">
            <button
              onClick={props.onClose}
              class="px-3.5 py-1.5 text-xs text-white/60 hover:text-white rounded-lg"
            >
              Cancel
            </button>
            <button
              onClick={props.onCreateProject}
              disabled={!props.projName.trim()}
              class="px-4 py-1.5 text-xs bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded-lg font-medium"
            >
              Create Workspace
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};

interface McpTemplate {
  label: string;
  desc: string;
  json: {
    name: string;
    transport?: 'stdio' | 'sse' | 'http';
    command?: string;
    args?: string[];
    env?: Record<string, string>;
    url?: string;
  };
}

const MCP_TEMPLATES: McpTemplate[] = [
  {
    label: 'SQLite',
    desc: 'Query & inspect local SQLite databases via npx',
    json: {
      name: 'SQLite Explorer',
      command: 'npx',
      args: ['-y', '@modelcontextprotocol/server-sqlite', '--db-path', './test.db'],
    },
  },
  {
    label: 'Filesystem',
    desc: 'Read, write & search project directories',
    json: {
      name: 'Local Filesystem',
      command: 'npx',
      args: ['-y', '@modelcontextprotocol/server-filesystem', './'],
    },
  },
  {
    label: 'GitHub',
    desc: 'Inspect PRs, issues, commits & repos via npx',
    json: {
      name: 'GitHub Tools',
      command: 'npx',
      args: ['-y', '@modelcontextprotocol/server-github'],
      env: {
        GITHUB_PERSONAL_ACCESS_TOKEN: 'ghp_your_token_here',
      },
    },
  },
  {
    label: 'Postgres',
    desc: 'Connect & query PostgreSQL databases',
    json: {
      name: 'PostgreSQL DB',
      command: 'npx',
      args: ['-y', '@modelcontextprotocol/server-postgres', 'postgresql://user:pass@localhost:5432/mydb'],
    },
  },
  {
    label: 'Memory',
    desc: 'Knowledge graph & persistent context memory',
    json: {
      name: 'Memory Graph',
      command: 'npx',
      args: ['-y', '@modelcontextprotocol/server-memory'],
    },
  },
  {
    label: 'Brave Search',
    desc: 'Live web & news search via Brave API',
    json: {
      name: 'Brave Search',
      command: 'npx',
      args: ['-y', '@modelcontextprotocol/server-brave-search'],
      env: {
        BRAVE_API_KEY: 'BSA_your_brave_api_key',
      },
    },
  },
  {
    label: 'Git (uvx)',
    desc: 'Git repository operations via Python uvx',
    json: {
      name: 'Git CLI (uvx)',
      command: 'uvx',
      args: ['mcp-server-git', '--repository', '.'],
    },
  },
  {
    label: 'SSE / HTTP',
    desc: 'Connect to remote or local HTTP/SSE MCP server',
    json: {
      name: 'Remote MCP',
      transport: 'sse',
      url: 'http://127.0.0.1:8000/sse',
    },
  },
];

export const McpServerModal: Component<{
  isOpen: boolean;
  isEditing?: boolean;
  initialMode?: 'form' | 'json';
  onClose: () => void;
  onSave: (server: SaveMcpServerPayload) => Promise<boolean>;
  onTest?: (server: SaveMcpServerPayload) => Promise<{ ok: boolean; tools_count?: number; error?: string }>;
  initialData?: McpServerConfig | null;
}> = (props) => {
  const [name, setName] = createSignal('');
  const [jsonText, setJsonText] = createSignal('');
  const [jsonError, setJsonError] = createSignal<string | null>(null);
  const [detectedMultiServers, setDetectedMultiServers] = createSignal<Array<{ name: string; [key: string]: any }>>([]);
  const [importingAll, setImportingAll] = createSignal(false);
  const [isSaving, setIsSaving] = createSignal(false);

  const parseAndValidateJson = (text: string) => {
    setJsonText(text);
    if (!text.trim()) {
      setJsonError(null);
      setDetectedMultiServers([]);
      return;
    }
    try {
      const parsed = JSON.parse(text);
      setJsonError(null);

      // Support Claude Desktop format: { "mcpServers": { ... } }
      if (parsed && typeof parsed === 'object' && parsed.mcpServers && typeof parsed.mcpServers === 'object') {
        const entries = Object.entries(parsed.mcpServers);
        if (entries.length > 0) {
          const servers = entries.map(([sName, sData]: [string, any]) => ({
            name: sName,
            ...(typeof sData === 'object' && sData !== null ? sData : {}),
          }));
          setDetectedMultiServers(servers);
          if (!name()) setName(servers[0].name);
          return;
        }
      }

      // Support top-level map of servers: { "sqlite": { ... }, "github": { ... } }
      if (
        parsed &&
        typeof parsed === 'object' &&
        !parsed.command &&
        !parsed.url &&
        !parsed.args &&
        !parsed.transport &&
        !parsed.name
      ) {
        const entries = Object.entries(parsed);
        const isServerMap = entries.length > 0 && entries.every(([, v]) => typeof v === 'object' && v !== null && ((v as any).command || (v as any).url || (v as any).args));
        if (isServerMap) {
          const servers = entries.map(([sName, sData]: [string, any]) => ({
            name: sName,
            ...(typeof sData === 'object' && sData !== null ? sData : {}),
          }));
          setDetectedMultiServers(servers);
          if (!name()) setName(servers[0].name);
          return;
        }
      }

      // Single server
      setDetectedMultiServers([]);
      if (parsed.name && typeof parsed.name === 'string' && !name()) {
        setName(parsed.name);
      }
    } catch (err: any) {
      setJsonError(err?.message || 'Invalid JSON syntax');
    }
  };

  const handleFormatJson = () => {
    try {
      const parsed = JSON.parse(jsonText());
      setJsonText(JSON.stringify(parsed, null, 2));
      setJsonError(null);
    } catch (e: any) {
      setJsonError(e?.message || 'Invalid JSON syntax');
    }
  };

  const handleApplyTemplate = (tmpl: McpTemplate) => {
    setName(tmpl.json.name);
    const jsonStr = JSON.stringify(tmpl.json, null, 2);
    parseAndValidateJson(jsonStr);
  };

  // Sync state whenever modal opens or initialData changes
  createEffect(() => {
    if (props.isOpen) {
      setIsSaving(false);
      setImportingAll(false);
      setDetectedMultiServers([]);
      setJsonError(null);

      if (props.initialData) {
        setName(props.initialData.name || '');

        const cfg: any = {
          name: props.initialData.name,
          transport: props.initialData.transport || 'stdio',
        };
        if (props.initialData.url) cfg.url = props.initialData.url;
        if (props.initialData.command) cfg.command = props.initialData.command;
        try {
          const parsedArgs = JSON.parse(props.initialData.args || '[]');
          if (Array.isArray(parsedArgs) && parsedArgs.length > 0) {
            cfg.args = parsedArgs;
          }
        } catch {}

        try {
          const parsedEnv = JSON.parse(props.initialData.env || '{}');
          if (typeof parsedEnv === 'object' && parsedEnv !== null && Object.keys(parsedEnv).length > 0) {
            for (const [k, v] of Object.entries(parsedEnv)) {
              if (cfg[k] === undefined) {
                cfg[k] = v;
              }
            }
          }
        } catch {}

        setJsonText(JSON.stringify(cfg, null, 2));
      } else {
        const defaultTmpl = MCP_TEMPLATES[0];
        setName(defaultTmpl.json.name);
        setJsonText(JSON.stringify(defaultTmpl.json, null, 2));
      }
    }
  });

  const handleSave = async () => {
    const raw = jsonText().trim();
    if (!raw) {
      setJsonError('JSON configuration cannot be empty');
      return;
    }

    let parsed: any;
    try {
      parsed = JSON.parse(raw);
    } catch (e: any) {
      setJsonError(`Invalid JSON syntax: ${e?.message}`);
      return;
    }

    let finalName = name().trim();
    if (!finalName && parsed.name) {
      finalName = String(parsed.name).trim();
      setName(finalName);
    }

    if (!finalName) {
      setJsonError('Please provide a Server / Tool Name');
      return;
    }

    const payload: SaveMcpServerPayload = {
      id: props.initialData?.id || 'mcp_' + Date.now().toString(36) + '_' + Math.random().toString(36).substring(2, 6),
      name: finalName,
      raw_json: parsed,
      is_enabled: true,
    };

    setIsSaving(true);
    try {
      const ok = await props.onSave(payload);
      if (ok) {
        props.onClose();
      }
    } finally {
      setIsSaving(false);
    }
  };

  const handleImportAll = async () => {
    const servers = detectedMultiServers();
    if (servers.length === 0) return;
    setImportingAll(true);
    try {
      let saved = 0;
      for (const s of servers) {
        const payload: SaveMcpServerPayload = {
          id: 'mcp_' + Date.now().toString(36) + '_' + Math.random().toString(36).substring(2, 6),
          name: s.name || 'MCP Server',
          raw_json: s,
          is_enabled: true,
        };
        const ok = await props.onSave(payload);
        if (ok) saved++;
      }
      if (saved > 0) {
        props.onClose();
      }
    } finally {
      setImportingAll(false);
    }
  };

  return (
    <Show when={props.isOpen}>
      <div class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
        <div class="bg-theme-card border border-theme-border rounded-xl max-w-2xl w-full p-5 space-y-4 shadow-2xl">
          <div class="flex items-center justify-between border-b border-theme-border pb-3">
            <div class="flex items-center gap-2">
              <Server class="w-4 h-4 text-theme-cyan" />
              <h3 class="text-sm font-semibold text-theme-text">
                {props.isEditing ? 'Edit MCP Server' : 'Add MCP Server'}
              </h3>
            </div>

            <button
              onClick={props.onClose}
              class="text-theme-text/70 hover:text-theme-text p-1 rounded-lg hover:bg-theme-bg transition-colors"
            >
              <X class="w-4 h-4" />
            </button>
          </div>

          <div class="space-y-3.5 text-xs">
            {/* Server Name Input */}
            <div>
              <label class="block text-theme-text/80 mb-1 font-medium">Server / Tool Name *</label>
              <input
                type="text"
                placeholder="e.g. SQLite Explorer, GitHub Tools, Remote Weather SSE"
                value={name()}
                onInput={(e) => {
                  setName(e.currentTarget.value);
                  setJsonError(null);
                }}
                class="w-full px-3 py-2 bg-theme-bg border border-theme-border rounded-lg text-theme-text text-xs focus:outline-none focus:border-theme-cyan"
              />
            </div>

            {/* Presets and formatting toolbar */}
            <div class="space-y-1.5">
              <div class="flex items-center justify-between gap-2 flex-wrap">
                <div class="flex items-center gap-1.5 flex-wrap">
                  <span class="text-2xs text-theme-text/50 font-medium">Presets:</span>
                  <For each={MCP_TEMPLATES}>
                    {(tmpl) => (
                      <button
                        type="button"
                        onClick={() => handleApplyTemplate(tmpl)}
                        class="px-2 py-0.5 text-2xs bg-theme-bg hover:bg-theme-cyan/15 hover:text-theme-cyan hover:border-theme-cyan/40 border border-theme-border text-theme-text/80 rounded-md transition-colors"
                        title={tmpl.desc}
                      >
                        {tmpl.label}
                      </button>
                    )}
                  </For>
                </div>

                <div class="flex items-center gap-2">
                  <button
                    type="button"
                    onClick={handleFormatJson}
                    class="px-2 py-0.5 text-2xs bg-theme-bg hover:bg-theme-card border border-theme-border hover:border-theme-cyan/50 text-theme-text/80 rounded-md transition-colors flex items-center gap-1"
                    title="Prettify and format JSON"
                  >
                    Format JSON
                  </button>
                  <Show
                    when={!jsonError() && jsonText().trim()}
                    fallback={
                      <Show when={jsonError()}>
                        <span class="text-2xs text-rose-400 font-mono">Invalid JSON</span>
                      </Show>
                    }
                  >
                    <span class="text-2xs text-emerald-400 font-medium flex items-center gap-1">
                      <Check class="w-3 h-3" /> Valid JSON
                    </span>
                  </Show>
                </div>
              </div>

              {/* JSON Textarea */}
              <div class="relative">
                <textarea
                  rows={13}
                  value={jsonText()}
                  onInput={(e) => parseAndValidateJson(e.currentTarget.value)}
                  placeholder={`// Paste single server config, e.g.:\n{\n  "name": "SQLite Explorer",\n  "command": "npx",\n  "args": ["-y", "@modelcontextprotocol/server-sqlite", "--db-path", "./test.db"]\n}\n\n// Or paste Claude Desktop "mcpServers" config directly!`}
                  class={`w-full p-3 font-mono text-xs bg-theme-bg border rounded-lg text-theme-text focus:outline-none custom-scrollbar leading-relaxed ${
                    jsonError() ? 'border-rose-500/60 focus:border-rose-400' : 'border-theme-border focus:border-theme-cyan'
                  }`}
                />
              </div>

              {/* Detected multiple servers banner */}
              <Show when={detectedMultiServers().length > 1}>
                <div class="p-2.5 bg-theme-cyan/10 border border-theme-cyan/30 rounded-lg flex items-center justify-between gap-2 flex-wrap">
                  <div class="flex items-center gap-1.5 flex-wrap">
                    <span class="text-2xs text-theme-cyan font-semibold">
                      Detected {detectedMultiServers().length} servers in config:
                    </span>
                    <For each={detectedMultiServers()}>
                      {(s) => (
                        <button
                          type="button"
                          onClick={() => {
                            setName(s.name);
                            setJsonText(JSON.stringify(s, null, 2));
                          }}
                          class={`px-2 py-0.5 text-2xs rounded border transition-colors ${
                            name() === s.name
                              ? 'bg-theme-cyan text-theme-card font-bold border-theme-cyan'
                              : 'bg-theme-bg border-theme-border text-theme-text/80 hover:border-theme-cyan/50'
                          }`}
                        >
                          {s.name}
                        </button>
                      )}
                    </For>
                  </div>
                  <button
                    type="button"
                    onClick={handleImportAll}
                    disabled={importingAll()}
                    class="px-2.5 py-1 text-2xs bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card font-bold rounded shadow-sm transition-colors flex items-center gap-1"
                  >
                    <Show when={importingAll()} fallback={<Plus class="w-3 h-3" />}>
                      <Loader2 class="w-3 h-3 animate-spin" />
                    </Show>
                    <span>Import All ({detectedMultiServers().length}) Servers</span>
                  </button>
                </div>
              </Show>

              {/* Error message */}
              <Show when={jsonError()}>
                <div class="text-2xs text-rose-400 font-mono bg-rose-500/10 p-2 rounded border border-rose-500/20 flex items-start gap-1.5">
                  <AlertCircle class="w-3.5 h-3.5 shrink-0 mt-0.5" />
                  <span>{jsonError()}</span>
                </div>
              </Show>
            </div>
          </div>

          {/* Modal Actions */}
          <div class="flex items-center justify-between border-t border-theme-border pt-3 flex-wrap gap-2">
            <span class="text-2xs text-theme-text/50 hidden sm:inline">
              Raw JSON is injected into LLM prompt for autonomous execution via Python 3 / terminal.
            </span>

            <div class="flex items-center gap-2 ml-auto">
              <button
                type="button"
                onClick={props.onClose}
                class="px-3.5 py-1.5 text-xs text-theme-text/60 hover:text-theme-text rounded-lg"
              >
                Cancel
              </button>
              <button
                type="button"
                onClick={handleSave}
                disabled={isSaving() || !!jsonError() || !name().trim()}
                class="px-4 py-1.5 text-xs bg-theme-cyan hover:bg-theme-cyan/90 disabled:opacity-50 text-theme-card font-bold rounded-lg shadow-sm transition-colors flex items-center gap-1.5"
              >
                <Show when={isSaving()}>
                  <Loader2 class="w-3.5 h-3.5 animate-spin" />
                </Show>
                <span>{props.isEditing ? 'Save Changes' : 'Save MCP Server'}</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </Show>
  );
};

