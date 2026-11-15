import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';

// Vitest blanks `?raw` CSS imports, so read the file directly.
const tokens = readFileSync(join(process.cwd(), 'src/theme/tokens.css'), 'utf8');

function block(selector: RegExp): Record<string, string> {
  const match = selector.exec(tokens);
  const out: Record<string, string> = {};
  if (!match) return out;
  for (const m of (match[1] ?? '').matchAll(/--([\w-]+):\s*(#[0-9a-f]{6})/gi))
    out[m[1] ?? ''] = m[2] ?? '';
  return out;
}

const light = block(/^:root \{([^}]*)\}/m);
const dark = { ...light, ...block(/^:root\[data-theme='dark'\] \{([^}]*)\}/m) };

function luminance(hex: string): number {
  const [r, g, b] = [1, 3, 5].map((i) => {
    const c = parseInt(hex.slice(i, i + 2), 16) / 255;
    return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
  }) as [number, number, number];
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x) as [number, number];
  return (hi + 0.05) / (lo + 0.05);
}

/** A token's value; a missing token fails the test with its name. */
function tok(t: Record<string, string>, name: string): string {
  const value = t[name];
  if (value === undefined) throw new Error(`missing token --${name}`);
  return value;
}

const ROLES = ['root', 'third', 'fifth', 'seventh', 'extension', 'single'] as const;

describe.each([
  ['light', light],
  ['dark', dark],
])('%s theme', (_name, t) => {
  it.each(ROLES)('role %s: its text reaches 4.5:1 on its fill', (role) => {
    expect(contrast(tok(t, `role-${role}`), tok(t, `role-${role}-text`))).toBeGreaterThanOrEqual(
      4.5,
    );
  });
  it.each(ROLES)('role %s: the marker edge reaches 3:1 on the neck', (role) => {
    // The outline separates the fill from the wood, so the edge must contrast with the wood.
    expect(role).toBeTruthy();
    expect(contrast(tok(t, 'marker-edge'), tok(t, 'neck-wood'))).toBeGreaterThanOrEqual(3);
  });
  it('body text reaches 4.5:1', () => {
    expect(contrast(tok(t, 'fg'), tok(t, 'bg'))).toBeGreaterThanOrEqual(4.5);
    expect(contrast(tok(t, 'muted'), tok(t, 'bg'))).toBeGreaterThanOrEqual(4.5);
  });
  it('the accent button text reaches 4.5:1', () => {
    expect(contrast(tok(t, 'accent'), tok(t, 'accent-fg'))).toBeGreaterThanOrEqual(4.5);
  });
  it('the scale ring reaches 3:1 on the neck', () => {
    expect(contrast(tok(t, 'role-scale'), tok(t, 'neck-wood'))).toBeGreaterThanOrEqual(3);
  });
});
