import type { ViewPosition } from '../bindings/ViewPosition';

interface Props {
  position: ViewPosition;
  x: number;
  y: number;
  width: number;
  radius: number;
}

/** The text lines of a tooltip (FB-1.6): note and octave, degree, interval, role, and the theory spelling. */
export function tooltipLines(position: ViewPosition): string[] {
  const t = position.tooltip;
  const lines = [t.note_octave, `Degree ${t.degree}`];
  if (t.interval !== null) lines.push(`Interval ${t.interval}`);
  lines.push(t.role);
  if (t.theory_spelling !== null) lines.push(`Theory spelling ${t.theory_spelling}`);
  return lines;
}

const LINE = 17;
const WIDTH = 150;

/** The hover or focus tooltip, drawn inside the neck SVG so it scrolls with it. */
export function Tooltip({ position, x, y, width, radius }: Props) {
  const lines = tooltipLines(position);
  const height = lines.length * LINE + 12;
  const left = Math.min(Math.max(x - WIDTH / 2, 2), Math.max(2, width - WIDTH - 2));
  const top = y - radius - height - 6 < 0 ? y + radius + 6 : y - radius - height - 6;
  return (
    <g className="tooltip" role="tooltip" data-testid="tooltip" pointerEvents="none">
      <rect x={left} y={top} width={WIDTH} height={height} rx={6} />
      {lines.map((line, i) => (
        <text
          key={line}
          x={left + 10}
          y={top + 8 + (i + 0.5) * LINE}
          dominantBaseline="central"
          fontSize={14.5}
        >
          {line}
        </text>
      ))}
    </g>
  );
}
