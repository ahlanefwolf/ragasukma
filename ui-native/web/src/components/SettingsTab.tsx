import { Component, createSignal } from 'solid-js';
import { 
  Settings, 
  Server, 
  Monitor, 
  Key, 
  Save, 
  Check, 
  Power,
  ShieldAlert
} from 'lucide-solid';

export const SettingsTab: Component = () => {
  const [serverPort, setServerPort] = createSignal('4242');
  const [autoLaunch, setAutoLaunch] = createSignal(true);
  const [alwaysOnTop, setAlwaysOnTop] = createSignal(true);
  const [geminiApiKey, setGeminiApiKey] = createSignal('');
  const [savedSuccess, setSavedSuccess] = createSignal(false);

  const handleSave = () => {
    setSavedSuccess(true);
    setTimeout(() => setSavedSuccess(false), 2500);
  };

  return (
    <div class="space-y-6">
      {/* Header Banner */}
      <div class="rounded-2xl bg-gradient-to-r from-[#282832] via-[#202028] to-[#1E1E24] p-6 border border-[#3B3B48]">
        <span class="px-2.5 py-1 rounded-full text-xs font-bold uppercase tracking-wider bg-[#689D4B]/20 text-[#91AE6E] border border-[#689D4B]/30">
          System & Overlay Preferences
        </span>
        <h2 class="text-3xl font-bangers tracking-wide text-[#F2F2F2] mt-2">
          PENGATURAN SISTEM & DESKTOP
        </h2>
        <p class="text-sm text-[#F2F2F2]/70 mt-1 max-w-xl">
          Konfigurasi konektivitas server internal, integrasi Windows DWM Desktop, dan API Key AI LLM backend.
        </p>
      </div>

      {/* Overlay Window Options */}
      <div class="p-5 rounded-2xl bg-[#1E1E24] border border-[#3B3B48] space-y-4">
        <h3 class="text-lg font-bangers text-[#91AE6E] flex items-center gap-2">
          <Monitor size={18} />
          WINDOW OVERLAY DESKTOP (WGPU)
        </h3>

        <div class="space-y-3">
          {/* Always on Top */}
          <div class="flex items-center justify-between p-3.5 rounded-xl bg-[#282832] border border-[#3B3B48]">
            <div>
              <h4 class="font-bold text-sm text-[#F2F2F2]">Always On Top (Selalu di Depan)</h4>
              <p class="text-xs text-[#F2F2F2]/60 mt-0.5">Avatar tetap melayang di atas jendela aplikasi dan game lain.</p>
            </div>
            <button
              onClick={() => setAlwaysOnTop(!alwaysOnTop())}
              class={`w-12 h-6 rounded-full transition-colors relative ${
                alwaysOnTop() ? 'bg-[#689D4B]' : 'bg-[#181820]'
              }`}
            >
              <span 
                class={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${
                  alwaysOnTop() ? 'translate-x-6' : 'translate-x-0'
                }`}
              />
            </button>
          </div>

          {/* Auto Launch */}
          <div class="flex items-center justify-between p-3.5 rounded-xl bg-[#282832] border border-[#3B3B48]">
            <div>
              <h4 class="font-bold text-sm text-[#F2F2F2]">Jalankan Saat Windows Startup</h4>
              <p class="text-xs text-[#F2F2F2]/60 mt-0.5">Otomatis membuka Ragasukma Overlay saat komputer menyala.</p>
            </div>
            <button
              onClick={() => setAutoLaunch(!autoLaunch())}
              class={`w-12 h-6 rounded-full transition-colors relative ${
                autoLaunch() ? 'bg-[#689D4B]' : 'bg-[#181820]'
              }`}
            >
              <span 
                class={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${
                  autoLaunch() ? 'translate-x-6' : 'translate-x-0'
                }`}
              />
            </button>
          </div>
        </div>
      </div>

      {/* Network & API Keys */}
      <div class="p-5 rounded-2xl bg-[#1E1E24] border border-[#3B3B48] space-y-4">
        <h3 class="text-lg font-bangers text-[#D96868] flex items-center gap-2">
          <Key size={18} />
          JARINGAN & API BACKEND
        </h3>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#F2F2F2]/80">Port Localhost Server</label>
            <input 
              type="text" 
              value={serverPort()}
              onInput={(e) => setServerPort(e.currentTarget.value)}
              class="w-full bg-[#282832] border border-[#3B3B48] rounded-xl px-4 py-2.5 text-sm text-[#F2F2F2] outline-none focus:border-[#D96868] transition-all"
            />
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-[#F2F2F2]/80">Google Gemini API Key</label>
            <input 
              type="password" 
              value={geminiApiKey()}
              onInput={(e) => setGeminiApiKey(e.currentTarget.value)}
              placeholder="AIzaSy..."
              class="w-full bg-[#282832] border border-[#3B3B48] rounded-xl px-4 py-2.5 text-sm text-[#F2F2F2] outline-none focus:border-[#D96868] transition-all"
            />
          </div>
        </div>

        <div class="pt-3 border-t border-[#3B3B48]/60 flex justify-end">
          <button
            onClick={handleSave}
            class="px-6 py-2.5 rounded-xl font-bold text-sm bg-[#689D4B] hover:bg-[#5b8c40] text-white shadow-md anime-shadow-green flex items-center gap-2 transition-all"
          >
            {savedSuccess() ? (
              <>
                <Check size={16} class="stroke-[3]" />
                <span>Pengaturan Disimpan!</span>
              </>
            ) : (
              <>
                <Save size={16} />
                <span>Simpan Perubahan</span>
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};
