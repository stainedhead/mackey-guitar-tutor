import { useEffect, useState } from 'react';
import type { FretRange } from '../bindings/FretRange';
import { MIN_FRET_WIDTH } from '../fretboard/geometry';

/** The four layouts of 07-ux-flows: Mac, iPad landscape, iPad portrait, and a narrow window. */
export type LayoutKind = 'mac' | 'ipad-landscape' | 'ipad-portrait' | 'narrow';

/** Width below which the sidebar hides and panels collapse (E-FB-9, Split View one-third). */
export const NARROW_WIDTH = 700;

/** What a layout decides: the default visible frets and the smallest fret width. */
export interface LayoutInfo {
  kind: LayoutKind;
  touch: boolean;
  minFretWidth: number;
  defaultRange: FretRange;
}

/** Picks the layout for a window size and input type (pure; tested without a window). */
export function layoutFor(width: number, height: number, touch: boolean): LayoutInfo {
  const minFretWidth = touch ? MIN_FRET_WIDTH.ipad : MIN_FRET_WIDTH.mac;
  if (width < NARROW_WIDTH) {
    return { kind: 'narrow', touch, minFretWidth, defaultRange: { from: 0, to: 12 } };
  }
  if (touch) {
    return height > width
      ? { kind: 'ipad-portrait', touch, minFretWidth, defaultRange: { from: 0, to: 7 } }
      : { kind: 'ipad-landscape', touch, minFretWidth, defaultRange: { from: 0, to: 12 } };
  }
  return { kind: 'mac', touch, minFretWidth, defaultRange: { from: 0, to: 15 } };
}

function isTouch(): boolean {
  return navigator.maxTouchPoints > 0 || window.matchMedia('(pointer: coarse)').matches;
}

/** The current layout, following window resizes and rotation. */
export function useLayout(): LayoutInfo {
  const [info, setInfo] = useState<LayoutInfo>(() =>
    layoutFor(window.innerWidth, window.innerHeight, isTouch()),
  );
  useEffect(() => {
    const update = (): void => {
      setInfo((old) => {
        const next = layoutFor(window.innerWidth, window.innerHeight, isTouch());
        return old.kind === next.kind && old.touch === next.touch ? old : next;
      });
    };
    window.addEventListener('resize', update);
    return () => {
      window.removeEventListener('resize', update);
    };
  }, []);
  return info;
}
