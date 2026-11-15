import { memo } from 'react';
import type { NoteRole } from '../bindings/NoteRole';

/** The CSS class that carries a role's colour tokens (G-2). */
export function roleClass(role: NoteRole): string {
  return `note-${role.toLowerCase()}`;
}

/** The shape a role is drawn as; colour is never the only cue (G-2). */
export type Shape = 'square' | 'circle' | 'ring' | 'diamond';

/** Root is a rounded square, a lone note a diamond, a scale tone a ring, chord tones circles. */
export function shapeFor(role: NoteRole): Shape {
  switch (role) {
    case 'Root':
      return 'square';
    case 'ScaleTone':
      return 'ring';
    case 'SingleNote':
      return 'diamond';
    case 'Third':
    case 'Fifth':
    case 'Seventh':
    case 'Extension':
      return 'circle';
  }
}

interface Props {
  role: NoteRole;
  x: number;
  y: number;
  radius: number;
  label: string;
  fontSize: number;
  order: number | null;
}

/** One note: its shape, its text and, with "Show order", the play-order number. */
export const NoteMarker = memo(function NoteMarker({
  role,
  x,
  y,
  radius,
  label,
  fontSize,
  order,
}: Props) {
  const cls = `marker ${roleClass(role)}`;
  const shape = shapeFor(role);
  return (
    <g className={cls} data-role={role} data-shape={shape} transform={`translate(${x} ${y})`}>
      {shape === 'square' && (
        <rect
          className="marker-shape"
          x={-radius}
          y={-radius}
          width={radius * 2}
          height={radius * 2}
          rx={radius * 0.3}
        />
      )}
      {shape === 'diamond' && (
        <rect
          className="marker-shape"
          x={-radius * 0.85}
          y={-radius * 0.85}
          width={radius * 1.7}
          height={radius * 1.7}
          transform="rotate(45)"
        />
      )}
      {(shape === 'circle' || shape === 'ring') && (
        <circle className="marker-shape" r={shape === 'ring' ? radius - 1.5 : radius} />
      )}
      {label !== '' && (
        <text
          className="marker-label"
          textAnchor="middle"
          dominantBaseline="central"
          fontSize={fontSize}
        >
          {label}
        </text>
      )}
      {order !== null && (
        <g className="marker-order" transform={`translate(${radius * 0.8} ${-radius * 0.8})`}>
          <circle r={7} />
          <text textAnchor="middle" dominantBaseline="central" fontSize={10}>
            {order}
          </text>
        </g>
      )}
    </g>
  );
});
