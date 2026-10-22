import { API_BASE, fetchWithTimeout } from './base';

// Workspace & Agent Service Interface

export interface PromptSkill {
  id: string;
  name: string;
  slug: string;
  description: string;
  system_prompt: string;
  tags: string;
  is_active: number;
  created_at: number;
  updated_at: number;
}

export interface McpServerConfig {
  id: string;
  name: string;
  transport: 'stdio' | 'sse' | 'http';
  command?: string;
  args: string;
  env: string;
  url?: string;
  is_enabled: number;
  created_at: number;
}

export interface Project {
  id: string;
  name: string;
  description: string;
  root_path?: string; // Stored as JSON array or single path
  custom_instructions?: string;
  created_at: number;
  updated_at: number;
}

export interface AgentSession {
  id: string;
  project_id: string;
  title: string;
  status: 'idle' | 'running' | 'waiting_tool' | 'completed' | 'failed';
  model?: string;
  temperature?: number;
  source_chat_id?: string;
  metadata?: string;
  created_at: number;
  updated_at: number;
}

export interface WorkspaceFileItem {
  relative_path: string;
  name: string;
  is_dir: boolean;
  size: number;
  line_count?: number;
}

export interface FileContentResult {
  path: string;
  total_lines: number;
  start_line: number;
  end_line: number;
  content: string;
}

export interface AgentMessage {
  id: string;
  session_id: string;
  sequence_num: number;
  role: 'user' | 'assistant' | 'system' | 'tool';
  content?: string;
  thought?: string;
  tool_calls?: string;
  tool_call_id?: string;
  status: 'streaming' | 'success' | 'error';
  token_usage_input?: number;
  token_usage_output?: number;
  created_at: number;
}

export function parseProjectFolders(rootPath?: string): string[] {
  if (!rootPath) return [];
  try {
    const parsed = JSON.parse(rootPath);
    if (Array.isArray(parsed)) return parsed.map((p) => String(p));
  } catch {}
  return [rootPath];
}

