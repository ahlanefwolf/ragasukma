/**
 * Ragasukma AI Companion - Media Controller Module
 * Bertanggung jawab mengontrol video/audio HTML5 dan tombol player di web apa pun (YouTube, Spotify Web, dsb).
 */

(function (window) {
  window.RagasukmaMedia = {
    pause() {
      const mediaEls = document.querySelectorAll("video, audio");
      mediaEls.forEach(m => {
        try { m.pause(); } catch (_) {}
      });
      const pauseBtns = document.querySelectorAll("[aria-label*='pause' i], [aria-label*='jeda' i], [data-title-no-tooltip='Pause'], .play-pause-btn");
      pauseBtns.forEach(b => {
        try { b.click(); } catch (_) {}
      });
      console.log("⏸️ [Ragasukma Media] Media dijeda di:", window.location.hostname);
    },

    play() {
      const mediaEls = document.querySelectorAll("video, audio");
      mediaEls.forEach(m => {
        try { m.play(); } catch (_) {}
      });
      const playBtns = document.querySelectorAll("[aria-label*='play' i], [aria-label*='putar' i], [data-title-no-tooltip='Play'], .play-pause-btn");
      playBtns.forEach(b => {
        try { b.click(); } catch (_) {}
      });
      console.log("▶️ [Ragasukma Media] Media dilanjutkan di:", window.location.hostname);
    },

    toggle() {
      const video = document.querySelector("video, audio");
      if (video) {
        if (video.paused) video.play(); else video.pause();
      }
    },

    next() {
      const nextBtn = document.querySelector("[aria-label*='next' i], [aria-label*='berikutnya' i], [aria-label*='selanjutnya' i], [title*='Next' i], [title*='Berikutnya' i], [data-action*='next' i], .next-button");
      if (nextBtn) nextBtn.click();
      console.log("⏭️ [Ragasukma Media] Next video/track di:", window.location.hostname);
    },

    prev() {
      const prevBtn = document.querySelector("[aria-label*='previous' i], [aria-label*='sebelumnya' i], [title*='Prev' i], [title*='Sebelumnya' i], [data-action*='prev' i], .prev-button");
      if (prevBtn) prevBtn.click();
      console.log("⏮️ [Ragasukma Media] Prev video/track di:", window.location.hostname);
    }
  };
})(window);
