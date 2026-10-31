import { Component, createSignal, onMount, Show, createMemo, createEffect } from 'solid-js';
import { 
  FolderGit2, 
  Layers, 
  Sparkles, 
  Server,
  RefreshCw
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
import { findActiveTagQuery, insertFileTag } from './fileTags';

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

  // Note: Auto-scroll on send is now handled via onMessageSent callback in handleSendMessage

  // Autocomplete @tagging in chat input
  const [autocompleteOpen, setAutocompleteOpen] = createSignal(false);
  const [autocompleteQuery, setAutocompleteQuery] = createSignal('');
  const [autocompleteIndex, setAutocompleteIndex] = createSignal(0);
  const [autocompleteCursorPos, setAutocompleteCursorPos] = createSignal(0);

  const autocompleteSuggestions = createMemo(() => {
    if (!autocompleteOpen()) return [];
    const q = autocompleteQuery().toLowerCase();
    const all = files.workspaceFiles();

    // No query yet: the raw list is sorted root-folder-first, so a plain
    // `slice(0, 10)` showed only folders and never a single real file. Prefer
    // actual files here so "@" immediately surfaces something taggable.
    if (!q) {
      const realFiles = all.filter((f) => !f.is_dir);
      return (realFiles.length > 0 ? realFiles : all).slice(0, 10);
    }

    // Rank matches: basename prefix > basename contains > path contains.
    // `includes()` alone buried `agentService.ts` behind unrelated paths.
    const scored: { file: WorkspaceFileItem; score: number }[] = [];
    for (const f of all) {
      const name = f.name.toLowerCase();
      const path = f.relative_path.toLowerCase();
      let score = -1;
      if (name.startsWith(q)) score = 0;
      else if (name.includes(q)) score = 1;
      else if (path.includes(q)) score = 2;
      if (score >= 0) scored.push({ file: f, score });
    }
    scored.sort((a, b) => a.score - b.score || a.file.relative_path.localeCompare(b.file.relative_path));
    return scored.slice(0, 10).map((s) => s.file);
  });

  const selectAutocompleteFile = (file: WorkspaceFileItem) => {
    const current = chat.userInput();
    // Prefer the live caret: the user may have moved it with the mouse/arrows
    // after the last `input` event.
    const cursorPos = chatInputRef?.selectionStart ?? autocompleteCursorPos();
    // Replaces ONLY the `@token` under the caret, so previously inserted tags
    // in the same prompt survive (multi-tag prompts are supported).
    const { value, caret } = insertFileTag(current, cursorPos, file.relative_path);
    chat.setUserInput(value);
    setAutocompleteCursorPos(caret);
    setAutocompleteOpen(false);

    setTimeout(() => {
      if (chatInputRef) {
        chatInputRef.focus();
        chatInputRef.setSelectionRange(caret, caret);
      }
    }, 20);
  };

  const handleChatInput = (e: InputEvent & { currentTarget: HTMLTextAreaElement }) => {
    const target = e.currentTarget;
    const val = target.value;
    chat.setUserInput(val);

    const cursorPos = target.selectionStart || 0;
    setAutocompleteCursorPos(cursorPos);
    // Shared grammar with the tag renderer & the inserter: a tag is `@` plus
    // non-whitespace/non-`@` chars, so earlier tags in the prompt stay intact.
    const active = findActiveTagQuery(val, cursorPos);

    if (active) {
      setAutocompleteQuery(active.query);
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
        scrollToBottom(true);
        if (chatInputRef) chatInputRef.style.height = 'auto';
      });
    }
  };

  let projectLoadToken = 0;
  const loadProjectData = async (projectId: string) => {
    const token = ++projectLoadToken;
    const [sessionsResult, filesResult, mcpResult, skillsResult] = await Promise.allSettled([
      chat.loadSessions(projectId),
      files.loadProjectFiles(projectId),
      ws.loadProjectMcpServers(projectId),
      agentService.getProjectSkills(projectId),
    ]);
    if (token !== projectLoadToken || ws.selectedProjectId() !== projectId) return;
    if (skillsResult.status === 'fulfilled') ws.setSkills(skillsResult.value);
    else console.error('Failed to load project skills', skillsResult.reason);
    for (const [name, result] of [['sessions', sessionsResult], ['files', filesResult], ['MCP servers', mcpResult]] as const) {
      if (result.status === 'rejected') console.error(`Failed to load project ${name}`, result.reason);
    }
  };

  // Initial Load
  const initData = async () => {
    ws.setLoading(true);
    try {
      // Step 1: Load models first, before loading sessions
      // This ensures models are available when selectSession/loadSessions runs
      await chat.refreshDynamicModels();

      // Step 2: Load projects, skills, and MCP servers
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
        await loadProjectData(targetProj);
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
  });

  return (
    <div class="h-full flex flex-col space-y-2.5 overflow-hidden">
      {/* ── Top Header Bar ─────────────────────────────────────────────── */}
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3 pb-3 border-b border-white/10">
        <div class="flex items-center gap-2.5">
          <div class="p-2 bg-theme-cyan/10 text-theme-cyan border border-theme-cyan/20">
            <Layers class="w-5 h-5" />
          </div>
          <div>
            <h2 class="text-base font-semibold text-theme-text">Agent Workspace</h2>
            <p class="text-xs text-theme-text/50">Multi-folder Projects, Task Sessions & MCP</p>
          </div>
        </div>

        {/* Sub-tabs Toggle */}
        <div class="flex items-center gap-1 bg-white/5 p-1 border border-white/5 self-start sm:self-auto">
          <button
            onClick={() => ws.setActiveSubTab('projects')}
            class={`px-3 py-1.5 text-xs font-medium transition-colors flex items-center gap-1.5 ${
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
            class={`px-3 py-1.5 text-xs font-medium transition-colors flex items-center gap-1.5 ${
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
        <Show
          when={!ws.loading() || ws.projects().length > 0}
          fallback={
            <div class="flex-1 flex flex-col items-center justify-center p-8 text-center min-h-[400px]">
              <div class="relative flex items-center justify-center mb-4">
                <div class="w-10 h-10 rounded-full border-2 border-theme-cyan/20 border-t-theme-cyan animate-spin" />
                <RefreshCw class="w-4 h-4 text-theme-cyan absolute animate-spin" />
              </div>
              <h4 class="text-sm font-semibold text-theme-text mb-1 tracking-wide">Loading Workspace...</h4>
              <p class="text-xs text-theme-text/50 max-w-xs">
                Fetching projects, sessions, and agent configuration.
              </p>
            </div>
          }
        >
          <div class="flex-1 flex flex-row gap-2.5 min-h-0 overflow-hidden">
          {/* Left Column: Projects List & Sessions */}
          <div class="w-72 xl:w-80 shrink-0 flex flex-col min-h-0 h-full overflow-hidden">
            <AgentSidebar
              projects={ws.projects()}
              selectedProjectId={ws.selectedProjectId()}
              onSelectProject={(id) => {
                ws.setSelectedProjectId(id);
                localStorage.setItem('ragasukma_active_project_id', id);
                loadProjectData(id);
              }}
              onNewProject={() => {
                ws.setProjFolders([]);
                ws.setFolderInput('');
                ws.setFolderValidationMsg(null);
                ws.setShowNewProjectModal(true);
              }}
              onDeleteProject={(id, name) => ws.handleDeleteProject(id, name, async (fallbackId) => {
                if (fallbackId) {
                  ws.setSelectedProjectId(fallbackId);
                  localStorage.setItem('ragasukma_active_project_id', fallbackId);
                  await loadProjectData(fallbackId);
                } else {
                  projectLoadToken++;
                  ws.setSelectedProjectId(null);
                  localStorage.removeItem('ragasukma_active_project_id');
                  chat.setSessions([]);
                  chat.setSelectedSessionId(null);
                  chat.setMessages([]);
                  files.setWorkspaceFiles([]);
                  ws.setSkills([]);
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
              getLineCount={files.getLineCount}
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
                getLineCount={files.getLineCount}
                setChatInputRef={(el) => (chatInputRef = el)}
                userInput={chat.userInput()}
                onChatInput={handleChatInput}
                onChatKeyDown={handleChatKeyDown}
                onSendMessage={() => chat.handleSendMessage(() => {
                  scrollToBottom(true);
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
                onClose={files.closeLineRangePicker}
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