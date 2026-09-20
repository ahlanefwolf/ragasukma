/**
 * Ragasukma AI Companion - DOM Extractor Module
 * Bertanggung jawab mengekstrak snapshot visual & struktural DOM (Headings, Buttons, Inputs, Links, Text Summary).
 */

(function (window) {
  function isVisible(el) {
    if (!el) return false;
    try {
      if (window.RagasukmaClick && typeof window.RagasukmaClick.isElementVisible === "function") {
        return window.RagasukmaClick.isElementVisible(el);
      }
      const style = window.getComputedStyle ? window.getComputedStyle(el) : null;
      if (style && (style.display === "none" || style.visibility === "hidden" || style.opacity === "0")) return false;
      const rect = el.getBoundingClientRect ? el.getBoundingClientRect() : null;
      if (rect && rect.width > 0 && rect.height > 0) return true;
      if (el.offsetParent !== null && el.offsetParent !== undefined) return true;
      return true;
    } catch (_) {
      return true;
    }
  }

  window.RagasukmaDOM = {
    buildSelector(el) {
      if (!el) return "";
      if (el.id) return `#${el.id}`;
      if (el.getAttribute("aria-label")) return `[aria-label='${el.getAttribute("aria-label")}']`;
      if (el.name) return `${el.tagName.toLowerCase()}[name='${el.name}']`;
      if (el.className && typeof el.className === "string") {
        const firstClass = el.className.trim().split(/\s+/)[0];
        if (firstClass && !firstClass.includes(":") && !firstClass.includes("/")) {
          return `${el.tagName.toLowerCase()}.${firstClass}`;
        }
      }
      return el.tagName.toLowerCase();
    },

    extractSnapshot() {
      if (!document.body) return null;

      const MAX_LEN = 150;

      // 1. Buttons & Clickables (Semua tombol yang <= 150 karakter)
      const buttons = [];
      const btnElements = Array.from(document.querySelectorAll("button, [role='button'], input[type='button'], input[type='submit'], a.btn, a.button"));
      
      for (const b of btnElements) {
        if (!isVisible(b)) continue;
        const rawText = (b.innerText || b.getAttribute("aria-label") || b.title || b.value || "").trim();
        if (rawText.length > 0 && rawText.length <= MAX_LEN) {
          buttons.push({
            tag: b.tagName.toLowerCase(),
            id: b.id || undefined,
            class: b.className && typeof b.className === "string" && b.className.length <= MAX_LEN ? b.className.trim() : undefined,
            text: rawText,
            aria_label: b.getAttribute("aria-label") && b.getAttribute("aria-label").length <= MAX_LEN ? b.getAttribute("aria-label") : undefined,
            selector: this.buildSelector(b)
          });
        }
      }

      // 2. Inputs & Search Boxes (Semua input yang <= 150 karakter)
      const inputs = [];
      const inputElements = Array.from(document.querySelectorAll("input:not([type='hidden']):not([type='file']):not([type='checkbox']):not([type='radio']):not([type='button']):not([type='submit']), textarea, [contenteditable='true'], [role='textbox']"));
      for (const inp of inputElements) {
        if (!isVisible(inp)) continue;
        const val = inp.value ? inp.value.trim() : (inp.isContentEditable ? inp.innerText.trim() : "");
        const placeholder = (inp.placeholder || inp.getAttribute("aria-label") || inp.getAttribute("data-placeholder") || "").trim();
        
        if (val.length <= MAX_LEN && placeholder.length <= MAX_LEN) {
          inputs.push({
            tag: inp.tagName.toLowerCase(),
            id: inp.id || undefined,
            name: inp.name && inp.name.length <= MAX_LEN ? inp.name : undefined,
            placeholder: placeholder || undefined,
            aria_label: inp.getAttribute("aria-label") && inp.getAttribute("aria-label").length <= MAX_LEN ? inp.getAttribute("aria-label") : undefined,
            selector: this.buildSelector(inp),
            text: val || undefined
          });
        }
      }

      // 3. Ekstrak teks dari Link (a[href]) untuk dimasukkan ke kumpulan teks
      const linkElements = Array.from(document.querySelectorAll("a[href]"));
      const linkTexts = [];
      for (const a of linkElements) {
        if (!isVisible(a)) continue;
        const text = (a.innerText || a.getAttribute("title") || a.getAttribute("aria-label") || "").trim();
        const href = a.getAttribute("href") || "";
        if (text.length > 0 && text.length <= MAX_LEN && !href.startsWith("javascript:") && !href.startsWith("#")) {
          linkTexts.push(text);
        }
      }

      // 4. Semua Tulisan di Layar (Universal DOM + Web Components + Shadow DOM + Links/Buttons)
      const texts = [];
      const seen = new Set();

      function addTextLine(str) {
        if (!str) return;
        const clean = str.trim().replace(/\s+/g, " ");
        if (clean.length > 0 && clean.length <= MAX_LEN && !seen.has(clean)) {
          seen.add(clean);
          texts.push(clean);
        }
      }

      // 4a. Tambahkan teks dari judul halaman & link-link
      if (document.title) {
        addTextLine(document.title);
      }
      for (const lt of linkTexts) {
        const lines = lt.split("\n");
        for (const line of lines) addTextLine(line);
      }

      // 4b. Tambahkan teks dari seluruh elemen visual teks universal
      const textNodes = Array.from(document.querySelectorAll("h1, h2, h3, h4, h5, h6, [role='heading'], p, span, li, label, summary, td, th, [role='article'], [role='listitem'], [role='option'], [id*='title'], [class*='title']"));
      for (const el of textNodes) {
        if (!isVisible(el)) continue;
        const direct = (el.innerText || el.textContent || "").trim();
        if (direct && direct.length <= MAX_LEN) {
          const lines = direct.split("\n");
          for (const line of lines) addTextLine(line);
        }
      }

      // 4c. Tambahkan teks dari button & controls
      for (const b of buttons) {
        if (b.text) {
          const lines = b.text.split("\n");
          for (const line of lines) addTextLine(line);
        }
      }

      // 4d. Tambahkan teks dari document.body.innerText secara keseluruhan
      if (document.body && document.body.innerText) {
        const bodyLines = document.body.innerText.split("\n");
        for (const line of bodyLines) {
          addTextLine(line);
        }
      }

      // 5. Ringkasan Isi Halaman (text_summary) yang bersih & padat untuk AI
      const summaryLines = texts.slice(0, 40);
      const text_summary = summaryLines.join(" • ");

      return {
        url: window.location.href,
        domain: window.location.hostname,
        title: document.title || window.location.hostname,
        timestamp: Date.now(),
        buttons: buttons,
        inputs: inputs,
        texts: texts,
        text_summary: text_summary
      };
    }
  };
})(window);
