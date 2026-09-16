import { Component } from 'solid-js';
import { Brain, UserCheck } from 'lucide-solid';

interface PersonaAiSectionProps {
  assistantName: string;
  setAssistantName: (val: string) => void;
  userCallsign: string;
  setUserCallsign: (val: string) => void;
  personalityRole: string;
  setPersonalityRole: (val: string) => void;
  llmBaseUrl: string;
  setLlmBaseUrl: (val: string) => void;
  llmModel: string;
  setLlmModel: (val: string) => void;
  llmApiKey: string;
  setLlmApiKey: (val: string) => void;
  groqApiKey: string;
  setGroqApiKey: (val: string) => void;
}

export const PersonaAiSection: Component<PersonaAiSectionProps> = (props) => {
  return (
    <div class="space-y-5 animate-in fade-in duration-200">
      {/* 1. Persona & Identitas */}
      <div class="p-4 sm:p-5 rounded-2xl bg-theme-surface border border-theme-border space-y-4">
        <h3 class="text-base sm:text-lg font-carter text-theme-cyan flex items-center gap-2">
          <UserCheck size={18} />
          1. PERSONA & IDENTITAS ASISTEN
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 sm:gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Nama Asisten</label>
            <input
              type="text"
              value={props.assistantName}
              onInput={(e) => props.setAssistantName(e.currentTarget.value)}
              placeholder="Laras"
              class="w-full bg-theme-card border border-theme-border rounded-xl px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            />
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Panggilan Pengguna</label>
            <input
              type="text"
              value={props.userCallsign}
              onInput={(e) => props.setUserCallsign(e.currentTarget.value)}
              placeholder="Mas"
              class="w-full bg-theme-card border border-theme-border rounded-xl px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            />
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Peran Kepribadian</label>
            <select
              value={props.personalityRole}
              onChange={(e) => props.setPersonalityRole(e.currentTarget.value)}
              class="w-full bg-theme-card border border-theme-border rounded-xl px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            >
              <option value="partner">Partner (Hangat & Santai)</option>
              <option value="assistant">Assistant (Formal & Rapi)</option>
              <option value="tsundere">Tsundere (Khas Anime)</option>
              <option value="maid">Maid (Pelayan Santun)</option>
            </select>
          </div>
        </div>
      </div>

      {/* 2. Pengaturan AI Brain (LLM Provider) */}
      <div class="p-4 sm:p-5 rounded-2xl bg-theme-surface border border-theme-border space-y-4">
        <h3 class="text-base sm:text-lg font-carter text-theme-rose flex items-center gap-2">
          <Brain size={18} />
          2. AI BRAIN (LLM PROVIDER)
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">LLM Base URL (OpenAI Compatible)</label>
            <input
              type="text"
              value={props.llmBaseUrl}
              onInput={(e) => props.setLlmBaseUrl(e.currentTarget.value)}
              placeholder="http://localhost:20128/v1"
              class="w-full bg-theme-card border border-theme-border rounded-xl px-3.5 py-2.5 text-sm font-mono text-theme-text outline-none focus:border-theme-cyan transition-colors"
            />
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Model Name</label>
            <input
              type="text"
              value={props.llmModel}
              onInput={(e) => props.setLlmModel(e.currentTarget.value)}
              placeholder="ag/gemini-3.8-flash-low"
              class="w-full bg-theme-card border border-theme-border rounded-xl px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            />
          </div>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">LLM API Key (Opsional / Gateway)</label>
            <input
              type="password"
              value={props.llmApiKey}
              onInput={(e) => props.setLlmApiKey(e.currentTarget.value)}
              placeholder="sk-..."
              class="w-full bg-theme-card border border-theme-border rounded-xl px-3.5 py-2.5 text-sm font-mono text-theme-text outline-none focus:border-theme-cyan transition-colors"
            />
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Groq API Key (Fast Whisper STT / LLM)</label>
            <input
              type="password"
              value={props.groqApiKey}
              onInput={(e) => props.setGroqApiKey(e.currentTarget.value)}
              placeholder="gsk_..."
              class="w-full bg-theme-card border border-theme-border rounded-xl px-3.5 py-2.5 text-sm font-mono text-theme-text outline-none focus:border-theme-cyan transition-colors"
            />
          </div>
        </div>
      </div>
    </div>
  );
};
