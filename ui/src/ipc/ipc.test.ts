import { describe, expect, it } from 'vitest';
import { insideTauri, selectCommands, selectTunerEvents, toAppError } from './index';
import { mockCommands } from '../mocks/mockCommands';
import { mockTunerEvents } from '../mocks/mockTuner';
import { tauriCommands, tauriTunerEvents } from './tauri';

describe('selectCommands', () => {
  const cases = [
    { label: 'mock mode outside Tauri', mode: 'mock', inTauri: false, mock: true },
    { label: 'mock mode inside Tauri', mode: 'mock', inTauri: true, mock: true },
    { label: 'plain browser build', mode: 'production', inTauri: false, mock: true },
    { label: 'the app', mode: 'production', inTauri: true, mock: false },
  ];
  it.each(cases)('$label', ({ mode, inTauri, mock }) => {
    expect(selectCommands(mode, inTauri)).toBe(mock ? mockCommands : tauriCommands);
    expect(selectTunerEvents(mode, inTauri)).toBe(mock ? mockTunerEvents : tauriTunerEvents);
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
    expect(toAppError({ kind: 'NoAudioOutput', message: 'No audio output', details: 'x' })).toEqual(
      {
        kind: 'NoAudioOutput',
        message: 'No audio output',
        details: 'x',
      },
    );
    expect(toAppError({ message: 'Hi' })).toEqual({
      kind: 'Internal',
      message: 'Hi',
      details: null,
    });
  });
  it('wraps anything else in the plain message', () => {
    expect(toAppError('boom')).toEqual({
      kind: 'Internal',
      message: 'Something went wrong',
      details: 'boom',
    });
    expect(toAppError(42).details).toBeNull();
  });
});

describe('mockCommands', () => {
  it('answers app_info', async () => {
    const info = await mockCommands.appInfo();
    expect(info.name).toBe('Mackey Guitar Tutor');
  });
});
