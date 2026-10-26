import { Component, For, Show, createSignal } from 'solid-js';
import { Sparkles, Trash2, Plus, Server, Pencil, Wrench, Loader2, CheckCircle2, AlertCircle, Terminal, Globe } from 'lucide-solid';
import { PromptSkill, McpServerConfig } from '../../services';

export const SkillsSubTab: Component<{
  skills: PromptSkill[];
  onNewSkill: () => void;
  onEditSkill: (skill: PromptSkill) => void;
  onDeleteSkill: (id: string, name: string) => void;
}> = (props) => {
  return (
    <div class="space-y-4">
      <div class="flex items-center justify-between bg-theme-card/60 border border-theme-border p-4 rounded-xl shadow-sm">
        <div>
          <h3 class="text-sm font-semibold text-theme-text">Prompt Skills Library</h3>
          <p class="text-xs text-theme-text/50 mt-0.5">
            Store custom personas, coding rules, or system prompts that can be attached to any workspace project.
          </p>
        </div>
        <button
          onClick={props.onNewSkill}
          class="px-3.5 py-1.5 text-xs bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card rounded-lg font-bold flex items-center gap-1.5 transition-colors shadow-sm"
        >
          <Plus class="w-3.5 h-3.5" />
          New Prompt Skill
        </button>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
        <For each={props.skills}>
          {(skill) => (
            <div class="bg-theme-card border border-theme-border rounded-xl p-4 flex flex-col justify-between space-y-3 group hover:border-theme-cyan/50 transition-colors shadow-sm">
              <div>
                <div class="flex items-center justify-between mb-1.5">
                  <span class="text-xs font-semibold text-theme-text flex items-center gap-1.5">
                    <Sparkles class="w-3.5 h-3.5 text-theme-cyan" />
                    {skill.name}
                  </span>
                  <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-all">
                    <button
                      onClick={() => props.onEditSkill(skill)}
                      class="p-1 text-theme-text/70 hover:text-theme-cyan rounded transition-colors"
                      title="Edit Skill"
                    >
                      <Pencil class="w-3.5 h-3.5" />
                    </button>
                    <button
                      onClick={() => props.onDeleteSkill(skill.id, skill.name)}
                      class="p-1 text-theme-text/70 hover:text-theme-rose rounded transition-colors"
                      title="Delete Skill"
                    >
                      <Trash2 class="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>
                <span class="text-2xs font-mono bg-theme-cyan/10 px-2 py-0.5 rounded text-theme-cyan border border-theme-cyan/20">
                  @{skill.slug}
                </span>
                <p class="text-xs text-theme-text/60 mt-2 line-clamp-2">
                  {skill.description || 'No description'}
                </p>
              </div>

              <div class="pt-2 border-t border-theme-border/60">
                <div class="text-2xs text-theme-text/70 uppercase tracking-wider mb-1 font-mono">System Prompt Preview:</div>
                <p class="text-xs font-mono text-theme-text/80 line-clamp-3 bg-theme-bg/60 p-2 rounded border border-theme-border/60">
                  {skill.system_prompt}
                </p>
              </div>
            </div>
          )}
        </For>
        <Show when={props.skills.length === 0}>
          <div class="col-span-full py-12 text-center text-theme-text/70 bg-theme-card/40 border border-theme-border rounded-xl">
            <Sparkles class="w-10 h-10 mx-auto text-theme-text/20 mb-2" />
            <p class="text-xs">No custom prompt skills created yet.</p>
            <p class="text-xs text-theme-text/60 mt-1">
              Add skills like "Senior Rust Reviewer" or "SolidJS Expert" to inject specialized guidance.
            </p>
          </div>
        </Show>
      </div>
    </div>
  );
};

