import { Component, For, Show } from 'solid-js';
import { Sparkles, Trash2, Plus, Server, Pencil } from 'lucide-solid';
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
                      class="p-1 text-theme-text/40 hover:text-theme-cyan rounded transition-colors"
                      title="Edit Skill"
                    >
                      <Pencil class="w-3.5 h-3.5" />
                    </button>
                    <button
                      onClick={() => props.onDeleteSkill(skill.id, skill.name)}
                      class="p-1 text-theme-text/40 hover:text-theme-rose rounded transition-colors"
                      title="Delete Skill"
                    >
                      <Trash2 class="w-3.5 h-3.5" />
                    </button>
                  </div>
                </div>
                <span class="text-[10px] font-mono bg-theme-cyan/10 px-2 py-0.5 rounded text-theme-cyan border border-theme-cyan/20">
                  @{skill.slug}
                </span>
                <p class="text-xs text-theme-text/60 mt-2 line-clamp-2">
                  {skill.description || 'No description'}
                </p>
              </div>

              <div class="pt-2 border-t border-theme-border/60">
                <div class="text-[10px] text-theme-text/40 uppercase tracking-wider mb-1 font-mono">System Prompt Preview:</div>
                <p class="text-[11px] font-mono text-theme-text/80 line-clamp-3 bg-theme-bg/60 p-2 rounded border border-theme-border/60">
                  {skill.system_prompt}
                </p>
              </div>
            </div>
          )}
        </For>
        <Show when={props.skills.length === 0}>
          <div class="col-span-full py-12 text-center text-theme-text/40 bg-theme-card/40 border border-theme-border rounded-xl">
            <Sparkles class="w-10 h-10 mx-auto text-theme-text/20 mb-2" />
            <p class="text-xs">No custom prompt skills created yet.</p>
            <p class="text-[11px] text-theme-text/30 mt-1">
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
}> = (props) => {
  return (
    <div class="space-y-4">
      <div class="bg-theme-card/60 border border-theme-border p-4 rounded-xl flex items-center justify-between shadow-sm">
        <div>
          <h3 class="text-sm font-semibold text-theme-text">Model Context Protocol (MCP) Servers</h3>
          <p class="text-xs text-theme-text/50 mt-0.5">
            Open standard connecting AI agents to external tools (Filesystem, Terminal, Postgres, GitHub, Brave Search).
          </p>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <For each={props.mcpServers}>
          {(server) => (
            <div class="bg-theme-card border border-theme-border rounded-xl p-4 space-y-2 shadow-sm">
              <div class="flex items-center justify-between">
                <span class="text-xs font-semibold text-theme-text flex items-center gap-1.5">
                  <Server class="w-3.5 h-3.5 text-theme-cyan" />
                  {server.name}
                </span>
                <span class="text-[10px] px-2 py-0.5 rounded uppercase font-semibold bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/30">
                  {server.transport}
                </span>
              </div>
              <div class="text-xs font-mono text-theme-text/60 bg-theme-bg/60 p-2 rounded border border-theme-border/60">
                Command: {server.command || '-'}
              </div>
            </div>
          )}
        </For>
        <Show when={props.mcpServers.length === 0}>
          <div class="col-span-full py-12 text-center text-theme-text/40 bg-theme-card/40 border border-theme-border rounded-xl">
            <Server class="w-10 h-10 mx-auto text-theme-text/20 mb-2" />
            <p class="text-xs">No MCP servers registered yet.</p>
            <p class="text-[11px] text-theme-text/30 mt-1">
              MCP servers can be registered in configuration to safely extend the agent's tool-calling capabilities.
            </p>
          </div>
        </Show>
      </div>
    </div>
  );
};
