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
 * Manage audio unlock and direct streaming music playback on smartphone browser.
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
      this.musicAudio.crossOrigin = 'anonymous';

      this.voiceAudio = new Audio();
      this.voiceAudio.preload = 'auto';
      this.voiceAudio.crossOrigin = 'anonymous';
      // Use playsinline so iOS / Android routes directly to pure Media Speaker
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
   * Unlock audio engine in mobile browser via user gesture (click / touch)
   */
  public async unlockAudio(): Promise<boolean> {
    if (this.isUnlocked) return true;

    try {
      if (this.audioCtx && this.audioCtx.state === 'suspended') {
        await this.audioCtx.resume();
      }

      // Play silent 10ms oscillator to release Chrome Android autoplay lock
      if (this.audioCtx) {
        const osc = this.audioCtx.createOscillator();
        const gain = this.audioCtx.createGain();
        gain.gain.value = 0.001; // nyaris senyap
        osc.connect(gain);
        gain.connect(this.audioCtx.destination);
        osc.start();
        osc.stop(this.audioCtx.currentTime + 0.01);
      }

      // Priming HTMLAudioElement for mobile browser
      if (this.musicAudio) {
        this.musicAudio.load();
      }
      if (this.voiceAudio) {
        this.voiceAudio.load();
      }
      this.initVoiceAnalyserOnce();

      this.isUnlocked = true;
      console.log('[MobileAudio] Audio gesture lock successfully unlocked');
      return true;
    } catch (err) {
      console.warn('[MobileAudio] Failed to unlock audio context:', err);
      return false;
    }
  }

  public async playMusic(trackTitleOrQuery?: string) {
    if (!this.musicAudio) return;

    try {
      const res = await api.listMusic();
      const list: YtTrackInfo[] = Array.isArray(res) ? res : res?.tracks || [];
      if (list.length === 0) {
        console.warn('[MobileAudio] Music library empty on server');
        return;
      }

      let matched: YtTrackInfo | undefined;
      const raw = (trackTitleOrQuery || '').trim();

      if (raw) {
        const clean = (s: string) =>
          s.toLowerCase().replace(/[^a-z0-9\s]/g, ' ').replace(/\s+/g, ' ').trim();
        const qClean = clean(raw);
        const qWords = qClean.split(' ').filter(Boolean);

        // 1. Check Exact Match
        matched = list.find(
          (t) =>
            t.filename === raw ||
            t.title === raw ||
            t.id === raw ||
            t.title.toLowerCase() === raw.toLowerCase() ||
            t.filename.toLowerCase() === raw.toLowerCase()
        );

        // 2. Check Substring
        if (!matched) {
          matched = list.find((t) => {
            const tClean = clean(`${t.title} ${t.filename}`);
            return tClean.includes(qClean) || qClean.includes(clean(t.title));
          });
        }

        // 3. Check Multi-word Token
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
          console.warn('[MobileAudio] Autoplay prevented by mobile browser (touch/tap required):', err);
        });
      }
      this.emitTrackState();
    } catch (err) {
      console.error('[MobileAudio] Failed to play music:', err);
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
        console.warn('[MobileAudio] Resume prevented:', err);
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
    }
  }

  public seekMusic(seconds: number) {
    if (this.musicAudio) {
      this.musicAudio.currentTime = seconds;
      this.emitTrackState();
    }
  }

  private isDucked = false;
  private preDuckVolume = 1.0;

  /**
   * Temporarily lowers music volume while AI assistant speaks (audio ducking)
   */
  public duckMusic(duckLevel: number = 0.2) {
    if (this.musicAudio && !this.musicAudio.paused && !this.isDucked) {
      this.isDucked = true;
      this.preDuckVolume = this.musicAudio.volume;
      this.musicAudio.volume = Math.max(0, Math.min(1, duckLevel));
    }
  }

  /**
   * Restores normal music volume after AI assistant finishes speaking
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
  private activeBufferSource: AudioBufferSourceNode | null = null;
  private isVoicePlaying = false;
  private voiceGainNode: GainNode | null = null;
  private voiceAnalyser: AnalyserNode | null = null;
  private voiceFreqData: Uint8Array | null = null;
  private voiceTimeData: Uint8Array | null = null;

  private ensureVoiceAnalyserReady() {
    this.initVoiceAnalyserOnce();
  }

  private initVoiceAnalyserOnce() {
    if (!this.audioCtx) return;
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
        // Backend WAV normalized to 0.92 peak, optimal gain (1.05) applied for clarity without distortion
        this.voiceGainNode.gain.value = 1.05;
        this.voiceAnalyser.connect(this.voiceGainNode);
        this.voiceGainNode.connect(this.audioCtx.destination);
      }
      if (this.voiceAudio && !this.voiceSourceNode) {
        try {
          const source = this.audioCtx.createMediaElementSource(this.voiceAudio);
          source.connect(this.voiceAnalyser);
          this.voiceSourceNode = source;
        } catch {}
      }
    } catch (e) {
      console.warn('[MobileAudio] Web Audio Analyser init note:', e);
    }
  }

  public async playVoice(
    url: string,
    callbacks?: {
      onPlaying?: () => void;
      onEnded?: () => void;
      onError?: (err: any) => void;
    }
  ) {
    this.stopVoice();
    this.initVoiceAnalyserOnce();

    if (this.audioCtx && this.audioCtx.state === 'suspended') {
      try {
        await this.audioCtx.resume();
      } catch {}
    }

    let hasRevoked = false;
    const safeRevoke = () => {
      if (!hasRevoked && url.startsWith('blob:')) {
        hasRevoked = true;
        try {
          URL.revokeObjectURL(url);
        } catch {}
      }
    };

    // Primary: Web Audio API AudioBufferSourceNode
    // 100% immune to Mobile CORS muting, Range header stalls, and guarantees exact onended timing!
    if (this.audioCtx && this.voiceAnalyser && this.voiceGainNode) {
      try {
        const resp = await fetch(url, { mode: 'cors' });
        if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
        const arrayBuf = await resp.arrayBuffer();
        safeRevoke();

        const audioBuf = await this.audioCtx.decodeAudioData(arrayBuf);
        const source = this.audioCtx.createBufferSource();
        source.buffer = audioBuf;
        source.connect(this.voiceAnalyser);

        this.activeBufferSource = source;
        this.isVoicePlaying = true;

        source.onended = () => {
          if (this.activeBufferSource === source) {
            this.activeBufferSource = null;
            this.isVoicePlaying = false;
            callbacks?.onEnded?.();
          }
        };

        source.start(0);
        callbacks?.onPlaying?.();
        return;
      } catch (err) {
        console.warn('[MobileAudio] Web Audio buffer playback fallback to HTMLAudioElement:', err);
      }
    }

    // Fallback: HTMLAudioElement
    if (!this.voiceAudio) {
      safeRevoke();
      callbacks?.onError?.('Audio element unavailable');
      return;
    }

    this.voiceAudio.onplaying = () => {
      this.isVoicePlaying = true;
      callbacks?.onPlaying?.();
    };

    this.voiceAudio.onended = () => {
      this.isVoicePlaying = false;
      safeRevoke();
      callbacks?.onEnded?.();
    };

    this.voiceAudio.onerror = (e) => {
      this.isVoicePlaying = false;
      safeRevoke();
      callbacks?.onError?.(e);
    };

    this.voiceAudio.src = url;
    this.voiceAudio.currentTime = 0;
    this.voiceAudio.volume = 1.0;
    const p = this.voiceAudio.play();
    if (p !== undefined) {
      p.catch((err) => {
        this.isVoicePlaying = false;
        safeRevoke();
        callbacks?.onError?.(err);
      });
    }
  }

  public stopVoice() {
    this.isVoicePlaying = false;
    if (this.activeBufferSource) {
      try {
        this.activeBufferSource.stop();
        this.activeBufferSource.disconnect();
      } catch {}
      this.activeBufferSource = null;
    }
    if (this.voiceAudio) {
      try {
        this.voiceAudio.pause();
        this.voiceAudio.currentTime = 0;
        this.voiceAudio.src = '';
      } catch {}
    }
  }

  public attachVoiceAudio(_audio: HTMLAudioElement) {
    // Reusable voiceAudio already attached
  }

  public detachVoiceAudio() {
    // Reusable voiceAudio remains alive; simply stopping playback is sufficient
    this.stopVoice();
  }

  public isVoiceAudioActive(): boolean {
    return this.isVoicePlaying || !!(this.voiceAudio && !this.voiceAudio.paused && !this.voiceAudio.ended);
  }

  public getVoiceLipWeights(): MobileLipWeights {
    const zero: MobileLipWeights = { aa: 0, ih: 0, ou: 0, ee: 0, oh: 0, mouthOpen: 0 };
    if (!this.isVoiceAudioActive()) {
      return zero;
    }

    if (!this.voiceAnalyser || !this.voiceFreqData || !this.voiceTimeData) {
      // Lightweight sinusoidal fallback if Analyser not ready
      const cur = this.voiceAudio ? this.voiceAudio.currentTime : Date.now() / 1000;
      const open = Math.pow(Math.abs(Math.sin(cur * 16.0)), 1.3) * 0.55;
      return { aa: open, ih: 0, ou: 0, ee: 0, oh: 0, mouthOpen: open };
    }

    this.voiceAnalyser.getByteTimeDomainData(this.voiceTimeData as any);
    this.voiceAnalyser.getByteFrequencyData(this.voiceFreqData as any);

    // 1. RMS Energy from time-domain waveform (128 is zero baseline)
    let sumSq = 0;
    const len = this.voiceTimeData.length;
    for (let i = 0; i < len; i++) {
      const norm = (this.voiceTimeData[i] - 128) / 128;
      sumSq += norm * norm;
    }
    const rms = Math.sqrt(sumSq / len);

    // Noise floor: if below 0.015, treat as silent / breath pause between words
    if (rms < 0.015) {
      return zero;
    }

    const mouthOpen = Math.min(0.65, Math.pow(Math.max(0, (rms - 0.015) * 6.5), 0.85) * 0.65);

    // 2. Formant Energy Bands from FFT Frequency Data:
    // (sampleRate ~44100-48000 Hz, fftSize 512 => 256 bins, width bin ~86-94 Hz)
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

    // AIUEO vocal viseme weights (anime-proportioned, anti-overextended mouth)
    const aa = Math.min(1.0, rMid * 1.3) * mouthOpen;
    const ih = Math.min(1.0, rHigh * 1.6) * mouthOpen * 0.75;
    const ou = Math.min(1.0, Math.max(0, rLow - rMid * 0.3) * 1.5) * mouthOpen * 0.70;
    const ee = Math.min(1.0, rMidHigh * 1.5) * mouthOpen * 0.80;
    const oh = Math.min(1.0, (rLow * 0.65 + rMid * 0.45) * 1.3) * mouthOpen * 0.85;

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

  public destroy() {
    this.stopAll();
    if (this.audioCtx) {
      try {
        this.audioCtx.close();
      } catch {}
      this.audioCtx = null;
    }
  }
}

export const globalAudioSync = new MobileAudioSyncManager();
