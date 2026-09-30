import { api, YtTrackInfo } from '../api';

export interface MobileTrackState {
  track: YtTrackInfo | null;
  isPlaying: boolean;
  isPaused: boolean;
  currentTime: number;
  duration: number;
}

export interface MobileLipWeights {
  aa: number;
  ih: number;
  ou: number;
  ee: number;
  oh: number;
  mouthOpen: number;
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
        try {
          this.audioCtx = new AudioCtxClass({ latencyHint: 'playback' });
        } catch {
          this.audioCtx = new AudioCtxClass();
        }
      }

      this.musicAudio = new Audio();
      this.musicAudio.preload = 'auto';

      this.voiceAudio = new Audio();
      this.voiceAudio.preload = 'auto';
      // Gunakan playsinline agar iOS / Android merutekan langsung ke Media Speaker murni
      this.voiceAudio.setAttribute('playsinline', 'true');
      (this.voiceAudio as any).playsInline = true;

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
      if (this.voiceAudio) {
        this.voiceAudio.load();
      }
      this.initVoiceAnalyserOnce();

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

  private isDucked = false;
  private preDuckVolume = 1.0;

  /**
   * Turunkan volume musik sementara saat AI asisten berbicara (audio ducking)
   */
  public duckMusic(duckLevel: number = 0.2) {
    if (this.musicAudio && !this.musicAudio.paused && !this.isDucked) {
      this.isDucked = true;
      this.preDuckVolume = this.musicAudio.volume;
      this.musicAudio.volume = Math.max(0, Math.min(1, duckLevel));
    }
  }

  /**
   * Pulihkan volume musik normal setelah AI asisten selesai berbicara
   */
  public unduckMusic() {
    if (this.musicAudio && this.isDucked) {
      this.isDucked = false;
      this.musicAudio.volume = this.preDuckVolume;
    }
  }

  public setVolume(vol: number) {
    if (this.musicAudio) {
      this.musicAudio.volume = Math.max(0, Math.min(1, vol));
      this.preDuckVolume = this.musicAudio.volume;
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

  // ---------------------------------------------------------------------------
  // Supertonic-3 Neural Voice Audio & Audio-Driven Lip Sync (A-I-U-E-O Visemes)
  // ---------------------------------------------------------------------------
  private voiceAudio: HTMLAudioElement | null = null;
  private voiceSourceNode: MediaElementAudioSourceNode | null = null;
  private voiceGainNode: GainNode | null = null;
  private voiceAnalyser: AnalyserNode | null = null;
  private voiceFreqData: Uint8Array | null = null;
  private voiceTimeData: Uint8Array | null = null;

  private ensureVoiceAnalyserReady() {
    this.initVoiceAnalyserOnce();
  }

  private initVoiceAnalyserOnce() {
    if (!this.audioCtx || !this.voiceAudio || this.voiceSourceNode) return;
    try {
      if (!this.voiceAnalyser) {
        this.voiceAnalyser = this.audioCtx.createAnalyser();
        this.voiceAnalyser.fftSize = 512;
        this.voiceAnalyser.smoothingTimeConstant = 0.35;
        this.voiceFreqData = new Uint8Array(this.voiceAnalyser.frequencyBinCount);
        this.voiceTimeData = new Uint8Array(this.voiceAnalyser.fftSize);
      }
      if (!this.voiceGainNode) {
        this.voiceGainNode = this.audioCtx.createGain();
        // Backend WAV sudah ternormalisasi ke 0.92 peak, beri gain netral/optimal (1.05) agar jernih dan lantang tanpa distorsi
        this.voiceGainNode.gain.value = 1.05;
      }
      const source = this.audioCtx.createMediaElementSource(this.voiceAudio);
      source.connect(this.voiceAnalyser);
      this.voiceAnalyser.connect(this.voiceGainNode);
      this.voiceGainNode.connect(this.audioCtx.destination);
      this.voiceSourceNode = source;
    } catch (e) {
      console.warn('[MobileAudio] Web Audio Analyser init note:', e);
    }
  }

  public playVoice(
    url: string,
    callbacks?: {
      onPlaying?: () => void;
      onEnded?: () => void;
      onError?: (err: any) => void;
    }
  ) {
    if (!this.voiceAudio) return;
    this.initVoiceAnalyserOnce();

    if (this.audioCtx && this.audioCtx.state === 'suspended') {
      this.audioCtx.resume().catch(() => {});
    }

    // Bersihkan listener sebelumnya
    this.voiceAudio.onplaying = null;
    this.voiceAudio.onended = null;
    this.voiceAudio.onerror = null;

    let hasRevoked = false;
    const safeRevoke = () => {
      if (!hasRevoked && url.startsWith('blob:')) {
        hasRevoked = true;
        try {
          URL.revokeObjectURL(url);
        } catch {}
      }
    };

    this.voiceAudio.onplaying = () => {
      callbacks?.onPlaying?.();
    };

    this.voiceAudio.onended = () => {
      safeRevoke();
      callbacks?.onEnded?.();
    };

    this.voiceAudio.onerror = (e) => {
      safeRevoke();
      callbacks?.onError?.(e);
    };

    this.voiceAudio.src = url;
    this.voiceAudio.currentTime = 0;
    this.voiceAudio.volume = 1.0;
    const p = this.voiceAudio.play();
    if (p !== undefined) {
      p.catch((err) => {
        safeRevoke();
        callbacks?.onError?.(err);
      });
    }
  }

  public stopVoice() {
    if (this.voiceAudio) {
      this.voiceAudio.pause();
      this.voiceAudio.src = '';
    }
  }

  public attachVoiceAudio(audio: HTMLAudioElement) {
    // Reusable voiceAudio sudah otomatis terpasang
  }

  public detachVoiceAudio() {
    // Reusable voiceAudio tetap hidup, cukup hentikan pemutaran
    this.stopVoice();
  }

  public isVoiceAudioActive(): boolean {
    return !!(this.voiceAudio && !this.voiceAudio.paused && !this.voiceAudio.ended);
  }

  public getVoiceLipWeights(): MobileLipWeights {
    const zero: MobileLipWeights = { aa: 0, ih: 0, ou: 0, ee: 0, oh: 0, mouthOpen: 0 };
    if (!this.voiceAudio || this.voiceAudio.paused || this.voiceAudio.ended) {
      return zero;
    }

    if (!this.voiceAnalyser || !this.voiceFreqData || !this.voiceTimeData) {
      // Fallback sinusoidal ringan jika Analyser belum siap
      const cur = this.voiceAudio.currentTime;
      const open = Math.pow(Math.abs(Math.sin(cur * 16.0)), 1.3) * 0.55;
      return { aa: open, ih: 0, ou: 0, ee: 0, oh: 0, mouthOpen: open };
    }

    this.voiceAnalyser.getByteTimeDomainData(this.voiceTimeData as any);
    this.voiceAnalyser.getByteFrequencyData(this.voiceFreqData as any);

    // 1. RMS Energy dari waveform time-domain (128 adalah titik nol baseline)
    let sumSq = 0;
    const len = this.voiceTimeData.length;
    for (let i = 0; i < len; i++) {
      const norm = (this.voiceTimeData[i] - 128) / 128;
      sumSq += norm * norm;
    }
    const rms = Math.sqrt(sumSq / len);

    // Noise floor: jika di bawah 0.015, anggap senyap / jeda napas antar kata
    if (rms < 0.015) {
      return zero;
    }

    const mouthOpen = Math.min(1.0, Math.pow(Math.max(0, (rms - 0.015) * 6.5), 0.8) * 0.9);

    // 2. Formant Energy Bands dari FFT Frequency Data:
    // (sampleRate ~44100-48000 Hz, fftSize 512 => 256 bins, lebar bin ~86-94 Hz)
    // Low band (~200 - 650 Hz): bins 2..7 (vokal U & O)
    // Mid band (~700 - 1500 Hz): bins 8..17 (vokal A)
    // Mid-High band (~1500 - 2400 Hz): bins 18..27 (vokal E)
    // High band (~2400 - 3800 Hz): bins 28..44 (vokal I)
    let eLow = 0;
    for (let i = 2; i <= 7 && i < this.voiceFreqData.length; i++) eLow += this.voiceFreqData[i];
    let eMid = 0;
    for (let i = 8; i <= 17 && i < this.voiceFreqData.length; i++) eMid += this.voiceFreqData[i];
    let eMidHigh = 0;
    for (let i = 18; i <= 27 && i < this.voiceFreqData.length; i++) eMidHigh += this.voiceFreqData[i];
    let eHigh = 0;
    for (let i = 28; i <= 44 && i < this.voiceFreqData.length; i++) eHigh += this.voiceFreqData[i];

    const eTotal = eLow + eMid + eMidHigh + eHigh + 1e-4;
    const rLow = eLow / eTotal;
    const rMid = eMid / eTotal;
    const rMidHigh = eMidHigh / eTotal;
    const rHigh = eHigh / eTotal;

    // Bobot viseme vokal AIUEO
    const aa = Math.min(1.0, rMid * 1.55) * mouthOpen;
    const ih = Math.min(1.0, rHigh * 1.85) * mouthOpen * 0.75;
    const ou = Math.min(1.0, Math.max(0, rLow - rMid * 0.3) * 1.7) * mouthOpen * 0.70;
    const ee = Math.min(1.0, rMidHigh * 1.75) * mouthOpen * 0.80;
    const oh = Math.min(1.0, (rLow * 0.65 + rMid * 0.45) * 1.4) * mouthOpen * 0.85;

    return {
      aa,
      ih,
      ou,
      ee,
      oh,
      mouthOpen,
    };
  }

  public stopAll() {
    this.stopMusic();
    this.stopVoice();
  }
}

export const globalAudioSync = new MobileAudioSyncManager();
