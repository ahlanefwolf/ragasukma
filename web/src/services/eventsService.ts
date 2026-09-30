import { API_BASE } from './base';

// ---------------------------------------------------------------------------
// 1. APP STATE SSE STREAM (/api/events/state)
// Menggunakan event 'state' untuk visual AI dan event 'action' untuk trigger skill
// ---------------------------------------------------------------------------
export interface CoreStatePayload {
  state: string;
  emotion?: string;
  voice_muted?: boolean;
  mic_muted?: boolean;
  is_running?: boolean;
}

export interface ClientAction {
  id?: number;
  action: string;
  payload?: string;
}

type StateListener = (state: CoreStatePayload) => void;
type ActionListener = (action: ClientAction) => void;
type LogListener = (log: any) => void;
type ConnectionListener = (connected: boolean) => void;

let stateEventSource: EventSource | null = null;
const stateListeners = new Set<StateListener>();
const actionListeners = new Set<ActionListener>();
const logListeners = new Set<LogListener>();
const connectionListeners = new Set<ConnectionListener>();
let stateReconnectTimer: number | null = null;
let lastKnownState: CoreStatePayload | null = null;
let isConnected = false;

function startStateSSE() {
  if (stateEventSource || stateReconnectTimer) return;

  try {
    stateEventSource = new EventSource(`${API_BASE}/api/events/state`);

    stateEventSource.onopen = () => {
      isConnected = true;
      connectionListeners.forEach((cb) => cb(true));
    };

    // Event: 'state' -> Murni status visual AI dan token
    stateEventSource.addEventListener('state', (event: MessageEvent) => {
      try {
        const data: CoreStatePayload = JSON.parse(event.data);
        lastKnownState = data;
        isConnected = true;
        stateListeners.forEach((cb) => cb(data));
      } catch (err) {
        console.error('Failed to parse SSE state:', err);
      }
    });

    // Event: 'action' -> Murni aksi perintah yang dipicu AI (contoh: music:play)
    stateEventSource.addEventListener('action', (event: MessageEvent) => {
      try {
        const act: ClientAction = JSON.parse(event.data);
        actionListeners.forEach((cb) => cb(act));
      } catch (err) {
        console.error('Failed to parse SSE action:', err);
      }
    });

    // Event: 'log' -> Aliran log aktivitas konsol real-time (Zero Polling)
    stateEventSource.addEventListener('log', (event: MessageEvent) => {
      try {
        const logEntry = JSON.parse(event.data);
        logListeners.forEach((cb) => cb(logEntry));
      } catch (err) {
        console.error('Failed to parse SSE log:', err);
      }
    });

    stateEventSource.onerror = () => {
      isConnected = false;
      connectionListeners.forEach((cb) => cb(false));
      try {
        stateEventSource?.close();
      } catch {}
      stateEventSource = null;

      if (!stateReconnectTimer && (stateListeners.size > 0 || actionListeners.size > 0)) {
        stateReconnectTimer = window.setTimeout(() => {
          stateReconnectTimer = null;
          startStateSSE();
        }, 1500);
      }
    };
  } catch (err) {
    console.error('Failed to init State EventSource:', err);
  }
}

function stopStateSSEIfNoListeners() {
  if (stateListeners.size === 0 && actionListeners.size === 0 && logListeners.size === 0 && connectionListeners.size === 0) {
    if (stateReconnectTimer) {
      clearTimeout(stateReconnectTimer);
      stateReconnectTimer = null;
    }
    if (stateEventSource) {
      try {
        stateEventSource.close();
      } catch {}
      stateEventSource = null;
    }
    isConnected = false;
  }
}

function forceCloseSSE() {
  if (stateReconnectTimer) {
    clearTimeout(stateReconnectTimer);
    stateReconnectTimer = null;
  }
  stateListeners.clear();
  actionListeners.clear();
  logListeners.clear();
  connectionListeners.clear();
  if (stateEventSource) {
    try {
      stateEventSource.close();
    } catch {}
    stateEventSource = null;
  }
  isConnected = false;
}



export const eventsService = {
  // Single Client Session Claim (1x saat tab dibuka)
  async claimSession(token: string, activeTab?: string): Promise<boolean> {
    try {
      const res = await fetch(`${API_BASE}/api/session/claim`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ token, active_tab: activeTab || 'menu' }),
      });
      return res.ok;
    } catch {
      return false;
    }
  },

  // Notify server of active tab change
  async notifyActiveTab(activeTab: string): Promise<void> {
    try {
      await fetch(`${API_BASE}/api/session/tab`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ active_tab: activeTab }),
      });
    } catch {}
  },

  subscribeState(
    onStateChange: (state: CoreStatePayload) => void,
    onConnectionChange?: (connected: boolean) => void
  ): () => void {
    stateListeners.add(onStateChange);
    if (onConnectionChange) {
      connectionListeners.add(onConnectionChange);
      onConnectionChange(isConnected);
    }
    if (lastKnownState) {
      onStateChange(lastKnownState);
    }
    startStateSSE();

    return () => {
      stateListeners.delete(onStateChange);
      if (onConnectionChange) {
        connectionListeners.delete(onConnectionChange);
      }
      stopStateSSEIfNoListeners();
    };
  },

  // 2. Subscribe Action Events (Perintah aksi AI murni, seperti musik, notifikasi)
  subscribeActions(onAction: (act: ClientAction) => void): () => void {
    actionListeners.add(onAction);
    startStateSSE();

    return () => {
      actionListeners.delete(onAction);
      stopStateSSEIfNoListeners();
    };
  },

  // 3. Subscribe Live Logs via SSE (Realtime zero-polling console stream)
  subscribeLogs(onLog: (log: any) => void): () => void {
    logListeners.add(onLog);
    startStateSSE();

    return () => {
      logListeners.delete(onLog);
      stopStateSSEIfNoListeners();
    };
  },

  isSSEConnected(): boolean {
    return isConnected;
  },

  async getActionsSince(lastId: number): Promise<{ actions: ClientAction[]; last_id: number }> {
    const res = await fetch(`${API_BASE}/api/actions/since?last_id=${lastId}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  },

  forceCloseSSE,
};
