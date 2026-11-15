import { useCallback, useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react';
import type { FretboardView } from '../bindings/FretboardView';
import type { FretRange } from '../bindings/FretRange';
import { pinchSpan, useSwipe, type Swipe } from '../gestures';
import { EdgeArrows } from './EdgeArrows';
import { Fretboard } from './Fretboard';
import { layoutNeck, rangeScrollLeft, scaleLengthForRange } from './geometry';
import { RangeControls } from './RangeControls';

interface Props {
  view: FretboardView;
  /** The frets the neck is zoomed to (target); the neck scrolls to show them. */
  zoom: FretRange;
  minFretWidth: number;
  showFretNumbers: boolean;
  touch: boolean;
  onZoom: (range: FretRange) => void;
  /** The frets actually in the viewport changed (for off-range counts). */
  onVisible: (range: FretRange) => void;
  onPlayNote: (string: number, fret: number) => void;
  onCellClick: (string: number, fret: number) => void;
  onSwipe: (swipe: Swipe) => void;
}

const FALLBACK_WIDTH = 960;

function useElementWidth(ref: React.RefObject<HTMLElement | null>): number {
  const [width, setWidth] = useState(FALLBACK_WIDTH);
  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return undefined;
    const measure = (): void => {
      if (el.clientWidth > 0) setWidth(el.clientWidth);
    };
    measure();
    if (typeof ResizeObserver === 'undefined') return undefined;
    const observer = new ResizeObserver(measure);
    observer.observe(el);
    return () => {
      observer.disconnect();
    };
  }, [ref]);
  return width;
}

/**
 * The neck in a scrolling frame: sized so the zoomed frets fill the frame (never narrower than
 * the minimum fret widths, FB-1.4), with edge arrows, range controls and touch gestures.
 */
export function FretboardPanel({
  view,
  zoom,
  minFretWidth,
  showFretNumbers,
  touch,
  onZoom,
  onVisible,
  onPlayNote,
  onCellClick,
  onSwipe,
}: Props) {
  const frame = useRef<HTMLDivElement>(null);
  const width = useElementWidth(frame);
  const scaleLength = scaleLengthForRange(zoom.from, zoom.to, width, minFretWidth);
  const layout = useMemo(
    () =>
      layoutNeck({
        fretCount: view.fret_count,
        capo: view.capo,
        scaleLength,
        minFretWidth,
        mirrored: view.mirrored,
        flipped: view.flipped,
      }),
    [view.fret_count, view.capo, scaleLength, minFretWidth, view.mirrored, view.flipped],
  );
  const programmatic = useRef(0);
  const report = useRef<number | null>(null);
  const toReport = useCallback(() => {
    const el = frame.current;
    if (!el) return;
    onVisible(layout.fretsBetween(el.scrollLeft, el.scrollLeft + el.clientWidth));
  }, [layout, onVisible]);

  useLayoutEffect(() => {
    const el = frame.current;
    if (!el) return;
    programmatic.current = Date.now();
    el.scrollLeft = rangeScrollLeft(layout, zoom.from, zoom.to);
    toReport();
  }, [layout, zoom.from, zoom.to, toReport]);

  useEffect(
    () => () => {
      if (report.current !== null) window.clearTimeout(report.current);
    },
    [],
  );

  const pinchBase = useRef(zoom);
  const onPinch = useCallback(
    (start: number, now: number) => {
      const span = pinchSpan(pinchBase.current.to - pinchBase.current.from, start, now, 5, 24);
      const from = Math.min(pinchBase.current.from, Math.max(0, view.fret_count - span));
      onZoom({ from, to: Math.min(view.fret_count, from + span) });
    },
    [onZoom, view.fret_count],
  );
  useEffect(() => {
    pinchBase.current = zoom;
  }, [zoom]);
  useSwipe(frame, { onSwipe, onPinch });

  const visibleNow = zoom;
  return (
    <section className="neck-dock" aria-label="Fretboard">
      <div className="neck-frame">
        <div
          ref={frame}
          className={touch ? 'neck-scroll neck-touch' : 'neck-scroll'}
          data-testid="neck-scroll"
          onScroll={() => {
            if (Date.now() - programmatic.current < 150) return;
            if (report.current !== null) window.clearTimeout(report.current);
            report.current = window.setTimeout(toReport, 150);
          }}
        >
          <Fretboard
            view={view}
            scaleLength={scaleLength}
            minFretWidth={minFretWidth}
            showFretNumbers={showFretNumbers}
            onPlayNote={onPlayNote}
            onCellClick={onCellClick}
          />
        </div>
        <EdgeArrows
          offRange={view.off_range}
          visible={visibleNow}
          mirrored={view.mirrored}
          onFit={() => {
            onZoom(view.fit_range);
          }}
        />
      </div>
      <RangeControls
        zoom={zoom}
        fretCount={view.fret_count}
        fitRange={view.fit_range}
        onZoom={onZoom}
      />
    </section>
  );
}
