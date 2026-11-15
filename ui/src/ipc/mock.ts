import type { Commands } from './types';

/**
 * The mocked command layer for Vitest, Playwright and plain-browser development. It returns
 * canned data only; it must never contain music theory (ADR 0007).
 */
export const mockCommands: Commands = {
  appInfo: () =>
    Promise.resolve({ name: 'Mackey Guitar Tutor', version: '0.0.0-mock', os: 'mock' }),
};
