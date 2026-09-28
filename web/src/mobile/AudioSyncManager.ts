import { api, YtTrackInfo } from '../api';

export interface MobileTrackState {
  track: YtTrackInfo | null;
  isPlaying: boolean;
  isPaused: boolean;
  currentTime: number;
  duration: number;
}

/**
 * MobileAudioSyncManager
 * Mengelola audio unlock dan pemutaran musik streaming langsung di browser smartphone.
 */
export class MobileAudioSyncManager {
  private isUnlocked = false;
  private audioCtx: AudioContext | null = null;
  private musicAudio: HTMLAudioElement | null = null;
  private currentTrack: YtTrackInfo | null = null;
  private onTrackStateChangeCb: ((state: MobileTrackState) => void) | null = null;
  private onStateChangeCb: ((state: string) => void) | null = null;

  constructor() {
    if (typeof window !== 'undefined') {
      const AudioCtxClass = window.AudioContext || (window as any).webkitAudioContext;
      if (AudioCtxClass) {
        this.audioCtx = new AudioCtxClass();
      }

      this.musicAudio = new Audio();
      this.musicAudio.preload = 'auto';

      this.musicAudio.addEventListener('timeupdate', () => {
        this.emitTrackState();
      });

      this.musicAudio.addEventListener('play', () => {
        this.onStateChangeCb?.('playing');
        this.emitTrackState();
      });

      this.musicAudio.addEventListener('pause', () => {
        this.onStateChangeCb?.('paused');
        this.emitTrackState();
      });

      this.musicAudio.addEventListener('ended', () => {
        this.onStateChangeCb?.('idle');
        this.currentTrack = null;
        this.emitTrackState();
      });

      this.musicAudio.addEventListener('error', (e) => {
        console.warn('[MobileAudio] Error audio stream:', e);
        this.onStateChangeCb?.('idle');
        this.emitTrackState();
      });
    }
  }

  private emitTrackState() {
    if (!this.onTrackStateChangeCb) return;
    const isPlaying = !!(this.musicAudio && !this.musicAudio.paused && !this.musicAudio.ended);
    const isPaused = !!(
      this.musicAudio &&
      this.musicAudio.paused &&
      this.musicAudio.currentTime > 0 &&
      !this.musicAudio.ended
    );
    this.onTrackStateChangeCb({
      track: this.currentTrack,
      isPlaying,
      isPaused,
      currentTime: this.musicAudio ? this.musicAudio.currentTime : 0,
      duration: this.musicAudio ? this.musicAudio.duration || this.currentTrack?.duration || 0 : 0,
    });
  }

  /**
   * Unlock audio engine di mobile browser via gesture user (klik / touch)
   */
  public async unlockAudio(): Promise<boolean> {
    if (this.isUnlocked) return true;

    try {
      if (this.audioCtx && this.audioCtx.state === 'suspended') {
        await this.audioCtx.resume();
      }

      // Bunyikan oscillator hening 10ms untuk membebaskan autoplay lock Chrome Android
      if (this.audioCtx) {
        const osc = this.audioCtx.createOscillator();
        const gain = this.audioCtx.createGain();
        gain.gain.value = 0.001; // nyaris senyap
        osc.connect(gain);
        gain.connect(this.audioCtx.destination);
        osc.start();
        osc.stop(this.audioCtx.currentTime + 0.01);
      }

      // Priming HTMLAudioElement untuk browser mobile
      if (this.musicAudio) {
        this.musicAudio.load();
      }

      this.isUnlocked = true;
      console.log('[MobileAudio] Audio gesture lock berhasil dibuka');
      return true;
    } catch (err) {
      console.warn('[MobileAudio] Gagal unlock audio context:', err);
      return false;
    }
  }

  public async playMusic(trackTitleOrQuery?: string) {
    if (!this.musicAudio) return;

    try {
      const res = await api.listMusic();
      const list: YtTrackInfo[] = Array.isArray(res) ? res : res?.tracks || [];
      if (list.length === 0) {
        console.warn('[MobileAudio] Library musik kosong di server');
        return;
      }

      let matched: YtTrackInfo | undefined;
      const raw = (trackTitleOrQuery || '').trim();

      if (raw) {
        const clean = (s: string) =>
          s.toLowerCase().replace(/[^a-z0-9\s]/g, ' ').replace(/\s+/g, ' ').trim();
        const qClean = clean(raw);
        const qWords = qClean.split(' ').filter(Boolean);

        // 1. Cek Exact Match
        matched = list.find(
          (t) =>
            t.filename === raw ||
            t.title === raw ||
            t.id === raw ||
            t.title.toLowerCase() === raw.toLowerCase() ||
            t.filename.toLowerCase() === raw.toLowerCase()
        );

        // 2. Cek Substring
        if (!matched) {
          matched = list.find((t) => {
            const tClean = clean(`${t.title} ${t.filename}`);
            return tClean.includes(qClean) || qClean.includes(clean(t.title));
          });
        }

        // 3. Cek Multi-word Token
        if (!matched && qWords.length > 0) {
          matched = list.find((t) => {
            const tClean = clean(`${t.title} ${t.filename}`);
            return qWords.every((w) => tClean.includes(w));
          });
        }
      }

      const trackToPlay = matched || list[0];
      if (!trackToPlay || !trackToPlay.filename) return;

      this.currentTrack = trackToPlay;
      const url = api.getMusicUrl(trackToPlay.filename);
      this.musicAudio.src = url;
      this.musicAudio.currentTime = 0;

      const playPromise = this.musicAudio.play();
      if (playPromise !== undefined) {
        playPromise.catch((err) => {
          console.warn('[MobileAudio] Autoplay tertahan browser mobile (perlu sentuhan/tap):', err);
        });
      }
      this.emitTrackState();
    } catch (err) {
      console.error('[MobileAudio] Gagal memutar musik:', err);
    }
  }

  public pauseMusic() {
    if (this.musicAudio) {
      this.musicAudio.pause();
      this.emitTrackState();
    }
  }

  public resumeMusic() {
    if (this.musicAudio && this.currentTrack) {
      this.musicAudio.play().catch((err) => {
        console.warn('[MobileAudio] Resume tertahan:', err);
      });
      this.emitTrackState();
    }
  }

  public stopMusic() {
    if (this.musicAudio) {
      this.musicAudio.pause();
      this.musicAudio.currentTime = 0;
      this.currentTrack = null;
      this.emitTrackState();
      this.onStateChangeCb?.('idle');
    }
  }

  public seekMusic(secs: number) {
    if (this.musicAudio) {
      this.musicAudio.currentTime = secs;
      this.emitTrackState();
    }
  }

  public setOnStateChange(cb: (state: string) => void) {
    this.onStateChangeCb = cb;
  }

  public setOnTrackStateChange(cb: (state: MobileTrackState) => void) {
    this.onTrackStateChangeCb = cb;
  }

  public getCurrentTrack(): YtTrackInfo | null {
    return this.currentTrack;
  }

  public isStreamPlaying(): boolean {
    return !!(this.musicAudio && !this.musicAudio.paused && !this.musicAudio.ended);
  }

  public stopAll() {
    this.stopMusic();
  }
}
