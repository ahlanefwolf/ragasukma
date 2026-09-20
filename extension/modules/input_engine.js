/**
 * Ragasukma AI Companion - Input Engine Module
 * Bertanggung jawab menangani pengetikan universal, formulir web, search box,
 * serta editor modern berbasis ContentEditable / ProseMirror / Lexical (seperti ChatGPT, Gemini, WhatsApp).
 */

(function (window) {
  window.RagasukmaInput = {
    findBestSearchInput() {
      const prioritySelectors = [
        "input[name='search_query']", // YouTube search box
        "#search-input input",         // YouTube desktop
        "input#search",                // YouTube
        "#prompt-textarea",            // ChatGPT
        "[contenteditable='true'][role='textbox']",
        "[contenteditable='true']",
        "div[role='textbox']",
        "textarea[name*='prompt' i]",
        "textarea[placeholder*='message' i]",
        "textarea[placeholder*='chat' i]",
        "textarea[placeholder*='tanya' i]",
        "textarea[placeholder*='ask' i]",
        "input[type='search']",
        "input[role='searchbox']",
        "input[name*='search' i]",
        "input[name*='query' i]",
        "input[name*='q' i]",
        "input[id*='search' i]",
        "input[placeholder*='search' i]",
        "input[placeholder*='cari' i]",
        "input[aria-label*='search' i]",
        "input[aria-label*='cari' i]",
        "textarea[name*='q' i]",
        "textarea[placeholder*='search' i]",
        "textarea[placeholder*='cari' i]",
        "textarea",
        "input[data-testid*='search' i]",
        "input[type='text'][placeholder]",
        "input[type='text']"
      ];

      for (const sel of prioritySelectors) {
        try {
          const elements = Array.from(document.querySelectorAll(sel));
          const visible = elements.find(el => 
            window.RagasukmaClick ? window.RagasukmaClick.isElementVisible(el) : (el.offsetParent !== null) && 
            el.type !== "file" && 
            el.type !== "checkbox" && 
            el.type !== "radio"
          );
          if (visible) return visible;
        } catch (_) {}
      }
      return null;
    },

    setNativeInputValue(element, val) {
      if (!element) return;
      if (element.type === "file" || element.type === "checkbox" || element.type === "radio") {
        return;
      }

      if (element.isContentEditable || element.getAttribute("contenteditable") === "true" || element.getAttribute("role") === "textbox") {
        try {
          element.focus();
          const success = document.execCommand("selectAll", false, null) && document.execCommand("insertText", false, val);
          if (!success) {
            element.innerText = val;
          }
        } catch (_) {
          element.innerText = val;
        }
        return;
      }

      try {
        const tag = element.tagName ? element.tagName.toLowerCase() : "input";
        const proto = tag === "textarea" ? window.HTMLTextAreaElement.prototype : window.HTMLInputElement.prototype;
        const setter = Object.getOwnPropertyDescriptor(proto, "value")?.set;
        if (setter) {
          setter.call(element, val);
        } else {
          element.value = val;
        }
      } catch (_) {
        element.value = val;
      }
    },

    triggerInputEvents(input, val) {
      if (!input) return;
      try {
        input.dispatchEvent(new InputEvent("input", { bubbles: true, cancelable: true, composed: true, data: val, inputType: "insertText" }));
      } catch (_) {}
      input.dispatchEvent(new Event("input", { bubbles: true, cancelable: true, composed: true }));
      input.dispatchEvent(new Event("change", { bubbles: true, cancelable: true, composed: true }));
    },

    triggerEnterSubmit(input) {
      if (!input) return;
      const enterInit = { key: "Enter", code: "Enter", keyCode: 13, which: 13, bubbles: true, cancelable: true, composed: true, view: window };
      input.dispatchEvent(new KeyboardEvent("keydown", enterInit));
      input.dispatchEvent(new KeyboardEvent("keypress", enterInit));
      input.dispatchEvent(new KeyboardEvent("keyup", enterInit));

      // Cari tombol submit/search di container terdekat atau document
      const container = input.closest("form, [role='search'], ytd-searchbox, header, nav, div") || document;
      const searchBtn = container.querySelector("button[type='submit'], input[type='submit'], button#search-icon-legacy, [aria-label*='Search' i], [aria-label*='Cari' i], [title*='Search' i], [title*='Cari' i], [data-testid*='send-button' i]") ||
                        document.querySelector("button[type='submit'], input[type='submit'], button#search-icon-legacy, [aria-label*='Search' i], [aria-label*='Cari' i], [data-testid*='send-button' i]");
      
      if (searchBtn && typeof searchBtn.click === "function") {
        if (window.RagasukmaClick && typeof window.RagasukmaClick.triggerClick === "function") {
          window.RagasukmaClick.triggerClick(searchBtn);
        } else {
          searchBtn.click();
        }
      } else if (input.form && typeof input.form.submit === "function") {
        try { input.form.submit(); } catch (_) {}
      }
    },

    executeType(sel, text, pressEnter) {
      let input = null;

      if (sel) {
        try {
          input = document.querySelector(sel);
        } catch (_) {}

        if (!input) {
          const selLower = sel.toLowerCase();
          const inputs = Array.from(document.querySelectorAll("input:not([type='hidden']):not([type='file']):not([type='checkbox']):not([type='radio']), textarea, [contenteditable='true'], [role='textbox']"));
          input = inputs.find(i => {
            const name = (i.name || i.placeholder || i.getAttribute("aria-label") || i.id || "").toLowerCase();
            return name.includes(selLower);
          });
        }
      }

      // Jika selector tidak diberikan atau tidak cocok, cek elemen yang sedang aktif
      if (!input) {
        const active = document.activeElement;
        if (active && (active.tagName === "INPUT" || active.tagName === "TEXTAREA" || active.isContentEditable || active.getAttribute("role") === "textbox")) {
          if (active.type !== "file" && active.type !== "checkbox" && active.type !== "radio") {
            input = active;
          }
        }
      }

      // Fallback ke input search terbaik di halaman (YouTube, Tokopedia, Google, dll)
      if (!input) {
        input = this.findBestSearchInput();
      }

      if (input) {
        try { input.scrollIntoView({ behavior: "smooth", block: "center" }); } catch (_) {}
        try { input.focus(); } catch (_) {}
        try { input.click(); } catch (_) {}
        this.setNativeInputValue(input, text);
        this.triggerInputEvents(input, text);

        if (pressEnter) {
          setTimeout(() => {
            this.triggerEnterSubmit(input);
          }, 120);
        }
        console.log("✍️ [Ragasukma Input] Berhasil ketik teks universal:", text, input);
        return true;
      }

      console.warn("⚠️ [Ragasukma Input] Input tidak ditemukan untuk mengetik:", sel, text);
      return false;
    }
  };
})(window);
