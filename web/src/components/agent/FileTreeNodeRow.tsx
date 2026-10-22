import { Component, For, Show } from 'solid-js';
import {
  Folder,
  ChevronRight,
  FileCode,
  FileText,
  Copy,
  Hash,
  AtSign
} from 'lucide-solid';
import { WorkspaceFileItem } from '../../services';

export interface FileTreeNode {
  name: string;
  path: string;
  is_dir: boolean;
  item?: WorkspaceFileItem;
  children: FileTreeNode[];
}

export const FileTreeNodeRow: Component<{
  node: FileTreeNode;
  depth: number;
  isExpanded: (path: string) => boolean;
  onToggleDir: (path: string) => void;
  onSelectFile: (file: WorkspaceFileItem) => void;
  onOpenRange: (file: WorkspaceFileItem) => void;
  onCopyTag: (path: string) => void;
  onTagFolder: (path: string) => void;
}> = (props) => {
  return (
    <div>
      {/* Directory Node */}
      <Show
        when={props.node.is_dir}
        fallback={
          /* File Item Row */
          <div
            onClick={() => props.node.item && props.onSelectFile(props.node.item)}
            style={{ 'padding-left': `${props.depth * 14 + 6}px` }}
            class="flex items-center justify-between p-1.5 rounded-lg hover:bg-white/5 cursor-pointer text-xs group transition-colors border border-transparent hover:border-white/5"
          >
            <div class="flex items-center gap-1.5 min-w-0 flex-1">
              <Show
                when={props.node.name.endsWith('.rs') || props.node.name.endsWith('.ts') || props.node.name.endsWith('.tsx') || props.node.name.endsWith('.js')}
                fallback={<FileText class="w-3.5 h-3.5 text-white/40 shrink-0" />}
              >
                <FileCode class="w-3.5 h-3.5 text-indigo-400 shrink-0" />
              </Show>
              <span class="truncate font-mono text-[11px] text-white/80 group-hover:text-white">
                {props.node.name}
              </span>
            </div>

            <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
              <Show when={props.node.item?.line_count !== undefined && props.node.item?.line_count !== null}>
                <span class="text-[9px] text-white/30 font-mono mr-1">
                  {props.node.item?.line_count}L
                </span>
              </Show>
              {/* Range Picker Button */}
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  if (props.node.item) props.onOpenRange(props.node.item);
                }}
                class="p-1 text-white/40 hover:text-indigo-300 hover:bg-indigo-500/10 rounded transition-colors"
                title="Select line range (@file:lines) with visual code picker"
              >
                <Hash class="w-3 h-3" />
              </button>
              {/* Copy Tag Button */}
              <button
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  props.onCopyTag(props.node.path);
                }}
                class="p-1 text-white/40 hover:text-white hover:bg-white/10 rounded transition-colors"
                title="Copy @file tag to clipboard"
              >
                <Copy class="w-3 h-3" />
              </button>
            </div>
          </div>
        }
      >
        {/* Directory Row */}
        <div
          onClick={() => props.onToggleDir(props.node.path)}
          style={{ 'padding-left': `${props.depth * 14 + 6}px` }}
          class="flex items-center justify-between p-1.5 rounded-lg hover:bg-white/5 cursor-pointer text-xs group transition-colors select-none text-white/70 hover:text-white"
        >
          <div class="flex items-center gap-1.5 min-w-0 flex-1">
            <ChevronRight
              class={`w-3.5 h-3.5 text-white/40 transition-transform ${
                props.isExpanded(props.node.path) ? 'rotate-90 text-white/80' : ''
              }`}
            />
            <Folder
              class={`w-3.5 h-3.5 ${
                props.isExpanded(props.node.path) ? 'text-amber-300' : 'text-amber-400/80'
              } shrink-0`}
            />
            <span class="truncate font-semibold text-[11px]">{props.node.name}</span>
            <span class="text-[10px] text-white/30 font-mono">({props.node.children.length})</span>
          </div>

          <div class="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
            <button
              type="button"
              onClick={(e) => {
                e.stopPropagation();
                props.onTagFolder(props.node.path);
              }}
              class="p-1 text-white/40 hover:text-amber-300 hover:bg-amber-500/10 rounded transition-colors"
              title="Mention folder in chat prompt"
            >
              <AtSign class="w-3 h-3" />
            </button>
          </div>
        </div>

        <Show when={props.isExpanded(props.node.path)}>
          <div class="space-y-0.5">
            <For each={props.node.children}>
              {(child) => (
                <FileTreeNodeRow
                  node={child}
                  depth={props.depth + 1}
                  isExpanded={props.isExpanded}
                  onToggleDir={props.onToggleDir}
                  onSelectFile={props.onSelectFile}
                  onOpenRange={props.onOpenRange}
                  onCopyTag={props.onCopyTag}
                  onTagFolder={props.onTagFolder}
                />
              )}
            </For>
          </div>
        </Show>
      </Show>
    </div>
  );
};
