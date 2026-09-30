import { Component, For, createEffect } from 'solid-js';
import { Brain, UserCheck, Cpu } from 'lucide-solid';
import { ANTIGRAVITY_PRESET_MODELS } from '../../services';

interface PersonaAiSectionProps {
  assistantName: string;
  setAssistantName: (val: string) => void;
  userCallsign: string;
  setUserCallsign: (val: string) => void;
  assistantGender: string;
  setAssistantGender: (val: string) => void;
  personalityRole: string;
  setPersonalityRole: (val: string) => void;
  llmProvider: string;
  setLlmProvider: (val: string) => void;
  llmBaseUrl: string;
  setLlmBaseUrl: (val: string) => void;
  llmModel: string;
  setLlmModel: (val: string) => void;
  llmApiKey: string;
  setLlmApiKey: (val: string) => void;
  ttsVoiceName?: string;
  setTtsVoiceName?: (val: string) => void;
}

export const PersonaAiSection: Component<PersonaAiSectionProps> = (props) => {
  let genderSelectRef: HTMLSelectElement | undefined;
  let roleSelectRef: HTMLSelectElement | undefined;
  let modelSelectRef: HTMLSelectElement | undefined;

  createEffect(() => {
    if (genderSelectRef && props.assistantGender) {
      genderSelectRef.value = props.assistantGender;
    }
  });

  createEffect(() => {
    if (roleSelectRef && props.personalityRole) {
      roleSelectRef.value = props.personalityRole;
    }
  });

  createEffect(() => {
    if (modelSelectRef && props.llmModel) {
      modelSelectRef.value = props.llmModel;
    }
  });

  return (
    <div class="space-y-5 animate-in fade-in duration-200">
      {/* 1. Persona & Identitas */}
      <div class="p-4 sm:p-5 bg-theme-surface border border-theme-border space-y-4">
        <h3 class="text-base sm:text-lg font-bold text-theme-cyan flex items-center gap-2">
          <UserCheck size={18} />
          1. Persona & Identitas Asisten
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Nama Asisten</label>
            <input
              type="text"
              value={props.assistantName}
              onInput={(e) => props.setAssistantName(e.currentTarget.value)}
              placeholder="Laras"
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            />
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Panggilan Pengguna</label>
            <input
              type="text"
              value={props.userCallsign}
              onInput={(e) => props.setUserCallsign(e.currentTarget.value)}
              placeholder="Mas"
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            />
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Gender Asisten</label>
            <select
              ref={genderSelectRef}
              value={props.assistantGender || 'female'}
              onChange={(e) => {
                const g = e.currentTarget.value;
                props.setAssistantGender(g);
                if (g === 'male' && props.ttsVoiceName?.startsWith('F')) {
                  props.setTtsVoiceName?.('M1');
                } else if (g === 'female' && props.ttsVoiceName?.startsWith('M')) {
                  props.setTtsVoiceName?.('F4');
                }
              }}
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            >
              <option value="female">Cewek (Female)</option>
              <option value="male">Cowok (Male)</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Peran Kepribadian</label>
            <select
              ref={roleSelectRef}
              value={props.personalityRole || 'partner'}
              onChange={(e) => props.setPersonalityRole(e.currentTarget.value)}
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            >
              <option value="partner">{props.assistantGender === 'male' ? 'Partner (Husbando Hangat)' : 'Partner (Waifu Manis)'}</option>
              <option value="assistant">{props.assistantGender === 'male' ? 'Assistant (Butler Profesional)' : 'Assistant (Sekretaris Cerdas)'}</option>
              <option value="friend">{props.assistantGender === 'male' ? 'Friend (Sahabat Bro Asyik)' : 'Friend (Sahabat Bestie Seru)'}</option>
              <option value="tsundere">Tsundere (Khas Anime)</option>
              <option value="maid">{props.assistantGender === 'male' ? 'Butler (Pelayan Santun)' : 'Maid (Pelayan Santun)'}</option>
            </select>
          </div>
        </div>
      </div>

      {/* 2. Pengaturan AI Brain (LLM Provider) */}
      <div class="p-4 sm:p-5 bg-theme-surface border border-theme-border space-y-4">
        <div class="flex items-center justify-between flex-wrap gap-2">
          <h3 class="text-base sm:text-lg font-bold text-theme-rose flex items-center gap-2">
            <Brain size={18} />
            2. AI Brain (LLM Provider)
          </h3>
          <span class="text-xs text-theme-text/50 font-mono">
            {props.llmProvider === 'local' ? '⚡ Ragasukma Local Engine' : '🌐 Custom AI Assistant'}
          </span>
        </div>

        {/* Pilihan Provider (2 Card Bersih: Lokal vs Custom) */}
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <button
            type="button"
            onClick={() => {
              props.setLlmProvider('local');
            }}
            class={`p-3.5 border text-left transition-all ${
              props.llmProvider === 'local'
                ? 'bg-theme-cyan/10 border-theme-cyan text-theme-text shadow-sm'
                : 'bg-theme-card border-theme-border text-theme-text/70 hover:border-theme-text/30'
            }`}
          >
            <div class="font-bold text-xs flex items-center gap-1.5 text-theme-cyan">
              <span>⚡ Lokal (Ragasukma Engine)</span>
            </div>
            <div class="text-xs text-theme-text/60 mt-1 leading-relaxed">
              API & routing otomatis dikelola sistem (saat ini ditenagai Antigravity pool, siap berkembang). Tetap bebas memilih model AI Assistant.
            </div>
          </button>

          <button
            type="button"
            onClick={() => {
              props.setLlmProvider('custom');
            }}
            class={`p-3.5 border text-left transition-all ${
              props.llmProvider === 'custom'
                ? 'bg-theme-rose/10 border-theme-rose text-theme-text shadow-sm'
                : 'bg-theme-card border-theme-border text-theme-text/70 hover:border-theme-text/30'
            }`}
          >
            <div class="font-bold text-xs flex items-center gap-1.5 text-theme-rose">
              <span>🌐 Custom / User AI Assistant</span>
            </div>
            <div class="text-xs text-theme-text/60 mt-1 leading-relaxed">
              Koneksikan ke server runtime sendiri (Ollama lokal, LM Studio, LocalAI) atau API eksternal (Groq, OpenRouter, OpenAI).
            </div>
          </button>
        </div>

        {/* Tampilan Konfigurasi Sesuai Provider */}
        {props.llmProvider === 'local' ? (
          <div class="p-4 bg-theme-bg/60 border border-theme-border/80 space-y-3">
            <div class="space-y-1.5">
              <label class="text-xs font-semibold text-theme-text/80 flex items-center justify-between">
                <span class="flex items-center gap-1.5 font-bold">
                  <Cpu size={14} class="text-theme-cyan" />
                  Pilih Model AI Assistant (Engine Lokal)
                </span>
                <span class="text-xs text-theme-cyan font-mono bg-theme-cyan/10 px-2 py-0.5 border border-theme-cyan/20 font-bold">
                  Otomatis Routing Internal
                </span>
              </label>
              <select
                ref={modelSelectRef}
                value={props.llmModel}
                onChange={(e) => props.setLlmModel(e.currentTarget.value)}
                class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm font-mono text-theme-text font-medium outline-none focus:border-theme-cyan transition-colors"
              >
                {ANTIGRAVITY_PRESET_MODELS.map((m) => (
                  <option value={m.id}>
                    {m.id}
                  </option>
                ))}
              </select>
            </div>
          </div>
        ) : (
          <div class="space-y-3">
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div class="space-y-1.5">
                <label class="text-xs font-semibold text-theme-text/80">LLM Base URL (Server Assistant)</label>
                <input
                  type="text"
                  value={props.llmBaseUrl}
                  onInput={(e) => props.setLlmBaseUrl(e.currentTarget.value)}
                  placeholder="http://localhost:11434/v1 atau https://api.openai.com/v1"
                  class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm font-mono text-theme-text outline-none focus:border-theme-cyan transition-colors"
                />
              </div>

              <div class="space-y-1.5">
                <label class="text-xs font-semibold text-theme-text/80">Model Name</label>
                <input
                  type="text"
                  value={props.llmModel}
                  onInput={(e) => props.setLlmModel(e.currentTarget.value)}
                  placeholder="qwen2.5:7b, llama3.2, mistral, gpt-4o, dll."
                  class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
                />
              </div>
            </div>

            <div class="space-y-1.5">
              <label class="text-xs font-semibold text-theme-text/80">LLM API Key (Opsional untuk Server Lokal)</label>
              <input
                type="password"
                value={props.llmApiKey}
                onInput={(e) => props.setLlmApiKey(e.currentTarget.value)}
                placeholder="Kosongkan jika Ollama/LM Studio lokal tanpa auth"
                class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm font-mono text-theme-text outline-none focus:border-theme-cyan transition-colors"
              />
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
