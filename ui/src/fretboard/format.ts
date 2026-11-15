import type { LabelsMode } from '../bindings/LabelsMode';
import type { Note } from '../bindings/Note';
import type { ViewPosition } from '../bindings/ViewPosition';

/** Formats a note Rust sent (letter plus accidental count) for display: "F#", "Bb". */
export function formatNote(note: Note): string {
  const acc = note.accidental;
  const mark = acc > 0 ? '#'.repeat(acc) : 'b'.repeat(-acc);
  return `${note.letter}${mark}`;
}

/** The text inside a marker for a labels mode. Every text is precomputed by Rust. */
export function labelFor(position: ViewPosition, mode: LabelsMode): string {
  switch (mode) {
    case 'NoteName':
      return position.labels.note_name;
    case 'ScaleDegree':
      return position.labels.scale_degree;
    case 'Interval':
      return position.labels.interval ?? position.labels.note_name;
    case 'Finger':
      return position.labels.finger ?? position.labels.note_name;
    case 'None':
      return '';
  }
}
