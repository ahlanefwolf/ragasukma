// Ragasukma AI Background Service Worker
// Meneruskan snapshot DOM, Menyelaraskan State Multi-Tab, dan Mengeksekusi Aksi Tab Tanpa Membuka Ulang

console.log("🚀 [Ragasukma Background] Service worker aktif");

const browserAPI = typeof chrome !== "undefined" ? chrome : (typeof browser !== "undefined" ? browser : null);

let lastKnownActiveTabId = null;
let lastKnownWindowId = null;
let currentTabsList = [];
let backendAvailable = true;
let lastBackendFailureTime = 0;
const BACKEND_COOLDOWN_MS = 4000;

function isBackendInCooldown() {
  return !backendAvailable && Date.now() - lastBackendFailureTime < BACKEND_COOLDOWN_MS;
}

function markBackendSuccess() {
  backendAvailable = true;
}

function markBackendFailure() {
  backendAvailable = false;
  lastBackendFailureTime = Date.now();
}

// Injeksi otomatis content script ke seluruh tab yang sudah terbuka
function injectContentScriptsToAllTabs() {
  if (!browserAPI || !browserAPI.tabs || !browserAPI.scripting) return;
  try {
    browserAPI.tabs.query({}, (tabs) => {
      if (browserAPI.runtime && browserAPI.runtime.lastError) return;
      if (!tabs) return;
      currentTabsList = tabs.map(t => ({
        id: t.id,
        url: t.url || "",
        title: t.title || "",
        active: Boolean(t.active),
        window_id: t.windowId
      }));
      for (const t of tabs) {
        if (t.id && t.url && (t.url.startsWith("http://") || t.url.startsWith("https://"))) {
          if (t.url.includes("127.0.0.1:42128") || t.url.includes("localhost:42128")) continue;
          browserAPI.scripting.executeScript({
            target: { tabId: t.id },
            files: [
              "modules/click_engine.js",
              "modules/dom_extractor.js",
              "modules/input_engine.js",
              "modules/media_controller.js",
              "content.js"
            ]
          }).catch(() => {});
        }
      }
    });
  } catch (_) {}
}

if (browserAPI && browserAPI.runtime) {
  if (browserAPI.runtime.onInstalled) {
    browserAPI.runtime.onInstalled.addListener(() => {
      injectContentScriptsToAllTabs();
      setTimeout(syncTabs, 500);
    });
  }
  if (browserAPI.runtime.onStartup) {
    browserAPI.runtime.onStartup.addListener(() => {
      injectContentScriptsToAllTabs();
      setTimeout(syncTabs, 500);
    });
  }
}
setTimeout(injectContentScriptsToAllTabs, 500);

function syncTabs() {
  if (!browserAPI || !browserAPI.tabs) return;
  if (isBackendInCooldown()) return;

  try {
    browserAPI.tabs.query({ lastFocusedWindow: true, active: true }, (activeTabs) => {
      if (browserAPI.runtime && browserAPI.runtime.lastError) return;

      let trueActiveTabId = lastKnownActiveTabId;
      if (activeTabs && activeTabs.length > 0 && activeTabs[0].id) {
        trueActiveTabId = activeTabs[0].id;
        lastKnownActiveTabId = trueActiveTabId;
        lastKnownWindowId = activeTabs[0].windowId;
      }

      browserAPI.tabs.query({}, (tabs) => {
        if (browserAPI.runtime && browserAPI.runtime.lastError) return;
        if (!tabs) return;

        currentTabsList = tabs.map(t => ({
          id: t.id,
          url: t.url || "",
          title: t.title || "",
          active: Boolean(t.id === trueActiveTabId),
          window_id: t.windowId
        }));

        fetch("http://127.0.0.1:42128/api/browser/tabs", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({ tabs: currentTabsList }),
          keepalive: true
        })
          .then(res => {
            if (res.ok) markBackendSuccess();
            else markBackendFailure();
          })
          .catch(() => {
            markBackendFailure();
          });
      });
    });
  } catch (_) {}
}

