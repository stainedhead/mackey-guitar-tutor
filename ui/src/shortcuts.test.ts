import { describe, expect, it } from 'vitest';
import {
  SHORTCUTS,
  findConflicts,
  isActivatable,
  isTypingTarget,
  matchShortcut,
  type KeyInput,
} from './shortcuts';

const press = (key: string, extra: Partial<KeyInput> = {}): KeyInput => ({
  key,
  shiftKey: false,
  metaKey: false,
  ctrlKey: false,
  altKey: false,
  ...extra,
});

describe('shortcut table', () => {
  it('no two commands share a key (AC-UX-7)', () => {
    expect(findConflicts(SHORTCUTS)).toEqual([]);
  });
  it('detects a conflict when one exists', () => {
    const first = SHORTCUTS[0];
    if (!first) throw new Error('no shortcuts');
    const dup = [...SHORTCUTS, { ...first, id: 'dup' }];
    expect(findConflicts(dup)).toEqual([[first.id, 'dup']]);
  });
  it('every shortcut has a label and a display', () => {
    for (const s of SHORTCUTS) {
      expect(s.label, s.id).not.toBe('');
      expect(s.display, s.id).not.toBe('');
    }
  });
});

describe('matchShortcut', () => {
  it('matches the arrow keys and ignores unknown keys', () => {
    expect(matchShortcut(press('ArrowDown'))?.id).toBe('chord-next');
    expect(matchShortcut(press('q'))).toBeNull();
  });
  it('ignores Alt chords', () => {
    expect(matchShortcut(press('ArrowDown', { altKey: true }))).toBeNull();
  });
  it('matches Cmd shortcuts only with Cmd or Ctrl', () => {
    expect(matchShortcut(press(','))).toBeNull();
    expect(matchShortcut(press(',', { metaKey: true }))?.id).toBe('settings');
    expect(matchShortcut(press(',', { ctrlKey: true }))?.id).toBe('settings');
  });
});

describe('typing targets (UX-6)', () => {
  it('treats text fields and selects as typing', () => {
    for (const tag of ['input', 'textarea', 'select']) {
      expect(isTypingTarget(document.createElement(tag)), tag).toBe(true);
    }
    expect(isTypingTarget(document.createElement('div'))).toBe(false);
    expect(isTypingTarget(null)).toBe(false);
  });
  it('knows which elements Space already activates', () => {
    expect(isActivatable(document.createElement('button'))).toBe(true);
    expect(isActivatable(document.createElement('div'))).toBe(false);
  });
});
