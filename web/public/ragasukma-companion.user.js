// ==UserScript==
// @name         Ragasukma AI Web Companion
// @namespace    https://github.com/ahlanefwolf/ragasukma
// @version      1.2.0
// @description  Jembatan cerdas & ringan menghubungkan YouTube, YouTube Music, dan browser ke Ragasukma AI Voice Assistant
// @author       Ahlan Saputra
// @match        *://www.youtube.com/*
// @match        *://music.youtube.com/*
// @match        *://*/*
// @icon         https://raw.githubusercontent.com/ahlanefwolf/ragasukma/main/icons/128x128.png
// @grant        none
// @run-at       document-end
// ==/UserScript==

(function () {
  'use strict';
  console.log("⚡ Ragasukma AI Web Companion Aktif pada:", window.location.hostname);

  let socket = null;
  let isConnected = false;

  function sendTabInfo() {
    if (isConnected && socket && socket.readyState === WebSocket.OPEN) {
      socket.send(JSON.stringify({
        url: window.location.href,
        title: document.title,
        domain: window.location.hostname
      }));
    }
  }

  function connect() {
    socket = new WebSocket("ws://127.0.0.1:42128/ws/browser");
    socket.onopen = () => {
      isConnected = true;
      console.log("✅ Ragasukma AI Companion: Terhubung ke Desktop Assistant!");
      sendTabInfo();
    };
    socket.onmessage = async (event) => {
      try { handleCommand(JSON.parse(event.data)); }
      catch (err) { console.error("Gagal memproses aksi AI:", err); }
    };
    socket.onclose = () => { isConnected = false; setTimeout(connect, 3000); };
    socket.onerror = () => { socket.close(); };
  }

  function simulateClick(el) {
    ['mouseenter', 'mouseover', 'mousedown', 'mouseup', 'click'].forEach(type => {
      el.dispatchEvent(new MouseEvent(type, { bubbles: true, cancelable: true, view: window }));
    });
  }

  function handleCommand(data) {
    if (!data || !data.action) return;
    if (window.location.hostname.includes("youtube.com")) {
      const video = document.querySelector("video");
      if (data.action === "pause" && video) video.pause();
      if (data.action === "play" && video) video.play();
      if (data.action === "toggle" && video) video.paused ? video.play() : video.pause();
      if (data.action === "next") {
        const btn = document.querySelector(".next-button") ||
          document.querySelector("button[aria-label='Lagu berikutnya']") ||
          document.querySelector("button[aria-label='Next track']");
        if (btn) btn.click();
      }
      if (data.action === "prev") {
        const btn = document.querySelector(".previous-button") ||
          document.querySelector("button[aria-label='Lagu sebelumnya']") ||
          document.querySelector("button[aria-label='Previous track']");
        if (btn) btn.click();
      }
    }
  }

  // Cari elemen play di halaman search YTM
  function findYTMSearchTarget() {
    return document.querySelector(
      "ytmusic-responsive-list-item-renderer ytmusic-play-button-renderer[aria-label^='Play']"
    ) || document.querySelector(
      "ytmusic-card-shelf-renderer a.thumbnail-link"
    );
  }

  // Auto-play search YTM: cek langsung dulu, kalau belum ada pasang MutationObserver
  function autoPlayYTMusicSearch() {
    console.log("🔍 [Ragasukma] Halaman search YT Music — menunggu lagu teratas...");

    // Cek apakah elemen sudah ada
    const existing = findYTMSearchTarget();
    if (existing) {
      console.log("▶️ [Ragasukma] Elemen sudah ada, langsung play!", existing);
      simulateClick(existing);
      return;
    }

    // Belum ada — pasang MutationObserver
    let triggered = false;
    const timer = setTimeout(() => {
      observer.disconnect();
      console.warn("⚠️ [Ragasukma] Timeout 15s, elemen tidak muncul.");
    }, 15000);

    const observer = new MutationObserver(() => {
      if (triggered) return;
      const target = findYTMSearchTarget();
      if (target) {
        triggered = true;
        observer.disconnect();
        clearTimeout(timer);
        console.log("▶️ [Ragasukma] Memutar lagu teratas di YouTube Music!", target);
        simulateClick(target);
      }
    });

    observer.observe(document.documentElement, { childList: true, subtree: true });
  }

  // Auto-play search YouTube biasa
  function autoPlayYouTubeSearch() {
    const existing = document.querySelector("ytd-video-renderer a#thumbnail") ||
      document.querySelector("ytd-video-renderer a#video-title");
    if (existing) { simulateClick(existing); return; }

    let triggered = false;
    const observer = new MutationObserver(() => {
      if (triggered) return;
      const el = document.querySelector("ytd-video-renderer a#thumbnail") ||
        document.querySelector("ytd-video-renderer a#video-title");
      if (el) {
        triggered = true;
        observer.disconnect();
        console.log("▶️ [Ragasukma] Memutar video teratas di YouTube!");
        simulateClick(el);
      }
    });
    observer.observe(document.documentElement, { childList: true, subtree: true });
    setTimeout(() => observer.disconnect(), 15000);
  }

  // Auto-play halaman /watch (video/musik langsung)
  function autoPlayWatch() {
    console.log("⏳ [Ragasukma] Menyiapkan auto-play /watch...");
    let attempts = 0;
    const interval = setInterval(() => {
      attempts++;
      const video = document.querySelector("video");
      if (video) {
        if (video.paused) {
          video.muted = true;
          video.play().then(() => {
            setTimeout(() => { video.muted = false; }, 200);
          }).catch(() => {
            const btn = document.querySelector("#play-pause-button") ||
              document.querySelector("tp-yt-paper-icon-button#play-pause-button");
            if (btn) btn.click();
          });
        } else {
          video.muted = false;
          clearInterval(interval);
        }
      }
      if (attempts >= 40) clearInterval(interval);
    }, 300);
  }

  function checkAndAutoPlay() {
    const host = window.location.hostname;
    const path = window.location.pathname;
    const isYTMusic = host.includes("music.youtube.com");
    const isYT = host.includes("youtube.com") && !isYTMusic;

    if (isYTMusic && path.includes("/watch")) autoPlayWatch();
    else if (isYTMusic && path.includes("/search")) autoPlayYTMusicSearch();
    else if (isYT && path.includes("/results")) autoPlayYouTubeSearch();
    else if (isYT && path.includes("/watch")) autoPlayWatch();
  }

  // Pantau judul tab untuk kirim info ke backend
  let lastTitle = document.title;
  setInterval(() => {
    if (document.title !== lastTitle) {
      lastTitle = document.title;
      sendTabInfo();
    }
  }, 2000);

  // Intercept history.pushState — cara YTM navigasi SPA
  let lastUrl = window.location.href;
  const origPushState = history.pushState.bind(history);
  history.pushState = function (...args) {
    origPushState(...args);
    const newUrl = window.location.href;
    if (newUrl !== lastUrl) {
      lastUrl = newUrl;
      sendTabInfo();
      checkAndAutoPlay();
    }
  };

  // Jalankan saat pertama load
  checkAndAutoPlay();

  connect();
})();