// Tangani Perintah Tab dari Backend (Switch, Navigate, Close, Focus, Interaksi)
function handleBrowserCommand(cmd) {
  if (!browserAPI || !browserAPI.tabs) return;

  if (cmd.action === "SWITCH_TAB") {
    browserAPI.tabs.query({}, (tabs) => {
      if (browserAPI.runtime && browserAPI.runtime.lastError) return;
      if (!tabs) return;

      const targetQuery = (cmd.query || "").toLowerCase();
      const targetDomain = (cmd.domain || "").toLowerCase();

      let matchedTab = tabs.find(t => {
        const url = (t.url || "").toLowerCase();
        const title = (t.title || "").toLowerCase();
        if (targetDomain && url.includes(targetDomain)) return true;
        if (targetQuery && (url.includes(targetQuery) || title.includes(targetQuery))) return true;
        return false;
      });

      if (matchedTab && matchedTab.id) {
        lastKnownActiveTabId = matchedTab.id;
        lastKnownWindowId = matchedTab.windowId;

        if (matchedTab.windowId && browserAPI.windows) {
          browserAPI.windows.update(matchedTab.windowId, { focused: true }).catch(() => {});
        }

        browserAPI.tabs.update(matchedTab.id, { active: true }, () => {
          if (browserAPI.runtime && browserAPI.runtime.lastError) return;
          setTimeout(syncTabs, 100);
          setTimeout(() => {
            activeTabWorker.requestSnapshot(matchedTab.id);
          }, 150);
        });
      }
    });
  } else if (cmd.action === "SWITCH_OR_NAVIGATE") {
    browserAPI.tabs.query({}, (tabs) => {
      if (browserAPI.runtime && browserAPI.runtime.lastError) return;
      if (!tabs) return;

      const targetQuery = (cmd.query || "").toLowerCase();
      const targetDomain = (cmd.domain || "").toLowerCase();

      let matchedTab = tabs.find(t => {
        const url = (t.url || "").toLowerCase();
        const title = (t.title || "").toLowerCase();
        if (targetDomain && url.includes(targetDomain)) return true;
        if (targetQuery && (url.includes(targetQuery) || title.includes(targetQuery))) return true;
        return false;
      });

      if (matchedTab && matchedTab.id) {
        lastKnownActiveTabId = matchedTab.id;
        lastKnownWindowId = matchedTab.windowId;

        if (matchedTab.windowId && browserAPI.windows) {
          browserAPI.windows.update(matchedTab.windowId, { focused: true }).catch(() => {});
        }

        const isSearchQuery = Boolean(cmd.query && cmd.query.trim());
        const updateProps = { active: true };

        if (cmd.new_url && !isSearchQuery) {
          updateProps.url = cmd.new_url;
        }

        browserAPI.tabs.update(matchedTab.id, updateProps, () => {
          if (browserAPI.runtime && browserAPI.runtime.lastError) {
            if (cmd.new_url) {
              browserAPI.tabs.create({ url: cmd.new_url, active: true }, () => setTimeout(syncTabs, 400));
            }
          } else {
            if (isSearchQuery) {
              browserAPI.tabs.sendMessage(matchedTab.id, {
                type: "EXECUTE_BROWSER_ACTION",
                command: {
                  action: "IN_PAGE_SEARCH",
                  query: cmd.query,
                  new_url: cmd.new_url
                }
              }, () => {
                if (browserAPI.runtime && browserAPI.runtime.lastError) {}
              });
            }
            setTimeout(syncTabs, 100);
            setTimeout(() => {
              activeTabWorker.requestSnapshot(matchedTab.id);
            }, 150);
          }
        });
      } else if (cmd.new_url) {
        browserAPI.tabs.create({ url: cmd.new_url, active: true }, (newTab) => {
          if (browserAPI.runtime && browserAPI.runtime.lastError) return;
          if (newTab && newTab.id) {
            lastKnownActiveTabId = newTab.id;
          }
          setTimeout(syncTabs, 500);
        });
      }
    });
  } else if (cmd.action === "CLOSE_TAB") {
    browserAPI.tabs.query({}, (tabs) => {
      if (browserAPI.runtime && browserAPI.runtime.lastError) return;
      if (!tabs) return;

      const targetQuery = (cmd.query || "").toLowerCase();
      const matched = tabs.find(t => {
        const url = (t.url || "").toLowerCase();
        const title = (t.title || "").toLowerCase();
        return url.includes(targetQuery) || title.includes(targetQuery);
      });
      if (matched && matched.id) {
        browserAPI.tabs.remove(matched.id, () => {
          if (browserAPI.runtime && browserAPI.runtime.lastError) {}
          syncTabs();
        });
      }
    });
  } else if (
    cmd.action === "MEDIA_PLAY" ||
    cmd.action === "MEDIA_PAUSE" ||
    cmd.action === "MEDIA_TOGGLE" ||
    cmd.action === "MEDIA_NEXT" ||
    cmd.action === "MEDIA_PREV" ||
    cmd.action === "CLICK_TEXT" ||
    cmd.action === "CLICK_BUTTON" ||
    cmd.action === "CLICK_ELEMENT" ||
    cmd.action === "TYPE_TEXT" ||
    cmd.action === "PRESS_ENTER" ||
    cmd.action === "SCROLL"
  ) {
    browserAPI.tabs.query({}, (tabs) => {
      if (browserAPI.runtime && browserAPI.runtime.lastError) return;
      if (!tabs || tabs.length === 0) return;

      let targetTab = null;
      if (lastKnownActiveTabId) {
        targetTab = tabs.find(t => t.id === lastKnownActiveTabId && t.url && !t.url.startsWith("chrome://") && !t.url.startsWith("about:"));
      }
      if (!targetTab) {
        targetTab = tabs.find(t => t.active && t.url && !t.url.startsWith("chrome://") && !t.url.startsWith("about:"));
      }
      if (!targetTab && cmd.domain) {
        targetTab = tabs.find(t => (t.url || "").toLowerCase().includes(cmd.domain.toLowerCase()));
      }
      if (!targetTab) {
        targetTab = tabs.find(t => t.url && !t.url.startsWith("chrome://") && !t.url.startsWith("about:"));
      }

      if (!targetTab || !targetTab.id) return;

      if (targetTab.windowId && browserAPI.windows) {
        browserAPI.windows.update(targetTab.windowId, { focused: true }).catch(() => {});
      }

      browserAPI.tabs.sendMessage(targetTab.id, { type: "EXECUTE_BROWSER_ACTION", command: cmd }, (_res) => {
        // Konsumsi lastError secara aman untuk mencegah pesan uncaught di console
        if (browserAPI.runtime && browserAPI.runtime.lastError) {}
      });
    });
  }
}

