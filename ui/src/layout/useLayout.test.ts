import { describe, expect, it } from 'vitest';
import { layoutFor } from './useLayout';

describe('layoutFor', () => {
  const cases = [
    { label: 'Mac window', w: 1280, h: 800, touch: false, kind: 'mac', to: 15 },
    { label: 'iPad landscape', w: 1180, h: 820, touch: true, kind: 'ipad-landscape', to: 12 },
    { label: 'iPad portrait', w: 820, h: 1180, touch: true, kind: 'ipad-portrait', to: 7 },
    { label: 'Split View third', w: 400, h: 800, touch: true, kind: 'narrow', to: 12 },
    { label: 'narrow Mac window', w: 600, h: 800, touch: false, kind: 'narrow', to: 12 },
  ] as const;
  it.each(cases)('$label', ({ w, h, touch, kind, to }) => {
    const info = layoutFor(w, h, touch);
    expect(info.kind).toBe(kind);
    expect(info.defaultRange).toEqual({ from: 0, to });
  });
  it('touch layouts use the larger minimum fret width', () => {
    expect(layoutFor(1180, 820, true).minFretWidth).toBeGreaterThan(
      layoutFor(1280, 800, false).minFretWidth,
    );
  });
});
