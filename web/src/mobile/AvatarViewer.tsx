import { Component, Show, createSignal, createEffect, onMount, onCleanup } from 'solid-js';
import { API_BASE } from '../services/base';
import {
  AvatarViewerProps,
  FpsLimit,
  PosesConfig,
  RenderQuality,
  VrmSceneManager,
  AvatarCameraControls,
  AvatarLoadingOverlay,
  Avatar2DFallback,
} from './avatar';

export type { AvatarViewerProps };

export const AvatarViewer: Component<AvatarViewerProps> = (props) => {
  let rootContainerRef: HTMLDivElement | undefined;
  let canvasContainerRef: HTMLDivElement | undefined;

  const [avatarType, setAvatarType] = createSignal<'vrm' | '2d'>('vrm');
  const [isLoading, setIsLoading] = createSignal(true);
  const [loadingProgress, setLoadingProgress] = createSignal<number>(0);
  const [isVrmLoaded, setIsVrmLoaded] = createSignal(false);
  const [vrmLoadError, setVrmLoadError] = createSignal(false);
  const [posesConfig, setPosesConfig] = createSignal<PosesConfig | null>(null);
  const [isFullscreen, setIsFullscreen] = createSignal(false);
  const [renderQuality, setRenderQuality] = createSignal<RenderQuality>(
    (typeof localStorage !== 'undefined' && localStorage.getItem('avatar_render_quality') as RenderQuality) || 'hd'
  );
  const [fpsLimit, setFpsLimit] = createSignal<FpsLimit>(
    (typeof localStorage !== 'undefined' && Number(localStorage.getItem('avatar_fps_limit')) as FpsLimit) || 60
  );

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

  const handleCycleQuality = () => {
    const order: RenderQuality[] = ['sd', 'hd', 'ultra'];
    const nextIdx = (order.indexOf(renderQuality()) + 1) % order.length;
    const nextQ = order[nextIdx];
    setRenderQuality(nextQ);
    sceneManager.setRenderQuality(nextQ);
    try {
      localStorage.setItem('avatar_render_quality', nextQ);
    } catch (_) {}
  };

  const handleCycleFps = () => {
    const order: FpsLimit[] = [60, 30, 0];
    const nextIdx = (order.indexOf(fpsLimit()) + 1) % order.length;
    const nextFps = order[nextIdx];
    setFpsLimit(nextFps);
    sceneManager.setFpsLimit(nextFps);
    try {
      localStorage.setItem('avatar_fps_limit', String(nextFps));
    } catch (_) {}
  };

  createEffect(() => {
    sceneManager.setPaused(!!props.paused);
  });

  onMount(() => {
    // 1. Initialize 3D canvas immediately without blocking on network fetch
    setTimeout(() => {
      startVrmScene();
      sceneManager.setRenderQuality(renderQuality());
      sceneManager.setFpsLimit(fpsLimit());
    }, 50);

    // 2. Fetch poses.json in background
    fetch(`${API_BASE}/api/poses`)
      .then((res) => (res.ok ? res.json() : null))
      .then((p) => {
        if (p) setPosesConfig(p);
      })
      .catch(() => {});

    // 3. Check avatar type from config in parallel
    fetch(`${API_BASE}/api/config`)
      .then((res) => (res.ok ? res.json() : null))
      .then((cfg) => {
        if (cfg && cfg.avatar_type === '2d') {
          setAvatarType('2d');
          setIsLoading(false);
        }
      })
      .catch(() => {});

    // Synchronize fullscreen state from browser events
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
    <div
      ref={rootContainerRef}
      class="relative w-full h-full flex items-center justify-center overflow-hidden select-none touch-none"
    >
      {/* Sleek Floating Controls (Fullscreen) */}
      <Show when={avatarType() === 'vrm' && isVrmLoaded()}>
        <AvatarCameraControls
          isFullscreen={isFullscreen()}
          quality={renderQuality()}
          fpsLimit={fpsLimit()}
          onZoomIn={() => sceneManager.zoomIn()}
          onZoomOut={() => sceneManager.zoomOut()}
          onResetCamera={() => sceneManager.resetCameraView()}
          onCycleQuality={handleCycleQuality}
          onCycleFps={handleCycleFps}
          onToggleFullscreen={toggleFullscreen}
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
        style={{
          display: avatarType() === 'vrm' && !vrmLoadError() ? 'flex' : 'none',
        }}
      />

      {/* Cinematic Portrait Depth Blur */}
      <Show when={avatarType() === 'vrm' && !vrmLoadError()}>
        <div
          class="absolute inset-0 z-15 pointer-events-none transition-opacity duration-500"
          style={{
            'backdrop-filter': 'blur(3px)',
            '-webkit-backdrop-filter': 'blur(3px)',
            'mask-image': 'radial-gradient(ellipse 65% 75% at center, transparent 38%, black 100%)',
            '-webkit-mask-image': 'radial-gradient(ellipse 65% 75% at center, transparent 38%, black 100%)',
            background: 'radial-gradient(ellipse 70% 80% at center, transparent 45%, rgba(0, 0, 0, 0.2) 100%)',
          }}
        />
      </Show>

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
