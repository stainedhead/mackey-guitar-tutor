import { useRef } from 'react';
import type { ExplorerPanel } from '../bindings/ExplorerPanel';
import type { Degree } from '../bindings/Degree';
import { useSwipe, type Swipe } from '../gestures';

interface Props {
  panel: ExplorerPanel;
  badge: string | null;
  onSelect: (degree: Degree) => void;
  onSevenths: (on: boolean) => void;
  onStep: (direction: 'next' | 'prev') => void;
}

/** The diatonic chord strip (FB-4.1, FB-4.2): Roman numerals and chord names, sevenths toggle. */
export function ChordStrip({ panel, badge, onSelect, onSevenths, onStep }: Props) {
  const ref = useRef<HTMLDivElement>(null);
  useSwipe(ref, {
    onSwipe: (swipe: Swipe) => {
      if (swipe.direction === 'up') onStep('prev');
      if (swipe.direction === 'down') onStep('next');
    },
  });
  return (
    <section className="chord-strip" aria-label="Chords in the key">
      <div ref={ref} className="chips" role="group" aria-label="Diatonic chords">
        {panel.diatonic.map((chip) => (
          <button
            key={chip.degree}
            type="button"
            className={chip.selected ? 'chip selected' : 'chip'}
            aria-pressed={chip.selected}
            aria-label={`${chip.roman}, ${chip.symbol}`}
            onClick={() => {
              onSelect(chip.degree);
            }}
          >
            <span className="chip-roman">{chip.roman}</span>
            <span className="chip-symbol">{chip.symbol}</span>
          </button>
        ))}
      </div>
      <div className="strip-tools">
        <button
          type="button"
          aria-label="Previous chord in the key"
          onClick={() => {
            onStep('prev');
          }}
        >
          ▲
        </button>
        <button
          type="button"
          aria-label="Next chord in the key"
          onClick={() => {
            onStep('next');
          }}
        >
          ▼
        </button>
        <label className="check">
          <input
            type="checkbox"
            checked={panel.selection.show_sevenths}
            onChange={(e) => {
              onSevenths(e.target.checked);
            }}
          />
          Sevenths
        </label>
        {badge !== null && (
          <span className="badge" data-testid="key-badge">
            {badge}
          </span>
        )}
      </div>
    </section>
  );
}
