import { useState } from 'react';
import type { ExplorerPanel } from '../bindings/ExplorerPanel';
import type { LabelsMode } from '../bindings/LabelsMode';
import type { Mode } from '../bindings/Mode';
import type { Note } from '../bindings/Note';
import type { TuningChoice } from '../bindings/TuningChoice';
import type { ViewType } from '../bindings/ViewType';

const LABEL_CHOICES: readonly { id: LabelsMode; label: string }[] = [
  { id: 'NoteName', label: 'Note names' },
  { id: 'ScaleDegree', label: 'Scale degrees' },
  { id: 'Interval', label: 'Intervals' },
  { id: 'Finger', label: 'Fingers' },
  { id: 'None', label: 'No labels' },
];

const VIEW_CHOICES: readonly { id: ViewType; label: string }[] = [
  { id: 'AllTones', label: 'All tones' },
  { id: 'Voicing', label: 'Voicing' },
  { id: 'Triad', label: 'Triad' },
  { id: 'Arpeggio', label: 'Arpeggio' },
];

interface Props {
  panel: ExplorerPanel;
  hasChord: boolean;
  onKey: (tonic: Note, mode: Mode) => void;
  onTuning: (tuning: TuningChoice) => void;
  onCapo: (capo: number) => void;
  onLabels: (labels: LabelsMode) => void;
  onView: (view: ViewType) => void;
  onKeyStep: (direction: 'next' | 'prev') => void;
  onRelative: () => void;
}

const noteId = (n: Note): string => `${n.letter}:${n.accidental.toString()}`;

/** Key picker, tuning and capo, labels and view type (FB-2, LH-2, LH-3, G-2.2). */
export function Toolbar({
  panel,
  hasChord,
  onKey,
  onTuning,
  onCapo,
  onLabels,
  onView,
  onKeyStep,
  onRelative,
}: Props) {
  const [customOpen, setCustomOpen] = useState(false);
  const [custom, setCustom] = useState<string[]>(['E2', 'A2', 'D3', 'G3', 'B3', 'E4']);
  const selectedMode = panel.key.modes.find((m) => m.selected);
  const selectedTonic = panel.key.tonic_choices.find((t) => t.selected);
  const tuning = panel.instrument.tuning_choices.find((t) => t.selected);
  return (
    <div className="toolbar" role="toolbar" aria-label="Explore settings">
      <div className="tool-group key-picker">
        <strong className="key-label" data-testid="key-label" aria-live="off">
          {panel.key.label}
        </strong>
        <label>
          Tonic
          <select
            value={selectedTonic ? noteId(selectedTonic.note) : ''}
            onChange={(e) => {
              const pick = panel.key.tonic_choices.find((t) => noteId(t.note) === e.target.value);
              if (pick && selectedMode) onKey(pick.note, selectedMode.mode);
            }}
          >
            {panel.key.tonic_choices.map((t) => (
              <option key={noteId(t.note)} value={noteId(t.note)}>
                {t.label}
              </option>
            ))}
          </select>
        </label>
        <label>
          Mode
          <select
            value={selectedMode?.mode ?? 'Major'}
            onChange={(e) => {
              const pick = panel.key.modes.find((m) => m.mode === e.target.value);
              if (pick && selectedTonic) onKey(selectedTonic.note, pick.mode);
            }}
          >
            {panel.key.modes.map((m) => (
              <option key={m.mode} value={m.mode}>
                {m.label}
              </option>
            ))}
          </select>
        </label>
        <button
          type="button"
          aria-label="Previous key around the circle of fifths"
          onClick={() => {
            onKeyStep('prev');
          }}
        >
          ‹
        </button>
        <button
          type="button"
          aria-label="Next key around the circle of fifths"
          onClick={() => {
            onKeyStep('next');
          }}
        >
          ›
        </button>
        <button type="button" onClick={onRelative}>
          Relative
        </button>
        {panel.key.enharmonic_hint !== null && (
          <span className="hint">{panel.key.enharmonic_hint}</span>
        )}
      </div>

      <div className="tool-group">
        <label>
          Tuning
          <select
            value={tuning ? JSON.stringify(tuning.choice) : ''}
            onChange={(e) => {
              if (e.target.value === 'custom') {
                setCustomOpen(true);
                return;
              }
              setCustomOpen(false);
              onTuning(JSON.parse(e.target.value) as TuningChoice);
            }}
          >
            {panel.instrument.tuning_choices.map((t) => (
              <option key={t.label} value={JSON.stringify(t.choice)}>
                {t.label} ({t.pitches})
              </option>
            ))}
            <option value="custom">Custom…</option>
          </select>
        </label>
        <label>
          Capo
          <select
            value={panel.instrument.capo}
            onChange={(e) => {
              onCapo(Number(e.target.value));
            }}
          >
            {Array.from({ length: 13 }, (_, n) => (
              <option key={n} value={n}>
                {n === 0 ? 'None' : `Fret ${n.toString()}`}
              </option>
            ))}
          </select>
        </label>
      </div>

      <div className="tool-group">
        <label>
          Labels
          <select
            value={panel.selection.labels}
            onChange={(e) => {
              onLabels(e.target.value as LabelsMode);
            }}
          >
            {LABEL_CHOICES.filter((l) => l.id !== 'Interval' || hasChord).map((l) => (
              <option key={l.id} value={l.id}>
                {l.label}
              </option>
            ))}
          </select>
        </label>
        <div role="radiogroup" aria-label="View type" className="segmented">
          {VIEW_CHOICES.map((v) => (
            <button
              key={v.id}
              type="button"
              role="radio"
              aria-checked={panel.selection.view === v.id}
              disabled={v.id !== 'AllTones' && !hasChord}
              onClick={() => {
                onView(v.id);
              }}
            >
              {v.label}
            </button>
          ))}
        </div>
      </div>

      {panel.instrument.warnings.length > 0 && (
        <ul className="warnings" role="status">
          {panel.instrument.warnings.map((w) => (
            <li key={w}>
              <span aria-hidden="true">⚠ </span>
              {w}
            </li>
          ))}
        </ul>
      )}

      {customOpen && (
        <form
          className="custom-tuning"
          onSubmit={(e) => {
            e.preventDefault();
            onTuning({ Custom: custom });
          }}
        >
          <fieldset>
            <legend>Custom tuning, low string to high</legend>
            {custom.map((value, i) => (
              <label key={i}>
                String {6 - i}
                <input
                  type="text"
                  value={value}
                  size={4}
                  onChange={(e) => {
                    setCustom(custom.map((c, j) => (j === i ? e.target.value : c)));
                  }}
                />
              </label>
            ))}
            <button type="submit">Use this tuning</button>
          </fieldset>
        </form>
      )}
    </div>
  );
}
