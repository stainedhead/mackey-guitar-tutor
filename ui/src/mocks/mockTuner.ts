import type { TunerReading } from '../bindings/TunerReading';
import type { TunerEvents, Unsubscribe } from '../ipc/types';

const listeners = new Set<(reading: TunerReading) => void>();

/** The mock event source: readings reach whoever listens, when a test or script emits them. */
export const mockTunerEvents: TunerEvents = {
  listen: (handler): Promise<Unsubscribe> => {
    listeners.add(handler);
    return Promise.resolve(() => {
      listeners.delete(handler);
    });
  },
};

/** Delivers a reading to every listener (what Rust's `tuner_reading` event does). */
export function emitMockTunerReading(reading: TunerReading): void {
  for (const listener of [...listeners]) listener(reading);
}

/** How many components are listening (tests check that unmounting unsubscribes). */
export function mockTunerListenerCount(): number {
  return listeners.size;
}

const QUIET = { rms: 0.001, peak: 0.002, clipping: false };
const NORMAL = { rms: 0.1, peak: 0.3, clipping: false };

/** A reading with sensible empty values; override what the scene needs. */
export function mockReading(partial: Partial<TunerReading> = {}): TunerReading {
  return {
    status: 'Listening',
    note: null,
    octave: null,
    cents: null,
    frequency_hz: null,
    confidence: 0,
    level: QUIET,
    in_tune: false,
    direction: null,
    target: null,
    far_from_any: false,
    ticks: [],
    notice: null,
    t_ms: 0,
    ...partial,
  };
}

const A = { letter: 'A', accidental: 0 } as const;

/**
 * A scripted series for Vitest and demos: listening, a flat A, nearly there, in tune, too
 * quiet, then no stable pitch. These are readings the Rust tuner would emit; the mock
 * decides nothing about pitch.
 */
export const TUNER_SCRIPT: readonly TunerReading[] = [
  mockReading({ t_ms: 0 }),
  mockReading({
    status: 'Pitch',
    note: A,
    octave: 2,
    cents: -18,
    frequency_hz: 108.9,
    confidence: 0.95,
    level: NORMAL,
    direction: 'Tighten',
    target: { string: 5, cents: -18 },
    t_ms: 100,
  }),
  mockReading({
    status: 'Pitch',
    note: A,
    octave: 2,
    cents: -4,
    frequency_hz: 109.7,
    confidence: 0.96,
    level: NORMAL,
    direction: 'Tighten',
    target: { string: 5, cents: -4 },
    t_ms: 200,
  }),
  mockReading({
    status: 'Pitch',
    note: A,
    octave: 2,
    cents: 0.5,
    frequency_hz: 110.03,
    confidence: 0.97,
    level: NORMAL,
    in_tune: true,
    direction: 'InTune',
    target: { string: 5, cents: 0.5 },
    ticks: [5],
    t_ms: 600,
  }),
  mockReading({ status: 'TooQuiet', t_ms: 4000 }),
  mockReading({ status: 'NoStablePitch', level: NORMAL, t_ms: 4600 }),
];

/** Lets a Playwright page push readings: `window.__mackeyMock.emitTuner(reading)`. */
export function installMockHooks(win: Window): void {
  (win as Window & { __mackeyMock?: unknown }).__mackeyMock = {
    emitTuner: emitMockTunerReading,
    tunerScript: TUNER_SCRIPT,
    listenerCount: mockTunerListenerCount,
  };
}
