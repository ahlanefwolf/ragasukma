import { API_BASE } from './base';
import { configService, UserConfig, getChromeSttCode } from './configService';
import { duckMusic, unduckMusic } from '../playerStore';

export interface VoiceSessionOptions {
  isDead: () => boolean;
  isMobileClient: () => boolean;
}

export class VoiceSessionManager {
  private isDead: () => boolean;
  private isMobileClient: () => boolean;

  // Runtime State Flags
  private runtimeConfig: UserConfig | null = null;
  private recognition: any = null;
  private isTtsSpeaking = false;
  private isTestVoiceActive = false;
  private isListeningActive = false;
  private isVoiceMuted = false;
  private isMicMuted = false;
  private isAssistantRunning = false;
  private isRecognitionRunning = false;
  private isRecognitionStarting = false;
  private isWaitingForUserGesture = false;
  private startingTimeoutId: any = null;
  private listeningWatchdogTimer: any = null;
  private removeGlobalGestureListeners: (() => void) | null = null;

  // VAD Audio Nodes
  private vadAudioCtx: AudioContext | null = null;
  private vadStream: MediaStream | null = null;
  private vadInterval: any = null;

  // Echo Guard Buffer
  private assistantSpeechHistory: Array<{ text: string; tokens: string[]; timestamp: number }> = [];

  // TTS Deduplication & Session Isolation
  private lastSpokenText = '';
  private lastSpokenTime = 0;
  private speakSessionId = 0;
  private nativeSpeakTimer: any = null;

  constructor(options: VoiceSessionOptions) {
    this.isDead = options.isDead;
    this.isMobileClient = options.isMobileClient;

    if (!this.isMobileClient() && typeof window !== 'undefined') {
      // Inisialisasi SpeechRecognition secara sinkron sejak detik 0
      this.initSpeechRecognition();
      this.setupGlobalGestureListeners();
    }
  }

  private setupGlobalGestureListeners() {
    const onUserGesture = () => {
      // 1. Resume Web Audio context jika sempat suspended oleh autoplay policy
      if (this.vadAudioCtx && this.vadAudioCtx.state === 'suspended') {
        this.vadAudioCtx.resume().catch(() => {});
      }

      // 2. Jika pengenalan suara sempat tertahan izin browser (not-allowed / gesture required)
      if (this.isWaitingForUserGesture) {
        this.isWaitingForUserGesture = false;
        console.log('[Chrome Web STT] Interaksi pengguna terdeteksi, memulihkan sesi mikrofon...');
        this.ensureMicrophonePermission().then(() => {
          this.startRecognitionSafely();
        });
      } else if (
        this.isAssistantRunning &&
        !this.isMicMuted &&
        !this.isTtsSpeaking &&
        !this.isRecognitionRunning &&
        !this.isRecognitionStarting
      ) {
        // Recovery otomatis jika sesi asisten aktif tapi recognition belum jalan
        this.startRecognitionSafely();
      }
    };

    window.addEventListener('pointerdown', onUserGesture, { passive: true });
    window.addEventListener('keydown', onUserGesture, { passive: true });

    this.removeGlobalGestureListeners = () => {
      window.removeEventListener('pointerdown', onUserGesture);
      window.removeEventListener('keydown', onUserGesture);
    };
  }

  public async init() {
    if (this.isMobileClient() || typeof window === 'undefined') return;

    if (!this.recognition) {
      this.initSpeechRecognition();
    }

    try {
      this.runtimeConfig = await configService.getConfig();
      if (this.recognition && this.runtimeConfig?.lang) {
        this.recognition.lang = getChromeSttCode(this.runtimeConfig.lang);
      }
    } catch (_) {}

    try {
      const res = await fetch(`${API_BASE}/api/state`);
      if (res.ok) {
        const initState = await res.json();
        if (initState && typeof initState.is_running === 'boolean') {
          this.isAssistantRunning = initState.is_running;
        }
        if (initState && (typeof initState.voice_muted === 'boolean' || typeof initState.mic_muted === 'boolean')) {
          this.updateMuteState(Boolean(initState.voice_muted), Boolean(initState.mic_muted));
        }
      }
    } catch (_) {}

    // Jika asisten aktif (misal setelah F5 refresh saat sesi sedang berjalan), segera jalankan STT!
    if (this.isAssistantRunning && !this.isMicMuted && !this.isTtsSpeaking && !this.isRecognitionRunning) {
      console.log('[Voice Session] Auto-starting recognition saat inisialisasi sesi...');
      this.startRecognitionSafely();
      this.ensureMicrophonePermission();
    }
  }

