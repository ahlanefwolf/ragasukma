import { API_BASE, fetchWithTimeout } from './base';
import { MusicStatus } from './musicService';

export interface AppStatePayload {
  state: string;
  emotion?: string;
  is_running: boolean;
  is_paused: boolean;
  voice_muted: boolean;
  mic_muted: boolean;
  active_session_token?: string;
  music_state?: MusicStatus;
}

export const menuService = {
  async sendCommand(cmd: string): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/cmd`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ cmd }),
      });
      return res.ok;
    } catch {
      return false;
    }
  },

  async getState(): Promise<AppStatePayload | null> {
    try {
      const res = await fetch(`${API_BASE}/api/state`);
      if (!res.ok) return null;
      return await res.json();
    } catch {
      return null;
    }
  },

  async setState(payload: AppStatePayload): Promise<boolean> {
    try {
      const res = await fetch(`${API_BASE}/api/state`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(payload),
      });
      return res.ok;
    } catch {
      return false;
    }
  },
};
