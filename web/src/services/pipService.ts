import { globalStatus, currentTime, duration, pauseMusic, resumeMusic, nextTrack, prevTrack, formatDuration } from '../playerStore';

class PipManager {
  private canvas: HTMLCanvasElement | null = null;
  private ctx: CanvasRenderingContext2D | null = null;
  private video: HTMLVideoElement | null = null;
  private animFrameId: number | null = null;
  private isPipActive = false;
  private currentCoverImage: HTMLImageElement | null = null;
  private lastThumbnailUrl = '';

  init() {
    if (this.canvas) return;

    this.canvas = document.createElement('canvas');
    this.canvas.width = 512;
    this.canvas.height = 512;
    this.ctx = this.canvas.getContext('2d');

    this.video = document.createElement('video');
    this.video.muted = true;
    this.video.autoplay = true;
    this.video.playsInline = true;

    // Sinkronisasi status PiP saat user menutup jendela PiP secara manual
    this.video.addEventListener('leavepictureinpicture', () => {
      this.isPipActive = false;
      this.stopRenderLoop();
    });

    // Registrasi MediaSession API bawaan browser OS (agar tombol PiP OS next/prev/play/pause berfungsi)
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
    }
  }

  private updateMediaSession() {
    if (!('mediaSession' in navigator)) return;
    const track = globalStatus().current_track;
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
      navigator.mediaSession.playbackState = globalStatus().is_playing ? 'playing' : 'paused';
    }
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
    const w = this.canvas.width;
    const h = this.canvas.height;
    const track = globalStatus().current_track;
    const isPlaying = globalStatus().is_playing;

    // Background Gradient Gelap Tema Ragasukma
    const bgGrad = ctx.createLinearGradient(0, 0, 0, h);
    bgGrad.addColorStop(0, '#161D24');
    bgGrad.addColorStop(1, '#0D1117');
    ctx.fillStyle = bgGrad;
    ctx.fillRect(0, 0, w, h);

    // Album Artwork / Thumbnail
    if (track?.thumbnail && track.thumbnail !== this.lastThumbnailUrl) {
      this.loadThumbnail(track.thumbnail);
    }

    if (this.currentCoverImage) {
      ctx.save();
      // Rounded Card Cover
      const coverSize = 280;
      const x = (w - coverSize) / 2;
      const y = 30;
      const radius = 20;

      ctx.beginPath();
      ctx.moveTo(x + radius, y);
      ctx.lineTo(x + coverSize - radius, y);
      ctx.quadraticCurveTo(x + coverSize, y, x + coverSize, y + radius);
      ctx.lineTo(x + coverSize, y + coverSize - radius);
      ctx.quadraticCurveTo(x + coverSize, y + coverSize, x + coverSize - radius, y + coverSize);
      ctx.lineTo(x + radius, y + coverSize);
      ctx.quadraticCurveTo(x, y + coverSize, x, y + coverSize - radius);
      ctx.lineTo(x, y + radius);
      ctx.quadraticCurveTo(x, y, x + radius, y);
      ctx.closePath();
      ctx.clip();

      ctx.drawImage(this.currentCoverImage, x, y, coverSize, coverSize);
      ctx.restore();

      // Border Glow
      ctx.strokeStyle = '#7BDFF2';
      ctx.lineWidth = 3;
      ctx.stroke();
    } else {
      // Placeholder Cover
      const coverSize = 280;
      const x = (w - coverSize) / 2;
      const y = 30;
      ctx.fillStyle = '#232D36';
      ctx.beginPath();
      ctx.roundRect(x, y, coverSize, coverSize, 20);
      ctx.fill();
      ctx.strokeStyle = '#2F3B46';
      ctx.lineWidth = 2;
      ctx.stroke();

      ctx.fillStyle = '#7BDFF2';
      ctx.font = 'bold 36px Carter One, sans-serif';
      ctx.textAlign = 'center';
      ctx.fillText('🎵 Ragasukma', w / 2, y + 150);
    }

    // Title Text - Extra Bold & Besar
    ctx.fillStyle = '#FFFFFF';
    ctx.font = 'bold 26px Inter, sans-serif';
    ctx.textAlign = 'center';
    const title = track?.title || 'Ragasukma Music Player';
    const cleanTitle = title.length > 26 ? title.substring(0, 24) + '...' : title;
    ctx.fillText(cleanTitle, w / 2, 360);

    // Artist / Uploader Text
    ctx.fillStyle = '#7BDFF2';
    ctx.font = '600 18px Inter, sans-serif';
    const artist = track?.uploader || 'Local Library';
    const cleanArtist = artist.length > 32 ? artist.substring(0, 30) + '...' : artist;
    ctx.fillText(cleanArtist, w / 2, 395);

    // Timeline Progress Bar
    const barX = 40;
    const barY = 435;
    const barW = w - 80;
    const barH = 8;
    const dur = duration() || track?.duration || 100;
    const cur = currentTime();
    const pct = Math.min(1, Math.max(0, dur > 0 ? cur / dur : 0));

    // Bar Background
    ctx.fillStyle = '#2F3B46';
    ctx.beginPath();
    ctx.roundRect(barX, barY, barW, barH, 4);
    ctx.fill();

    // Active Bar Progress with Cyan Glow
    ctx.fillStyle = '#7BDFF2';
    ctx.beginPath();
    ctx.roundRect(barX, barY, barW * pct, barH, 4);
    ctx.fill();

    // Time Indicators & Status Badge
    ctx.fillStyle = '#94A3B8';
    ctx.font = 'bold 15px monospace';
    ctx.textAlign = 'left';
    ctx.fillText(formatDuration(Math.floor(cur)), barX, barY + 30);
    ctx.textAlign = 'right';
    ctx.fillText(formatDuration(Math.floor(dur)), barX + barW, barY + 30);

    // State Badge
    ctx.textAlign = 'center';
    ctx.fillStyle = isPlaying ? '#7BDFF2' : '#F2B5D4';
    ctx.font = 'bold 16px Inter, sans-serif';
    ctx.fillText(isPlaying ? '▶ SEDANG MEMUTAR' : '⏸ DIJEDA', w / 2, barY + 30);
  }

  private startRenderLoop() {
    const loop = () => {
      this.drawFrame();
      this.updateMediaSession();
      if (this.isPipActive) {
        this.animFrameId = requestAnimationFrame(loop);
      }
    };
    this.animFrameId = requestAnimationFrame(loop);
  }

  private stopRenderLoop() {
    if (this.animFrameId) {
      cancelAnimationFrame(this.animFrameId);
      this.animFrameId = null;
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
      // Stream canvas ke video element
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
