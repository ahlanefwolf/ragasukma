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
  Loader2,
  Server,
  Zap,
  Plus
} from 'lucide-solid';
import { Project, PromptSkill, McpServerConfig, parseProjectFolders } from '../../services';

export const AgentWorkspaceHeader: Component<{
  currentProject: Project | null | undefined;
  showWorkspaceSettings: boolean;
  onToggleWorkspaceSettings: () => void;
  skills: PromptSkill[];
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
    <div class="bg-theme-card/80 border border-theme-border px-3.5 py-2 space-y-2 shrink-0 shadow-sm">
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-2.5">
        <div class="flex items-center gap-2.5 min-w-0">
          <div class="p-1.5 bg-theme-cyan/10 text-theme-cyan border border-theme-cyan/20 shrink-0">
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

        {/* Workspace Settings Button */}
        <div class="flex items-center sm:justify-end shrink-0">
          <button
            type="button"
            onClick={props.onToggleWorkspaceSettings}
            class="text-2xs text-theme-text/60 hover:text-theme-text flex items-center gap-1 transition-colors px-2.5 py-1 border border-theme-border hover:border-theme-cyan/40 hover:bg-theme-bg/60 shrink-0"
          >
            <span>{props.showWorkspaceSettings ? 'Close Settings' : 'Workspace Settings'}</span>
            <ChevronDown class={`w-3 h-3 transition-transform ${props.showWorkspaceSettings ? 'rotate-180 text-theme-cyan' : ''}`} />
          </button>
        </div>
      </div>

      {/* Expanded Quick Settings Panel */}
      <Show when={props.showWorkspaceSettings}>
        <div class="pt-3 border-t border-theme-border/60 space-y-4">
          {/* Auto-detected Skills Section */}
          <div class="bg-theme-bg/40 border border-theme-border/70 p-3 space-y-2.5">
            <div>
              <label class="text-xs text-theme-text font-semibold flex items-center gap-1.5">
                <Sparkles class="w-3.5 h-3.5 text-theme-cyan" />
                Auto-detected Agent Skills ({props.skills.length})
              </label>
              <p class="text-xs text-theme-text/50 mt-0.5">
                Found in attached folders. Loaded on-demand into prompt context when matching tasks are initiated.
              </p>
            </div>

            <Show
              when={props.skills.length > 0}
              fallback={
                <div class="text-xs text-theme-text/70 py-2 italic font-mono bg-theme-bg/30 p-2 border border-dashed border-theme-border/60">
                  No skill files detected in attached folders. Add SKILL.md or .skill/ directories to customize prompt behaviors.
                </div>
              }
            >
              <div class="flex flex-wrap gap-1.5">
                <For each={props.skills}>
                  {(s) => (
                    <span class="inline-flex items-center gap-1 text-2xs px-2 py-1 bg-theme-bg border border-theme-border font-mono text-theme-text/80 hover:border-theme-cyan/50 hover:text-theme-cyan transition-colors">
                      <Sparkles class="w-2.5 h-2.5 text-theme-cyan shrink-0" />
                      {s.name}
                    </span>
                  )}
                </For>
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
                  <div class="flex items-center gap-1 px-2 py-0.5 bg-theme-bg border border-theme-border text-xs text-theme-text/80 font-mono">
                    <Folder class="w-3 h-3 text-gi-status-warning shrink-0" />
                    <span class="truncate max-w-[200px]" title={folder}>{folder}</span>
                    <button
                      type="button"
                      onClick={() => props.onOpenFolderInOs(folder)}
                      class="text-theme-text/70 hover:text-theme-cyan p-0.5 transition-colors ml-0.5"
                      title="Open folder in OS File Manager"
                    >
                      <ExternalLink class="w-3 h-3" />
                    </button>
                    <button
                      type="button"
                      onClick={() => props.onRemoveFolder(folder)}
                      class="text-theme-text/70 hover:text-theme-rose p-0.5 transition-colors"
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
              class="flex-1 px-3 py-1.5 text-xs bg-theme-bg border border-theme-border text-theme-text placeholder-theme-text/30 focus:outline-none focus:border-theme-cyan font-mono"
            />
            <button
              onClick={props.onAddFolder}
              disabled={props.validatingFolder || !props.folderInput.trim()}
              class="px-2.5 py-1 text-xs bg-theme-cyan hover:bg-theme-cyan/90 disabled:opacity-50 text-theme-card font-bold flex items-center gap-1 transition-colors shrink-0 shadow-sm"
            >
              <Plus class="w-3.5 h-3.5" />
              <span>Add</span>
            </button>
            <button
              type="button"
              onClick={props.onBrowseFolder}
              disabled={props.browsingFolder}
              class="px-2.5 py-1 text-xs bg-theme-card hover:bg-theme-bg disabled:opacity-50 text-theme-text font-medium flex items-center gap-1.5 transition-colors shrink-0 border border-theme-border"
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