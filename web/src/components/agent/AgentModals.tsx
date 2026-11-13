import { Component, For, Show } from 'solid-js';
import {
  FolderPlus,
  Folder,
  FolderOpen,
  Plus,
  Loader2,
  X,
} from 'lucide-solid';

export const CreateProjectModal: Component<{
  isOpen: boolean;
  onClose: () => void;
  projName: string;
  setProjName: (v: string) => void;
  projDesc: string;
  setProjDesc: (v: string) => void;
  projInstructions: string;
  setProjInstructions: (v: string) => void;
  projFolders: string[];
  folderInput: string;
  setFolderInput: (v: string) => void;
  validatingFolder: boolean;
  browsingFolder: boolean;
  folderValidationMsg: { valid: boolean; text: string } | null;
  onBrowseFolder: () => void;
  onValidateAndAddFolder: () => void;
  onRemoveFolder: (f: string) => void;
  onCreateProject: () => void;
}> = (props) => {
  return (
    <Show when={props.isOpen}>
      <div class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
        <div class="bg-theme-surface border border-theme-border max-w-lg w-full p-5 space-y-4 shadow-2xl">
          <div class="flex items-center justify-between border-b border-theme-border pb-3">
            <h3 class="text-sm font-semibold text-white flex items-center gap-2">
              <FolderPlus class="w-4 h-4 text-gi-primary" />
              Create New Workspace Project
            </h3>
            <button
              onClick={props.onClose}
              class="text-white/70 hover:text-white"
            >
              <X class="w-4 h-4" />
            </button>
          </div>

          <div class="space-y-3 text-xs">
            <div>
              <label class="block text-white/70 mb-1 font-medium">Project Name *</label>
              <input
                type="text"
                placeholder="e.g. Backend API, Frontend App"
                value={props.projName}
                onInput={(e) => props.setProjName(e.currentTarget.value)}
                class="w-full px-3 py-2 bg-white/5 border border-white/10 text-white focus:outline-none focus:border-gi-primary"
              />
            </div>

            <div>
              <label class="block text-white/70 mb-1 font-medium">Description</label>
              <input
                type="text"
                placeholder="Brief workspace description..."
                value={props.projDesc}
                onInput={(e) => props.setProjDesc(e.currentTarget.value)}
                class="w-full px-3 py-2 bg-white/5 border border-white/10 text-white focus:outline-none focus:border-gi-primary"
              />
            </div>

            <div>
              <label class="block text-white/70 mb-1 font-medium">Workspace Folders (Can attach multiple)</label>
              <div class="flex items-center gap-2 mb-2">
                <input
                  type="text"
                  placeholder="Absolute or relative folder path..."
                  value={props.folderInput}
                  onInput={(e) => props.setFolderInput(e.currentTarget.value)}
                  onKeyDown={(e) => {
                    if (e.key === 'Enter') props.onValidateAndAddFolder();
                  }}
                  class="flex-1 px-3 py-1.5 bg-white/5 border border-white/10 text-white font-mono text-xs focus:outline-none focus:border-gi-primary"
                />
                <button
                  onClick={props.onBrowseFolder}
                  disabled={props.browsingFolder}
                  class="px-3 py-1.5 bg-white/10 hover:bg-white/20 disabled:opacity-50 text-white font-semibold flex items-center gap-1"
                  title="Browse folder using native dialog"
                >
                  <Show when={props.browsingFolder} fallback={<FolderOpen class="w-3.5 h-3.5" />}>
                    <Loader2 class="w-3.5 h-3.5 animate-spin" />
                  </Show>
                  <span>Browse</span>
                </button>
                <button
                  onClick={props.onValidateAndAddFolder}
                  disabled={props.validatingFolder || !props.folderInput.trim()}
                  class="px-3 py-1.5 bg-white/10 hover:bg-white/20 disabled:opacity-50 text-white font-semibold flex items-center gap-1"
                >
                  <Show when={props.validatingFolder} fallback={<Plus class="w-3.5 h-3.5" />}>
                    <Loader2 class="w-3.5 h-3.5 animate-spin" />
                  </Show>
                  <span>Add</span>
                </button>
              </div>

              <Show when={props.folderValidationMsg}>
                <div class={`text-xs mb-2 px-2.5 py-1 flex items-center gap-1.5 ${
                  props.folderValidationMsg?.valid
                    ? 'bg-gi-status-success/10 text-gi-status-success border border-gi-status-success/20'
                    : 'bg-gi-status-danger/10 text-gi-status-danger border border-gi-status-danger/20'
                }`}>
                  <span>{props.folderValidationMsg?.text}</span>
                </div>
              </Show>

              {/* List of staged folders */}
              <div class="space-y-1 max-h-32 overflow-y-auto">
                <For each={props.projFolders}>
                  {(f) => (
                    <div class="flex items-center justify-between p-2 bg-white/5 border border-white/5 font-mono text-xs">
                      <span class="truncate flex items-center gap-1.5 text-white/90">
                        <Folder class="w-3.5 h-3.5 text-gi-status-warning shrink-0" />
                        {f}
                      </span>
                      <button
                        onClick={() => props.onRemoveFolder(f)}
                        class="text-white/70 hover:text-gi-status-danger p-0.5 ml-2"
                      >
                        <X class="w-3.5 h-3.5" />
                      </button>
                    </div>
                  )}
                </For>
              </div>
            </div>

            <div>
              <label class="block text-white/70 mb-1 font-medium">Custom System Instructions (Optional)</label>
              <textarea
                rows={2}
                placeholder="Specific instructions that will always be included in the system prompt for this project..."
                value={props.projInstructions}
                onInput={(e) => props.setProjInstructions(e.currentTarget.value)}
                class="w-full px-3 py-2 bg-white/5 border border-white/10 text-white focus:outline-none focus:border-gi-primary"
              />
            </div>
          </div>

          <div class="flex items-center justify-end gap-2 border-t border-theme-border pt-3">
            <button
              onClick={props.onClose}
              class="px-3.5 py-1.5 text-xs text-white/60 hover:text-white"
            >
              Cancel
            </button>
            <button
              onClick={props.onCreateProject}
              disabled={!props.projName.trim()}
              class="px-4 py-1.5 text-xs bg-gi-primary hover:bg-gi-primary-hover disabled:opacity-50 text-gi-primary-foreground font-semibold"
            >
              Create Workspace
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};
