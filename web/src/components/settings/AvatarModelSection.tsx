import { Component } from 'solid-js';
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
            <label class="text-xs font-semibold text-theme-text/80">Tipe Avatar Utama</label>
            <select
              value={props.avatarType}
              onChange={(e) => props.setAvatarType(e.currentTarget.value)}
              class="w-full bg-theme-card border border-theme-border px-3.5 py-2.5 text-sm text-theme-text outline-none focus:border-theme-cyan transition-colors"
            >
              <option value="vrm">VRM 3D Model</option>
              <option value="2d">2D Image / Vector Avatar</option>
            </select>
          </div>

          <div class="space-y-1.5">
            <label class="text-xs font-semibold text-theme-text/80">Status Model Aktif</label>
            <div class="p-2.5 bg-theme-card border border-theme-border flex items-center justify-between text-xs">
              <div class="flex items-center gap-2 truncate">
                <CheckCircle2 size={15} class="text-theme-cyan shrink-0" />
                <span class="text-theme-text font-mono truncate">
                  {props.assetInfo?.vrm_exists ? `char.vrm (${props.assetInfo?.vrm_size_mb.toFixed(2)} MB)` : 'char.vrm tidak ditemukan'}
                </span>
              </div>
              <span class="text-xs px-2 py-0.5 bg-theme-cyan/20 text-theme-cyan font-bold uppercase">
                {props.assetInfo?.vrm_exists ? 'Aktif' : 'Kosong'}
              </span>
            </div>
          </div>
        </div>

        {/* VRM 3D Model Upload Card */}
        <div class="p-4 bg-theme-card border border-theme-border space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-sm font-bold text-theme-text flex items-center gap-2">
              <Upload size={16} class="text-theme-cyan" />
              Upload Model VRM 3D (.vrm / .glb)
            </span>
            <span class="text-xs font-mono text-theme-cyan bg-theme-cyan/10 border border-theme-cyan/20 px-2 py-0.5">
              Wajib .vrm / .glb
            </span>
          </div>

          <p class="text-xs text-theme-text/70 leading-relaxed">
            Format yang didukung: <strong class="text-theme-cyan">.vrm</strong> (standar VRM 0.x/1.0 dari VRoid Studio / Blender) atau <strong class="text-theme-cyan">.glb</strong>. File otomatis disimpan ke <code class="text-theme-cyan font-mono">assets/char.vrm</code> dan langsung dimuat ulang (hot-reload).
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
                <span>{props.isUploadingVrm ? 'Mengunggah & Memuat...' : 'Pilih File VRM (.vrm / .glb)'}</span>
              </span>
            </label>

            {props.assetInfo?.vrm_exists && (
              <span class="text-xs text-theme-text/60 font-mono">
                Ukuran: {props.assetInfo?.vrm_size_mb.toFixed(2)} MB
              </span>
            )}
          </div>
        </div>

        {/* 2D Image Avatar Upload Card */}
        <div class="p-4 bg-theme-card border border-theme-border space-y-3">
          <div class="flex items-center justify-between">
            <span class="text-sm font-bold text-theme-text flex items-center gap-2">
              <ImageIcon size={16} class="text-theme-rose" />
              Upload Gambar Avatar 2D (State-Based)
            </span>
            <span class="text-xs font-mono text-theme-rose bg-theme-rose/10 border border-theme-rose/20 px-2 py-0.5">
              Format: PNG (Transparan) / JPG
            </span>
          </div>

          <div class="p-2.5 bg-theme-bg border border-theme-border text-xs text-theme-text/80 leading-relaxed space-y-1">
            <div class="flex items-center gap-1.5 font-bold text-theme-rose">
              <span>Ketentuan Format & Gambar:</span>
            </div>
            <ul class="list-disc list-inside space-y-0.5 text-xs text-theme-text/70">
              <li><strong class="text-theme-text">PNG Transparan (Disarankan)</strong> agar karakter tampil tanpa kotak background.</li>
              <li>Mendukung file <strong class="text-theme-text">.png</strong>, <strong class="text-theme-text">.jpg</strong>, <strong class="text-theme-text">.jpeg</strong> (Resolusi disarankan min 512x512 px).</li>
              <li>Setiap slot ekspresi (<code class="font-mono text-theme-cyan">idle</code>, <code class="font-mono text-theme-cyan">listening</code>, <code class="font-mono text-theme-cyan">talking</code>, <code class="font-mono text-theme-cyan">thinking</code>) dapat memiliki gambar tersendiri.</li>
            </ul>
          </div>

          <div class="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div class="space-y-1.5">
              <label class="text-xs font-semibold text-theme-text/80">Pilih Slot Ekspresi</label>
              <select
                value={props.selectedAvatarSlot}
                onChange={(e) => props.setSelectedAvatarSlot(e.currentTarget.value)}
                class="w-full bg-theme-surface border border-theme-border px-3 py-2 text-xs text-theme-text outline-none focus:border-theme-rose transition-colors"
              >
                <option value="idle">Idle (Diam / Normal)</option>
                <option value="listening">Listening (Mendengarkan)</option>
                <option value="talking">Talking / Speaking (Bicara)</option>
                <option value="thinking">Thinking (Berpikir)</option>
              </select>
            </div>

            <div class="space-y-1.5">
              <label class="text-xs font-semibold text-theme-text/80">Pilih File (.png / .jpg / .jpeg)</label>
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
                  <span>{props.isUploadingAvatar ? 'Mengunggah...' : `Unggah PNG/JPG untuk '${props.selectedAvatarSlot}'`}</span>
                </span>
              </label>
            </div>
          </div>

          {props.assetInfo?.images && props.assetInfo!.images.length > 0 && (
            <div class="pt-2 border-t border-theme-border/60">
              <span class="text-xs text-theme-text/60 block mb-1.5">File gambar tersimpan di assets/:</span>
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
