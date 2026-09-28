import { API_BASE } from './base';
import { configService, UserConfig } from './configService';

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
  private listeningWatchdogTimer: any = null;
  private isRecognitionStarting = false;

  // VAD Audio Nodes
  private vadAudioCtx: AudioContext | null = null;
  private vadStream: MediaStream | null = null;
  private vadInterval: any = null;

  // Echo Guard Buffer
  private assistantSpeechHistory: Array<{ text: string; tokens: string[]; timestamp: number }> = [];

  // TTS Deduplication
  private lastSpokenText = '';
  private lastSpokenTime = 0;

  constructor(options: VoiceSessionOptions) {
    this.isDead = options.isDead;
    this.isMobileClient = options.isMobileClient;
  }

  public async init() {
    if (this.isMobileClient() || typeof window === 'undefined') return;

    try {
      this.runtimeConfig = await configService.getConfig();
    } catch (_) {}

    try {
      const res = await fetch(`${API_BASE}/api/state`);
      if (res.ok) {
        const initState = await res.json();
        if (initState && typeof initState.is_running === 'boolean') {
          this.isAssistantRunning = initState.is_running;
        }
      }
    } catch (_) {}

    this.initSpeechRecognition();
  }

  // ---------------------------------------------------------------------------
  // 1. Anti-Echolalia Echo Guard
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

    const cutoff = Date.now() - 25000;
    while (
      this.assistantSpeechHistory.length > 8 ||
      (this.assistantSpeechHistory.length > 0 && this.assistantSpeechHistory[0].timestamp < cutoff)
    ) {
      this.assistantSpeechHistory.shift();
    }
  }

  private isEchoOfAssistantSpeech(transcript: string): boolean {
    const now = Date.now();
    const activeEntries = this.assistantSpeechHistory.filter((e) => now - e.timestamp < 25000);
    if (activeEntries.length === 0) return false;

    const transLower = transcript.toLowerCase();
    const transTokens = transLower
      .replace(/[^a-z0-9\s]/g, ' ')
      .split(/\s+/)
      .filter((w) => w.length >= 3);

    if (transTokens.length === 0) return false;

    // 1. Cek Trigram (3 kata berurutan) yang identik dengan teks asisten
    if (transTokens.length >= 3) {
      for (let i = 0; i <= transTokens.length - 3; i++) {
        const trigram = `${transTokens[i]} ${transTokens[i + 1]} ${transTokens[i + 2]}`;
        for (const entry of activeEntries) {
          if (entry.text.includes(trigram)) {
            console.warn('[Echo Guard] Terdeteksi trigram gema asisten:', trigram);
            return true;
          }
        }
      }
    }

    // 2. Cek Token Overlap Ratio
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
    if (transTokens.length >= 3 && matchRatio >= 0.35) {
      console.warn(
        `[Echo Guard] Terdeteksi overlap kata (${Math.round(matchRatio * 100)}%) gema asisten:`,
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
      this.recognition.lang = this.runtimeConfig?.stt_lang || 'id-ID';

      this.recognition.onspeechstart = () => {
        if (this.isTestVoiceActive || this.isTtsSpeaking || !this.isAssistantRunning) return;

        if (!this.isMicMuted) {
          this.isListeningActive = true;
          this.cancelListeningWatchdog();
          // Watchdog: jika dalam 3.5 detik tidak ada hasil teks final, paksa kembali ke idle
          this.listeningWatchdogTimer = setTimeout(() => {
            this.resetToIdle('watchdog timeout 3.5s');
          }, 3500);

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

              this.resetToIdle('transcript sent');
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
      };

      this.recognition.onerror = (e: any) => {
        this.isRecognitionStarting = false;
        console.warn('[Chrome Web STT] Error:', e.error);
        this.resetToIdle('recognition error: ' + e.error);
        if (e.error === 'not-allowed') {
          const unlockGesture = () => {
            console.log('[Chrome Web STT] Interaksi user terdeteksi, memulihkan sesi mikrofon...');
            this.ensureMicrophonePermission().then(() => this.startRecognitionSafely());
            window.removeEventListener('click', unlockGesture);
            window.removeEventListener('keydown', unlockGesture);
          };
          window.addEventListener('click', unlockGesture, { once: true });
          window.addEventListener('keydown', unlockGesture, { once: true });
        }
      };

      this.recognition.onend = () => {
        this.isRecognitionStarting = false;
        this.resetToIdle('recognition ended');
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
    if (this.isAssistantRunning === running) return;
    this.isAssistantRunning = running;
    console.log('[Voice Session] Assistant running status changed:', running);

    if (running) {
      if (!this.isMicMuted) {
        this.startRecognitionSafely();
        this.ensureMicrophonePermission();
      }
    } else {
      this.resetToIdle('assistant stopped / standby');
      this.stopRecognition();
    }
  }

  public stopRecognition() {
    this.cancelListeningWatchdog();
    if (this.recognition) {
      try {
        this.recognition.abort();
      } catch (_) {}
    }
    this.isRecognitionStarting = false;
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
      console.log(`[Chrome Web STT] Reset listening ke idle (${reason})`);
      fetch(`${API_BASE}/api/tts/state`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ state: 'idle', emotion: 'neutral' }),
      }).catch(() => {});
    }
  }

  public startRecognitionSafely() {
    if (this.isDead() || this.isMicMuted || !this.isAssistantRunning || this.isRecognitionStarting) return;
    try {
      this.isRecognitionStarting = true;
      this.recognition?.start();
    } catch (e: any) {
      if (e?.name !== 'InvalidStateError') {
        console.warn('[Chrome Web STT] Start failed:', e);
      }
    } finally {
      setTimeout(() => {
        this.isRecognitionStarting = false;
      }, 300);
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
            if (typeof window !== 'undefined' && 'speechSynthesis' in window) {
              window.speechSynthesis.cancel();
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
  // 4. Speech Synthesis (TTS)
  // ---------------------------------------------------------------------------
  public handleAssistantSpeak(payload: any) {
    if (this.isMobileClient()) return;
    if (this.isVoiceMuted) {
      console.log('[TTS] Diabaikan di desktop karena audio desktop dimatikan (Full Mobile aktif)');
      return;
    }

    let textToSpeak = '';
    let speakEmotion = 'neutral';
    try {
      const parsed = typeof payload === 'string' ? JSON.parse(payload) : payload;
      textToSpeak = parsed.text || '';
      speakEmotion = parsed.emotion || 'neutral';
    } catch {
      textToSpeak = typeof payload === 'string' ? payload : '';
    }
    textToSpeak = textToSpeak.trim();
    if (!textToSpeak) return;

    const now = Date.now();
    if (textToSpeak === this.lastSpokenText && now - this.lastSpokenTime < 1500) {
      console.warn('[TTS] Duplicate speak action suppressed:', textToSpeak);
      return;
    }
    this.lastSpokenText = textToSpeak;
    this.lastSpokenTime = now;
    this.recordAssistantSpeech(textToSpeak);

    if (textToSpeak && typeof window !== 'undefined' && 'speechSynthesis' in window) {
      window.speechSynthesis.cancel();
      this.isTtsSpeaking = true;
      try {
        this.recognition?.abort();
      } catch {}

      const splitSentences = (txt: string): string[] => {
        const matches = txt.match(/[^.!?]+[.!?]+(\s|$)|[^.!?]+$/g);
        return matches ? matches.map((s) => s.trim()).filter(Boolean) : [txt.trim()];
      };

      const sentences = splitSentences(textToSpeak);
      if (sentences.length === 0) {
        this.isTtsSpeaking = false;
        return;
      }

      const cfg = this.runtimeConfig;
      const voices = window.speechSynthesis.getVoices();
      const matchedVoice = cfg?.tts_voice_name ? voices.find((v) => v.name === cfg.tts_voice_name) : null;

      let currentIndex = 0;
      let keepAliveTimer: any = null;

      const finishAll = () => {
        if (keepAliveTimer) {
          clearInterval(keepAliveTimer);
          keepAliveTimer = null;
        }
        this.isTtsSpeaking = false;

        fetch(`${API_BASE}/api/tts/state`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ speaking: false, emotion: 'neutral' }),
        }).catch(() => {});

        setTimeout(() => {
          if (!this.isDead() && !this.isTtsSpeaking && !this.isMicMuted) {
            this.startRecognitionSafely();
          }
        }, 600);
      };

      const speakNextSentence = () => {
        if (this.isVoiceMuted) {
          finishAll();
          return;
        }
        if (currentIndex >= sentences.length) {
          finishAll();
          return;
        }

        const sentence = sentences[currentIndex];
        const utter = new SpeechSynthesisUtterance(sentence);
        if (matchedVoice) {
          utter.voice = matchedVoice;
          utter.lang = matchedVoice.lang;
        } else if (cfg?.tts_lang) {
          utter.lang = cfg.tts_lang;
        }

        utter.rate = cfg?.tts_speed || 1.0;
        utter.pitch = cfg?.tts_pitch || 1.0;

        utter.onstart = () => {
          if (keepAliveTimer) clearInterval(keepAliveTimer);
          keepAliveTimer = setInterval(() => {
            if (window.speechSynthesis.speaking) {
              window.speechSynthesis.pause();
              window.speechSynthesis.resume();
            }
          }, 4500);

          fetch(`${API_BASE}/api/tts/state`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ speaking: true, emotion: speakEmotion }),
          }).catch(() => {});
        };

        let safetyTimer: any = null;

        utter.onend = () => {
          if (safetyTimer) clearTimeout(safetyTimer);
          if (keepAliveTimer) {
            clearInterval(keepAliveTimer);
            keepAliveTimer = null;
          }
          currentIndex++;
          speakNextSentence();
        };

        utter.onerror = (e: any) => {
          if (safetyTimer) clearTimeout(safetyTimer);
          if (keepAliveTimer) {
            clearInterval(keepAliveTimer);
            keepAliveTimer = null;
          }
          if (e?.error === 'canceled' || e?.error === 'interrupted') {
            finishAll();
            return;
          }
          console.warn('[TTS] Sentence error:', e);
          currentIndex++;
          speakNextSentence();
        };

        // Safety fallback: jika Chrome Linux menahan onend, paksa lanjut setelah estimasi durasi selesai
        const estimatedDurationMs = Math.max(2000, (sentence.length / 15) * 1000 + 1500);
        safetyTimer = setTimeout(() => {
          console.warn('[TTS] Safety timer triggered for chunk');
          if (keepAliveTimer) {
            clearInterval(keepAliveTimer);
            keepAliveTimer = null;
          }
          currentIndex++;
          speakNextSentence();
        }, estimatedDurationMs);

        try {
          if (window.speechSynthesis.paused) {
            window.speechSynthesis.resume();
          }
          window.speechSynthesis.speak(utter);
        } catch (err) {
          console.warn('[TTS] Speak error:', err);
          if (safetyTimer) clearTimeout(safetyTimer);
          finishAll();
        }
      };

      speakNextSentence();
    }
  }

  // ---------------------------------------------------------------------------
  // 5. External Event Synchronizers
  // ---------------------------------------------------------------------------
  public updateMuteState(voiceMuted: boolean, micMuted: boolean) {
    const wasVoiceMuted = this.isVoiceMuted;
    this.isVoiceMuted = voiceMuted;

    if (this.isVoiceMuted && !wasVoiceMuted) {
      if (typeof window !== 'undefined' && 'speechSynthesis' in window) {
        try {
          window.speechSynthesis.cancel();
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
    if (cfg?.stt_lang && this.recognition && this.recognition.lang !== cfg.stt_lang) {
      this.recognition.lang = cfg.stt_lang;
      console.log('[Chrome Web STT] Reaktif ganti bahasa STT ke:', cfg.stt_lang);
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
    if (typeof window !== 'undefined' && 'speechSynthesis' in window) {
      try {
        window.speechSynthesis.cancel();
      } catch {}
    }
    if (this.recognition) {
      try {
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
  }
}
