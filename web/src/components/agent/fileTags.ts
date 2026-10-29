/**
 * Canonical `@file` tag grammar shared by the chat input (autocomplete),
 * the message bubble renderer and any other place that parses tags.
 *
 * A tag is `@` immediately followed by one or more non-whitespace,
 * non-`@` characters: `@src/foo.rs`, `@src/foo.rs:10-20`.
 *
 * IMPORTANT — why this file exists:
 * The old implementation used `@[^\n]*`, whose character class allowed
 * whitespace. Two bugs came from that:
 *   1. Inserting a 2nd file tag replaced everything from the FIRST `@`
 *      to the end of the line, so the prompt could only ever hold 1 tag.
 *   2. Rendering highlighted the whole sentence after the first `@` as one
 *      tag, which is why entire messages turned cyan.
 * Keep every consumer on THIS grammar so both bugs stay fixed.
 */

const TAG_CORE = /@([^\s@]+)/gu;

/** Sentence punctuation that is never part of a path / line-range suffix. */
const TRAILING_PUNCT = /[,.;!?)\]}>"'»]+$/u;

/** Trim trailing sentence punctuation from a raw `@token`. */
export function stripTagPunctuation(rawTag: string): string {
  return rawTag.replace(TRAILING_PUNCT, '');
}

/** `@src/foo.rs:10-20` -> `src/foo.rs` */
export function tagToFilePath(rawTag: string): string {
  const withoutAt = rawTag.startsWith('@') ? rawTag.slice(1) : rawTag;
  return withoutAt.replace(/:\d+(?:-\d+)?$/, '');
}

export interface ParsedTagToken {
  text: string;
  isTag: boolean;
  filePath?: string;
}

/**
 * Split message content into plain-text and `@tag` tokens for rendering.
 * Pure text (including the removed punctuation) is preserved verbatim.
 */
export function parseFileTags(text: string): ParsedTagToken[] {
  const source = text || '';
  if (!source.includes('@')) {
    return source ? [{ text: source, isTag: false }] : [];
  }

  const result: ParsedTagToken[] = [];
  // Fresh regex per call: a module-level /g regex keeps `lastIndex` state and
  // would silently skip tags on the next invocation.
  const re = new RegExp(TAG_CORE.source, TAG_CORE.flags);
  let lastIndex = 0;
  let match: RegExpExecArray | null;

  while ((match = re.exec(source)) !== null) {
    const rawTag = `@${match[1]}`;
    const tag = stripTagPunctuation(rawTag);
    const tagIndex = match.index;

    if (tag === '@') {
      continue;
    }

    if (tagIndex > lastIndex) {
      result.push({ text: source.slice(lastIndex, tagIndex), isTag: false });
    }

    result.push({ text: tag, isTag: true, filePath: tagToFilePath(tag) });
    lastIndex = tagIndex + tag.length;
  }

  if (lastIndex < source.length) {
    result.push({ text: source.slice(lastIndex), isTag: false });
  }

  return result;
}

export interface ActiveTagQuery {
  /** Index of the `@` character that opens the token being typed. */
  start: number;
  /** Text typed after `@` (may be empty right after typing `@`). */
  query: string;
}

/**
 * Find the `@`-token the caret currently sits inside.
 * Returns `null` when the caret is not in a tag (whitespace or another `@`
 * already terminated it, or the `@` looks like part of an email address).
 */
export function findActiveTagQuery(text: string, caret: number): ActiveTagQuery | null {
  const safeCaret = Math.max(0, Math.min(caret, text.length));
  const before = text.slice(0, safeCaret);
  const at = before.lastIndexOf('@');
  if (at === -1) return null;

  const query = before.slice(at + 1);
  if (/[\s@]/.test(query)) return null;

  const prev = at === 0 ? '' : before[at - 1];
  if (prev && !/[\s([{<>"']/.test(prev)) return null;

  return { start: at, query };
}

export interface InsertTagResult {
  value: string;
  caret: number;
}

/**
 * Insert `@filePath ` at the caret, replacing only the token being typed.
 * Earlier tags elsewhere in the prompt are left untouched, so the input can
 * hold as many tags as the user wants.
 */
export function insertFileTag(
  text: string,
  caret: number,
  filePath: string,
  options?: { addTrailingSpace?: boolean }
): InsertTagResult {
  const safeCaret = Math.max(0, Math.min(caret, text.length));
  const insert = `@${filePath}${options?.addTrailingSpace === false ? '' : ' '}`;
  const active = findActiveTagQuery(text, safeCaret);
  const replaceStart = active ? active.start : safeCaret;

  const value = text.slice(0, replaceStart) + insert + text.slice(safeCaret);
  return { value, caret: replaceStart + insert.length };
}
