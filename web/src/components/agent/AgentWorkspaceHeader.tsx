import { Component, For, Show } from 'solid-js';
import { 
  FolderGit2, 
  Sparkles, 
  Folder, 
  FolderPlus, 
  FolderOpen, 
  Check, 
  X, 
  ChevronDown, 
  ExternalLink, 
  Loader2 
} from 'lucide-solid';
import { Project, PromptSkill, parseProjectFolders } from '../../services';

export const AgentWorkspaceHeader: Component<{
  currentProject: Project | null | undefined;
  showWorkspaceSettings: boolean;
  onToggleWorkspaceSettings: () => void;
  skills: PromptSkill[];
  activeProjectSkillIds: string[];
  savingProjectSkills: boolean;
  onToggleProjectSkill: (skillId: string) => void;
  onOpenSkillsTab: () => void;
  folderInput: string;
  setFolderInput: (val: string) => void;
  validatingFolder: boolean;
  browsingFolder: boolean;
  onAddFolder: () => void;
  onBrowseFolder: () => void;
  onRemoveFolder: (path: string) => void;
  onOpenFolderInOs: (path: string) => void;
}> = (props) => {
  return (
    <div class="bg-theme-card/80 border border-theme-border rounded-xl px-3.5 py-2 space-y-2 shrink-0 shadow-sm">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2.5">
        <div class="flex items-center gap-2.5 min-w-0">
          <div class="p-1.5 rounded-lg bg-theme-cyan/10 text-theme-cyan border border-theme-cyan/20 shrink-0">
            <FolderGit2 class="w-4 h-4" />
          </div>
          <div class="min-w-0">
            <div class="flex items-center gap-2">
              <span class="text-xs font-semibold text-theme-text truncate max-w-[200px]" title={props.currentProject?.name}>
                {props.currentProject?.name}
              </span>
              <span class="text-2xs text-theme-text/70 font-mono shrink-0">
                ({parseProjectFolders(props.currentProject?.root_path).length} folder{parseProjectFolders(props.currentProject?.root_path).length === 1 ? '' : 's'})
              </span>
            </div>
          </div>
        </div>

        {/* Quick Active Skills Pill Strip & Manage Button */}
        <div class="flex items-center gap-2 flex-wrap sm:justify-end">
          {/* Active Skills Pills (Always accessible at a glance for the user) */}
          <div class="flex items-center gap-1.5 flex-wrap">
            <For each={props.skills.filter(s => props.activeProjectSkillIds.includes(s.id))}>
              {(skill) => (
                <span
                  class="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-xs font-medium bg-theme-cyan/15 text-theme-cyan border border-theme-cyan/30 shadow-xs transition-all hover:bg-theme-cyan/25"
                  title={`${skill.name}: ${skill.description || 'Active prompt skill'}`}
                >
                  <Sparkles class="w-2.5 h-2.5 shrink-0 text-theme-cyan animate-pulse" />
                  <span class="truncate max-w-[120px]">{skill.name}</span>
                  <button
                    type="button"
                    disabled={props.savingProjectSkills}
                    onClick={(e) => {
                      e.stopPropagation();
                      props.onToggleProjectSkill(skill.id);
                    }}
                    class="hover:text-rose-400 p-0.5 rounded-full hover:bg-theme-bg/50 transition-colors"
                    title="Disable this skill from workspace"
                  >
                    <X class="w-2.5 h-2.5" />
                  </button>
                </span>
              )}
            </For>

            {/* Quick "+ Skill" pill button */}
            <button
              type="button"
              onClick={props.onToggleWorkspaceSettings}
              class={`inline-flex items-center gap-1 px-2.5 py-0.5 rounded-full text-xs font-medium transition-all ${
                props.activeProjectSkillIds.length === 0
                  ? 'bg-amber-500/15 text-amber-300 border border-amber-500/30 hover:bg-amber-500/25 shadow-xs'
                  : 'bg-theme-bg/80 text-theme-text/70 border border-theme-border hover:border-theme-cyan/40 hover:text-theme-cyan'
              }`}
              title={props.activeProjectSkillIds.length === 0 ? 'No active skills. Click to add skills to this workspace.' : 'Manage prompt skills'}
            >
              <Sparkles class="w-2.5 h-2.5 shrink-0" />
              <span>
                {props.activeProjectSkillIds.length === 0 ? '+ Add AI Skill' : '+ Skill'}
              </span>
            </button>
          </div>

          <button
            type="button"
            onClick={props.onToggleWorkspaceSettings}
            class="text-2xs text-theme-text/60 hover:text-theme-text flex items-center gap-1 transition-colors px-2.5 py-1 rounded-lg border border-theme-border hover:border-theme-cyan/40 hover:bg-theme-bg/60 shrink-0"
          >
            <span>{props.showWorkspaceSettings ? 'Close Settings' : 'Workspace Settings'}</span>
            <ChevronDown class={`w-3 h-3 transition-transform ${props.showWorkspaceSettings ? 'rotate-180 text-theme-cyan' : ''}`} />
          </button>
        </div>
      </div>

      {/* Expanded Quick Settings Panel */}
      <Show when={props.showWorkspaceSettings}>
        <div class="pt-3 border-t border-theme-border/60 space-y-4">
          {/* Active Skills Selection */}
          <div class="bg-theme-bg/40 border border-theme-border/70 rounded-xl p-3 space-y-2.5">
            <div class="flex items-center justify-between">
              <div>
                <label class="text-xs text-theme-text font-semibold flex items-center gap-1.5">
                  <Sparkles class="w-3.5 h-3.5 text-theme-cyan" />
                  Workspace Prompt Skills
                </label>
                <p class="text-xs text-theme-text/50 mt-0.5">
                  Select specialized skills used by the AI agent to complete tasks in this workspace.
                </p>
              </div>
              <button
                type="button"
                onClick={props.onOpenSkillsTab}
                class="text-xs text-theme-cyan hover:underline shrink-0 font-medium"
              >
                Open Skills Library &rarr;
              </button>
            </div>

            {/* Interactive Layperson-Friendly Skill Cards */}
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-2">
              <For each={props.skills}>
                {(skill) => {
                  const isActive = () => props.activeProjectSkillIds.includes(skill.id);
                  return (
                    <div
                      onClick={() => !props.savingProjectSkills && props.onToggleProjectSkill(skill.id)}
                      class={`cursor-pointer rounded-lg p-2.5 border transition-all text-left flex flex-col justify-between gap-2 select-none ${
                        isActive()
                          ? 'bg-theme-cyan/10 border-theme-cyan/50 shadow-xs'
                          : 'bg-theme-card/60 border-theme-border hover:border-theme-border-hover hover:bg-theme-card'
                      }`}
                    >
                      <div class="space-y-1">
                        <div class="flex items-center justify-between gap-1.5">
                          <span class="text-xs font-semibold text-theme-text truncate flex items-center gap-1.5">
                            <span class={`w-2 h-2 rounded-full shrink-0 ${isActive() ? 'bg-theme-cyan shadow-xs shadow-theme-cyan/50' : 'bg-theme-text/20'}`} />
                            {skill.name}
                          </span>
                          <span
                            class={`text-2xs px-1.5 py-0.5 rounded font-mono font-medium ${
                              isActive()
                                ? 'bg-theme-cyan/20 text-theme-cyan'
                                : 'bg-theme-bg text-theme-text/70'
                            }`}
                          >
                            {isActive() ? 'Active' : 'Inactive'}
                          </span>
                        </div>
                        <p class="text-xs text-theme-text/60 line-clamp-2 leading-tight">
                          {skill.description || 'Specialized AI skill instructions to accelerate workflow and increase accuracy.'}
                        </p>
                      </div>

                      <div class="flex items-center justify-between pt-1 border-t border-theme-border/40 text-2xs text-theme-text/70">
                        <span class="font-mono truncate max-w-[140px]">
                          {skill.slug ? `@${skill.slug}` : 'skill'}
                        </span>
                        <span class="text-theme-cyan/70 font-medium">
                          {isActive() ? 'Click to disable' : 'Click to enable'}
                        </span>
                      </div>
                    </div>
                  );
                }}
              </For>
            </div>

            <Show when={props.skills.length === 0}>
              <div class="p-3 text-center rounded-lg border border-dashed border-theme-border bg-theme-bg/30">
                <p class="text-xs text-theme-text/60 mb-2">No prompt skills registered in library yet.</p>
                <button
                  type="button"
                  onClick={props.onOpenSkillsTab}
                  class="text-xs px-3 py-1 bg-theme-cyan text-theme-bg font-medium rounded-lg hover:bg-theme-cyan/90 transition-colors inline-flex items-center gap-1.5"
                >
                  <Sparkles class="w-3.5 h-3.5" />
                  Add First Skill
                </button>
              </div>
            </Show>
          </div>

          {/* Attached Folders List */}
          <div>
            <label class="text-2xs text-theme-text/60 uppercase tracking-wider font-semibold block mb-1">
              Attached Workspace Folders
            </label>
            <div class="flex flex-wrap gap-1.5">
              <For each={parseProjectFolders(props.currentProject?.root_path)}>
                {(folder) => (
                  <div class="flex items-center gap-1 px-2 py-0.5 rounded-md bg-theme-bg border border-theme-border text-xs text-theme-text/80 font-mono">
                    <Folder class="w-3 h-3 text-amber-400 shrink-0" />
                    <span class="truncate max-w-[200px]" title={folder}>{folder}</span>
                    <button
                      type="button"
                      onClick={() => props.onOpenFolderInOs(folder)}
                      class="text-theme-text/70 hover:text-theme-cyan p-0.5 rounded transition-colors ml-0.5"
                      title="Open folder in OS File Manager"
                    >
                      <ExternalLink class="w-3 h-3" />
                    </button>
                    <button
                      type="button"
                      onClick={() => props.onRemoveFolder(folder)}
                      class="text-theme-text/70 hover:text-theme-rose p-0.5 rounded transition-colors"
                      title="Detach folder"
                    >
                      <X class="w-3 h-3" />
                    </button>
                  </div>
                )}
              </For>
              <Show when={parseProjectFolders(props.currentProject?.root_path).length === 0}>
                <span class="text-xs text-theme-text/70 italic">No folders attached to this workspace yet</span>
              </Show>
            </div>
          </div>

          <div class="flex items-center gap-2 pt-1">
            <input
              type="text"
              placeholder="Add local folder path..."
              value={props.folderInput}
              onInput={(e) => props.setFolderInput(e.currentTarget.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter') props.onAddFolder();
              }}
              class="flex-1 px-3 py-1.5 text-xs bg-theme-bg border border-theme-border rounded-lg text-theme-text placeholder-theme-text/30 focus:outline-none focus:border-theme-cyan font-mono"
            />
            <button
              onClick={props.onAddFolder}
              disabled={props.validatingFolder || !props.folderInput.trim()}
              class="px-2.5 py-1 text-xs bg-theme-cyan hover:bg-theme-cyan/90 disabled:opacity-50 text-theme-card font-bold rounded-lg flex items-center gap-1 transition-colors shrink-0 shadow-sm"
            >
              <Show when={props.validatingFolder} fallback={<FolderPlus class="w-3.5 h-3.5" />}>
                <Loader2 class="w-3.5 h-3.5 animate-spin" />
              </Show>
              Attach
            </button>
            <button
              type="button"
              onClick={props.onBrowseFolder}
              disabled={props.browsingFolder}
              class="px-2.5 py-1 text-xs bg-theme-card hover:bg-theme-bg disabled:opacity-50 text-theme-text rounded-lg font-medium flex items-center gap-1.5 transition-colors shrink-0 border border-theme-border"
              title="Open file manager dialog to choose folder directly"
            >
              <Show when={props.browsingFolder} fallback={<FolderOpen class="w-3.5 h-3.5 text-theme-cyan" />}>
                <Loader2 class="w-3.5 h-3.5 animate-spin" />
              </Show>
              Browse Folder
            </button>
          </div>
        </div>
      </Show>
    </div>
  );
};
