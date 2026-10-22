import { createSignal, createMemo, onMount } from 'solid-js';
import { 
  agentService, 
  Project, 
  PromptSkill, 
  McpServerConfig, 
  parseProjectFolders 
} from '../../services';
import { showConfirm } from '../ConfirmModal';

export function useWorkspaceState(showToast: (msg: string) => void) {
  const [activeSubTab, setActiveSubTab] = createSignal<'projects' | 'skills' | 'mcp'>('projects');
  const [loading, setLoading] = createSignal(false);
  const [projects, setProjects] = createSignal<Project[]>([]);
  const [skills, setSkills] = createSignal<PromptSkill[]>([]);
  const [mcpServers, setMcpServers] = createSignal<McpServerConfig[]>([]);
  const [selectedProjectId, setSelectedProjectId] = createSignal<string | null>(null);

  // Active Project Skills
  const [activeProjectSkillIds, setActiveProjectSkillIds] = createSignal<string[]>([]);
  const [savingProjectSkills, setSavingProjectSkills] = createSignal(false);

  // Modals
  const [showNewProjectModal, setShowNewProjectModal] = createSignal(false);
  const [showNewSkillModal, setShowNewSkillModal] = createSignal(false);
  const [editingSkillId, setEditingSkillId] = createSignal<string | null>(null);

  // New Project Form
  const [projName, setProjName] = createSignal('');
  const [projDesc, setProjDesc] = createSignal('');
  const [projInstructions, setProjInstructions] = createSignal('');
  const [projFolders, setProjFolders] = createSignal<string[]>([]);
  const [folderInput, setFolderInput] = createSignal('');
  const [validatingFolder, setValidatingFolder] = createSignal(false);
  const [folderValidationMsg, setFolderValidationMsg] = createSignal<{ valid: boolean; text: string } | null>(null);
  const [browsingFolder, setBrowsingFolder] = createSignal(false);

  // Prompt Skill Template & Form
  const defaultSkillTemplate = `---
name: Rust Architecture Expert
description: Expert in idiomatic high-performance Rust systems design
tags: rust, coding, backend
---

You are a world-class Rust systems architect. 
Write idiomatic code, prioritize zero-cost abstractions, and maintain uncompromising memory safety.`;

  const [skillMarkdown, setSkillMarkdown] = createSignal(defaultSkillTemplate);

  const currentProject = createMemo(() => {
    return projects().find((p) => p.id === selectedProjectId()) || null;
  });

  const loadProjectSkills = async (projectId: string) => {
    try {
      const active = await agentService.getProjectSkills(projectId);
      setActiveProjectSkillIds(active.map((s) => s.id));
    } catch (e) {
      console.error('Failed to load project prompt skills:', e);
    }
  };

  const handleToggleProjectSkill = async (skillId: string) => {
    const projId = selectedProjectId();
    if (!projId) return;

    const current = activeProjectSkillIds();
    const updated = current.includes(skillId)
      ? current.filter((id) => id !== skillId)
      : [...current, skillId];

    setActiveProjectSkillIds(updated);
    setSavingProjectSkills(true);
    try {
      const ok = await agentService.setProjectSkills(projId, updated);
      if (ok) {
        showToast('Project prompt skills updated');
      } else {
        showToast('Failed to update prompt skills');
      }
    } catch (e: any) {
      showToast(e.message || 'Failed to save skills');
    } finally {
      setSavingProjectSkills(false);
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
      title: 'Hapus Project Workspace',
      message: `Hapus workspace "${name}" dan seluruh sesi task di dalamnya? Folder fisik di disk tetap aman.`,
      confirmText: 'Hapus Workspace',
      cancelText: 'Batal',
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

  const parseSkillMarkdown = (raw: string) => {
    let name = '';
    let description = '';
    let tags: string[] = [];
    let systemPrompt = '';

    const trimmed = raw.trim();
    if (trimmed.startsWith('---')) {
      const parts = trimmed.split('---');
      if (parts.length >= 3) {
        const frontmatter = parts[1];
        systemPrompt = parts.slice(2).join('---').trim();

        for (const line of frontmatter.split('\n')) {
          const l = line.trim();
          if (l.toLowerCase().startsWith('name:')) {
            name = l.substring(5).trim();
          } else if (l.toLowerCase().startsWith('description:')) {
            description = l.substring(12).trim();
          } else if (l.toLowerCase().startsWith('tags:')) {
            tags = l.substring(5).split(',').map((t) => t.trim()).filter(Boolean);
          }
        }
      } else {
        systemPrompt = trimmed;
      }
    } else {
      systemPrompt = trimmed;
    }

    if (!name) {
      const firstLine = systemPrompt.split('\n')[0] || 'Custom Skill';
      name = firstLine.replace(/^#+\s*/, '').slice(0, 40).trim() || 'Custom Skill';
    }

    const slug = name.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'skill';
    return { name, description, tags, systemPrompt, slug };
  };

  const skillToMarkdown = (skill: PromptSkill) => {
    let tagsStr = '';
    try {
      const parsed = typeof skill.tags === 'string' ? JSON.parse(skill.tags) : skill.tags;
      if (Array.isArray(parsed)) tagsStr = parsed.join(', ');
    } catch {}

    return `---
name: ${skill.name}
description: ${skill.description || ''}
tags: ${tagsStr}
---

${skill.system_prompt}
`.trim();
  };

  const handleOpenNewSkill = () => {
    setEditingSkillId(null);
    setSkillMarkdown(defaultSkillTemplate);
    setShowNewSkillModal(true);
  };

  const handleOpenEditSkill = (skill: PromptSkill) => {
    setEditingSkillId(skill.id);
    setSkillMarkdown(skillToMarkdown(skill));
    setShowNewSkillModal(true);
  };

  const handleCreateSkill = async () => {
    const raw = skillMarkdown().trim();
    if (!raw) {
      showToast('SKILL.md Markdown content cannot be empty!');
      return;
    }

    const parsed = parseSkillMarkdown(raw);
    if (!parsed.systemPrompt) {
      showToast('System prompt instructions cannot be empty!');
      return;
    }

    const editId = editingSkillId();
    if (editId) {
      const res = await agentService.updatePromptSkill(editId, {
        name: parsed.name,
        slug: parsed.slug,
        description: parsed.description,
        system_prompt: parsed.systemPrompt,
        tags: parsed.tags,
      });

      if (res.ok && res.skill) {
        showToast(`Prompt skill "${res.skill.name}" updated!`);
        setShowNewSkillModal(false);
        setEditingSkillId(null);
        setSkillMarkdown(defaultSkillTemplate);
        const sList = await agentService.getPromptSkills();
        setSkills(sList);
      } else {
        showToast(res.error || 'Failed to update skill');
      }
    } else {
      const res = await agentService.createPromptSkill({
        name: parsed.name,
        slug: parsed.slug,
        description: parsed.description,
        system_prompt: parsed.systemPrompt,
        tags: parsed.tags,
      });

      if (res.ok && res.skill) {
        showToast(`Prompt skill "${res.skill.name}" added!`);
        setShowNewSkillModal(false);
        setEditingSkillId(null);
        setSkillMarkdown(defaultSkillTemplate);
        const sList = await agentService.getPromptSkills();
        setSkills(sList);
      } else {
        showToast(res.error || 'Failed to add skill');
      }
    }
  };

  const handleDeleteSkill = async (id: string, name: string) => {
    const ok = await showConfirm({
      title: 'Hapus Prompt Skill',
      message: `Hapus prompt skill preset "${name}"? Aturan persona ini tidak akan diinjeksikan lagi ke agent.`,
      confirmText: 'Hapus Skill',
      cancelText: 'Batal',
      isDanger: true,
    });
    if (!ok) return;
    const okDel = await agentService.deletePromptSkill(id);
    if (okDel) {
      showToast(`Prompt skill "${name}" deleted`);
      const sList = await agentService.getPromptSkills();
      setSkills(sList);
    } else {
      showToast('Failed to delete skill');
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
    activeProjectSkillIds,
    setActiveProjectSkillIds,
    savingProjectSkills,
    loadProjectSkills,
    handleToggleProjectSkill,
    showNewProjectModal,
    setShowNewProjectModal,
    showNewSkillModal,
    setShowNewSkillModal,
    editingSkillId,
    setEditingSkillId,
    handleOpenNewSkill,
    handleOpenEditSkill,
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
    skillMarkdown,
    setSkillMarkdown,
    defaultSkillTemplate,
    parseSkillMarkdown,
    handleOpenFolderInFileManager,
    handleBrowseFolderForModal,
    handleBrowseFolderForActiveProject,
    handleValidateAndAddFolder,
    handleRemoveFolder,
    handleAddFolderToActiveProject,
    handleRemoveFolderFromActiveProject,
    handleCreateProject,
    handleDeleteProject,
    handleCreateSkill,
    handleDeleteSkill,
  };
}
