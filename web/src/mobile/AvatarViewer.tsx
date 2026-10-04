import { Component, Show, createSignal, onMount, onCleanup } from 'solid-js';
import { API_BASE } from '../services/base';
import {
  AvatarViewerProps,
  PosesConfig,
  VrmSceneManager,
  AvatarCameraControls,
  AvatarLoadingOverlay,
  Avatar2DFallback,
} from './avatar';

export type { AvatarViewerProps };

export const AvatarViewer: Component<AvatarViewerProps> = (props) => {
  let canvasContainerRef: HTMLDivElement | undefined;

  const [avatarType, setAvatarType] = createSignal<'vrm' | '2d'>('vrm');
  const [isLoading, setIsLoading] = createSignal(true);
  const [loadingProgress, setLoadingProgress] = createSignal<number>(0);
  const [isVrmLoaded, setIsVrmLoaded] = createSignal(false);
  const [vrmLoadError, setVrmLoadError] = createSignal(false);
  const [posesConfig, setPosesConfig] = createSignal<PosesConfig | null>(null);
  const [isFullscreen, setIsFullscreen] = createSignal(false);

  const sceneManager = new VrmSceneManager();

  const toggleFullscreen = () => {
    if (!document.fullscreenElement) {
      document.documentElement.requestFullscreen?.().catch(() => {});
      setIsFullscreen(true);
    } else {
      document.exitFullscreen?.().catch(() => {});
      setIsFullscreen(false);
    }
  };

  const startVrmScene = () => {
    if (!canvasContainerRef) return;

    sceneManager.init(
      canvasContainerRef,
      () => ({
        state: props.state || 'idle',
        emotion: props.emotion,
        poses: posesConfig(),
      }),
      {
        onLoadStart: () => {
          setIsLoading(true);
          setVrmLoadError(false);
        },
        onProgress: (percent) => {
          setLoadingProgress(percent);
        },
        onLoaded: () => {
          setIsVrmLoaded(true);
          setIsLoading(false);
        },
        onError: () => {
          setVrmLoadError(true);
          setIsLoading(false);
        },
      }
    );
  };

  onMount(() => {
    // 1. Inisialisasi canvas 3D langsung tanpa menunggu network fetch selesai
    setTimeout(() => {
      startVrmScene();
    }, 50);

    // 2. Fetch poses.json secara paralel di background
    fetch(`${API_BASE}/api/poses`)
      .then((res) => (res.ok ? res.json() : null))
      .then((p) => {
        if (p) setPosesConfig(p);
      })
      .catch(() => {});

    // 3. Periksa avatar type dari config secara paralel
    fetch(`${API_BASE}/api/config`)
      .then((res) => (res.ok ? res.json() : null))
      .then((cfg) => {
        if (cfg && cfg.avatar_type === '2d') {
          setAvatarType('2d');
          setIsLoading(false);
        }
      })
      .catch(() => {});

    // Sinkronisasi status fullscreen dari browser event
    const onFullscreenChange = () => {
      setIsFullscreen(!!document.fullscreenElement);
    };
    document.addEventListener('fullscreenchange', onFullscreenChange);
    onCleanup(() => {
      document.removeEventListener('fullscreenchange', onFullscreenChange);
    });
  });

  onCleanup(() => {
    sceneManager.cleanup();
  });

  return (
    <div class="relative w-full h-full flex items-center justify-center overflow-hidden select-none touch-none">
      {/* Vertical Camera Control Toolbar aligned with Settings */}
      <Show when={avatarType() === 'vrm' && isVrmLoaded()}>
        <AvatarCameraControls
          isFullscreen={isFullscreen()}
          onZoomIn={() => sceneManager.zoomIn()}
          onZoomOut={() => sceneManager.zoomOut()}
          onToggleFullscreen={toggleFullscreen}
          onResetCamera={() => sceneManager.resetCameraView()}
        />
      </Show>

      {/* Loading Overlay When VRM Model is Downloading */}
      <Show when={isLoading() && avatarType() === 'vrm'}>
        <AvatarLoadingOverlay progress={loadingProgress()} />
      </Show>

      {/* 3D WebGL Canvas Container (Full Screen) */}
      <div
        ref={canvasContainerRef}
        class="w-full h-full relative z-10 flex items-center justify-center touch-none"
        style={{ display: avatarType() === 'vrm' && !vrmLoadError() ? 'flex' : 'none' }}
      />

      {/* 2D Sprite fallback if 2D mode or on fatal VRM load error */}
      <Show when={avatarType() === '2d' || vrmLoadError()}>
        <Avatar2DFallback
          state={props.state || 'idle'}
          hasError={vrmLoadError()}
          onRetry={() => {
            setVrmLoadError(false);
            setIsLoading(true);
            startVrmScene();
          }}
        />
      </Show>
    </div>
  );
};
