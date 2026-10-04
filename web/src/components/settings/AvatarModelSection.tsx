import { Component, createEffect } from 'solid-js';
import { User, RefreshCw, Upload, Image as ImageIcon, CheckCircle2 } from 'lucide-solid';
import { AssetInfo } from '../../api';

interface AvatarModelSectionProps {
  avatarType: string;
  setAvatarType: (val: string) => void;
  assetInfo: AssetInfo | null;
  loadAssetInfoData: () => Promise<void>;
  isUploadingVrm: boolean;
  handleVrmFileChange: (e: Event) => void;
  isUploadingAvatar: boolean;
  selectedAvatarSlot: string;
  setSelectedAvatarSlot: (val: string) => void;
  handleAvatarImageChange: (e: Event) => void;
}

export const AvatarModelSection: Component<AvatarModelSectionProps> = (props) => {
  let avatarTypeSelectRef: HTMLSelectElement | undefined;
  let slotSelectRef: HTMLSelectElement | undefined;

  createEffect(() => {
    if (avatarTypeSelectRef && props.avatarType) {
      avatarTypeSelectRef.value = props.avatarType;
    }
  });

  createEffect(() => {
    if (slotSelectRef && props.selectedAvatarSlot) {
      slotSelectRef.value = props.selectedAvatarSlot;
    }
  });

  return (
    <div class="space-y-5 animate-in fade-in duration-200">
      <div class="p-4 sm:p-5 bg-theme-surface border border-theme-border space-y-5">
        <div class="flex items-center justify-between">
          <h3 class="text-base sm:text-lg font-bold text-theme-cyan flex items-center gap-2">
            <User size={18} />
            Avatar & Model Assets (VRM / 2D)
          </h3>

          <button
            type="button"
            onClick={props.loadAssetInfoData}
            class="text-xs text-theme-text/60 hover:text-theme-cyan flex items-center gap-1.5 transition-colors"
          >
            <RefreshCw size={13} />
            <span>Refresh Info</span>
          </button>
        </div>

        {/* Avatar Type Selector */}
        <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Primary Avatar Type</label>
            <select
              ref={avatarTypeSelectRef}
              value={props.avatarType || 'vrm'}
              onChange={(e) => props.setAvatarType(e.currentTarget.value)}
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            >
              <option value="vrm">VRM 3D Model</option>
              <option value="2d">2D Image / Vector Avatar</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Active Model Status</label>
            <div class="p-2.5 bg-theme-card border border-theme-border flex items-center justify-between text-xs">
              <div class="flex items-center gap-2 truncate">
                <CheckCircle2 size={15} class="text-theme-cyan shrink-0" />
                <span class="text-theme-text font-mono truncate">
                  {props.assetInfo?.vrm_exists ? `char.vrm (${props.assetInfo?.vrm_size_mb.toFixed(2)} MB)` : 'char.vrm not found'}
                </span>
              </div>
              <span class="text-xs px-2 py-0.5 bg-theme-cyan/20 text-theme-cyan font-bold uppercase">
                {props.assetInfo?.vrm_exists ? 'Active' : 'Empty'}
              </span>
            </div>
          </div>
        </div>

        {/* VRM 3D Model Upload Card */}
        <div class="p-4 bg-theme-card border border-theme-border space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-sm font-bold text-theme-text flex items-center gap-2">
              <Upload size={16} class="text-theme-cyan" />
              Upload 3D VRM Model (.vrm / .glb)
            </span>
            <span class="text-xs font-mono text-theme-cyan bg-theme-cyan/10 border border-theme-cyan/20 px-2 py-0.5">
              Requires .vrm / .glb
            </span>
          </div>

          <p class="text-xs text-theme-text/70 leading-relaxed">
            Supported formats: <strong class="text-theme-cyan">.vrm</strong> (VRM 0.x/1.0 standard from VRoid Studio / Blender) or <strong class="text-theme-cyan">.glb</strong>. Files are saved to <code class="text-theme-cyan font-mono">assets/char.vrm</code> and hot-reloaded automatically.
          </p>

          <div class="flex items-center gap-3 pt-1">
            <label class="relative cursor-pointer">
              <input
                type="file"
                accept=".vrm,.glb"
                disabled={props.isUploadingVrm}
                onChange={props.handleVrmFileChange}
                class="sr-only"
              />
              <span class={`inline-flex items-center gap-2 px-4 py-2.5 font-bold text-xs transition-all ${
                props.isUploadingVrm
                  ? 'bg-theme-border text-theme-text/50 cursor-not-allowed'
                  : 'bg-theme-cyan hover:bg-[#4a5a3a] text-theme-card shadow anime-shadow-green cursor-pointer'
              }`}>
                <Upload size={14} />
                <span>{props.isUploadingVrm ? 'Uploading & Loading...' : 'Select VRM File (.vrm / .glb)'}</span>
              </span>
            </label>

            {props.assetInfo?.vrm_exists && (
              <span class="text-xs text-theme-text/60 font-mono">
                Size: {props.assetInfo?.vrm_size_mb.toFixed(2)} MB
              </span>
            )}
          </div>
        </div>

        {/* 2D Image Avatar Upload Card */}
        <div class="p-4 bg-theme-card border border-theme-border space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-sm font-bold text-theme-text flex items-center gap-2">
              <ImageIcon size={16} class="text-theme-rose" />
              Upload 2D Avatar Images (State-Based)
            </span>
            <span class="text-xs font-mono text-theme-rose bg-theme-rose/10 border border-theme-rose/20 px-2 py-0.5">
              Format: Transparent PNG / JPG
            </span>
          </div>

          <div class="p-2.5 bg-theme-bg border border-theme-border text-xs text-theme-text/80 leading-relaxed space-y-1">
            <div class="flex items-center gap-1.5 font-bold text-theme-rose">
              <span>Format & Image Guidelines:</span>
            </div>
            <ul class="list-disc list-inside space-y-0.5 text-xs text-theme-text/70">
              <li><strong class="text-theme-text">Transparent PNG (Recommended)</strong> so character renders without background bounding box.</li>
              <li>Supports <strong class="text-theme-text">.png</strong>, <strong class="text-theme-text">.jpg</strong>, <strong class="text-theme-text">.jpeg</strong> files (Recommended min 512x512 px).</li>
              <li>Each expression slot (<code class="font-mono text-theme-cyan">idle</code>, <code class="font-mono text-theme-cyan">listening</code>, <code class="font-mono text-theme-cyan">speaking</code>, <code class="font-mono text-theme-cyan">thinking</code>) can have its own distinct image.</li>
            </ul>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div class="space-y-1.5">
              <label class="text-xs font-semibold text-theme-text/80">Select Expression Slot</label>
              <select
                ref={slotSelectRef}
                value={props.selectedAvatarSlot || 'idle'}
                onChange={(e) => props.setSelectedAvatarSlot(e.currentTarget.value)}
                class="w-full bg-theme-surface border border-theme-border px-3 py-2 text-xs text-theme-text outline-none focus:border-theme-rose transition-colors"
              >
                <option value="idle">Idle (Normal)</option>
                <option value="listening">Listening</option>
                <option value="speaking">Speaking</option>
                <option value="thinking">Thinking</option>
              </select>
            </div>

            <div class="space-y-1.5">
              <label class="text-xs font-semibold text-theme-text/80">Select File (.png / .jpg / .jpeg)</label>
              <label class="relative block cursor-pointer">
                <input
                  type="file"
                  accept="image/png,image/jpeg,image/jpg"
                  disabled={props.isUploadingAvatar}
                  onChange={props.handleAvatarImageChange}
                  class="sr-only"
                />
                <span class={`inline-flex w-full items-center justify-center gap-2 px-4 py-2 font-bold text-xs transition-all ${
                  props.isUploadingAvatar
                    ? 'bg-theme-border text-theme-text/50 cursor-not-allowed'
                    : 'bg-theme-rose hover:bg-[#997539] text-theme-card shadow cursor-pointer'
                }`}>
                  <Upload size={14} />
                  <span>{props.isUploadingAvatar ? 'Uploading...' : `Upload PNG/JPG for '${props.selectedAvatarSlot}'`}</span>
                </span>
              </label>
            </div>
          </div>

          {props.assetInfo?.images && props.assetInfo!.images.length > 0 && (
            <div class="pt-2 border-t border-theme-border/60">
              <span class="text-xs text-theme-text/60 block mb-1.5">Image files stored in assets/:</span>
              <div class="flex flex-wrap gap-2">
                {props.assetInfo!.images.map((imgName) => (
                  <span class="text-xs font-mono px-2 py-1 bg-theme-surface border border-theme-border text-theme-cyan">
                    {imgName}
                  </span>
                ))}
              </div>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
