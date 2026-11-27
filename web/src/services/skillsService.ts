import { API_BASE, fetchWithTimeout } from './base';

export interface SkillItem {
  name: string;
  description: string;
  command: string;
}

export interface InstalledApp {
  name: string;
  exec: string;
  icon?: string;
}

export const skillsService = {
  async getSystemApps(): Promise<InstalledApp[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/system/apps`);
      if (!res.ok) return [];
      return await res.json();
    } catch {
      return [];
    }
  },

  async getExtensionPath(): Promise<{ folder: string; zip?: string; crx?: string } | null> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/system/extension-path`);
      if (!res.ok) return null;
      return await res.json();
    } catch {
      return null;
    }
  },
};
