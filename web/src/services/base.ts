import { createSignal } from 'solid-js';

export const API_BASE = 'http://127.0.0.1:42128';

export const [isSessionKilled, setIsSessionKilled] = createSignal(false);

export function killCurrentSession() {
  setIsSessionKilled(true);
}

export async function fetchWithTimeout(url: string, options: RequestInit = {}, timeoutMs = 8000): Promise<Response> {
  if (isSessionKilled()) {
    throw new Error('Session is killed in this tab');
  }
  const controller = new AbortController();
  const id = setTimeout(() => controller.abort(), timeoutMs);
  try {
    const res = await fetch(url, { ...options, signal: controller.signal });
    clearTimeout(id);
    return res;
  } catch (err) {
    clearTimeout(id);
    throw err;
  }
}
