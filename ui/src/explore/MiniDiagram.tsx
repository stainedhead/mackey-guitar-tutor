import { memo } from 'react';
import type { VoicingListEntry } from '../bindings/VoicingListEntry';

const STRINGS = 6;
const ROWS = 5;
const GAP_X = 14;
const GAP_Y = 15;
const LEFT = 18;
const TOP = 22;

/** "x32010": the fret per string, low to high, with x for muted. */
export function fretText(entry: VoicingListEntry): string {
  return entry.strings
    .map((s) => (s.kind === 'muted' ? 'x' : s.kind === 'open' ? '0' : s.fret.toString()))
    .join(entry.strings.some((s) => s.kind === 'fretted' && s.fret >= 10) ? ' ' : '');
}

/** A small chord diagram: strings low to high, a window of frets from the voicing's lowest. */
export const MiniDiagram = memo(function MiniDiagram({ entry }: { entry: VoicingListEntry }) {
  const first = Math.max(1, entry.lowest_fret);
  const x = (i: number): number => LEFT + i * GAP_X;
  const y = (fret: number): number => TOP + (fret - first + 0.5) * GAP_Y;
  const width = LEFT + (STRINGS - 1) * GAP_X + 12;
  const height = TOP + ROWS * GAP_Y + 6;
  return (
    <svg
      className="mini-diagram"
      width={width}
      height={height}
      viewBox={`0 0 ${width.toString()} ${height.toString()}`}
      role="img"
      aria-label={`${entry.name}: ${fretText(entry)}`}
    >
      {first > 1 && (
        <text className="mini-fret" x={2} y={y(first)} fontSize={11} dominantBaseline="central">
          {first}
        </text>
      )}
      {Array.from({ length: ROWS + 1 }, (_, r) => (
        <line
          key={`h${r.toString()}`}
          className={r === 0 && first === 1 ? 'mini-nut' : 'mini-line'}
          x1={x(0)}
          x2={x(STRINGS - 1)}
          y1={TOP + r * GAP_Y}
          y2={TOP + r * GAP_Y}
        />
      ))}
      {Array.from({ length: STRINGS }, (_, i) => (
        <line
          key={`v${i.toString()}`}
          className="mini-line"
          x1={x(i)}
          x2={x(i)}
          y1={TOP}
          y2={TOP + ROWS * GAP_Y}
        />
      ))}
      {entry.barres.map((b) => (
        <rect
          key={`b${b.fret.toString()}`}
          className="mini-barre"
          x={x(6 - b.from) - 5}
          y={y(b.fret) - 5}
          width={x(6 - b.to) - x(6 - b.from) + 10}
          height={10}
          rx={5}
        />
      ))}
      {entry.strings.map((s, i) => {
        const key = `s${i.toString()}`;
        if (s.kind === 'muted') {
          return (
            <text
              key={key}
              className="mini-mute"
              x={x(i)}
              y={TOP - 8}
              textAnchor="middle"
              fontSize={12}
            >
              ×
            </text>
          );
        }
        if (s.kind === 'open') {
          return <circle key={key} className="mini-open" cx={x(i)} cy={TOP - 9} r={4} />;
        }
        const finger = entry.fingers[i];
        return (
          <g key={key}>
            <circle className="mini-dot" cx={x(i)} cy={y(s.fret)} r={6} />
            {finger !== null && finger !== undefined && finger > 0 && (
              <text
                className="mini-finger"
                x={x(i)}
                y={y(s.fret)}
                textAnchor="middle"
                dominantBaseline="central"
                fontSize={9}
              >
                {finger}
              </text>
            )}
          </g>
        );
      })}
    </svg>
  );
});
