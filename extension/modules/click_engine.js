/**
 * Ragasukma AI Companion - Click Engine Module
 * Bertanggung jawab menemukan elemen web secara cerdas (berdasarkan teks, urutan, kartu, link)
 * dan mengeksekusi urutan pointer/mouse click yang presisi.
 */

(function (window) {
  function isElementVisible(el) {
    if (!el) return false;
    try {
      const style = window.getComputedStyle ? window.getComputedStyle(el) : null;
      if (style && (style.display === "none" || style.visibility === "hidden" || style.opacity === "0")) return false;
      const rect = el.getBoundingClientRect ? el.getBoundingClientRect() : null;
      if (rect && rect.width > 0 && rect.height > 0) return true;
      if (el.offsetParent !== null && el.offsetParent !== undefined) return true;
      // Shadow DOM / Custom Elements / Web Components generic
      if (el.tagName && el.tagName.includes("-")) return true;
      return Boolean((el.children && el.children.length > 0) || (el.textContent && el.textContent.trim().length > 0));
    } catch (_) {
      return true;
    }
  }

  function queryAllDeep(selector, root = document) {
    let results = [];
    try {
      results = Array.from(root.querySelectorAll(selector));
    } catch (_) {}
    try {
      const all = root.querySelectorAll("*");
      for (const el of all) {
        if (el.shadowRoot) {
          results = results.concat(queryAllDeep(selector, el.shadowRoot));
        }
      }
    } catch (_) {}
    return results;
  }

  function normalizeText(txt) {
    if (!txt) return "";
    return txt
      .toLowerCase()
      .replace(/[^\w\s\d]/g, " ") // Hapus tanda baca seperti tanda seru, koma, titik dua, dsb
      .replace(/\s+/g, " ")
      .trim();
  }

  function wordsMatchScore(targetText, queryText) {
    if (!targetText || !queryText) return 0;
    const targetNorm = normalizeText(targetText);
    const queryNorm = normalizeText(queryText);
    if (!targetNorm || !queryNorm) return 0;

    if (targetNorm === queryNorm) return 100;
    if (targetNorm.includes(queryNorm)) return 95;
    if (queryNorm.includes(targetNorm) && targetNorm.length > 2) return 90;

    const queryWords = queryNorm.split(" ").filter(w => w.length > 0);
    if (queryWords.length === 0) return 0;

    let matched = 0;
    for (const w of queryWords) {
      if (targetNorm.includes(w)) matched++;
    }
    return Math.floor((matched / queryWords.length) * 85);
  }

  window.RagasukmaClick = {
    isElementVisible,
    queryAllDeep,

    // Khusus mencari tombol / button / elemen aksi
    findButton(query) {
      if (!query) return null;
      let clean = query
        .replace(/^["'`“‘]+|["'`”’]+$/g, "")
        .toLowerCase()
        .replace(/^(tombol|button|btn)\s+/i, "")
        .trim();
      if (!clean) clean = query.replace(/^["'`“‘]+|["'`”’]+$/g, "").trim();
      if (!clean) return null;

      const btns = queryAllDeep("button, [role='button'], input[type='button'], input[type='submit'], a.btn, a.button, [role='tab'], [role='menuitem']")
        .filter(el => isElementVisible(el));

      let best = null;
      let maxScore = 0;
      for (const b of btns) {
        const text = (b.innerText || b.getAttribute("aria-label") || b.title || b.value || "");
        const score = wordsMatchScore(text, clean);
        if (score >= 40 && score > maxScore) {
          maxScore = score;
          best = b;
        }
      }
      return best || this.findText(clean);
    },

    // Khusus mencari teks / tulisan / link / judul di layar
    findText(query) {
      if (!query) return null;
      // Bersihkan tanda kutip pembungkus ("...", '...') dan prefix bahasa natural
      let cleanQuery = query
        .replace(/^["'`“‘]+|["'`”’]+$/g, "")
        .toLowerCase()
        .replace(/^(tulisan|teks|judul|kata|link)\s+/i, "")
        .trim();
      if (!cleanQuery) cleanQuery = query.replace(/^["'`“‘]+|["'`”’]+$/g, "").trim();
      if (!cleanQuery) return null;

      // Cari semua elemen teks visual di layar (termasuk button, link, heading, item)
      const allElements = queryAllDeep("button, [role='button'], a, [role='link'], h1, h2, h3, h4, h5, h6, [role='heading'], p, span, div, li, b, strong, em, label, summary, td, th, [role='article'], [role='tab'], [role='menuitem']")
        .filter(el => {
          if (!isElementVisible(el)) return false;
          const tag = el.tagName ? el.tagName.toLowerCase() : "";
          if (tag === "input" || tag === "textarea" || tag === "select" || tag === "form") return false;
          if (el.closest("input, textarea, select, [role='searchbox'], [role='combobox'], form")) return false;
          return true;
        });

      let bestMatch = null;
      let highestScore = 0;
      let shortestLength = Infinity;

      for (const el of allElements) {
        const ariaLabel = (el.getAttribute("aria-label") || "").trim();
        const title = (el.getAttribute("title") || "").trim();
        // Ambil direct text untuk mencocokkan teks paling spesifik
        const directText = (el.innerText || el.textContent || "").trim();

        const scoreAria = wordsMatchScore(ariaLabel, cleanQuery);
        const scoreTitle = wordsMatchScore(title, cleanQuery);
        const scoreText = wordsMatchScore(directText, cleanQuery);
        let currentScore = Math.max(scoreAria, scoreTitle, scoreText);

        // Berikan prioritas/bonus jika elemen itu sendiri merupakan link / button / tab interaktif murni
        const tag = el.tagName ? el.tagName.toLowerCase() : "";
        const role = el.getAttribute ? el.getAttribute("role") : null;
        const isInteractive = tag === "a" || tag === "button" || role === "button" || role === "link" || role === "tab";
        if (isInteractive && currentScore >= 40) {
          currentScore += 5; // Bonus preferensi tag interaktif
        }

        // Prioritaskan yang paling cocok dan memiliki panjang teks paling presisi (bukan container raksasa)
        if (currentScore >= 40) {
          const textLen = directText.length || 999;
          if (currentScore > highestScore || (currentScore === highestScore && textLen < shortestLength)) {
            highestScore = currentScore;
            shortestLength = textLen;
            bestMatch = el;
          }
        }
      }
      return bestMatch;
    },

    findSmartElement(query) {
      if (!query) return null;
      const lower = query.toLowerCase();
      if (lower.startsWith("tombol ") || lower.startsWith("button ") || lower.startsWith("btn ")) {
        return this.findButton(query) || this.findText(query);
      }
      return this.findText(query) || this.findButton(query);
    },

    // Menemukan elemen interaktif terdekat (cek elemen itu sendiri, anak di dalamnya, atau parent ke atas)
    findInteractiveTarget(el) {
      if (!el) return null;

      // 1. Cek apakah elemen itu sendiri adalah link atau button interaktif
      const tag = el.tagName ? el.tagName.toLowerCase() : "";
      const role = el.getAttribute ? el.getAttribute("role") : null;
      if (tag === "a" && el.href) {
        return { element: el, type: "link", href: el.href };
      }
      if (tag === "button" || role === "button" || role === "link" || role === "tab" || role === "menuitem") {
        return { element: el, type: "button", href: el.getAttribute("href") || el.dataset?.href || null };
      }

      // 2. Cek apakah di DALAM elemen ini (child/descendant) terdapat tag <a> atau <button>
      try {
        const innerLink = el.querySelector("a[href]");
        if (innerLink && innerLink.href) {
          return { element: innerLink, type: "link", href: innerLink.href };
        }
        const innerBtn = el.querySelector("button, [role='button'], [role='tab'], [role='link']");
        if (innerBtn) {
          return { element: innerBtn, type: "button", href: innerBtn.getAttribute("href") || innerBtn.dataset?.href || null };
        }
      } catch (_) {}

      // 3. Cek ke ATAS (parent/ancestor) hingga 15 tingkat
      let curr = el.parentElement || (el.getRootNode && el.getRootNode() instanceof ShadowRoot ? el.getRootNode().host : null);
      let depth = 0;

      while (curr && depth < 15) {
        if (curr.nodeType === 1) { // ELEMENT_NODE
          const cTag = curr.tagName ? curr.tagName.toLowerCase() : "";
          const cRole = curr.getAttribute ? curr.getAttribute("role") : null;

          if (cTag === "a" && curr.href && !curr.href.startsWith("javascript:") && !curr.href.startsWith("#")) {
            return { element: curr, type: "link", href: curr.href };
          }

          if (
            cTag === "button" ||
            cTag === "summary" ||
            cRole === "button" ||
            cRole === "link" ||
            cRole === "tab" ||
            cRole === "menuitem" ||
            cRole === "option"
          ) {
            return { element: curr, type: "button", href: curr.getAttribute("href") || curr.dataset?.href || curr.dataset?.url || null };
          }

          const dataHref = curr.getAttribute ? (curr.getAttribute("data-href") || curr.getAttribute("data-url") || curr.getAttribute("href")) : null;
          if (dataHref && !dataHref.startsWith("javascript:") && !dataHref.startsWith("#")) {
            return { element: curr, type: "link", href: dataHref };
          }

          // Card di YouTube atau SPA (ytd-thumbnail, dsb)
          if (cTag.includes("renderer") || cTag.includes("card") || cTag.includes("item")) {
            const innerL = curr.querySelector ? curr.querySelector("a[href]") : null;
            if (innerL && innerL.href) {
              return { element: innerL, type: "link", href: innerL.href };
            }
          }
        }

        curr = curr.parentElement || (curr.getRootNode && curr.getRootNode() instanceof ShadowRoot ? curr.getRootNode().host : null);
        depth++;
      }

      return null;
    },

    triggerClick(el) {
      if (!el) return;

      // Cari elemen aksi interaktif (bisa dirinya sendiri, anak link <a> di dalamnya, atau parent pembungkusnya)
      const interactiveInfo = this.findInteractiveTarget(el);
      const targetActionEl = interactiveInfo?.element || el;

      // Scroll ke elemen target & beri fokus
      try {
        targetActionEl.scrollIntoView({ behavior: "smooth", block: "center" });
      } catch (_) {
        try { el.scrollIntoView({ behavior: "smooth", block: "center" }); } catch (_) {}
      }

      try {
        if (typeof targetActionEl.focus === "function") targetActionEl.focus();
        else if (typeof el.focus === "function") el.focus();
      } catch (_) {}

      // Hitung koordinat klik
      const rect = targetActionEl.getBoundingClientRect();
      const clientX = rect.left + (rect.width > 0 ? rect.width / 2 : 10);
      const clientY = rect.top + (rect.height > 0 ? rect.height / 2 : 10);
      const opts = { bubbles: true, cancelable: true, composed: true, view: window, clientX, clientY };

      function dispatchClickEvents(target) {
        if (!target) return;
        try { target.dispatchEvent(new PointerEvent("pointerdown", opts)); } catch (_) {}
        try { target.dispatchEvent(new MouseEvent("mousedown", opts)); } catch (_) {}
        try { target.dispatchEvent(new PointerEvent("pointerup", opts)); } catch (_) {}
        try { target.dispatchEvent(new MouseEvent("mouseup", opts)); } catch (_) {}
        try { target.click(); } catch (_) {}
      }

      // 1. Eksekusi klik langsung pada elemen interaktif yang ditemukan (tag <a> atau <button>)
      dispatchClickEvents(targetActionEl);

      // 2. Jika elemen target berbeda dari elemen teks awal, dispatch juga ke elemen teks
      if (targetActionEl !== el) {
        dispatchClickEvents(el);
      }

      // 3. Fallback navigasi URL jika event SPA tidak otomatis berpindah halaman dalam 200ms
      const rawHref = interactiveInfo?.href || targetActionEl.getAttribute?.("href") || el.getAttribute?.("href");
      if (rawHref && typeof rawHref === "string" && !rawHref.startsWith("javascript:") && !rawHref.startsWith("#")) {
        const startUrl = window.location.href;
        setTimeout(() => {
          try {
            const currentUrl = window.location.href;
            const fullTargetUrl = new URL(rawHref, currentUrl).href;
            // Jika setelah 200ms URL masih sama persis, trigger navigasi paksa
            if (currentUrl === startUrl && currentUrl !== fullTargetUrl && !targetActionEl.target?.includes("_blank")) {
              console.log("🔗 [Ragasukma Navigasi Induk] Membuka URL target:", fullTargetUrl);
              window.location.href = fullTargetUrl;
            }
          } catch (_) {}
        }, 200);
      }
    }
  };
})(window);