export const McpSubTab: Component<{
  mcpServers: McpServerConfig[];
  onNewServer: () => void;
  onEditServer: (server: McpServerConfig) => void;
  onDeleteServer: (id: string, name: string) => void;
  onTestServer: (server: McpServerConfig) => Promise<{ ok: boolean; tools_count?: number; error?: string }>;
}> = (props) => {
  const [testStates, setTestStates] = createSignal<Record<string, { testing: boolean; result?: { ok: boolean; tools_count?: number; error?: string } }>>({});

  const handleTest = async (server: McpServerConfig) => {
    setTestStates((prev) => ({ ...prev, [server.id]: { testing: true } }));
    try {
      const res = await props.onTestServer(server);
      setTestStates((prev) => ({ ...prev, [server.id]: { testing: false, result: res } }));
    } catch (e: any) {
      setTestStates((prev) => ({
        ...prev,
        [server.id]: { testing: false, result: { ok: false, error: e?.message || 'Failed to connect' } },
      }));
    }
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
      <div class="bg-theme-card/60 border border-theme-border p-4 rounded-xl flex items-center justify-between shadow-sm">
        <div>
          <h3 class="text-sm font-semibold text-theme-text">Model Context Protocol (MCP) Servers</h3>
          <p class="text-xs text-theme-text/50 mt-0.5">
            Open standard connecting AI agents to external tools (Filesystem, Terminal, Postgres, GitHub, Brave Search).
          </p>
        </div>
        <button
          onClick={props.onNewServer}
          class="px-3.5 py-1.5 text-xs bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card rounded-lg font-bold flex items-center gap-1.5 transition-colors shadow-sm shrink-0"
        >
          <Plus class="w-3.5 h-3.5" />
          Add MCP Server
        </button>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <For each={props.mcpServers}>
          {(server) => {
            const cardState = () => testStates()[server.id];
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
                          : 'bg-indigo-500/15 text-indigo-300 border-indigo-500/30'
                      }`}>
                        <Show when={isStdio()} fallback={<Globe class="w-2.5 h-2.5" />}>
                          <Terminal class="w-2.5 h-2.5" />
                        </Show>
                        {server.transport}
                      </span>
                      <div class="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity ml-1">
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
                  <div class="text-xs font-mono text-theme-text/70 bg-theme-bg/60 p-2 rounded border border-theme-border/60 break-all space-y-1">
                    <Show when={isStdio()} fallback={
                      <div><span class="text-theme-text/40">URL:</span> {server.url || '-'}</div>
                    }>
                      <div><span class="text-theme-text/40">Cmd:</span> {server.command || '-'}</div>
                      <Show when={getArgsSummary(server.args)}>
                        <div class="text-2xs text-theme-text/50 truncate">
                          <span class="text-theme-text/40">Args:</span> {getArgsSummary(server.args)}
                        </div>
                      </Show>
                    </Show>
                  </div>
                </div>

                {/* Footer with Test Connection & Live Status */}
                <div class="pt-2 border-t border-theme-border/60 flex items-center justify-between gap-2">
                  <div class="flex-1 min-w-0">
                    <Show when={cardState()?.result}>
                      <div class={`text-2xs flex items-center gap-1 truncate ${
                        cardState()?.result?.ok
                          ? 'text-emerald-400'
                          : 'text-rose-400'
                      }`}>
                        <Show when={cardState()?.result?.ok} fallback={<AlertCircle class="w-3 h-3 shrink-0" />}>
                          <CheckCircle2 class="w-3 h-3 shrink-0" />
                        </Show>
                        <span class="truncate">
                          {cardState()?.result?.ok
                            ? `Active (${cardState()?.result?.tools_count ?? 0} tools ready)`
                            : cardState()?.result?.error || 'Failed'}
                        </span>
                      </div>
                    </Show>
                  </div>

                  <button
                    onClick={() => handleTest(server)}
                    disabled={cardState()?.testing}
                    class="px-2.5 py-1 text-2xs bg-theme-bg hover:bg-theme-card border border-theme-border hover:border-theme-cyan/50 text-theme-text rounded-md font-medium flex items-center gap-1 transition-colors shrink-0"
                    title="Ping and test tool discovery for this server"
                  >
                    <Show when={cardState()?.testing} fallback={<Wrench class="w-3 h-3 text-theme-cyan" />}>
                      <Loader2 class="w-3 h-3 animate-spin text-theme-cyan" />
                    </Show>
                    <span>{cardState()?.testing ? 'Testing...' : 'Test Connection'}</span>
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
            <button
              onClick={props.onNewServer}
              class="mt-3 px-3.5 py-1.5 text-xs bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card rounded-lg font-bold inline-flex items-center gap-1.5 transition-colors shadow-sm"
            >
              <Plus class="w-3.5 h-3.5" />
              Add First MCP Server
            </button>
          </div>
        </Show>
      </div>
    </div>
  );
};
