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
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2 min-w-0">
          <FolderGit2 class="w-4 h-4 text-theme-cyan shrink-0" />
          <span class="text-xs font-semibold text-theme-text truncate">{props.currentProject?.name}</span>
          <span class="text-[10px] text-theme-text/40 shrink-0">
            ({parseProjectFolders(props.currentProject?.root_path).length} folder{parseProjectFolders(props.currentProject?.root_path).length === 1 ? '' : 's'})
          </span>
        </div>
        <button
          type="button"
          onClick={props.onToggleWorkspaceSettings}
          class="text-[10px] text-theme-text/50 hover:text-theme-text flex items-center gap-1 transition-colors px-2 py-0.5 rounded hover:bg-theme-bg/60 shrink-0"
        >
          <span>{props.showWorkspaceSettings ? 'Close Settings' : 'Manage Folders & Skills'}</span>
          <ChevronDown class={`w-3 h-3 transition-transform ${props.showWorkspaceSettings ? 'rotate-180' : ''}`} />
        </button>
      </div>

      {/* Expanded Quick Settings Panel */}
      <Show when={props.showWorkspaceSettings}>
        <div class="pt-2 border-t border-theme-border/60 space-y-3">
          {/* Active Skills Selection */}
          <div>
            <div class="flex items-center justify-between mb-1.5">
              <label class="text-[10px] text-theme-text/60 uppercase tracking-wider font-semibold flex items-center gap-1">
                <Sparkles class="w-3 h-3 text-theme-cyan" />
                Active Prompt Skills for this Workspace
              </label>
              <button
                type="button"
                onClick={props.onOpenSkillsTab}
                class="text-[10px] text-theme-cyan hover:underline"
              >
                Manage Library &rarr;
              </button>
            </div>
            <div class="flex flex-wrap gap-1.5">
              <For each={props.skills}>
                {(skill) => {
                  const isActive = () => props.activeProjectSkillIds.includes(skill.id);
                  return (
                    <button
                      type="button"
                      disabled={props.savingProjectSkills}
                      onClick={() => props.onToggleProjectSkill(skill.id)}
                      class={`px-2 py-1 rounded-md text-[11px] font-mono flex items-center gap-1 transition-all ${
                        isActive()
                          ? 'bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/40 font-semibold shadow-sm'
                          : 'bg-theme-bg/60 text-theme-text/50 border border-theme-border hover:text-theme-text'
                      }`}
                    >
                      <Show when={isActive()}>
                        <Check class="w-2.5 h-2.5 text-theme-cyan" />
                      </Show>
                      <span>{skill.name}</span>
                    </button>
                  );
                }}
              </For>
              <Show when={props.skills.length === 0}>
                <span class="text-xs text-theme-text/40 italic">No skills registered yet</span>
              </Show>
            </div>
          </div>

          {/* Attached Folders List */}
          <div>
            <label class="text-[10px] text-theme-text/60 uppercase tracking-wider font-semibold block mb-1">
              Attached Workspace Folders
            </label>
            <div class="flex flex-wrap gap-1.5">
              <For each={parseProjectFolders(props.currentProject?.root_path)}>
                {(folder) => (
                  <div class="flex items-center gap-1 px-2 py-0.5 rounded-md bg-theme-bg border border-theme-border text-[11px] text-theme-text/80 font-mono">
                    <Folder class="w-3 h-3 text-amber-400 shrink-0" />
                    <span class="truncate max-w-[200px]" title={folder}>{folder}</span>
                    <button
                      type="button"
                      onClick={() => props.onOpenFolderInOs(folder)}
                      class="text-theme-text/40 hover:text-theme-cyan p-0.5 rounded transition-colors ml-0.5"
                      title="Open folder in OS File Manager"
                    >
                      <ExternalLink class="w-3 h-3" />
                    </button>
                    <button
                      type="button"
                      onClick={() => props.onRemoveFolder(folder)}
                      class="text-theme-text/40 hover:text-theme-rose p-0.5 rounded transition-colors"
                      title="Detach folder"
                    >
                      <X class="w-3 h-3" />
                    </button>
                  </div>
                )}
              </For>
              <Show when={parseProjectFolders(props.currentProject?.root_path).length === 0}>
                <span class="text-xs text-theme-text/40 italic">No folders attached to this workspace yet</span>
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
