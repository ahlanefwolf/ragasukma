/**
 * Ragasukma AI Web Companion - Content Script Main Coordinator
 * Merakit modul DOM Extractor, Click Engine, Input Engine, dan Media Controller.
 */

(function () {
  // Cegah injeksi berulang di frame/subframe yang bukan window utama
  if (window.top !== window.self) return;

  // Abaikan dan jangan jalankan companion di internal dashboard Ragasukma (127.0.0.1:42128)
  if (window.location.host === "127.0.0.1:42128" || window.location.host === "localhost:42128") return;

  console.log("⚡ Ragasukma AI Companion aktif pada:", window.location.hostname);

  let lastSentUrl = "";
  let lastSentTitle = "";
  let debounceTimer = null;

  let lastSnapshotHash = "";

  function computeSimpleHash(str) {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      hash = ((hash << 5) - hash) + str.charCodeAt(i);
      hash |= 0;
    }
    return String(hash);
  }

  function sendSnapshot(force = false) {
    try {
      // Jika bukan force, HANYA kirim jika halaman saat ini sedang aktif & visible
      if (!force && document.visibilityState !== "visible") {
        return;
      }
      if (!window.RagasukmaDOM || typeof window.RagasukmaDOM.extractSnapshot !== "function") return;
      const snapshot = window.RagasukmaDOM.extractSnapshot();
      if (!snapshot) return;

      const textsJoined = (snapshot.texts || []).join("|");
      const currentHash = computeSimpleHash(snapshot.url + snapshot.title + textsJoined);

      // Jika bukan force dan isi halaman sama persis, abaikan
      if (!force && currentHash === lastSnapshotHash) {
        return;
      }

      lastSentUrl = snapshot.url;
      lastSentTitle = snapshot.title;
      lastSnapshotHash = currentHash;
      console.log("📸 [Ragasukma Extension] Mengirim snapshot:", snapshot.title, snapshot.url, `(${snapshot.texts?.length || 0} teks)`);

      // Kirim HANYA via Background Service Worker (karena background script terbebas dari CSP HTTPS website)
      const extApi = typeof chrome !== "undefined" && chrome.runtime ? chrome : (typeof browser !== "undefined" && browser.runtime ? browser : null);
      if (extApi && extApi.runtime && extApi.runtime.id && extApi.runtime.sendMessage) {
        try {
          extApi.runtime.sendMessage({ type: "DOM_SNAPSHOT", data: snapshot }, (resp) => {
            if (extApi.runtime && extApi.runtime.lastError) {
              // Ignore disconnected port/context invalidated quietly
            } else {
              console.log("💾 [Ragasukma Extension] Snapshot berhasil disimpan ke active_tab_browser.json");
            }
          });
        } catch (_) {}
      }
    } catch (e) {
      console.warn("⚠️ [Ragasukma Extension] Info snapshot:", e.message || e);
    }
  }

  // JALANKAN SAAT CONTENT SCRIPT INJECT HANYA JIKA TAB SEDANG VISIBLE
  if (document.visibilityState === "visible") {
    sendSnapshot(true);
  }

  function triggerSnapshotDebounced(delay = 150, force = false) {
    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      sendSnapshot(force);
    }, delay);
  }

  // User Interaction & Manual Activity Listeners (Klik manual, scroll, ketik user, fokus window/tab)
  // Memastikan snapshot selalu segar saat user berinteraksi langsung di browser
  window.addEventListener("focus", () => {
    lastSnapshotHash = "";
    sendSnapshot(true);
  }, { passive: true });
  window.addEventListener("click", () => triggerSnapshotDebounced(150, true), { passive: true });
  window.addEventListener("keyup", () => triggerSnapshotDebounced(300, false), { passive: true });
  window.addEventListener("scroll", () => triggerSnapshotDebounced(400, false), { passive: true });

  // Event Listeners Lifecycle & Navigasi
  if (document.readyState === "complete" || document.readyState === "interactive") {
    triggerSnapshotDebounced(50, true);
  } else {
    window.addEventListener("DOMContentLoaded", () => triggerSnapshotDebounced(50, true));
  }

  window.addEventListener("load", () => triggerSnapshotDebounced(100, true));

  // Intercept History API (pushState & replaceState) untuk SPA Universal (React, Vue, Next.js, Solid, Svelte)
  try {
    const originalPushState = history.pushState;
    history.pushState = function (...args) {
      originalPushState.apply(this, args);
      lastSnapshotHash = "";
      triggerSnapshotDebounced(150, true);
    };

    const originalReplaceState = history.replaceState;
    history.replaceState = function (...args) {
      originalReplaceState.apply(this, args);
      lastSnapshotHash = "";
      triggerSnapshotDebounced(250, true);
    };
  } catch (_) {}

  // Intercept fetch & XMLHttpRequest untuk menangkap update data dinamis / API load di SPA
  try {
    const originalFetch = window.fetch;
    if (originalFetch) {
      window.fetch = async function (...args) {
        try {
          const response = await originalFetch.apply(this, args);
          // HANYA trigger jika request bukan ke backend lokal Ragasukma
          const urlStr = typeof args[0] === "string" ? args[0] : (args[0] && args[0].url ? args[0].url : "");
          if (!urlStr.includes("42128") && !urlStr.includes("127.0.0.1") && document.visibilityState === "visible") {
            triggerSnapshotDebounced(500, false);
          }
          return response;
        } catch (err) {
          throw err;
        }
      };
    }

    const OriginalXHR = window.XMLHttpRequest;
    if (OriginalXHR) {
      const originalOpen = OriginalXHR.prototype.open;
      const originalSend = OriginalXHR.prototype.send;

      OriginalXHR.prototype.open = function (method, url, ...rest) {
        this._ragasukma_url = typeof url === "string" ? url : "";
        return originalOpen.call(this, method, url, ...rest);
      };

      OriginalXHR.prototype.send = function (...args) {
        this.addEventListener("load", () => {
          const url = this._ragasukma_url || "";
          if (!url.includes("42128") && !url.includes("127.0.0.1") && document.visibilityState === "visible") {
            triggerSnapshotDebounced(500, false);
          }
        }, { passive: true });
        return originalSend.apply(this, args);
      };
    }
  } catch (_) {}

  // Universal SPA, YouTube/SPA Events, History, and Visibility Events
  window.addEventListener("popstate", () => {
    lastSnapshotHash = "";
    triggerSnapshotDebounced(20, true);
  });
  window.addEventListener("hashchange", () => {
    lastSnapshotHash = "";
    triggerSnapshotDebounced(20, true);
  });
  window.addEventListener("pageshow", () => {
    lastSnapshotHash = "";
    sendSnapshot(true);
  });
  window.addEventListener("yt-navigate-finish", () => {
    lastSnapshotHash = "";
    triggerSnapshotDebounced(150, true);
  });
  window.addEventListener("yt-page-data-updated", () => {
    lastSnapshotHash = "";
    triggerSnapshotDebounced(200, true);
  });
  document.addEventListener("visibilitychange", () => {
    lastSnapshotHash = "";
    if (document.visibilityState === "visible") {
      setTimeout(() => sendSnapshot(true), 30);
      setTimeout(() => sendSnapshot(true), 250);
    } else {
      // BATALKAN SEMUA TIMER SAAT TAB MENJADI HIDDEN/BACKGROUND
      if (debounceTimer) {
        clearTimeout(debounceTimer);
        debounceTimer = null;
      }
    }
  });

  // Reaktif terhadap perubahan DOM struktural & teks (HANYA jika tab sedang aktif/visible)
  try {
    const observer = new MutationObserver((mutations) => {
      if (document.visibilityState !== "visible") return;

      // Filter mutasi: periksa penambahan/penghapusan node atau perubahan teks konten
      let hasSignificantMutation = false;
      for (const m of mutations) {
        if (m.type === "childList" && (m.addedNodes.length > 0 || m.removedNodes.length > 0)) {
          hasSignificantMutation = true;
          break;
        } else if (m.type === "characterData") {
          hasSignificantMutation = true;
          break;
        }
      }

      if (hasSignificantMutation) {
        triggerSnapshotDebounced(500, false);
      }
    });

    const observerConfig = {
      childList: true,
      subtree: true,
      characterData: true
    };

    if (document.body) {
      observer.observe(document.body, observerConfig);
    } else {
      window.addEventListener("DOMContentLoaded", () => {
        if (document.body) {
          observer.observe(document.body, observerConfig);
        }
      });
    }
  } catch (_) {}

  // Monitor perubahan URL & Judul secara realtime (HANYA jika tab sedang aktif/visible)
  let lastObservedUrl = window.location.href;
  let lastTitle = document.title;
  setInterval(() => {
    if (document.visibilityState !== "visible") return;
    if (window.location.href !== lastObservedUrl || document.title !== lastTitle) {
      lastObservedUrl = window.location.href;
      lastTitle = document.title;
      lastSnapshotHash = "";
      triggerSnapshotDebounced(300, true);
    }
  }, 500);

    // Router Eksekusi Aksi dari Background Script
    const extApi = typeof chrome !== "undefined" && chrome.runtime ? chrome : (typeof browser !== "undefined" && browser.runtime ? browser : null);
    if (extApi && extApi.runtime && extApi.runtime.onMessage) {
      extApi.runtime.onMessage.addListener((message, sender, sendResponse) => {
        if (message && message.type === "REQUEST_SNAPSHOT") {
          lastSnapshotHash = "";
          sendSnapshot(true);
          sendResponse({ status: "ok" });
          return;
        }

        if (message && message.type === "EXECUTE_BROWSER_ACTION" && message.command) {
        const cmd = message.command;
        console.log("🎮 [Ragasukma DOM Action] Menerima aksi:", cmd.action, cmd);

        // 1. Pencarian In-Page Universal
        if (cmd.action === "IN_PAGE_SEARCH") {
          const query = cmd.query || "";
          const newUrl = cmd.new_url || "";

          console.log("🔍 [Ragasukma In-Page Search] Menjalankan pencarian universal:", query, newUrl);
          const searchInput = window.RagasukmaInput.findBestSearchInput();

          if (searchInput && query) {
            try { searchInput.focus(); } catch (_) {}
            try { searchInput.select(); } catch (_) {}
            window.RagasukmaInput.setNativeInputValue(searchInput, query);
            window.RagasukmaInput.triggerInputEvents(searchInput, query);

            setTimeout(() => {
              window.RagasukmaInput.triggerEnterSubmit(searchInput);
            }, 80);
          } else if (newUrl) {
            window.location.href = newUrl;
          }
          sendResponse({ status: "ok", action: "IN_PAGE_SEARCH" });
        } 
        // 2. Media Controller
        else if (cmd.action === "MEDIA_PAUSE") {
          window.RagasukmaMedia.pause();
          sendResponse({ status: "ok", action: "MEDIA_PAUSE" });
        } else if (cmd.action === "MEDIA_PLAY") {
          window.RagasukmaMedia.play();
          sendResponse({ status: "ok", action: "MEDIA_PLAY" });
        } else if (cmd.action === "MEDIA_TOGGLE") {
          window.RagasukmaMedia.toggle();
          sendResponse({ status: "ok", action: "MEDIA_TOGGLE" });
        } else if (cmd.action === "MEDIA_NEXT") {
          window.RagasukmaMedia.next();
          sendResponse({ status: "ok", action: "MEDIA_NEXT" });
        } else if (cmd.action === "MEDIA_PREV") {
          window.RagasukmaMedia.prev();
          sendResponse({ status: "ok", action: "MEDIA_PREV" });
        } 
        // 3. Click Engine (Klik Tulisan vs Klik Tombol vs Universal)
        else if (cmd.action === "CLICK_TEXT") {
          const rawSel = (cmd.selector || "").trim();
          const target = window.RagasukmaClick.findText(rawSel) || 
                         window.RagasukmaClick.findButton(rawSel) || 
                         window.RagasukmaClick.findSmartElement(rawSel);

          if (target) {
            window.RagasukmaClick.triggerClick(target);
            console.log("👆 [Ragasukma Click] Berhasil klik tulisan/link:", rawSel, target);
            // Langsung ambil snapshot baru setelah eksekusi klik
            setTimeout(() => sendSnapshot(true), 100);
            setTimeout(() => sendSnapshot(true), 400);
          } else {
            console.warn("⚠️ [Ragasukma Click] Tulisan tidak ditemukan untuk klik:", rawSel);
          }
          sendResponse({ status: "ok", action: "CLICK_TEXT", found: Boolean(target) });
        }
        else if (cmd.action === "CLICK_BUTTON") {
          const rawSel = (cmd.selector || "").trim();
          const target = window.RagasukmaClick.findButton(rawSel) || 
                         window.RagasukmaClick.findText(rawSel) || 
                         window.RagasukmaClick.findSmartElement(rawSel);

          if (target) {
            window.RagasukmaClick.triggerClick(target);
            console.log("👆 [Ragasukma Click] Berhasil klik tombol:", rawSel, target);
            // Langsung ambil snapshot baru setelah eksekusi klik tombol
            setTimeout(() => sendSnapshot(true), 100);
            setTimeout(() => sendSnapshot(true), 400);
          } else {
            console.warn("⚠️ [Ragasukma Click] Tombol tidak ditemukan untuk klik:", rawSel);
          }
          sendResponse({ status: "ok", action: "CLICK_BUTTON", found: Boolean(target) });
        }
        else if (cmd.action === "CLICK_ELEMENT") {
          const rawSel = (cmd.selector || "").trim();
          const target = window.RagasukmaClick.findSmartElement(rawSel) || 
                         window.RagasukmaClick.findText(rawSel) || 
                         window.RagasukmaClick.findButton(rawSel);

          if (target) {
            window.RagasukmaClick.triggerClick(target);
            console.log("👆 [Ragasukma Click] Berhasil klik elemen universal:", rawSel, target);
            // Langsung ambil snapshot baru setelah eksekusi klik elemen
            setTimeout(() => sendSnapshot(true), 100);
            setTimeout(() => sendSnapshot(true), 400);
          } else {
            console.warn("⚠️ [Ragasukma Click] Elemen tidak ditemukan untuk klik:", rawSel);
          }
          sendResponse({ status: "ok", action: "CLICK_ELEMENT", found: Boolean(target) });
        } 
        // 4. Input Engine (Pengetikan Teks Tanpa Auto-Enter)
        else if (cmd.action === "TYPE_TEXT") {
          const sel = (cmd.selector || "").trim();
          const text = cmd.text_to_type || "";
          const found = window.RagasukmaInput.executeType(sel, text, false);
          if (found) {
            setTimeout(() => sendSnapshot(true), 100);
          }
          sendResponse({ status: "ok", action: "TYPE_TEXT", found: found });
        } 
        // 5. Tekan Tombol Enter
        else if (cmd.action === "PRESS_ENTER") {
          const activeEl = document.activeElement;
          const searchInput = window.RagasukmaInput.findBestSearchInput();
          const targetEl = (activeEl && (activeEl.tagName === "INPUT" || activeEl.tagName === "TEXTAREA" || activeEl.isContentEditable || activeEl.getAttribute("role") === "textbox")) ? activeEl : searchInput;

          if (targetEl) {
            window.RagasukmaInput.triggerEnterSubmit(targetEl);
            console.log("⌨️ [Ragasukma Key] Berhasil kirim pesan/Enter pada:", targetEl);
            setTimeout(() => sendSnapshot(true), 150);
            setTimeout(() => sendSnapshot(true), 500);
          } else {
            console.warn("⚠️ [Ragasukma Key] Tidak ada input aktif untuk Enter.");
          }
          sendResponse({ status: "ok", action: "PRESS_ENTER" });
        }
        // 6. Scroll Halaman
        else if (cmd.action === "SCROLL") {
          const y = cmd.scroll_y || 400;
          window.scrollBy({ top: y, behavior: "smooth" });
          setTimeout(() => sendSnapshot(true), 250);
          sendResponse({ status: "ok", action: "SCROLL" });
        }
        return true;
      }
      return false;
    });
  }
})();
