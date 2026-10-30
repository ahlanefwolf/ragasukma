import { API_BASE, fetchWithTimeout } from './base';

// Workspace & Agent Service Interface

export interface PromptSkill {
  id: string;
  name: string;
  slug: string;
  description: string;
  system_prompt: string;
  tags: string;
  file_path?: string;
  is_active: number;
  created_at: number;
  updated_at: number;
}

export interface SaveMcpServerPayload {
  id?: string;
  name: string;
  transport?: 'stdio' | 'sse' | 'http';
  command?: string;
  args?: string[];
  env?: Record<string, string>;
  url?: string;
  raw_json?: any;
  is_enabled?: boolean;
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
  total_tokens_input?: number;
  total_tokens_output?: number;
}

export interface WorkspaceFileItem {
  relative_path: string;
  name: string;
  is_dir: boolean;
  size: number;
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
  hidden?: boolean; // Flag for internal/system messages that should not appear in chat UI
}

export interface Goal {
  id: string;
  project_id: string;
  session_id?: string;
  parent_goal_id?: string;
  title: string;
  demand: string;
  success_criteria: string; // JSON Array
  constraints: string;      // JSON Array
  status: 'initialized' | 'discovery' | 'action' | 'verification' | 'completed' | 'failed';
  conclusion?: string;
  created_at: number;
  updated_at: number;
}

export interface EvidenceCacheItem {
  id: string;
  project_id: string;
  file_path: string;
  file_version: string;
  start_line?: number;
  end_line?: number;
  symbol_name?: string;
  summary: string;
  raw_snippet?: string;
  is_stale: number;
  created_at: number;
  updated_at: number;
}

export interface GoalVerificationItem {
  id: string;
  goal_id: string;
  iteration: number;
  test_command: string;
  test_output: string;
  is_passed: number;
  feedback?: string;
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
  async getMessages(
    sessionId: string,
    opts?: { limit?: number; offset?: number; before_sequence_num?: number }
  ): Promise<AgentMessage[]> {
    try {
      const params = new URLSearchParams();
      if (opts?.limit !== undefined) params.set('limit', String(opts.limit));
      if (opts?.offset !== undefined) params.set('offset', String(opts.offset));
      if (opts?.before_sequence_num !== undefined) params.set('before_sequence_num', String(opts.before_sequence_num));
      const qs = params.toString();
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/sessions/${sessionId}/messages${qs ? `?${qs}` : ''}`);
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

  // Agent Skills (Auto-detected from project folders & global .agents)
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

  async getProjectMcpServers(projectId: string): Promise<string[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/projects/${projectId}/mcp`);
      if (!res.ok) return [];
      const data = await res.json();
      return Array.isArray(data.server_ids) ? data.server_ids : [];
    } catch (e) {
      console.error('Failed to get project MCP servers:', e);
      return [];
    }
  },

  async setProjectMcpServers(projectId: string, serverIds: string[]): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/projects/${projectId}/mcp`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ server_ids: serverIds }),
      });
      return res.ok;
    } catch (e) {
      console.error('Failed to set project MCP servers:', e);
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

  async saveMcpServer(server: SaveMcpServerPayload): Promise<{ ok: boolean; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/mcp`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(server),
      });
      if (!res.ok) {
        const data = await res.json().catch(() => ({}));
        return { ok: false, error: data.error || `HTTP ${res.status}` };
      }
      return { ok: true };
    } catch (e: any) {
      return { ok: false, error: e?.message || 'Failed to save MCP server' };
    }
  },

  async deleteMcpServer(id: string): Promise<{ ok: boolean; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/mcp/${encodeURIComponent(id)}/delete`, {
        method: 'POST',
      });
      if (!res.ok) {
        const data = await res.json().catch(() => ({}));
        return { ok: false, error: data.error || `HTTP ${res.status}` };
      }
      return { ok: true };
    } catch (e: any) {
      return { ok: false, error: e?.message || 'Failed to delete MCP server' };
    }
  },

  async testMcpServer(server: SaveMcpServerPayload): Promise<{ ok: boolean; tools_count?: number; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/mcp/test`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(server),
      });
      const data = await res.json().catch(() => ({}));
      if (!res.ok || data.error) {
        return { ok: false, error: data.error || `HTTP ${res.status}` };
      }
      return { ok: true, tools_count: data.tools ? data.tools.length : 0 };
    } catch (e: any) {
      return { ok: false, error: e?.message || 'Test connection failed' };
    }
  },


  // Goals & Evidence
  async getProjectGoals(projectId: string): Promise<Goal[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/projects/${projectId}/goals`);
      if (!res.ok) return [];
      return await res.json();
    } catch (e) {
      console.error('Failed to get project goals:', e);
      return [];
    }
  },

  async createGoal(
    projectId: string,
    payload: {
      id?: string;
      session_id?: string;
      parent_goal_id?: string;
      title: string;
      demand: string;
      success_criteria?: string[];
      constraints?: string[];
      reuse_keywords?: string[];
    }
  ): Promise<{ ok: boolean; goal?: Goal; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/projects/${projectId}/goals`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      const data = await res.json();
      if (!res.ok) {
        return { ok: false, error: data?.error || 'Failed to create goal' };
      }
      return { ok: true, goal: data };
    } catch (e: any) {
      return { ok: false, error: e?.message || 'Network error' };
    }
  },

  async getGoalDetails(goalId: string): Promise<Goal | null> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/goals/${goalId}`);
      if (!res.ok) return null;
      return await res.json();
    } catch (e) {
      console.error('Failed to get goal details:', e);
      return null;
    }
  },

  async getGoalEvidence(goalId: string): Promise<EvidenceCacheItem[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/goals/${goalId}/evidence`);
      if (!res.ok) return [];
      return await res.json();
    } catch (e) {
      console.error('Failed to get goal evidence:', e);
      return [];
    }
  },

  async getGoalVerifications(goalId: string): Promise<GoalVerificationItem[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/goals/${goalId}/verifications`);
      if (!res.ok) return [];
      return await res.json();
    } catch (e) {
      console.error('Failed to get goal verifications:', e);
      return [];
    }
  },

  async concludeGoal(
    goalId: string,
    conclusion: string
  ): Promise<{ ok: boolean; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/agent/goals/${goalId}/conclude`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ conclusion }),
      });
      const data = await res.json();
      if (!res.ok) {
        return { ok: false, error: data?.error || 'Failed to conclude goal' };
      }
      return { ok: true };
    } catch (e: any) {
      return { ok: false, error: e?.message || 'Network error' };
    }
  },
};