export const agentService = {
  // Projects
  async getProjects(): Promise<Project[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/projects`);
      if (!res.ok) return [];
      return await res.json();
    } catch (e) {
      console.error('Failed to get agent projects:', e);
      return [];
    }
  },

  async getProjectFiles(projectId: string): Promise<WorkspaceFileItem[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/projects/${projectId}/files`);
      if (!res.ok) return [];
      const data = await res.json();
      return data.files || [];
    } catch (e) {
      console.error('Failed to get project files:', e);
      return [];
    }
  },

  async getFileContent(
    projectId: string,
    path: string,
    startLine?: number,
    endLine?: number
  ): Promise<FileContentResult | null> {
    try {
      const params = new URLSearchParams({ path });
      if (startLine !== undefined) params.append('start_line', String(startLine));
      if (endLine !== undefined) params.append('end_line', String(endLine));

      const res = await fetchWithTimeout(
        `${API_BASE}/api/agent/projects/${projectId}/file-content?${params.toString()}`
      );
      if (!res.ok) return null;
      return await res.json();
    } catch (e) {
      console.error('Failed to get file content:', e);
      return null;
    }
  },

  async createProject(payload: {
    name: string;
    description?: string;
    folders?: string[];
    custom_instructions?: string;
  }): Promise<{ ok: boolean; project?: Project; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/projects`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      const data = await res.json();
      if (!res.ok) {
        return { ok: false, error: data?.error || 'Failed to create project' };
      }
      return { ok: true, project: data };
    } catch (e: any) {
      return { ok: false, error: e?.message || 'Network error' };
    }
  },

  async updateProject(
    id: string,
    payload: {
      name: string;
      description?: string;
      folders?: string[];
      custom_instructions?: string;
    }
  ): Promise<{ ok: boolean; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/projects/${id}/update`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      const data = await res.json();
      if (!res.ok) {
        return { ok: false, error: data?.error || 'Failed to update project' };
      }
      return { ok: true };
    } catch (e: any) {
      return { ok: false, error: e?.message || 'Network error' };
    }
  },

  async deleteProject(id: string): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/projects/${id}/delete`, {
        method: 'POST',
      });
      return res.ok;
    } catch (e) {
      console.error('Failed to delete project:', e);
      return false;
    }
  },

  async validateFolder(path: string): Promise<{ valid: boolean; path?: string; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/validate-folder`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ path }),
      });
      return await res.json();
    } catch (e: any) {
      return { valid: false, error: e?.message || 'Network error' };
    }
  },

  async browseFolder(initialPath?: string): Promise<{ ok: boolean; path?: string; cancelled?: boolean; error?: string }> {
    try {
      const res = await fetchWithTimeout(
        `${API_BASE}/api/agent/browse-folder`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ initial_path: initialPath }),
        },
        180000 // 3 minutes timeout for user to select folder in dialog
      );
      return await res.json();
    } catch (e: any) {
      return { ok: false, error: e?.message || 'Failed to open file manager' };
    }
  },

  async openInFileManager(path: string): Promise<{ ok: boolean; error?: string }> {
    try {
      const res = await fetchWithTimeout(
        `${API_BASE}/api/agent/open-in-file-manager`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ path }),
        },
        15000
      );
      return await res.json();
    } catch (e: any) {
      return { ok: false, error: e?.message || 'Failed to open file manager' };
    }
  },

  // Sessions
  async getSessions(projectId: string): Promise<AgentSession[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/projects/${projectId}/sessions`);
      if (!res.ok) return [];
      return await res.json();
    } catch (e) {
      console.error('Failed to get agent sessions:', e);
      return [];
    }
  },

  async createSession(
    projectId: string,
    payload: {
      title: string;
      model?: string;
      temperature?: number;
      source_chat_id?: string;
    }
  ): Promise<{ ok: boolean; session?: AgentSession; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/projects/${projectId}/sessions`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      const data = await res.json();
      if (!res.ok) {
        return { ok: false, error: data?.error || 'Failed to create session' };
      }
      return { ok: true, session: data };
    } catch (e: any) {
      return { ok: false, error: e?.message || 'Network error' };
    }
  },

  async deleteSession(id: string): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/sessions/${id}/delete`, {
        method: 'POST',
      });
      return res.ok;
    } catch (e) {
      console.error('Failed to delete session:', e);
      return false;
    }
  },

  async runSession(sessionId: string, model?: string): Promise<{ ok: boolean; reply?: string; error?: string }> {
    try {
      // Agent workspace multi-turn execution takes time (up to 5 minutes for coding & tool calls)
      const res = await fetchWithTimeout(
        `${API_BASE}/api/agent/sessions/${sessionId}/run`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ model: model || undefined }),
        },
        300000
      );
      const data = await res.json();
      if (!res.ok) {
        return { ok: false, error: data?.error || 'Failed to run agent' };
      }
      return { ok: true, reply: data?.reply };
    } catch (e: any) {
      return { ok: false, error: e?.message || 'Network error' };
    }
  },

  async stopSession(sessionId: string): Promise<boolean> {
    try {
      const res = await fetch(`${API_BASE}/api/agent/sessions/${sessionId}/stop`, {
        method: 'POST',
      });
      return res.ok;
    } catch (e) {
      console.error('Failed to stop session', e);
      return false;
    }
  },

  async createSessionFromChat(payload: {
    chat_id: string;
    project_id: string;
    title?: string;
  }): Promise<{ ok: boolean; session?: AgentSession; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/sessions/from-chat`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      const data = await res.json();
      if (!res.ok) {
        return { ok: false, error: data?.error || 'Failed to create session from chat' };
      }
      return { ok: true, session: data };
    } catch (e: any) {
      return { ok: false, error: e?.message || 'Network error' };
    }
  },

  // Messages
  async getMessages(sessionId: string): Promise<AgentMessage[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/sessions/${sessionId}/messages`);
      if (!res.ok) return [];
      return await res.json();
    } catch (e) {
      console.error('Failed to get agent messages:', e);
      return [];
    }
  },

  async createMessage(
    sessionId: string,
    payload: {
      role: string;
      content?: string;
      thought?: string;
      tool_calls?: unknown[];
      tool_call_id?: string;
      status?: string;
    }
  ): Promise<{ ok: boolean; message?: AgentMessage; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/sessions/${sessionId}/messages`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      const data = await res.json();
      if (!res.ok) {
        return { ok: false, error: data?.error || 'Failed to create message' };
      }
      return { ok: true, message: data };
    } catch (e: any) {
      return { ok: false, error: e?.message || 'Network error' };
    }
  },

  // Prompt Skills
  async getPromptSkills(): Promise<PromptSkill[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/skills`);
      if (!res.ok) return [];
      return await res.json();
    } catch (e) {
      console.error('Failed to get prompt skills:', e);
      return [];
    }
  },

  async createPromptSkill(payload: {
    name: string;
    slug: string;
    description?: string;
    system_prompt: string;
    tags?: string[];
  }): Promise<{ ok: boolean; skill?: PromptSkill; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/skills`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      const data = await res.json();
      if (!res.ok) {
        return { ok: false, error: data?.error || 'Failed to create skill' };
      }
      return { ok: true, skill: data };
    } catch (e: any) {
      return { ok: false, error: e?.message || 'Network error' };
    }
  },

  async deletePromptSkill(id: string): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/skills/${id}/delete`, {
        method: 'POST',
      });
      return res.ok;
    } catch (e) {
      console.error('Failed to delete prompt skill:', e);
      return false;
    }
  },

  async getProjectSkills(projectId: string): Promise<PromptSkill[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/projects/${projectId}/skills`);
      if (!res.ok) return [];
      return await res.json();
    } catch (e) {
      console.error('Failed to get project skills:', e);
      return [];
    }
  },

  async setProjectSkills(projectId: string, skillIds: string[]): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/projects/${projectId}/skills`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ skill_ids: skillIds }),
      });
      return res.ok;
    } catch (e) {
      console.error('Failed to set project skills:', e);
      return false;
    }
  },

  // MCP Servers
  async getMcpServers(): Promise<McpServerConfig[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/mcp`);
      if (!res.ok) return [];
      return await res.json();
    } catch (e) {
      console.error('Failed to get MCP servers:', e);
      return [];
    }
  },
};
