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
let tunerWs: WebSocket | null = null;
let tunerWsConnecting = false;

function getTunerWs(): WebSocket | null {
  if (typeof window === 'undefined') return null;
  if (tunerWs && tunerWs.readyState === WebSocket.OPEN) {
    return tunerWs;
  }
  if (tunerWsConnecting) return null;

  try {
    tunerWsConnecting = true;
    const wsProto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const wsHost = API_BASE.replace(/^https?:\/\//, '');
    const wsUrl = `${wsProto}//${wsHost}/api/ws`;
    const socket = new WebSocket(wsUrl);

    socket.onopen = () => {
      tunerWsConnecting = false;
      tunerWs = socket;
    };
    socket.onerror = () => {
      tunerWsConnecting = false;
      tunerWs = null;
    };
    socket.onclose = () => {
      tunerWsConnecting = false;
      tunerWs = null;
    };
    return socket;
  } catch {
    tunerWsConnecting = false;
    return null;
  }
}

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
    // 1. Ultra-fast WebSocket path (sub-millisecond 60 FPS slider sync)
    const ws = getTunerWs();
    if (ws && ws.readyState === WebSocket.OPEN) {
      try {
        ws.send(JSON.stringify({
          type: 'tuner_live',
          active,
          bones,
          emotion,
          state,
        }));
        return true;
      } catch {}
    }

    // 2. HTTP Fallback
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
        return true;
      }
      return false;
    }
  },
};
