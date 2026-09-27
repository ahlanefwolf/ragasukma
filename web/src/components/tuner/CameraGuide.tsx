import { Component } from 'solid-js';
import { 
  Move, 
  RotateCw, 
  MousePointer, 
  ZoomIn, 
  RotateCcw, 
  Compass 
} from 'lucide-solid';

export const CameraGuide: Component = () => {
  return (
    <div class="bg-theme-bg p-3 border border-theme-border space-y-2">
      <div class="flex items-center justify-between">
        <h3 class="text-sm font-bold text-theme-cyan flex items-center gap-1.5">
          <Compass size={14} />
          Panduan Navigasi Layar Avatar
        </h3>
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
  );
};
