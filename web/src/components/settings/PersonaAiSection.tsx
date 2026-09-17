import { Component } from 'solid-js';
import { Brain, UserCheck } from 'lucide-solid';

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
  groqApiKey: string;
  setGroqApiKey: (val: string) => void;
}

const ANTIGRAVITY_MODELS = [
  { id: 'ag/gemini-3.8-flash-high', name: 'Gemini 3.8 Flash High (Sangat Cepat & Cerdas)', tag: 'Rekomendasi' },
  { id: 'ag/claude-3-5-sonnet', name: 'Claude 3.5 Sonnet (Coding & Tool Calling)', tag: 'Coding' },
  { id: 'ag/claude-3-7-sonnet', name: 'Claude 3.7 Sonnet (Thinking & Coding)', tag: 'Coding' },
  { id: 'ag/claude-sonnet-4-6', name: 'Claude Sonnet 4.6 (Penalaran Alami & Humanis)', tag: 'Smart' },
  { id: 'ag/claude-opus-4-6-thinking', name: 'Claude Opus 4.6 Thinking (Deep Reasoning)', tag: 'Thinking' },
  { id: 'ag/gemini-3.7-flash-high', name: 'Gemini 3.7 Flash High (Responsif)', tag: 'Populer' },
  { id: 'ag/gpt-4o', name: 'GPT-4o (High Speed Multimodal)', tag: '' },
];

