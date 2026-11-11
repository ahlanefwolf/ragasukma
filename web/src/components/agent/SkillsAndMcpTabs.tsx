import { Component, For, Show, createSignal } from 'solid-js';
import { 
  Trash2, 
  Plus, 
  Server, 
  Pencil, 
  CheckCircle2, 
  Copy, 
  Check, 
  FileCode,
  RefreshCw,
  Wrench,
  X,
  Sparkles,
  Cpu
} from 'lucide-solid';
import { McpServerConfig, McpServerTool, agentService, modelCatalogService } from '../../services';
import { SearchableSelect } from '../SearchableSelect';

export const McpSubTab: Component<{
  mcpServers: McpServerConfig[];
  activeSessionId?: string | null;
  activeSessionMcpIds?: string[];
  onToggleSessionMcp?: (sessionId: string, serverId: string) => void;
  onNewServer: (initialMode?: 'form' | 'json') => void;
  onEditServer: (server: McpServerConfig) => void;
  onDeleteServer: (id: string, name: string) => void;
  onRefreshServers?: () => void;
}> = (props) => {
  const [copiedServerId, setCopiedServerId] = createSignal<string | null>(null);

  // Tools viewer modal state
  const [selectedToolsServer, setSelectedToolsServer] = createSignal<McpServerConfig | null>(null);
  const [serverTools, setServerTools] = createSignal<McpServerTool[]>([]);
  const [isLoadingTools, setIsLoadingTools] = createSignal(false);
  const [selectedToolDetail, setSelectedToolDetail] = createSignal<McpServerTool | null>(null);

  // Sync tools modal state
  const [syncingServer, setSyncingServer] = createSignal<McpServerConfig | null>(null);
  const [syncModelInput, setSyncModelInput] = createSignal('');
  const [isSyncing, setIsSyncing] = createSignal(false);
  const [syncMessage, setSyncMessage] = createSignal<string | null>(null);
  const [availableModels, setAvailableModels] = createSignal<Array<{ id: string; label: string }>>([]);
  const [isRefreshingModels, setIsRefreshingModels] = createSignal(false);

  const refreshDynamicModels = async () => {
    setIsRefreshingModels(true);
    try {
      const models = await modelCatalogService.getModels().catch(() => []);
      const list = [...models];
      const current = syncModelInput();
      if (current && !list.some((model) => model.id === current)) {
        list.unshift({ id: current, label: `${current} (Current)` });
      }
      setAvailableModels(list);
    } finally {
      setIsRefreshingModels(false);
    }
  };

  const displayModels = () => {
    const list = [...availableModels()];
    const current = syncModelInput();
    if (current && !list.some((model) => model.id === current)) {
      list.unshift({ id: current, label: `${current} (Current)` });
    }
    return list;
  };

  const handleCopyConfig = (server: McpServerConfig) => {
    navigator.clipboard.writeText(server.config || '');
    setCopiedServerId(server.id);
    setTimeout(() => {
      setCopiedServerId(null);
    }, 2000);
  };

  const handleOpenToolsModal = async (server: McpServerConfig) => {
    setSelectedToolsServer(server);
    setSelectedToolDetail(null);
    setIsLoadingTools(true);
    try {
      const tools = await agentService.listMcpServerTools(server.id);
      setServerTools(tools);
      if (tools.length > 0) {
        setSelectedToolDetail(tools[0]);
      }
    } catch {
      setServerTools([]);
    } finally {
      setIsLoadingTools(false);
    }
  };

  const handleOpenSyncModal = (server: McpServerConfig) => {
    setSyncingServer(server);
    setSyncModelInput('');
    setSyncMessage(null);
    if (availableModels().length === 0) {
      void refreshDynamicModels();
    }
  };

  const handleExecuteSync = async () => {
    const s = syncingServer();
    if (!s) return;
    setIsSyncing(true);
    setSyncMessage(null);
    try {
      const res = await agentService.syncMcpServer(s.id, syncModelInput().trim() || undefined);
      if (res.ok) {
        setSyncMessage(`Synced ${res.tools_count ?? 0} tool(s) successfully!`);
        if (props.onRefreshServers) props.onRefreshServers();
      } else {
        setSyncMessage(`Sync error: ${res.error || 'Unknown error'}`);
      }
    } catch (e: any) {
      setSyncMessage(`Sync failed: ${e?.message || 'Request error'}`);
    } finally {
      setIsSyncing(false);
    }
  };

  const handleToggleEnabled = async (server: McpServerConfig) => {
    const newEnabled = server.is_enabled === 0;
    await agentService.saveMcpServer({
      id: server.id,
      name: server.name,
      config: server.config,
      is_enabled: newEnabled,
    });
    if (props.onRefreshServers) {
      props.onRefreshServers();
    }
  };

  const getConfigSummary = (config: string): string => {
    const lines = config.trim().split('\n').map(l => l.trim()).filter(Boolean);
    if (lines.length === 0) return 'Empty configuration';
    return lines.slice(0, 3).join(' | ');
  };

  return (
    <div class="space-y-4">
      <div class="bg-theme-card/60 border border-theme-border p-4 flex items-center justify-between shadow-sm flex-wrap gap-3">
        <div>
          <div class="flex items-center gap-2">
            <h3 class="text-sm font-semibold text-theme-text">Model Context Protocol (MCP) Servers</h3>
            <span class="text-xs px-2 py-0.5 font-mono bg-theme-bg border border-theme-border text-theme-cyan">
              {props.activeSessionMcpIds ? props.activeSessionMcpIds.length : 0}/{props.mcpServers.length} Active
            </span>
          </div>
          <p class="text-xs text-theme-text/50 mt-0.5">
            Configure external MCP servers and toggle active tools for the active session (synchronized with TUI).
          </p>
        </div>
        <div class="flex items-center gap-2 shrink-0">
          <button
            onClick={() => props.onRefreshServers && props.onRefreshServers()}
            class="px-3 py-1.5 text-xs bg-theme-bg hover:bg-theme-card border border-theme-border hover:border-theme-cyan/50 text-theme-text font-medium flex items-center gap-1.5 transition-colors shadow-sm"
            title="Refresh list"
          >
            <RefreshCw class="w-3.5 h-3.5 text-theme-cyan" />
            Refresh
          </button>
          <button
            onClick={() => props.onNewServer('json')}
            class="px-3.5 py-1.5 text-xs bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card font-bold flex items-center gap-1.5 transition-colors shadow-sm"
          >
            <Plus class="w-3.5 h-3.5" />
            Add MCP Server
          </button>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <For each={props.mcpServers}>
          {(server) => {
            const isSessionActive = () =>
              props.activeSessionMcpIds ? props.activeSessionMcpIds.includes(server.id) : server.is_enabled !== 0;

            const handleToggle = () => {
              if (props.activeSessionId && props.onToggleSessionMcp) {
                props.onToggleSessionMcp(props.activeSessionId, server.id);
              } else {
                handleToggleEnabled(server);
              }
            };

            return (
              <div class="bg-theme-card border border-theme-border p-4 flex flex-col justify-between space-y-3 group hover:border-theme-cyan/50 transition-colors shadow-sm">
                <div>
                  <div class="flex items-center justify-between mb-2">
                    <span class="text-xs font-semibold text-theme-text flex items-center gap-1.5">
                      <Server class="w-3.5 h-3.5 text-theme-cyan" />
                      {server.name}
                    </span>
                    <div class="flex items-center gap-1.5">
                      <button
                        onClick={handleToggle}
                        class={`text-2xs px-2 py-0.5 uppercase font-semibold border flex items-center gap-1 transition-colors ${
                          isSessionActive()
                            ? 'bg-theme-cyan/15 text-theme-cyan border-theme-cyan/30 hover:bg-theme-cyan/25'
                            : 'bg-theme-border/20 text-theme-text/40 border-theme-border/40 hover:bg-theme-border/40'
                        }`}
                        title={props.activeSessionId ? "Toggle active for current session" : "Click to toggle"}
                      >
                        {isSessionActive() ? 'Active in Session' : 'Inactive'}
                      </button>

                      <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity ml-1">
                        <button
                          onClick={() => handleCopyConfig(server)}
                          class="p-1 text-theme-text/70 hover:text-theme-cyan transition-colors"
                          title="Copy Configuration"
                        >
                          <Show when={copiedServerId() === server.id} fallback={<Copy class="w-3.5 h-3.5" />}>
                            <Check class="w-3.5 h-3.5 text-gi-status-success" />
                          </Show>
                        </button>
                        <button
                          onClick={() => props.onEditServer(server)}
                          class="p-1 text-theme-text/70 hover:text-theme-cyan transition-colors"
                          title="Edit MCP Server"
                        >
                          <Pencil class="w-3.5 h-3.5" />
                        </button>
                        <button
                          onClick={() => props.onDeleteServer(server.id, server.name)}
                          class="p-1 text-theme-text/70 hover:text-theme-rose transition-colors"
                          title="Delete MCP Server"
                        >
                          <Trash2 class="w-3.5 h-3.5" />
                        </button>
                      </div>
                    </div>
                  </div>

                  {/* Config preview summary */}
                  <div class="text-xs font-mono text-theme-text/70 bg-theme-bg/60 p-2.5 border border-theme-border/60 break-all space-y-1">
                    <div class="text-2xs text-theme-text/50 truncate font-semibold uppercase">Configuration (Markdown/JSON)</div>
                    <div class="text-2xs text-theme-text/80 line-clamp-3 whitespace-pre-wrap font-mono">
                      {getConfigSummary(server.config)}
                    </div>
                  </div>
                </div>

                {/* Footer with Tools & Sync buttons */}
                <div class="pt-2 border-t border-theme-border/60 flex items-center justify-between gap-2">
                  <div class="flex items-center gap-2">
                    <button
                      onClick={() => handleOpenToolsModal(server)}
                      class="px-2.5 py-1 text-2xs bg-theme-bg hover:bg-theme-card border border-theme-border hover:border-theme-cyan/50 text-theme-text font-medium flex items-center gap-1.5 transition-colors shrink-0"
                      title="View discovered tools"
                    >
                      <Wrench class="w-3 h-3 text-theme-cyan" />
                      Tools
                    </button>
                    <button
                      onClick={() => handleOpenSyncModal(server)}
                      class="px-2.5 py-1 text-2xs bg-theme-bg hover:bg-theme-card border border-theme-border hover:border-theme-cyan/50 text-theme-text font-medium flex items-center gap-1.5 transition-colors shrink-0"
                      title="Discover and sync tools with AI runner"
                    >
                      <Sparkles class="w-3 h-3 text-theme-amber" />
                      Sync
                    </button>
                  </div>

                  <div class="flex items-center gap-1.5 text-2xs text-theme-text/50 font-medium truncate">
                    <span class="font-mono text-theme-text/40">{server.id}</span>
                  </div>
                </div>
              </div>
            );
          }}
        </For>
        <Show when={props.mcpServers.length === 0}>
          <div class="col-span-full py-12 text-center text-theme-text/70 bg-theme-card/40 border border-theme-border">
            <Server class="w-10 h-10 mx-auto text-theme-text/20 mb-2" />
            <p class="text-xs font-semibold">No MCP servers registered yet.</p>
            <p class="text-xs text-theme-text/60 mt-1 max-w-sm mx-auto">
              Add Model Context Protocol servers to give your autonomous coding agent access to local tools, web search, GitHub, and custom CLI utilities.
            </p>
            <div class="mt-3 flex items-center justify-center gap-2">
              <button
                onClick={() => props.onNewServer('json')}
                class="px-3.5 py-1.5 text-xs bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card font-bold inline-flex items-center gap-1.5 transition-colors shadow-sm"
              >
                <Plus class="w-3.5 h-3.5" />
                Add First MCP Server
              </button>
            </div>
          </div>
        </Show>
      </div>

      {/* ── TOOLS VIEWER MODAL ─────────────────────────────────────── */}
      <Show when={selectedToolsServer()}>
        <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
          <div class="bg-theme-card border border-theme-border w-full max-w-3xl max-h-[85vh] flex flex-col shadow-2xl">
            <div class="flex items-center justify-between p-4 border-b border-theme-border">
              <div class="flex items-center gap-2">
                <Wrench class="w-4 h-4 text-theme-cyan" />
                <h3 class="text-sm font-semibold text-theme-text">
                  Tools for {selectedToolsServer()?.name}
                </h3>
                <span class="text-2xs bg-theme-bg px-2 py-0.5 border border-theme-border text-theme-text/60">
                  {serverTools().length} tool(s)
                </span>
              </div>
              <button
                onClick={() => setSelectedToolsServer(null)}
                class="p-1 text-theme-text/60 hover:text-theme-text"
              >
                <X class="w-4 h-4" />
              </button>
            </div>

            <div class="flex-1 overflow-hidden grid grid-cols-1 md:grid-cols-3 divide-y md:divide-y-0 md:divide-x divide-theme-border min-h-[350px]">
              {/* Tool list */}
              <div class="overflow-y-auto p-2 space-y-1">
                <Show when={!isLoadingTools()} fallback={<div class="p-4 text-xs text-theme-text/40">Loading tools...</div>}>
                  <For each={serverTools()}>
                    {(tool) => (
                      <button
                        onClick={() => setSelectedToolDetail(tool)}
                        class={`w-full text-left p-2 text-xs font-mono transition-colors border ${
                          selectedToolDetail()?.name === tool.name
                            ? 'bg-theme-cyan/15 text-theme-cyan border-theme-cyan/40 font-semibold'
                            : 'bg-theme-bg/40 hover:bg-theme-bg text-theme-text/80 border-transparent'
                        }`}
                      >
                        {tool.name}
                      </button>
                    )}
                  </For>
                  <Show when={serverTools().length === 0}>
                    <div class="p-4 text-center text-xs text-theme-text/40">
                      No tools synced yet. Click "Sync" to discover tools.
                    </div>
                  </Show>
                </Show>
              </div>

              {/* Tool detail spec */}
              <div class="md:col-span-2 overflow-y-auto p-4 bg-theme-bg/30">
                <Show when={selectedToolDetail()} fallback={<div class="text-xs text-theme-text/40">Select a tool to view specification</div>}>
                  <div class="space-y-3">
                    <div class="text-xs font-bold text-theme-cyan font-mono border-b border-theme-border pb-1">
                      {selectedToolDetail()?.name}
                    </div>
                    <pre class="text-2xs font-mono text-theme-text/80 bg-theme-bg p-3 border border-theme-border whitespace-pre-wrap overflow-x-auto">
                      {selectedToolDetail()?.config}
                    </pre>
                  </div>
                </Show>
              </div>
            </div>

            <div class="p-3 border-t border-theme-border flex justify-end">
              <button
                onClick={() => setSelectedToolsServer(null)}
                class="px-4 py-1.5 text-xs bg-theme-bg hover:bg-theme-border text-theme-text font-medium border border-theme-border"
              >
                Close
              </button>
            </div>
          </div>
        </div>
      </Show>

      {/* ── SYNC TOOLS MODAL ───────────────────────────────────────── */}
      <Show when={syncingServer()}>
        <div class="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm">
          <div class="bg-theme-card border border-theme-border w-full max-w-md p-5 flex flex-col space-y-4 shadow-2xl">
            <div class="flex items-center justify-between">
              <div class="flex items-center gap-2">
                <Sparkles class="w-4 h-4 text-theme-amber" />
                <h3 class="text-sm font-semibold text-theme-text">
                  Sync Tools: {syncingServer()?.name}
                </h3>
              </div>
              <button
                onClick={() => setSyncingServer(null)}
                class="p-1 text-theme-text/60 hover:text-theme-text"
              >
                <X class="w-4 h-4" />
              </button>
            </div>

            <p class="text-xs text-theme-text/60">
              Run AI discovery runner to inspect this MCP server and register its tool definitions into the project database.
            </p>

            <div class="space-y-1.5">
              <label class="text-2xs font-semibold text-theme-text/80 uppercase flex items-center justify-between">
                <span class="flex items-center gap-1.5 font-bold">
                  <Cpu class="w-3.5 h-3.5 text-theme-cyan" />
                  Select AI Discovery Model
                </span>
                <span class="text-3xs text-theme-cyan font-mono bg-theme-cyan/10 px-1.5 py-0.5 border border-theme-cyan/20">
                  Optional
                </span>
              </label>
              <div class="flex items-center gap-1.5">
                <div class="flex-1 min-w-0">
                  <SearchableSelect
                    value={syncModelInput()}
                    onChange={(val) => setSyncModelInput(val)}
                    options={[
                      { value: '', label: 'Default / System Recommended' },
                      ...displayModels().map((model) => ({
                        value: model.id,
                        label: model.label,
                        badge: model.id.startsWith('ag/')
                          ? 'Antigravity'
                          : model.id.startsWith('cx/')
                          ? 'Codex'
                          : model.id.includes('/')
                          ? 'API'
                          : undefined,
                      })),
                    ]}
                    placeholder="Select or search Model..."
                    searchPlaceholder="Filter AI model..."
                    buttonClass="w-full flex items-center justify-between px-3 py-2 bg-theme-bg border border-theme-border text-xs font-mono text-theme-text font-medium hover:border-theme-cyan transition-colors"
                    dropdownClass="absolute left-0 mt-1 w-full bg-theme-surface border border-theme-border shadow-2xl z-50 overflow-hidden flex flex-col max-h-56"
                  />
                </div>
                <button
                  type="button"
                  onClick={() => void refreshDynamicModels()}
                  disabled={isRefreshingModels()}
                  title="Refresh available models"
                  class="px-2.5 py-2 bg-theme-bg hover:bg-theme-card border border-theme-border text-theme-cyan transition-colors flex items-center justify-center shrink-0 disabled:opacity-50"
                >
                  <RefreshCw class={`w-3.5 h-3.5 ${isRefreshingModels() ? 'animate-spin' : ''}`} />
                </button>
              </div>
            </div>

            <Show when={syncMessage()}>
              <div class="text-2xs p-2.5 bg-theme-bg border border-theme-border font-mono text-theme-cyan">
                {syncMessage()}
              </div>
            </Show>

            <div class="flex items-center justify-end gap-2 pt-2">
              <button
                onClick={() => setSyncingServer(null)}
                class="px-3.5 py-1.5 text-xs bg-theme-bg hover:bg-theme-border text-theme-text font-medium border border-theme-border"
              >
                Cancel
              </button>
              <button
                disabled={isSyncing()}
                onClick={handleExecuteSync}
                class="px-4 py-1.5 text-xs bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card font-bold flex items-center gap-1.5 disabled:opacity-50"
              >
                <Show when={isSyncing()} fallback={<Sparkles class="w-3.5 h-3.5" />}>
                  <RefreshCw class="w-3.5 h-3.5 animate-spin" />
                </Show>
                {isSyncing() ? 'Syncing...' : 'Start Discovery'}
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};