import type { TunerReading } from '../bindings/TunerReading';
import type { TunerTarget } from '../bindings/TunerTarget';
import { formatNote } from '../fretboard/format';

/** What the tuner display says for a reading: all text comes from Rust's status and numbers. */
export interface Description {
  /** The big text: "A2", "In tune", or an instruction. */
  headline: string;
  /** A second line, or null. */
  detail: string | null;
  /** Visual state; never the only cue (the text and the check mark carry it too). */
  tone: 'idle' | 'warn' | 'near' | 'good';
  /** The text for the live region, throttled by the caller. */
  speech: string;
}

/** "A2" for a reading's note and octave. */
export function noteText(reading: TunerReading): string {
  return reading.note === null
    ? ''
    : `${formatNote(reading.note)}${(reading.octave ?? 0).toString()}`;
}

/** "18 cents flat", "3 cents sharp", "In tune". */
export function centsText(cents: number): string {
  const rounded = Math.round(Math.abs(cents));
  if (rounded === 0) return 'In tune';
  return `${rounded.toString()} cents ${cents < 0 ? 'flat' : 'sharp'}`;
}

/** Turns a reading into display text (TU-1, TU-2, E-TU-3, E-TU-4, E-TU-6). */
export function describe(reading: TunerReading | null, hasTargets: boolean): Description {
  if (reading === null || reading.status === 'Listening') {
    return { headline: 'Listening…', detail: 'Play a note', tone: 'idle', speech: '' };
  }
  switch (reading.status) {
    case 'TooQuiet':
      return {
        headline: 'Too quiet',
        detail: 'Move closer or turn up your input',
        tone: 'warn',
        speech: 'Too quiet',
      };
    case 'TooLoud':
      return {
        headline: 'Too loud',
        detail: 'Turn down your input',
        tone: 'warn',
        speech: 'Too loud',
      };
    case 'NoStablePitch':
    case 'LowConfidence':
      return {
        headline: 'Play a single string',
        detail: null,
        tone: 'idle',
        speech: 'Play a single string',
      };
    case 'Pitch':
      break;
  }
  if (reading.far_from_any && hasTargets) {
    return {
      headline: noteText(reading),
      detail: 'Far from any string: check the tuning setting',
      tone: 'warn',
      speech: 'Far from any string',
    };
  }
  const cents = reading.target?.cents ?? reading.cents ?? 0;
  if (reading.in_tune) {
    return {
      headline: noteText(reading),
      detail: 'In tune',
      tone: 'good',
      speech: `${noteText(reading)}, in tune`,
    };
  }
  const direction =
    reading.direction === 'Tighten' ? 'Tighten' : reading.direction === 'Loosen' ? 'Loosen' : null;
  return {
    headline: noteText(reading),
    detail: direction ? `${direction}: ${centsText(cents)}` : centsText(cents),
    tone: Math.abs(cents) <= 10 ? 'near' : 'idle',
    speech: `${noteText(reading)}, ${centsText(cents)}`,
  };
}

/** The pitch name of a target string: "E2". */
export function targetText(target: TunerTarget): string {
  return `${formatNote(target.pitch.note)}${target.pitch.octave.toString()}`;
}
