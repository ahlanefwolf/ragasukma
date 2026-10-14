import { Component } from 'solid-js';
import { RotateCcw } from 'lucide-solid';
import { BONE_CATEGORIES, BONE_LABELS } from './tunerBones';

interface BoneSlidersProps {
  activeCategory: string;
  onSelectCategory: (id: string) => void;
  getBoneRotation: (boneName: string) => [number, number, number];
  onUpdateBoneAxis: (bone: string, axis: 0 | 1 | 2, val: number) => void;
  onResetSingleBone: (bone: string) => void;
  onResetSingleAxis: (bone: string, axis: 0 | 1 | 2) => void;
}

export const BoneSliders: Component<BoneSlidersProps> = (props) => {
  return (
    <div class="space-y-4">
      {/* Category Navigation Pills */}
      <div class="flex items-center gap-2 overflow-x-auto pb-1 custom-scrollbar">
        {BONE_CATEGORIES.map((cat) => {
          const isCurrent = () => props.activeCategory === cat.id;

          return (
            <button
              onClick={() => props.onSelectCategory(cat.id)}
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
          .find((c) => c.id === props.activeCategory)
          ?.bones.map((boneName) => {
            const vals = () => props.getBoneRotation(boneName);

            return (
              <div class="p-4 sm:p-5 bg-theme-surface border border-theme-border space-y-3 shadow-lg">
                <div class="flex items-center justify-between border-b border-theme-border/50 pb-2.5">
                  <div>
                    <h4 class="text-theme-text">
                      {BONE_LABELS[boneName] || boneName}
                    </h4>
                    <span class="text-xs font-mono text-theme-text/40">
                      ID: {boneName}
                    </span>
                  </div>
                  <button
                    onClick={() => props.onResetSingleBone(boneName)}
                    class="px-2.5 py-1 text-xs font-semibold bg-theme-card hover:bg-theme-rose/20 text-theme-rose border border-theme-rose/30 flex items-center gap-1 transition-all"
                    title="Reset this bone to [0, 0, 0]"
                  >
                    <RotateCcw size={11} />
                    <span>Reset</span>
                  </button>
                </div>

                {/* X Axis */}
                <div class="space-y-1">
                  <div class="flex items-center justify-between text-xs">
                    <span class="text-theme-rose font-bold">
                      Rotation X (Pitch)
                    </span>
                    <div class="flex items-center gap-1.5">
                      <input
                        type="number"
                        min="-3.14"
                        max="3.14"
                        step="0.01"
                        value={parseFloat(vals()[0].toFixed(2))}
                        onInput={(e) => props.onUpdateBoneAxis(boneName, 0, parseFloat(e.currentTarget.value) || 0)}
                        class="w-20 bg-theme-card border border-theme-border px-2.5 py-1 text-xs text-right font-mono font-semibold text-theme-text outline-none focus:border-theme-rose transition-colors"
                      />
                      <button
                        onClick={() => props.onResetSingleAxis(boneName, 0)}
                        class="px-2 py-1 text-xs font-bold bg-theme-card hover:bg-theme-rose/30 text-theme-rose border border-theme-border transition-colors"
                        title="Reset X to 0.00"
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
                    onInput={(e) => props.onUpdateBoneAxis(boneName, 0, parseFloat(e.currentTarget.value) || 0)}
                    class="w-full h-1.5 bg-theme-card appearance-none cursor-pointer accent-theme-rose"
                  />
                </div>

                {/* Y Axis */}
                <div class="space-y-1">
                  <div class="flex items-center justify-between text-xs">
                    <span class="text-theme-cyan font-bold">
                      Rotation Y (Yaw)
                    </span>
                    <div class="flex items-center gap-1.5">
                      <input
                        type="number"
                        min="-3.14"
                        max="3.14"
                        step="0.01"
                        value={parseFloat(vals()[1].toFixed(2))}
                        onInput={(e) => props.onUpdateBoneAxis(boneName, 1, parseFloat(e.currentTarget.value) || 0)}
                        class="w-20 bg-theme-card border border-theme-border px-2.5 py-1 text-xs text-right font-mono font-semibold text-theme-text outline-none focus:border-theme-cyan transition-colors"
                      />
                      <button
                        onClick={() => props.onResetSingleAxis(boneName, 1)}
                        class="px-2 py-1 text-xs font-bold bg-theme-card hover:bg-theme-cyan/30 text-theme-cyan border border-theme-border transition-colors"
                        title="Reset Y to 0.00"
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
                    onInput={(e) => props.onUpdateBoneAxis(boneName, 1, parseFloat(e.currentTarget.value) || 0)}
                    class="w-full h-1.5 bg-theme-card appearance-none cursor-pointer accent-theme-cyan"
                  />
                </div>

                {/* Z Axis */}
                <div class="space-y-1">
                  <div class="flex items-center justify-between text-xs">
                    <span class="text-theme-text font-bold">
                      Rotation Z (Roll)
                    </span>
                    <div class="flex items-center gap-1.5">
                      <input
                        type="number"
                        min="-3.14"
                        max="3.14"
                        step="0.01"
                        value={parseFloat(vals()[2].toFixed(2))}
                        onInput={(e) => props.onUpdateBoneAxis(boneName, 2, parseFloat(e.currentTarget.value) || 0)}
                        class="w-20 bg-theme-card border border-theme-border px-2.5 py-1 text-xs text-right font-mono font-semibold text-theme-text outline-none focus:border-theme-text transition-colors"
                      />
                      <button
                        onClick={() => props.onResetSingleAxis(boneName, 2)}
                        class="px-2 py-1 text-xs font-bold bg-theme-card hover:bg-theme-text/30 text-theme-text border border-theme-border transition-colors"
                        title="Reset Z to 0.00"
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
                    onInput={(e) => props.onUpdateBoneAxis(boneName, 2, parseFloat(e.currentTarget.value) || 0)}
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
