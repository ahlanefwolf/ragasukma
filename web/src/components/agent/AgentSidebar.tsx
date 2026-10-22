import { Component, For, Show } from 'solid-js';
import { 
  FolderGit2, 
  MessageSquare, 
  Files, 
  Plus, 
  Trash2, 
  Search, 
  UnfoldVertical, 
  FoldVertical, 
  RefreshCw, 
  ExternalLink, 
  Loader2, 
  Bot 
} from 'lucide-solid';
import { Project, AgentSession, WorkspaceFileItem, parseProjectFolders } from '../../services';
import { FileTreeNode, FileTreeNodeRow } from './FileTreeNodeRow';

export const AgentSidebar: Component<{
  projects: Project[];
  selectedProjectId: string | null;
  onSelectProject: (id: string) => void;
  onNewProject: () => void;
  onDeleteProject: (id: string, name: string) => void;
  currentProject: Project | null | undefined;
  sidebarTab: 'sessions' | 'files';
  setSidebarTab: (tab: 'sessions' | 'files') => void;
  sessions: AgentSession[];
  selectedSessionId: string | null;
  onSelectSession: (sess: AgentSession) => void;
  onQuickCreateSession: () => void;
  onDeleteSession: (id: string, title: string) => void;
  isSessionRunning: (id?: string | null) => boolean;
  fileSearch: string;
  setFileSearch: (val: string) => void;
  workspaceFiles: WorkspaceFileItem[];
  filteredWorkspaceFiles: WorkspaceFileItem[];
  fileTree: FileTreeNode[];
  loadingFiles: boolean;
  onRefreshFiles: () => void;
  onExpandAllDirs: () => void;
  onCollapseAllDirs: () => void;
  onOpenFolderInOs: () => void;
  isDirExpanded: (path: string) => boolean;
  onToggleDir: (path: string) => void;
  onInsertFileTag: (path: string) => void;
  onOpenRangePicker: (file: WorkspaceFileItem) => void;
  onCopyFileTag: (path: string) => void;
}> = (props) => {
  return (
    <div class="lg:col-span-4 flex flex-col space-y-2.5 min-h-0 h-full overflow-hidden">
      {/* Projects Header & Selector */}
      <div class="bg-white/[0.03] border border-white/10 rounded-xl p-3 flex flex-col space-y-2 shrink-0">
        <div class="flex items-center justify-between">
          <span class="text-xs font-semibold text-white uppercase tracking-wider">Workspaces</span>
          <button
            onClick={props.onNewProject}
            class="px-2 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium flex items-center gap-1 transition-colors"
          >
            <Plus class="w-3.5 h-3.5" />
            New Project
          </button>
        </div>

        {/* Projects List */}
        <div class="space-y-1 max-h-36 overflow-y-auto pr-1 custom-scrollbar">
          <Show when={props.projects.length === 0}>
            <div class="text-xs text-white/40 py-4 text-center">
              No workspace projects yet
            </div>
          </Show>
          <For each={props.projects}>
            {(proj) => {
              const folders = parseProjectFolders(proj.root_path);
              const isSelected = () => props.selectedProjectId === proj.id;
              return (
                <div
                  onClick={() => props.onSelectProject(proj.id)}
                  class={`p-2.5 rounded-lg border text-left cursor-pointer transition-all flex items-center justify-between group ${
                    isSelected()
                      ? 'bg-indigo-600/15 border-indigo-500/40 text-white shadow-sm'
                      : 'bg-white/[0.02] border-white/5 text-white/70 hover:bg-white/5 hover:text-white'
                  }`}
                >
                  <div class="min-w-0 flex-1">
                    <div class="text-xs font-semibold truncate flex items-center gap-1.5">
                      <FolderGit2 class={`w-3.5 h-3.5 ${isSelected() ? 'text-indigo-400' : 'text-white/40'}`} />
                      {proj.name}
                    </div>
                    <div class="text-[10px] text-white/40 truncate mt-0.5">
                      {folders.length} attached folder{folders.length > 1 ? 's' : ''}
                    </div>
                  </div>

                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      props.onDeleteProject(proj.id, proj.name);
                    }}
                    class="opacity-0 group-hover:opacity-100 p-1 text-white/40 hover:text-rose-400 hover:bg-rose-500/10 rounded transition-all ml-2"
                    title="Delete Workspace"
                  >
                    <Trash2 class="w-3.5 h-3.5" />
                  </button>
                </div>
              );
            }}
          </For>
        </div>
      </div>

      {/* Sessions List & Workspace Files under Selected Project */}
      <Show when={props.currentProject}>
        <div class="bg-white/[0.03] border border-white/10 rounded-xl p-3 flex-1 flex flex-col space-y-2 min-h-0 overflow-hidden">
          {/* Tab Switcher: Tasks vs Files */}
          <div class="flex items-center gap-1 bg-white/5 p-1 rounded-lg border border-white/10">
            <button
              onClick={() => props.setSidebarTab('sessions')}
              class={`flex-1 py-1 px-2 text-xs font-semibold rounded transition-colors flex items-center justify-center gap-1.5 ${
                props.sidebarTab === 'sessions'
                  ? 'bg-indigo-600 text-white shadow-sm'
                  : 'text-white/60 hover:text-white'
              }`}
            >
              <MessageSquare class="w-3.5 h-3.5" />
              Tasks ({props.sessions.length})
            </button>
            <button
              onClick={() => props.setSidebarTab('files')}
              class={`flex-1 py-1 px-2 text-xs font-semibold rounded transition-colors flex items-center justify-center gap-1.5 ${
                props.sidebarTab === 'files'
                  ? 'bg-indigo-600 text-white shadow-sm'
                  : 'text-white/60 hover:text-white'
              }`}
            >
              <Files class="w-3.5 h-3.5" />
              Files ({props.workspaceFiles.filter((f) => !f.is_dir).length})
            </button>
          </div>

          {/* Task Sessions List */}
          <Show when={props.sidebarTab === 'sessions'}>
            <div class="flex items-center justify-between">
              <div>
                <span class="text-xs font-semibold text-white uppercase tracking-wider">Task Sessions</span>
                <span class="text-[10px] text-white/40 block">in {props.currentProject?.name}</span>
              </div>
              <button
                onClick={props.onQuickCreateSession}
                class="px-2.5 py-1 text-xs bg-indigo-600 hover:bg-indigo-500 text-white rounded-lg font-medium flex items-center gap-1 transition-colors"
              >
                <Plus class="w-3.5 h-3.5" />
                New Task
              </button>
            </div>

            <div class="flex-1 overflow-y-auto space-y-1.5 pr-1 min-h-0 custom-scrollbar">
              <Show when={props.sessions.length === 0}>
                <div class="text-xs text-white/40 py-8 text-center flex flex-col items-center justify-center">
                  <Bot class="w-7 h-7 text-white/20 mb-2" />
                  <span>No task sessions in this project yet</span>
                  <button
                    onClick={props.onQuickCreateSession}
                    class="mt-2 text-xs text-indigo-400 hover:text-indigo-300 font-medium underline"
                  >
                    + Create first task
                  </button>
                </div>
              </Show>
              <For each={props.sessions}>
                {(sess) => {
                  const isSelected = () => props.selectedSessionId === sess.id;
                  const isRunning = () => props.isSessionRunning(sess.id);
                  return (
                    <div
                      onClick={() => props.onSelectSession(sess)}
                      class={`p-2.5 rounded-lg border text-left cursor-pointer transition-all flex items-center justify-between group ${
                        isSelected()
                          ? 'bg-indigo-600/20 border-indigo-500/50 text-white shadow-sm'
                          : 'bg-white/[0.02] border-white/5 text-white/70 hover:bg-white/5 hover:text-white'
                      }`}
                    >
                      <div class="min-w-0 flex-1">
                        <div class="text-xs font-medium truncate flex items-center gap-1.5">
                          <Show
                            when={isRunning()}
                            fallback={<MessageSquare class="w-3 h-3 text-indigo-400 shrink-0" />}
                          >
                            <Loader2 class="w-3 h-3 text-amber-400 animate-spin shrink-0" />
                          </Show>
                          <span class={isRunning() ? 'text-amber-200 font-semibold truncate' : 'truncate'}>
                            {sess.title}
                          </span>
                        </div>
                        <div class="flex items-center gap-2 mt-1">
                          <Show
                            when={isRunning()}
                            fallback={
                              <span class="text-[9px] uppercase px-1.5 py-0.2 rounded bg-white/5 text-white/50">
                                {sess.status}
                              </span>
                            }
                          >
                            <span class="text-[9px] uppercase px-1.5 py-0.2 rounded bg-amber-500/15 text-amber-300 border border-amber-500/30 font-medium flex items-center gap-1 animate-pulse">
                              <span class="w-1.5 h-1.5 rounded-full bg-amber-400 animate-ping"></span>
                              running
                            </span>
                          </Show>
                          <span class="text-[10px] text-white/30 truncate">
                            {sess.model || 'default'}
                          </span>
                        </div>
                      </div>

                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          props.onDeleteSession(sess.id, sess.title);
                        }}
                        class="opacity-0 group-hover:opacity-100 p-1 text-white/40 hover:text-rose-400 hover:bg-rose-500/10 rounded transition-all ml-2"
                        title="Delete Session"
                      >
                        <Trash2 class="w-3.5 h-3.5" />
                      </button>
                    </div>
                  );
                }}
              </For>
            </div>
          </Show>

          {/* Workspace Files List */}
          <Show when={props.sidebarTab === 'files'}>
            <div class="flex-1 flex flex-col space-y-2 min-h-0 overflow-hidden">
              <div class="flex items-center gap-1.5 shrink-0">
                <div class="relative flex-1">
                  <Search class="w-3 h-3 text-white/40 absolute left-2 top-2" />
                  <input
                    type="text"
                    placeholder="Filter files..."
                    value={props.fileSearch}
                    onInput={(e) => props.setFileSearch(e.currentTarget.value)}
                    class="w-full pl-6 pr-2 py-1 text-xs bg-white/5 border border-white/10 rounded-lg text-white placeholder-white/30 focus:outline-none focus:border-indigo-500"
                  />
                </div>
                <button
                  type="button"
                  onClick={props.onExpandAllDirs}
                  class="p-1.5 bg-white/5 hover:bg-white/10 text-white/60 hover:text-white rounded-lg transition-colors border border-white/10 shrink-0"
                  title="Expand All Folders"
                >
                  <UnfoldVertical class="w-3.5 h-3.5" />
                </button>
                <button
                  type="button"
                  onClick={props.onCollapseAllDirs}
                  class="p-1.5 bg-white/5 hover:bg-white/10 text-white/60 hover:text-white rounded-lg transition-colors border border-white/10 shrink-0"
                  title="Collapse All Folders"
                >
                  <FoldVertical class="w-3.5 h-3.5" />
                </button>
                <button
                  onClick={props.onRefreshFiles}
                  class="p-1.5 bg-white/5 hover:bg-white/10 text-white/60 hover:text-white rounded-lg transition-colors border border-white/10 shrink-0"
                  title="Refresh file list"
                >
                  <RefreshCw class={`w-3.5 h-3.5 ${props.loadingFiles ? 'animate-spin' : ''}`} />
                </button>
                <button
                  type="button"
                  onClick={props.onOpenFolderInOs}
                  class="p-1.5 bg-white/5 hover:bg-white/10 text-indigo-400 hover:text-indigo-300 rounded-lg transition-colors border border-white/10 shrink-0"
                  title="Open workspace folder in OS file manager"
                >
                  <ExternalLink class="w-3.5 h-3.5" />
                </button>
              </div>

              <div class="flex items-center justify-between text-[10px] text-white/40 px-1 shrink-0">
                <span>Click item to mention <code class="text-indigo-400">@file</code></span>
                <span>{props.filteredWorkspaceFiles.filter((f) => !f.is_dir).length} files</span>
              </div>

              <div class="flex-1 overflow-y-auto space-y-0.5 pr-1 min-h-0 custom-scrollbar">
                <Show when={props.loadingFiles}>
                  <div class="flex items-center justify-center py-8 text-xs text-white/40 gap-2">
                    <Loader2 class="w-4 h-4 animate-spin text-indigo-400" />
                    <span>Scanning workspace files...</span>
                  </div>
                </Show>

                <Show when={!props.loadingFiles && props.fileTree.length === 0}>
                  <div class="text-xs text-white/40 py-8 text-center flex flex-col items-center justify-center">
                    <Files class="w-7 h-7 text-white/20 mb-2" />
                    <span>No files found in workspace folders</span>
                  </div>
                </Show>

                <For each={props.fileTree}>
                  {(rootNode) => (
                    <FileTreeNodeRow
                      node={rootNode}
                      depth={0}
                      isExpanded={props.isDirExpanded}
                      onToggleDir={props.onToggleDir}
                      onSelectFile={(f) => props.onInsertFileTag(f.relative_path)}
                      onOpenRange={props.onOpenRangePicker}
                      onCopyTag={props.onCopyFileTag}
                      onTagFolder={(path) => props.onInsertFileTag(path)}
                    />
                  )}
                </For>
              </div>
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
};
