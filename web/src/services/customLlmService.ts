import { API_BASE, fetchWithTimeout } from './base';

export interface CustomLlmAccount {
  prefix: string;
  name: string;
  base_url: string;
  api_key: string;
  enabled: boolean;
}

export const customLlmService = {
  async getAccounts(): Promise<CustomLlmAccount[]> {
    const response = await fetchWithTimeout(`${API_BASE}/api/llm/custom/accounts`);
    if (!response.ok) {
      const message = response.status === 404
        ? 'Custom provider API route not found (404). Restart or update the backend server.'
        : `Failed to load accounts (${response.status})`;
      throw new Error(message);
    }
    const data = await response.json();
    return (data || []).map((item: any) => ({
      prefix: String(item.prefix || '').trim().toLowerCase(),
      name: String(item.name || ''),
      base_url: String(item.base_url || item.baseUrl || ''),
      api_key: String(item.api_key || item.apiKey || ''),
      enabled: item.enabled ?? true,
    }));
  },
  async saveAccounts(accounts: CustomLlmAccount[]): Promise<void> {
    const response = await fetchWithTimeout(`${API_BASE}/api/llm/custom/accounts`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ accounts }),
    });
    if (!response.ok) {
      const data = await response.json().catch(() => ({}));
      throw new Error(data.error || `Failed to save accounts (${response.status})`);
    }
  },
  async deleteAccount(prefix: string): Promise<void> {
    const response = await fetchWithTimeout(`${API_BASE}/api/llm/custom/accounts/delete`, {
      method: 'POST', headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ prefix }),
    });
    if (!response.ok) {
      const data = await response.json().catch(() => ({}));
      throw new Error(data.error || `Gagal menghapus akun (${response.status})`);
    }
  },
};
