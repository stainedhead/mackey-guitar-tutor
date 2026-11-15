import { describe, expect, it } from 'vitest';
import { insideTauri, selectCommands, toAppError } from './index';
import { mockCommands } from './mock';
import { tauriCommands } from './tauri';

describe('selectCommands', () => {
  const cases = [
    { label: 'mock mode outside Tauri', mode: 'mock', inTauri: false, want: mockCommands },
    { label: 'mock mode inside Tauri', mode: 'mock', inTauri: true, want: mockCommands },
    { label: 'plain browser build', mode: 'production', inTauri: false, want: mockCommands },
    { label: 'the app', mode: 'production', inTauri: true, want: tauriCommands },
  ];
  it.each(cases)('$label', ({ mode, inTauri, want }) => {
    expect(selectCommands(mode, inTauri)).toBe(want);
  });
});

describe('insideTauri', () => {
  it('detects the injected internals object', () => {
    expect(insideTauri({})).toBe(false);
    expect(insideTauri({ __TAURI_INTERNALS__: {} })).toBe(true);
  });
});

describe('toAppError', () => {
  it('keeps a serialised AppError', () => {
    expect(toAppError({ message: 'No audio output', details: 'x' })).toEqual({
      message: 'No audio output',
      details: 'x',
    });
    expect(toAppError({ message: 'Hi' })).toEqual({ message: 'Hi', details: null });
  });
  it('wraps anything else in the plain message', () => {
    expect(toAppError('boom')).toEqual({ message: 'Something went wrong', details: 'boom' });
    expect(toAppError(42)).toEqual({ message: 'Something went wrong', details: null });
  });
});

describe('mockCommands', () => {
  it('answers app_info with canned data', async () => {
    const info = await mockCommands.appInfo();
    expect(info.name).toBe('Mackey Guitar Tutor');
  });
});
