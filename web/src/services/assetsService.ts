import { API_BASE, fetchWithTimeout } from './base';

export interface AssetInfo {
  vrm_exists: boolean;
  vrm_size_bytes: number;
  vrm_size_mb: number;
  vrm_path: string;
  images: string[];
}

let cachedAssetInfo: AssetInfo | null = null;
let inflightAssetRequest: Promise<AssetInfo | null> | null = null;

export const assetsService = {
  async getAssetsInfo(forceRefresh = false): Promise<AssetInfo | null> {
    if (!forceRefresh && cachedAssetInfo) {
      return cachedAssetInfo;
    }
    if (inflightAssetRequest) {
      return inflightAssetRequest;
    }

    inflightAssetRequest = (async () => {
      try {
        const res = await fetchWithTimeout(`${API_BASE}/api/assets/info`);
        if (!res.ok) return cachedAssetInfo;
        const data = await res.json();
        cachedAssetInfo = data;
        return data;
      } catch {
        return cachedAssetInfo;
      } finally {
        inflightAssetRequest = null;
      }
    })();

    return inflightAssetRequest;
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
        cachedAssetInfo = null; // Invalidate cache so data refreshes
        const data = await res.json().catch(() => ({}));
        return { ok: true, message: data.message };
      }
      return { ok: false, message: 'Failed to upload VRM file' };
    } catch (e: any) {
      return { ok: false, message: e?.message || 'Connection error' };
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
      if (res.ok) {
        cachedAssetInfo = null; // Invalidate cache
      }
      return res.ok;
    } catch {
      return false;
    }
  },
};
