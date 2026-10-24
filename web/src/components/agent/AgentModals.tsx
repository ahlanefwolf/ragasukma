import { Component, For, Show } from 'solid-js';
import {
  FileCode,
  Sparkles,
  FolderPlus,
  Folder,
  FolderOpen,
  Plus,
  Loader2,
  X,
  Copy,
  AtSign,
  CheckCircle2,
  AlertCircle,
  Check
} from 'lucide-solid';
import { WorkspaceFileItem } from '../../services';

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
        <div class="bg-[#181d26] border border-white/10 rounded-xl max-w-lg w-full p-5 space-y-4 shadow-2xl">
          <div class="flex items-center justify-between border-b border-white/10 pb-3">
            <h3 class="text-sm font-semibold text-white flex items-center gap-2">
              <FolderPlus class="w-4 h-4 text-indigo-400" />
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
                class="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white focus:outline-none focus:border-indigo-500"
              />
            </div>

            <div>
              <label class="block text-white/70 mb-1 font-medium">Description</label>
              <input
                type="text"
                placeholder="Brief workspace description..."
                value={props.projDesc}
                onInput={(e) => props.setProjDesc(e.currentTarget.value)}
                class="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white focus:outline-none focus:border-indigo-500"
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
                  class="flex-1 px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white font-mono placeholder-white/30 focus:outline-none focus:border-indigo-500"
                />
                <button
                  onClick={props.onValidateAndAddFolder}
                  disabled={props.validatingFolder || !props.folderInput.trim()}
                  class="px-3 py-2 bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded-lg font-medium flex items-center gap-1 transition-colors shrink-0"
                >
                  <Show when={props.validatingFolder} fallback={<Plus class="w-3.5 h-3.5" />}>
                    <Loader2 class="w-3.5 h-3.5 animate-spin" />
                  </Show>
                  Add
                </button>
                <button
                  type="button"
                  onClick={props.onBrowseFolder}
                  disabled={props.browsingFolder}
                  class="px-3 py-2 bg-white/10 hover:bg-white/15 disabled:opacity-50 text-white rounded-lg font-medium flex items-center gap-1.5 transition-colors shrink-0 border border-white/10"
                  title="Open OS file manager to choose a folder directly"
                >
                  <Show when={props.browsingFolder} fallback={<FolderOpen class="w-3.5 h-3.5 text-indigo-400" />}>
                    <Loader2 class="w-3.5 h-3.5 animate-spin" />
                  </Show>
                  Browse Folder
                </button>
              </div>

              {/* Validation message badge */}
              <Show when={props.folderValidationMsg}>
                <div class={`text-xs mb-2 px-2.5 py-1 rounded flex items-center gap-1.5 ${
                  props.folderValidationMsg?.valid
                    ? 'bg-emerald-500/10 text-emerald-300 border border-emerald-500/20'
                    : 'bg-rose-500/10 text-rose-300 border border-rose-500/20'
                }`}>
                  <span>{props.folderValidationMsg?.text}</span>
                </div>
              </Show>

              {/* List of staged folders */}
              <div class="space-y-1 max-h-32 overflow-y-auto">
                <For each={props.projFolders}>
                  {(f) => (
                    <div class="flex items-center justify-between p-2 bg-white/5 border border-white/5 rounded-lg font-mono text-xs">
                      <span class="truncate flex items-center gap-1.5 text-white/90">
                        <Folder class="w-3.5 h-3.5 text-amber-400 shrink-0" />
                        {f}
                      </span>
                      <button
                        onClick={() => props.onRemoveFolder(f)}
                        class="text-white/70 hover:text-rose-400 p-0.5 rounded ml-2"
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
                class="w-full px-3 py-2 bg-white/5 border border-white/10 rounded-lg text-white focus:outline-none focus:border-indigo-500"
              />
            </div>
          </div>

          <div class="flex items-center justify-end gap-2 border-t border-white/10 pt-3">
            <button
              onClick={props.onClose}
              class="px-3.5 py-1.5 text-xs text-white/60 hover:text-white rounded-lg"
            >
              Cancel
            </button>
            <button
              onClick={props.onCreateProject}
              disabled={!props.projName.trim()}
              class="px-4 py-1.5 text-xs bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded-lg font-medium"
            >
              Create Workspace
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};

export const CreateSkillModal: Component<{
  isOpen: boolean;
  isEditing?: boolean;
  onClose: () => void;
  skillMarkdown: string;
  setSkillMarkdown: (v: string) => void;
  parseSkillMarkdown: (raw: string) => {
    name: string;
    description: string;
    tags: string[];
    systemPrompt: string;
    slug: string;
  };
  onCreateSkill: () => void;
}> = (props) => {
  return (
    <Show when={props.isOpen}>
      <div class="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm flex items-center justify-center p-4">
        <div class="bg-theme-card border border-theme-border rounded-xl max-w-lg w-full p-5 space-y-4 shadow-2xl">
          <div class="flex items-center justify-between border-b border-theme-border pb-3">
            <h3 class="text-sm font-semibold text-theme-text flex items-center gap-2">
              <Sparkles class="w-4 h-4 text-theme-cyan" />
              <span>{props.isEditing ? 'Edit Prompt Skill' : 'Create New Prompt Skill'}</span>
            </h3>
            <button
              onClick={props.onClose}
              class="text-theme-text/70 hover:text-theme-text"
            >
              <X class="w-4 h-4" />
            </button>
          </div>

          <div class="space-y-3 text-xs">
            <div class="flex items-center justify-between">
              <label class="block text-theme-text/70 font-medium">Skill Prompt Configuration</label>
              <span class="text-2xs text-theme-text/70">Markdown + YAML Frontmatter</span>
            </div>

            <div>
              <textarea
                rows={12}
                placeholder={`---
name: Skill Name
description: Brief skill role description
tags: web, coding
---

Full prompt instructions for the AI model...`}
                value={props.skillMarkdown}
                onInput={(e) => props.setSkillMarkdown(e.currentTarget.value)}
                class="w-full px-3 py-2.5 bg-theme-bg border border-theme-border rounded-lg text-theme-text font-mono text-xs leading-relaxed focus:outline-none focus:border-theme-cyan custom-scrollbar"
              />
            </div>

            {/* Live Preview of parsed metadata */}
            <div class="p-2.5 rounded-lg bg-theme-bg/60 border border-theme-border/60 space-y-1 text-xs">
              <div class="flex items-center gap-2">
                <span class="text-theme-text/70">Name:</span>
                <span class="text-theme-cyan font-semibold">{props.parseSkillMarkdown(props.skillMarkdown).name || '-'}</span>
                <span class="text-theme-border">•</span>
                <span class="text-theme-text/70">Slug:</span>
                <span class="text-theme-text/60 font-mono">{props.parseSkillMarkdown(props.skillMarkdown).slug || '-'}</span>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-theme-text/70">Description:</span>
                <span class="text-theme-text/70 truncate">{props.parseSkillMarkdown(props.skillMarkdown).description || '(No description)'}</span>
              </div>
              <Show when={props.parseSkillMarkdown(props.skillMarkdown).tags.length > 0}>
                <div class="flex items-center gap-1.5 pt-0.5">
                  <span class="text-theme-text/70">Tags:</span>
                  <div class="flex flex-wrap gap-1">
                    <For each={props.parseSkillMarkdown(props.skillMarkdown).tags}>
                      {(t) => (
                        <span class="px-1.5 py-0.2 rounded bg-theme-cyan/15 text-theme-cyan text-2xs border border-theme-cyan/30">
                          {t}
                        </span>
                      )}
                    </For>
                  </div>
                </div>
              </Show>
            </div>
          </div>

          <div class="flex items-center justify-end gap-2 border-t border-theme-border pt-3">
            <button
              onClick={props.onClose}
              class="px-3.5 py-1.5 text-xs text-theme-text/60 hover:text-theme-text rounded-lg"
            >
              Cancel
            </button>
            <button
              onClick={props.onCreateSkill}
              disabled={!props.skillMarkdown.trim()}
              class="px-4 py-1.5 text-xs bg-theme-cyan hover:bg-theme-cyan/90 disabled:opacity-50 text-theme-card font-bold rounded-lg shadow-sm transition-colors"
            >
              {props.isEditing ? 'Save Changes' : 'Create Skill'}
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};

