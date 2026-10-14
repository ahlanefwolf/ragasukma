import {
  globalStatus,
  currentTime,
  duration,
  isPlaying,
  pauseMusic,
  resumeMusic,
  nextTrack,
  prevTrack,
  seekMusic,
  formatDuration,
  onPlayStateChange,
} from '../playerStore';

const GI_THEME = {
  canvas: '#0F1218',
  surface: '#181D26',
  surfaceElevated: '#222936',
  borderSubtle: '#333C4D',
  borderGold: '#9E8046',
  textPrimary: '#EDEAE1',
  textSecondary: '#9FA7B8',
  textMuted: '#5F6675',
  primary: '#E5B85C',
  primaryHover: '#F3CE7A',
  danger: '#E05345',
} as const;

class PipManager {
  private canvas: HTMLCanvasElement | null = null;
  private ctx: CanvasRenderingContext2D | null = null;
  private video: HTMLVideoElement | null = null;
  private timerId: any = null;
  private workerTimer: Worker | null = null;
  private isPipActive = false;
  private currentCoverImage: HTMLImageElement | null = null;
  private lastThumbnailUrl = '';
  private isSyncing = false;

  init() {
    if (this.canvas) return;

    this.canvas = document.createElement('canvas');
    // Render on buffer 2x HiDPI / Retina so that font, border, and image super tajam (HD)
    const dpr = 2;
    this.canvas.width = 680 * dpr;
    this.canvas.height = 210 * dpr;
    this.ctx = this.canvas.getContext('2d', { alpha: false });

    this.video = document.createElement('video');
    this.video.muted = true;
    this.video.autoplay = true;
    this.video.playsInline = true;
    // Attach video to DOM invisibly so the browser does not throttle it
    this.video.style.position = 'fixed';
    this.video.style.width = '1px';
    this.video.style.height = '1px';
    this.video.style.opacity = '0';
    this.video.style.pointerEvents = 'none';
    this.video.style.top = '-9999px';
    document.body.appendChild(this.video);

    // Synchronize PiP status when user manually closes the PiP window
    this.video.addEventListener('leavepictureinpicture', () => {
      this.isPipActive = false;
      this.stopRenderLoop();
    });

    // Synchronize when user clicks Play/Pause in floating PiP window
    this.video.addEventListener('pause', () => {
      if (this.isSyncing) return;
      if (isPlaying()) {
        this.isSyncing = true;
        pauseMusic();
        setTimeout(() => { this.isSyncing = false; }, 100);
      }
    });

    this.video.addEventListener('play', () => {
      if (this.isSyncing) return;
      if (!isPlaying()) {
        this.isSyncing = true;
        resumeMusic();
        setTimeout(() => { this.isSyncing = false; }, 100);
      }
    });

    // Registrasi MediaSession API bawaan browser OS
    if ('mediaSession' in navigator) {
      navigator.mediaSession.setActionHandler('play', () => {
        resumeMusic();
      });
      navigator.mediaSession.setActionHandler('pause', () => {
        pauseMusic();
      });
      navigator.mediaSession.setActionHandler('previoustrack', () => {
        prevTrack();
      });
      navigator.mediaSession.setActionHandler('nexttrack', () => {
        nextTrack();
      });
      try {
        navigator.mediaSession.setActionHandler('seekto', (details) => {
          if (details.seekTime !== undefined && details.seekTime !== null) {
            seekMusic(details.seekTime);
          }
        });
      } catch (_) {}
    }
  }

  // Dipanggil direct from playerStore when audio play
  onAudioPlay() {
    this.updateMediaSession(true);
    if (this.video && this.video.paused && !this.isSyncing) {
      this.isSyncing = true;
      this.video.play().catch(() => {}).finally(() => {
        this.isSyncing = false;
      });
    }
  }

  // Dipanggil direct from playerStore when audio pause
  onAudioPause() {
    this.updateMediaSession(false);
    if (this.video && !this.video.paused && !this.isSyncing) {
      this.isSyncing = true;
      this.video.pause();
      setTimeout(() => { this.isSyncing = false; }, 50);
    }
  }

