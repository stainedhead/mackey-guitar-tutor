import { useEffect, type RefObject } from 'react';

/** A swipe direction. */
export type Direction = 'left' | 'right' | 'up' | 'down';

/** A recognised swipe: its direction and how many fingers made it. */
export interface Swipe {
  direction: Direction;
  fingers: 1 | 2;
}

/** Least travel, in pixels, for a swipe. */
export const SWIPE_DISTANCE = 48;

/**
 * Classifies a touch movement: the dominant axis must travel at least `SWIPE_DISTANCE` and
 * clearly more than the other axis (1.5 times), so a diagonal drag is not a swipe. Two or more
 * fingers count as a two-finger swipe.
 */
export function classifySwipe(dx: number, dy: number, fingers: number): Swipe | null {
  const ax = Math.abs(dx);
  const ay = Math.abs(dy);
  const count: 1 | 2 = fingers >= 2 ? 2 : 1;
  if (ax >= SWIPE_DISTANCE && ax >= ay * 1.5) {
    return { direction: dx < 0 ? 'left' : 'right', fingers: count };
  }
  if (ay >= SWIPE_DISTANCE && ay >= ax * 1.5) {
    return { direction: dy < 0 ? 'up' : 'down', fingers: count };
  }
  return null;
}

/** The new visible span after a pinch: it shrinks when the fingers spread, within `min..=max`. */
export function pinchSpan(
  startSpan: number,
  startDistance: number,
  distance: number,
  min: number,
  max: number,
): number {
  if (startDistance <= 0 || distance <= 0) return startSpan;
  const span = Math.round((startSpan * startDistance) / distance);
  return Math.min(max, Math.max(min, span));
}

/** What a swipe on an element does. */
export interface SwipeHandlers {
  onSwipe: (swipe: Swipe) => void;
  /** Called while two fingers pinch, with the ratio of the current to the start distance. */
  onPinch?: (startDistance: number, distance: number) => void;
}

/** Recognises touch swipes and pinches on an element (iPad). Mouse input is ignored. */
export function useSwipe(ref: RefObject<Element | null>, handlers: SwipeHandlers): void {
  const { onSwipe, onPinch } = handlers;
  useEffect(() => {
    const el = ref.current;
    if (!el) return undefined;
    const points = new Map<number, { x: number; y: number }>();
    let start: { x: number; y: number } | null = null;
    let maxFingers = 0;
    let pinchStart = 0;
    const distance = (): number => {
      const [a, b] = [...points.values()];
      return a && b ? Math.hypot(a.x - b.x, a.y - b.y) : 0;
    };
    const down = (e: Event): void => {
      const p = e as PointerEvent;
      if (p.pointerType !== 'touch') return;
      points.set(p.pointerId, { x: p.clientX, y: p.clientY });
      if (points.size === 1) {
        start = { x: p.clientX, y: p.clientY };
        maxFingers = 1;
      } else {
        maxFingers = Math.max(maxFingers, points.size);
        pinchStart = distance();
      }
    };
    const move = (e: Event): void => {
      const p = e as PointerEvent;
      if (!points.has(p.pointerId)) return;
      points.set(p.pointerId, { x: p.clientX, y: p.clientY });
      if (points.size === 2 && onPinch) onPinch(pinchStart, distance());
    };
    const up = (e: Event): void => {
      const p = e as PointerEvent;
      const last = points.get(p.pointerId);
      if (!last) return;
      points.set(p.pointerId, { x: p.clientX, y: p.clientY });
      points.delete(p.pointerId);
      if (points.size === 0 && start) {
        const swipe = classifySwipe(p.clientX - start.x, p.clientY - start.y, maxFingers);
        if (swipe) onSwipe(swipe);
        start = null;
      }
    };
    el.addEventListener('pointerdown', down);
    el.addEventListener('pointermove', move);
    el.addEventListener('pointerup', up);
    el.addEventListener('pointercancel', up);
    return () => {
      el.removeEventListener('pointerdown', down);
      el.removeEventListener('pointermove', move);
      el.removeEventListener('pointerup', up);
      el.removeEventListener('pointercancel', up);
    };
  }, [ref, onSwipe, onPinch]);
}
