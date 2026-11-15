import { invoke } from '@tauri-apps/api/core';
import type { AppError } from '../bindings/AppError';
import type { AppInfo } from '../bindings/AppInfo';
import type { Commands } from './types';

/** Tauri rejects with the serialised `Err` value; anything else becomes a plain message. */
export function toAppError(e: unknown): AppError {
  if (typeof e === 'object' && e !== null && 'message' in e && typeof e.message === 'string') {
    const details = 'details' in e && typeof e.details === 'string' ? e.details : null;
    return { message: e.message, details };
  }
  return { message: 'Something went wrong', details: typeof e === 'string' ? e : null };
}

async function call<T>(name: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(name, args);
  } catch (e) {
    // Commands reject with the plain AppError data Rust serialised (as Tauri itself does), so
    // callers can render `.message` directly; it is not an Error subclass on purpose.
    // eslint-disable-next-line @typescript-eslint/only-throw-error
    throw toAppError(e);
  }
}

/** The real command layer: Tauri `invoke`. Used inside the app only. */
export const tauriCommands: Commands = {
  appInfo: () => call<AppInfo>('app_info'),
};
