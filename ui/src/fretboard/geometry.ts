/**
 * Layout maths for the fretboard SVG: pixels only. This is drawing geometry, not music theory:
 * which notes light up, and every label, arrives from Rust in the `FretboardView`.
 */

/** Smallest width of one fret space (FB-1.4): 24 px on the Mac, 28 pt on the iPad. */
export const MIN_FRET_WIDTH = { mac: 24, ipad: 28 } as const;

/** Vertical distance between strings. */
export const STRING_GAP = 32;
/** Space for the string labels at the near edge. */
export const LABEL_MARGIN = 40;
/** Width of the open-string zone beside the nut. */
export const OPEN_ZONE = 48;
/** Padding after the last fret. */
export const END_PADDING = 16;
/** Space above the first string and below the last. */
export const VERTICAL_PADDING = 22;
/** Space under the neck for fret numbers. */
export const NUMBERS_HEIGHT = 26;

/** The share of the scale length from the nut to wire `n` (12th-root-of-2 spacing). */
export function wireFraction(n: number): number {
  return 1 - Math.pow(2, -n / 12);
}

/** The natural width of fret space `n` (1-based) as a share of the scale length. */
export function spaceFraction(n: number): number {
  return wireFraction(n) - wireFraction(n - 1);
}

/** Width in pixels of fret space `n` for a scale length, with the minimum applied. */
export function spaceWidth(n: number, scaleLength: number, minWidth: number): number {
  return Math.max(minWidth, scaleLength * spaceFraction(n));
}

/** Total width of fret spaces `from..=to` (1-based). */
export function spansWidth(
  from: number,
  to: number,
  scaleLength: number,
  minWidth: number,
): number {
  let sum = 0;
  for (let n = Math.max(1, from); n <= to; n += 1) sum += spaceWidth(n, scaleLength, minWidth);
  return sum;
}

/** Width of the neck chrome that is not fret space: labels, open zone and end padding. */
export function chromeWidth(showOpenZone: boolean): number {
  return LABEL_MARGIN + (showOpenZone ? OPEN_ZONE : 0) + END_PADDING;
}

/**
 * The scale length at which frets `from..=to` exactly fill `width` pixels (bisection; the
 * minimum fret width makes the sum non-linear). When even a zero scale length does not fit,
 * the minimum widths win and the neck is wider than `width`: the caller scrolls.
 */
export function scaleLengthForRange(
  from: number,
  to: number,
  width: number,
  minWidth: number,
): number {
  const available = width - chromeWidth(from <= 0);
  if (spansWidth(from, to, 0, minWidth) >= available) return 0;
  let low = 0;
  let high = 40_000;
  for (let i = 0; i < 40; i += 1) {
    const mid = (low + high) / 2;
    if (spansWidth(from, to, mid, minWidth) < available) low = mid;
    else high = mid;
  }
  return (low + high) / 2;
}

/** What the layout needs to know about the neck and its container. */
export interface NeckInput {
  fretCount: number;
  capo: number;
  scaleLength: number;
  minFretWidth: number;
  mirrored: boolean;
  flipped: boolean;
}

/** Positions on the neck, in SVG pixels. */
export interface NeckLayout {
  width: number;
  height: number;
  /** Left edge of fret space `n` (n >= 1) before mirroring, and its width. */
  spaces: readonly { fret: number; left: number; width: number }[];
  /** x of the centre of the marker for a position (before mirroring is applied: already applied). */
  xOfFret: (fret: number, isOpen: boolean) => number;
  /** x of fret wire `n` (0 is the nut), mirroring applied. */
  wireX: (n: number) => number;
  /** x of fret wire `n` before mirroring. */
  rawWireX: (n: number) => number;
  /** x of the nut, mirroring applied. */
  nutX: number;
  /** y of the string line. */
  yOfString: (string: number) => number;
  /** x for the string labels. */
  labelX: number;
  /** The string and fret under a point, or null. */
  cellAt: (x: number, y: number) => { string: number; fret: number } | null;
  /** The inclusive fret range whose spaces overlap the pixel span `left..=right`. */
  fretsBetween: (left: number, right: number) => { from: number; to: number };
  /** Mirrors an x coordinate when the neck is left-handed. */
  mirror: (x: number) => number;
}

/** Lays out the whole neck (all frets) at a scale length. */
export function layoutNeck(input: NeckInput): NeckLayout {
  const { fretCount, capo, scaleLength, minFretWidth, mirrored, flipped } = input;
  const rawWires: number[] = [LABEL_MARGIN + OPEN_ZONE];
  const spaces: { fret: number; left: number; width: number }[] = [];
  for (let n = 1; n <= fretCount; n += 1) {
    const left = rawWires[n - 1] ?? 0;
    const width = spaceWidth(n, scaleLength, minFretWidth);
    spaces.push({ fret: n, left, width });
    rawWires.push(left + width);
  }
  const last = rawWires[fretCount] ?? 0;
  const width = last + END_PADDING;
  const height = VERTICAL_PADDING * 2 + STRING_GAP * 5 + NUMBERS_HEIGHT;
  const mirror = (x: number): number => (mirrored ? width - x : x);
  const wireOf = (n: number): number => rawWires[Math.min(Math.max(n, 0), fretCount)] ?? 0;
  const rawX = (fret: number, isOpen: boolean): number => {
    if (isOpen) {
      // Open strings ring beside the nut, or just before the capo bar (the capo is the new nut).
      return capo > 0 ? wireOf(capo) - 16 : wireOf(0) - OPEN_ZONE / 2;
    }
    const space = spaces[fret - 1];
    return space ? space.left + space.width / 2 : wireOf(0);
  };
  const row = (string: number): number => (flipped ? string - 1 : 6 - string);
  const yOfString = (string: number): number => VERTICAL_PADDING + row(string) * STRING_GAP;
  return {
    width,
    height,
    spaces,
    mirror,
    xOfFret: (fret, isOpen) => mirror(rawX(fret, isOpen)),
    wireX: (n) => mirror(wireOf(n)),
    rawWireX: wireOf,
    nutX: mirror(wireOf(0)),
    yOfString,
    labelX: mirror(LABEL_MARGIN / 2),
    cellAt: (x, y) => {
      const rx = mirror(x);
      const r = Math.round((y - VERTICAL_PADDING) / STRING_GAP);
      if (r < 0 || r > 5) return null;
      const string = flipped ? r + 1 : 6 - r;
      if (rx < wireOf(0)) return { string, fret: capo };
      const space = spaces.find((s) => rx >= s.left && rx < s.left + s.width);
      return space ? { string, fret: space.fret } : null;
    },
    fretsBetween: (left, right) => {
      const a = mirror(left);
      const b = mirror(right);
      const lo = Math.min(a, b);
      const hi = Math.max(a, b);
      const inside = spaces.filter((s) => s.left + s.width > lo && s.left < hi);
      const first = inside[0];
      const lastSpace = inside[inside.length - 1];
      return {
        from: lo < wireOf(0) ? 0 : (first?.fret ?? 1),
        to: lastSpace?.fret ?? fretCount,
      };
    },
  };
}

/** The scroll offset that puts the left edge of frets `from..=to` at the container's edge. */
export function rangeScrollLeft(layout: NeckLayout, from: number, to: number): number {
  const rawLeft = from <= 0 ? 0 : layout.rawWireX(from - 1);
  const rawRight = layout.rawWireX(to);
  const edges = [layout.mirror(rawLeft), layout.mirror(rawRight)];
  return Math.max(0, Math.min(...edges));
}
