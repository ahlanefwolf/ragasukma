import { API_BASE, fetchWithTimeout } from './base';

export interface ChatTurnItem {
  time: string;
  speaker: string;
  message: string;
  id?: string;
  timestamp?: number;
  emotion?: string;
}

export interface ChatHistoryDay {
  date: string;
  turns: ChatTurnItem[];
  total_turns?: number;
  page?: number;
  limit?: number;
  total_pages?: number;
  has_more?: boolean;
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

  async getHistoryByDate(date?: string, page = 1, limit = 20, order?: 'asc' | 'desc'): Promise<ChatHistoryDay> {
    try {
      const params = new URLSearchParams();
      if (date) params.append('date', date);
      params.append('page', page.toString());
      params.append('limit', limit.toString());
      if (order) params.append('order', order);

      const url = `${API_BASE}/api/history/view?${params.toString()}`;
      const res = await fetchWithTimeout(url);
      if (!res.ok) {
        return { date: date || '', turns: [], total_turns: 0, page: 1, limit, total_pages: 1, has_more: false };
      }
      return await res.json();
    } catch (e) {
      console.error('Failed to get history content:', e);
      return { date: date || '', turns: [], total_turns: 0, page: 1, limit, total_pages: 1, has_more: false };
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

  subscribeChatEvents(onMessage: (item: ChatTurnItem) => void): () => void {
    let authKey = '';
    try {
      const params = new URLSearchParams(window.location.search);
      authKey = params.get('key') || '';
      if (!authKey && window.location.hash.includes('?')) {
        const hashParams = new URLSearchParams(window.location.hash.split('?')[1]);
        authKey = hashParams.get('key') || '';
      }
    } catch {}

    const url = `${API_BASE}/api/chat/events${authKey ? `?key=${encodeURIComponent(authKey)}` : ''}`;
    let es: EventSource | null = null;
    let isClosed = false;
    let reconnectTimer: any = null;

    const connect = () => {
      if (isClosed) return;
      if (es) {
        es.close();
        es = null;
      }

      try {
        es = new EventSource(url);

        es.onmessage = (e) => {
          try {
            if (!e.data || e.data.trim() === '') return;
            const data = JSON.parse(e.data);
            if (data.type === 'chat_turn' && data.item) {
              onMessage(data.item);
            }
          } catch (err) {
            console.error('Failed to parse chat SSE event:', err);
          }
        };

        es.addEventListener('ping', () => {
          // Heartbeat keep-alive from server
        });

        es.onerror = (e) => {
          console.warn('Chat SSE connection error:', e);
          if (es) {
            es.close();
            es = null;
          }
          if (!isClosed && !reconnectTimer) {
            reconnectTimer = setTimeout(() => {
              reconnectTimer = null;
              connect();
            }, 2500);
          }
        };
      } catch (err) {
        console.error('Failed to create chat EventSource:', err);
      }
    };

    connect();

    const handleVisibilityChange = () => {
      if (document.visibilityState === 'visible' && !isClosed) {
        if (!es || es.readyState === EventSource.CLOSED) {
          connect();
        }
      }
    };
    document.addEventListener('visibilitychange', handleVisibilityChange);

    return () => {
      isClosed = true;
      if (reconnectTimer) {
        clearTimeout(reconnectTimer);
        reconnectTimer = null;
      }
      document.removeEventListener('visibilitychange', handleVisibilityChange);
      if (es) {
        es.close();
        es = null;
      }
    };
  },
};