// Polling Perintah Tab dari Backend
let isPolling = false;
async function pollBrowserCommands() {
  if (isPolling) return;
  isPolling = true;

  try {
    const res = await fetch("http://127.0.0.1:42128/api/browser/command/next");
    if (res.ok) {
      markBackendSuccess();
      const data = await res.json();
      if (data && data.has_command && data.command) {
        handleBrowserCommand(data.command);
        setTimeout(pollBrowserCommands, 100);
      }
    } else {
      markBackendFailure();
    }
  } catch (_) {
    markBackendFailure();
  } finally {
    isPolling = false;
  }
}

// Polling interval teratur 1000ms
setInterval(pollBrowserCommands, 1000);

if (browserAPI && browserAPI.alarms) {
  browserAPI.alarms.create("ragasukma_keepalive", { periodInMinutes: 1.0 });
  browserAPI.alarms.onAlarm.addListener((alarm) => {
    if (alarm.name === "ragasukma_keepalive") {
      pollBrowserCommands();
    }
  });
}

// Event Listeners Multi-Tab
if (browserAPI && browserAPI.tabs) {
  browserAPI.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
    if (changeInfo.status === "complete" || changeInfo.url) {
      syncTabs();
      if (tab && tab.url && !tab.url.startsWith("chrome://") && !tab.url.startsWith("about:") && !tab.url.includes("127.0.0.1:42128")) {
        activeTabWorker.requestSnapshot(tabId);
      }
    }
  });

  browserAPI.tabs.onActivated.addListener((activeInfo) => {
    lastKnownActiveTabId = activeInfo.tabId;
    syncTabs();
    activeTabWorker.requestSnapshot(activeInfo.tabId);
  });

  if (browserAPI.windows && browserAPI.windows.onFocusChanged) {
    browserAPI.windows.onFocusChanged.addListener((windowId) => {
      if (windowId === browserAPI.windows.WINDOW_ID_NONE) return;
      lastKnownWindowId = windowId;
      syncTabs();
    });
  }
}

// =========================================================================
// 🔄 DEDICATED ACTIVE TAB SYNC WORKER
// =========================================================================
class ActiveTabWorker {
  constructor() {
    this.lastActiveTabId = null;
    this.lastUrl = "";
    this.isRunning = false;
    this.intervalId = null;
  }

  start() {
    if (this.intervalId) clearInterval(this.intervalId);
    this.intervalId = setInterval(() => this.tick(), 1000);
  }

