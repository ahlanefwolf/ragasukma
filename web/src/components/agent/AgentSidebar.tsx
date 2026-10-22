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
      <div class="bg-theme-card/80 border border-theme-border rounded-xl p-3 flex flex-col space-y-2 shrink-0 shadow-sm">
        <div class="flex items-center justify-between">
          <span class="text-xs font-semibold text-theme-text uppercase tracking-wider">Workspaces</span>
          <button
            onClick={props.onNewProject}
            class="px-2 py-1 text-xs bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card font-bold rounded-lg flex items-center gap-1 transition-colors shadow-sm"
          >
            <Plus class="w-3.5 h-3.5" />
            <span>New</span>
          </button>
        </div>

        {/* Workspaces List / Switcher */}
        <div class="space-y-1 max-h-40 overflow-y-auto custom-scrollbar">
          <For each={props.projects}>
            {(proj) => {
              const folders = parseProjectFolders(proj.root_path);
              const isSelected = () => proj.id === props.selectedProjectId;
              return (
                <div
                  onClick={() => props.onSelectProject(proj.id)}
                  class={`p-2 rounded-lg cursor-pointer transition-all flex items-center justify-between group border ${
                    isSelected()
                      ? 'bg-theme-cyan/15 border-theme-cyan/40 text-theme-text shadow-sm'
                      : 'bg-theme-bg/60 border-theme-border/60 text-theme-text/70 hover:bg-theme-bg hover:text-theme-text'
                  }`}
                >
                  <div class="min-w-0 flex-1">
                    <div class="text-xs font-semibold truncate flex items-center gap-1.5">
                      <FolderGit2 class={`w-3.5 h-3.5 ${isSelected() ? 'text-theme-cyan' : 'text-theme-text/40'}`} />
                      {proj.name}
                    </div>
                    <div class="text-[10px] text-theme-text/40 truncate mt-0.5">
                      {folders.length} attached folder{folders.length > 1 ? 's' : ''}
                    </div>
                  </div>

                  <button
                    onClick={(e) => {
                      e.stopPropagation();
                      props.onDeleteProject(proj.id, proj.name);
                    }}
                    class="opacity-0 group-hover:opacity-100 p-1 text-theme-text/40 hover:text-theme-rose hover:bg-theme-rose/10 rounded transition-all ml-2"
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
        <div class="bg-theme-card/80 border border-theme-border rounded-xl p-3 flex-1 flex flex-col space-y-2 min-h-0 overflow-hidden shadow-sm">
          {/* Tab Switcher: Tasks vs Files */}
          <div class="flex items-center gap-1 bg-theme-bg/60 p-1 rounded-lg border border-theme-border/60">
            <button
              onClick={() => props.setSidebarTab('sessions')}
              class={`flex-1 py-1 px-2 text-xs font-semibold rounded transition-colors flex items-center justify-center gap-1.5 ${
                props.sidebarTab === 'sessions'
                  ? 'bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/40 font-semibold shadow-sm'
                  : 'text-theme-text/60 hover:text-theme-text'
              }`}
            >
              <MessageSquare class="w-3.5 h-3.5" />
              Tasks ({props.sessions.length})
            </button>
            <button
              onClick={() => props.setSidebarTab('files')}
              class={`flex-1 py-1 px-2 text-xs font-semibold rounded transition-colors flex items-center justify-center gap-1.5 ${
                props.sidebarTab === 'files'
                  ? 'bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/40 font-semibold shadow-sm'
                  : 'text-theme-text/60 hover:text-theme-text'
              }`}
            >
              <Files class="w-3.5 h-3.5" />
              Files ({props.workspaceFiles.filter((f) => !f.is_dir).length})
            </button>
          </div>

          {/* Task Sessions List */}
          <Show when={props.sidebarTab === 'sessions'}>
            <div class="flex items-center justify-between shrink-0 px-0.5">
              <span class="text-[10px] text-theme-text/40 uppercase font-semibold">Active Tasks</span>
              <button
                onClick={props.onQuickCreateSession}
                class="text-[10px] text-theme-cyan hover:underline flex items-center gap-0.5"
              >
                <Plus class="w-3 h-3" />
                <span>New Task</span>
              </button>
            </div>

            <div class="flex-1 overflow-y-auto space-y-1.5 pr-0.5 custom-scrollbar min-h-0">
              <Show when={props.sessions.length === 0}>
                <div class="text-xs text-theme-text/40 py-8 text-center flex flex-col items-center justify-center">
                  <Bot class="w-7 h-7 text-theme-text/20 mb-2" />
                  <span>No tasks yet in this workspace</span>
                  <button
                    onClick={props.onQuickCreateSession}
                    class="mt-2 text-xs text-theme-cyan hover:underline"
                  >
                    Create first task
                  </button>
                </div>
              </Show>

              <For each={props.sessions}>
                {(sess) => {
                  const isSelected = () => sess.id === props.selectedSessionId;
                  const isRunning = () => props.isSessionRunning(sess.id);

                  return (
                    <div
                      onClick={() => props.onSelectSession(sess)}
                      class={`p-2 rounded-lg cursor-pointer transition-all flex items-center justify-between group border ${
                        isSelected()
                          ? 'bg-theme-cyan/15 border-theme-cyan/40 text-theme-text shadow-sm'
                          : 'bg-theme-bg/60 border-theme-border/60 text-theme-text/70 hover:bg-theme-bg hover:text-theme-text'
                      }`}
                    >
                      <div class="min-w-0 flex-1">
                        <div class="text-xs font-medium truncate flex items-center gap-1.5">
                          <MessageSquare class={`w-3.5 h-3.5 shrink-0 ${isSelected() ? 'text-theme-cyan' : 'text-theme-text/40'}`} />
                          <span class="truncate">{sess.title}</span>
                        </div>
                        <div class="text-[10px] text-theme-text/40 flex items-center gap-1.5 mt-0.5">
                          <Show when={isRunning()}>
                            <span class="text-[9px] uppercase px-1.5 py-0.2 rounded bg-amber-500/15 text-amber-300 border border-amber-500/30 font-medium flex items-center gap-1 animate-pulse">
                              <span class="w-1.5 h-1.5 rounded-full bg-amber-400 animate-ping"></span>
                              running
                            </span>
                          </Show>
                          <span class="text-[10px] text-theme-text/40 truncate">
                            {sess.model || 'default'}
                          </span>
                        </div>
                      </div>

                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          props.onDeleteSession(sess.id, sess.title);
                        }}
                        class="opacity-0 group-hover:opacity-100 p-1 text-theme-text/40 hover:text-theme-rose hover:bg-theme-rose/10 rounded transition-all ml-2"
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
                  <Search class="w-3 h-3 text-theme-text/40 absolute left-2 top-2" />
                  <input
                    type="text"
                    placeholder="Filter files..."
                    value={props.fileSearch}
                    onInput={(e) => props.setFileSearch(e.currentTarget.value)}
                    class="w-full pl-6 pr-2 py-1 text-xs bg-theme-bg border border-theme-border rounded-lg text-theme-text placeholder-theme-text/30 focus:outline-none focus:border-theme-cyan"
                  />
                </div>
                <button
                  type="button"
                  onClick={props.onExpandAllDirs}
                  class="p-1.5 bg-theme-bg hover:bg-theme-card text-theme-text/60 hover:text-theme-text rounded-lg transition-colors border border-theme-border shrink-0"
                  title="Expand All Folders"
                >
                  <UnfoldVertical class="w-3.5 h-3.5" />
                </button>
                <button
                  type="button"
                  onClick={props.onCollapseAllDirs}
                  class="p-1.5 bg-theme-bg hover:bg-theme-card text-theme-text/60 hover:text-theme-text rounded-lg transition-colors border border-theme-border shrink-0"
                  title="Collapse All Folders"
                >
                  <FoldVertical class="w-3.5 h-3.5" />
                </button>
                <button
                  type="button"
                  onClick={props.onRefreshFiles}
                  disabled={props.loadingFiles}
                  class="p-1.5 bg-theme-bg hover:bg-theme-card text-theme-text/60 hover:text-theme-text rounded-lg transition-colors border border-theme-border shrink-0"
                  title="Refresh File Index"
                >
                  <RefreshCw class={`w-3.5 h-3.5 ${props.loadingFiles ? 'animate-spin text-theme-cyan' : ''}`} />
                </button>
              </div>

              {/* File Tree Rows */}
              <div class="flex-1 overflow-y-auto space-y-0.5 custom-scrollbar bg-theme-bg/50 border border-theme-border/60 rounded-lg p-1 min-h-0">
                <Show when={props.loadingFiles && props.fileTree.length === 0}>
                  <div class="flex items-center justify-center gap-1.5 text-xs text-theme-text/50 py-8">
                    <Loader2 class="w-4 h-4 animate-spin text-theme-cyan" />
                    <span>Scanning workspace files...</span>
                  </div>
                </Show>

                <Show when={!props.loadingFiles && props.fileTree.length === 0}>
                  <div class="text-xs text-theme-text/40 py-8 text-center flex flex-col items-center justify-center">
                    <Files class="w-7 h-7 text-theme-text/20 mb-2" />
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
