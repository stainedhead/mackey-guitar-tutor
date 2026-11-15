import type { Commands } from '../bindings/commands';
import type { TunerReading } from '../bindings/TunerReading';

export type { Commands } from '../bindings/commands';

/** Stops listening when called. */
export type Unsubscribe = () => void;

/** The one event the UI listens to: tuner readings (about 30 per second). */
export interface TunerEvents {
  listen: (handler: (reading: TunerReading) => void) => Promise<Unsubscribe>;
}

/** The command layer plus the events, as one object tests can replace. */
export interface Ipc {
  commands: Commands;
  tunerEvents: TunerEvents;
}
