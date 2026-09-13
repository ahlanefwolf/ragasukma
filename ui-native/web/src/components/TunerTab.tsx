import { Component, createSignal, onMount, onCleanup } from 'solid-js';
import { 
  Save, 
  RotateCcw,
  Play,
  Square,
  RefreshCw,
  Sliders,
  Sparkles
} from 'lucide-solid';
import { api, PosesConfig } from '../api';
import { BONE_CATEGORIES, BONE_LABELS, MIRROR_MAP } from './tunerBones';

interface TunerTabProps {
  showToast: (msg: string) => void;
}

export const TunerTab: Component<TunerTabProps> = (props) => {
  const [activePoseTab, setActivePoseTab] = createSignal<'pose_a' | 'pose_b'>('pose_a');
  const [selectedState, setSelectedState] = createSignal('idle');
  const [selectedEmotion, setSelectedEmotion] = createSignal('shy');
  const [speed, setSpeed] = createSignal(1.2);
  const [activeBoneCategory, setActiveBoneCategory] = createSignal('body');
  const [posesData, setPosesData] = createSignal<PosesConfig>({});
  const [isSaving, setIsSaving] = createSignal(false);
  const [isPreviewing, setIsPreviewing] = createSignal(false);

  let previewInterval: number | null = null;
  let previewTime = 0;

  const getCurrentKeyframeBones = (): Record<string, [number, number, number]> => {
    const st = posesData()[selectedState()];
    if (!st) return {};
    return activePoseTab() === 'pose_a' ? st.pose_a || {} : st.pose_b || {};
  };

  const syncLiveTuner = (active = true) => {
    if (isPreviewing()) return;
    const bones = getCurrentKeyframeBones();
    api.setTunerLive(active, bones);
  };

  const loadPoses = async () => {
    const data = await api.getPoses();
    if (data && Object.keys(data).length > 0) {
      setPosesData(data);
      const curState = data[selectedState()];
      if (curState) {
        setSelectedEmotion(curState.emotion || 'neutral');
        setSpeed(curState.speed || 1.2);
      }
      syncLiveTuner(true);
    }
  };

  onMount(() => {
    loadPoses();
  });

  onCleanup(() => {
    if (previewInterval) clearInterval(previewInterval);
    api.setTunerLive(false);
  });

  const handleStateChange = (st: string) => {
    setSelectedState(st);
    const cur = posesData()[st];
    if (cur) {
      setSelectedEmotion(cur.emotion || 'neutral');
      setSpeed(cur.speed || 1.2);
    }
    syncLiveTuner(true);
  };

  const handleKeyframeTabChange = (tab: 'pose_a' | 'pose_b') => {
    setActivePoseTab(tab);
    syncLiveTuner(true);
  };

  const getBoneRotation = (boneName: string): [number, number, number] => {
    const st = posesData()[selectedState()];
    if (!st) return [0, 0, 0];
    const keyframe = activePoseTab() === 'pose_a' ? st.pose_a : st.pose_b;
    return keyframe?.[boneName] || [0, 0, 0];
  };

  const updateBoneAxis = (bone: string, axis: 0 | 1 | 2, val: number) => {
    const clamped = Math.max(-3.14, Math.min(3.14, isNaN(val) ? 0 : val));
    const all = { ...posesData() };
    const stKey = selectedState();
    let st = all[stKey];
    if (!st) {
      st = { emotion: selectedEmotion(), speed: speed(), pose_a: {}, pose_b: {} };
      all[stKey] = st;
    }

    const keyframeName = activePoseTab();
    const curPose = { ...(st[keyframeName] || {}) };
    const arr = [...(curPose[bone] || [0, 0, 0])] as [number, number, number];
    arr[axis] = clamped;
    curPose[bone] = arr;
    st[keyframeName] = curPose;
    setPosesData(all);

    api.setTunerLive(true, curPose);
  };

  const resetSingleBone = (bone: string) => {
    const all = { ...posesData() };
    const st = all[selectedState()];
    if (!st) return;

    const keyframeName = activePoseTab();
    const curPose = { ...(st[keyframeName] || {}) };
    curPose[bone] = [0, 0, 0];
    st[keyframeName] = curPose;
    setPosesData(all);

    api.setTunerLive(true, curPose);
    props.showToast(`Bone "${BONE_LABELS[bone] || bone}" di-reset ke [0, 0, 0]`);
  };

  const resetSingleAxis = (bone: string, axis: 0 | 1 | 2) => {
    updateBoneAxis(bone, axis, 0);
  };

  const handleCopy = (from: 'a' | 'b', to: 'a' | 'b') => {
    const all = { ...posesData() };
    const st = all[selectedState()];
    if (!st) return;

    const source = from === 'a' ? st.pose_a : st.pose_b;
    if (to === 'a') {
      st.pose_a = JSON.parse(JSON.stringify(source || {}));
    } else {
      st.pose_b = JSON.parse(JSON.stringify(source || {}));
    }
    setPosesData(all);
    syncLiveTuner(true);
    props.showToast(`Menyalin semua rotasi Pose ${from.toUpperCase()} ke Pose ${to.toUpperCase()}`);
  };

  const handleMirror = (from: 'a' | 'b', to: 'a' | 'b') => {
    const all = { ...posesData() };
    const st = all[selectedState()];
    if (!st) return;

    const source = from === 'a' ? st.pose_a : st.pose_b;
    const mirrored: Record<string, [number, number, number]> = {};

    for (const [bone, rot] of Object.entries(source || {})) {
      const targetBone = MIRROR_MAP[bone] || bone;
      mirrored[targetBone] = [
        parseFloat(rot[0].toFixed(3)),
        parseFloat((-rot[1]).toFixed(3)),
        parseFloat((-rot[2]).toFixed(3))
      ];
    }

    if (to === 'a') {
      st.pose_a = mirrored;
    } else {
      st.pose_b = mirrored;
    }
    setPosesData(all);
    syncLiveTuner(true);
    props.showToast(`Mirroring simetri kiri-kanan Pose ${from.toUpperCase()} ke Pose ${to.toUpperCase()}`);
  };

  const togglePreview = () => {
    const next = !isPreviewing();
    setIsPreviewing(next);

    if (next) {
      previewTime = 0;
      props.showToast('Preview loop animasi A ⇄ B aktif');
      previewInterval = window.setInterval(() => {
        previewTime += 0.03 * (speed() || 1.2) * 2.2;
        const t = (Math.sin(previewTime) + 1.0) / 2.0;

        const st = posesData()[selectedState()];
        if (!st) return;

        const poseA = st.pose_a || {};
        const poseB = st.pose_b || {};
        const interpolated: Record<string, [number, number, number]> = {};

        const keys = new Set([...Object.keys(poseA), ...Object.keys(poseB)]);
        for (const k of keys) {
          const a = poseA[k] || [0, 0, 0];
          const b = poseB[k] || [0, 0, 0];
          interpolated[k] = [
            a[0] + (b[0] - a[0]) * t,
            a[1] + (b[1] - a[1]) * t,
            a[2] + (b[2] - a[2]) * t,
          ];
        }
        api.setTunerLive(true, interpolated);
      }, 33);
    } else {
      if (previewInterval) clearInterval(previewInterval);
      previewInterval = null;
      syncLiveTuner(true);
      props.showToast('Preview loop dihentikan');
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    const all = { ...posesData() };
    if (all[selectedState()]) {
      all[selectedState()].emotion = selectedEmotion();
      all[selectedState()].speed = speed();
    }

    const ok = await api.savePoses(all);
    setIsSaving(false);
    if (ok) {
      props.showToast(`Berhasil menyimpan pose ke poses.json & sync live WGPU!`);
    } else {
      props.showToast(`Gagal menyimpan poses.json`);
    }
  };

  const handleResetAllKeyframe = () => {
    const all = { ...posesData() };
    const st = all[selectedState()];
    if (st) {
      if (activePoseTab() === 'pose_a') st.pose_a = {};
      else st.pose_b = {};
    }
    setPosesData(all);
    syncLiveTuner(true);
    props.showToast(`Semua bone keyframe ${activePoseTab().toUpperCase()} di-reset ke 0`);
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

          {/* Action Tools */}
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
              onClick={() => handleMirror('b', 'a')}
              class="px-2.5 py-1.5 rounded-lg text-xs font-semibold bg-[#282832] hover:bg-[#323240] text-[#F2F2F2] border border-[#3B3B48]"
              title="Mirror simetri B ke A"
            >
              Mirror B→A
            </button>

            <button 
              onClick={togglePreview}
              class={`px-3 py-1.5 rounded-lg text-xs font-bold flex items-center gap-1.5 transition-all ${
                isPreviewing()
                  ? 'bg-[#689D4B] text-white shadow anime-shadow-green'
                  : 'bg-[#282832] hover:bg-[#323240] text-[#F2F2F2] border border-[#3B3B48]'
              }`}
            >
              {isPreviewing() ? <Square size={13} class="fill-current" /> : <Play size={13} class="fill-current" />}
              <span>{isPreviewing() ? 'Stop Preview' : 'Preview A⇄B'}</span>
            </button>

            <button 
              onClick={handleResetAllKeyframe}
              class="px-2.5 py-1.5 rounded-lg text-xs font-semibold bg-[#282832] hover:bg-[#D96868]/20 text-[#D96868] border border-[#3B3B48]"
            >
              Reset All
            </button>

            <button 
              onClick={handleSave}
              disabled={isSaving()}
              class="px-4 py-1.5 rounded-lg text-xs font-bold bg-[#D96868] hover:bg-[#c45a5a] text-white shadow anime-shadow-red flex items-center gap-1.5"
            >
              <Save size={14} />
              <span>{isSaving() ? 'Menyimpan...' : 'Simpan Live'}</span>
            </button>
          </div>
        </div>

        {/* State, Emotion, Speed & Keyframe Selector Bar */}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 pt-3 border-t border-[#3B3B48]/60">
          <div class="space-y-1">
            <label class="text-xs font-semibold text-[#F2F2F2]/70">Target State</label>
            <select
              value={selectedState()}
              onChange={(e) => handleStateChange(e.currentTarget.value)}
              class="w-full bg-[#282832] border border-[#3B3B48] rounded-xl px-3 py-2 text-xs text-[#F2F2F2] outline-none"
            >
              <option value="idle">Idle (Standby)</option>
              <option value="listening">Listening (Mendengarkan)</option>
              <option value="thinking">Thinking (Berpikir)</option>
              <option value="speaking">Speaking (Bicara)</option>
            </select>
          </div>

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

          <div class="space-y-1">
            <div class="flex justify-between text-xs font-semibold">
              <span class="text-[#F2F2F2]/70">Speed Siklus</span>
              <span class="text-[#91AE6E] font-mono">{speed().toFixed(1)}x</span>
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

          <div class="space-y-1">
            <label class="text-xs font-semibold text-[#F2F2F2]/70">Keyframe</label>
            <div class="grid grid-cols-2 gap-1 bg-[#282832] p-1 rounded-xl border border-[#3B3B48]">
              <button
                onClick={() => handleKeyframeTabChange('pose_a')}
                class={`py-1 rounded-lg text-xs font-bold transition-all ${
                  activePoseTab() === 'pose_a'
                    ? 'bg-[#D96868] text-white shadow'
                    : 'text-[#F2F2F2]/60 hover:text-[#F2F2F2]'
                }`}
              >
                Pose A
              </button>
              <button
                onClick={() => handleKeyframeTabChange('pose_b')}
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
        {BONE_CATEGORIES.map((cat) => {
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
        {BONE_CATEGORIES
          .find((c) => c.id === activeBoneCategory())
          ?.bones.map((boneName) => {
            const vals = () => getBoneRotation(boneName);

            return (
              <div class="p-5 rounded-2xl bg-[#1E1E24] border border-[#3B3B48] space-y-3.5">
                <div class="flex items-center justify-between border-b border-[#3B3B48]/50 pb-2">
                  <div>
                    <h4 class="font-bold text-sm text-[#F2F2F2]">
                      {BONE_LABELS[boneName] || boneName}
                    </h4>
                    <span class="text-[10px] font-mono text-[#F2F2F2]/40">
                      ID: {boneName}
                    </span>
                  </div>
                  <button
                    onClick={() => resetSingleBone(boneName)}
                    class="px-2 py-1 rounded text-[11px] font-semibold bg-[#282832] hover:bg-[#D96868]/20 text-[#D96868] border border-[#3B3B48] flex items-center gap-1 transition-all"
                    title="Reset bone ini ke [0, 0, 0]"
                  >
                    <RotateCcw size={11} />
                    <span>Reset</span>
                  </button>
                </div>

                {/* X Axis */}
                <div class="space-y-1">
                  <div class="flex items-center justify-between text-xs">
                    <span class="text-[#D96868] font-bold">
                      Rotasi X (Pitch)
                    </span>
                    <div class="flex items-center gap-1.5">
                      <input
                        type="number"
                        min="-3.14"
                        max="3.14"
                        step="0.01"
                        value={parseFloat(vals()[0].toFixed(2))}
                        onInput={(e) => updateBoneAxis(boneName, 0, parseFloat(e.currentTarget.value) || 0)}
                        class="w-16 bg-[#282832] border border-[#3B3B48] rounded px-1.5 py-0.5 text-xs text-right font-mono text-[#F2F2F2] outline-none focus:border-[#D96868]"
                      />
                      <button
                        onClick={() => resetSingleAxis(boneName, 0)}
                        class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-[#282832] hover:bg-[#D96868]/30 text-[#D96868] border border-[#3B3B48] transition-colors"
                        title="Reset X ke 0.00"
                      >
                        0
                      </button>
                    </div>
                  </div>
                  <input
                    type="range"
                    min="-3.14"
                    max="3.14"
                    step="0.01"
                    value={vals()[0]}
                    onInput={(e) => updateBoneAxis(boneName, 0, parseFloat(e.currentTarget.value) || 0)}
                    class="w-full h-1.5 bg-[#282832] rounded-lg appearance-none cursor-pointer accent-[#D96868]"
                  />
                </div>

                {/* Y Axis */}
                <div class="space-y-1">
                  <div class="flex items-center justify-between text-xs">
                    <span class="text-[#91AE6E] font-bold">
                      Rotasi Y (Yaw)
                    </span>
                    <div class="flex items-center gap-1.5">
                      <input
                        type="number"
                        min="-3.14"
                        max="3.14"
                        step="0.01"
                        value={parseFloat(vals()[1].toFixed(2))}
                        onInput={(e) => updateBoneAxis(boneName, 1, parseFloat(e.currentTarget.value) || 0)}
                        class="w-16 bg-[#282832] border border-[#3B3B48] rounded px-1.5 py-0.5 text-xs text-right font-mono text-[#F2F2F2] outline-none focus:border-[#91AE6E]"
                      />
                      <button
                        onClick={() => resetSingleAxis(boneName, 1)}
                        class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-[#282832] hover:bg-[#91AE6E]/30 text-[#91AE6E] border border-[#3B3B48] transition-colors"
                        title="Reset Y ke 0.00"
                      >
                        0
                      </button>
                    </div>
                  </div>
                  <input
                    type="range"
                    min="-3.14"
                    max="3.14"
                    step="0.01"
                    value={vals()[1]}
                    onInput={(e) => updateBoneAxis(boneName, 1, parseFloat(e.currentTarget.value) || 0)}
                    class="w-full h-1.5 bg-[#282832] rounded-lg appearance-none cursor-pointer accent-[#91AE6E]"
                  />
                </div>

                {/* Z Axis */}
                <div class="space-y-1">
                  <div class="flex items-center justify-between text-xs">
                    <span class="text-[#689D4B] font-bold">
                      Rotasi Z (Roll)
                    </span>
                    <div class="flex items-center gap-1.5">
                      <input
                        type="number"
                        min="-3.14"
                        max="3.14"
                        step="0.01"
                        value={parseFloat(vals()[2].toFixed(2))}
                        onInput={(e) => updateBoneAxis(boneName, 2, parseFloat(e.currentTarget.value) || 0)}
                        class="w-16 bg-[#282832] border border-[#3B3B48] rounded px-1.5 py-0.5 text-xs text-right font-mono text-[#F2F2F2] outline-none focus:border-[#689D4B]"
                      />
                      <button
                        onClick={() => resetSingleAxis(boneName, 2)}
                        class="px-1.5 py-0.5 rounded text-[10px] font-bold bg-[#282832] hover:bg-[#689D4B]/30 text-[#689D4B] border border-[#3B3B48] transition-colors"
                        title="Reset Z ke 0.00"
                      >
                        0
                      </button>
                    </div>
                  </div>
                  <input
                    type="range"
                    min="-3.14"
                    max="3.14"
                    step="0.01"
                    value={vals()[2]}
                    onInput={(e) => updateBoneAxis(boneName, 2, parseFloat(e.currentTarget.value) || 0)}
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
