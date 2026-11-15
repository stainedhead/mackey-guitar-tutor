import { mockCommands } from '../mocks/mockCommands';
import { mockTunerEvents } from '../mocks/mockTuner';
import { tauriCommands, tauriTunerEvents } from './tauri';
import type { Commands, TunerEvents } from './types';

export type { Commands, Ipc, TunerEvents, Unsubscribe } from './types';
export { toAppError } from './tauri';

/** True when running inside the Tauri web view (Tauri injects `__TAURI_INTERNALS__`). */
export function insideTauri(win: object = window): boolean {
  return '__TAURI_INTERNALS__' in win;
}

/**
 * Picks the command layer: the mock in `vite --mode mock`, in tests and in any plain
 * browser; real Tauri `invoke` calls inside the app.
 */
export function selectCommands(mode: string, inTauri: boolean): Commands {
  return mode === 'mock' || !inTauri ? mockCommands : tauriCommands;
}

/** The matching event source. */
export function selectTunerEvents(mode: string, inTauri: boolean): TunerEvents {
  return mode === 'mock' || !inTauri ? mockTunerEvents : tauriTunerEvents;
}

/** The command layer the UI uses. The UI calls Rust only through this object. */
export const commands: Commands = selectCommands(import.meta.env.MODE, insideTauri());
export const tunerEvents: TunerEvents = selectTunerEvents(import.meta.env.MODE, insideTauri());
