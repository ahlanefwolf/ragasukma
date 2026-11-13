import { Component, For, Show, createSignal } from 'solid-js';
import {
  Server,
  CheckCircle2,
  Copy,
  Check,
  RefreshCw,
  Wrench,
  X,
} from 'lucide-solid';
import { McpServerConfig, McpServerTool, agentService } from '../../services';

export const McpSubTab: Component<{
  mcpServers: McpServerConfig[];
  activeSessionId?: string | null;
  activeSessionMcpIds?: string[];
  onRefreshServers?: () => void;
}> = (props) => {
  const [copiedServerId, setCopiedServerId] = createSignal<string | null>(null);

  // Tools viewer modal state
  const [selectedToolsServer, setSelectedToolsServer] = createSignal<McpServerConfig | null>(null);
  const [serverTools, setServerTools] = createSignal<McpServerTool[]>([]);
  const [isLoadingTools, setIsLoadingTools] = createSignal(false);
  const [selectedToolDetail, setSelectedToolDetail] = createSignal<McpServerTool | null>(null);

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

  const getConfigSummary = (config: string): string => {
    const lines = config
      .trim()
      .split('\n')
      .map((l) => l.trim())
      .filter(Boolean);
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
            MCP servers are registered, configured, and activated automatically via chat prompts.
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
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
        <Show
          when={props.mcpServers.length > 0}
          fallback={
            <div class="col-span-full bg-theme-card/40 border border-theme-border border-dashed p-12 text-center">
              <div class="w-12 h-12 bg-theme-bg border border-theme-border flex items-center justify-center mx-auto mb-3 text-theme-text/40">
                <Server class="w-6 h-6" />
              </div>
              <h4 class="text-sm font-semibold text-theme-text">No MCP Servers Registered</h4>
              <p class="text-xs text-theme-text/50 max-w-md mx-auto mt-1">
                Ask the agent in chat to register or activate an MCP server for your session.
              </p>
            </div>
          }
        >
          <For each={props.mcpServers}>
            {(server) => {
              const isActiveInSession = () =>
                Boolean(props.activeSessionId && props.activeSessionMcpIds?.includes(server.id));

              return (
                <div
                  class={`bg-theme-card border transition-all flex flex-col justify-between p-4 shadow-sm relative overflow-hidden group ${
                    isActiveInSession()
                      ? 'border-theme-cyan/40 hover:border-theme-cyan/70'
                      : 'border-theme-border/60 opacity-75 hover:opacity-100'
                  }`}
                >
                  <div
                    class={`absolute top-0 left-0 w-1 h-full ${
                      isActiveInSession() ? 'bg-theme-cyan' : 'bg-theme-border'
                    }`}
                  />

                  <div>
                    <div class="flex items-start justify-between gap-2 pl-2">
                      <div class="min-w-0">
                        <div class="flex items-center gap-2">
                          <span class="font-bold text-sm text-theme-text truncate" title={server.name}>
                            {server.name}
                          </span>
                        </div>
                        <div class="flex items-center gap-1.5 mt-1">
                          <span class="text-2xs px-1.5 py-0.5 font-mono bg-theme-bg border border-theme-border text-theme-text/70">
                            ID: {server.id}
                          </span>
                        </div>
                      </div>

                      <span
                        class={`px-2 py-1 text-2xs font-mono uppercase tracking-wider border flex items-center gap-1 shrink-0 ${
                          isActiveInSession()
                            ? 'bg-emerald-950/50 text-emerald-400 border-emerald-700/50'
                            : 'bg-theme-bg text-theme-text/50 border-theme-border'
                        }`}
                      >
                        <Show when={isActiveInSession()}>
                          <CheckCircle2 class="w-3 h-3" />
                        </Show>
                        {isActiveInSession() ? 'Active in Session' : 'Inactive'}
                      </span>
                    </div>

                    <div class="mt-3 pl-2">
                      <div class="bg-theme-bg/90 border border-theme-border p-2.5 font-mono text-2xs text-theme-text/70 overflow-hidden text-ellipsis whitespace-nowrap">
                        {getConfigSummary(server.config || '')}
                      </div>
                    </div>
                  </div>

                  <div class="flex items-center justify-between gap-2 mt-4 pt-3 border-t border-theme-border/60 pl-2">
                    <div class="flex items-center gap-1.5">
                      <button
                        onClick={() => handleOpenToolsModal(server)}
                        class="px-2 py-1 text-2xs bg-theme-bg hover:bg-theme-border text-theme-cyan border border-theme-border flex items-center gap-1 transition-colors font-medium"
                        title="View discovered tools"
                      >
                        <Wrench class="w-3 h-3" />
                        Tools
                      </button>
                      <button
                        onClick={() => handleCopyConfig(server)}
                        class="px-2 py-1 text-2xs bg-theme-bg hover:bg-theme-border text-theme-text/70 hover:text-theme-text border border-theme-border flex items-center gap-1 transition-colors"
                        title="Copy Raw Config"
                      >
                        <Show when={copiedServerId() === server.id} fallback={<Copy class="w-3 h-3" />}>
                          <Check class="w-3 h-3 text-emerald-400" />
                        </Show>
                        {copiedServerId() === server.id ? 'Copied' : 'Copy'}
                      </button>
                    </div>
                  </div>
                </div>
              );
            }}
          </For>
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

            <div class="flex-1 min-h-0 flex divide-x divide-theme-border overflow-hidden">
              <div class="w-1/3 overflow-y-auto p-2 space-y-1 bg-theme-bg/50">
                <Show
                  when={!isLoadingTools()}
                  fallback={<div class="p-4 text-xs text-theme-text/50">Loading tools...</div>}
                >
                  <Show
                    when={serverTools().length > 0}
                    fallback={<div class="p-4 text-xs text-theme-text/40">No tools registered yet.</div>}
                  >
                    <For each={serverTools()}>
                      {(tool) => (
                        <button
                          onClick={() => setSelectedToolDetail(tool)}
                          class={`w-full text-left px-2.5 py-2 text-xs font-mono truncate border transition-colors ${
                            selectedToolDetail()?.name === tool.name
                              ? 'bg-theme-cyan/10 border-theme-cyan/50 text-theme-cyan font-semibold'
                              : 'border-transparent hover:bg-theme-card text-theme-text/80'
                          }`}
                        >
                          {tool.name}
                        </button>
                      )}
                    </For>
                  </Show>
                </Show>
              </div>

              <div class="w-2/3 overflow-y-auto p-4 bg-theme-bg/30">
                <Show
                  when={selectedToolDetail()}
                  fallback={<div class="text-xs text-theme-text/40">Select a tool to view specification</div>}
                >
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
    </div>
  );
};