  private updateMediaSession(playingState?: boolean) {
    if (!('mediaSession' in navigator)) return;
    const track = globalStatus().current_track;
    const playing = playingState !== undefined ? playingState : isPlaying();

    if (track) {
      navigator.mediaSession.metadata = new MediaMetadata({
        title: track.title || 'Ragasukma Music',
        artist: track.uploader || 'Ragasukma Assistant',
        album: 'YouTube Music Library',
        artwork: track.thumbnail
          ? [
              { src: track.thumbnail, sizes: '96x96', type: 'image/jpeg' },
              { src: track.thumbnail, sizes: '256x256', type: 'image/jpeg' },
              { src: track.thumbnail, sizes: '512x512', type: 'image/jpeg' },
            ]
          : [],
      });
    }

    navigator.mediaSession.playbackState = playing ? 'playing' : 'paused';

    try {
      const dur = duration() || track?.duration || 0;
      const cur = currentTime() || 0;
      if (dur > 0 && isFinite(dur) && isFinite(cur) && cur <= dur) {
        navigator.mediaSession.setPositionState({
          duration: dur,
          playbackRate: playing ? 1.0 : 0.0,
          position: Math.min(cur, dur),
        });
      }
    } catch (_) {}
  }

  private loadThumbnail(url?: string) {
    if (!url || url === this.lastThumbnailUrl) return;
    this.lastThumbnailUrl = url;
    const img = new Image();
    img.crossOrigin = 'anonymous';
    img.src = url;
    img.onload = () => {
      this.currentCoverImage = img;
    };
    img.onerror = () => {
      this.currentCoverImage = null;
    };
  }

