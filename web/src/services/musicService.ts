import { API_BASE, fetchWithTimeout } from './base';

export interface YtDlpInfo {
  available: boolean;
  version: string;
}

export interface YtTrackInfo {
  id: string;
  title: string;
  uploader: string;
  duration: number; // detik
  thumbnail?: string;
  filename: string;
}

export interface MusicListResponse {
  music_dir: string;
  tracks: YtTrackInfo[];
  volume?: number;
}

export interface MusicStatus {
  current_track?: YtTrackInfo;
  is_playing: boolean;
  is_paused: boolean;
  volume: number;
}

export const musicService = {
  async installYtDlp(): Promise<{ success: boolean; message: string; info?: YtDlpInfo }> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/music/install-ytdlp`, { method: 'POST' }, 120000);
      if (!res.ok) return { success: false, message: 'Failed to install' };
      return await res.json();
    } catch (e: any) {
      return { success: false, message: e?.message || 'Connection error' };
    }
  },

  async checkYtDlp(): Promise<YtDlpInfo> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/music/check`);
      if (!res.ok) return { available: false, version: '' };
      return await res.json();
    } catch {
      return { available: false, version: '' };
    }
  },

  async listMusic(): Promise<MusicListResponse> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/music/list`);
      if (!res.ok) return { music_dir: '', tracks: [] };
      const data = await res.json();
      if (Array.isArray(data)) {
        return { music_dir: '', tracks: data };
      }
      return {
        music_dir: data.music_dir || '',
        tracks: Array.isArray(data.tracks) ? data.tracks : [],
        volume: typeof data.volume === 'number' ? data.volume : undefined,
      };
    } catch {
      return { music_dir: '', tracks: [] };
    }
  },

  async downloadMusic(query: string): Promise<{ success: boolean; track?: YtTrackInfo; error?: string }> {
    try {
      const res = await fetchWithTimeout(
        `${API_BASE}/api/music/download`,
        {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ query }),
        },
        60000
      );
      if (!res.ok) return { success: false, error: 'Failed to download' };
      return await res.json();
    } catch (e: any) {
      return { success: false, error: e?.message || 'Connection error' };
    }
  },

  async controlMusic(action: string, payload?: string): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/music/control`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ action, payload }),
      });
      return res.ok;
    } catch {
      return false;
    }
  },

  getMusicUrl(filename: string): string {
    return `${API_BASE}/music/${encodeURIComponent(filename)}`;
  },

  async deleteMusic(filename: string): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/music/delete`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ filename }),
      });
      return res.ok;
    } catch {
      return false;
    }
  },
};
