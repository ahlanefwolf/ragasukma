import { API_BASE } from './base';

// ---------------------------------------------------------------------------
// 1. REALTIME WEBSOCKET STREAM (/api/ws)
// Mengirim event 'state' (visual/status AI), 'action' (trigger skill/audio),
// 'log' (live console logs), and 'model_status' bidirectionally.
// ---------------------------------------------------------------------------
export interface CoreStatePayload {
  state: string;
  emotion?: string;
  voice_muted?: boolean;
  mic_muted?: boolean;
  is_running?: boolean;
}

export interface ModelDownloadStatus {
  status: 'ready' | 'checking' | 'downloading' | 'extracting' | 'error' | string;
  downloaded_bytes: number;
  total_bytes: number;
  percent: number;
  speed_mbps: number;
  message: string;
  model_name?: string;
}

export interface ClientAction {
  id?: number;
  action: string;
  payload?: string;
}

type StateListener = (state: CoreStatePayload) => void;
type ActionListener = (action: ClientAction) => void;
type LogListener = (log: any) => void;
type ModelStatusListener = (status: ModelDownloadStatus) => void;
type ConnectionListener = (connected: boolean) => void;

const stateListeners = new Set<StateListener>();
const actionListeners = new Set<ActionListener>();
const logListeners = new Set<LogListener>();
const modelStatusListeners = new Set<ModelStatusListener>();
const connectionListeners = new Set<ConnectionListener>();
let lastKnownState: CoreStatePayload | null = null;
let lastKnownModelStatus: ModelDownloadStatus | null = null;
let lastKnownActionId: number = 0;
let lastKnownLogId: number = 0;
let isConnected = false;
let stateFetchPromise: Promise<CoreStatePayload | null> | null = null;

function fetchStateOnce(): Promise<CoreStatePayload | null> {
  if (lastKnownState) return Promise.resolve(lastKnownState);
  if (stateFetchPromise) return stateFetchPromise;

  stateFetchPromise = fetch(`${API_BASE}/api/state`)
    .then((res) => (res.ok ? res.json() : null))
    .then((data) => {
      if (data && data.state) {
        const corePayload: CoreStatePayload = {
          state: data.state,
          emotion: data.emotion,
          voice_muted: data.voice_muted,
          mic_muted: data.mic_muted,
          is_running: data.is_running,
        };
        lastKnownState = corePayload;
        stateListeners.forEach((cb) => cb(corePayload));
        return corePayload;
      }
      return null;
    })
    .catch(() => null)
    .finally(() => {
      stateFetchPromise = null;
    });

  return stateFetchPromise;
}

let activeWs: WebSocket | null = null;
let wsReconnectTimer: number | null = null;
let wsPingInterval: number | null = null;
let listenersInitialized = false;

function setupLifecycleListeners() {
  if (listenersInitialized || typeof window === 'undefined') return;
  listenersInitialized = true;

  const tryReconnectNow = () => {
    if (stateListeners.size === 0 && actionListeners.size === 0 && logListeners.size === 0 && modelStatusListeners.size === 0) {
      return;
    }
    if (!activeWs || activeWs.readyState === WebSocket.CLOSED || activeWs.readyState === WebSocket.CLOSING) {
      if (wsReconnectTimer) {
        clearTimeout(wsReconnectTimer);
        wsReconnectTimer = null;
      }
      startWsStream();
    }
  };

  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'visible') {
      tryReconnectNow();
    }
  });

  window.addEventListener('online', tryReconnectNow);
  window.addEventListener('focus', tryReconnectNow);
}

function getWsUrl(): string {
  if (typeof window === 'undefined') return 'ws://127.0.0.1:42128/api/ws';
  const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
  const host = window.location.host;
  if (!host.includes('127.0.0.1') && !host.includes('localhost')) {
    return `${proto}//${host}/api/ws`;
  }
  return 'ws://127.0.0.1:42128/api/ws';
}