  async tick() {
    if (this.isRunning) return;
    this.isRunning = true;
    try {
      if (!browserAPI || !browserAPI.tabs) return;

      browserAPI.tabs.query({ lastFocusedWindow: true, active: true }, (activeTabs) => {
        if (browserAPI.runtime && browserAPI.runtime.lastError) return;
        if (!activeTabs || activeTabs.length === 0) return;

        const currentTab = activeTabs[0];
        if (!currentTab || !currentTab.id) return;

        const url = currentTab.url || "";
        if (url.startsWith("chrome://") || url.startsWith("about:") || url.includes("127.0.0.1:42128") || url.includes("localhost:42128")) {
          return;
        }

        const isNewTab = this.lastActiveTabId !== currentTab.id;
        const isNewUrl = this.lastUrl !== url;

        if (isNewTab || isNewUrl) {
          this.lastActiveTabId = currentTab.id;
          this.lastUrl = url;
          lastKnownActiveTabId = currentTab.id;
          lastKnownWindowId = currentTab.windowId;

          syncTabs();
          this.requestSnapshot(currentTab.id);
        }
      });
    } catch (_) {
    } finally {
      this.isRunning = false;
    }
  }

  requestSnapshot(tabId) {
    if (!tabId || !browserAPI.tabs) return;
    try {
      browserAPI.tabs.sendMessage(tabId, { type: "REQUEST_SNAPSHOT", force: true }, (_res) => {
        const err = browserAPI.runtime ? browserAPI.runtime.lastError : null;
        if (err && browserAPI.scripting) {
          // Tab belum memiliki content script -> injeksi otomatis
          browserAPI.scripting.executeScript({
            target: { tabId: tabId },
            files: [
              "modules/click_engine.js",
              "modules/dom_extractor.js",
              "modules/input_engine.js",
              "modules/media_controller.js",
              "content.js"
            ]
          }).then(() => {
            setTimeout(() => {
              browserAPI.tabs.sendMessage(tabId, { type: "REQUEST_SNAPSHOT", force: true }, () => {
                if (browserAPI.runtime && browserAPI.runtime.lastError) {}
              });
            }, 120);
          }).catch(() => {});
        }
      });
    } catch (_) {}
  }
}

const activeTabWorker = new ActiveTabWorker();
activeTabWorker.start();

setTimeout(syncTabs, 500);
setTimeout(pollBrowserCommands, 500);

let lastSavedSnapshotTime = 0;
let lastSavedSnapshotUrl = "";

// Listener Pesan Content Script
if (browserAPI && browserAPI.runtime && browserAPI.runtime.onMessage) {
  browserAPI.runtime.onMessage.addListener((message, sender, sendResponse) => {
    if (message && message.type === "DOM_SNAPSHOT") {
      if (message.data && message.data.url && (message.data.url.includes("127.0.0.1:42128") || message.data.url.includes("localhost:42128"))) {
        return false;
      }

      if (isBackendInCooldown()) {
        if (typeof sendResponse === "function") sendResponse({ status: "backend_cooldown" });
        return false;
      }

      const now = Date.now();
      const currentUrl = (message.data && message.data.url) || "";

      if (currentUrl === lastSavedSnapshotUrl && now - lastSavedSnapshotTime < 800) {
        if (typeof sendResponse === "function") sendResponse({ status: "throttled" });
        return false;
      }

      lastSavedSnapshotTime = now;
      lastSavedSnapshotUrl = currentUrl;

      const senderTabId = (sender && sender.tab) ? sender.tab.id : lastKnownActiveTabId;
      if (sender && sender.tab) {
        lastKnownActiveTabId = sender.tab.id;
        lastKnownWindowId = sender.tab.windowId;
      }

      if (senderTabId) {
        const found = currentTabsList.find(t => t.id === senderTabId);
        if (found) {
          if (message.data.url) found.url = message.data.url;
          if (message.data.title) found.title = message.data.title;
        }
      }

      const tabList = (currentTabsList && currentTabsList.length > 0 ? currentTabsList : []).map(t => ({
        id: t.id,
        url: (t.id === senderTabId && message.data.url) ? message.data.url : (t.url || ""),
        title: (t.id === senderTabId && message.data.title) ? message.data.title : (t.title || ""),
        active: Boolean(t.id === senderTabId),
        window_id: t.window_id || t.windowId
      }));

      const payload = {
        ...message.data,
        tabs: tabList
      };

      fetch("http://127.0.0.1:42128/api/browser/snapshot", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
        keepalive: true
      })
        .then(async (res) => {
          if (!res.ok) {
            markBackendFailure();
            throw new Error(`HTTP ${res.status}`);
          }
          markBackendSuccess();
          return res.json();
        })
        .then(data => {
          if (typeof sendResponse === "function") {
            try { sendResponse({ status: "ok", data }); } catch (_) {}
          }
        })
        .catch(() => {
          markBackendFailure();
          if (typeof sendResponse === "function") {
            try { sendResponse({ status: "error" }); } catch (_) {}
          }
        });

      return true;
    }
  });
}
