import { useMemo } from 'react';
import { commands, tunerEvents } from './ipc';
import { AppShell } from './layout/AppShell';
import { StoreProvider } from './state/context';
import { UiStore } from './state/store';

/** The app: one store over the command layer, and the shell. */
export function App({ store }: { store?: UiStore }) {
  const owned = useMemo(() => store ?? new UiStore(commands, tunerEvents), [store]);
  return (
    <StoreProvider store={owned}>
      <AppShell />
    </StoreProvider>
  );
}
