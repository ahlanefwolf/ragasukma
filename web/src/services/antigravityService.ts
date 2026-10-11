import { API_BASE, fetchWithTimeout } from './base';

export interface AntigravityAccount {
  id: string;
  email?: string;
  project_id?: string;
  is_active: boolean;
  is_rate_limited: boolean;
  rate_limited_seconds_left: number;
}

interface ModelQuotaInfo {
  model_id: string;
  display_name: string;
  remaining_fraction: number;
  remaining_percentage: number;
  reset_time?: string;
}

export interface AntigravityQuotaResponse {
  status: string;
  account_id: string;
  email?: string;
  project_id?: string;
  plan?: string;
  models: ModelQuotaInfo[];
  weekly: ModelQuotaInfo[];
  error?: string;
}

export const antigravityService = {
  async getAccounts(): Promise<AntigravityAccount[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/llm/antigravity/accounts`);
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
      const res = await fetchWithTimeout(`${API_BASE}/api/llm/antigravity/oauth/url${openDirectly ? '?open=1' : ''}`);
      if (res.ok) {
        const data = await res.json();
        return data.auth_url || '';
      }
      return '';
    } catch {
      return '';
    }
  },

  async addAccount(refreshToken: string, email?: string, projectId?: string): Promise<{ status: string; id?: string; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/llm/antigravity/accounts`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          refresh_token: refreshToken,
          email: email || undefined,
          project_id: projectId || undefined,
        }),
      });
      return await res.json();
    } catch (err: any) {
      return { status: 'error', error: err?.message || 'Network error' };
    }
  },

  async deleteAccount(id: string): Promise<{ status: string; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/llm/antigravity/accounts/delete`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id }),
      });
      return await res.json();
    } catch (err: any) {
      return { status: 'error', error: err?.message || 'Network error' };
    }
  },

  async refreshAccount(id: string): Promise<{ status: string; project_id?: string; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/llm/antigravity/accounts/refresh`, {
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
      const res = await fetchWithTimeout(`${API_BASE}/api/llm/antigravity/accounts/toggle`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ id }),
      });
      return await res.json();
    } catch (err: any) {
      return { status: 'error', error: err?.message || 'Network error' };
    }
  },

  async getAccountQuota(id: string): Promise<AntigravityQuotaResponse> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/llm/antigravity/accounts/quota`, {
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
        models: [],
        weekly: [],
        error: `HTTP error ${res.status}`,
      };
    } catch (err: any) {
      return {
        status: 'error',
        account_id: id,
        models: [],
        weekly: [],
        error: err?.message || 'Failed to load quota',
      };
    }
  },

  async getDynamicModels(): Promise<Array<{ model_id: string; display_name: string; remaining_percentage: number }>> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/llm/antigravity/models`);
      if (res.ok) {
        const data = await res.json();
        if (data.status === 'success' && Array.isArray(data.models)) {
          return data.models;
        }
      }
      return [];
    } catch {
      return [];
    }
  },
};