export const PersonaAiSection: Component<PersonaAiSectionProps> = (props) => {
  return (
    <div class="space-y-5 animate-in fade-in duration-200">
      {/* 1. Persona & Identitas */}
      <div class="p-4 sm:p-5 rounded-2xl bg-theme-surface border border-theme-border space-y-4">
        <h3 class="text-base sm:text-lg font-carter text-theme-cyan flex items-center gap-2">
          <UserCheck size={18} />
          1. PERSONA & IDENTITAS ASISTEN
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 sm:gap-4">
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
            <label class="text-xs font-semibold text-theme-text/80">Gender Asisten</label>
            <select
              value={props.assistantGender || 'female'}
              onChange={(e) => props.setAssistantGender(e.currentTarget.value)}
              class="w-full bg-theme-card border border-theme-border rounded-xl px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            >
              <option value="female">Cewek (Female)</option>
              <option value="male">Cowok (Male)</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Peran Kepribadian</label>
            <select
              value={props.personalityRole}
              onChange={(e) => props.setPersonalityRole(e.currentTarget.value)}
              class="w-full bg-theme-card border border-theme-border rounded-xl px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
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
      <div class="p-4 sm:p-5 rounded-2xl bg-theme-surface border border-theme-border space-y-4">
        <div class="flex items-center justify-between flex-wrap gap-2">
          <h3 class="text-base sm:text-lg font-carter text-theme-rose flex items-center gap-2">
            <Brain size={18} />
            2. AI BRAIN (LLM PROVIDER)
          </h3>
          <span class="text-xs text-theme-text/50 font-mono">
            {props.llmProvider === 'antigravity' ? '⚡ Native Antigravity Pool' : '🌐 External Custom Gateway'}
          </span>
        </div>

        {/* Pilihan Provider (2 Card Bersih) */}
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
          <button
            type="button"
            onClick={() => {
              props.setLlmProvider('antigravity');
              if (!props.llmModel.startsWith('ag/')) {
                props.setLlmModel('ag/gemini-3.8-flash-high');
              }
            }}
            class={`p-3.5 rounded-xl border text-left transition-all ${
              props.llmProvider === 'antigravity'
                ? 'bg-theme-cyan/10 border-theme-cyan text-theme-text shadow-sm'
                : 'bg-theme-card border-theme-border text-theme-text/70 hover:border-theme-text/30'
            }`}
          >
            <div class="font-bold text-xs flex items-center gap-1.5 text-theme-cyan">
              <span>🚀 Antigravity (Google OAuth)</span>
            </div>
            <div class="text-[11px] text-theme-text/60 mt-1 leading-relaxed">
              Multi-akun gratis & tanpa API key langsung ke pool Google.
            </div>
          </button>

          <button
            type="button"
            onClick={() => {
              props.setLlmProvider('custom');
              if (props.llmModel.startsWith('ag/')) {
                props.setLlmModel('gpt-4o');
              }
            }}
            class={`p-3.5 rounded-xl border text-left transition-all ${
              props.llmProvider !== 'antigravity'
                ? 'bg-theme-rose/10 border-theme-rose text-theme-text shadow-sm'
                : 'bg-theme-card border-theme-border text-theme-text/70 hover:border-theme-text/30'
            }`}
          >
            <div class="font-bold text-xs flex items-center gap-1.5 text-theme-rose">
              <span>🌐 Custom / OpenAI-Compatible</span>
            </div>
            <div class="text-[11px] text-theme-text/60 mt-1 leading-relaxed">
              OpenAI, Groq, OpenRouter, DeepSeek, Ollama Lokal, dll.
            </div>
          </button>
        </div>

        {/* Tampilan Konfigurasi Sesuai Provider */}
        {props.llmProvider === 'antigravity' ? (
          <div class="p-3.5 rounded-xl bg-theme-bg/60 border border-theme-border/80 space-y-3">
            <div class="space-y-1.5">
              <label class="text-xs font-semibold text-theme-text/80 flex items-center justify-between">
                <span>Pilih Model Antigravity</span>
                <span class="text-[10px] text-theme-cyan font-normal">Terkoneksi langsung ke pool akun Google</span>
              </label>
              <select
                value={props.llmModel}
                onChange={(e) => props.setLlmModel(e.currentTarget.value)}
                class="w-full bg-theme-card border border-theme-border rounded-xl px-3.5 py-2.5 text-sm text-theme-text font-medium outline-none focus:border-theme-cyan transition-colors"
              >
                {ANTIGRAVITY_MODELS.map((m) => (
                  <option value={m.id}>
                    {m.name} {m.tag ? `[${m.tag}]` : ''}
                  </option>
                ))}
              </select>
            </div>
            <p class="text-[11px] text-theme-text/50">
              💡 Tidak perlu mengisi Base URL atau API Key. Kelola akun Google kamu di tab menu <b>Provider LLM &gt; Antigravity</b>.
            </p>
          </div>
        ) : (
          <div class="space-y-3">
            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div class="space-y-1.5">
                <label class="text-xs font-semibold text-theme-text/80">LLM Base URL</label>
                <input
                  type="text"
                  value={props.llmBaseUrl}
                  onInput={(e) => props.setLlmBaseUrl(e.currentTarget.value)}
                  placeholder="https://api.openai.com/v1"
                  class="w-full bg-theme-card border border-theme-border rounded-xl px-3.5 py-2.5 text-sm font-mono text-theme-text outline-none focus:border-theme-cyan transition-colors"
                />
              </div>

              <div class="space-y-1.5">
                <label class="text-xs font-semibold text-theme-text/80">Model Name</label>
                <input
                  type="text"
                  value={props.llmModel}
                  onInput={(e) => props.setLlmModel(e.currentTarget.value)}
                  placeholder="gpt-4o, claude-3-5-sonnet, llama3, dll."
                  class="w-full bg-theme-card border border-theme-border rounded-xl px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
                />
              </div>
            </div>

            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div class="space-y-1.5">
                <label class="text-xs font-semibold text-theme-text/80">LLM API Key</label>
                <input
                  type="password"
                  value={props.llmApiKey}
                  onInput={(e) => props.setLlmApiKey(e.currentTarget.value)}
                  placeholder="sk-..."
                  class="w-full bg-theme-card border border-theme-border rounded-xl px-3.5 py-2.5 text-sm font-mono text-theme-text outline-none focus:border-theme-cyan transition-colors"
                />
              </div>

              <div class="space-y-1.5">
                <label class="text-xs font-semibold text-theme-text/80">Groq API Key (Fast Whisper STT)</label>
                <input
                  type="password"
                  value={props.groqApiKey}
                  onInput={(e) => props.setGroqApiKey(e.currentTarget.value)}
                  placeholder="gsk_..."
                  class="w-full bg-theme-card border border-theme-border rounded-xl px-3.5 py-2.5 text-sm font-mono text-theme-text outline-none focus:border-theme-cyan transition-colors"
                />
              </div>
            </div>
            <p class="text-[11px] text-theme-text/50">
              💡 Mendukung semua endpoint OpenAI-compatible termasuk <b>Ollama lokal</b> (<code class="text-theme-cyan">http://127.0.0.1:11434/v1</code>), OpenRouter, Groq, dan DeepSeek. Kosongkan API Key jika menggunakan Ollama lokal.
            </p>
          </div>
        )}
      </div>
    </div>
  );
};
