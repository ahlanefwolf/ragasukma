import { API_BASE, fetchWithTimeout } from './base';

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
  voice_interruption?: boolean;
}

export const configService = {
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
};
