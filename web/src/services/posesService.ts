import { API_BASE, fetchWithTimeout } from './base';

export interface SinglePose {
  bones: Record<string, [number, number, number]>;
}

export interface PoseKeyframes {
  emotion?: string;
  speed: number;
  poses?: SinglePose[];
}

export type PosesConfig = Record<string, PoseKeyframes>;

let liveTunerAbortController: AbortController | null = null;

export const posesService = {
  async getPoses(): Promise<PosesConfig | null> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/poses`);
      if (!res.ok) return null;
      return await res.json();
    } catch {
      return null;
    }
  },

  async savePoses(poses: PosesConfig): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/poses`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(poses),
      });
      return res.ok;
    } catch {
      return false;
    }
  },

  async setTunerPreview(
    poses: PosesConfig,
    emotion?: string,
    state?: string
  ): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/tuner/preview`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ poses, emotion, state }),
      }, 4000);
      return res.ok;
    } catch {
      return false;
    }
  },

  async setTunerLive(
    active: boolean,
    bones: Record<string, [number, number, number]> = {},
    emotion?: string,
    state?: string
  ): Promise<boolean> {
    // Batalkan request slider in-flight sebelumnya jika masih berjalan
    if (liveTunerAbortController) {
      liveTunerAbortController.abort();
    }
    liveTunerAbortController = new AbortController();

    try {
      const res = await fetch(`${API_BASE}/api/tuner/live`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ active, bones, emotion, state }),
        signal: liveTunerAbortController.signal,
      });
      return res.ok;
    } catch (err: any) {
      if (err?.name === 'AbortError') {
        // Normal saat dibatalkan oleh slider input baru
        return true;
      }
      return false;
    }
  },
};
