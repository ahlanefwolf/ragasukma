import changelogRaw from '../../../CHANGELOG.md?raw';

/**
 * Single source of truth for release history.
 *
 * The changelog lives in the repository root `CHANGELOG.md` and is imported at
 * build time via Vite's `?raw` suffix, so the dashboard can never show a
 * version list that has drifted from the file maintainers actually edit.
 */

export interface ChangelogEntry {
  /** Normalised version without the leading `v` (e.g. "0.4.0"). */
  version: string;
  /** Original tag as written in the changelog (e.g. "v0.4.0"). */
  tag: string;
  /** Release date in `YYYY-MM-DD`, or `''` when unspecified. */
  date: string;
  /** Markdown body of the entry, with the heading line stripped. */
  notes: string;
}

const HEADING_RE = /^##\s+\[?v?([\w.\-]+)\]?(?:\s*[-–—]\s*(\d{4}-\d{2}-\d{2}))?/;

function splitSections(markdown: string): ChangelogEntry[] {
  const lines = markdown.split(/\r?\n/);
  const entries: ChangelogEntry[] = [];
  let current: ChangelogEntry | null = null;
  let buffer: string[] = [];

  const flush = () => {
    if (!current) return;
    current.notes = buffer.join('\n').replace(/^\s*\n+/, '').replace(/\s+$/, '');
    entries.push(current);
  };

  for (const line of lines) {
    const match = line.match(HEADING_RE);
    if (match) {
      flush();
      const version = match[1];
      current = {
        version,
        tag: `v${version}`,
        date: match[2] ?? '',
        notes: '',
      };
      buffer = [];
      continue;
    }
    if (current) buffer.push(line);
  }
  flush();

  return entries;
}

/** All releases, newest first (as authored in the changelog). */
export const CHANGELOG_ENTRIES: ChangelogEntry[] = splitSections(changelogRaw);

/**
 * Looks up a single release by version. Accepts a tag with or without the
 * leading `v` and tolerates trailing whitespace.
 */
export function getChangelogEntry(version: string): ChangelogEntry | undefined {
  const needle = version.trim().replace(/^v/i, '');
  if (!needle) return undefined;
  return CHANGELOG_ENTRIES.find((e) => e.version === needle);
}

/** The most recent release recorded in the changelog, if any. */
export function getLatestChangelogEntry(): ChangelogEntry | undefined {
  return CHANGELOG_ENTRIES[0];
}
