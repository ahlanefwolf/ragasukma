/**
 * Ragasukma AI Web Companion - Content Script
 * Menghubungkan Tab Browser (YouTube, YouTube Music, dll) dengan Backend Ragasukma AI
 */

(function () {
  console.log("⚡ Ragasukma AI Companion terhubung pada:", window.location.hostname);

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
      console.log("✅ Ragasukma AI Companion: Terhubung ke Desktop Server!");
      sendTabInfo();
    };

    socket.onmessage = async (event) => {
      try {
        const data = JSON.parse(event.data);
        handleCommand(data);
      } catch (err) {
        console.error("Gagal memproses pesan AI:", err);
      }
    };

    socket.onclose = () => {
      isConnected = false;
      setTimeout(connect, 3000); // Reconnect otomatis setiap 3 detik jika server mati
    };

    socket.onerror = () => {
      socket.close();
    };
  }

  function handleCommand(data) {
    if (!data || !data.action) return;

    // 1. YouTube & YouTube Music Media Control
    if (window.location.hostname.includes("youtube.com")) {
    // 2. Aksi Khusus YouTube & YouTube Music (Play / Pause / Next / Prev)
    if (window.location.hostname.includes("youtube.com") || window.location.hostname.includes("music.youtube.com")) {
      const video = document.querySelector("video");
      if (data.action === "pause" && video) video.pause();
      if (data.action === "play" && video) video.play();
      if (data.action === "toggle" && video) {
        video.paused ? video.play() : video.pause();
      }

      if (data.action === "next") {
        const nextBtn = document.querySelector(".next-button") || document.querySelector("button[aria-label='Lagu berikutnya']") || document.querySelector("button[aria-label='Next track']");
        if (nextBtn) nextBtn.click();
      }

      if (data.action === "prev") {
        const prevBtn = document.querySelector(".previous-button") || document.querySelector("button[aria-label='Lagu sebelumnya']") || document.querySelector("button[aria-label='Previous track']");
        if (prevBtn) prevBtn.click();
      }
    }
  }

  // 3. Fitur Auto-Play Otomatis untuk YouTube & YouTube Music (Bypass Autoplay Policy)
  function checkAndAutoPlayMusic() {
    // A. Jika di halaman Watch / Playlist YouTube Music (/watch?v=...)
    if (window.location.hostname.includes("music.youtube.com") && window.location.pathname.includes("/watch")) {
      console.log("⏳ [Ragasukma] Menyiapkan auto-play YouTube Music...");

      let attempts = 0;
      const interval = setInterval(() => {
        attempts++;
        const video = document.querySelector("video");
        const playBtn = document.querySelector("#play-pause-button") || 
                        document.querySelector(".play-pause-button") ||
                        document.querySelector("ytmusic-player-bar #play-pause-button") ||
                        document.querySelector("tp-yt-paper-icon-button#play-pause-button");

        if (video) {
          if (video.paused) {
            // Trik Resmi Browser: Play dalam kondisi Muted (selalu diizinkan 100% oleh browser policy)
            video.muted = true;
            
            const playPromise = video.play();
            if (playPromise !== undefined) {
              playPromise.then(() => {
                // Begitu stream audio berjalan, kembalikan volume (unmute)
                setTimeout(() => {
                  video.muted = false;
                  console.log("🔊 [Ragasukma] Autoplay sukses & audio unmuted!");
                }, 200);
              }).catch(() => {
                if (playBtn) playBtn.click();
              });
            }
          } else {
            video.muted = false;
            clearInterval(interval);
            console.log("✅ [Ragasukma] Musik berputar aktif dengan suara!");
          }
        }

        if (attempts >= 40) {
          clearInterval(interval);
        }
      }, 300);
      return;
    }

    // B. Jika di halaman Search YouTube Music (/search?q=...)
    if (window.location.hostname.includes("music.youtube.com") && window.location.pathname.includes("/search")) {
      console.log("🔍 [Ragasukma] Mendeteksi halaman search YouTube Music, mencari lagu/playlist teratas...");
      let attempts = 0;
      const interval = setInterval(() => {
        attempts++;

        const topPlayBtn = document.querySelector("ytmusic-responsive-list-item-renderer #play-button") ||
                           document.querySelector("ytmusic-card-shelf-renderer #play-button") ||
                           document.querySelector("ytmusic-shelf-renderer ytmusic-responsive-list-item-renderer .play-button") ||
                           document.querySelector("ytmusic-responsive-list-item-renderer .flex-columns a") ||
                           document.querySelector("ytmusic-responsive-list-item-renderer a.yt-simple-endpoint");

        if (topPlayBtn) {
          clearInterval(interval);
          console.log("▶️ [Ragasukma] Memutar lagu/playlist teratas di YouTube Music!");
          topPlayBtn.click();
        } else if (attempts >= 40) {
          clearInterval(interval);
        }
      }, 350);
      return;
    }

    // C. Jika di halaman Search YouTube Video Biasa (/results?search_query=...)
    if (window.location.hostname.includes("youtube.com") && !window.location.hostname.includes("music.youtube.com") && window.location.pathname.includes("/results")) {
      let attempts = 0;
      const interval = setInterval(() => {
        attempts++;
        const topVideo = document.querySelector("ytd-video-renderer a#thumbnail") ||
                         document.querySelector("ytd-video-renderer a#video-title");

        if (topVideo) {
          clearInterval(interval);
          console.log("▶️ [Ragasukma] Memutar video hasil pencarian teratas di YouTube...");
          topVideo.click();
        } else if (attempts >= 25) {
          clearInterval(interval);
        }
      }, 400);
    }
  }

  let lastTitle = document.title;
  setInterval(() => {
    if (document.title !== lastTitle) {
      lastTitle = document.title;
      sendTabInfo();
    }
  }, 2000);

  window.addEventListener("yt-navigate-finish", () => {
    setTimeout(sendTabInfo, 1000);
  });

  if (document.readyState === "complete" || document.readyState === "interactive") {
    checkAndAutoPlayMusic();
  } else {
    window.addEventListener("DOMContentLoaded", checkAndAutoPlayMusic);
  }

  // Coba hubungkan saat script dimuat
  connect();
})();
