/**
 * Ragasukma AI Companion - Content Script
 * Minimal DOM Extractor, Click/Input Dispatcher, and Demonstration Recorder.
 */

"use strict";

(function () {
  if (window.top !== window.self) return;
  if (window.location.host === "127.0.0.1:42128" || window.location.host === "localhost:42128") return;

  const api = typeof chrome !== "undefined" ? chrome : (typeof browser !== "undefined" ? browser : null);

  function isVisible(el) {
    if (!el) return false;
    const style = window.getComputedStyle?.(el);
    if (style && (style.display === "none" || style.visibility === "hidden" || style.opacity === "0")) return false;
    const r = el.getBoundingClientRect?.();
    return Boolean(r && r.width > 0 && r.height > 0);
  }

  function buildSelectorCandidates(el) {
    if (!el) return { primary: "", fallbacks: [] };
    const tag = el.tagName.toLowerCase();
    const candidates = [];

    // Priority 1: Semantic ID (if not auto-generated hash)
    if (el.id && typeof el.id === "string") {
      const idTrim = el.id.trim();
      const isRandomHash = /^(:r\w+:|[0-9_-]+|[a-z]{1,3}\d+[a-z\d]*)$/i.test(idTrim);
      if (idTrim.length > 0 && !isRandomHash) {
        try {
          if (document.querySelectorAll(`#${idTrim}`).length === 1) {
            candidates.push(`#${idTrim}`);
          }
        } catch (_) {}
      }
    }

    // Priority 2: Distinct Form / Accessibility attributes (aria-label, placeholder, name)
    const aria = el.getAttribute?.("aria-label");
    if (aria) {
      const escapedAria = aria.replace(/'/g, "\\'");
      candidates.push(`${tag}[aria-label='${escapedAria}']`);
      candidates.push(`[aria-label='${escapedAria}']`);
    }

    const placeholder = el.getAttribute?.("placeholder");
    if (placeholder) {
      const escapedPl = placeholder.replace(/'/g, "\\'");
      candidates.push(`${tag}[placeholder='${escapedPl}']`);
    }

    if (el.name) {
      candidates.push(`${tag}[name='${el.name}']`);
    }

    // Priority 3: Test ID / Action
    const testId = el.getAttribute?.("data-testid") || el.getAttribute?.("data-action");
    if (testId) candidates.push(`[data-testid='${testId}']`);

    // Priority 4: Semantic Container & Web Component Hierarchy (helps disambiguate elements with identical classes in different containers)
    let parentContainer = el.parentElement;
    while (parentContainer && parentContainer !== document.body) {
      if (
        parentContainer.tagName.includes("-") ||
        parentContainer.matches("article, [role='article'], li, [role='listitem'], [role='row'], form, [role='search'], [role='main'], h1, h2, h3, h4")
      ) {
        break;
      }
      parentContainer = parentContainer.parentElement;
    }

    if (parentContainer && parentContainer !== el && parentContainer !== document.body) {
      const pTag = parentContainer.tagName.toLowerCase();
      const pId = parentContainer.id && !/^(:r\w+:|[0-9_-]+|[a-z]{1,3}\d+[a-z\d]*)$/i.test(parentContainer.id)
        ? `#${parentContainer.id.trim()}`
        : (parentContainer.getAttribute("aria-label") ? `${pTag}[aria-label='${parentContainer.getAttribute("aria-label").replace(/'/g, "\\'")}']` : pTag);
      candidates.push(`${pId} ${tag}`);
    }

    // Priority 5: Semantic Class (avoid generic utility/state classes)
    const cls = Array.from(el.classList || []).filter(c => 
      c && !["btn", "button", "active", "focus", "selected", "disabled"].includes(c)
    );
    for (const c of cls.slice(0, 2)) {
      candidates.push(`${tag}.${c}`);
    }

    if (candidates.length === 0) candidates.push(tag);

    // Find the first truly unique candidate (matches === 1 in DOM) if possible
    const unique = Array.from(new Set(candidates));
    let bestPrimary = unique[0] || tag;
    for (const cand of unique) {
      try {
        if (document.querySelectorAll(cand).length === 1) {
          bestPrimary = cand;
          break;
        }
      } catch (_) {}
    }

    const fallbacks = unique.filter(c => c !== bestPrimary);
    return {
      primary: bestPrimary,
      fallbacks: fallbacks
    };
  }

  function buildSelector(el) {
    return buildSelectorCandidates(el).primary;
  }

  // --- Snapshot Extractor ---
  function extractSnapshot() {
    const buttons = [];
    const btnNodes = document.querySelectorAll("button, [role='button'], [role='option'], [role='menuitem'], a, li[role='option'], input[type='button'], input[type='submit']");
    for (const b of btnNodes) {
      if (!isVisible(b)) continue;
      const text = (b.innerText || b.getAttribute("aria-label") || b.value || "").trim();
      if (text && text.length <= 100) {
        const { primary, fallbacks } = buildSelectorCandidates(b);
        buttons.push({
          tag: b.tagName.toLowerCase(),
          selector: primary,
          fallbacks: fallbacks.length > 0 ? fallbacks : undefined,
          text: text,
          aria_label: b.getAttribute("aria-label") || undefined
        });
      }
    }

    const inputs = [];
    const inpNodes = document.querySelectorAll("input:not([type='hidden']):not([type='file']):not([type='button']):not([type='submit']):not([type='reset']):not([type='checkbox']):not([type='radio']), textarea, [contenteditable='true']");
    for (const inp of inpNodes) {
      if (!isVisible(inp)) continue;
      const val = inp.value ? inp.value.trim() : (inp.isContentEditable ? inp.innerText.trim() : "");
      const placeholder = inp.placeholder || inp.getAttribute("aria-label") || "";
      const { primary, fallbacks } = buildSelectorCandidates(inp);
      inputs.push({
        tag: inp.tagName.toLowerCase(),
        selector: primary,
        fallbacks: fallbacks.length > 0 ? fallbacks : undefined,
        text: val || undefined,
        placeholder: inp.placeholder || undefined,
        aria_label: inp.getAttribute("aria-label") || undefined
      });
    }

    const texts = [];
    const textNodes = document.querySelectorAll("h1, h2, h3, h4, p, span, li, [role='article']");
    for (const t of textNodes) {
      if (!isVisible(t)) continue;
      const str = (t.innerText || "").trim().replace(/\s+/g, " ");
      if (str.length >= 10 && str.length <= 150 && !texts.includes(str)) texts.push(str);
    }

    return {
      url: window.location.href,
      domain: window.location.hostname,
      title: document.title || window.location.hostname,
      timestamp: Date.now(),
      buttons,
      inputs,
      texts,
      text_summary: texts.join("\n\n")
    };
  }

  let lastHash = "";
  function sendSnapshot() {
    try {
      if (!api?.runtime?.id) return;
      const snap = extractSnapshot();
      if (!snap) return;

      // Representative hash covers URL, Title, element counts, sample texts, and input values
      const sampleTexts = snap.texts.length > 0 ? (snap.texts[0] + snap.texts[snap.texts.length - 1]) : "";
      const inputValues = snap.inputs.map(i => `${i.selector || ""}=${i.text || ""}`).join("|");
      const hash = `${snap.url}::${snap.title}::${snap.texts.length}::${snap.buttons.length}::${snap.inputs.length}::${sampleTexts}::${inputValues}`;
      if (hash === lastHash) return;
      lastHash = hash;

      const p = api.runtime.sendMessage({ type: "DOM_SNAPSHOT", data: snap }, () => {
        if (api.runtime?.lastError) {
          // Extension context reloaded
        }
      });
      if (p && typeof p.catch === "function") {
        p.catch(() => {});
      }
    } catch (_) {}
  }

  // --- Interaction Execution ---
  function ensureElementInView(el) {
    if (!el) return;
    try {
      el.scrollIntoView({ behavior: "smooth", block: "center", inline: "nearest" });
      const rect = el.getBoundingClientRect();
      // If occluded by a fixed header (negative Y position or close to top)
      if (rect.top < 60) {
        window.scrollBy({ top: rect.top - 80, behavior: "smooth" });
      }
    } catch (_) {
      el.scrollIntoView?.(true);
    }
  }

  function clickElement(target) {
    if (!target) return;
    ensureElementInView(target);
    target.focus?.();
    const opts = { bubbles: true, cancelable: true, composed: true, view: window };
    target.dispatchEvent(new MouseEvent("mousedown", opts));
    target.dispatchEvent(new MouseEvent("mouseup", opts));
    target.click();
    setTimeout(sendSnapshot, 200);
  }

  function isWritable(el) {
    if (!el) return false;
    const tag = el.tagName.toLowerCase();
    if (tag === "textarea" || el.isContentEditable) return true;
    if (tag === "input") {
      const type = (el.type || "text").toLowerCase();
      return !["submit", "button", "reset", "checkbox", "radio", "hidden", "file", "image"].includes(type);
    }
    return false;
  }

  function typeElement(sel, text, fallbacks) {
    let el = null;
    if (sel) {
      try {
        const cand = document.querySelector(sel);
        if (isWritable(cand)) el = cand;
      } catch (_) {}
    }
    // Fallback selectors if primary selector not found or not a writable element
    if (!el && Array.isArray(fallbacks)) {
      for (const fb of fallbacks) {
        try {
          const cand = document.querySelector(fb);
          if (isWritable(cand)) {
            el = cand;
            break;
          }
        } catch (_) {}
      }
    }
    // Final fallback: Look for activeElement or common form input/textarea on page
    if (!el) {
      if (document.activeElement && isWritable(document.activeElement)) {
        el = document.activeElement;
      } else {
        const genericInputs = document.querySelectorAll("textarea[name='q'], input[name='q'], input[type='search'], textarea, input[type='text'], input:not([type='hidden'])");
        for (const cand of genericInputs) {
          if (isWritable(cand) && isVisible(cand)) {
            el = cand;
            break;
          }
        }
      }
    }
    if (!el) return false;
    ensureElementInView(el);
    el.focus?.();

    if (el.isContentEditable) {
      el.innerText = text;
    } else {
      const proto = el.tagName === "TEXTAREA" ? window.HTMLTextAreaElement.prototype : window.HTMLInputElement.prototype;
      const setter = Object.getOwnPropertyDescriptor(proto, "value")?.set;
      if (setter) setter.call(el, text);
      else el.value = text;
    }
    el.dispatchEvent(new Event("input", { bubbles: true }));
    el.dispatchEvent(new Event("change", { bubbles: true }));
    setTimeout(sendSnapshot, 100);
    return true;
  }

  function enterElement() {
    const el = document.activeElement || document.querySelector("input, textarea");
    if (!el) return;
    const ev = { key: "Enter", code: "Enter", keyCode: 13, which: 13, bubbles: true, cancelable: true, composed: true, view: window };
    el.dispatchEvent(new KeyboardEvent("keydown", ev));
    el.dispatchEvent(new KeyboardEvent("keypress", ev));
    el.dispatchEvent(new KeyboardEvent("keyup", ev));
    el.form?.submit?.();
    setTimeout(sendSnapshot, 200);
  }

  // --- Action Listener ---
  api?.runtime?.onMessage?.addListener((msg, _sender, sendResponse) => {
    if (msg?.type === "REQUEST_SNAPSHOT") {
      lastHash = "";
      sendSnapshot();
      sendResponse({ status: "ok" });
      return;
    }

    if (msg?.type === "EXECUTE_BROWSER_ACTION" && msg.command) {
      const cmd = msg.command;
      if (cmd.action.startsWith("CLICK_")) {
        const sel = (cmd.selector || "").trim();
        let el = null;
        if (sel) {
          try { el = document.querySelector(sel); } catch (_) {}
        }
        // Multi-strategy fallback selectors
        if (!el && Array.isArray(cmd.fallback_selectors)) {
          for (const fb of cmd.fallback_selectors) {
            try {
              el = document.querySelector(fb);
              if (el) break;
            } catch (_) {}
          }
        }
        // Fallback by Text
        if (!el && sel) {
          el = Array.from(document.querySelectorAll("button, a, [role='button']"))
            .find(b => (b.innerText || b.getAttribute("aria-label") || "").toLowerCase().includes(sel.toLowerCase()));
        }
        clickElement(el);
      } else if (cmd.action === "TYPE_TEXT") {
        typeElement(cmd.selector, cmd.text_to_type || "", cmd.fallback_selectors);
      } else if (cmd.action === "PRESS_ENTER") {
        enterElement();
      } else if (cmd.action === "SCROLL") {
        window.scrollBy({ top: cmd.scroll_y || 400, behavior: "smooth" });
      }
      sendResponse({ status: "ok" });
    }
  });

  // Init & SPA Reactive Observers
  sendSnapshot();
  window.addEventListener("focus", sendSnapshot, { passive: true });

  // Detect DOM mutations in SPAs with debouncing (500ms)
  let domTimer = null;
  const observer = new MutationObserver(() => {
    clearTimeout(domTimer);
    domTimer = setTimeout(sendSnapshot, 500);
  });
  if (document.body) {
    observer.observe(document.body, { childList: true, subtree: true });
  } else {
    window.addEventListener("DOMContentLoaded", () => {
      observer.observe(document.body, { childList: true, subtree: true });
    }, { once: true });
  }

  // Detect URL transitions in SPAs (pushState & popstate)
  window.addEventListener("popstate", () => setTimeout(sendSnapshot, 200), { passive: true });
  const origPushState = history.pushState;
  if (origPushState) {
    history.pushState = function (...args) {
      const res = origPushState.apply(this, args);
      setTimeout(sendSnapshot, 250);
      return res;
    };
  }
})();
