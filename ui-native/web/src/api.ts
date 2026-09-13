const API_BASE = 'http://127.0.0.1:4242';

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
  enable_terminal_log: boolean;
}

export interface SkillItem {
  name: string;
  description: string;
  command: string;
}

export interface PoseKeyframes {
  emotion?: string;
  speed: number;
  pose_a: Record<string, [number, number, number]>;
  pose_b: Record<string, [number, number, number]>;
}

export type PosesConfig = Record<string, PoseKeyframes>;

export interface AppStatePayload {
  state: string;
  is_paused: boolean;
  voice_muted: boolean;
  mic_muted: boolean;
}

export const api = {
  // Config
  async getConfig(): Promise<UserConfig | null> {
    try {
      const res = await fetch(`${API_BASE}/api/config`);
      if (!res.ok) return null;
      return await res.json();
    } catch {
      return null;
    }
  },

  async saveConfig(config: UserConfig): Promise<boolean> {
    try {
      const res = await fetch(`${API_BASE}/api/config`, {
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
      const res = await fetch(`${API_BASE}/api/poses`);
      if (!res.ok) return null;
      return await res.json();
    } catch {
      return null;
    }
  },

  async savePoses(poses: PosesConfig): Promise<boolean> {
    try {
      const res = await fetch(`${API_BASE}/api/poses`, {
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
      const res = await fetch(`${API_BASE}/api/skills`);
      if (!res.ok) return [];
      return await res.json();
    } catch {
      return [];
    }
  },

  async saveSkill(skill: SkillItem): Promise<boolean> {
    try {
      const res = await fetch(`${API_BASE}/api/skills`, {
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
      const res = await fetch(`${API_BASE}/api/skills/delete`, {
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
      const res = await fetch(`${API_BASE}/api/skills/run`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ command }),
      });
      return res.ok;
    } catch {
      return false;
    }
  },

  // Live Tuner Sync
  async setTunerLive(active: boolean, bones: Record<string, [number, number, number]> = {}): Promise<boolean> {
    try {
      const res = await fetch(`${API_BASE}/api/tuner/live`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ active, bones }),
      });
      return res.ok;
    } catch {
      return false;
    }
  },

  // State
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
