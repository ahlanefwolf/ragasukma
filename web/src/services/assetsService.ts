import { API_BASE, fetchWithTimeout } from './base';

export interface AssetInfo {
  vrm_exists: boolean;
  vrm_size_bytes: number;
  vrm_size_mb: number;
  vrm_path: string;
  images: string[];
}

export const assetsService = {
  async getAssetsInfo(): Promise<AssetInfo | null> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/assets/info`);
      if (!res.ok) return null;
      return await res.json();
    } catch {
      return null;
    }
  },

  async uploadVrm(file: File): Promise<{ ok: boolean; message?: string }> {
    try {
      const buffer = await file.arrayBuffer();
      const res = await fetch(`${API_BASE}/api/upload/vrm`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/octet-stream',
        },
        body: buffer,
      });
      if (res.ok) {
        const data = await res.json().catch(() => ({}));
        return { ok: true, message: data.message };
      }
      return { ok: false, message: 'Gagal mengunggah file VRM' };
    } catch (e: any) {
      return { ok: false, message: e?.message || 'Koneksi error' };
    }
  },

  async uploadAvatarImage(slot: string, file: File): Promise<boolean> {
    try {
      const buffer = await file.arrayBuffer();
      const res = await fetch(`${API_BASE}/api/upload/avatar?slot=${encodeURIComponent(slot)}`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/octet-stream',
        },
        body: buffer,
      });
      return res.ok;
    } catch {
      return false;
    }
  },
};
