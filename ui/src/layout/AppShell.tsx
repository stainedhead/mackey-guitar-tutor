import { useCallback, useEffect, useMemo } from 'react';
import { LiveRegion } from '../a11y/LiveRegion';
import { ExploreScreen } from '../explore/ExploreScreen';
import { SettingsScreen } from '../settings/SettingsScreen';
import { useShortcuts } from '../useShortcuts';
import { TunerScreen } from '../tuner/TunerScreen';
import { useStore } from '../state/context';
import { useUiState } from '../state/store';
import type { ShortcutAction } from '../shortcuts';
import { Sidebar } from './Sidebar';
import { useLayout } from './useLayout';

/** The window: sidebar (or tabs), the current screen, the notice line and the live region. */
export function AppShell() {
  const store = useStore();
  const layout = useLayout();
  const area = useUiState(store, (s) => s.area);
  const sidebarOpen = useUiState(store, (s) => s.sidebarOpen);
  const notice = useUiState(store, (s) => s.notice);
  const live = useUiState(store, (s) => s.liveMessage);
  const theme = useUiState(store, (s) => s.settings?.display.theme ?? 'System');

  useEffect(() => {
    store.init().catch(() => undefined);
  }, [store]);

  // Theme: "System" follows prefers-color-scheme through CSS; Light and Dark force it.
  useEffect(() => {
    const root = document.documentElement;
    if (theme === 'System') root.removeAttribute('data-theme');
    else root.setAttribute('data-theme', theme.toLowerCase());
  }, [theme]);

  const run = useCallback(
    (action: ShortcutAction) => {
      switch (action.type) {
        case 'cycle':
          void store.cycle(action.action);
          break;
        case 'play':
          void store.play();
          break;
        case 'stop':
          void store.commands.stopPlayback();
          break;
        case 'area':
          store.setArea(action.area);
          break;
        case 'sidebar':
          store.toggleSidebar();
          break;
      }
    },
    [store],
  );
  const enabled = useCallback(
    (action: ShortcutAction) =>
      action.type === 'area' || action.type === 'sidebar' || area === 'explore',
    [area],
  );
  useShortcuts(run, enabled);

  const narrow = layout.kind === 'narrow';
  const tabs = layout.kind === 'ipad-portrait';
  const showSidebar = tabs || (!narrow && sidebarOpen) || (narrow && sidebarOpen && false);
  const menu = useMemo(() => narrow, [narrow]);
  return (
    <div className={`app layout-${layout.kind}`} data-layout={layout.kind}>
      {showSidebar && (
        <Sidebar
          area={area}
          onSelect={(a) => {
            store.setArea(a);
          }}
          tabs={tabs}
        />
      )}
      <div className="main-col">
        {menu && (
          <nav className="narrow-nav" aria-label="Main">
            {(['explore', 'tuner', 'settings'] as const).map((a) => (
              <button
                key={a}
                type="button"
                aria-current={a === area ? 'page' : undefined}
                onClick={() => {
                  store.setArea(a);
                }}
              >
                {a === 'explore' ? 'Explore' : a === 'tuner' ? 'Tuner' : 'Settings'}
              </button>
            ))}
          </nav>
        )}
        {notice !== null && (
          <p className="notice" data-testid="notice" role="status">
            {notice}
            <button
              type="button"
              className="link"
              aria-label="Dismiss"
              onClick={() => {
                store.setNotice(null);
              }}
            >
              ✕
            </button>
          </p>
        )}
        <main className="content">
          {area === 'explore' && <ExploreScreen layout={layout} />}
          {area === 'tuner' && <TunerScreen />}
          {area === 'settings' && <SettingsScreen />}
        </main>
      </div>
      <LiveRegion message={live} />
    </div>
  );
}
