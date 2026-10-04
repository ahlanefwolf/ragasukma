import { API_BASE, fetchWithTimeout } from './base';

export interface ChatTurnItem {
  time: string;
  speaker: string;
  message: string;
}

export interface ChatHistoryDay {
  date: string;
  turns: ChatTurnItem[];
}

export const historyService = {
  async getHistoryDates(): Promise<string[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/history/dates`);
      if (!res.ok) return [];
      return await res.json();
    } catch (e) {
      console.error('Failed to get history dates:', e);
      return [];
    }
  },

  async getHistoryByDate(date?: string): Promise<ChatHistoryDay> {
    try {
      const url = date ? `${API_BASE}/api/history/view?date=${encodeURIComponent(date)}` : `${API_BASE}/api/history/view`;
      const res = await fetchWithTimeout(url);
      if (!res.ok) return { date: date || '', turns: [] };
      return await res.json();
    } catch (e) {
      console.error('Failed to get history content:', e);
      return { date: date || '', turns: [] };
    }
  },

  async deleteHistory(date: string): Promise<{ success: boolean; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/history/delete`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ date }),
      });
      return await res.json();
    } catch (e: any) {
      return { success: false, error: e?.message || 'Network error' };
    }
  },
};
