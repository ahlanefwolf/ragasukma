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
              class="text-white/40 hover:text-white"
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
                <div class={`text-[11px] mb-2 px-2.5 py-1 rounded flex items-center gap-1.5 ${
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
                    <div class="flex items-center justify-between p-2 bg-white/5 border border-white/5 rounded-lg font-mono text-[11px]">
                      <span class="truncate flex items-center gap-1.5 text-white/90">
                        <Folder class="w-3.5 h-3.5 text-amber-400 shrink-0" />
                        {f}
                      </span>
                      <button
                        onClick={() => props.onRemoveFolder(f)}
                        class="text-white/40 hover:text-rose-400 p-0.5 rounded ml-2"
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
        <div class="bg-[#181d26] border border-white/10 rounded-xl max-w-lg w-full p-5 space-y-4 shadow-2xl">
          <div class="flex items-center justify-between border-b border-white/10 pb-3">
            <h3 class="text-sm font-semibold text-white flex items-center gap-2">
              <Sparkles class="w-4 h-4 text-indigo-400" />
              Create New Prompt Skill
            </h3>
            <button
              onClick={props.onClose}
              class="text-white/40 hover:text-white"
            >
              <X class="w-4 h-4" />
            </button>
          </div>

          <div class="space-y-3 text-xs">
            <div class="flex items-center justify-between">
              <label class="block text-white/70 font-medium">SKILL.md Format (Markdown + YAML Frontmatter)</label>
              <span class="text-[10px] text-white/40">Auto-detects name, description & tags</span>
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
                class="w-full px-3 py-2.5 bg-black/30 border border-white/10 rounded-lg text-white font-mono text-[11px] leading-relaxed focus:outline-none focus:border-indigo-500 custom-scrollbar"
              />
            </div>

            {/* Live Preview of parsed metadata */}
            <div class="p-2.5 rounded-lg bg-white/[0.02] border border-white/5 space-y-1 text-[11px]">
              <div class="flex items-center gap-2">
                <span class="text-white/40">Name:</span>
                <span class="text-indigo-300 font-semibold">{props.parseSkillMarkdown(props.skillMarkdown).name || '-'}</span>
                <span class="text-white/20">|</span>
                <span class="text-white/40">Slug:</span>
                <span class="text-white/60 font-mono">{props.parseSkillMarkdown(props.skillMarkdown).slug || '-'}</span>
              </div>
              <div class="flex items-center gap-2">
                <span class="text-white/40">Description:</span>
                <span class="text-white/70 truncate">{props.parseSkillMarkdown(props.skillMarkdown).description || '(No description)'}</span>
              </div>
              <Show when={props.parseSkillMarkdown(props.skillMarkdown).tags.length > 0}>
                <div class="flex items-center gap-1.5 pt-0.5">
                  <span class="text-white/40">Tags:</span>
                  <div class="flex flex-wrap gap-1">
                    <For each={props.parseSkillMarkdown(props.skillMarkdown).tags}>
                      {(t) => (
                        <span class="px-1.5 py-0.2 rounded bg-indigo-500/15 text-indigo-300 text-[10px] border border-indigo-500/30">
                          {t}
                        </span>
                      )}
                    </For>
                  </div>
                </div>
              </Show>
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
              onClick={props.onCreateSkill}
              disabled={!props.skillMarkdown.trim()}
              class="px-4 py-1.5 text-xs bg-indigo-600 hover:bg-indigo-500 disabled:opacity-50 text-white rounded-lg font-medium shadow-sm transition-colors"
            >
              Save Skill
            </button>
          </div>
        </div>
      </div>
    </Show>
  );
};

export const LineRangePickerModal: Component<{
  file: WorkspaceFileItem | null;
  onClose: () => void;
  tagStartLine: string;
  tagEndLine: string;
  loadingFilePreview: boolean;
  highlightedCodeLines: string[];
  currentSelectedTag: string;
  onResetSelection: () => void;
  onLineClick: (line: number, isShiftKey: boolean) => void;
  onGutterMouseDown: (line: number) => void;
  onGutterMouseEnter: (line: number) => void;
  onGutterMouseUp: () => void;
  onCodeTextMouseUp: (e: MouseEvent) => void;
  isLineSelected: (line: number) => boolean;
  onInsertTag: (path: string, s?: string, e?: string) => void;
  onCopyTag?: () => void;
}> = (props) => {
  return (
    <Show when={props.file}>
      <div 
        class="fixed inset-0 z-50 bg-black/80 backdrop-blur-md flex items-center justify-center p-3 sm:p-5"
        onMouseUp={props.onGutterMouseUp}
      >
        <div class="bg-[#141820] border border-white/10 rounded-2xl max-w-5xl w-full h-[85vh] flex flex-col shadow-2xl overflow-hidden animate-in fade-in zoom-in-95 duration-150">
          {/* Modal Header */}
          <div class="px-5 py-3 border-b border-white/10 flex items-center justify-between bg-white/[0.02] shrink-0">
            <div class="flex items-center gap-2.5 min-w-0">
              <FileCode class="w-5 h-5 text-indigo-400 shrink-0" />
              <div class="min-w-0">
                <div class="flex items-center gap-2">
                  <h3 class="text-sm font-semibold text-white font-mono truncate">
                    @{props.file?.relative_path}
                  </h3>
                  <span class="px-1.5 py-0.5 rounded bg-indigo-500/20 text-indigo-300 border border-indigo-500/30 text-[10px] uppercase font-mono font-bold shrink-0">
                    {props.file?.name.split('.').pop() || 'code'}
                  </span>
                </div>
                <p class="text-[11px] text-white/40 mt-0.5">
                  Total: {props.highlightedCodeLines.length || props.file?.line_count || 0} lines • Select text with mouse cursor or click line numbers to choose range
                </p>
              </div>
            </div>

            <div class="flex items-center gap-2">
              <Show when={props.onCopyTag}>
                <button
                  type="button"
                  onClick={props.onCopyTag}
                  class="px-2.5 py-1 text-xs bg-white/5 hover:bg-white/10 text-white/70 hover:text-white rounded-lg flex items-center gap-1.5 transition-colors border border-white/10"
                  title="Copy tag @file:lines"
                >
                  <Copy class="w-3.5 h-3.5" />
                  Copy Tag
                </button>
              </Show>
              <button
                onClick={props.onClose}
                class="p-1.5 text-white/40 hover:text-white rounded-lg hover:bg-white/5 transition-colors"
              >
                <X class="w-4 h-4" />
              </button>
            </div>
          </div>

          {/* Selection Status Banner */}
          <div class="px-5 py-2.5 bg-indigo-500/10 border-b border-indigo-500/20 flex flex-wrap items-center justify-between gap-2 shrink-0">
            <div class="flex items-center gap-2 min-w-0">
              <span class="text-indigo-400 text-xs font-semibold flex items-center gap-1 shrink-0">
                <AtSign class="w-3.5 h-3.5" /> Tag to insert:
              </span>
              <span class="font-mono text-xs text-white bg-black/50 px-2.5 py-1 rounded-lg border border-indigo-500/30 font-bold truncate">
                {props.currentSelectedTag}
              </span>
              <Show when={props.tagStartLine}>
                <span class="text-[11px] text-indigo-300 font-medium">
                  ({props.tagEndLine && props.tagEndLine !== props.tagStartLine 
                    ? `${parseInt(props.tagEndLine, 10) - parseInt(props.tagStartLine, 10) + 1} lines selected` 
                    : '1 line selected'})
                </span>
              </Show>
              <Show when={!props.tagStartLine}>
                <span class="text-[11px] text-white/40 italic">
                  (Entire file will be referenced)
                </span>
              </Show>
            </div>

            <Show when={props.tagStartLine}>
              <button
                type="button"
                onClick={props.onResetSelection}
                class="text-[11px] text-indigo-300 hover:text-white underline underline-offset-2 transition-colors"
              >
                Reset selection (Reference whole file)
              </button>
            </Show>
          </div>

          {/* Code Viewer Body with Line Numbers & Prism Highlighting */}
          <div 
            class="flex-1 overflow-auto bg-[#0d1117] font-mono text-xs custom-scrollbar select-text"
            onMouseUp={props.onCodeTextMouseUp}
          >
            <Show when={props.loadingFilePreview}>
              <div class="flex flex-col items-center justify-center h-full py-16 text-white/40 gap-2">
                <Loader2 class="w-6 h-6 animate-spin text-indigo-400" />
                <span>Loading file content and highlighting syntax...</span>
              </div>
            </Show>

            <Show when={!props.loadingFilePreview && props.highlightedCodeLines.length === 0}>
              <div class="text-center py-16 text-white/30 italic">
                File is empty or contains no readable text content
              </div>
            </Show>

            <Show when={!props.loadingFilePreview && props.highlightedCodeLines.length > 0}>
              <div class="min-w-full inline-block py-2">
                <For each={props.highlightedCodeLines}>
                  {(lineHtml, idx) => {
                    const lineNum = idx() + 1;
                    const selected = () => props.isLineSelected(lineNum);
                    return (
                      <div
                        data-line={lineNum}
                        class={`flex items-stretch hover:bg-white/[0.04] transition-colors leading-6 ${
                          selected()
                            ? 'bg-indigo-500/20 border-l-2 border-indigo-400'
                            : 'border-l-2 border-transparent'
                        }`}
                      >
                        {/* Gutter Line Number */}
                        <div
                          onMouseDown={() => props.onGutterMouseDown(lineNum)}
                          onMouseEnter={() => props.onGutterMouseEnter(lineNum)}
                          onClick={(e) => props.onLineClick(lineNum, e.shiftKey)}
                          class={`w-14 select-none text-right pr-3.5 font-mono text-[11px] cursor-pointer transition-colors shrink-0 ${
                            selected()
                              ? 'text-indigo-300 font-bold bg-indigo-500/30'
                              : 'text-white/30 hover:text-white/70'
                          }`}
                          title={`Line ${lineNum} (Click or drag to select)`}
                        >
                          {lineNum}
                        </div>

                        {/* Code Line Content */}
                        <div
                          class="pl-4 pr-6 flex-1 whitespace-pre overflow-x-visible font-mono"
                          innerHTML={lineHtml || ' '}
                        />
                      </div>
                    );
                  }}
                </For>
              </div>
            </Show>
          </div>

          {/* Modal Footer */}
          <div class="px-5 py-3 border-t border-white/10 flex items-center justify-between bg-white/[0.02] shrink-0">
            <span class="text-xs text-white/40 hidden sm:inline">
              Tip: Highlight code text or click-drag line numbers to select a line range
            </span>

            <div class="flex items-center gap-2 ml-auto">
              <button
                onClick={props.onClose}
                class="px-3.5 py-1.5 text-xs text-white/60 hover:text-white rounded-lg hover:bg-white/5 transition-colors"
              >
                Close
              </button>
              <button
                onClick={() => {
                  if (props.file) {
                    props.onInsertTag(props.file.relative_path, props.tagStartLine, props.tagEndLine);
                  }
                }}
                class="px-4 py-1.5 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium flex items-center gap-1.5 transition-colors shadow-lg shadow-indigo-600/20"
              >
                <Plus class="w-3.5 h-3.5" />
                Insert Tag into Prompt
              </button>
            </div>
          </div>
        </div>
      </div>
    </Show>
  );
};
