import { mockCommands } from './mock';
import { tauriCommands } from './tauri';
import type { Commands } from './types';

export type { Commands } from './types';
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

/** The command layer the UI uses. The UI calls Rust only through this object. */
export const commands: Commands = selectCommands(import.meta.env.MODE, insideTauri());
