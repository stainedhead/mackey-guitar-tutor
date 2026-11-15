import type { FretRange } from '../bindings/FretRange';

interface Props {
  zoom: FretRange;
  fretCount: number;
  fitRange: FretRange;
  onZoom: (range: FretRange) => void;
}

const MIN_SPAN = 5;
const SHIFT = 3;

/** Range handles (drag and keyboard), shift buttons and "Fit selection" (FB-1.5). */
export function RangeControls({ zoom, fretCount, fitRange, onZoom }: Props) {
  const setFrom = (from: number): void => {
    onZoom({ from, to: Math.max(zoom.to, Math.min(fretCount, from + MIN_SPAN)) });
  };
  const setTo = (to: number): void => {
    onZoom({ from: Math.min(zoom.from, Math.max(0, to - MIN_SPAN)), to });
  };
  const shift = (by: number): void => {
    const span = zoom.to - zoom.from;
    const from = Math.min(Math.max(0, zoom.from + by), Math.max(0, fretCount - span));
    onZoom({ from, to: from + span });
  };
  return (
    <div className="range-controls" role="group" aria-label="Visible frets">
      <button
        type="button"
        onClick={() => {
          shift(-SHIFT);
        }}
        aria-label="Show lower frets"
      >
        ◀ Lower
      </button>
      <label>
        First fret
        <input
          type="range"
          min={0}
          max={Math.max(0, fretCount - MIN_SPAN)}
          value={zoom.from}
          aria-valuetext={`Fret ${zoom.from.toString()}`}
          onChange={(e) => {
            setFrom(Number(e.target.value));
          }}
        />
      </label>
      <label>
        Last fret
        <input
          type="range"
          min={MIN_SPAN}
          max={fretCount}
          value={zoom.to}
          aria-valuetext={`Fret ${zoom.to.toString()}`}
          onChange={(e) => {
            setTo(Number(e.target.value));
          }}
        />
      </label>
      <button
        type="button"
        onClick={() => {
          shift(SHIFT);
        }}
        aria-label="Show higher frets"
      >
        Higher ▶
      </button>
      <button
        type="button"
        onClick={() => {
          onZoom(fitRange);
        }}
      >
        Fit selection
      </button>
    </div>
  );
}
