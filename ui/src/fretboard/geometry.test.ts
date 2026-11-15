import { describe, expect, it } from 'vitest';
import {
  MIN_FRET_WIDTH,
  layoutNeck,
  scaleLengthForRange,
  spaceFraction,
  spansWidth,
  wireFraction,
} from './geometry';

describe('fret spacing', () => {
  it('the 12th wire is half the scale length', () => {
    expect(wireFraction(12)).toBeCloseTo(0.5, 10);
  });
  it('the 22 spaces sum to the 22nd wire fraction', () => {
    let sum = 0;
    for (let n = 1; n <= 22; n += 1) sum += spaceFraction(n);
    expect(sum).toBeCloseTo(wireFraction(22), 10);
  });
  it('spaces shrink towards the body', () => {
    expect(spaceFraction(1)).toBeGreaterThan(spaceFraction(2));
    expect(spaceFraction(21)).toBeGreaterThan(spaceFraction(22));
  });
});

describe('scaleLengthForRange', () => {
  const cases = [
    { from: 0, to: 12, width: 1000 },
    { from: 0, to: 15, width: 1200 },
    { from: 5, to: 9, width: 600 },
  ];
  it.each(cases)('frets $from-$to fill $width px', ({ from, to, width }) => {
    const length = scaleLengthForRange(from, to, width, MIN_FRET_WIDTH.mac);
    const chrome = 40 + (from <= 0 ? 48 : 0) + 16;
    expect(spansWidth(from, to, length, MIN_FRET_WIDTH.mac) + chrome).toBeCloseTo(width, 0);
  });
  it('minimum widths win when the range cannot fit', () => {
    expect(scaleLengthForRange(0, 22, 200, MIN_FRET_WIDTH.mac)).toBe(0);
  });
});

describe('layoutNeck', () => {
  const base = {
    fretCount: 22,
    capo: 0,
    scaleLength: 1400,
    minFretWidth: 24,
    mirrored: false,
    flipped: false,
  };
  it('no fret space is narrower than the minimum', () => {
    const layout = layoutNeck({ ...base, scaleLength: 300 });
    expect(layout.spaces).toHaveLength(22);
    for (const s of layout.spaces) expect(s.width).toBeGreaterThanOrEqual(24);
  });
  it('mirroring puts the nut on the right (AC-LH-1.1)', () => {
    const normal = layoutNeck(base);
    const mirrored = layoutNeck({ ...base, mirrored: true });
    expect(normal.nutX).toBeLessThan(normal.width / 2);
    expect(mirrored.nutX).toBeGreaterThan(mirrored.width / 2);
    expect(mirrored.nutX).toBeCloseTo(normal.width - normal.nutX, 5);
  });
  it('string 1 is drawn at the bottom and flipping reverses the order', () => {
    const normal = layoutNeck(base);
    const flipped = layoutNeck({ ...base, flipped: true });
    expect(normal.yOfString(1)).toBeGreaterThan(normal.yOfString(6));
    expect(flipped.yOfString(1)).toBeLessThan(flipped.yOfString(6));
  });
  it('finds the cell under a point', () => {
    const layout = layoutNeck(base);
    const space = layout.spaces[2];
    expect(space).toBeDefined();
    const x = layout.rawWireX(0) + 1;
    expect(layout.cellAt(x, layout.yOfString(3))?.string).toBe(3);
  });
});
