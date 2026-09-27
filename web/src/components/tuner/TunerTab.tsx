import { Component, createSignal, onMount, onCleanup } from 'solid-js';
import { api, PosesConfig, SinglePose, PoseKeyframes } from '../../api';
import { MIRROR_MAP } from './tunerBones';
import { TunerControls } from './TunerControls';
import { BoneSliders } from './BoneSliders';

interface TunerTabProps {
  showToast: (msg: string) => void;
}

export const TunerTab: Component<TunerTabProps> = (props) => {
  const [selectedState, setSelectedState] = createSignal('idle');
  const [selectedEmotion, setSelectedEmotion] = createSignal('neutral');
  const [speed, setSpeed] = createSignal(1.0);
  const [activePoseIndex, setActivePoseIndex] = createSignal(0);
  const [activeBoneCategory, setActiveBoneCategory] = createSignal('body');
  const [posesData, setPosesData] = createSignal<PosesConfig>({});
  const [isSaving, setIsSaving] = createSignal(false);
  const [isPreviewing, setIsPreviewing] = createSignal(false);

  let syncTimeout: number | null = null;
  let lastSyncTime = 0;
  let isSyncing = false;
  let pendingPose: Record<string, [number, number, number]> | null = null;

  // Helper untuk memastikan state selalu memiliki array poses terstruktur
  const ensurePosesArray = (st?: PoseKeyframes): SinglePose[] => {
    if (!st || !st.poses || st.poses.length === 0) {
      return [{ bones: {} }];
    }
    return st.poses;
  };

  const currentPosesList = (): SinglePose[] => {
    const all = posesData();
    const st = all[selectedState()];
    return ensurePosesArray(st);
  };

  const getCurrentKeyframeBones = (): Record<string, [number, number, number]> => {
    const list = currentPosesList();
    const idx = Math.min(activePoseIndex(), Math.max(0, list.length - 1));
    return list[idx]?.bones || {};
  };

  const syncLiveTuner = (active = true) => {
    if (isPreviewing()) return;
    const bones = getCurrentKeyframeBones();
    api.setTunerLive(active, bones, selectedEmotion(), selectedState());
  };

  const loadPoses = async () => {
    const data = await api.getPoses();
    if (data && Object.keys(data).length > 0) {
      // Normalisasi semua state ke array poses
      const normalized: PosesConfig = {};
      for (const [k, v] of Object.entries(data)) {
        normalized[k] = {
          speed: v.speed || 1.0,
          poses: ensurePosesArray(v),
        };
      }
      setPosesData(normalized);
      const curState = normalized[selectedState()];
      if (curState) {
        setSpeed(curState.speed || 1.0);
      }
      syncLiveTuner(true);
    }
  };

  onMount(() => {
    loadPoses();
  });

  onCleanup(() => {
    if (syncTimeout) { clearTimeout(syncTimeout); syncTimeout = null; }
    pendingPose = null;
    api.setTunerLive(false, {}, 'neutral');
  });

  const handleStateChange = (st: string) => {
    setSelectedState(st);
    setActivePoseIndex(0);
    const cur = posesData()[st];
    if (cur) {
      setSpeed(cur.speed || 1.0);
    }
    syncLiveTuner(true);
  };

  const handleSelectPoseIndex = (idx: number) => {
    setActivePoseIndex(idx);
    syncLiveTuner(true);
  };

  const getBoneRotation = (boneName: string): [number, number, number] => {
    const bones = getCurrentKeyframeBones();
    return bones[boneName] || [0, 0, 0];
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
    const THROTTLE_MS = 50;

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

  const updateBoneAxis = (bone: string, axis: 0 | 1 | 2, val: number) => {
    const clamped = Math.max(-3.14, Math.min(3.14, isNaN(val) ? 0 : val));
    const all = { ...posesData() };
    const stKey = selectedState();
    let st = all[stKey];
    if (!st) {
      st = { speed: speed(), poses: [{ bones: {} }] };
      all[stKey] = st;
    }

    const poses = [...(st.poses || ensurePosesArray(st))];
    const idx = Math.min(activePoseIndex(), Math.max(0, poses.length - 1));
    const curPose = { ...(poses[idx] || { bones: {} }) };
    const curBones = { ...(curPose.bones || {}) };

    const arr = [...(curBones[bone] || [0, 0, 0])] as [number, number, number];
    arr[axis] = clamped;
    curBones[bone] = arr;
    curPose.bones = curBones;
    poses[idx] = curPose;
    st.poses = poses;
    all[stKey] = st;
    setPosesData(all);

    scheduleLiveSync(curBones);
  };

  const resetSingleBone = (bone: string) => {
    const all = { ...posesData() };
    const stKey = selectedState();
    const st = all[stKey];
    if (!st) return;

    const poses = [...(st.poses || ensurePosesArray(st))];
    const idx = Math.min(activePoseIndex(), Math.max(0, poses.length - 1));
    const curPose = { ...(poses[idx] || { bones: {} }) };
    const curBones = { ...(curPose.bones || {}) };

    curBones[bone] = [0, 0, 0];
    curPose.bones = curBones;
    poses[idx] = curPose;
    st.poses = poses;
    all[stKey] = st;
    setPosesData(all);

    api.setTunerLive(true, curBones, selectedEmotion(), selectedState());
    props.showToast(`Bone "${bone}" di-reset ke [0, 0, 0]`);
  };

  const resetSingleAxis = (bone: string, axis: 0 | 1 | 2) => {
    updateBoneAxis(bone, axis, 0);
  };

  // Tambah Pose Baru
  const handleAddPose = () => {
    const all = { ...posesData() };
    const stKey = selectedState();
    const st = all[stKey] || { speed: speed(), poses: [] };
    const poses = [...(st.poses || ensurePosesArray(st))];

    const newIndex = poses.length + 1;
    poses.push({
      bones: {},
    });

    st.poses = poses;
    all[stKey] = st;
    setPosesData(all);
    setActivePoseIndex(poses.length - 1);
    syncLiveTuner(true);
    props.showToast(`Pose baru "Pose ${newIndex}" ditambahkan!`);
  };

  // Duplikat Pose Aktif
  const handleDuplicatePose = () => {
    const all = { ...posesData() };
    const stKey = selectedState();
    const st = all[stKey] || { speed: speed(), poses: [] };
    const poses = [...(st.poses || ensurePosesArray(st))];

    const curIdx = Math.min(activePoseIndex(), Math.max(0, poses.length - 1));
    const sourceBones = poses[curIdx]?.bones || {};
    const newIndex = poses.length + 1;

    poses.push({
      bones: JSON.parse(JSON.stringify(sourceBones)),
    });

    st.poses = poses;
    all[stKey] = st;
    setPosesData(all);
    setActivePoseIndex(poses.length - 1);
    syncLiveTuner(true);
    props.showToast(`Pose ${curIdx + 1} berhasil diduplikasi menjadi Pose ${newIndex}!`);
  };

  // Hapus Pose Tertentu
  const handleDeletePose = (idx: number) => {
    const all = { ...posesData() };
    const stKey = selectedState();
    const st = all[stKey];
    if (!st) return;

    let poses = [...(st.poses || ensurePosesArray(st))];
    if (poses.length <= 1) {
      props.showToast('Minimal harus ada 1 pose keyframe!');
      return;
    }

    poses.splice(idx, 1);
    st.poses = poses;
    all[stKey] = st;
    setPosesData(all);
    setActivePoseIndex(Math.max(0, idx - 1));
    syncLiveTuner(true);
    props.showToast(`Pose berhasil dihapus`);
  };

  // Mirror Simetri Kiri-Kanan pada Pose Aktif
  const handleMirrorActivePose = () => {
    const all = { ...posesData() };
    const stKey = selectedState();
    const st = all[stKey];
    if (!st) return;

    const poses = [...(st.poses || ensurePosesArray(st))];
    const idx = Math.min(activePoseIndex(), Math.max(0, poses.length - 1));
    const curBones = poses[idx]?.bones || {};
    const mirrored: Record<string, [number, number, number]> = {};

    for (const [bone, rot] of Object.entries(curBones)) {
      const targetBone = MIRROR_MAP[bone] || bone;
      mirrored[targetBone] = [
        parseFloat(rot[0].toFixed(3)),
        parseFloat((-rot[1]).toFixed(3)),
        parseFloat((-rot[2]).toFixed(3)),
      ];
    }

    poses[idx] = { ...poses[idx], bones: mirrored };
    st.poses = poses;
    all[stKey] = st;
    setPosesData(all);
    syncLiveTuner(true);
    props.showToast(`Mirroring simetri kiri-kanan pada Pose ${idx + 1} berhasil`);
  };

  // Reset Pose Aktif ke Netral
  const handleResetActivePose = () => {
    const all = { ...posesData() };
    const stKey = selectedState();
    const st = all[stKey];
    if (!st) return;

    const poses = [...(st.poses || ensurePosesArray(st))];
    const idx = Math.min(activePoseIndex(), Math.max(0, poses.length - 1));

    poses[idx] = { ...poses[idx], bones: {} };
    st.poses = poses;
    all[stKey] = st;
    setPosesData(all);
    syncLiveTuner(true);
    props.showToast(`Semua bone pada Pose ${idx + 1} di-reset ke [0, 0, 0]`);
  };

  // Toggle Preview Loop
  const togglePreview = async () => {
    const next = !isPreviewing();

    if (syncTimeout) { clearTimeout(syncTimeout); syncTimeout = null; }

    if (next) {
      const all = { ...posesData() };
      if (all[selectedState()]) {
        all[selectedState()].speed = speed();
      }
      await api.setTunerPreview(all, selectedEmotion(), selectedState());
      setIsPreviewing(true);
      props.showToast(`Preview loop animasi ${selectedState().toUpperCase()} aktif (60 FPS Native SLERP)`);
    } else {
      setIsPreviewing(false);
      syncLiveTuner(true);
      props.showToast('Kembali ke mode edit keyframe');
    }
  };

  // Simpan ke poses.json
  const handleSave = async () => {
    setIsSaving(true);
    const all = { ...posesData() };
    if (all[selectedState()]) {
      all[selectedState()].speed = speed();
    }

    const ok = await api.savePoses(all);
    setIsSaving(false);
    if (ok) {
      props.showToast('Berhasil menyimpan konfigurasi pose ke poses.json!');
    } else {
      props.showToast('Gagal menyimpan poses.json');
    }
  };

  return (
    <div class="space-y-6">
      <TunerControls
        selectedState={selectedState()}
        selectedEmotion={selectedEmotion()}
        speed={speed()}
        posesList={currentPosesList()}
        activePoseIndex={activePoseIndex()}
        isSaving={isSaving()}
        isPreviewing={isPreviewing()}
        onSelectState={handleStateChange}
        onSelectEmotion={(emo) => {
          setSelectedEmotion(emo);
          if (isPreviewing()) {
            api.setTunerPreview(posesData(), emo, selectedState());
          } else {
            api.setTunerLive(true, getCurrentKeyframeBones(), emo, selectedState());
          }
        }}
        onChangeSpeed={(sp) => {
          setSpeed(sp);
          const all = { ...posesData() };
          if (all[selectedState()]) {
            all[selectedState()].speed = sp;
            setPosesData(all);
          }
          if (isPreviewing()) {
            api.setTunerPreview(all, selectedEmotion(), selectedState());
          }
        }}
        onSelectPoseIndex={handleSelectPoseIndex}
        onAddPose={handleAddPose}
        onDuplicatePose={handleDuplicatePose}
        onDeletePose={handleDeletePose}
        onMirrorActivePose={handleMirrorActivePose}
        onResetActivePose={handleResetActivePose}
        onTogglePreview={togglePreview}
        onSave={handleSave}
      />

      <BoneSliders
        activeCategory={activeBoneCategory()}
        onSelectCategory={setActiveBoneCategory}
        getBoneRotation={getBoneRotation}
        onUpdateBoneAxis={updateBoneAxis}
        onResetSingleBone={resetSingleBone}
        onResetSingleAxis={resetSingleAxis}
      />
    </div>
  );
};
