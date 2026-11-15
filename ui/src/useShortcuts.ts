import { useEffect } from 'react';
import { isActivatable, isTypingTarget, matchShortcut, type ShortcutAction } from './shortcuts';

/**
 * Listens for the keyboard shortcuts on the window. Single keys are ignored while a text field
 * has focus (UX-6); Space leaves buttons to their own activation (D-28); a key the neck already
 * handled (`defaultPrevented`) is left alone.
 */
export function useShortcuts(
  run: (action: ShortcutAction) => void,
  enabled: (action: ShortcutAction) => boolean,
): void {
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent): void => {
      if (event.defaultPrevented || event.isComposing) return;
      const shortcut = matchShortcut(event);
      if (!shortcut) return;
      if (!shortcut.meta && isTypingTarget(event.target)) return;
      if (shortcut.key === ' ' && isActivatable(event.target)) return;
      if (!enabled(shortcut.action)) return;
      event.preventDefault();
      run(shortcut.action);
    };
    window.addEventListener('keydown', onKeyDown);
    return () => {
      window.removeEventListener('keydown', onKeyDown);
    };
  }, [run, enabled]);
}