  // ---------------------------------------------------------------------------
  // 1. Anti-Echolalia Echo Guard
  // ---------------------------------------------------------------------------
  // 1. Anti-Echolalia Echo Guard (Acoustic Speaker Feedback Protection)
  // ---------------------------------------------------------------------------
  private recordAssistantSpeech(rawText: string) {
    if (!rawText) return;
    const cleanTokens = rawText
      .toLowerCase()
      .replace(/[^a-z0-9\s]/g, ' ')
      .split(/\s+/)
      .filter((w) => w.length >= 3);

    this.assistantSpeechHistory.push({
      text: rawText.toLowerCase(),
      tokens: cleanTokens,
      timestamp: Date.now(),
    });

    // Pantulan akustik speaker ke mikrofon dipantau hingga 15 detik setelah suara diputar
    const cutoff = Date.now() - 15000;
    while (
      this.assistantSpeechHistory.length > 8 ||
      (this.assistantSpeechHistory.length > 0 && this.assistantSpeechHistory[0].timestamp < cutoff)
    ) {
      this.assistantSpeechHistory.shift();
    }
  }

  private isEchoOfAssistantSpeech(transcript: string): boolean {
    const now = Date.now();
    // Periksa pantulan akustik hingga 15 detik terakhir
    const activeEntries = this.assistantSpeechHistory.filter((e) => now - e.timestamp < 15000);
    if (activeEntries.length === 0) return false;

    const transLower = transcript.toLowerCase().trim();
    if (!transLower) return false;

    // 1. Cek kecocokan langsung atau substring lengkap
    for (const entry of activeEntries) {
      if (entry.text.includes(transLower) || transLower.includes(entry.text)) {
        console.warn('[Echo Guard] Terdeteksi substring pantulan suara asisten:', transLower);
        return true;
      }
    }

    const transTokens = transLower
      .replace(/[^a-z0-9\s]/g, ' ')
      .split(/\s+/)
      .filter((w) => w.length >= 3);

    if (transTokens.length === 0) return false;

    // 2. Cek Trigram (3 kata berurutan) yang identik dengan teks asisten
    if (transTokens.length >= 3) {
      for (let i = 0; i <= transTokens.length - 3; i++) {
        const trigram = `${transTokens[i]} ${transTokens[i + 1]} ${transTokens[i + 2]}`;
        for (const entry of activeEntries) {
          if (entry.text.includes(trigram)) {
            console.warn('[Echo Guard] Terdeteksi pantulan trigram speaker asisten:', trigram);
            return true;
          }
        }
      }
    }

    // 3. Cek Token Overlap Ratio (> 50%)
    const historyTokenSet = new Set<string>();
    for (const entry of activeEntries) {
      for (const t of entry.tokens) {
        historyTokenSet.add(t);
      }
    }

    let matchCount = 0;
    for (const token of transTokens) {
      if (historyTokenSet.has(token)) {
        matchCount++;
      }
    }

    const matchRatio = matchCount / transTokens.length;
    if (transTokens.length >= 3 && matchRatio >= 0.50) {
      console.warn(
        `[Echo Guard] Terdeteksi overlap tinggi (${Math.round(matchRatio * 100)}%) pantulan speaker:`,
        transcript
      );
      return true;
    }

    return false;
  }

