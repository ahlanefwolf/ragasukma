import { API_BASE, fetchWithTimeout } from './base';

export interface TunnelStatusResponse {
  status: 'stopped' | 'starting' | 'running';
  url: string | null;
  auth_key: string;
  error?: string;
}

export const remoteService = {
  async getTunnelStatus(): Promise<TunnelStatusResponse> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/remote/tunnel/status`);
      if (res.ok) {
        return await res.json();
      }
    } catch (e) {
      console.error('[RemoteService] getTunnelStatus error:', e);
    }
    return { status: 'stopped', url: null, auth_key: '' };
  },

  async startTunnel(): Promise<TunnelStatusResponse> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/remote/tunnel/start`, {
        method: 'POST',
      }, 15000);
      return await res.json();
    } catch (e) {
      console.error('[RemoteService] startTunnel error:', e);
      return { status: 'stopped', url: null, auth_key: '', error: String(e) };
    }
  },

  async stopTunnel(): Promise<TunnelStatusResponse> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/remote/tunnel/stop`, {
        method: 'POST',
      });
      return await res.json();
    } catch (e) {
      console.error('[RemoteService] stopTunnel error:', e);
      return { status: 'stopped', url: null, auth_key: '', error: String(e) };
    }
  },

  async sendText(text: string, authKey: string): Promise<{ success?: boolean; error?: string }> {
    try {
      const res = await fetch(`${API_BASE}/api/remote/text?key=${encodeURIComponent(authKey)}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ text }),
      });
      return await res.json();
    } catch (e) {
      return { error: String(e) };
    }
  },
};
