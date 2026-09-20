import { Component, createSignal, onMount, onCleanup } from 'solid-js';
import { 
  Save, 
  RotateCcw, 
  Play, 
  Square, 
  Sliders,
  Move,
  RotateCw,
  ZoomIn,
  MousePointer,
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
  const [selectedEmotion, setSelectedEmotion] = createSignal('neutral');
  const [speed, setSpeed] = createSignal(2.5);
  const [activeBoneCategory, setActiveBoneCategory] = createSignal('body');
  const [posesData, setPosesData] = createSignal<PosesConfig>({});
  const [isSaving, setIsSaving] = createSignal(false);
  const [isPreviewing, setIsPreviewing] = createSignal(false);

  let previewInterval: number | null = null;
  let syncTimeout: number | null = null;
  let lastSyncTime = 0;
  let isSyncing = false;
  let pendingPose: Record<string, [number, number, number]> | null = null;

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
    if (syncTimeout) { clearTimeout(syncTimeout); syncTimeout = null; }
    pendingPose = null;
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

  const flushLiveSync = async (pose: Record<string, [number, number, number]>) => {
    if (isPreviewing()) return;
    isSyncing = true;
    lastSyncTime = Date.now();
    try {
      await api.setTunerLive(true, pose, selectedEmotion(), selectedState());
    } finally {
      isSyncing = false;
      if (pendingPose) {
        const nextPose = pendingPose;
        pendingPose = null;
        flushLiveSync(nextPose);
      }
    }
  };

  const scheduleLiveSync = (pose: Record<string, [number, number, number]>) => {
    if (isPreviewing()) return;
    const now = Date.now();
    const elapsed = now - lastSyncTime;
    const THROTTLE_MS = 50; // Maksimal 20 req/detik (gerakan 3D tetap halus & network bebas flood)

    if (elapsed >= THROTTLE_MS && !isSyncing) {
      if (syncTimeout) { clearTimeout(syncTimeout); syncTimeout = null; }
      flushLiveSync(pose);
    } else {
      pendingPose = pose;
      if (!syncTimeout) {
        syncTimeout = window.setTimeout(() => {
          syncTimeout = null;
          if (pendingPose && !isSyncing) {
            const nextPose = pendingPose;
            pendingPose = null;
            flushLiveSync(nextPose);
          }
        }, Math.max(10, THROTTLE_MS - elapsed));
      }
    }
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
      <div class="p-4 sm:p-6 bg-theme-surface border border-theme-border space-y-4">
        {/* 1. Header Banner */}
        <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
          <div class="flex items-center gap-3">
            <div class="p-2.5 bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/30 shrink-0">
              <Sliders size={22} />
            </div>
            <div>
              <div class="flex items-center gap-2">
                <h2 class="text-xl sm:text-2xl font-carter tracking-wide text-theme-text leading-none">
                  POSE & MOTION TUNER
                </h2>
                <span class="px-2 py-0.5 text-xs font-mono font-bold bg-theme-bg text-theme-cyan border border-theme-border">
                  poses.json
                </span>
              </div>
              <p class="text-xs text-theme-text/60 mt-0.5">
                Visual bone keyframe editor & dynamic emotion preview
              </p>
            </div>
          </div>

          <div class="flex items-center gap-2">
            <span class="text-xs font-mono px-2.5 py-1 bg-theme-card text-theme-cyan border border-theme-border font-semibold">
              State: <strong class="uppercase text-theme-text">{selectedState()}</strong>
            </span>
          </div>
        </div>

        {/* 2. Navigasi Viewport & Gestur Kamera 3D Grid */}
        <div class="bg-theme-bg p-3 border border-theme-border space-y-2">
          <div class="flex items-center justify-between">
            <span class="text-xs font-bold uppercase tracking-wider text-theme-cyan flex items-center gap-1.5">
              <Sparkles size={13} />
              Panduan Navigasi Layar Avatar
            </span>
            <span class="text-xs text-theme-text/50">Interaksi Mouse & Keyboard</span>
          </div>

          <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-5 gap-2">
            {/* Card 1 */}
            <div class="p-2 bg-theme-surface border border-theme-border/60 flex flex-col justify-between gap-1 hover:border-theme-cyan/40 transition-colors">
              <span class="text-xs text-theme-text/60 flex items-center gap-1">
                <Move size={11} class="text-theme-cyan" />
                Pindah Jendela
              </span>
              <kbd class="px-2 py-1 bg-theme-card text-theme-cyan text-xs font-mono font-bold text-center border border-theme-border">
                Drag Kiri
              </kbd>
            </div>

            {/* Card 2 */}
            <div class="p-2 bg-theme-surface border border-theme-border/60 flex flex-col justify-between gap-1 hover:border-theme-rose/40 transition-colors">
              <span class="text-xs text-theme-text/60 flex items-center gap-1">
                <RotateCw size={11} class="text-theme-rose" />
                Orbit Putar 3D
              </span>
              <kbd class="px-2 py-1 bg-theme-card text-theme-rose text-xs font-mono font-bold text-center border border-theme-border">
                Drag Kanan
              </kbd>
            </div>

            {/* Card 3 */}
            <div class="p-2 bg-theme-surface border border-theme-border/60 flex flex-col justify-between gap-1 hover:border-theme-rose/40 transition-colors">
              <span class="text-xs text-theme-text/60 flex items-center gap-1">
                <MousePointer size={11} class="text-theme-rose" />
                Pan / Geser Model
              </span>
              <kbd class="px-2 py-1 bg-theme-card text-theme-rose text-xs font-mono font-bold text-center border border-theme-border">
                Ctrl + Drag Kanan
              </kbd>
            </div>

            {/* Card 4 */}
            <div class="p-2 bg-theme-surface border border-theme-border/60 flex flex-col justify-between gap-1 hover:border-theme-text/40 transition-colors">
              <span class="text-xs text-theme-text/60 flex items-center gap-1">
                <ZoomIn size={11} class="text-theme-text/80" />
                Zoom POV
              </span>
              <kbd class="px-2 py-1 bg-theme-card text-theme-text text-xs font-mono font-bold text-center border border-theme-border">
                Scroll Mouse
              </kbd>
            </div>

            {/* Card 5 */}
            <div class="p-2 bg-theme-surface border border-theme-border/60 flex flex-col justify-between gap-1 col-span-2 sm:col-span-1 hover:border-theme-text/40 transition-colors">
              <span class="text-xs text-theme-text/60 flex items-center gap-1">
                <RotateCcw size={11} class="text-theme-text/80" />
                Reset Kamera
              </span>
              <kbd class="px-2 py-1 bg-theme-card text-theme-text text-xs font-mono font-bold text-center border border-theme-border">
                Klik Tengah
              </kbd>
            </div>
          </div>
        </div>

        {/* 3. Structured Action Toolbar (Between: Kiri = Transfer & Reset, Kanan = Preview & Simpan) */}
        <div class="p-2.5 bg-theme-bg border border-theme-border flex flex-col md:flex-row md:items-center justify-between gap-2.5">
          {/* Sisi Kiri: Keyframe Copy & Mirror + Reset Tools */}
          <div class="flex flex-wrap items-center gap-1.5 sm:gap-2">
            <div class="flex items-center bg-theme-card p-1 border border-theme-border">
              <button 
                onClick={() => handleCopy('a', 'b')}
                class="px-2.5 py-1.5 text-xs font-semibold text-theme-text/80 hover:text-theme-text hover:bg-theme-surface transition-colors"
                title="Copy Pose A ke Pose B"
              >
                A→B
              </button>
              <button 
                onClick={() => handleCopy('b', 'a')}
                class="px-2.5 py-1.5 text-xs font-semibold text-theme-text/80 hover:text-theme-text hover:bg-theme-surface transition-colors"
                title="Copy Pose B ke Pose A"
              >
                B→A
              </button>
              <div class="w-px h-3.5 bg-theme-border mx-1" />
              <button 
                onClick={() => handleMirror('a', 'b')}
                class="px-2.5 py-1.5 text-xs font-semibold text-theme-text/80 hover:text-theme-text hover:bg-theme-surface transition-colors"
                title="Mirror simetri A ke B"
              >
                Mir A→B
              </button>
              <button 
                onClick={() => handleMirror('b', 'a')}
                class="px-2.5 py-1.5 text-xs font-semibold text-theme-text/80 hover:text-theme-text hover:bg-theme-surface transition-colors"
                title="Mirror simetri B ke A"
              >
                Mir B→A
              </button>
            </div>

            <button 
              onClick={handleResetAllKeyframe}
              class="px-3 py-1.5 text-xs font-semibold bg-theme-card hover:bg-theme-rose/20 text-theme-rose border border-theme-rose/30 flex items-center gap-1.5 transition-all"
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
              class={`px-3.5 py-1.5 text-xs font-bold flex items-center gap-1.5 transition-all shadow-sm ${
                isPreviewing()
                  ? 'bg-theme-rose text-theme-card anime-shadow-red font-bold'
                  : 'bg-theme-card hover:bg-theme-border text-theme-text border border-theme-border'
              }`}
            >
              {isPreviewing() ? <Square size={13} class="fill-current" /> : <Play size={13} class="fill-current" />}
              <span>{isPreviewing() ? 'Stop Preview' : 'Preview A⇄B'}</span>
            </button>

            <button 
              onClick={handleSave}
              disabled={isSaving()}
              class="px-4 py-1.5 text-xs font-bold bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card shadow-md anime-shadow-green flex items-center gap-1.5 transition-all"
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
            <label class="text-xs font-semibold text-theme-text/70 block h-4">Target State</label>
            <select
              value={selectedState()}
              onChange={(e) => handleStateChange(e.currentTarget.value)}
              class="w-full h-[38px] bg-theme-card border border-theme-border px-3 text-xs text-theme-text outline-none focus:border-theme-cyan transition-colors"
            >
              <option value="idle">Idle (Standby)</option>
              <option value="listening">Listening (Mendengarkan)</option>
              <option value="thinking">Thinking (Berpikir)</option>
              <option value="speaking">Speaking (Bicara)</option>
            </select>
          </div>

          {/* Ekspresi Emosi */}
          <div class="space-y-1.5 flex flex-col justify-between">
            <label class="text-xs font-semibold text-theme-text/70 block h-4">Ekspresi Emosi</label>
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
              class="w-full h-[38px] bg-theme-card border border-theme-border px-3 text-xs text-theme-text outline-none focus:border-theme-cyan transition-colors"
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
              <label class="text-xs font-semibold text-theme-text/70">Speed Siklus</label>
              <span class="text-xs font-mono font-bold text-theme-cyan">{speed().toFixed(1)}s</span>
            </div>
            <div class="w-full h-[38px] bg-theme-card border border-theme-border px-3 flex items-center">
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
                class="w-full h-1.5 bg-theme-bg appearance-none cursor-pointer accent-theme-cyan"
              />
            </div>
          </div>

          {/* Keyframe Pose A / B Toggle */}
          <div class="space-y-1.5 flex flex-col justify-between">
            <label class="text-xs font-semibold text-theme-text/70 block h-4">Keyframe Target</label>
            <div class="grid grid-cols-2 gap-1.5 bg-theme-card p-1 border border-theme-border h-[38px] items-center">
              <button
                onClick={() => handleKeyframeTabChange('pose_a')}
                class={`h-full flex items-center justify-center text-xs font-bold transition-all ${
                  activePoseTab() === 'pose_a'
                    ? 'bg-theme-rose text-theme-card shadow'
                    : 'text-theme-text/60 hover:text-theme-text'
                }`}
              >
                Pose A
              </button>
              <button
                onClick={() => handleKeyframeTabChange('pose_b')}
                class={`h-full flex items-center justify-center text-xs font-bold transition-all ${
                  activePoseTab() === 'pose_b'
                    ? 'bg-theme-cyan text-theme-card shadow'
                    : 'text-theme-text/60 hover:text-theme-text'
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
              class={`px-4 py-2 text-xs font-bold whitespace-nowrap transition-all ${
                isCurrent()
                  ? 'bg-theme-cyan text-theme-card shadow anime-shadow-green font-bold'
                  : 'bg-theme-surface text-theme-text/70 border border-theme-border hover:bg-theme-card hover:text-theme-text'
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
              <div class="p-4 sm:p-5 bg-theme-surface border border-theme-border space-y-3 shadow-lg">
                <div class="flex items-center justify-between border-b border-theme-border/50 pb-2.5">
                  <div>
                    <h4 class="font-bold text-sm text-theme-text">
                      {BONE_LABELS[boneName] || boneName}
                    </h4>
                    <span class="text-xs font-mono text-theme-text/40">
                      ID: {boneName}
                    </span>
                  </div>
                  <button
                    onClick={() => resetSingleBone(boneName)}
                    class="px-2.5 py-1 text-xs font-semibold bg-theme-card hover:bg-theme-rose/20 text-theme-rose border border-theme-rose/30 flex items-center gap-1 transition-all"
                    title="Reset bone ini ke [0, 0, 0]"
                  >
                    <RotateCcw size={11} />
                    <span>Reset</span>
                  </button>
                </div>

                {/* X Axis */}
                <div class="space-y-1">
                  <div class="flex items-center justify-between text-xs">
                    <span class="text-theme-rose font-bold">
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
                        class="w-20 bg-theme-card border border-theme-border px-2.5 py-1 text-xs text-right font-mono font-semibold text-theme-text outline-none focus:border-theme-rose transition-colors"
                      />
                      <button
                        onClick={() => resetSingleAxis(boneName, 0)}
                        class="px-2 py-1 text-xs font-bold bg-theme-card hover:bg-theme-rose/30 text-theme-rose border border-theme-border transition-colors"
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
                    class="w-full h-1.5 bg-theme-card appearance-none cursor-pointer accent-theme-rose"
                  />
                </div>

                {/* Y Axis */}
                <div class="space-y-1">
                  <div class="flex items-center justify-between text-xs">
                    <span class="text-theme-cyan font-bold">
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
                        class="w-20 bg-theme-card border border-theme-border px-2.5 py-1 text-xs text-right font-mono font-semibold text-theme-text outline-none focus:border-theme-cyan transition-colors"
                      />
                      <button
                        onClick={() => resetSingleAxis(boneName, 1)}
                        class="px-2 py-1 text-xs font-bold bg-theme-card hover:bg-theme-cyan/30 text-theme-cyan border border-theme-border transition-colors"
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
                    class="w-full h-1.5 bg-theme-card appearance-none cursor-pointer accent-theme-cyan"
                  />
                </div>

                {/* Z Axis */}
                <div class="space-y-1">
                  <div class="flex items-center justify-between text-xs">
                    <span class="text-theme-text font-bold">
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
                        class="w-20 bg-theme-card border border-theme-border px-2.5 py-1 text-xs text-right font-mono font-semibold text-theme-text outline-none focus:border-theme-text transition-colors"
                      />
                      <button
                        onClick={() => resetSingleAxis(boneName, 2)}
                        class="px-2 py-1 text-xs font-bold bg-theme-card hover:bg-theme-text/30 text-theme-text border border-theme-border transition-colors"
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
                    class="w-full h-1.5 bg-theme-card appearance-none cursor-pointer accent-theme-text"
                  />
                </div>
              </div>
            );
          })}
      </div>
    </div>
  );
};
