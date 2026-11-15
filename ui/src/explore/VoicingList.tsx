import type { FretboardView } from '../bindings/FretboardView';
import type { VoicingGroup } from '../bindings/VoicingGroup';
import type { VoicingListEntry } from '../bindings/VoicingListEntry';
import { MiniDiagram } from './MiniDiagram';

const GROUPS: readonly { id: VoicingGroup; title: string }[] = [
  { id: 'Open', title: 'Open' },
  { id: 'BarreAndCaged', title: 'Barre and CAGED' },
  { id: 'Other', title: 'Other' },
];

interface Props {
  view: FretboardView;
  onPick: (index: number) => void;
}

/** The voicing list grouped Open, Barre and CAGED, Other, with mini diagrams (FB-5.2). */
export function VoicingList({ view, onPick }: Props) {
  const entries = view.voicing_list;
  if (entries.length === 0) return null;
  const indexed: { entry: VoicingListEntry; index: number }[] = entries.map((entry, index) => ({
    entry,
    index,
  }));
  return (
    <section className="voicing-list" aria-label="Voicings">
      {GROUPS.map((group) => {
        const items = indexed.filter((i) => i.entry.group === group.id);
        if (items.length === 0) return null;
        return (
          <div key={group.id} className="voicing-group">
            <h3>{group.title}</h3>
            <ul>
              {items.map(({ entry, index }) => (
                <li key={`${group.id}-${index.toString()}`}>
                  <button
                    type="button"
                    className={entry.selected ? 'voicing-item selected' : 'voicing-item'}
                    aria-pressed={entry.selected}
                    onClick={() => {
                      onPick(index);
                    }}
                  >
                    <MiniDiagram entry={entry} />
                    <span className="voicing-name">{entry.name}</span>
                    {entry.found_by_search && <span className="voicing-tag">Found by search</span>}
                    {entry.capo_label && (
                      <span className="voicing-capo">{entry.capo_label.text}</span>
                    )}
                  </button>
                </li>
              ))}
            </ul>
          </div>
        );
      })}
    </section>
  );
}
