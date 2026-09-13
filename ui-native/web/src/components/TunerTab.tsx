import { Component, createSignal } from 'solid-js';
import { 
  Sliders, 
  Copy, 
  RotateCw, 
  RotateCcw, 
  Play, 
  RefreshCw, 
  Save, 
  Trash2,
  Smile,
  Activity,
  User
} from 'lucide-solid';

interface TunerTabProps {
  showToast: (msg: string) => void;
}

export const TunerTab: Component<TunerTabProps> = (props) => {
  const [activePoseTab, setActivePoseTab] = createSignal<'pose_a' | 'pose_b'>('pose_a');
  const [selectedState, setSelectedState] = createSignal('idle');
  const [selectedEmotion, setSelectedEmotion] = createSignal('shy');
  const [speed, setSpeed] = createSignal(1.2);
  const [activeBoneCategory, setActiveBoneCategory] = createSignal('spine');

  // Bone rotasi Euler dummy per kategori untuk tuning
  const [boneValues, setBoneValues] = createSignal<Record<string, [number, number, number]>>({
    head: [-0.08, -0.12, -0.36],
    chest: [0.0, 0.0, 0.0],
    spine: [0.29, 0.0, 0.0],
    hips: [0.0, 0.0, 0.0],
    leftUpperArm: [-0.26, -1.09, -0.69],
    leftLowerArm: [-0.56, -2.59, -0.12],
    leftHand: [-0.07, 0.21, 1.07],
    rightUpperArm: [-0.27, -0.15, 0.59],
    rightLowerArm: [0.15, 1.24, 0.62],
    rightHand: [0.68, 0.68, -0.53],
    leftUpperLeg: [-1.54, -0.05, 0.05],
    leftLowerLeg: [1.90, 0.0, 0.0],
    leftFoot: [0.47, 0.05, 0.0],
    rightUpperLeg: [-1.55, 0.05, -0.05],
    rightLowerLeg: [1.90, 0.0, 0.0],
    rightFoot: [0.47, -0.05, 0.0],
  });

  const boneCategories = [
    { id: 'spine', label: 'Kepala & Badan', bones: ['head', 'chest', 'spine', 'hips'] },
    { id: 'arm_left', label: 'Lengan Kiri', bones: ['leftUpperArm', 'leftLowerArm', 'leftHand'] },
    { id: 'arm_right', label: 'Lengan Kanan', bones: ['rightUpperArm', 'rightLowerArm', 'rightHand'] },
    { id: 'leg_left', label: 'Kaki Kiri', bones: ['leftUpperLeg', 'leftLowerLeg', 'leftFoot'] },
    { id: 'leg_right', label: 'Kaki Kanan', bones: ['rightUpperLeg', 'rightLowerLeg', 'rightFoot'] },
  ];

  const updateBoneAxis = (bone: string, axis: 0 | 1 | 2, val: number) => {
    const cur = { ...boneValues() };
    const arr = [...(cur[bone] || [0, 0, 0])] as [number, number, number];
    arr[axis] = val;
    cur[bone] = arr;
    setBoneValues(cur);
  };

  const handleCopy = (from: 'a' | 'b', to: 'a' | 'b') => {
    props.showToast(`Menyalin semua rotasi Pose ${from.toUpperCase()} ke Pose ${to.toUpperCase()}`);
  };

  const handleMirror = (from: 'a' | 'b', to: 'a' | 'b') => {
    props.showToast(`Mirroring simetri kiri-kanan Pose ${from.toUpperCase()} ke Pose ${to.toUpperCase()}`);
  };

  const handleSave = () => {
    props.showToast(`Berhasil menyimpan keyframe ${selectedState()} ke poses.json!`);
  };

  const handleResetAll = () => {
    props.showToast('Semua nilai pose di-reset ke default T-Pose');
  };

  return (
    <div class="space-y-6">
      {/* Top Tuner Bar */}
      <div class="p-5 rounded-2xl bg-[#1E1E24] border border-[#3B3B48] space-y-4">
        <div class="flex flex-wrap items-center justify-between gap-4">
          <div class="flex items-center gap-2">
            <h2 class="text-2xl font-bangers tracking-wide text-[#F2F2F2]">
              POSE & MOTION TUNER
            </h2>
            <span class="px-2 py-0.5 rounded text-[11px] font-mono bg-[#282832] text-[#91AE6E] border border-[#3B3B48]">
              poses.json
            </span>
          </div>

          {/* Helper Action Buttons */}
          <div class="flex flex-wrap items-center gap-2">
            <button 
              onClick={() => handleCopy('a', 'b')}
              class="px-2.5 py-1.5 rounded-lg text-xs font-semibold bg-[#282832] hover:bg-[#323240] text-[#F2F2F2] border border-[#3B3B48]"
              title="Copy Pose A ke Pose B"
            >
              A→B
            </button>
            <button 
              onClick={() => handleCopy('b', 'a')}
              class="px-2.5 py-1.5 rounded-lg text-xs font-semibold bg-[#282832] hover:bg-[#323240] text-[#F2F2F2] border border-[#3B3B48]"
              title="Copy Pose B ke Pose A"
            >
              B→A
            </button>
            <button 
              onClick={() => handleMirror('a', 'b')}
              class="px-2.5 py-1.5 rounded-lg text-xs font-semibold bg-[#282832] hover:bg-[#323240] text-[#F2F2F2] border border-[#3B3B48]"
              title="Mirror simetri A ke B"
            >
              Mirror A→B
            </button>
            <button 
              onClick={handleResetAll}
              class="px-2.5 py-1.5 rounded-lg text-xs font-semibold bg-[#282832] hover:bg-[#D96868]/20 text-[#D96868] border border-[#3B3B48]"
            >
              Reset All
            </button>
            <button 
              onClick={handleSave}
              class="px-4 py-1.5 rounded-lg text-xs font-bold bg-[#D96868] hover:bg-[#c45a5a] text-white shadow anime-shadow-red flex items-center gap-1.5"
            >
              <Save size={14} />
              <span>Simpan</span>
            </button>
          </div>
        </div>

        {/* State, Emotion, Speed & Keyframe Selector Bar */}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 pt-3 border-t border-[#3B3B48]/60">
          {/* State Select */}
          <div class="space-y-1">
            <label class="text-xs font-semibold text-[#F2F2F2]/70">Target State</label>
            <select
              value={selectedState()}
              onChange={(e) => setSelectedState(e.currentTarget.value)}
              class="w-full bg-[#282832] border border-[#3B3B48] rounded-xl px-3 py-2 text-xs text-[#F2F2F2] outline-none"
            >
              <option value="idle">Idle (Standby)</option>
              <option value="listening">Listening (Mendengarkan)</option>
              <option value="thinking">Thinking (Berpikir)</option>
              <option value="speaking">Speaking (Bicara)</option>
            </select>
          </div>

          {/* Emotion Select */}
          <div class="space-y-1">
            <label class="text-xs font-semibold text-[#F2F2F2]/70">Ekspresi Emosi</label>
            <select
              value={selectedEmotion()}
              onChange={(e) => setSelectedEmotion(e.currentTarget.value)}
              class="w-full bg-[#282832] border border-[#3B3B48] rounded-xl px-3 py-2 text-xs text-[#F2F2F2] outline-none"
            >
              <option value="neutral">Neutral</option>
              <option value="happy">Happy</option>
              <option value="shy">Shy</option>
              <option value="surprised">Surprised</option>
              <option value="thinking">Thinking</option>
              <option value="sad">Sad</option>
              <option value="excited">Excited</option>
              <option value="confused">Confused</option>
            </select>
          </div>

          {/* Speed Slider */}
          <div class="space-y-1">
            <div class="flex justify-between text-xs font-semibold">
              <span class="text-[#F2F2F2]/70">Speed Siklus</span>
              <span class="text-[#91AE6E] font-mono">{speed()}x</span>
            </div>
            <input 
              type="range" 
              min="0.2" 
              max="4.0" 
              step="0.1"
              value={speed()}
              onInput={(e) => setSpeed(parseFloat(e.currentTarget.value))}
              class="w-full h-2 bg-[#282832] rounded-lg appearance-none cursor-pointer accent-[#91AE6E] mt-2"
            />
          </div>

          {/* Pose A / B Toggle */}
          <div class="space-y-1">
            <label class="text-xs font-semibold text-[#F2F2F2]/70">Keyframe</label>
            <div class="grid grid-cols-2 gap-1 bg-[#282832] p-1 rounded-xl border border-[#3B3B48]">
              <button
                onClick={() => setActivePoseTab('pose_a')}
                class={`py-1 rounded-lg text-xs font-bold transition-all ${
                  activePoseTab() === 'pose_a'
                    ? 'bg-[#D96868] text-white shadow'
                    : 'text-[#F2F2F2]/60 hover:text-[#F2F2F2]'
                }`}
              >
                Pose A
              </button>
              <button
                onClick={() => setActivePoseTab('pose_b')}
                class={`py-1 rounded-lg text-xs font-bold transition-all ${
                  activePoseTab() === 'pose_b'
                    ? 'bg-[#689D4B] text-white shadow'
                    : 'text-[#F2F2F2]/60 hover:text-[#F2F2F2]'
                }`}
              >
                Pose B
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Category Tabs */}
      <div class="flex flex-wrap gap-2">
        {boneCategories.map((cat) => {
          const isCurrent = () => activeBoneCategory() === cat.id;

          return (
            <button
              onClick={() => setActiveBoneCategory(cat.id)}
              class={`px-4 py-2 rounded-xl text-xs font-bold transition-all ${
                isCurrent()
                  ? 'bg-[#689D4B] text-white shadow anime-shadow-green'
                  : 'bg-[#1E1E24] text-[#F2F2F2]/70 border border-[#3B3B48] hover:bg-[#282832]'
              }`}
            >
              {cat.label}
            </button>
          );
        })}
      </div>

      {/* Euler Sliders Grid for Active Category */}
      <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
        {boneCategories
          .find((c) => c.id === activeBoneCategory())
          ?.bones.map((boneName) => {
            const vals = () => boneValues()[boneName] || [0, 0, 0];

            return (
              <div class="p-5 rounded-2xl bg-[#1E1E24] border border-[#3B3B48] space-y-3">
                <div class="flex items-center justify-between">
                  <h4 class="font-bold text-sm text-[#F2F2F2] font-mono capitalize">
                    {boneName}
                  </h4>
                  <span class="text-[11px] font-mono text-[#91AE6E]">
                    [{vals()[0].toFixed(2)}, {vals()[1].toFixed(2)}, {vals()[2].toFixed(2)}]
                  </span>
                </div>

                {/* X Axis */}
                <div class="space-y-1">
                  <div class="flex justify-between text-[11px] text-[#F2F2F2]/60">
                    <span class="text-[#D96868] font-bold">Rotasi X (Pitch)</span>
                    <span class="font-mono">{vals()[0].toFixed(2)} rad</span>
                  </div>
                  <input
                    type="range"
                    min="-3.14"
                    max="3.14"
                    step="0.01"
                    value={vals()[0]}
                    onInput={(e) => updateBoneAxis(boneName, 0, parseFloat(e.currentTarget.value))}
                    class="w-full h-1.5 bg-[#282832] rounded-lg appearance-none cursor-pointer accent-[#D96868]"
                  />
                </div>

                {/* Y Axis */}
                <div class="space-y-1">
                  <div class="flex justify-between text-[11px] text-[#F2F2F2]/60">
                    <span class="text-[#91AE6E] font-bold">Rotasi Y (Yaw)</span>
                    <span class="font-mono">{vals()[1].toFixed(2)} rad</span>
                  </div>
                  <input
                    type="range"
                    min="-3.14"
                    max="3.14"
                    step="0.01"
                    value={vals()[1]}
                    onInput={(e) => updateBoneAxis(boneName, 1, parseFloat(e.currentTarget.value))}
                    class="w-full h-1.5 bg-[#282832] rounded-lg appearance-none cursor-pointer accent-[#91AE6E]"
                  />
                </div>

                {/* Z Axis */}
                <div class="space-y-1">
                  <div class="flex justify-between text-[11px] text-[#F2F2F2]/60">
                    <span class="text-[#689D4B] font-bold">Rotasi Z (Roll)</span>
                    <span class="font-mono">{vals()[2].toFixed(2)} rad</span>
                  </div>
                  <input
                    type="range"
                    min="-3.14"
                    max="3.14"
                    step="0.01"
                    value={vals()[2]}
                    onInput={(e) => updateBoneAxis(boneName, 2, parseFloat(e.currentTarget.value))}
                    class="w-full h-1.5 bg-[#282832] rounded-lg appearance-none cursor-pointer accent-[#689D4B]"
                  />
                </div>
              </div>
            );
          })}
      </div>
    </div>
  );
};
