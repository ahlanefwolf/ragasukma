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
    <div class="bg-white/[0.03] border border-white/10 rounded-xl px-3.5 py-2 space-y-2 shrink-0">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2 min-w-0">
          <FolderGit2 class="w-4 h-4 text-indigo-400 shrink-0" />
          <span class="text-xs font-semibold text-white truncate">{props.currentProject?.name}</span>
          <span class="text-[10px] text-white/40 shrink-0">
            ({parseProjectFolders(props.currentProject?.root_path).length} folder{parseProjectFolders(props.currentProject?.root_path).length === 1 ? '' : 's'})
          </span>
        </div>
        <button
          type="button"
          onClick={props.onToggleWorkspaceSettings}
          class="text-[10px] text-white/50 hover:text-white flex items-center gap-1 transition-colors px-2 py-0.5 rounded hover:bg-white/5 shrink-0"
        >
          <span>{props.showWorkspaceSettings ? 'Close Settings' : 'Manage Folders & Skills'}</span>
          <ChevronDown class={`w-3 h-3 transition-transform ${props.showWorkspaceSettings ? 'rotate-180' : ''}`} />
        </button>
      </div>

      <Show when={props.showWorkspaceSettings}>
        <div class="pt-2 border-t border-white/5 space-y-3">
          {/* Prompt Skills Activation for Current Project */}
          <div class="space-y-1.5 bg-black/20 p-2.5 rounded-lg border border-white/5">
            <div class="flex items-center justify-between">
              <span class="text-xs font-semibold text-indigo-300 flex items-center gap-1.5">
                <Sparkles class="w-3.5 h-3.5" />
                Active Prompt Skills in Project
              </span>
              <span class="text-[10px] text-white/40">
                {props.activeProjectSkillIds.length} of {props.skills.length} active
              </span>
            </div>
            
            <Show when={props.skills.length === 0}>
              <p class="text-[11px] text-white/40 py-1">
                No skills in library yet. Add one in the sub-tab{' '}
                <button onClick={props.onOpenSkillsTab} class="text-indigo-400 underline">
                  Prompt Skills
                </button>.
              </p>
            </Show>

            <div class="flex flex-wrap gap-1.5 max-h-28 overflow-y-auto custom-scrollbar pt-1">
              <For each={props.skills}>
                {(sk) => {
                  const isEnabled = () => props.activeProjectSkillIds.includes(sk.id);
                  return (
                    <button
                      type="button"
                      onClick={() => props.onToggleProjectSkill(sk.id)}
                      disabled={props.savingProjectSkills}
                      class={`px-2.5 py-1 rounded-lg text-xs font-medium border flex items-center gap-1.5 transition-all ${
                        isEnabled()
                          ? 'bg-indigo-600/30 border-indigo-500/60 text-indigo-200 shadow-sm'
                          : 'bg-white/5 border-white/10 text-white/50 hover:bg-white/10 hover:text-white'
                      }`}
                      title={sk.description || sk.name}
                    >
                      <span class={`w-2 h-2 rounded-full ${isEnabled() ? 'bg-indigo-400' : 'bg-white/20'}`}></span>
                      <span>{sk.name}</span>
                      <Show when={isEnabled()}>
                        <Check class="w-3 h-3 text-indigo-400 ml-0.5" />
                      </Show>
                    </button>
                  );
                }}
              </For>
            </div>
          </div>

          {/* Attached Folders List */}
          <div class="space-y-1">
            <span class="text-[11px] font-medium text-white/60 block">Attached Folders (Workspace Scope):</span>
            <div class="flex flex-wrap gap-1.5 max-h-24 overflow-y-auto custom-scrollbar">
              <For each={parseProjectFolders(props.currentProject?.root_path)}>
                {(folderPath) => (
                  <div class="flex items-center gap-1.5 px-2 py-0.5 bg-white/5 border border-white/10 rounded-md text-[11px] text-white font-mono group">
                    <Folder class="w-3.5 h-3.5 text-amber-400 shrink-0" />
                    <span class="truncate max-w-xs">{folderPath}</span>
                    <button
                      type="button"
                      onClick={() => props.onOpenFolderInOs(folderPath)}
                      class="text-white/40 hover:text-indigo-400 p-0.5 rounded transition-colors"
                      title="Open this folder in OS file manager"
                    >
                      <ExternalLink class="w-3 h-3" />
                    </button>
                    <button
                      onClick={() => props.onRemoveFolder(folderPath)}
                      class="text-white/40 hover:text-rose-400 p-0.5 rounded transition-colors"
                      title="Remove folder from workspace"
                    >
                      <X class="w-3 h-3" />
                    </button>
                  </div>
                )}
              </For>
              <Show when={parseProjectFolders(props.currentProject?.root_path).length === 0}>
                <span class="text-xs text-white/40 italic">No folders attached to this workspace yet</span>
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
              class="flex-1 px-3 py-1.5 text-xs bg-black/40 border border-white/10 rounded-lg text-white placeholder-white/30 focus:outline-none focus:border-indigo-500 font-mono"
            />
            <button
              onClick={props.onAddFolder}
              disabled={props.validatingFolder || !props.folderInput.trim()}
              class="px-2.5 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded-lg font-medium flex items-center gap-1 transition-colors shrink-0"
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
              class="px-2.5 py-1 text-xs bg-white/10 hover:bg-white/15 disabled:opacity-50 text-white rounded-lg font-medium flex items-center gap-1.5 transition-colors shrink-0 border border-white/10"
              title="Open file manager dialog to choose folder directly"
            >
              <Show when={props.browsingFolder} fallback={<FolderOpen class="w-3.5 h-3.5 text-indigo-400" />}>
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
