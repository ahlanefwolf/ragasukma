import { Component, createSignal } from 'solid-js';
import { 
  Activity, 
  Wind, 
  Hand, 
  RotateCw, 
  Save, 
  Sliders,
  Check
} from 'lucide-solid';

export const PoseTab: Component = () => {
  const [breathIntensity, setBreathIntensity] = createSignal(1.0);
  const [idleSpeed, setIdleSpeed] = createSignal(1.2);
  const [speakingSpeed, setSpeakingSpeed] = createSignal(0.8);
  const [fingerCurlRatio, setFingerCurlRatio] = createSignal(0.85);
  const [savedSuccess, setSavedSuccess] = createSignal(false);

  const handleSavePoses = () => {
    setSavedSuccess(true);
    setTimeout(() => setSavedSuccess(false), 2500);
  };

  return (
    <div class="space-y-6">
      {/* Header Banner */}
      <div class="rounded-2xl bg-gradient-to-r from-[#282832] via-[#202028] to-[#1E1E24] p-6 border border-[#3B3B48]">
        <span class="px-2.5 py-1 rounded-full text-xs font-bold uppercase tracking-wider bg-[#91AE6E]/20 text-[#91AE6E] border border-[#91AE6E]/30">
          Inverse Kinematics & Pose Interpolator
        </span>
        <h2 class="text-3xl font-bangers tracking-wide text-[#F2F2F2] mt-2">
          INSPEKTOR GESTUR & POSES.JSON
        </h2>
        <p class="text-sm text-[#F2F2F2]/70 mt-1 max-w-xl">
          Atur parameter kecepatan interpolasi sinusoidal (1:1 Three.js formula), frekuensi nafas halus pada dada (chest), serta kelengkungan jari tangan.
        </p>
      </div>

      {/* Main Sliders */}
      <div class="p-5 rounded-2xl bg-[#1E1E24] border border-[#3B3B48] space-y-6">
        <h3 class="text-lg font-bangers text-[#91AE6E] flex items-center gap-2">
          <Activity size={18} />
          TUNING ANIMASI REAL-TIME
        </h3>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
          {/* Breath Wave */}
          <div class="space-y-2">
            <div class="flex items-center justify-between text-xs font-semibold">
              <span class="flex items-center gap-1.5 text-[#F2F2F2]/80">
                <Wind size={14} class="text-[#91AE6E]" />
                Intensitas Ayunan Nafas (Chest/Spine)
              </span>
              <span class="text-[#91AE6E] font-mono">{breathIntensity()}x</span>
            </div>
            <input 
              type="range" 
              min="0.0" 
              max="2.5" 
              step="0.1"
              value={breathIntensity()}
              onInput={(e) => setBreathIntensity(parseFloat(e.currentTarget.value))}
              class="w-full h-2 bg-[#282832] rounded-lg appearance-none cursor-pointer accent-[#91AE6E]"
            />
          </div>

          {/* Finger Curl */}
          <div class="space-y-2">
            <div class="flex items-center justify-between text-xs font-semibold">
              <span class="flex items-center gap-1.5 text-[#F2F2F2]/80">
                <Hand size={14} class="text-[#D96868]" />
                Kelengkungan Jari Alami (Finger Curl)
              </span>
              <span class="text-[#D96868] font-mono">{fingerCurlRatio()}x</span>
            </div>
            <input 
              type="range" 
              min="0.2" 
              max="1.5" 
              step="0.05"
              value={fingerCurlRatio()}
              onInput={(e) => setFingerCurlRatio(parseFloat(e.currentTarget.value))}
              class="w-full h-2 bg-[#282832] rounded-lg appearance-none cursor-pointer accent-[#D96868]"
            />
          </div>

          {/* Idle Cycle Speed */}
          <div class="space-y-2">
            <div class="flex items-center justify-between text-xs font-semibold">
              <span class="flex items-center gap-1.5 text-[#F2F2F2]/80">
                <RotateCw size={14} class="text-[#689D4B]" />
                Kecepatan Siklus Idle (poses.json speed)
              </span>
              <span class="text-[#689D4B] font-mono">{idleSpeed()}</span>
            </div>
            <input 
              type="range" 
              min="0.3" 
              max="3.0" 
              step="0.1"
              value={idleSpeed()}
              onInput={(e) => setIdleSpeed(parseFloat(e.currentTarget.value))}
              class="w-full h-2 bg-[#282832] rounded-lg appearance-none cursor-pointer accent-[#689D4B]"
            />
          </div>

          {/* Speaking Cycle Speed */}
          <div class="space-y-2">
            <div class="flex items-center justify-between text-xs font-semibold">
              <span class="flex items-center gap-1.5 text-[#F2F2F2]/80">
                <RotateCw size={14} class="text-[#D96868]" />
                Kecepatan Gestur Bicara (Speaking Speed)
              </span>
              <span class="text-[#D96868] font-mono">{speakingSpeed()}</span>
            </div>
            <input 
              type="range" 
              min="0.3" 
              max="3.0" 
              step="0.1"
              value={speakingSpeed()}
              onInput={(e) => setSpeakingSpeed(parseFloat(e.currentTarget.value))}
              class="w-full h-2 bg-[#282832] rounded-lg appearance-none cursor-pointer accent-[#D96868]"
            />
          </div>
        </div>

        {/* Save Config Button */}
        <div class="pt-3 border-t border-[#3B3B48]/60 flex justify-end">
          <button
            onClick={handleSavePoses}
            class="px-6 py-2.5 rounded-xl font-bold text-sm bg-[#D96868] hover:bg-[#c45a5a] text-white shadow-md anime-shadow-red flex items-center gap-2 transition-all"
          >
            {savedSuccess() ? (
              <>
                <Check size={16} class="stroke-[3]" />
                <span>Tersimpan ke poses.json!</span>
              </>
            ) : (
              <>
                <Save size={16} />
                <span>Simpan Konfigurasi Pose</span>
              </>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};
