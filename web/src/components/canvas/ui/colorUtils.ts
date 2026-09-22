/**
 * Utilitas normalisasi dan konversi format warna: Hex <-> RGBA <-> Transparent
 */

export interface ParsedColor {
  hex: string;
  alpha: number; // 0 - 100
}

export function parseColor(col: string): ParsedColor {
  if (!col || col === 'transparent') {
    return { hex: '#29483A', alpha: 0 };
  }
  if (col.startsWith('rgba')) {
    const match = col.match(
      /rgba?\s*\(\s*(\d+)\s*,\s*(\d+)\s*,\s*(\d+)\s*(?:,\s*([\d.]+)\s*)?\)/i
    );
    if (match) {
      const r = parseInt(match[1], 10);
      const g = parseInt(match[2], 10);
      const b = parseInt(match[3], 10);
      const a = match[4] !== undefined ? Math.round(parseFloat(match[4]) * 100) : 100;
      const toHex = (n: number) => n.toString(16).padStart(2, '0');
      return { hex: `#${toHex(r)}${toHex(g)}${toHex(b)}`, alpha: a };
    }
  }
  if (col.startsWith('#')) {
    if (col.length === 9) {
      const base = col.slice(0, 7);
      const a = Math.round((parseInt(col.slice(7, 9), 16) / 255) * 100);
      return { hex: base, alpha: a };
    }
    return { hex: col.length === 7 ? col : '#29483A', alpha: 100 };
  }
  return { hex: '#29483A', alpha: 100 };
}

export function toColorString(hex: string, alphaPercent: number): string {
  if (alphaPercent <= 0) return 'transparent';
  if (alphaPercent >= 100) return hex;
  const r = parseInt(hex.slice(1, 3), 16) || 0;
  const g = parseInt(hex.slice(3, 5), 16) || 0;
  const b = parseInt(hex.slice(5, 7), 16) || 0;
  const a = (alphaPercent / 100).toFixed(2).replace(/\.?0+$/, '');
  return `rgba(${r}, ${g}, ${b}, ${a})`;
}