  private drawFrame() {
    if (!this.ctx || !this.canvas) return;
    const ctx = this.ctx;
    const dpr = 2;
    const w = 680;
    const h = 210;
    const track = globalStatus().current_track;
    const isPlaying = globalStatus().is_playing;
    const now = performance.now() / 1000;

    ctx.save();
    ctx.scale(dpr, dpr);
    ctx.imageSmoothingEnabled = true;
    ctx.imageSmoothingQuality = 'high';

    // ── 1. BACKGROUND & AMBIENT GLOW ──
    ctx.clearRect(0, 0, w, h);

    // Genshin Deep Canvas Gradient
    const bgGrad = ctx.createLinearGradient(0, 0, w, h);
    bgGrad.addColorStop(0, GI_THEME.canvas);
    bgGrad.addColorStop(0.5, GI_THEME.surface);
    bgGrad.addColorStop(1, GI_THEME.surfaceElevated);
    ctx.fillStyle = bgGrad;
    ctx.fillRect(0, 0, w, h);

    // Album Artwork / Thumbnail Loader
    if (track?.thumbnail && track.thumbnail !== this.lastThumbnailUrl) {
      this.loadThumbnail(track.thumbnail);
    }

    // Dynamic Ambient Glow from Album
    if (this.currentCoverImage) {
      ctx.save();
      ctx.globalAlpha = 0.12;
      ctx.filter = 'blur(40px)';
      ctx.drawImage(this.currentCoverImage, -20, -20, 260, h + 40);
      ctx.restore();
    } else {
      const orb = ctx.createRadialGradient(90, h / 2, 10, 90, h / 2, 160);
      orb.addColorStop(0, 'rgba(229, 184, 92, 0.25)');
      orb.addColorStop(1, 'rgba(15, 18, 24, 0)');
      ctx.fillStyle = orb;
      ctx.fillRect(0, 0, w, h);
    }

    // Crisp Window Frame Border
    ctx.save();
    ctx.strokeStyle = GI_THEME.borderSubtle;
    ctx.lineWidth = 1;
    ctx.strokeRect(6, 6, w - 12, h - 12);
    ctx.restore();

    // ── 2. LEFT COLUMN: ALBUM ARTWORK CARD (Zero Radius Square) ──
    const coverSize = 166;
    const coverX = 22;
    const coverY = 22;

    // Drop shadow behind cover
    ctx.save();
    ctx.shadowColor = 'rgba(15, 18, 24, 0.4)';
    ctx.shadowBlur = 12;
    ctx.shadowOffsetX = 2;
    ctx.shadowOffsetY = 4;
    ctx.fillStyle = GI_THEME.surfaceElevated;
    ctx.fillRect(coverX, coverY, coverSize, coverSize);
    ctx.restore();

    // Draw Thumbnail inside sharp rect
    ctx.save();
    ctx.beginPath();
    ctx.rect(coverX, coverY, coverSize, coverSize);
    ctx.clip();

    if (this.currentCoverImage) {
      ctx.drawImage(this.currentCoverImage, coverX, coverY, coverSize, coverSize);
    } else {
      const cardGrad = ctx.createLinearGradient(coverX, coverY, coverX + coverSize, coverY + coverSize);
      cardGrad.addColorStop(0, GI_THEME.surface);
      cardGrad.addColorStop(1, GI_THEME.surfaceElevated);
      ctx.fillStyle = cardGrad;
      ctx.fillRect(coverX, coverY, coverSize, coverSize);

      ctx.fillStyle = GI_THEME.primary;
      ctx.textAlign = 'center';
      ctx.textBaseline = 'middle';

      // Clean vector music note
      const mx = coverX + coverSize / 2;
      const my = coverY + coverSize / 2 - 12;
      ctx.save();
      ctx.beginPath();
      ctx.arc(mx - 8, my + 8, 5, 0, Math.PI * 2);
      ctx.arc(mx + 8, my + 4, 5, 0, Math.PI * 2);
      ctx.fill();
      ctx.beginPath();
      ctx.rect(mx - 4, my - 12, 3, 20);
      ctx.rect(mx + 12, my - 16, 3, 20);
      ctx.rect(mx - 4, my - 16, 19, 5);
      ctx.fill();
      ctx.restore();

      ctx.fillStyle = GI_THEME.textPrimary;
      ctx.font = '600 12px "Plus Jakarta Sans", sans-serif';
      ctx.fillText('Ragasukma', coverX + coverSize / 2, coverY + coverSize / 2 + 24);
    }

    // Sheen
    const sheen = ctx.createLinearGradient(coverX, coverY, coverX, coverY + 60);
    sheen.addColorStop(0, 'rgba(255, 255, 255, 0.3)');
    sheen.addColorStop(1, 'rgba(255, 255, 255, 0)');
    ctx.fillStyle = sheen;
    ctx.fillRect(coverX, coverY, coverSize, 60);

    // Live Equalizer Overlay inside Album (Bottom-Right)
    if (isPlaying) {
      const eqW = 46;
      const eqH = 26;
      const eqX = coverX + coverSize - eqW - 6;
      const eqY = coverY + coverSize - eqH - 6;
      ctx.fillStyle = 'rgba(15, 18, 24, 0.88)';
      ctx.fillRect(eqX, eqY, eqW, eqH);
      ctx.strokeStyle = GI_THEME.borderGold;
      ctx.lineWidth = 1;
      ctx.strokeRect(eqX, eqY, eqW, eqH);

      // 4 Animated Waveform Bars
      for (let i = 0; i < 4; i++) {
        const barH = 4 + Math.abs(Math.sin(now * 5 + i * 1.3)) * 14;
        const bX = eqX + 6 + i * 9;
        const bY = eqY + eqH - 5 - barH;
        ctx.fillStyle = i % 2 === 0 ? GI_THEME.primary : GI_THEME.borderGold;
        ctx.fillRect(bX, bY, 4, barH);
      }
    }
    ctx.restore();

    // Crisp Border Stroke on Cover (Gold Ornaments)
    ctx.save();
    ctx.strokeStyle = isPlaying ? GI_THEME.primary : GI_THEME.borderSubtle;
    ctx.lineWidth = 1;
    ctx.strokeRect(coverX, coverY, coverSize, coverSize);
    ctx.restore();

    // ── 3. RIGHT COLUMN: METADATA & CONTROLS ──
    const contentX = 210;
    const contentW = w - contentX - 22;

    // Top Header: Status Badge & Brand Watermark
    ctx.save();
    const badgeH = 26;
    const badgeY = 22;
    const statusText = isPlaying ? 'NOW PLAYING' : 'PAUSED';

    ctx.font = '700 11px "Plus Jakarta Sans", sans-serif';
    const textMetrics = ctx.measureText(statusText);
    const badgePaddingX = 12;
    const dotSize = 6;
    const dotGap = 8;
    const badgeW = badgePaddingX + dotSize + dotGap + Math.ceil(textMetrics.width) + badgePaddingX;

    // Badge Card Background & Clean Border
    ctx.fillStyle = isPlaying ? 'rgba(229, 184, 92, 0.18)' : 'rgba(224, 83, 69, 0.18)';
    ctx.fillRect(contentX, badgeY, badgeW, badgeH);
    ctx.strokeStyle = isPlaying ? GI_THEME.primary : GI_THEME.danger;
    ctx.lineWidth = 1;
    ctx.strokeRect(contentX, badgeY, badgeW, badgeH);

    // Indicator Dot (Centered Vertically)
    const dotX = contentX + badgePaddingX;
    const dotY = Math.round(badgeY + (badgeH - dotSize) / 2);
    ctx.fillStyle = isPlaying ? GI_THEME.primary : GI_THEME.danger;
    ctx.fillRect(dotX, dotY, dotSize, dotSize);

    // Status Text (Crisp Math Baseline)
    ctx.fillStyle = isPlaying ? GI_THEME.textPrimary : GI_THEME.danger;
    ctx.textAlign = 'left';
    ctx.textBaseline = 'middle';
    ctx.fillText(statusText, dotX + dotSize + dotGap, Math.round(badgeY + badgeH / 2) + 0.5);

    // Brand Watermark on Top-Right
    ctx.textAlign = 'right';
    ctx.textBaseline = 'middle';
    ctx.font = '600 11px "JetBrains Mono", monospace';
    ctx.fillStyle = GI_THEME.textSecondary;
    ctx.fillText('RAGASUKMA • PIP', contentX + contentW, Math.round(badgeY + badgeH / 2));
    ctx.restore();

    // Track Metadata (Title & Artist)
    const title = track?.title || 'Ragasukma Music Player';
    const artist = track?.uploader || 'YouTube Desktop Audio Stream';

    // Title (Lora style heading)
    ctx.save();
    ctx.fillStyle = GI_THEME.textPrimary;
    ctx.font = '600 20px "Lora", "Plus Jakarta Sans", serif';
    ctx.textAlign = 'left';
    ctx.textBaseline = 'middle';

    const maxTitleLen = 38;
    const cleanTitle = title.length > maxTitleLen ? title.substring(0, maxTitleLen - 2) + '...' : title;
    ctx.fillText(cleanTitle, contentX, 72);

    // Artist
    ctx.fillStyle = GI_THEME.textSecondary;
    ctx.font = '600 13px "Plus Jakarta Sans", sans-serif';
    const maxArtistLen = 44;
    const cleanArtist = artist.length > maxArtistLen ? artist.substring(0, maxArtistLen - 2) + '...' : artist;
    ctx.fillText(cleanArtist, contentX, 94);
    ctx.restore();

    // ── 4. TIMELINE PROGRESS BAR ──
    const barX = contentX;
    const barY = 132;
    const barW = contentW;
    const barH = 6;
    const dur = duration() || track?.duration || 100;
    const cur = currentTime();
    const pct = Math.min(1, Math.max(0, dur > 0 ? cur / dur : 0));

    // Background Bar
    ctx.save();
    ctx.fillStyle = GI_THEME.surface;
    ctx.fillRect(barX, barY, barW, barH);
    ctx.strokeStyle = GI_THEME.borderSubtle;
    ctx.lineWidth = 1;
    ctx.strokeRect(barX, barY, barW, barH);

    // Filled Gradient Bar (Wish Gold / Primary)
    if (pct > 0) {
      const progGrad = ctx.createLinearGradient(barX, 0, barX + barW, 0);
      progGrad.addColorStop(0, GI_THEME.borderGold);
      progGrad.addColorStop(0.5, GI_THEME.primary);
      progGrad.addColorStop(1, GI_THEME.primaryHover);

      ctx.fillStyle = progGrad;
      ctx.fillRect(barX, barY, barW * pct, barH);

      // Square Knob
      const knopX = barX + barW * pct - 3.5;
      const knopY = barY + barH / 2 - 3.5;
      ctx.fillStyle = GI_THEME.textPrimary;
      ctx.fillRect(knopX, knopY, 7, 7);
    }
    ctx.restore();

    // Time Indicators
    ctx.save();
    ctx.fillStyle = GI_THEME.textSecondary;
    ctx.font = '600 12px "JetBrains Mono", monospace';
    ctx.textBaseline = 'top';

    ctx.textAlign = 'left';
    ctx.fillText(formatDuration(Math.floor(cur)), barX, barY + 12);

    ctx.textAlign = 'right';
    ctx.fillText(formatDuration(Math.floor(dur)), barX + barW, barY + 12);
    ctx.restore();

    // Subtle Bottom Footer Hint
    ctx.save();
    ctx.textAlign = 'left';
    ctx.textBaseline = 'bottom';
    ctx.font = '500 11px "Plus Jakarta Sans", sans-serif';
    ctx.fillStyle = GI_THEME.textMuted;
    ctx.fillText('Press Space / Use OS Media Controls to Play & Pause', contentX, h - 18);
    ctx.restore();

    ctx.restore(); // Restore HiDPI scale
  }

