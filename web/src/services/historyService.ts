import { API_BASE, fetchWithTimeout } from './base';

export interface ChatTurnItem {
  time: string;
  speaker: string;
  message: string;
}

export interface ChatHistoryDay {
  date: string;
  turns: ChatTurnItem[];
  total_turns?: number;
  page?: number;
  limit?: number;
  total_pages?: number;
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

  async getHistoryByDate(date?: string, page = 1, limit = 20): Promise<ChatHistoryDay> {
    try {
      const params = new URLSearchParams();
      if (date) params.append('date', date);
      params.append('page', page.toString());
      params.append('limit', limit.toString());

      const url = `${API_BASE}/api/history/view?${params.toString()}`;
      const res = await fetchWithTimeout(url);
      if (!res.ok) {
        return { date: date || '', turns: [], total_turns: 0, page: 1, limit, total_pages: 1 };
      }
      return await res.json();
    } catch (e) {
      console.error('Failed to get history content:', e);
      return { date: date || '', turns: [], total_turns: 0, page: 1, limit, total_pages: 1 };
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
