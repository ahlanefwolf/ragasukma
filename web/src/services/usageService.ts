import { API_BASE, fetchWithTimeout } from './base';

export interface LlmUsageRecord {
  id: string;
  timestamp: number;
  date_str: string;
  time_str: string;
  provider: string;
  model: string;
  input_tokens: number;
  cached_tokens: number;
  output_tokens: number;
  total_tokens: number;
  duration_ms: number;
  status: string;
}

export interface LlmUsageSummary {
  total_requests: number;
  total_input_tokens: number;
  total_cached_tokens: number;
  total_output_tokens: number;
  total_tokens: number;
  cache_hit_rate_pct: number;
  estimated_saved_tokens: number;
}

export interface LlmUsagePaginatedResponse {
  items: LlmUsageRecord[];
  total: number;
  page: number;
  limit: number;
  total_pages: number;
}

export const usageService = {
  async getUsageSummary(provider?: string, date?: string): Promise<LlmUsageSummary> {
    try {
      const params = new URLSearchParams();
      if (provider && provider !== 'all') params.append('provider', provider);
      if (date && date !== 'all') params.append('date', date);
      const url = `${API_BASE}/api/llm/usage/summary?${params.toString()}`;
      const res = await fetchWithTimeout(url);
      if (!res.ok) {
        return {
          total_requests: 0,
          total_input_tokens: 0,
          total_cached_tokens: 0,
          total_output_tokens: 0,
          total_tokens: 0,
          cache_hit_rate_pct: 0,
          estimated_saved_tokens: 0,
        };
      }
      return await res.json();
    } catch (e) {
      console.error('Failed to get usage summary:', e);
      return {
        total_requests: 0,
        total_input_tokens: 0,
        total_cached_tokens: 0,
        total_output_tokens: 0,
        total_tokens: 0,
        cache_hit_rate_pct: 0,
        estimated_saved_tokens: 0,
      };
    }
  },

  async getUsageRecords(
    provider?: string,
    date?: string,
    page = 1,
    limit = 25
  ): Promise<LlmUsagePaginatedResponse> {
    try {
      const params = new URLSearchParams();
      if (provider && provider !== 'all') params.append('provider', provider);
      if (date && date !== 'all') params.append('date', date);
      params.append('page', page.toString());
      params.append('limit', limit.toString());
      const url = `${API_BASE}/api/llm/usage/records?${params.toString()}`;
      const res = await fetchWithTimeout(url);
      if (!res.ok) {
        return { items: [], total: 0, page: 1, limit, total_pages: 1 };
      }
      return await res.json();
    } catch (e) {
      console.error('Failed to get usage records:', e);
      return { items: [], total: 0, page: 1, limit, total_pages: 1 };
    }
  },

  async clearUsage(): Promise<{ success: boolean; error?: string }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/llm/usage/clear`, {
        method: 'POST',
      });
      return await res.json();
    } catch (e: any) {
      return { success: false, error: e?.message || 'Network error' };
    }
  },
};