  private startRenderLoop() {
    this.stopRenderLoop();

    const render = () => {
      this.drawFrame();
      this.updateMediaSession();
    };

    // Render frame pertama seketika
    render();

    // ── Web Worker Timer ──
    // Chromium aggressively freezes requestAnimationFrame and throttles standard setInterval
    // when web tab is minimized or in the background. Web Worker timer runs in a separate thread
    // so PiP render loop maintains a constant real-time 30 FPS without freezing!
    try {
      const blob = new Blob(
        [`let t = null; self.onmessage = function(e) { if(e.data === 'start') { t = setInterval(function() { postMessage('tick'); }, 1000/30); } else if(e.data === 'stop') { clearInterval(t); } };`],
        { type: 'application/javascript' }
      );
      const workerUrl = URL.createObjectURL(blob);
      this.workerTimer = new Worker(workerUrl);
      this.workerTimer.onmessage = () => {
        if (this.isPipActive) {
          render();
        }
      };
      this.workerTimer.postMessage('start');
    } catch {
      // Fallback standard interval if Web Worker is not permitted
      this.timerId = setInterval(() => {
        if (this.isPipActive) {
          render();
        }
      }, 1000 / 30);
    }
  }

  private stopRenderLoop() {
    if (this.workerTimer) {
      try {
        this.workerTimer.postMessage('stop');
        this.workerTimer.terminate();
      } catch (_) {}
      this.workerTimer = null;
    }
    if (this.timerId !== null) {
      clearInterval(this.timerId);
      this.timerId = null;
    }
  }

  async togglePip(): Promise<boolean> {
    this.init();
    if (!this.canvas || !this.video) return false;

    if (document.pictureInPictureElement) {
      await document.exitPictureInPicture().catch(() => {});
      this.isPipActive = false;
      this.stopRenderLoop();
      return false;
    }

    try {
      this.drawFrame();
      // Stream canvas to video element
      if (!(this.video as any).srcObject) {
        const stream = (this.canvas as any).captureStream ? (this.canvas as any).captureStream(30) : null;
        if (stream) {
          (this.video as any).srcObject = stream;
          await this.video.play().catch(() => {});
        }
      }

      await this.video.requestPictureInPicture();
      this.isPipActive = true;
      this.startRenderLoop();
      return true;
    } catch (err) {
      console.error('[PiP Error]:', err);
      return false;
    }
  }

  isSupported(): boolean {
    return 'pictureInPictureEnabled' in document && document.pictureInPictureEnabled;
  }

  isActive(): boolean {
    return this.isPipActive || !!document.pictureInPictureElement;
  }
}

export const pipManager = new PipManager();

// Automatically synchronize when playback status changes
onPlayStateChange((playing) => {
  if (playing) {
    pipManager.onAudioPlay();
  } else {
    pipManager.onAudioPause();
  }
});
