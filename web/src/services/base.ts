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
  const id = timeoutMs > 0 ? setTimeout(() => {
    controller.abort(new Error(`Permintaan timeout setelah ${timeoutMs / 1000} detik`));
  }, timeoutMs) : null;

  try {
    const res = await fetch(url, { ...options, signal: controller.signal });
    if (id) clearTimeout(id);
    return res;
  } catch (err: any) {
    if (id) clearTimeout(id);
    if (err?.name === 'AbortError' && controller.signal.reason instanceof Error) {
      throw controller.signal.reason;
    }
    throw err;
  }
}
