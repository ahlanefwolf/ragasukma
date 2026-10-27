import { Component, For, Show, createSignal } from 'solid-js';
import { 
  Trash2, 
  Plus, 
  Server, 
  Pencil, 
  CheckCircle2, 
  Terminal, 
  Globe, 
  Copy, 
  Check, 
  FileCode
} from 'lucide-solid';
import { McpServerConfig } from '../../services';

export const McpSubTab: Component<{
  mcpServers: McpServerConfig[];
  onNewServer: (initialMode?: 'form' | 'json') => void;
  onEditServer: (server: McpServerConfig) => void;
  onDeleteServer: (id: string, name: string) => void;
  onRefreshServers?: () => void;
}> = (props) => {
  const [copiedServerId, setCopiedServerId] = createSignal<string | null>(null);

  const handleCopyJson = (server: McpServerConfig) => {
    let argsArr: any = undefined;
    try {
      const parsed = JSON.parse(server.args || '[]');
      if (Array.isArray(parsed) && parsed.length > 0) argsArr = parsed;
    } catch {}

    let envObj: Record<string, any> = {};
    try {
      const parsed = JSON.parse(server.env || '{}');
      if (typeof parsed === 'object' && parsed !== null) envObj = parsed;
    } catch {}

    const payload: any = {
      name: server.name,
      transport: server.transport,
    };
    if (server.transport === 'stdio') {
      if (server.command) payload.command = server.command;
      if (argsArr) payload.args = argsArr;
    } else {
      if (server.url) payload.url = server.url;
    }

    for (const [k, v] of Object.entries(envObj)) {
      payload[k] = v;
    }

    navigator.clipboard.writeText(JSON.stringify(payload, null, 2));
    setCopiedServerId(server.id);
    setTimeout(() => {
      setCopiedServerId(null);
    }, 2000);
  };

  const getArgsSummary = (rawArgs: string): string => {
    try {
      const parsed = JSON.parse(rawArgs);
      if (Array.isArray(parsed) && parsed.length > 0) {
        return parsed.join(' ');
      }
    } catch {}
    return rawArgs || '';
  };

  return (
    <div class="space-y-4">
      <div class="bg-theme-card/60 border border-theme-border p-4 rounded-xl flex items-center justify-between shadow-sm flex-wrap gap-3">
        <div>
          <h3 class="text-sm font-semibold text-theme-text">Model Context Protocol (MCP) Servers</h3>
          <p class="text-xs text-theme-text/50 mt-0.5">
            Connect autonomous coding agents to custom external tools (Postgres, GitHub, Brave Search, Filesystem, or custom scripts via stdio/HTTP/SSE).
          </p>
        </div>
        <div class="flex items-center gap-2 shrink-0">
          <button
            onClick={() => props.onNewServer('json')}
            class="px-3 py-1.5 text-xs bg-theme-bg hover:bg-theme-card border border-theme-border hover:border-theme-cyan/50 text-theme-text rounded-lg font-medium flex items-center gap-1.5 transition-colors shadow-sm"
            title="Paste standard MCP JSON or Claude Desktop configuration"
          >
            <FileCode class="w-3.5 h-3.5 text-theme-cyan" />
            Paste JSON Config
          </button>
          <button
            onClick={() => props.onNewServer('json')}
            class="px-3.5 py-1.5 text-xs bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card rounded-lg font-bold flex items-center gap-1.5 transition-colors shadow-sm"
          >
            <Plus class="w-3.5 h-3.5" />
            Add MCP Server
          </button>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <For each={props.mcpServers}>
          {(server) => {
            const isStdio = () => server.transport === 'stdio';

            return (
              <div class="bg-theme-card border border-theme-border rounded-xl p-4 flex flex-col justify-between space-y-3 group hover:border-theme-cyan/50 transition-colors shadow-sm">
                <div>
                  <div class="flex items-center justify-between mb-2">
                    <span class="text-xs font-semibold text-theme-text flex items-center gap-1.5">
                      <Server class="w-3.5 h-3.5 text-theme-cyan" />
                      {server.name}
                    </span>
                    <div class="flex items-center gap-1">
                      <span class={`text-2xs px-2 py-0.5 rounded uppercase font-semibold border flex items-center gap-1 ${
                        isStdio()
                          ? 'bg-theme-cyan/15 text-theme-cyan border-theme-cyan/30'
                          : 'bg-gi-status-info/15 text-gi-status-info border-gi-status-info/30'
                      }`}>
                        <Show when={isStdio()} fallback={<Globe class="w-2.5 h-2.5" />}>
                          <Terminal class="w-2.5 h-2.5" />
                        </Show>
                        {server.transport}
                      </span>
                      <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity ml-1">
                        <button
                          onClick={() => handleCopyJson(server)}
                          class="p-1 text-theme-text/70 hover:text-theme-cyan rounded transition-colors"
                          title="Copy Server Config as JSON"
                        >
                          <Show when={copiedServerId() === server.id} fallback={<Copy class="w-3.5 h-3.5" />}>
                            <Check class="w-3.5 h-3.5 text-gi-status-success" />
                          </Show>
                        </button>
                        <button
                          onClick={() => props.onEditServer(server)}
                          class="p-1 text-theme-text/70 hover:text-theme-cyan rounded transition-colors"
                          title="Edit MCP Server"
                        >
                          <Pencil class="w-3.5 h-3.5" />
                        </button>
                        <button
                          onClick={() => props.onDeleteServer(server.id, server.name)}
                          class="p-1 text-theme-text/70 hover:text-theme-rose rounded transition-colors"
                          title="Delete MCP Server"
                        >
                          <Trash2 class="w-3.5 h-3.5" />
                        </button>
                      </div>
                    </div>
                  </div>

                  {/* Command or URL details */}
                  <div class="text-xs font-mono text-theme-text/70 bg-theme-bg/60 p-2.5 rounded-lg border border-theme-border/60 break-all space-y-1">
                    <Show when={isStdio()} fallback={
                      <div><span class="text-theme-cyan/80 font-semibold">URL:</span> {server.url || '-'}</div>
                    }>
                      <div><span class="text-theme-cyan/80 font-semibold">Cmd:</span> {server.command || '-'}</div>
                      <Show when={getArgsSummary(server.args)}>
                        <div class="text-2xs text-theme-text/60 truncate">
                          <span class="text-theme-text/40">Args:</span> {getArgsSummary(server.args)}
                        </div>
                      </Show>
                    </Show>
                  </div>
                </div>

                {/* Footer with Prompt Injection Status & Copy JSON */}
                <div class="pt-2 border-t border-theme-border/60 flex items-center justify-between gap-2">
                  <div class="flex items-center gap-1.5 text-2xs text-theme-cyan/90 font-medium truncate">
                    <CheckCircle2 class="w-3.5 h-3.5 text-gi-status-success shrink-0" />
                    <span class="truncate">Active & Injected to Prompt</span>
                  </div>

                  <button
                    onClick={() => handleCopyJson(server)}
                    class="px-2.5 py-1 text-2xs bg-theme-bg hover:bg-theme-card border border-theme-border hover:border-theme-cyan/50 text-theme-text/80 hover:text-theme-text rounded-md font-medium flex items-center gap-1.5 transition-colors shrink-0"
                    title="Copy raw JSON configuration to clipboard"
                  >
                    <Show when={copiedServerId() === server.id} fallback={<Copy class="w-3 h-3 text-theme-text/60" />}>
                      <Check class="w-3 h-3 text-gi-status-success" />
                    </Show>
                    <span>{copiedServerId() === server.id ? 'Copied' : 'Copy JSON'}</span>
                  </button>
                </div>
              </div>
            );
          }}
        </For>
        <Show when={props.mcpServers.length === 0}>
          <div class="col-span-full py-12 text-center text-theme-text/70 bg-theme-card/40 border border-theme-border rounded-xl">
            <Server class="w-10 h-10 mx-auto text-theme-text/20 mb-2" />
            <p class="text-xs font-semibold">No MCP servers registered yet.</p>
            <p class="text-xs text-theme-text/60 mt-1 max-w-sm mx-auto">
              Add Model Context Protocol servers to give your autonomous coding agent access to local databases, web search, GitHub tools, and custom CLI utilities.
            </p>
            <div class="mt-3 flex items-center justify-center gap-2">
              <button
                onClick={() => props.onNewServer('json')}
                class="px-3.5 py-1.5 text-xs bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card rounded-lg font-bold inline-flex items-center gap-1.5 transition-colors shadow-sm"
              >
                <Plus class="w-3.5 h-3.5" />
                Add First MCP Server
              </button>
              <button
                onClick={() => props.onNewServer('json')}
                class="px-3.5 py-1.5 text-xs bg-theme-bg hover:bg-theme-card border border-theme-border hover:border-theme-cyan/50 text-theme-text rounded-lg font-medium inline-flex items-center gap-1.5 transition-colors shadow-sm"
              >
                <FileCode class="w-3.5 h-3.5 text-theme-cyan" />
                Paste JSON Config
              </button>
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
};
