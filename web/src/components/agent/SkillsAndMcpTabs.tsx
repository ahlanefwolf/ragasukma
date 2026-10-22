import { Component, For, Show } from 'solid-js';
import { Sparkles, Trash2, Plus, Server } from 'lucide-solid';
import { PromptSkill, McpServerConfig } from '../../services';

export const SkillsSubTab: Component<{
  skills: PromptSkill[];
  onNewSkill: () => void;
  onDeleteSkill: (id: string, name: string) => void;
}> = (props) => {
  return (
    <div class="space-y-4">
      <div class="flex items-center justify-between bg-white/[0.02] border border-white/5 p-4 rounded-xl">
        <div>
          <h3 class="text-sm font-semibold text-white">Prompt Skills Library</h3>
          <p class="text-xs text-white/50 mt-0.5">
            Store custom personas, coding rules, or system prompts that can be attached to any workspace project.
          </p>
        </div>
        <button
          onClick={props.onNewSkill}
          class="px-3 py-1.5 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium flex items-center gap-1.5 transition-colors"
        >
          <Plus class="w-3.5 h-3.5" />
          New Prompt Skill
        </button>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-3">
        <For each={props.skills}>
          {(skill) => (
            <div class="bg-white/[0.03] border border-white/10 rounded-xl p-4 flex flex-col justify-between space-y-3 group hover:border-indigo-500/40 transition-colors">
              <div>
                <div class="flex items-center justify-between mb-1.5">
                  <span class="text-xs font-semibold text-white flex items-center gap-1.5">
                    <Sparkles class="w-3.5 h-3.5 text-indigo-400" />
                    {skill.name}
                  </span>
                  <button
                    onClick={() => props.onDeleteSkill(skill.id, skill.name)}
                    class="opacity-0 group-hover:opacity-100 p-1 text-white/40 hover:text-rose-400 rounded transition-all"
                    title="Delete Skill"
                  >
                    <Trash2 class="w-3.5 h-3.5" />
                  </button>
                </div>
                <span class="text-[10px] font-mono bg-white/5 px-2 py-0.5 rounded text-indigo-300">
                  @{skill.slug}
                </span>
                <p class="text-xs text-white/60 mt-2 line-clamp-2">
                  {skill.description || 'No description'}
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
  );
};

export const McpSubTab: Component<{
  mcpServers: McpServerConfig[];
}> = (props) => {
  return (
    <div class="space-y-4">
      <div class="bg-white/[0.02] border border-white/5 p-4 rounded-xl flex items-center justify-between">
        <div>
          <h3 class="text-sm font-semibold text-white">Model Context Protocol (MCP) Servers</h3>
          <p class="text-xs text-white/50 mt-0.5">
            Open standard connecting AI agents to external tools (Filesystem, Terminal, Postgres, GitHub, Brave Search).
          </p>
        </div>
      </div>

      <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
        <For each={props.mcpServers}>
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
        <Show when={props.mcpServers.length === 0}>
          <div class="col-span-full py-12 text-center text-white/40 bg-white/[0.02] border border-white/5 rounded-xl">
            <Server class="w-10 h-10 mx-auto text-white/20 mb-2" />
            <p class="text-xs">No MCP servers registered yet.</p>
            <p class="text-[11px] text-white/30 mt-1">
              MCP servers can be registered in configuration to safely extend the agent's tool-calling capabilities.
            </p>
          </div>
        </Show>
      </div>
    </div>
  );
};
