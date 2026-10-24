import { Component } from 'solid-js';
import { 
  Save, 
  RotateCcw, 
  Play, 
  Square, 
  Sliders, 
  Plus, 
  Copy, 
  Trash2, 
  FlipHorizontal 
} from 'lucide-solid';
import { CameraGuide } from './CameraGuide';
import { SinglePose } from '../../services/posesService';

interface TunerControlsProps {
  selectedState: string;
  selectedEmotion: string;
  speed: number;
  posesList: SinglePose[];
  activePoseIndex: number;
  isSaving: boolean;
  isPreviewing: boolean;
  onSelectState: (st: string) => void;
  onSelectEmotion: (emo: string) => void;
  onChangeSpeed: (sp: number) => void;
  onSelectPoseIndex: (idx: number) => void;
  onAddPose: () => void;
  onDuplicatePose: () => void;
  onDeletePose: (idx: number) => void;
  onMirrorActivePose: () => void;
  onResetActivePose: () => void;
  onTogglePreview: () => void;
  onSave: () => void;
}

export const TunerControls: Component<TunerControlsProps> = (props) => {
  return (
    <div class="p-4 sm:p-6 bg-gradient-to-r from-theme-card via-theme-surface to-theme-bg border border-theme-border space-y-4 shadow-sm">
      {/* 1. Header Banner */}
      <div class="flex flex-col sm:flex-row sm:items-center justify-between gap-3">
        <div class="flex items-center gap-3">
          <div class="p-2.5 bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/30 shrink-0">
            <Sliders size={22} />
          </div>
          <div>
            <div class="flex items-center gap-2">
              <h2 class="font-heading tracking-tight text-theme-text leading-none">
                Pose & Motion Tuner
              </h2>
              <span class="px-2 py-0.5 text-xs font-mono font-bold bg-theme-bg text-theme-cyan border border-theme-border">
                poses.json
              </span>
            </div>
            <p class="text-xs text-theme-text/60 mt-0.5">
              Multi-pose keyframe editor with natural breathing & dynamic preview
            </p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <span class="text-xs font-mono px-2.5 py-1 bg-theme-card text-theme-cyan border border-theme-border font-semibold">
            State: <strong class="uppercase text-theme-text">{props.selectedState}</strong> ({props.posesList.length} Poses)
          </span>
        </div>
      </div>

      {/* 2. 3D Viewport Navigation & Camera Gestures */}
      <CameraGuide />

      {/* 3. Action Toolbar (Keyframe Management, Mirror, Preview & Save) */}
      <div class="p-2.5 bg-theme-bg border border-theme-border flex flex-col md:flex-row md:items-center justify-between gap-2.5">
        {/* Left Side: Keyframe Tools (Add, Duplicate, Mirror, Reset, Delete) */}
        <div class="flex flex-wrap items-center gap-1.5 sm:gap-2">
          <button
            onClick={props.onAddPose}
            class="px-3 py-1.5 text-xs font-semibold bg-theme-card hover:bg-theme-cyan/20 text-theme-cyan border border-theme-cyan/30 flex items-center gap-1.5 transition-all shadow-sm"
            title="Add new keyframe pose at the end"
          >
            <Plus size={13} />
            <span>Add Pose</span>
          </button>

          <button
            onClick={props.onDuplicatePose}
            class="px-3 py-1.5 text-xs font-semibold bg-theme-card hover:bg-theme-surface text-theme-text border border-theme-border flex items-center gap-1.5 transition-all shadow-sm"
            title="Duplicate this active pose as a new keyframe"
          >
            <Copy size={13} />
            <span>Duplicate</span>
          </button>

          <button
            onClick={props.onMirrorActivePose}
            class="px-3 py-1.5 text-xs font-semibold bg-theme-card hover:bg-theme-surface text-theme-text border border-theme-border flex items-center gap-1.5 transition-all shadow-sm"
            title="Mirror left-right symmetry on active pose"
          >
            <FlipHorizontal size={13} />
            <span>Mirror</span>
          </button>

          <button
            onClick={props.onResetActivePose}
            class="px-3 py-1.5 text-xs font-semibold bg-theme-card hover:bg-theme-rose/20 text-theme-rose border border-theme-rose/30 flex items-center gap-1.5 transition-all shadow-sm"
            title="Reset all bones on active pose to [0, 0, 0]"
          >
            <RotateCcw size={12} />
            <span>Reset Pose</span>
          </button>

          {props.posesList.length > 1 && (
            <button
              onClick={() => props.onDeletePose(props.activePoseIndex)}
              class="px-2.5 py-1.5 text-xs font-semibold bg-theme-card hover:bg-theme-rose text-theme-rose hover:text-white border border-theme-rose/40 flex items-center gap-1 transition-all"
              title="Delete this pose from list"
            >
              <Trash2 size={12} />
              <span>Delete</span>
            </button>
          )}
        </div>

        {/* Right Side: Preview Loop & Save */}
        <div class="flex items-center gap-2 justify-end">
          <button
            onClick={props.onTogglePreview}
            class={`px-3.5 py-1.5 text-xs font-bold flex items-center gap-1.5 transition-all shadow-sm ${
              props.isPreviewing
                ? 'bg-theme-rose text-theme-card anime-shadow-red font-bold'
                : 'bg-theme-card hover:bg-theme-border text-theme-text border border-theme-border'
            }`}
          >
            {props.isPreviewing ? <Square size={13} class="fill-current" /> : <Play size={13} class="fill-current" />}
            <span>{props.isPreviewing ? 'Stop Preview' : `Preview Loop (${props.posesList.length} Poses)`}</span>
          </button>

          <button
            onClick={props.onSave}
            disabled={props.isSaving}
            class="px-4 py-1.5 text-xs font-bold bg-theme-cyan hover:bg-theme-cyan/90 text-theme-card shadow-md anime-shadow-green flex items-center gap-1.5 transition-all"
          >
            <Save size={13} />
            <span>{props.isSaving ? 'Saving...' : 'Save Live'}</span>
          </button>
        </div>
      </div>

      {/* 4. State, Emotion & Speed Bar */}
      <div class="grid grid-cols-1 sm:grid-cols-3 gap-3 pt-2">
        {/* Target State */}
        <div class="space-y-1.5">
          <label class="text-xs font-semibold text-theme-text/70 block h-4">Target State</label>
          <select
            value={props.selectedState}
            onChange={(e) => props.onSelectState(e.currentTarget.value)}
            class="w-full h-[38px] bg-theme-card border border-theme-border px-3 text-xs text-theme-text outline-none focus:border-theme-cyan transition-colors"
          >
            <option value="idle">Idle (Standby)</option>
            <option value="listening">Listening</option>
            <option value="thinking">Thinking</option>
            <option value="speaking">Speaking</option>
          </select>
        </div>

        {/* Emotion Expression */}
        <div class="space-y-1.5">
          <label class="text-xs font-semibold text-theme-text/70 block h-4">Emotion Expression</label>
          <select
            value={props.selectedEmotion}
            onChange={(e) => props.onSelectEmotion(e.currentTarget.value)}
            class="w-full h-[38px] bg-theme-card border border-theme-border px-3 text-xs text-theme-text outline-none focus:border-theme-cyan transition-colors"
          >
            <option value="neutral">Neutral</option>
            <option value="happy">Happy</option>
            <option value="shy">Shy</option>
            <option value="surprised">Surprised</option>
            <option value="thinking">Thinking</option>
            <option value="sad">Sad</option>
            <option value="excited">Excited</option>
            <option value="angry">Angry</option>
          </select>
        </div>

        {/* Cycle Speed */}
        <div class="space-y-1.5">
          <div class="flex items-center justify-between h-4">
            <label class="text-xs font-semibold text-theme-text/70">Animation Cycle Speed</label>
            <span class="text-xs font-mono font-bold text-theme-cyan">{props.speed.toFixed(1)}s</span>
          </div>
          <div class="w-full h-[38px] bg-theme-card border border-theme-border px-3 flex items-center">
            <input 
              type="range" 
              min="0.5" 
              max="5.0" 
              step="0.1" 
              value={props.speed}
              onInput={(e) => props.onChangeSpeed(parseFloat(e.currentTarget.value) || 1.0)}
              class="w-full h-1.5 bg-theme-bg appearance-none cursor-pointer accent-theme-cyan"
            />
          </div>
        </div>
      </div>

      {/* 5. Dynamic Keyframe Pills Selector */}
      <div class="space-y-2 pt-1 border-t border-theme-border/60">
        <div class="flex items-center justify-between">
          <label class="text-xs font-semibold text-theme-text/70">
            Keyframe List ({props.posesList.length} Registered Poses)
          </label>
          <span class="text-xs text-theme-text/50">
            Select a pose below to edit bone sliders in real-time
          </span>
        </div>

        <div class="flex items-center gap-2 overflow-x-auto pb-1 custom-scrollbar">
          {props.posesList.map((pose, idx) => {
            const isActive = () => props.activePoseIndex === idx;
            const label = `Pose ${idx + 1}`;

            return (
              <button
                onClick={() => props.onSelectPoseIndex(idx)}
                class={`px-3.5 py-2 text-xs font-bold whitespace-nowrap border flex items-center gap-2 transition-all ${
                  isActive()
                    ? 'bg-theme-cyan text-theme-card border-theme-cyan shadow-sm anime-shadow-green'
                    : 'bg-theme-card text-theme-text/80 border-theme-border hover:border-theme-cyan/50 hover:text-theme-text'
                }`}
              >
                <span class={`w-4 h-4 rounded-full flex items-center justify-center text-2xs font-mono ${
                  isActive() ? 'bg-theme-card text-theme-cyan' : 'bg-theme-surface text-theme-text/70'
                }`}>
                  {idx + 1}
                </span>
                <span>{label}</span>
              </button>
            );
          })}

          <button
            onClick={props.onAddPose}
            class="px-3 py-2 text-xs font-semibold whitespace-nowrap bg-theme-surface text-theme-cyan border border-dashed border-theme-cyan/40 hover:bg-theme-cyan/10 hover:border-theme-cyan flex items-center gap-1.5 transition-all"
            title="Tambah pose baru"
          >
            <Plus size={13} />
            <span>+ New Pose</span>
          </button>
        </div>
      </div>
    </div>
  );
};
