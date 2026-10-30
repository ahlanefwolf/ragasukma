import { createSignal } from 'solid-js';

// In smartphone / remote tunnel, window.location.origin is https://xxx.trycloudflare.com
// Do not hardcode http://127.0.0.1:42128 because mobile devices cannot directly access laptop localhost!
export const API_BASE = typeof window !== 'undefined' && window.location.origin && !window.location.origin.includes('127.0.0.1') && !window.location.origin.includes('localhost')
  ? window.location.origin
  : 'http://127.0.0.1:42128';

const [isSessionKilled, setIsSessionKilled] = createSignal(false);

export function killCurrentSession() {
  setIsSessionKilled(true);
}

export async function fetchWithTimeout(url: string, options: RequestInit = {}, timeoutMs = 8000): Promise<Response> {
  if (isSessionKilled()) {
    throw new Error('Session is killed in this tab');
  }
  const controller = new AbortController();
  // NOTE: a `throw` inside setTimeout does NOT cancel the request — it only
  // produces an uncaught global error while the fetch keeps hanging forever.
  // Aborting the controller is what actually settles the promise.
  const id = timeoutMs > 0
    ? setTimeout(() => {
        controller.abort(new Error(`Request timed out after ${timeoutMs / 1000} seconds`));
      }, timeoutMs)
    : null;

  // If the caller supplied its own signal, honour it too.
  const externalSignal = options.signal;
  if (externalSignal) {
    if (externalSignal.aborted) {
      controller.abort(externalSignal.reason);
    } else {
      externalSignal.addEventListener('abort', () => controller.abort(externalSignal.reason), { once: true });
    }
  }

  try {
    return await fetch(url, { ...options, signal: controller.signal });
  } catch (err: any) {
    if (controller.signal.reason instanceof Error) {
      throw controller.signal.reason;
    }
    if (err?.name === 'AbortError') {
      throw new Error(`Request timed out after ${timeoutMs / 1000} seconds`);
    }
    throw err;
  } finally {
    if (id) clearTimeout(id);
  }
}