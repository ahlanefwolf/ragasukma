const API_BASE = 'http://127.0.0.1:42128';

async function fetchWithTimeout(url: string, options: RequestInit = {}, timeoutMs = 8000): Promise<Response> {
  const controller = new AbortController();
  const id = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const res = await fetch(url, { ...options, signal: controller.signal });
    clearTimeout(id);
    return res;
  } catch (err) {
    clearTimeout(id);
    throw err;
  }
}

export interface UserConfig {
  assistant_name: string;
  user_callsign: string;
  personality_role: string;
  stt_engine: string;
  groq_api_key: string;
  llm_base_url: string;
  llm_model: string;
  llm_api_key: string;
  voice_mode: string;
  mic_sensitivity: string;
  vad_silence_limit: string;
  noise_reduction_level: number;
  tts_engine: string;
  tts_lang: string;
  tts_speed: number;
  tts_eleven_labs_key: string;
  elevenlabs_voice_id: string;
  avatar_type: string;
  always_on_top?: boolean;
}

export interface SkillItem {
  name: string;
  description: string;
  command: string;
}

export interface InstalledApp {
  name: string;
  exec: string;
  icon?: string;
  comment?: string;
}

export interface PoseKeyframes {
  emotion?: string;
  speed: number;
  pose_a: Record<string, [number, number, number]>;
  pose_b: Record<string, [number, number, number]>;
}

export type PosesConfig = Record<string, PoseKeyframes>;

export interface LogEntry {
  id: number;
  timestamp: string;
  level: string;
  message: string;
}

export interface AppStatePayload {
  state: string;
  is_running: boolean;
  is_paused: boolean;
  voice_muted: boolean;
  mic_muted: boolean;
  active_session_token?: string;
}

export interface AssetInfo {
  vrm_exists: boolean;
  vrm_size_bytes: number;
  vrm_size_mb: number;
  vrm_path: string;
  images: string[];
}

export const api = {
  // Config
  async getConfig(): Promise<UserConfig | null> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/config`);
      if (!res.ok) return null;
      return await res.json();
    } catch {
      return null;
    }
  },

  async saveConfig(config: UserConfig): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/config`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      });
      return res.ok;
    } catch {
      return false;
    }
  },

  // Poses
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

  // Skills
  async getSkills(): Promise<SkillItem[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/skills`);
      if (!res.ok) return [];
      return await res.json();
    } catch {
      return [];
    }
  },

  async saveSkill(skill: SkillItem): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/skills`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(skill),
      });
      return res.ok;
    } catch {
      return false;
    }
  },

  async deleteSkill(name: string): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/skills/delete`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ name }),
      });
      return res.ok;
    } catch {
      return false;
    }
  },

  async runSkill(command: string): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/skills/run`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command }),
      });
      return res.ok;
    } catch {
      return false;
    }
  },

  async getSystemApps(): Promise<InstalledApp[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/system/apps`);
      if (!res.ok) return [];
      return await res.json();
    } catch {
      return [];
    }
  },

  // Live Tuner Preview (in-memory, tidak menyentuh poses.json di disk)
  async setTunerPreview(
    poses: PosesConfig,
    emotion?: string,
    state?: string
  ): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/tuner/preview`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ poses, emotion, state }),
      }, 4000);
      return res.ok;
    } catch {
      return false;
    }
  },

  // Live Tuner Sync (Gunakan standard fetch tanpa timeout abort agresif agar Firefox tidak trigger NS_BINDING_ABORT)
  async setTunerLive(
    active: boolean,
    bones: Record<string, [number, number, number]> = {},
    emotion?: string,
    state?: string
  ): Promise<boolean> {
    try {
      const res = await fetch(`${API_BASE}/api/tuner/live`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ active, bones, emotion, state }),
      });
      return res.ok;
    } catch {
      return false;
    }
  },

  // Logs
  async getLogs(): Promise<LogEntry[]> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/logs`, {}, 4000);
      if (!res.ok) return [];
      return await res.json();
    } catch {
      return [];
    }
  },

  async clearLogs(): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/logs/clear`, {
        method: 'POST',
      });
      return res.ok;
    } catch {
      return false;
    }
  },

  // State & Commands
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

  // Assets Upload & Management
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

  // Single Client Session Lock
  async sendHeartbeat(token: string): Promise<boolean> {
    try {
      const res = await fetch(`${API_BASE}/api/session/heartbeat`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token }),
      });
      if (!res.ok) return false;
      const data = await res.json();
      return !!data.is_active;
    } catch {
      return false;
    }
  },

  async takeoverSession(token: string): Promise<boolean> {
    try {
      const res = await fetch(`${API_BASE}/api/session/takeover`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token }),
      });
      return res.ok;
    } catch {
      return false;
    }
  },

  // Real-time EventSource (SSE) listener dengan callback koneksi offline / online
  subscribeState(
    onStateChange: (state: AppStatePayload) => void,
    onConnectionChange?: (connected: boolean) => void
  ): () => void {
    const es = new EventSource(`${API_BASE}/api/events`);
    es.onopen = () => {
      onConnectionChange?.(true);
    };
    es.onmessage = (event) => {
      try {
        const data: AppStatePayload = JSON.parse(event.data);
        onConnectionChange?.(true);
        onStateChange(data);
      } catch (err) {
        console.error('Failed to parse SSE state:', err);
      }
    };
    es.onerror = () => {
      onConnectionChange?.(false);
    };
    return () => {
      es.close();
    };
  }
};
