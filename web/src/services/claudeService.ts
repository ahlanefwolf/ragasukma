import { API_BASE, fetchWithTimeout } from './base';

export interface ClaudeAccount {
  id: string;
  email?: string;
  org_id?: string;
  plan_type?: string;
  is_active: boolean;
  is_rate_limited: boolean;
  rate_limited_seconds_left: number;
}

export interface ClaudeQuotaWindow {
  used: number;
  total: number;
  remaining: number;
  reset_at?: string;
}

export interface ClaudeQuotaResponse {
  status: string;
  account_id: string;
  email?: string;
  org_id?: string;
  plan?: string;
  limit_reached: boolean;
  session_quota?: ClaudeQuotaWindow;
  weekly_quota?: ClaudeQuotaWindow;
  reset_credits?: number;
  error?: string;
}

export interface ClaudeModelItem {
  id: string;
  model_id: string;
  display_name: string;
  provider: string;
}

export const claudeService = {
  async getAccounts(): Promise<ClaudeAccount[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/llm/claude/accounts`);
      if (res.ok) {
        return await res.json();
      }
      return [];
    } catch {
      return [];
    }
  },

  async getAuthUrl(openDirectly = false): Promise<string> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/llm/claude/oauth/url${openDirectly ? '?open=1' : ''}`);
      if (res.ok) {
        const data = await res.json();
        return data.auth_url || '';
      }
      return '';
    } catch {
      return '';
    }
  },

  async addAccount(refreshToken: string, email?: string, orgId?: string): Promise<{ status: string; id?: string; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/llm/claude/accounts`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          refreshToken,
          email: email || undefined,
          org_id: orgId || undefined,
        }),
      });
      return await res.json();
    } catch (err: any) {
      return { status: 'error', error: err?.message || 'Network error' };
    }
  },

  async deleteAccount(id: string): Promise<{ status: string; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/llm/claude/accounts/delete`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id }),
      });
      return await res.json();
    } catch (err: any) {
      return { status: 'error', error: err?.message || 'Network error' };
    }
  },

  async refreshAccount(id: string): Promise<{ status: string; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/llm/claude/accounts/refresh`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id }),
      });
      return await res.json();
    } catch (err: any) {
      return { status: 'error', error: err?.message || 'Network error' };
    }
  },

  async toggleAccount(id: string): Promise<{ status: string; is_active?: boolean; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/llm/claude/accounts/toggle`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id }),
      });
      return await res.json();
    } catch (err: any) {
      return { status: 'error', error: err?.message || 'Network error' };
    }
  },

  async getAccountQuota(id: string): Promise<ClaudeQuotaResponse> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/llm/claude/accounts/quota`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id }),
      });
      if (res.ok) {
        return await res.json();
      }
      return {
        status: 'error',
        account_id: id,
        limit_reached: false,
        error: `HTTP error ${res.status}`,
      };
    } catch (err: any) {
      return {
        status: 'error',
        account_id: id,
        limit_reached: false,
        error: err?.message || 'Failed to load quota',
      };
    }
  },

  async getModels(): Promise<ClaudeModelItem[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/llm/claude/models`);
      if (res.ok) {
        return await res.json();
      }
      return [];
    } catch {
      return [];
    }
  },
};