  // ---------------------------------------------------------------------------
  // 2. Speech Recognition (STT) Setup
  // ---------------------------------------------------------------------------
  private initSpeechRecognition() {
    const SpeechRecognition = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (!SpeechRecognition) {
      console.warn('[Chrome Web STT] Web Speech API tidak didukung di browser ini.');
      return;
    }

    try {
      this.recognition = new SpeechRecognition();
      this.recognition.continuous = true;
      this.recognition.interimResults = false;
      this.recognition.lang = getChromeSttCode(this.runtimeConfig?.lang || 'id');

      this.recognition.onspeechstart = () => {
        if (this.isTestVoiceActive || this.isTtsSpeaking || !this.isAssistantRunning) return;

        if (!this.isMicMuted) {
          this.isListeningActive = true;
          this.cancelListeningWatchdog();
          // Watchdog pengaman: hanya aktif jika browser hang / tidak pernah emit onspeechend (15 detik)
          this.listeningWatchdogTimer = setTimeout(() => {
            this.resetToIdle('watchdog timeout 15s');
          }, 15000);

          fetch(`${API_BASE}/api/tts/state`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ state: 'listening', emotion: 'neutral' }),
          }).catch(() => {});
        }
      };

      this.recognition.onspeechend = () => {
        // Pengguna selesai berbicara, beri jeda 1.2 detik untuk STT menghasilkan transkrip
        // Jika tidak ada kata yang ter-generate (cuma hembusan nafas/batuk), otomatis reset ke idle
        this.cancelListeningWatchdog();
        this.listeningWatchdogTimer = setTimeout(() => {
          this.resetToIdle('speech ended without transcript');
        }, 1200);
      };

      this.recognition.onnomatch = () => {
        this.resetToIdle('no speech match');
      };

      this.recognition.onresult = (event: any) => {
        if (this.isMicMuted || this.isTestVoiceActive || this.isTtsSpeaking || !this.isAssistantRunning) {
          console.log('[Chrome Web STT] Hasil diabaikan karena mic mute / TTS aktif / assistant standby');
          this.resetToIdle('result ignored');
          return;
        }

        for (let i = event.resultIndex; i < event.results.length; ++i) {
          if (event.results[i].isFinal) {
            const transcript = event.results[i][0].transcript?.trim();
            if (transcript) {
              this.cancelListeningWatchdog();
              if (this.isEchoOfAssistantSpeech(transcript)) {
                console.warn('[Echo Guard] Self-echo pantulan speaker diabaikan:', transcript);
                this.resetToIdle('echo guard matched');
                continue;
              }

              // Matikan status listening aktif lokal tanpa mengirimkan 'idle' ke backend!
              // Menghindari race condition di mana request 'idle' menimpa status 'Thinking' di backend
              this.isListeningActive = false;
              console.log('[Chrome Web STT] Sending transcript:', transcript);
              fetch(`${API_BASE}/api/stt/transcript`, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ text: transcript }),
              }).catch((err) => console.error('[Chrome Web STT] Failed to send transcript:', err));
            }
          }
        }
      };

      this.recognition.onstart = () => {
        this.isRecognitionStarting = false;
        this.isRecognitionRunning = true;
        this.isWaitingForUserGesture = false;
        if (this.startingTimeoutId) {
          clearTimeout(this.startingTimeoutId);
          this.startingTimeoutId = null;
        }
        console.log('[Chrome Web STT] Speech recognition started & active');
      };

      this.recognition.onerror = (e: any) => {
        this.isRecognitionStarting = false;
        if (this.startingTimeoutId) {
          clearTimeout(this.startingTimeoutId);
          this.startingTimeoutId = null;
        }

        // 'aborted' dan 'no-speech' adalah event normal lifecycle Web Speech API:
        // - 'aborted': dipicu ketika aplikasi sengaja memanggil recognition.abort() (misal saat TTS mulai bicara atau mic mute).
        // - 'no-speech': pengguna hening / tidak bersuara sebelum batas waktu hening.
        if (e.error === 'aborted' || e.error === 'no-speech') {
          return;
        }

        if (e.error === 'network') {
          console.warn('[Chrome Web STT] Gangguan jaringan Google STT terdeteksi, memulihkan koneksi STT...');
          setTimeout(() => {
            if (!this.isDead() && !this.isMicMuted && !this.isTtsSpeaking && this.isAssistantRunning) {
              this.startRecognitionSafely();
            }
          }, 1500);
          return;
        }

        console.warn('[Chrome Web STT] Error:', e.error);
        this.resetToIdle('recognition error: ' + e.error);

        if (e.error === 'not-allowed' || e.error === 'service-not-allowed') {
          this.isWaitingForUserGesture = true;
          this.isRecognitionRunning = false;
          console.warn(
            '[Chrome Web STT] Akses mikrofon terblokir oleh browser. Menunggu interaksi pengguna (klik/tombol)...'
          );
        }
      };

      this.recognition.onend = () => {
        this.isRecognitionStarting = false;
        this.isRecognitionRunning = false;
        if (this.startingTimeoutId) {
          clearTimeout(this.startingTimeoutId);
          this.startingTimeoutId = null;
        }

        if (this.isTtsSpeaking) {
          // Asisten sedang aktif berbicara lewat TTS, jangan ganggu state ke idle dan jangan restart listening
          return;
        }
        this.resetToIdle('recognition ended');

        // Jangan restart jika sedang menunggu gesture user (not-allowed)
        if (this.isWaitingForUserGesture) {
          return;
        }

        const shouldListen = !this.isDead() && !this.isMicMuted && !this.isTtsSpeaking && this.isAssistantRunning;
        if (shouldListen) {
          setTimeout(() => {
            this.startRecognitionSafely();
          }, 200);
        }
      };

      if (!this.isMicMuted && this.isAssistantRunning) {
        this.startRecognitionSafely();
        this.ensureMicrophonePermission();
      }
    } catch (err) {
      console.warn('[Chrome Web STT] Init failed:', err);
    }
  }

  public updateAssistantRunning(running: boolean) {
    const isStateSame = this.isAssistantRunning === running;
    this.isAssistantRunning = running;

    if (running) {
      // Jika running tapi recognition belum aktif (atau state baru berubah), jalankan recognition!
      if (!this.isMicMuted && (!this.isRecognitionRunning || !isStateSame)) {
        console.log('[Voice Session] Assistant running -> starting recognition...');
        this.startRecognitionSafely();
        this.ensureMicrophonePermission();
      }
    } else {
      if (!isStateSame) {
        console.log('[Voice Session] Assistant stopped -> stopping recognition...');
      }
      this.resetToIdle('assistant stopped / standby');
      this.stopRecognition();
    }
  }

  public stopRecognition() {
    this.cancelListeningWatchdog();
    this.isWaitingForUserGesture = false;
    this.isRecognitionRunning = false;
    this.isRecognitionStarting = false;
    if (this.startingTimeoutId) {
      clearTimeout(this.startingTimeoutId);
      this.startingTimeoutId = null;
    }
    if (this.recognition) {
      try {
        this.recognition.abort();
      } catch (_) {}
    }
  }

  private cancelListeningWatchdog() {
    if (this.listeningWatchdogTimer) {
      clearTimeout(this.listeningWatchdogTimer);
      this.listeningWatchdogTimer = null;
    }
  }

  private resetToIdle(reason: string) {
    this.cancelListeningWatchdog();
    if (this.isListeningActive) {
      this.isListeningActive = false;
      if (this.isTtsSpeaking) {
        console.log(`[Chrome Web STT] resetToIdle diabaikan karena TTS sedang aktif berbicara (${reason})`);
        return;
      }
      console.log(`[Chrome Web STT] Reset listening ke idle (${reason})`);
      fetch(`${API_BASE}/api/tts/state`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ state: 'idle', emotion: 'neutral' }),
      }).catch(() => {});
    }
  }

  public startRecognitionSafely() {
    if (
      this.isDead() ||
      this.isMicMuted ||
      !this.isAssistantRunning ||
      this.isTtsSpeaking ||
      this.isRecognitionRunning ||
      this.isRecognitionStarting ||
      this.isWaitingForUserGesture
    ) {
      return;
    }

    if (!this.recognition) {
      this.initSpeechRecognition();
      if (!this.recognition) return;
    }

    try {
      this.isRecognitionStarting = true;
      if (this.startingTimeoutId) {
        clearTimeout(this.startingTimeoutId);
      }
      this.startingTimeoutId = setTimeout(() => {
        this.isRecognitionStarting = false;
        this.startingTimeoutId = null;
      }, 500);

      this.recognition.start();
    } catch (e: any) {
      this.isRecognitionStarting = false;
      if (this.startingTimeoutId) {
        clearTimeout(this.startingTimeoutId);
        this.startingTimeoutId = null;
      }
      if (e?.name === 'InvalidStateError') {
        this.isRecognitionRunning = true;
      } else {
        console.warn('[Chrome Web STT] Start failed:', e);
      }
    }
  }

  // ---------------------------------------------------------------------------
  // 3. VAD Barge-In Monitoring
  // ---------------------------------------------------------------------------
  public stopVadMonitoring() {
    if (this.vadInterval) {
      clearInterval(this.vadInterval);
      this.vadInterval = null;
    }
    if (this.vadStream) {
      this.vadStream.getTracks().forEach((t) => t.stop());
      this.vadStream = null;
    }
    if (this.vadAudioCtx) {
      try {
        this.vadAudioCtx.close();
      } catch {}
      this.vadAudioCtx = null;
    }
  }

  private async startVadBargeInMonitoring() {
    if (this.vadInterval) return;
    try {
      const stream = await navigator.mediaDevices.getUserMedia({
        audio: {
          noiseSuppression: this.runtimeConfig?.vad_noise_suppression !== false,
          echoCancellation: true,
          autoGainControl: true,
        },
      });
      this.vadStream = stream;
      const ctx = new (window.AudioContext || (window as any).webkitAudioContext)();
      this.vadAudioCtx = ctx;
      const source = ctx.createMediaStreamSource(stream);
      const analyser = ctx.createAnalyser();
      analyser.fftSize = 512;
      source.connect(analyser);

      const buffer = new Uint8Array(analyser.frequencyBinCount);
      let consecutiveAbove = 0;

      this.vadInterval = setInterval(() => {
        if (this.isDead() || this.isMicMuted) return;
        if (!this.isTtsSpeaking || !this.runtimeConfig?.voice_interruption) {
          consecutiveAbove = 0;
          return;
        }

        analyser.getByteTimeDomainData(buffer);
        let sumSquares = 0;
        for (let i = 0; i < buffer.length; i++) {
          const norm = (buffer[i] - 128) / 128;
          sumSquares += norm * norm;
        }
        const rms = Math.sqrt(sumSquares / buffer.length);
        const threshold =
          this.runtimeConfig?.vad_threshold !== undefined ? this.runtimeConfig.vad_threshold : 0.15;

        if (rms >= threshold) {
          consecutiveAbove++;
          if (consecutiveAbove >= 2) {
            consecutiveAbove = 0;
            if (this.isMicMuted) return;
            console.log(
              `⚡ [VAD Barge-in] Suara user (${Math.round(rms * 100)}%) melampaui batas (${Math.round(
                threshold * 100
              )}%), mematikan suara asisten!`
            );
            this.speakSessionId++;
            if ((window as any).__activeSupertonicAudio) {
              try {
                (window as any).__activeSupertonicAudio.pause();
                (window as any).__activeSupertonicAudio = null;
              } catch {}
            }
            this.isTtsSpeaking = false;
            fetch(`${API_BASE}/api/tts/state`, {
              method: 'POST',
              headers: { 'Content-Type': 'application/json' },
              body: JSON.stringify({ speaking: false, state: 'listening' }),
            }).catch(() => {});
            setTimeout(() => {
              if (!this.isDead() && !this.isMicMuted && !this.isTtsSpeaking) {
                this.startRecognitionSafely();
              }
            }, 200);
          }
        } else {
          consecutiveAbove = 0;
        }
      }, 60);
    } catch (err: any) {
      console.warn('[VAD Barge-in] Inisialisasi analyser audio:', err?.message || err);
    }
  }

  private async ensureMicrophonePermission() {
    if (typeof navigator !== 'undefined' && 'mediaDevices' in navigator) {
      try {
        await this.startVadBargeInMonitoring();
        console.log('[Chrome Web STT] Izin audio & VAD analyser berhasil aktif');
        this.startRecognitionSafely();
      } catch (err: any) {
        console.warn('[Chrome Web STT] getUserMedia permission check:', err?.message || err);
      }
    }
  }

  // ---------------------------------------------------------------------------
  // 4. Speech Synthesis (Supertonic-3 Neural ONNX Audio Stream)
  // ---------------------------------------------------------------------------
  public handleAssistantSpeak(payload: any) {
    if (this.isMobileClient()) return;
    if (this.isVoiceMuted) {
      console.log('[TTS] Diabaikan di desktop karena audio desktop dimatikan (Full Mobile aktif)');
      return;
    }

    let textToSpeak = '';
    let speakEmotion = 'neutral';
    let audioUrl: string | null = null;
    let isNativePlayed = false;
    let durationSecs = 0;
    try {
      const parsed = typeof payload === 'string' ? JSON.parse(payload) : payload;
      textToSpeak = parsed.text || '';
      speakEmotion = parsed.emotion || 'neutral';
      audioUrl = parsed.audio_url || null;
      isNativePlayed = !!parsed.native_played;
      durationSecs = Number(parsed.duration) || 0;
    } catch {
      textToSpeak = typeof payload === 'string' ? payload : '';
    }
    textToSpeak = textToSpeak.trim();
    if (!textToSpeak && !audioUrl) return;

    const now = Date.now();
    if (textToSpeak && textToSpeak === this.lastSpokenText && now - this.lastSpokenTime < 1500) {
      console.warn('[TTS] Duplicate speak action suppressed:', textToSpeak);
      return;
    }
    if (textToSpeak) {
      this.lastSpokenText = textToSpeak;
      this.lastSpokenTime = now;
      this.recordAssistantSpeech(textToSpeak);
    }

    const sessionId = ++this.speakSessionId;

    if (isNativePlayed) {
      // Backend memutar audio langsung ke hardware speaker PC
      this.isTtsSpeaking = true;
      duckMusic(0.25);
      this.cancelListeningWatchdog();
      this.isListeningActive = false;
      try {
        this.recognition?.abort();
      } catch {}

      // Kunci microphone selama suara berputar di speaker PC ditambah jeda 400ms agar gaung hilang
      const waitMs =
        durationSecs > 0 ? durationSecs * 1000 + 400 : Math.max(3000, textToSpeak.length * 70);
      clearTimeout(this.nativeSpeakTimer);
      this.nativeSpeakTimer = setTimeout(() => {
        if (sessionId !== this.speakSessionId) return;
        this.isTtsSpeaking = false;
        unduckMusic();
        if (!this.isDead() && !this.isMicMuted && this.isAssistantRunning) {
          this.startRecognitionSafely();
        }
      }, waitMs);
      return;
    }

    // Hentikan pemutaran audio Supertonic sebelumnya jika ada
    try {
      if ((window as any).__activeSupertonicAudio) {
        (window as any).__activeSupertonicAudio.pause();
        (window as any).__activeSupertonicAudio = null;
      }
    } catch {}

    const playAudioUrl = (url: string, revokeOnEnd = false) => {
      this.isTtsSpeaking = true;
      this.cancelListeningWatchdog();
      this.isListeningActive = false;
      try {
        this.recognition?.abort();
      } catch {}

      const audio = new Audio(url);
      (window as any).__activeSupertonicAudio = audio;
      duckMusic(0.25);

      let hasReportedSpeaking = false;
      const notifySpeakingStart = () => {
        if (hasReportedSpeaking || sessionId !== this.speakSessionId) return;
        hasReportedSpeaking = true;
        // Beritahu backend bahwa suara SUDAH benar-benar berbunyi dari speaker (bukan masih loading)
        // Ini menjamin gerak bibir AIUEO sinkron 100% dengan gelombang suara
        fetch(`${API_BASE}/api/tts/state`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ speaking: true, emotion: speakEmotion }),
        }).catch(() => {});
      };

      audio.onplaying = notifySpeakingStart;
      audio.onplay = notifySpeakingStart;

      const finishSupertonic = () => {
        if (sessionId !== this.speakSessionId) return;
        this.isTtsSpeaking = false;
        unduckMusic();
        (window as any).__activeSupertonicAudio = null;
        if (revokeOnEnd) {
          try {
            URL.revokeObjectURL(url);
          } catch {}
        }

        fetch(`${API_BASE}/api/tts/state`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ speaking: false, emotion: 'neutral' }),
        }).catch(() => {});

        setTimeout(() => {
          if (!this.isDead() && !this.isTtsSpeaking && !this.isMicMuted && this.isAssistantRunning) {
            this.startRecognitionSafely();
          }
        }, 500);
      };

      audio.onended = finishSupertonic;
      audio.onerror = (e) => {
        console.warn('[Supertonic Audio] Playback error:', e);
        finishSupertonic();
      };

      audio.play().catch((err) => {
        console.warn('[Supertonic Audio] Autoplay blocked/interrupted:', err);
        finishSupertonic();
      });
    };

    if (audioUrl) {
      const fullUrl = audioUrl.startsWith('http') ? audioUrl : `${API_BASE}${audioUrl}`;
      playAudioUrl(fullUrl, false);
      return;
    }

    // Jika audio_url belum ada, minta backend mensintesis langsung via Supertonic
    fetch(`${API_BASE}/api/tts/synthesize`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        text: textToSpeak,
        voice: this.runtimeConfig?.tts_voice_name || 'F4',
        lang: this.runtimeConfig?.lang || 'id',
        speed: this.runtimeConfig?.tts_speed || 1.05,
      }),
    })
      .then((res) => {
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return res.blob();
      })
      .then((blob) => {
        const url = URL.createObjectURL(blob);
        playAudioUrl(url, true);
      })
      .catch((err) => {
        console.warn('[Supertonic Audio] Synthesize fetch error:', err);
        this.isTtsSpeaking = false;
      });
  }

  // ---------------------------------------------------------------------------
  // 5. External Event Synchronizers
  // ---------------------------------------------------------------------------
  public updateMuteState(voiceMuted: boolean, micMuted: boolean) {
    const wasVoiceMuted = this.isVoiceMuted;
    this.isVoiceMuted = voiceMuted;

    if (this.isVoiceMuted && !wasVoiceMuted) {
      if ((window as any).__activeSupertonicAudio) {
        try {
          (window as any).__activeSupertonicAudio.pause();
          (window as any).__activeSupertonicAudio = null;
        } catch {}
      }
      this.isTtsSpeaking = false;
    }

    if (this.isMicMuted !== micMuted) {
      this.isMicMuted = micMuted;
      if (this.isMicMuted) {
        try {
          this.recognition?.abort();
        } catch {}
      } else if (!this.isTtsSpeaking && !this.isDead()) {
        try {
          this.recognition?.start();
        } catch {}
      }
    }
  }

  public handleTestVoice(active: boolean) {
    this.isTestVoiceActive = active;
    if (active) {
      try {
        this.recognition?.abort();
      } catch {}
    } else {
      setTimeout(() => {
        if (!this.isDead() && !this.isMicMuted && !this.isTtsSpeaking) {
          try {
            this.recognition?.start();
          } catch {}
        }
      }, 300);
    }
  }

  public handleConfigUpdated(cfg: UserConfig) {
    this.runtimeConfig = cfg;
    const targetStt = getChromeSttCode(cfg?.lang || 'id');
    if (this.recognition && this.recognition.lang !== targetStt) {
      this.recognition.lang = targetStt;
      console.log('[Chrome Web STT] Reaktif ganti bahasa STT ke:', targetStt);
      try {
        this.recognition.stop();
      } catch {}
    }

    if (this.vadStream && cfg?.vad_noise_suppression !== undefined) {
      this.vadStream.getAudioTracks().forEach((track) => {
        try {
          track.applyConstraints({
            noiseSuppression: cfg.vad_noise_suppression !== false,
          });
        } catch (_) {}
      });
    }
  }

  public getRecognition() {
    return this.recognition;
  }

  public destroy() {
    this.cancelListeningWatchdog();
    this.stopVadMonitoring();
    this.removeGlobalGestureListeners?.();
    this.removeGlobalGestureListeners = null;
    if (this.startingTimeoutId) {
      clearTimeout(this.startingTimeoutId);
      this.startingTimeoutId = null;
    }
    if (this.nativeSpeakTimer) {
      clearTimeout(this.nativeSpeakTimer);
      this.nativeSpeakTimer = null;
    }
    if ((window as any).__activeSupertonicAudio) {
      try {
        (window as any).__activeSupertonicAudio.pause();
        (window as any).__activeSupertonicAudio = null;
      } catch {}
    }
    if (this.recognition) {
      try {
        this.recognition.onstart = null;
        this.recognition.onend = null;
        this.recognition.onerror = null;
        this.recognition.onresult = null;
        this.recognition.onspeechstart = null;
        this.recognition.onspeechend = null;
        this.recognition.onnomatch = null;
        this.recognition.abort();
      } catch {}
      this.recognition = null;
    }
    this.isRecognitionRunning = false;
    this.isRecognitionStarting = false;
    this.isWaitingForUserGesture = false;
  }
}
