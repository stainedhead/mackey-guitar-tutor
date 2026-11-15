import type { CycleAction } from './bindings/CycleAction';

/** The areas of the app whose shortcuts must not collide (UX-7). */
export type ShortcutArea = 'global' | 'explore';

/** What a shortcut does. */
export type ShortcutAction =
  | { type: 'cycle'; action: CycleAction }
  | { type: 'play' }
  | { type: 'stop' }
  | { type: 'area'; area: 'tuner' | 'settings' }
  | { type: 'sidebar' };

/** One keyboard shortcut. `meta` means Cmd on the Mac (Ctrl elsewhere). */
export interface Shortcut {
  id: string;
  area: ShortcutArea;
  /** `KeyboardEvent.key` in lower case ("arrowdown", "i", "[", ","). */
  key: string;
  shift: boolean;
  meta: boolean;
  /** Human text: "Next chord in the key". */
  label: string;
  /** The key as written in help text: "↓", "Shift-I". */
  display: string;
  action: ShortcutAction;
}

const cycle = (
  id: string,
  key: string,
  shift: boolean,
  action: CycleAction,
  label: string,
  display: string,
): Shortcut => ({
  id,
  area: 'explore',
  key,
  shift,
  meta: false,
  label,
  display,
  action: { type: 'cycle', action },
});

/** The Explore shortcuts of FB-9 and 07 §4, plus the global ones this screen needs. */
export const SHORTCUTS: readonly Shortcut[] = [
  cycle('chord-next', 'arrowdown', false, 'ChordNext', 'Next chord in the key', '↓'),
  cycle('chord-prev', 'arrowup', false, 'ChordPrev', 'Previous chord in the key', '↑'),
  cycle('voicing-next', 'arrowright', false, 'VoicingNext', 'Next voicing', '→'),
  cycle('voicing-prev', 'arrowleft', false, 'VoicingPrev', 'Previous voicing', '←'),
  cycle('inversion-next', 'i', false, 'InversionNext', 'Next inversion', 'I'),
  cycle('inversion-prev', 'i', true, 'InversionPrev', 'Previous inversion', 'Shift-I'),
  cycle('stringset-next', 's', false, 'StringSetNext', 'Next string set', 'S'),
  cycle('stringset-prev', 's', true, 'StringSetPrev', 'Previous string set', 'Shift-S'),
  cycle('position-next', 'p', false, 'PositionNext', 'Next position', 'P'),
  cycle('position-prev', 'p', true, 'PositionPrev', 'Previous position', 'Shift-P'),
  cycle('view', 'v', false, 'CycleView', 'Cycle the view type', 'V'),
  cycle('labels', 'l', false, 'CycleLabels', 'Cycle the labels', 'L'),
  cycle('overlay', 'o', false, 'ToggleOverlay', 'Toggle the scale overlay', 'O'),
  cycle('key-next', ']', false, 'KeyNext', 'Next key around the circle of fifths', ']'),
  cycle('key-prev', '[', false, 'KeyPrev', 'Previous key around the circle of fifths', '['),
  cycle('relative', 'r', false, 'RelativeToggle', 'Relative major or minor', 'R'),
  {
    id: 'play',
    area: 'explore',
    key: ' ',
    shift: false,
    meta: false,
    label: 'Play',
    display: 'Space',
    action: { type: 'play' },
  },
  {
    id: 'stop',
    area: 'explore',
    key: 'escape',
    shift: false,
    meta: false,
    label: 'Stop',
    display: 'Esc',
    action: { type: 'stop' },
  },
  {
    id: 'tuner',
    area: 'global',
    key: 't',
    shift: false,
    meta: true,
    label: 'Tuner',
    display: 'Cmd-T',
    action: { type: 'area', area: 'tuner' },
  },
  {
    id: 'settings',
    area: 'global',
    key: ',',
    shift: false,
    meta: true,
    label: 'Settings',
    display: 'Cmd-,',
    action: { type: 'area', area: 'settings' },
  },
  {
    id: 'sidebar',
    area: 'global',
    key: '0',
    shift: false,
    meta: true,
    label: 'Toggle the sidebar',
    display: 'Cmd-0',
    action: { type: 'sidebar' },
  },
];

/** True when single-key shortcuts must be ignored: a text field has focus (UX-6). */
export function isTypingTarget(target: EventTarget | null): boolean {
  if (!(target instanceof Element)) return false;
  return (
    target.closest('input, textarea, select, [contenteditable=""], [contenteditable="true"]') !==
    null
  );
}

/** True when the focus is on something that Space or Enter already activates (D-28). */
export function isActivatable(target: EventTarget | null): boolean {
  return (
    target instanceof Element &&
    target.closest('button, [role="button"], a[href], summary') !== null
  );
}

/** The pieces of a keyboard event the matcher reads (so tests need no DOM event). */
export interface KeyInput {
  key: string;
  shiftKey: boolean;
  metaKey: boolean;
  ctrlKey: boolean;
  altKey: boolean;
}

/** Finds the shortcut for a key press, or null. Modified presses match only Cmd/Ctrl shortcuts. */
export function matchShortcut(
  event: KeyInput,
  shortcuts: readonly Shortcut[] = SHORTCUTS,
): Shortcut | null {
  if (event.altKey) return null;
  const meta = event.metaKey || event.ctrlKey;
  const key = event.key.toLowerCase();
  return (
    shortcuts.find((s) => {
      if (s.meta !== meta || s.key !== key) return false;
      // Brackets and Space carry no meaningful shift state across keyboard layouts.
      return s.meta || s.shift === event.shiftKey || ['[', ']', ' ', 'escape'].includes(s.key);
    }) ?? null
  );
}

/** Pairs of shortcuts in the same area that share a key combination (UX-7); empty when valid. */
export function findConflicts(shortcuts: readonly Shortcut[]): [string, string][] {
  const seen = new Map<string, string>();
  const conflicts: [string, string][] = [];
  for (const s of shortcuts) {
    const combo = `${s.area}|${s.meta.toString()}|${s.shift.toString()}|${s.key}`;
    const other = seen.get(combo);
    if (other !== undefined) conflicts.push([other, s.id]);
    else seen.set(combo, s.id);
  }
  return conflicts;
}
