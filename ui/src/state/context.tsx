import { createContext, useContext, type ReactNode } from 'react';
import type { UiStore } from './store';

const StoreContext = createContext<UiStore | null>(null);

/** Gives components the store for this window. */
export function StoreProvider({ store, children }: { store: UiStore; children: ReactNode }) {
  return <StoreContext.Provider value={store}>{children}</StoreContext.Provider>;
}

/** The store; components call its actions and select what they show with `useUiState`. */
export function useStore(): UiStore {
  const store = useContext(StoreContext);
  if (!store) throw new Error('useStore needs a StoreProvider');
  return store;
}
