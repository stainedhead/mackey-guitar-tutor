import type { FretRange } from '../bindings/FretRange';
import type { OffRange } from '../bindings/OffRange';

interface Props {
  offRange: OffRange;
  visible: FretRange;
  mirrored: boolean;
  onFit: () => void;
}

/** The accessible name of an edge arrow (E-FB-5): "3 highlighted notes beyond fret 12". */
export function arrowLabel(count: number, fret: number, side: 'below' | 'above'): string {
  const notes = count === 1 ? '1 highlighted note' : `${count.toString()} highlighted notes`;
  return side === 'above'
    ? `${notes} beyond fret ${fret.toString()}`
    : `${notes} below fret ${fret.toString()}`;
}

/** Arrows at the edges of the neck when highlighted notes are out of view. */
export function EdgeArrows({ offRange, visible, mirrored, onFit }: Props) {
  const arrows: { side: 'below' | 'above'; count: number; fret: number }[] = [];
  if (offRange.below > 0) arrows.push({ side: 'below', count: offRange.below, fret: visible.from });
  if (offRange.above > 0) arrows.push({ side: 'above', count: offRange.above, fret: visible.to });
  return (
    <>
      {arrows.map((a) => {
        // Lower frets are on the left, except on a left-handed neck where the nut is right.
        const left = (a.side === 'below') !== mirrored;
        return (
          <button
            key={a.side}
            type="button"
            className={`edge-arrow ${left ? 'edge-left' : 'edge-right'}`}
            data-testid={`edge-arrow-${a.side}`}
            aria-label={arrowLabel(a.count, a.fret, a.side)}
            onClick={onFit}
          >
            {left ? '◀' : '▶'} {a.count}
          </button>
        );
      })}
    </>
  );
}
