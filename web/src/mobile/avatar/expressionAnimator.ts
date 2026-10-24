import * as THREE from 'three';
import { VRM } from '@pixiv/three-vrm';
import { ALL_KNOWN_EXPRESSION_KEYS } from './constants';
import { AvatarState } from './types';
import { globalAudioSync } from '../AudioSyncManager';

export class VrmExpressionAnimator {
  private currentExpressionWeights: Record<string, number> = {};

  public reset(): void {
    this.currentExpressionWeights = {};
  }

  public update(
    vrm: VRM,
    delta: number,
    currentState: AvatarState,
    emotionProp: string | undefined,
    stateElapsedTime: number
  ): void {
    const expressions = vrm.expressionManager;
    if (!expressions) return;

    // Synchronized with desktop app.rs: when Thinking state, expression automatically overrides to thinking
    const currentEmotion = currentState === 'thinking' ? 'thinking' : (emotionProp || 'neutral').toLowerCase();

    const targetWeights: Record<string, number> = {};
    for (const k of ALL_KNOWN_EXPRESSION_KEYS) {
      targetWeights[k] = 0.0;
    }

    let hasStrongEmotion = false;

    // 1. Main Emotion Layer (100% identical to resolve_emotion_layers in expressions.rs)
    switch (currentEmotion) {
      case 'happy':
      case 'senang':
        targetWeights['happy'] = 0.75;
        hasStrongEmotion = true;
        break;

      case 'shy':
      case 'malu':
        targetWeights['happy'] = 0.40;
        targetWeights['blink'] = 0.30;
        hasStrongEmotion = true;
        break;

      case 'excited':
      case 'semangat':
        targetWeights['happy'] = 0.75;
        targetWeights['surprised'] = 0.15;
        hasStrongEmotion = true;
        break;

      case 'surprised':
      case 'kaget':
        targetWeights['surprised'] = 0.50;
        hasStrongEmotion = true;
        break;

      case 'thinking':
      case 'mikir':
        targetWeights['ou'] = 0.35;
        targetWeights['sad'] = 0.20;
        hasStrongEmotion = true;
        break;

      case 'sad':
      case 'sedih':
        targetWeights['sad'] = 0.85;
        hasStrongEmotion = true;
        break;

      case 'angry':
      case 'marah':
        targetWeights['angry'] = 0.85;
        hasStrongEmotion = true;
        break;

      case 'neutral':
      case 'netral':
        break;

      default:
        if ((ALL_KNOWN_EXPRESSION_KEYS as readonly string[]).includes(currentEmotion)) {
          targetWeights[currentEmotion] = 0.75;
          hasStrongEmotion = true;
        }
        break;
    }

    // 2. Relaxed natural eye blinking (Identical to expressions.rs)
    let naturalBlinkWeight = 0.0;
    const blinkPeriod = currentState === 'speaking' ? 6.2 : 5.2;
    const blinkPhase = stateElapsedTime % blinkPeriod;
    const blinkDuration = 0.24; // Durasi kedip ~240ms
    if (blinkPhase < blinkDuration) {
      const t = blinkPhase / blinkDuration;
      const blinkDamp = hasStrongEmotion ? 0.6 : 1.0;
      naturalBlinkWeight = Math.pow(Math.sin(t * Math.PI), 1.5) * blinkDamp;
    }
    if (naturalBlinkWeight > 0.01) {
      targetWeights['blink'] = Math.min(1.0, (targetWeights['blink'] || 0) + naturalBlinkWeight);
    }

    // 3. Audio-Driven Lip Sync Vowels (A, I, U, E, O) when Speaking
    if (currentState === 'speaking') {
      // When speaking, damp emotions that open jaw/mouth (surprised & happy)
      // to avoid exaggerated double-mouth opening with vocal Lip Sync
      if (targetWeights['surprised']) {
        targetWeights['surprised'] = Math.min(0.20, targetWeights['surprised'] * 0.30);
      }
      if (targetWeights['happy']) {
        targetWeights['happy'] = Math.min(0.50, targetWeights['happy'] * 0.70);
      }

      const lip = globalAudioSync.getVoiceLipWeights();
      if (lip.mouthOpen > 0.015) {
        // Active audio from speaker: apply AIUEO vocal formant weights
        targetWeights['aa'] = lip.aa;
        targetWeights['ih'] = lip.ih;
        targetWeights['ou'] = lip.ou;
        targetWeights['ee'] = lip.ee;
        targetWeights['oh'] = lip.oh;
      }
    }

    // Smooth interpolation damping (~100ms) for expressions & mouth to prevent jitter
    // Notes: 'blink' is applied directly following a smooth sine curve without lag dampening, ensuring 100% perfect eye closure matching desktop
    const keysToUpdate = new Set([...ALL_KNOWN_EXPRESSION_KEYS, ...Object.keys(this.currentExpressionWeights)]);
    for (const k of keysToUpdate) {
      const clampedTarget = Math.min(0.85, targetWeights[k] || 0.0);
      if (k === 'blink') {
        this.currentExpressionWeights[k] = clampedTarget;
      } else {
        const curVal = this.currentExpressionWeights[k] || 0.0;
        const tgtVal = clampedTarget;
        const newVal = THREE.MathUtils.damp(curVal, tgtVal, 14.0, delta);
        this.currentExpressionWeights[k] = Math.abs(newVal) < 0.001 ? 0.0 : newVal;
      }
      expressions.setValue(k, Math.min(0.85, this.currentExpressionWeights[k]));
    }

    expressions.update();
  }
}
