// Ragasukma AI Background Service Worker
"use strict";

const API_BASE = "http://127.0.0.1:42128";
const api = typeof chrome !== "undefined" ? chrome : (typeof browser !== "undefined" ? browser : null);

let activeTabId = null;
let tabListCache = [];

function isUsable(tab) {
  return tab?.url && (tab.url.startsWith("http://") || tab.url.startsWith("https://")) &&
    !tab.url.includes("127.0.0.1:42128") && !tab.url.includes("localhost:42128");
}

function injectScripts() {
  if (!api?.tabs || !api?.scripting) return;
  api.tabs.query({}, (tabs) => {
    for (const t of tabs || []) {
      if (t.id && isUsable(t)) {
        api.scripting.executeScript({ target: { tabId: t.id }, files: ["content.js"] }).catch(() => {});
      }
    }
  });
}

api?.runtime?.onInstalled?.addListener(injectScripts);
api?.runtime?.onStartup?.addListener(injectScripts);
setTimeout(injectScripts, 500);

function syncTabs() {
  if (!api?.tabs) return;
  api.tabs.query({ lastFocusedWindow: true, active: true }, (actives) => {
    if (actives?.[0]?.id) activeTabId = actives[0].id;
    api.tabs.query({}, (tabs) => {
      if (!tabs) return;
      tabListCache = tabs.map(t => ({
        id: t.id,
        url: t.url || "",
        title: t.title || "",
        active: Boolean(t.id === activeTabId),
        window_id: t.windowId
      }));

      fetch(`${API_BASE}/api/browser/tabs`, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ tabs: tabListCache }),
        keepalive: true
      }).catch(() => {});
    });
  });
}

function requestSnapshot(tabId) {
  if (!tabId || !api?.tabs) return;
  api.tabs.sendMessage(tabId, { type: "REQUEST_SNAPSHOT" }, () => {
    if (api.runtime?.lastError && api.scripting) {
      api.scripting.executeScript({ target: { tabId }, files: ["content.js"] })
        .then(() => setTimeout(() => api.tabs.sendMessage(tabId, { type: "REQUEST_SNAPSHOT" }).catch(() => {}), 150))
        .catch(() => {});
    }
  });
}

function handleCommand(cmd) {
  if (!api?.tabs || !cmd) return;

  if (cmd.action === "SWITCH_TAB" || cmd.action === "SWITCH_OR_NAVIGATE") {
    api.tabs.query({}, (tabs) => {
      const q = (cmd.query || "").toLowerCase();
      const d = (cmd.domain || "").toLowerCase();
      const match = (tabs || []).find(t => {
        const u = (t.url || "").toLowerCase();
        const ti = (t.title || "").toLowerCase();
        return (d && u.includes(d)) || (q && (u.includes(q) || ti.includes(q)));
      });

      if (match?.id) {
        activeTabId = match.id;
        if (match.windowId && api.windows) api.windows.update(match.windowId, { focused: true }).catch(() => {});
        api.tabs.update(match.id, { active: true }, () => {
          syncTabs();
          setTimeout(() => requestSnapshot(match.id), 150);
        });
      } else if (cmd.new_url) {
        api.tabs.create({ url: cmd.new_url, active: true }, (nt) => {
          if (nt?.id) activeTabId = nt.id;
          setTimeout(syncTabs, 500);
        });
      }
    });
    return;
  }

  if (cmd.action === "CLOSE_TAB") {
    api.tabs.query({}, (tabs) => {
      const q = (cmd.query || "").toLowerCase();
      const match = (tabs || []).find(t => (t.url || "").toLowerCase().includes(q) || (t.title || "").toLowerCase().includes(q));
      if (match?.id) api.tabs.remove(match.id, syncTabs);
    });
    return;
  }

  if (cmd.action === "REFRESH_SNAPSHOT" || cmd.action === "REQUEST_DOM_SNAPSHOT") {
    api.tabs.query({ lastFocusedWindow: true, active: true }, (actives) => {
      const target = (actives?.[0] && isUsable(actives[0])) ? actives[0] : null;
      const tab = target || (activeTabId ? { id: activeTabId } : null);
      if (tab?.id) requestSnapshot(tab.id);
    });
    return;
  }

  // Active-tab user interaction (CLICK, TYPE, ENTER, SCROLL)
  api.tabs.query({ lastFocusedWindow: true, active: true }, (actives) => {
    const target = (actives?.[0] && isUsable(actives[0])) ? actives[0] : null;
    const tab = target || (activeTabId ? { id: activeTabId } : null);
    if (!tab?.id) return;

    api.tabs.get(tab.id, (info) => {
      if (api.runtime?.lastError || !info?.active) return;
      api.tabs.sendMessage(tab.id, { type: "EXECUTE_BROWSER_ACTION", command: cmd }).catch(() => {});
    });
  });
}

// Background Poller (100ms for ultra-responsive ping-pong)
setInterval(async () => {
  try {
    const res = await fetch(`${API_BASE}/api/browser/command/next`);
    if (!res.ok) return;
    const data = await res.json();
    if (data?.has_command && data?.command) handleCommand(data.command);
  } catch (_) {}
}, 100);

if (api?.tabs) {
  api.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
    if (changeInfo.status === "complete" || changeInfo.url) {
      syncTabs();
      if (isUsable(tab)) requestSnapshot(tabId);
    }
  });
  api.tabs.onActivated.addListener((info) => {
    activeTabId = info.tabId;
    syncTabs();
    requestSnapshot(info.tabId);
  });
}

// Keep-Alive Alarm Watchdog (Prevents MV3 Service Worker idle termination)
if (api?.alarms) {
  api.alarms.create("keepAlive", { periodInMinutes: 0.5 });
  api.alarms.onAlarm.addListener((alarm) => {
    if (alarm.name === "keepAlive") {
      syncTabs();
    }
  });
}

// Message Relay
api?.runtime?.onMessage?.addListener((message, sender, sendResponse) => {
  if (message?.type === "DOM_SNAPSHOT") {
    const sid = sender?.tab?.id || activeTabId;
    const payload = {
      ...message.data,
      is_loading: Boolean(sender?.tab?.status === "loading"),
      tabs: tabListCache.map(t => ({ ...t, active: t.id === sid }))
    };
    fetch(`${API_BASE}/api/browser/snapshot`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(payload),
      keepalive: true
    }).then(() => sendResponse?.({ status: "ok" })).catch(() => {});
    return true;
  }
});
