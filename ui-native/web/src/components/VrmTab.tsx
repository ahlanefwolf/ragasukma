import { Component, createSignal } from 'solid-js';
import { 
  Upload, 
  Check, 
  FileBox, 
  Layers, 
  Sparkles, 
  ShieldCheck,
  RefreshCw,
  FolderOpen
} from 'lucide-solid';

export const VrmTab: Component = () => {
  const [activeModel, setActiveModel] = createSignal('char.vrm');
  const [isUploading, setIsUploading] = createSignal(false);
  const [uploadSuccess, setUploadSuccess] = createSignal(false);

  const availableModels = [
    {
      id: 'char.vrm',
      name: 'Ragasukma Default Anime Girl',
      format: 'VRM 1.0 (GLTF 2.0)',
      size: '24.8 MB',
      joints: '137 Humanoid Bones',
      materials: '15 MToon Layers',
      isDefault: true
    },
    {
      id: 'maid_vroid.vrm',
      name: 'Maid Costume VRoid Edition',
      format: 'VRM 0.X Standard',
      size: '32.1 MB',
      joints: '142 Humanoid Bones',
      materials: '18 MToon Layers',
      isDefault: false
    }
  ];

  const handleFileUpload = (e: Event) => {
    const target = e.target as HTMLInputElement;
    if (!target.files || target.files.length === 0) return;

    const file = target.files[0];
    setIsUploading(true);

    // Simulasi transfer model ke AppData / assets
    setTimeout(() => {
      setIsUploading(false);
      setUploadSuccess(true);
      setTimeout(() => setUploadSuccess(false), 3000);
    }, 1500);
  };

  return (
    <div class="space-y-6">
      {/* Header Banner */}
      <div class="rounded-2xl bg-gradient-to-r from-[#282832] via-[#202028] to-[#1E1E24] p-6 border border-[#3B3B48] flex items-center justify-between">
        <div>
          <span class="px-2.5 py-1 rounded-full text-xs font-bold uppercase tracking-wider bg-[#91AE6E]/20 text-[#91AE6E] border border-[#91AE6E]/30">
            Universal VRM Model Manager
          </span>
          <h2 class="text-3xl font-bangers tracking-wide text-[#F2F2F2] mt-2">
            PENGATURAN KARAKTER 3D (VRM / GLB)
          </h2>
          <p class="text-sm text-[#F2F2F2]/70 mt-1 max-w-xl">
            Ganti model avatar VRoid Studio atau karakter VRM custom lainnya. Engine WGPU akan secara otomatis menyesuaikan auto-framing 9:16 dan MToon shader.
          </p>
        </div>
      </div>

      {/* Upload Drag & Drop Area */}
      <div class="p-6 rounded-2xl bg-[#1E1E24] border-2 border-dashed border-[#3B3B48] hover:border-[#D96868] transition-all text-center relative group">
        <input 
          type="file" 
          accept=".vrm,.glb" 
          onChange={handleFileUpload}
          class="absolute inset-0 w-full h-full opacity-0 cursor-pointer z-10"
        />
        <div class="flex flex-col items-center justify-center space-y-3 pointer-events-none">
          <div class="w-14 h-14 rounded-2xl bg-[#282832] group-hover:bg-[#D96868]/20 flex items-center justify-center text-[#D96868] transition-all group-hover:scale-110">
            {isUploading() ? (
              <RefreshCw size={28} class="animate-spin text-[#689D4B]" />
            ) : uploadSuccess() ? (
              <Check size={28} class="text-[#689D4B]" />
            ) : (
              <Upload size={28} />
            )}
          </div>
          <div>
            <h4 class="font-bangers text-lg text-[#F2F2F2]">
              {isUploading() 
                ? 'MENGUNGGAH DAN MENGANALISIS MODEL...' 
                : uploadSuccess() 
                ? 'MODEL BERHASIL DIMUAT KE ENGINE!' 
                : 'DRAG & DROP FILE .VRM ATAU .GLB DI SINI'}
            </h4>
            <p class="text-xs text-[#F2F2F2]/60 mt-0.5">
              Mendukung VRM 0.X, VRM 1.0 (VRoid Studio, Booth.pm, Blender UniVRM)
            </p>
          </div>
        </div>
      </div>

      {/* Model List Cards */}
      <div class="space-y-3">
        <h3 class="text-lg font-bangers text-[#91AE6E]">
          MODEL TERSIMPAN DI SISTEM
        </h3>

        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
          {availableModels.map((model) => {
            const isCurrent = () => activeModel() === model.id;

            return (
              <div 
                class={`p-5 rounded-2xl border transition-all ${
                  isCurrent()
                    ? 'bg-[#282832] border-[#689D4B] ring-2 ring-[#689D4B]/30 shadow-lg'
                    : 'bg-[#1E1E24] border-[#3B3B48] hover:border-[#F2F2F2]/30'
                }`}
              >
                <div class="flex items-start justify-between">
                  <div class="flex items-center gap-3">
                    <div class="w-10 h-10 rounded-xl bg-[#181820] flex items-center justify-center text-[#91AE6E] border border-[#3B3B48]">
                      <FileBox size={20} />
                    </div>
                    <div>
                      <h4 class="font-bold text-sm text-[#F2F2F2] leading-tight">
                        {model.name}
                      </h4>
                      <span class="text-xs text-[#91AE6E] font-mono">
                        {model.id}
                      </span>
                    </div>
                  </div>

                  {isCurrent() ? (
                    <span class="px-2.5 py-1 rounded-full text-[10px] font-bold uppercase bg-[#689D4B] text-white flex items-center gap-1">
                      <Check size={12} class="stroke-[3]" />
                      Aktif di Overlay
                    </span>
                  ) : (
                    <button
                      onClick={() => setActiveModel(model.id)}
                      class="px-3 py-1 rounded-lg text-xs font-semibold bg-[#282832] hover:bg-[#D96868] text-[#F2F2F2] hover:text-white transition-all border border-[#3B3B48]"
                    >
                      Pilih Model
                    </button>
                  )}
                </div>

                {/* Specs Pill List */}
                <div class="grid grid-cols-2 gap-2 mt-4 pt-3 border-t border-[#3B3B48]/60 text-xs">
                  <div class="flex items-center gap-1.5 text-[#F2F2F2]/70">
                    <ShieldCheck size={14} class="text-[#91AE6E]" />
                    <span>{model.format}</span>
                  </div>
                  <div class="flex items-center gap-1.5 text-[#F2F2F2]/70">
                    <Layers size={14} class="text-[#D96868]" />
                    <span>{model.materials}</span>
                  </div>
                </div>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
};
