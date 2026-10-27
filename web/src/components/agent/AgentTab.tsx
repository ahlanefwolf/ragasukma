import { Component, createSignal, onMount, Show, createMemo, createEffect, onCleanup } from 'solid-js';
import { 
  FolderGit2, 
  Layers, 
  Sparkles, 
  Server
} from 'lucide-solid';
import { 
  agentService, 
  WorkspaceFileItem, 
  parseProjectFolders 
} from '../../services';

import { McpSubTab } from './SkillsAndMcpTabs';
import { CreateProjectModal, McpServerModal } from './AgentModals';
import { AgentCodePanel } from './AgentCodePanel';
import { AgentSidebar } from './AgentSidebar';
import { AgentWorkspaceHeader } from './AgentWorkspaceHeader';
import { AgentChatPane } from './AgentChatPane';

import { useWorkspaceState } from './useWorkspaceState';
import { useChatSession } from './useChatSession';
import { useWorkspaceFiles } from './useWorkspaceFiles';

interface AgentTabProps {
  showToast: (msg: string) => void;
}

export const AgentTab: Component<AgentTabProps> = (props) => {
  let messagesContainerRef: HTMLDivElement | undefined;
  let chatInputRef: HTMLTextAreaElement | undefined;
  let isUserScrolledUp = false;

  const [sidebarTab, setSidebarTab] = createSignal<'sessions' | 'files'>('sessions');
  const [showWorkspaceSettings, setShowWorkspaceSettings] = createSignal(false);

  // 1. Workspace State (Projects, Skills, MCP, Folders)
  const ws = useWorkspaceState(props.showToast);

  // 2. Chat & Multi-task Session State (Sessions, SSE, Messages, Models)
  const chat = useChatSession(
    props.showToast,
    ws.selectedProjectId,
    async (pId) => files.loadProjectFiles(pId)
  );

  // 3. Project Workspace Files State (Tree, Tagging, Range Picker)
  const files = useWorkspaceFiles(
    props.showToast,
    ws.selectedProjectId,
    chat.userInput,
    chat.setUserInput,
    () => chatInputRef?.focus()
  );

  // Scroll management
  const handleMessagesScroll = () => {
    if (!messagesContainerRef) return;
    const { scrollTop, scrollHeight, clientHeight } = messagesContainerRef;
    isUserScrolledUp = scrollHeight - (scrollTop + clientHeight) > 120;
  };

  const scrollToBottom = (smooth = true, force = false) => {
    if (!messagesContainerRef) return;
    if (isUserScrolledUp && !force) return;
    messagesContainerRef.scrollTo({
      top: messagesContainerRef.scrollHeight,
      behavior: smooth ? 'smooth' : 'auto',
    });
  };

  // Reset scrolled up state on session switch
  createEffect(() => {
    chat.selectedSessionId();
    isUserScrolledUp = false;
  });

  // Scroll to bottom when sending message
  createEffect(() => {
    if (chat.sendingMessage()) {
      isUserScrolledUp = false;
      setTimeout(() => scrollToBottom(true, true), 30);
    }
  });

  // Autocomplete @tagging in chat input
  const [autocompleteOpen, setAutocompleteOpen] = createSignal(false);
  const [autocompleteQuery, setAutocompleteQuery] = createSignal('');
  const [autocompleteIndex, setAutocompleteIndex] = createSignal(0);
  const [autocompleteCursorPos, setAutocompleteCursorPos] = createSignal(0);

  const autocompleteSuggestions = createMemo(() => {
    if (!autocompleteOpen()) return [];
    const q = autocompleteQuery().toLowerCase();
    const all = files.workspaceFiles();
    if (!q) return all.slice(0, 10);
    return all
      .filter((f) => f.relative_path.toLowerCase().includes(q) || f.name.toLowerCase().includes(q))
      .slice(0, 10);
  });

  const selectAutocompleteFile = (file: WorkspaceFileItem) => {
    const current = chat.userInput();
    const cursorPos = autocompleteCursorPos();
    const textBefore = current.slice(0, cursorPos);
    const textAfter = current.slice(cursorPos);
    const updatedBefore = textBefore.replace(/@([a-zA-Z0-9_\-\./\\]*)$/, `@${file.relative_path} `);
    chat.setUserInput(updatedBefore + textAfter);
    setAutocompleteOpen(false);

    setTimeout(() => {
      if (chatInputRef) {
        chatInputRef.focus();
        const newPos = updatedBefore.length;
        chatInputRef.setSelectionRange(newPos, newPos);
      }
    }, 20);
  };

  const handleChatInput = (e: InputEvent & { currentTarget: HTMLTextAreaElement }) => {
    const target = e.currentTarget;
    const val = target.value;
    chat.setUserInput(val);

    const cursorPos = target.selectionStart || 0;
    setAutocompleteCursorPos(cursorPos);
    const textBefore = val.slice(0, cursorPos);
    const match = textBefore.match(/@([a-zA-Z0-9_\-\./\\]*)$/);

    if (match) {
      setAutocompleteQuery(match[1]);
      setAutocompleteIndex(0);
      setAutocompleteOpen(true);
    } else {
      setAutocompleteOpen(false);
    }
  };

  // Chat Input Send Shortcut Preference ('enter' | 'ctrl_enter')
  const [sendShortcut, setSendShortcut] = createSignal<'enter' | 'ctrl_enter'>(
    (localStorage.getItem('ragasukma_chat_shortcut') as 'ctrl_enter') || 'enter'
  );

  const toggleSendShortcut = () => {
    const next = sendShortcut() === 'enter' ? 'ctrl_enter' : 'enter';
    setSendShortcut(next);
    localStorage.setItem('ragasukma_chat_shortcut', next);
  };

  const handleChatKeyDown = (e: KeyboardEvent & { currentTarget: HTMLTextAreaElement }) => {
    if (autocompleteOpen() && autocompleteSuggestions().length > 0) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setAutocompleteIndex((prev) => (prev + 1) % autocompleteSuggestions().length);
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        setAutocompleteIndex((prev) => (prev - 1 + autocompleteSuggestions().length) % autocompleteSuggestions().length);
        return;
      }
      if (e.key === 'Enter' || e.key === 'Tab') {
        e.preventDefault();
        const item = autocompleteSuggestions()[autocompleteIndex()];
        if (item) selectAutocompleteFile(item);
        return;
      }
      if (e.key === 'Escape') {
        e.preventDefault();
        setAutocompleteOpen(false);
        return;
      }
    }

    const isSendTrigger = sendShortcut() === 'ctrl_enter'
      ? (e.key === 'Enter' && (e.ctrlKey || e.metaKey))
      : (e.key === 'Enter' && !e.shiftKey && !e.ctrlKey && !e.metaKey);

    if (isSendTrigger) {
      e.preventDefault();
      chat.handleSendMessage(() => {
        scrollToBottom(true, true);
        if (chatInputRef) chatInputRef.style.height = 'auto';
      });
    }
  };

  // Initial Load
  const initData = async () => {
    ws.setLoading(true);
    try {
      const [p, s, m] = await Promise.all([
        agentService.getProjects(),
        agentService.getPromptSkills(),
        agentService.getMcpServers(),
      ]);
      ws.setProjects(p);
      ws.setSkills(s);
      ws.setMcpServers(m);

      if (p.length > 0) {
        const savedProj = localStorage.getItem('ragasukma_active_project_id');
        const targetProj = (savedProj && p.some((proj) => proj.id === savedProj)) ? savedProj : p[0].id;
        ws.setSelectedProjectId(targetProj);
        await Promise.all([
          chat.loadSessions(targetProj),
          files.loadProjectFiles(targetProj),
          ws.loadProjectMcpServers(targetProj),
          agentService.getProjectSkills(targetProj).then((s) => ws.setSkills(s)),
        ]);
      }
    } catch (e) {
      console.error(e);
      props.showToast('Failed to load workspace data');
    } finally {
      ws.setLoading(false);
    }
  };

  onMount(() => {
    initData();
    chat.refreshDynamicModels();
  });

  return (
    <div class="h-full flex flex-col space-y-2.5 overflow-hidden">
      {/* ── Top Header Bar ─────────────────────────────────────────────── */}
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-3 border-b border-white/10">
        <div class="flex items-center gap-2.5">
          <div class="p-2 bg-indigo-500/10 text-indigo-400 border border-indigo-500/20 rounded-lg">
            <Layers class="w-5 h-5" />
          </div>
          <div>
            <h2 class="text-base font-semibold text-white">Agent Workspace</h2>
            <p class="text-xs text-white/50">Multi-folder Projects, Task Sessions & MCP</p>
          </div>
        </div>

        {/* Sub-tabs Toggle */}
        <div class="flex items-center gap-1 bg-white/5 p-1 rounded-lg border border-white/5 self-start sm:self-auto">
          <button
            onClick={() => ws.setActiveSubTab('projects')}
            class={`px-3 py-1.5 text-xs font-medium rounded-md transition-colors flex items-center gap-1.5 ${
              ws.activeSubTab() === 'projects'
                ? 'bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/40 font-semibold shadow-sm'
                : 'text-theme-text/60 hover:text-theme-text hover:bg-theme-bg/60'
            }`}
          >
            <FolderGit2 class="w-3.5 h-3.5" />
            Projects
          </button>
          <button
            onClick={() => ws.setActiveSubTab('mcp')}
            class={`px-3 py-1.5 text-xs font-medium rounded-md transition-colors flex items-center gap-1.5 ${
              ws.activeSubTab() === 'mcp'
                ? 'bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/40 font-semibold shadow-sm'
                : 'text-theme-text/60 hover:text-theme-text hover:bg-theme-bg/60'
            }`}
          >
            <Server class="w-3.5 h-3.5" />
            MCP Servers ({ws.mcpServers().length})
          </button>
        </div>
      </div>

      {/* ── SUB-TAB: PROJECTS & WORKSPACES ─────────────────────────────── */}
      <Show when={ws.activeSubTab() === 'projects'}>
        <div class="flex-1 flex flex-row gap-2.5 min-h-0 overflow-hidden">
          {/* Left Column: Projects List & Sessions */}
          <div class={`transition-all duration-200 flex flex-col min-h-0 h-full overflow-hidden ${
            files.selectedFileForTag() ? 'w-64 xl:w-72 shrink-0' : 'w-72 xl:w-80 shrink-0'
          }`}>
            <AgentSidebar
              projects={ws.projects()}
              selectedProjectId={ws.selectedProjectId()}
              onSelectProject={(id) => {
                ws.setSelectedProjectId(id);
                localStorage.setItem('ragasukma_active_project_id', id);
                chat.loadSessions(id);
                files.loadProjectFiles(id);
                ws.loadProjectMcpServers(id);
                agentService.getProjectSkills(id).then((s) => ws.setSkills(s));
              }}
              onNewProject={() => {
                ws.setProjFolders([]);
                ws.setFolderInput('');
                ws.setFolderValidationMsg(null);
                ws.setShowNewProjectModal(true);
              }}
              onDeleteProject={(id, name) => ws.handleDeleteProject(id, name, async (fallbackId) => {
                if (fallbackId) {
                  await chat.loadSessions(fallbackId);
                  files.loadProjectFiles(fallbackId);
                } else {
                  chat.setSessions([]);
                  chat.setSelectedSessionId(null);
                  chat.setMessages([]);
                }
              })}
              currentProject={ws.currentProject()}
              sidebarTab={sidebarTab()}
              setSidebarTab={setSidebarTab}
              sessions={chat.sessions()}
              selectedSessionId={chat.selectedSessionId()}
              onSelectSession={chat.selectSession}
              onQuickCreateSession={() => chat.handleQuickCreateSession(() => chatInputRef?.focus())}
              onDeleteSession={chat.handleDeleteSession}
              isSessionRunning={chat.isSessionRunning}
              fileSearch={files.fileSearch()}
              setFileSearch={files.setFileSearch}
              workspaceFiles={files.workspaceFiles()}
              filteredWorkspaceFiles={files.filteredWorkspaceFiles()}
              fileTree={files.fileTree()}
              loadingFiles={files.loadingFiles()}
              onRefreshFiles={() => {
                const pId = ws.selectedProjectId();
                if (pId) files.loadProjectFiles(pId);
              }}
              onExpandAllDirs={files.expandAllDirs}
              onCollapseAllDirs={files.collapseAllDirs}
              onOpenFolderInOs={() => {
                const proj = ws.currentProject();
                if (!proj) return;
                const folders = parseProjectFolders(proj.root_path);
                if (folders.length > 0) {
                  ws.handleOpenFolderInFileManager(folders[0]);
                } else {
                  props.showToast('No folders attached to workspace');
                }
              }}
              isDirExpanded={files.isDirExpanded}
              onToggleDir={files.toggleDir}
              onInsertFileTag={(path) => files.insertFileTag(path)}
              onOpenRangePicker={files.openLineRangePicker}
              onCopyFileTag={(path) => files.copyFileTag(path)}
            />
          </div>

          {/* Center Column: Execution / Chat Pane */}
          <div class="flex-1 min-w-[340px] flex flex-col space-y-2.5 min-h-0 h-full overflow-hidden">
            <Show when={ws.currentProject()} fallback={
              <div class="h-full bg-white/[0.02] border border-white/10 p-8 flex flex-col items-center justify-center text-center">
                <FolderGit2 class="w-12 h-12 text-white/20 mb-3" />
                <h3 class="text-sm font-semibold text-white">Select or Create a Workspace</h3>
                <p class="text-xs text-white/70 max-w-sm mt-1">
                  Select a workspace from the list on the left or create a new workspace by attaching existing local folders.
                </p>
              </div>
            }>
              {/* Workspace Context Bar */}
              <AgentWorkspaceHeader
                currentProject={ws.currentProject()}
                showWorkspaceSettings={showWorkspaceSettings()}
                onToggleWorkspaceSettings={() => setShowWorkspaceSettings(!showWorkspaceSettings())}
                skills={ws.skills()}
                mcpServers={ws.mcpServers()}
                activeProjectMcpServerIds={ws.activeProjectMcpServerIds()}
                savingProjectMcpServers={ws.savingProjectMcpServers()}
                onToggleProjectMcpServer={ws.handleToggleProjectMcpServer}
                onOpenMcpTab={() => ws.setActiveSubTab('mcp')}
                folderInput={ws.folderInput()}
                setFolderInput={ws.setFolderInput}
                validatingFolder={ws.validatingFolder()}
                browsingFolder={ws.browsingFolder()}
                onAddFolder={ws.handleAddFolderToActiveProject}
                onBrowseFolder={ws.handleBrowseFolderForActiveProject}
                onRemoveFolder={ws.handleRemoveFolderFromActiveProject}
                onOpenFolderInOs={ws.handleOpenFolderInFileManager}
              />

              {/* Active Session Chat Pane */}
              <AgentChatPane
                currentProject={ws.currentProject()}
                currentSession={chat.currentSession()}
                activeSessionModel={chat.activeSessionModel()}
                setActiveSessionModel={chat.setActiveSessionModel}
                defaultBrainModel={chat.defaultBrainModel()}
                availableModels={chat.availableModels()}
                isRefreshingModels={chat.isRefreshingModels()}
                onRefreshModels={chat.refreshDynamicModels}
                setMessagesContainerRef={(el) => (messagesContainerRef = el)}
                onMessagesScroll={handleMessagesScroll}
                visibleMessages={chat.visibleMessages()}
                loadingMessages={chat.loadingMessages()}
                hasMoreMessages={chat.hasMoreMessages()}
                loadingEarlier={chat.loadingEarlier()}
                onLoadOlderMessages={chat.loadOlderMessages}
                onLoadAllMessages={chat.loadAllMessages}
                isMessageCollapsed={chat.isMessageCollapsed}
                toggleCollapseMessage={chat.toggleCollapseMessage}
                isCurrentSessionBusy={chat.isCurrentSessionBusy()}
                currentStepInfo={chat.currentStepInfo()}
                streamingText={chat.streamingText()}
                streamingThought={chat.streamingThought()}
                onQuickCreateSession={() => chat.handleQuickCreateSession(() => chatInputRef?.focus())}
                onOpenFilesTab={() => {
                  setSidebarTab('files');
                  const pId = ws.selectedProjectId();
                  if (pId) files.loadProjectFiles(pId);
                }}
                autocompleteOpen={autocompleteOpen()}
                autocompleteSuggestions={autocompleteSuggestions()}
                autocompleteIndex={autocompleteIndex()}
                onSelectAutocompleteFile={selectAutocompleteFile}
                setChatInputRef={(el) => (chatInputRef = el)}
                userInput={chat.userInput()}
                onChatInput={handleChatInput}
                onChatKeyDown={handleChatKeyDown}
                onSendMessage={() => chat.handleSendMessage(() => {
                  scrollToBottom(true, true);
                  if (chatInputRef) chatInputRef.style.height = 'auto';
                })}
                onStopSession={chat.handleStopSession}
                sendShortcut={sendShortcut()}
                onToggleSendShortcut={toggleSendShortcut}
                showToast={props.showToast}
                onOpenFileInCodePanel={(filePath) => {
                  const allFiles = files.workspaceFiles();
                  const found = allFiles.find((f) => f.relative_path === filePath || filePath.endsWith(f.relative_path));
                  const targetFile: WorkspaceFileItem = found || {
                    relative_path: filePath,
                    name: filePath.split(/[/\\]/).pop() || filePath,
                    is_dir: false,
                    size: 0,
                  };
                  files.openLineRangePicker(targetFile);
                }}
              />
            </Show>
          </div>

          {/* Right Column: Docked Side Code Panel (Split Pane) */}
          <Show when={files.selectedFileForTag()}>
            <div class="w-1/2 min-w-[380px] xl:min-w-[480px] max-w-[50%] flex flex-col min-h-0 h-full overflow-hidden animate-in slide-in-from-right-4 duration-200">
              <AgentCodePanel
                file={files.selectedFileForTag()}
                onClose={() => files.setSelectedFileForTag(null)}
                tagStartLine={files.tagStartLine()}
                tagEndLine={files.tagEndLine()}
                loadingFilePreview={files.loadingFilePreview()}
                highlightedCodeLines={files.highlightedCodeLines()}
                currentSelectedTag={files.currentSelectedTag()}
                onResetSelection={files.handleResetSelection}
                onLineClick={files.handleLineClick}
                onGutterMouseDown={files.handleGutterMouseDown}
                onGutterMouseEnter={files.handleGutterMouseEnter}
                onGutterMouseUp={files.handleGutterMouseUp}
                onCodeTextMouseUp={files.handleCodeTextMouseUp}
                isLineSelected={files.isLineSelected}
                onInsertTag={(relPath, s, e) => files.insertFileTag(relPath, s, e)}
                onCopyTag={() => {
                  const tag = files.currentSelectedTag();
                  if (tag) {
                    navigator.clipboard.writeText(tag);
                    props.showToast(`Tag ${tag} copied to clipboard!`);
                  }
                }}
              />
            </div>
          </Show>
        </div>
      </Show>

      {/* ── SUB-TAB: MCP SERVERS REGISTRY ──────────────────────────────── */}
      <Show when={ws.activeSubTab() === 'mcp'}>
        <McpSubTab
          mcpServers={ws.mcpServers()}
          onNewServer={ws.handleOpenNewMcpServer}
          onEditServer={ws.handleOpenEditMcpServer}
          onDeleteServer={ws.handleDeleteMcpServer}
          onRefreshServers={ws.refreshMcpServers}
        />
      </Show>

      {/* ── MODALS ──────────────────────────────────────────────────────── */}
      <CreateProjectModal
        isOpen={ws.showNewProjectModal()}
        onClose={() => ws.setShowNewProjectModal(false)}
        projName={ws.projName()}
        setProjName={ws.setProjName}
        projDesc={ws.projDesc()}
        setProjDesc={ws.setProjDesc}
        projInstructions={ws.projInstructions()}
        setProjInstructions={ws.setProjInstructions}
        projFolders={ws.projFolders()}
        folderInput={ws.folderInput()}
        setFolderInput={(val) => {
          ws.setFolderInput(val);
          ws.setFolderValidationMsg(null);
        }}
        validatingFolder={ws.validatingFolder()}
        browsingFolder={ws.browsingFolder()}
        folderValidationMsg={ws.folderValidationMsg()}
        onBrowseFolder={ws.handleBrowseFolderForModal}
        onValidateAndAddFolder={ws.handleValidateAndAddFolder}
        onRemoveFolder={ws.handleRemoveFolder}
        onCreateProject={() => ws.handleCreateProject(async (id) => {
          await chat.loadSessions(id);
          files.loadProjectFiles(id);
        })}
      />

      <McpServerModal
        isOpen={ws.showMcpModal()}
        isEditing={!!ws.editingMcpServer()}
        initialMode={ws.mcpModalInitialMode()}
        onClose={() => {
          ws.setShowMcpModal(false);
          ws.setEditingMcpServer(null);
        }}
        onSave={ws.handleSaveMcpServer}
        onTest={ws.handleTestMcpServer}
        initialData={ws.editingMcpServer()}
      />
    </div>
  );
};
