import { Component, createSignal, onMount, onCleanup } from 'solid-js';
import { 
  Save, 
  RotateCcw, 
  Play, 
  Square, 
  Sliders 
} from 'lucide-solid';
import { api, PosesConfig } from '../api';
import { BONE_CATEGORIES, BONE_LABELS, MIRROR_MAP } from './tunerBones';

interface TunerTabProps {
  showToast: (msg: string) => void;
}

export const TunerTab: Component<TunerTabProps> = (props) => {
  const [activePoseTab, setActivePoseTab] = createSignal<'pose_a' | 'pose_b'>('pose_a');
  const [selectedState, setSelectedState] = createSignal('idle');
  const [selectedEmotion, setSelectedEmotion] = createSignal('neutral');
  const [speed, setSpeed] = createSignal(2.5);
  const [activeBoneCategory, setActiveBoneCategory] = createSignal('body');
  const [posesData, setPosesData] = createSignal<PosesConfig>({});
  const [isSaving, setIsSaving] = createSignal(false);
  const [isPreviewing, setIsPreviewing] = createSignal(false);

  let previewInterval: number | null = null;
  let syncTimeout: number | null = null;

  const getCurrentKeyframeBones = (): Record<string, [number, number, number]> => {
    const st = posesData()[selectedState()];
    if (!st) return {};
    return activePoseTab() === 'pose_a' ? st.pose_a || {} : st.pose_b || {};
  };

  const syncLiveTuner = (active = true) => {
    if (isPreviewing()) return;
    const bones = getCurrentKeyframeBones();
    api.setTunerLive(active, bones, selectedEmotion(), selectedState());
  };

  const loadPoses = async () => {
    const data = await api.getPoses();
    if (data && Object.keys(data).length > 0) {
      setPosesData(data);
      const curState = data[selectedState()];
      if (curState) {
        setSpeed(curState.speed || 2.5);
      }
      syncLiveTuner(true);
    }
  };

  onMount(() => {
    loadPoses();
  });

  onCleanup(() => {
    if (previewInterval) clearInterval(previewInterval);
    if (syncTimeout) { cancelAnimationFrame(syncTimeout); syncTimeout = null; }
    // Reset tuner mode ke false dan kosongkan live tuning bones di backend
    api.setTunerLive(false, {}, 'neutral');
  });

  const handleStateChange = (st: string) => {
    setSelectedState(st);
    const cur = posesData()[st];
    if (cur) {
      setSpeed(cur.speed || 2.5);
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
      st = { speed: speed(), pose_a: {}, pose_b: {} };
      all[stKey] = st;
    }

    const keyframeName = activePoseTab();
    const curPose = { ...(st[keyframeName] || {}) };
    const arr = [...(curPose[bone] || [0, 0, 0])] as [number, number, number];
    arr[axis] = clamped;
    curPose[bone] = arr;
    st[keyframeName] = curPose;
    setPosesData(all);

    // Throttle / requestAnimationFrame live sync agar tidak spam HTTP request
    scheduleLiveSync(curPose);
  };

  const scheduleLiveSync = (pose: Record<string, [number, number, number]>) => {
    if (syncTimeout) cancelAnimationFrame(syncTimeout);
    syncTimeout = requestAnimationFrame(() => {
      syncTimeout = null;
      if (isPreviewing()) return; // jangan override saat preview sedang berjalan
      api.setTunerLive(true, pose, selectedEmotion(), selectedState());
    });
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

    api.setTunerLive(true, curPose, selectedEmotion(), selectedState());
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

  const togglePreview = async () => {
    const next = !isPreviewing();

    if (syncTimeout) { cancelAnimationFrame(syncTimeout); syncTimeout = null; }

    if (next) {
      const all = { ...posesData() };
      if (all[selectedState()]) {
        all[selectedState()].speed = speed();
      }
      // Kirim data preview ke engine secara in-memory (tanpa overwrite poses.json di disk!)
      await api.setTunerPreview(all, selectedEmotion(), selectedState());
      setIsPreviewing(true);
      props.showToast(`Preview loop animasi ${selectedState().toUpperCase()} (60 FPS Native SLERP)`);
    } else {
      // Kembali ke mode freeze keyframe editing
      setIsPreviewing(false);
      syncLiveTuner(true);
      props.showToast('Kembali ke mode edit keyframe');
    }
  };

  const handleSave = async () => {
    setIsSaving(true);
    const all = { ...posesData() };
    if (all[selectedState()]) {
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
      {/* Top Tuner Control Bar */}
      <div class="p-4 sm:p-6 rounded-2xl bg-[#1D252E] border border-[#2F3B46] space-y-4">
        {/* 1. Header Banner */}
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
          <div class="flex items-center gap-3">
            <div class="p-2.5 rounded-xl bg-[#7BDFF2]/20 text-[#7BDFF2] border border-[#7BDFF2]/30 shrink-0">
              <Sliders size={22} />
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h2 class="text-xl sm:text-2xl font-bangers tracking-wide text-[#EFF7F6] leading-none">
                  POSE & MOTION TUNER
                </h2>
                <span class="px-2 py-0.5 rounded text-[10px] font-mono font-bold bg-[#161D24] text-[#7BDFF2] border border-[#2F3B46]">
                  poses.json
                </span>
              </div>
              <p class="text-xs text-[#EFF7F6]/60 mt-0.5">
                Visual bone keyframe editor & dynamic emotion preview
              </p>
            </div>
          </div>

          <div class="flex items-center gap-2">
            <span class="text-xs font-mono px-2.5 py-1 rounded-full bg-[#232D36] text-[#7BDFF2] border border-[#2F3B46] font-semibold">
              State: <strong class="uppercase text-[#EFF7F6]">{selectedState()}</strong>
            </span>
          </div>
        </div>

        {/* 2. Structured Action Toolbar (Between: Kiri = Transfer & Reset, Kanan = Preview & Simpan) */}
        <div class="p-2.5 rounded-xl bg-[#161D24] border border-[#2F3B46] flex flex-col md:flex-row md:items-center justify-between gap-2.5">
          {/* Sisi Kiri: Keyframe Copy & Mirror + Reset Tools */}
          <div class="flex flex-wrap items-center gap-1.5 sm:gap-2">
            <div class="flex items-center bg-[#232D36] p-1 rounded-lg border border-[#2F3B46]">
              <button 
                onClick={() => handleCopy('a', 'b')}
                class="px-2.5 py-1.5 rounded-md text-xs font-semibold text-[#EFF7F6]/80 hover:text-[#EFF7F6] hover:bg-[#1D252E] transition-colors"
                title="Copy Pose A ke Pose B"
              >
                A→B
              </button>
              <button 
                onClick={() => handleCopy('b', 'a')}
                class="px-2.5 py-1.5 rounded-md text-xs font-semibold text-[#EFF7F6]/80 hover:text-[#EFF7F6] hover:bg-[#1D252E] transition-colors"
                title="Copy Pose B ke Pose A"
              >
                B→A
              </button>
              <div class="w-px h-3.5 bg-[#2F3B46] mx-1" />
              <button 
                onClick={() => handleMirror('a', 'b')}
                class="px-2.5 py-1.5 rounded-md text-xs font-semibold text-[#EFF7F6]/80 hover:text-[#EFF7F6] hover:bg-[#1D252E] transition-colors"
                title="Mirror simetri A ke B"
              >
                Mir A→B
              </button>
              <button 
                onClick={() => handleMirror('b', 'a')}
                class="px-2.5 py-1.5 rounded-md text-xs font-semibold text-[#EFF7F6]/80 hover:text-[#EFF7F6] hover:bg-[#1D252E] transition-colors"
                title="Mirror simetri B ke A"
              >
                Mir B→A
              </button>
            </div>

            <button 
              onClick={handleResetAllKeyframe}
              class="px-3 py-1.5 rounded-lg text-xs font-semibold bg-[#232D36] hover:bg-[#FF7B7B]/20 text-[#FF7B7B] border border-[#FF7B7B]/30 flex items-center gap-1.5 transition-all"
              title="Reset seluruh bone keyframe aktif ke [0, 0, 0]"
            >
              <RotateCcw size={12} />
              <span>Reset</span>
            </button>
          </div>

          {/* Sisi Kanan: Preview Loop & CTA Simpan Live */}
          <div class="flex items-center gap-2 justify-end">
            <button 
              onClick={togglePreview}
              class={`px-3.5 py-1.5 rounded-lg text-xs font-bold flex items-center gap-1.5 transition-all shadow-sm ${
                isPreviewing()
                  ? 'bg-[#F2B5D4] text-[#232D36] anime-shadow-red font-bold'
                  : 'bg-[#232D36] hover:bg-[#2F3B46] text-[#EFF7F6] border border-[#2F3B46]'
              }`}
            >
              {isPreviewing() ? <Square size={13} class="fill-current" /> : <Play size={13} class="fill-current" />}
              <span>{isPreviewing() ? 'Stop Preview' : 'Preview A⇄B'}</span>
            </button>

            <button 
              onClick={handleSave}
              disabled={isSaving()}
              class="px-4 py-1.5 rounded-lg text-xs font-bold bg-[#7BDFF2] hover:bg-[#68cbdd] text-[#232D36] shadow-md anime-shadow-green flex items-center gap-1.5 transition-all"
            >
              <Save size={13} />
              <span>{isSaving() ? 'Menyimpan...' : 'Simpan Live'}</span>
            </button>
          </div>
        </div>

        {/* 3. State, Emotion, Speed & Keyframe Selector Bar */}
        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-3 pt-2">
          {/* Target State */}
          <div class="space-y-1.5 flex flex-col justify-between">
            <label class="text-xs font-semibold text-[#EFF7F6]/70 block h-4">Target State</label>
            <select
              value={selectedState()}
              onChange={(e) => handleStateChange(e.currentTarget.value)}
              class="w-full h-[38px] bg-[#232D36] border border-[#2F3B46] rounded-xl px-3 text-xs text-[#EFF7F6] outline-none focus:border-[#7BDFF2] transition-colors"
            >
              <option value="idle">Idle (Standby)</option>
              <option value="listening">Listening (Mendengarkan)</option>
              <option value="thinking">Thinking (Berpikir)</option>
              <option value="speaking">Speaking (Bicara)</option>
            </select>
          </div>

          {/* Ekspresi Emosi */}
          <div class="space-y-1.5 flex flex-col justify-between">
            <label class="text-xs font-semibold text-[#EFF7F6]/70 block h-4">Ekspresi Emosi</label>
            <select
              value={selectedEmotion()}
              onChange={(e) => {
                const val = e.currentTarget.value;
                setSelectedEmotion(val);
                if (isPreviewing()) {
                  api.setTunerPreview(posesData(), val, selectedState());
                } else {
                  api.setTunerLive(true, getCurrentKeyframeBones(), val, selectedState());
                }
              }}
              class="w-full h-[38px] bg-[#232D36] border border-[#2F3B46] rounded-xl px-3 text-xs text-[#EFF7F6] outline-none focus:border-[#7BDFF2] transition-colors"
            >
              <option value="neutral">Neutral (Biasa)</option>
              <option value="happy">Happy (Senang)</option>
              <option value="shy">Shy (Malu / Blushing)</option>
              <option value="surprised">Surprised (Kaget)</option>
              <option value="thinking">Thinking (Mikir)</option>
              <option value="sad">Sad (Sedih)</option>
              <option value="excited">Excited (Semangat)</option>
              <option value="angry">Angry (Marah / Cemberut)</option>
            </select>
          </div>

          {/* Speed Siklus */}
          <div class="space-y-1.5 flex flex-col justify-between">
            <div class="flex items-center justify-between h-4">
              <label class="text-xs font-semibold text-[#EFF7F6]/70">Speed Siklus</label>
              <span class="text-xs font-mono font-bold text-[#7BDFF2]">{speed().toFixed(1)}s</span>
            </div>
            <div class="w-full h-[38px] bg-[#232D36] border border-[#2F3B46] rounded-xl px-3 flex items-center">
              <input 
                type="range" 
                min="0.5" 
                max="5.0" 
                step="0.1" 
                value={speed()}
                onInput={(e) => {
                  const val = parseFloat(e.currentTarget.value);
                  setSpeed(val);
                  const all = { ...posesData() };
                  if (all[selectedState()]) {
                    all[selectedState()].speed = val;
                    setPosesData(all);
                  }
                  if (isPreviewing()) {
                    api.setTunerPreview(all, selectedEmotion(), selectedState());
                  }
                }}
                class="w-full h-1.5 bg-[#161D24] rounded-lg appearance-none cursor-pointer accent-[#7BDFF2]"
              />
            </div>
          </div>

          {/* Keyframe Pose A / B Toggle */}
          <div class="space-y-1.5 flex flex-col justify-between">
            <label class="text-xs font-semibold text-[#EFF7F6]/70 block h-4">Keyframe Target</label>
            <div class="grid grid-cols-2 gap-1.5 bg-[#232D36] p-1 rounded-xl border border-[#2F3B46] h-[38px] items-center">
              <button
                onClick={() => handleKeyframeTabChange('pose_a')}
                class={`h-full flex items-center justify-center rounded-lg text-xs font-bold transition-all ${
                  activePoseTab() === 'pose_a'
                    ? 'bg-[#F2B5D4] text-[#232D36] shadow'
                    : 'text-[#EFF7F6]/60 hover:text-[#EFF7F6]'
                }`}
              >
                Pose A
              </button>
              <button
                onClick={() => handleKeyframeTabChange('pose_b')}
                class={`h-full flex items-center justify-center rounded-lg text-xs font-bold transition-all ${
                  activePoseTab() === 'pose_b'
                    ? 'bg-[#7BDFF2] text-[#232D36] shadow'
                    : 'text-[#EFF7F6]/60 hover:text-[#EFF7F6]'
                }`}
              >
                Pose B
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Category Navigation Pills */}
      <div class="flex items-center gap-2 overflow-x-auto pb-1 custom-scrollbar">
        {BONE_CATEGORIES.map((cat) => {
          const isCurrent = () => activeBoneCategory() === cat.id;

          return (
            <button
              onClick={() => setActiveBoneCategory(cat.id)}
              class={`px-4 py-2 rounded-xl text-xs font-bold whitespace-nowrap transition-all ${
                isCurrent()
                  ? 'bg-[#7BDFF2] text-[#232D36] shadow anime-shadow-green font-bold'
                  : 'bg-[#1D252E] text-[#EFF7F6]/70 border border-[#2F3B46] hover:bg-[#232D36] hover:text-[#EFF7F6]'
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
              <div class="p-4 sm:p-5 rounded-2xl bg-[#1D252E] border border-[#2F3B46] space-y-3 shadow-lg">
                <div class="flex items-center justify-between border-b border-[#2F3B46]/50 pb-2.5">
                  <div>
                    <h4 class="font-bold text-sm text-[#EFF7F6]">
                      {BONE_LABELS[boneName] || boneName}
                    </h4>
                    <span class="text-[10px] font-mono text-[#EFF7F6]/40">
                      ID: {boneName}
                    </span>
                  </div>
                  <button
                    onClick={() => resetSingleBone(boneName)}
                    class="px-2.5 py-1 rounded-lg text-xs font-semibold bg-[#232D36] hover:bg-[#FF7B7B]/20 text-[#FF7B7B] border border-[#FF7B7B]/30 flex items-center gap-1 transition-all"
                    title="Reset bone ini ke [0, 0, 0]"
                  >
                    <RotateCcw size={11} />
                    <span>Reset</span>
                  </button>
                </div>

                {/* X Axis */}
                <div class="space-y-1">
                  <div class="flex items-center justify-between text-xs">
                    <span class="text-[#F2B5D4] font-bold">
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
                        class="w-20 bg-[#232D36] border border-[#2F3B46] rounded-lg px-2.5 py-1 text-xs text-right font-mono font-semibold text-[#EFF7F6] outline-none focus:border-[#F2B5D4] transition-colors"
                      />
                      <button
                        onClick={() => resetSingleAxis(boneName, 0)}
                        class="px-2 py-1 rounded-lg text-[10px] font-bold bg-[#232D36] hover:bg-[#F2B5D4]/30 text-[#F2B5D4] border border-[#2F3B46] transition-colors"
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
                    class="w-full h-1.5 bg-[#232D36] rounded-lg appearance-none cursor-pointer accent-[#F2B5D4]"
                  />
                </div>

                {/* Y Axis */}
                <div class="space-y-1">
                  <div class="flex items-center justify-between text-xs">
                    <span class="text-[#7BDFF2] font-bold">
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
                        class="w-20 bg-[#232D36] border border-[#2F3B46] rounded-lg px-2.5 py-1 text-xs text-right font-mono font-semibold text-[#EFF7F6] outline-none focus:border-[#7BDFF2] transition-colors"
                      />
                      <button
                        onClick={() => resetSingleAxis(boneName, 1)}
                        class="px-2 py-1 rounded-lg text-[10px] font-bold bg-[#232D36] hover:bg-[#7BDFF2]/30 text-[#7BDFF2] border border-[#2F3B46] transition-colors"
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
                    class="w-full h-1.5 bg-[#232D36] rounded-lg appearance-none cursor-pointer accent-[#7BDFF2]"
                  />
                </div>

                {/* Z Axis */}
                <div class="space-y-1">
                  <div class="flex items-center justify-between text-xs">
                    <span class="text-[#EFF7F6] font-bold">
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
                        class="w-20 bg-[#232D36] border border-[#2F3B46] rounded-lg px-2.5 py-1 text-xs text-right font-mono font-semibold text-[#EFF7F6] outline-none focus:border-[#EFF7F6] transition-colors"
                      />
                      <button
                        onClick={() => resetSingleAxis(boneName, 2)}
                        class="px-2 py-1 rounded-lg text-[10px] font-bold bg-[#232D36] hover:bg-[#EFF7F6]/30 text-[#EFF7F6] border border-[#2F3B46] transition-colors"
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
                    class="w-full h-1.5 bg-[#232D36] rounded-lg appearance-none cursor-pointer accent-[#EFF7F6]"
                  />
                </div>
              </div>
            );
          })}
      </div>
    </div>
  );
};
