import { Component, Show } from 'solid-js';
import { ModelDownloadStatus } from '../services/eventsService';
import { Download, AlertCircle, RefreshCw, Cpu } from 'lucide-solid';

interface ModelLoadingOverlayProps {
  modelStatus: ModelDownloadStatus | null;
  onRetry: () => void;
}

export const ModelLoadingOverlay: Component<ModelLoadingOverlayProps> = (props) => {
  const isError = () => props.modelStatus?.status === 'error';
  const isExtracting = () => props.modelStatus?.status === 'extracting';
  const percent = () => Math.min(100, Math.max(0, props.modelStatus?.percent || 0));
  const downloadedMb = () => ((props.modelStatus?.downloaded_bytes || 0) / (1024 * 1024)).toFixed(1);
  const totalMb = () => ((props.modelStatus?.total_bytes || 0) / (1024 * 1024)).toFixed(1);
  const speed = () => (props.modelStatus?.speed_mbps || 0).toFixed(2);

  return (
    <div class="fixed inset-0 z-[99999] bg-theme-bg/85 backdrop-blur-md flex items-center justify-center p-4 select-none animate-fade-in">
      <div class="w-full max-w-md bg-theme-surface border border-theme-border shadow-2xl p-6 sm:p-8 flex flex-col items-center text-center relative overflow-hidden text-theme-text">
        {/* Glow palette accents */}
        <div class="absolute -top-12 -left-12 w-32 h-32 bg-theme-cyan/15 blur-3xl rounded-full pointer-events-none" />
        <div class="absolute -bottom-12 -right-12 w-32 h-32 bg-theme-rose/15 blur-3xl rounded-full pointer-events-none" />

        {/* Icon Header */}
        <div class="relative mb-4">
          <Show
            when={!isError()}
            fallback={
              <div class="w-14 h-14 bg-theme-rose/20 border border-theme-rose/60 flex items-center justify-center text-theme-rose animate-bounce">
                <AlertCircle class="w-7 h-7" />
              </div>
            }
          >
            <div class="w-14 h-14 bg-theme-cyan/20 border border-theme-cyan/60 flex items-center justify-center text-theme-cyan relative">
              <Show
                when={isExtracting()}
                fallback={<Download class="w-7 h-7 animate-pulse" />}
              >
                <Cpu class="w-7 h-7 animate-spin" />
              </Show>
              <div class="absolute inset-0 border border-theme-cyan/40 animate-ping opacity-25" />
            </div>
          </Show>
        </div>

        {/* Title */}
        <h2 class="text-base sm:text-lg font-bold text-theme-text mb-1 tracking-wide font-mono">
          <Show
            when={!isError()}
            fallback="MODEL INITIALIZATION FAILED"
          >
            <Show when={isExtracting()} fallback="PREPARING VOICE SYNTHESIS">
              EXTRACTING SUPERTONIC-3 MODEL
            </Show>
          </Show>
        </h2>

        {/* Description */}
        <p class="text-xs text-theme-text/80 mb-6 max-w-sm leading-relaxed">
          <Show
            when={!isError()}
            fallback={props.modelStatus?.message || 'Failed to download Supertonic-3 neural weights.'}
          >
            <Show
              when={isExtracting()}
              fallback="Downloading Supertonic-3 ONNX neural weights for high-fidelity speech synthesis."
            >
              Unpacking ONNX model files and studio voice profiles. Please wait...
            </Show>
          </Show>
        </p>

        {/* Progress Bar (Visible if not error) */}
        <Show when={!isError()}>
          <div class="w-full bg-theme-bg border border-theme-border h-3.5 mb-3 p-0.5 overflow-hidden shadow-inner">
            <div
              class="h-full bg-theme-cyan transition-all duration-300 ease-out relative"
              style={{ width: `${percent()}%` }}
            >
              <div class="absolute inset-0 bg-[#F5F0E6]/25 animate-pulse" />
            </div>
          </div>

          {/* Stats info */}
          <div class="w-full flex items-center justify-between text-xs font-mono text-theme-text/80 mb-6 px-1">
            <span>
              {downloadedMb()} MB / {props.modelStatus?.total_bytes ? `${totalMb()} MB` : '...'}
            </span>
            <span class="text-theme-cyan font-bold">{percent().toFixed(0)}%</span>
            <span>
              <Show when={!isExtracting() && props.modelStatus?.speed_mbps}>
                {speed()} MB/s
              </Show>
              <Show when={isExtracting()}>
                Extracting...
              </Show>
            </span>
          </div>
        </Show>

        {/* Action Button if Error */}
        <Show when={isError()}>
          <button
            onClick={() => props.onRetry()}
            class="px-5 py-2.5 bg-theme-rose hover:bg-theme-rose/90 text-[#F5F0E6] font-bold text-xs tracking-wider uppercase flex items-center gap-2 border border-theme-rose transition-all shadow-md active:scale-95 cursor-pointer"
          >
            <RefreshCw class="w-4 h-4" />
            Retry Download
          </button>
        </Show>

        <Show when={!isError()}>
          <div class="text-[11px] text-theme-text/60 tracking-wider uppercase font-mono mt-1">
            Navigation and chat locked until ready
          </div>
        </Show>
      </div>
    </div>
  );
};