function startWsStream() {
  if (activeWs && (activeWs.readyState === WebSocket.OPEN || activeWs.readyState === WebSocket.CONNECTING)) {
    return;
  }
  if (wsReconnectTimer) return;

  try {
    const wsUrl = getWsUrl();
    const ws = new WebSocket(wsUrl);
    activeWs = ws;

    ws.onopen = () => {
      // A closed/replaced socket may still dispatch a late event.
      if (activeWs !== ws) return;

      isConnected = true;
      connectionListeners.forEach((cb) => cb(true));

      // Start client ping heartbeat every 15s to keep mobile connection alive.
      // This interval always belongs to the current active socket.
      if (wsPingInterval) clearInterval(wsPingInterval);
      wsPingInterval = window.setInterval(() => {
        if (activeWs === ws && ws.readyState === WebSocket.OPEN) {
          try {
            ws.send(JSON.stringify({ type: 'ping' }));
          } catch {}
        }
      }, 15000);
    };

    ws.onmessage = (event) => {
      if (activeWs !== ws) return;

      try {
        const parsed = JSON.parse(event.data);
        if (!parsed || !parsed.event) return;

        if (parsed.event === 'state' && parsed.data) {
          lastKnownState = parsed.data;
          isConnected = true;
          stateListeners.forEach((cb) => cb(parsed.data));
        } else if (parsed.event === 'action' && parsed.data) {
          const act: ClientAction = parsed.data;
          if (act.id !== undefined && act.id > lastKnownActionId) {
            lastKnownActionId = act.id;
          }
          actionListeners.forEach((cb) => cb(act));
        } else if (parsed.event === 'log' && parsed.data) {
          const logEntry = parsed.data;
          if (logEntry && logEntry.id && logEntry.id > lastKnownLogId) {
            lastKnownLogId = logEntry.id;
          }
          logListeners.forEach((cb) => cb(logEntry));
        } else if (parsed.event === 'model_status' && parsed.data) {
          lastKnownModelStatus = parsed.data;
          modelStatusListeners.forEach((cb) => cb(parsed.data));
        }
      } catch {}
    };

    ws.onerror = () => {
      if (activeWs !== ws) return;
      try { ws.close(); } catch {}
    };

    ws.onclose = () => {
      // Do not let a replaced socket tear down a newer connection.
      if (activeWs !== ws) return;

      isConnected = false;
      connectionListeners.forEach((cb) => cb(false));
      activeWs = null;
      if (wsPingInterval) {
        clearInterval(wsPingInterval);
        wsPingInterval = null;
      }

      if (!wsReconnectTimer && (stateListeners.size > 0 || actionListeners.size > 0 || logListeners.size > 0 || modelStatusListeners.size > 0)) {
        wsReconnectTimer = window.setTimeout(() => {
          wsReconnectTimer = null;
          startWsStream();
        }, 1500);
      }
    };
  } catch (e) {
    console.warn('[WebSocket] Init error:', e);
  }
}

function startStateEvents() {
  setupLifecycleListeners();
  startWsStream();
}

function stopWsIfNoListeners() {
  if (stateListeners.size === 0 && actionListeners.size === 0 && logListeners.size === 0 && modelStatusListeners.size === 0 && connectionListeners.size === 0) {
    if (wsReconnectTimer) {
      clearTimeout(wsReconnectTimer);
      wsReconnectTimer = null;
    }
    if (wsPingInterval) {
      clearInterval(wsPingInterval);
      wsPingInterval = null;
    }
    if (activeWs) {
      try {
        activeWs.close();
      } catch {}
      activeWs = null;
    }
    isConnected = false;
  }
}

function disconnectWs() {
  if (wsReconnectTimer) {
    clearTimeout(wsReconnectTimer);
    wsReconnectTimer = null;
  }
  if (wsPingInterval) {
    clearInterval(wsPingInterval);
    wsPingInterval = null;
  }
  stateListeners.clear();
  actionListeners.clear();
  logListeners.clear();
  modelStatusListeners.clear();
  connectionListeners.clear();
  if (activeWs) {
    try {
      activeWs.close();
    } catch {}
    activeWs = null;
  }
  isConnected = false;
}

// Backward compatibility alias
const forceCloseSSE = disconnectWs;



export const eventsService = {
  // Single Client Session Claim (1x when tab opened)
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
    } else {
      fetchStateOnce().then((st) => {
        if (st && stateListeners.has(onStateChange)) {
          onStateChange(st);
        }
      });
    }
    startStateEvents();

    return () => {
      stateListeners.delete(onStateChange);
      if (onConnectionChange) {
        connectionListeners.delete(onConnectionChange);
      }
      stopWsIfNoListeners();
    };
  },

  // 2. Subscribe Action Events (Perintah aksi AI murni, like music, notifikasi)
  subscribeActions(onAction: (act: ClientAction) => void): () => void {
    actionListeners.add(onAction);
    startStateEvents();

    return () => {
      actionListeners.delete(onAction);
      stopWsIfNoListeners();
    };
  },

  // 3. Subscribe Live Logs via WebSocket (Realtime zero-polling console stream)
  subscribeLogs(onLog: (log: any) => void): () => void {
    logListeners.add(onLog);
    startStateEvents();

    return () => {
      logListeners.delete(onLog);
      stopWsIfNoListeners();
    };
  },

  // 4. Subscribe Supertonic-3 Model Download Status via WebSocket
  subscribeModelStatus(onStatus: (status: ModelDownloadStatus) => void): () => void {
    modelStatusListeners.add(onStatus);
    if (lastKnownModelStatus) {
      onStatus(lastKnownModelStatus);
    }
    startStateEvents();

    return () => {
      modelStatusListeners.delete(onStatus);
      stopWsIfNoListeners();
    };
  },

  async getModelStatus(): Promise<ModelDownloadStatus> {
    const res = await fetch(`${API_BASE}/api/system/model-status`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  },

  async triggerModelDownload(): Promise<ModelDownloadStatus> {
    const res = await fetch(`${API_BASE}/api/system/model-download`, { method: 'POST' });
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  },

  isConnected(): boolean {
    return isConnected;
  },

  // Backward compatibility alias
  isSSEConnected(): boolean {
    return isConnected;
  },

  // HTTP Fallback: only used if client requires manual action polling without WS
  async getActionsSince(lastId: number): Promise<{ actions: ClientAction[]; last_id: number }> {
    const res = await fetch(`${API_BASE}/api/actions/since?last_id=${lastId}`);
    if (!res.ok) throw new Error(`HTTP ${res.status}`);
    return res.json();
  },

  disconnect: disconnectWs,
  forceCloseSSE,
};
