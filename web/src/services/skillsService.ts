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
  comment?: string;
}

export const skillsService = {
  async getSkills(): Promise<SkillItem[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/skills`);
      if (!res.ok) return [];
      return await res.json();
    } catch {
      return [];
    }
  },

  async saveSkill(skill: SkillItem): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/skills`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(skill),
      });
      return res.ok;
    } catch {
      return false;
    }
  },

  async deleteSkill(name: string): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/skills/delete`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name }),
      });
      return res.ok;
    } catch {
      return false;
    }
  },

  async runSkill(command: string): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/skills/run`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command }),
      });
      return res.ok;
    } catch {
      return false;
    }
  },

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
