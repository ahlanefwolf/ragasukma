import { createSignal, createMemo } from 'solid-js';
import { 
  agentService, 
  Project, 
  PromptSkill, 
  McpServerConfig,
  SaveMcpServerPayload,
  parseProjectFolders 
} from '../../services';
import { showConfirm } from '../ConfirmModal';

export function useWorkspaceState(showToast: (msg: string) => void) {
  const [activeSubTab, setActiveSubTab] = createSignal<'projects' | 'mcp'>('projects');
  const [loading, setLoading] = createSignal(false);
  const [projects, setProjects] = createSignal<Project[]>([]);
  const [selectedProjectId, setSelectedProjectId] = createSignal<string | null>(null);
  const [skills, setSkills] = createSignal<PromptSkill[]>([]);
  const [mcpServers, setMcpServers] = createSignal<McpServerConfig[]>([]);
  const [activeProjectMcpServerIds, setActiveProjectMcpServerIds] = createSignal<string[]>([]);
  const [sessionActiveMcpServerIds, setSessionActiveMcpServerIds] = createSignal<string[]>([]);
  const [savingProjectMcpServers, setSavingProjectMcpServers] = createSignal(false);
  const [savingSessionMcpServers, setSavingSessionMcpServers] = createSignal(false);

  // Modals
  const [showNewProjectModal, setShowNewProjectModal] = createSignal(false);
  const [showMcpModal, setShowMcpModal] = createSignal(false);
  const [editingMcpServer, setEditingMcpServer] = createSignal<McpServerConfig | null>(null);
  const [mcpModalInitialMode, setMcpModalInitialMode] = createSignal<'form' | 'json'>('json');

  // New Project Form
  const [projName, setProjName] = createSignal('');
  const [projDesc, setProjDesc] = createSignal('');
  const [projInstructions, setProjInstructions] = createSignal('');
  const [projFolders, setProjFolders] = createSignal<string[]>([]);
  const [folderInput, setFolderInput] = createSignal('');
  const [validatingFolder, setValidatingFolder] = createSignal(false);
  const [folderValidationMsg, setFolderValidationMsg] = createSignal<{ valid: boolean; text: string } | null>(null);
  const [browsingFolder, setBrowsingFolder] = createSignal(false);

  const currentProject = createMemo(() => {
    return projects().find((p) => p.id === selectedProjectId()) || null;
  });

  const loadSessionMcpServers = async (sessionId: string) => {
    try {
      const activeIds = await agentService.getSessionMcpServers(sessionId);
      setSessionActiveMcpServerIds(activeIds);
    } catch (e) {
      console.error('Failed to load session MCP servers:', e);
    }
  };

  const handleToggleSessionMcpServer = async (sessionId: string, serverId: string) => {
    if (!sessionId) return;
    const current = sessionActiveMcpServerIds();
    const willBeActive = !current.includes(serverId);
    const updated = willBeActive
      ? [...current, serverId]
      : current.filter((id) => id !== serverId);

    setSessionActiveMcpServerIds(updated);
    setSavingSessionMcpServers(true);
    try {
      const ok = await agentService.setSessionMcpServer(sessionId, serverId, willBeActive);
      if (ok) {
        showToast(`MCP server ${willBeActive ? 'enabled' : 'disabled'} for session`);
      } else {
        showToast('Failed to update session MCP server');
        // Rollback state on failure
        setSessionActiveMcpServerIds(current);
      }
    } catch (e: any) {
      showToast(e.message || 'Failed to toggle session MCP server');
      setSessionActiveMcpServerIds(current);
    } finally {
      setSavingSessionMcpServers(false);
    }
  };

  const loadProjectMcpServers = async (projectId: string) => {
    try {
      const activeIds = await agentService.getProjectMcpServers(projectId);
      setActiveProjectMcpServerIds(activeIds);
    } catch (e) {
      console.error('Failed to load project MCP servers:', e);
    }
  };

  const handleToggleProjectMcpServer = async (serverId: string) => {
    const projId = selectedProjectId();
    if (!projId) return;

    const current = activeProjectMcpServerIds();
    const updated = current.includes(serverId)
      ? current.filter((id) => id !== serverId)
      : [...current, serverId];

    setActiveProjectMcpServerIds(updated);
    setSavingProjectMcpServers(true);
    try {
      const ok = await agentService.setProjectMcpServers(projId, updated);
      if (ok) {
        showToast('Project MCP servers updated');
      } else {
        showToast('Failed to update project MCP servers');
      }
    } catch (e: any) {
      showToast(e.message || 'Failed to save project MCP servers');
    } finally {
      setSavingProjectMcpServers(false);
    }
  };

  const handleOpenFolderInFileManager = async (folderPath: string) => {
    try {
      const res = await agentService.openInFileManager(folderPath);
      if (res.ok) {
        showToast(`Opening "${folderPath}" in OS file manager...`);
      } else {
        showToast(res.error || 'Failed to open file manager');
      }
    } catch (e: any) {
      showToast(e.message || 'Failed to open file manager');
    }
  };

  const handleBrowseFolderForModal = async () => {
    setBrowsingFolder(true);
    showToast('Opening file manager dialog on desktop...');
    try {
      const res = await agentService.browseFolder(folderInput());
      if (res.ok && res.path) {
        if (projFolders().includes(res.path)) {
          setFolderValidationMsg({ valid: false, text: 'Folder already exists in this workspace list!' });
        } else {
          setProjFolders([...projFolders(), res.path]);
          setFolderInput('');
          setFolderValidationMsg({ valid: true, text: `Folder selected: ${res.path}` });
        }
      } else if (res.error) {
        showToast(res.error);
      }
    } catch (e: any) {
      showToast(e.message || 'Failed to open file manager');
    } finally {
      setBrowsingFolder(false);
    }
  };

  const handleBrowseFolderForActiveProject = async (onProjectUpdated?: (projId: string) => void) => {
    const proj = currentProject();
    if (!proj) return;

    setBrowsingFolder(true);
    showToast('Opening file manager dialog on desktop...');
    try {
      const res = await agentService.browseFolder();
      if (res.ok && res.path) {
        const current = parseProjectFolders(proj.root_path);
        if (current.includes(res.path)) {
          showToast('Folder already attached to this workspace!');
          return;
        }
        const updated = [...current, res.path];
        const updateRes = await agentService.updateProject(proj.id, {
          name: proj.name,
          description: proj.description,
          folders: updated,
          custom_instructions: proj.custom_instructions,
        });
        if (updateRes.ok) {
          showToast(`Folder "${res.path}" attached successfully!`);
          const pList = await agentService.getProjects();
          setProjects(pList);
          onProjectUpdated?.(proj.id);
        } else {
          showToast(updateRes.error || 'Failed to attach folder');
        }
      } else if (res.error) {
        showToast(res.error);
      }
    } catch (e: any) {
      showToast(e.message || 'Failed to open file manager');
    } finally {
      setBrowsingFolder(false);
    }
  };

  const handleValidateAndAddFolder = async () => {
    const raw = folderInput().trim();
    if (!raw) return;

    if (projFolders().includes(raw)) {
      setFolderValidationMsg({ valid: false, text: 'Folder already exists in this workspace list!' });
      return;
    }

    setValidatingFolder(true);
    setFolderValidationMsg(null);
    try {
      const res = await agentService.validateFolder(raw);
      if (res.valid && res.path) {
        setProjFolders([...projFolders(), res.path]);
        setFolderInput('');
        setFolderValidationMsg({ valid: true, text: `Valid folder: ${res.path}` });
      } else {
        setFolderValidationMsg({ valid: false, text: res.error || 'Folder not found on disk!' });
      }
    } catch (e: any) {
      setFolderValidationMsg({ valid: false, text: e.message || 'Failed to validate folder' });
    } finally {
      setValidatingFolder(false);
    }
  };

  const handleRemoveFolder = (folderToRemove: string) => {
    setProjFolders(projFolders().filter((f) => f !== folderToRemove));
  };

  const handleAddFolderToActiveProject = async () => {
    const proj = currentProject();
    if (!proj) return;
    const raw = folderInput().trim();
    if (!raw) return;

    const currentFolders = parseProjectFolders(proj.root_path);
    if (currentFolders.includes(raw)) {
      showToast('This folder is already part of the workspace');
      return;
    }

    setValidatingFolder(true);
    try {
      const res = await agentService.validateFolder(raw);
      if (!res.valid || !res.path) {
        showToast(res.error || 'Folder is invalid or does not exist on disk!');
        return;
      }

      const updated = [...currentFolders, res.path];
      const updateRes = await agentService.updateProject(proj.id, {
        name: proj.name,
        description: proj.description,
        folders: updated,
        custom_instructions: proj.custom_instructions,
      });

      if (updateRes.ok) {
        showToast('Folder added to workspace successfully');
        setFolderInput('');
        const pList = await agentService.getProjects();
        setProjects(pList);
      } else {
        showToast(updateRes.error || 'Failed to update workspace folders');
      }
    } finally {
      setValidatingFolder(false);
    }
  };

  const handleRemoveFolderFromActiveProject = async (folderToRemove: string) => {
    const proj = currentProject();
    if (!proj) return;
    const currentFolders = parseProjectFolders(proj.root_path);
    const updated = currentFolders.filter((f: string) => f !== folderToRemove);

    const updateRes = await agentService.updateProject(proj.id, {
      name: proj.name,
      description: proj.description,
      folders: updated,
      custom_instructions: proj.custom_instructions,
    });

    if (updateRes.ok) {
      showToast('Folder removed from workspace');
      const pList = await agentService.getProjects();
      setProjects(pList);
    } else {
      showToast(updateRes.error || 'Failed to remove folder');
    }
  };

  const handleCreateProject = async (onCreated: (projId: string) => Promise<void>) => {
    if (!projName().trim()) {
      showToast('Project name cannot be empty');
      return;
    }

    const res = await agentService.createProject({
      name: projName().trim(),
      description: projDesc().trim(),
      folders: projFolders(),
      custom_instructions: projInstructions().trim(),
    });

    if (res.ok && res.project) {
      showToast(`Project "${res.project.name}" created successfully!`);
      setShowNewProjectModal(false);
      setProjName('');
      setProjDesc('');
      setProjInstructions('');
      setProjFolders([]);
      setFolderInput('');
      setFolderValidationMsg(null);

      const pList = await agentService.getProjects();
      setProjects(pList);
      setSelectedProjectId(res.project.id);
      await onCreated(res.project.id);
    } else {
      showToast(res.error || 'Failed to create project');
    }
  };

  const handleDeleteProject = async (id: string, name: string, onDeleted: (fallbackId: string | null) => Promise<void>) => {
    const ok = await showConfirm({
      title: 'Delete Project Workspace',
      message: `Delete workspace "${name}" and all task sessions inside it? Physical folder on disk remains safe.`,
      confirmText: 'Delete Workspace',
      cancelText: 'Cancel',
      isDanger: true,
    });
    if (!ok) return;
    const okDel = await agentService.deleteProject(id);
    if (okDel) {
      showToast(`Workspace "${name}" deleted`);
      const pList = await agentService.getProjects();
      setProjects(pList);
      if (pList.length > 0) {
        setSelectedProjectId(pList[0].id);
        await onDeleted(pList[0].id);
      } else {
        setSelectedProjectId(null);
        await onDeleted(null);
      }
    } else {
      showToast('Failed to delete workspace');
    }
  };

  return {
    activeSubTab,
    setActiveSubTab,
    loading,
    setLoading,
    projects,
    setProjects,
    skills,
    setSkills,
    mcpServers,
    setMcpServers,
    selectedProjectId,
    setSelectedProjectId,
    currentProject,
    activeProjectMcpServerIds,
    sessionActiveMcpServerIds,
    savingProjectMcpServers,
    savingSessionMcpServers,
    loadProjectMcpServers,
    loadSessionMcpServers,
    handleToggleProjectMcpServer,
    handleToggleSessionMcpServer,
    showNewProjectModal,
    setShowNewProjectModal,
    projName,
    setProjName,
    projDesc,
    setProjDesc,
    projInstructions,
    setProjInstructions,
    projFolders,
    setProjFolders,
    folderInput,
    setFolderInput,
    validatingFolder,
    folderValidationMsg,
    setFolderValidationMsg,
    browsingFolder,
    handleOpenFolderInFileManager,
    handleBrowseFolderForModal,
    handleBrowseFolderForActiveProject,
    handleValidateAndAddFolder,
    handleRemoveFolder,
    handleAddFolderToActiveProject,
    handleRemoveFolderFromActiveProject,
    handleCreateProject,
    handleDeleteProject,
    showMcpModal,
    setShowMcpModal,
    editingMcpServer,
    setEditingMcpServer,
    mcpModalInitialMode,
    setMcpModalInitialMode,
    handleOpenNewMcpServer: (mode: 'form' | 'json' = 'json') => {
      setEditingMcpServer(null);
      setMcpModalInitialMode(mode);
      setShowMcpModal(true);
    },
    handleOpenEditMcpServer: (server: McpServerConfig, mode: 'form' | 'json' = 'json') => {
      setEditingMcpServer(server);
      setMcpModalInitialMode(mode);
      setShowMcpModal(true);
    },
    handleSaveMcpServer: async (payload: SaveMcpServerPayload): Promise<boolean> => {
      try {
        const res = await agentService.saveMcpServer(payload);
        if (res.ok) {
          showToast(`MCP server "${payload.name}" saved!`);
          setShowMcpModal(false);
          setEditingMcpServer(null);
          const mList = await agentService.getMcpServers();
          setMcpServers(mList);
          return true;
        } else {
          showToast(res.error || 'Failed to save MCP server');
          return false;
        }
      } catch (e: any) {
        showToast(e.message || 'Failed to save MCP server');
        return false;
      }
    },
    handleDeleteMcpServer: async (id: string, name: string) => {
      const ok = await showConfirm({
        title: 'Delete MCP Server',
        message: `Remove MCP server "${name}"? It will be unlinked from any workspace.`,
        confirmText: 'Delete Server',
        cancelText: 'Cancel',
        isDanger: true,
      });
      if (!ok) return;
      try {
        const okDel = await agentService.deleteMcpServer(id);
        if (okDel) {
          showToast(`MCP server "${name}" deleted`);
          const mList = await agentService.getMcpServers();
          setMcpServers(mList);
        } else {
          showToast('Failed to delete MCP server');
        }
      } catch (e: any) {
        showToast(e.message || 'Failed to delete MCP server');
      }
    },
    refreshMcpServers: async () => {
      try {
        const mList = await agentService.getMcpServers();
        setMcpServers(mList);
      } catch (e) {
        console.error('Failed to refresh MCP servers:', e);
      }
    },
    handleTestMcpServer: async (serverOrPayload: SaveMcpServerPayload | McpServerConfig) => {
      let payload: SaveMcpServerPayload;
      if ('created_at' in serverOrPayload) {
        payload = {
          id: serverOrPayload.id,
          name: serverOrPayload.name,
          config: serverOrPayload.config,
          is_enabled: serverOrPayload.is_enabled !== 0,
        };
      } else {
        payload = serverOrPayload;
      }
      return await agentService.testMcpServer(payload);
    },
  };
}