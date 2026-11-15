import { useEffect, useState } from 'react';
import type { AppInfo } from './bindings/AppInfo';
import { commands, toAppError } from './ipc';
import { NeckPlaceholder } from './NeckPlaceholder';

type Screen = 'explore' | 'tuner';

const SCREENS: readonly { id: Screen; label: string }[] = [
  { id: 'explore', label: 'Explore' },
  { id: 'tuner', label: 'Tuner' },
];

/** Placeholder shell: sidebar, a screen heading and the app info from Rust. */
export function App() {
  const [screen, setScreen] = useState<Screen>('explore');
  const [info, setInfo] = useState<AppInfo | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    let cancelled = false;
    commands
      .appInfo()
      .then((value) => {
        if (!cancelled) setInfo(value);
      })
      .catch((e: unknown) => {
        if (!cancelled) setError(toAppError(e).message);
      });
    return () => {
      cancelled = true;
    };
  }, []);

  const current = SCREENS.find((s) => s.id === screen);

  return (
    <div className="app">
      <nav className="sidebar" aria-label="Main">
        <h1 className="brand">Mackey</h1>
        <ul>
          {SCREENS.map((s) => (
            <li key={s.id}>
              <button
                type="button"
                aria-current={s.id === screen ? 'page' : undefined}
                onClick={() => {
                  setScreen(s.id);
                }}
              >
                {s.label}
              </button>
            </li>
          ))}
        </ul>
      </nav>
      <main className="content">
        <h2>{current?.label}</h2>
        {screen === 'explore' ? (
          <NeckPlaceholder />
        ) : (
          <p>The tuner arrives with the audio phase.</p>
        )}
        <p className="app-info" data-testid="app-info" role="status">
          {error ?? (info ? `${info.name} ${info.version} (${info.os})` : 'Loading')}
        </p>
      </main>
    </div>
  );
}
