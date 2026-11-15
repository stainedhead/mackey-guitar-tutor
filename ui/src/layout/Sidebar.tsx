import type { Area } from '../state/store';

const ITEMS: readonly { id: Area; label: string; group: string }[] = [
  { id: 'explore', label: 'Explore', group: 'Learn' },
  { id: 'tuner', label: 'Tuner', group: 'Tools' },
  { id: 'settings', label: 'Settings', group: 'App' },
];

interface Props {
  area: Area;
  onSelect: (area: Area) => void;
  /** The tab-bar form used on iPad portrait. */
  tabs: boolean;
}

/** Navigation: Explore, Tools (Tuner) and Settings. A sidebar, or tabs in iPad portrait. */
export function Sidebar({ area, onSelect, tabs }: Props) {
  return (
    <nav className={tabs ? 'sidebar tabs' : 'sidebar'} aria-label="Main">
      <h1 className="brand">Mackey</h1>
      <ul>
        {ITEMS.map((item) => (
          <li key={item.id}>
            <button
              type="button"
              aria-current={item.id === area ? 'page' : undefined}
              onClick={() => {
                onSelect(item.id);
              }}
            >
              {item.label}
            </button>
          </li>
        ))}
      </ul>
    </nav>
  );
}
