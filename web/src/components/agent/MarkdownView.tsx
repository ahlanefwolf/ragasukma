import { Component, createMemo } from 'solid-js';
import { marked, Tokens } from 'marked';
import Prism from 'prismjs';
import 'prismjs/themes/prism-tomorrow.css';
import 'prismjs/components/prism-typescript';
import 'prismjs/components/prism-jsx';
import 'prismjs/components/prism-tsx';
import 'prismjs/components/prism-rust';
import 'prismjs/components/prism-python';
import 'prismjs/components/prism-json';
import 'prismjs/components/prism-bash';
import 'prismjs/components/prism-toml';
import 'prismjs/components/prism-css';
import 'prismjs/components/prism-markdown';
import 'prismjs/components/prism-sql';
import 'prismjs/components/prism-yaml';

interface MarkdownViewProps {
  content: string;
}

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

function highlightMarkdownCode(code: string, lang: string): string {
  const cleanLang = (lang || '').toLowerCase().trim();
  const aliasMap: { [k: string]: string } = {
    js: 'javascript',
    ts: 'typescript',
    rs: 'rust',
    py: 'python',
    sh: 'bash',
    shell: 'bash',
    zsh: 'bash',
    yml: 'yaml',
    md: 'markdown',
  };
  const targetLang = aliasMap[cleanLang] || cleanLang;
  const grammar = Prism.languages[targetLang] || Prism.languages.javascript || Prism.languages.markup;

  try {
    return Prism.highlight(code, grammar, targetLang);
  } catch {
    return escapeHtml(code);
  }
}

const customRenderer = new marked.Renderer();

// Code blocks with syntax highlighting, language header, and copy button
customRenderer.code = function ({ text, lang }: Tokens.Code) {
  const language = lang ? lang.split(/\s+/)[0] : 'code';
  const cleanLang = language.toLowerCase();
  const highlighted = highlightMarkdownCode(text, cleanLang);

  return `
    <div class="code-block-wrapper my-2.5 rounded-xl bg-[#0d1117] border border-white/10 overflow-hidden shadow-lg font-mono">
      <div class="flex items-center justify-between px-3 py-1.5 bg-white/[0.04] border-b border-white/10 text-2xs">
        <div class="flex items-center gap-1.5">
          <span class="w-2 h-2 rounded-full bg-indigo-500/80"></span>
          <span class="text-indigo-300 font-semibold uppercase tracking-wider">${cleanLang}</span>
        </div>
        <button
          type="button"
          class="copy-code-btn px-2 py-0.5 rounded bg-white/5 hover:bg-white/15 text-white/60 hover:text-white transition-all flex items-center gap-1 cursor-pointer font-sans text-2xs"
        >
          Copy Code
        </button>
      </div>
      <pre class="p-3 overflow-x-auto text-xs leading-relaxed custom-scrollbar whitespace-pre"><code class="language-${cleanLang}">${highlighted}</code></pre>
    </div>
  `;
};

// Inline code (escape html to prevent rendering raw HTML tags like <textarea> or <script>)
customRenderer.codespan = function ({ text }: Tokens.Codespan) {
  return `<code class="px-1.5 py-0.5 mx-0.5 rounded bg-theme-cyan/10 text-theme-cyan font-mono text-xs border border-theme-cyan/20">${escapeHtml(text)}</code>`;
};

// Raw HTML blocks/tags inside markdown should be escaped to prevent UI breaking
customRenderer.html = function ({ text }: Tokens.HTML) {
  return escapeHtml(text);
};

// Links
customRenderer.link = function ({ href, text }: Tokens.Link) {
  return `<a href="${href}" target="_blank" rel="noopener noreferrer" class="text-theme-cyan hover:opacity-80 underline underline-offset-2">${text}</a>`;
};

export const MarkdownView: Component<MarkdownViewProps> = (props) => {
  const parsedHtml = createMemo(() => {
    const raw = props.content || '';
    try {
      return marked.parse(raw, {
        renderer: customRenderer,
        breaks: true,
        gfm: true,
      }) as string;
    } catch {
      return escapeHtml(raw);
    }
  });

  return (
    <div class="markdown-view-wrapper">
      <style>{`
        .markdown-view-wrapper .markdown-content {
          color: var(--text);
          font-size: 0.75rem;
          line-height: 1.625;
        }
        .markdown-view-wrapper .markdown-content p {
          margin-top: 0.35rem;
          margin-bottom: 0.35rem;
        }
        .markdown-view-wrapper .markdown-content h1 {
          font-size: 0.95rem;
          font-weight: 700;
          color: var(--text);
          margin-top: 0.85rem;
          margin-bottom: 0.35rem;
          border-bottom: 1px solid var(--border);
          padding-bottom: 0.25rem;
        }
        .markdown-view-wrapper .markdown-content h2 {
          font-size: 0.85rem;
          font-weight: 700;
          color: var(--text);
          margin-top: 0.75rem;
          margin-bottom: 0.3rem;
        }
        .markdown-view-wrapper .markdown-content h3 {
          font-size: 0.8rem;
          font-weight: 600;
          color: var(--cyan);
          margin-top: 0.65rem;
          margin-bottom: 0.25rem;
        }
        .markdown-view-wrapper .markdown-content ul {
          list-style-type: disc;
          padding-left: 1.25rem;
          margin-top: 0.35rem;
          margin-bottom: 0.35rem;
        }
        .markdown-view-wrapper .markdown-content ol {
          list-style-type: decimal;
          padding-left: 1.25rem;
          margin-top: 0.35rem;
          margin-bottom: 0.35rem;
        }
        .markdown-view-wrapper .markdown-content li {
          margin-top: 0.15rem;
          margin-bottom: 0.15rem;
        }
        .markdown-view-wrapper .markdown-content blockquote {
          border-left: 3px solid var(--cyan);
          padding-left: 0.75rem;
          margin-top: 0.5rem;
          margin-bottom: 0.5rem;
          color: var(--muted);
          font-style: italic;
        }
        .markdown-view-wrapper .markdown-content table {
          width: 100%;
          border-collapse: collapse;
          margin-top: 0.6rem;
          margin-bottom: 0.6rem;
          font-size: 0.75rem;
        }
        .markdown-view-wrapper .markdown-content th,
        .markdown-view-wrapper .markdown-content td {
          border: 1px solid var(--border);
          padding: 0.35rem 0.6rem;
        }
        .markdown-view-wrapper .markdown-content th {
          background-color: var(--surface);
          font-weight: 600;
          color: var(--text);
        }
        .markdown-view-wrapper .markdown-content hr {
          border: 0;
          border-top: 1px solid var(--border);
          margin: 0.75rem 0;
        }
      `}</style>
      <div
        class="markdown-content select-text"
        innerHTML={parsedHtml()}
        onClick={(e) => {
          const target = e.target as HTMLElement;
          const btn = target.closest('.copy-code-btn');
          if (btn) {
            const wrapper = btn.closest('.code-block-wrapper');
            const codeEl = wrapper?.querySelector('pre code');
            if (codeEl) {
              navigator.clipboard.writeText(codeEl.textContent || '');
              const prevText = btn.textContent;
              btn.textContent = '✓ Copied!';
              btn.classList.add('text-emerald-400');
              setTimeout(() => {
                btn.textContent = prevText;
                btn.classList.remove('text-emerald-400');
              }, 1800);
            }
          }
        }}
      />
    </div>
  );
};
