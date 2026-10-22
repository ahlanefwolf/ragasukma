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

function escapeCodeHtml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#39;');
}

export function highlightLine(line: string, filename?: string): string {
  if (!line || line.trim() === '') return '&nbsp;';
  const ext = filename?.split('.').pop()?.toLowerCase() || '';
  const langMap: { [key: string]: string } = {
    ts: 'typescript',
    tsx: 'tsx',
    js: 'javascript',
    jsx: 'jsx',
    rs: 'rust',
    py: 'python',
    json: 'json',
    css: 'css',
    scss: 'css',
    html: 'html',
    htm: 'html',
    sh: 'bash',
    bash: 'bash',
    zsh: 'bash',
    md: 'markdown',
    toml: 'toml',
    yaml: 'yaml',
    yml: 'yaml',
    sql: 'sql',
  };
  const lang = langMap[ext] || 'javascript';
  const grammar = Prism.languages[lang] || Prism.languages.javascript || Prism.languages.markup;

  try {
    return Prism.highlight(line, grammar, lang);
  } catch {
    return escapeCodeHtml(line);
  }
}

export function highlightCode(code: string, filename: string): string[] {
  const ext = filename.split('.').pop()?.toLowerCase() || '';
  const langMap: { [key: string]: string } = {
    ts: 'typescript',
    tsx: 'tsx',
    js: 'javascript',
    jsx: 'jsx',
    rs: 'rust',
    py: 'python',
    json: 'json',
    css: 'css',
    scss: 'css',
    html: 'html',
    htm: 'html',
    sh: 'bash',
    bash: 'bash',
    zsh: 'bash',
    md: 'markdown',
    toml: 'toml',
    yaml: 'yaml',
    yml: 'yaml',
    sql: 'sql',
  };
  const lang = langMap[ext] || 'javascript';
  const grammar = Prism.languages[lang] || Prism.languages.javascript || Prism.languages.markup;

  try {
    const highlighted = Prism.highlight(code, grammar, lang);
    return highlighted.split('\n');
  } catch {
    return code.split('\n').map((l) => escapeCodeHtml(l));
  }
}

export function countMessageLines(content?: string): number {
  return content ? content.split('\n').length : 0;
}
