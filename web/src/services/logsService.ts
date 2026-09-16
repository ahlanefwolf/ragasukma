import { API_BASE, fetchWithTimeout } from './base';

export interface LogEntry {
  id: number;
  timestamp: string;
  level: string;
  message: string;
}

export const logsService = {
  async getLogs(): Promise<LogEntry[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/logs`, {}, 4000);
      if (!res.ok) return [];
      return await res.json();
    } catch {
      return [];
    }
  },

  async clearLogs(): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/logs/clear`, {
        method: 'POST',
      });
      return res.ok;
    } catch {
      return false;
    }
  },
};
