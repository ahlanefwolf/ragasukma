import { API_BASE, fetchWithTimeout } from './base';

export interface UserConfig {
  assistant_name: string;
  user_callsign: string;
  personality_role: string;
  assistant_gender: string;
  llm_provider: string;
  llm_base_url: string;
  llm_model: string;
  llm_api_key: string;
  tts_lang: string;
  stt_lang?: string;
  tts_voice_name?: string;
  tts_speed: number;
  tts_pitch?: number;
  avatar_type: string;
  always_on_top: boolean;
  voice_interruption: boolean;
  vad_threshold?: number;
  vad_noise_suppression?: boolean;
}

let cachedConfig: UserConfig | null = null;
let inflightConfigRequest: Promise<UserConfig | null> | null = null;

export const configService = {
  async getConfig(forceRefresh = false): Promise<UserConfig | null> {
    if (!forceRefresh && cachedConfig) {
      return cachedConfig;
    }
    if (inflightConfigRequest) {
      return inflightConfigRequest;
    }

    inflightConfigRequest = (async () => {
      try {
        const res = await fetchWithTimeout(`${API_BASE}/api/config`);
        if (!res.ok) return cachedConfig;
        const data = await res.json();
        cachedConfig = data;
        return data;
      } catch {
        return cachedConfig;
      } finally {
        inflightConfigRequest = null;
      }
    })();

    return inflightConfigRequest;
  },

  async saveConfig(config: UserConfig): Promise<boolean> {
    try {
      const res = await fetchWithTimeout(`${API_BASE}/api/config`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(config),
      });
      if (res.ok) {
        cachedConfig = config;
        try {
          window.dispatchEvent(new CustomEvent('ragasukma:config_updated', { detail: config }));
        } catch {}
      }
      return res.ok;
    } catch {
      return false;
    }
  },

  getCachedConfig(): UserConfig | null {
    return cachedConfig;
  },
};
