import { API_BASE, fetchWithTimeout } from './base';

export interface CloudflaredInfo {
  available: boolean;
  version: string;
  path?: string | null;
}

export interface TunnelStatusResponse {
  status: 'stopped' | 'starting' | 'running';
  url: string | null;
  auth_key: string;
  error?: string;
  cloudflared?: CloudflaredInfo;
}

export interface CloudflaredInstallResponse {
  success: boolean;
  message: string;
  info?: CloudflaredInfo;
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

  async checkCloudflared(): Promise<CloudflaredInfo> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/remote/tunnel/check`);
      if (res.ok) {
        return await res.json();
      }
    } catch (e) {
      console.error('[RemoteService] checkCloudflared error:', e);
    }
    return { available: false, version: '' };
  },

  async installCloudflared(): Promise<CloudflaredInstallResponse> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/remote/tunnel/install`, {
        method: 'POST',
      }, 60000);
      return await res.json();
    } catch (e) {
      console.error('[RemoteService] installCloudflared error:', e);
      return { success: false, message: String(e) };
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
