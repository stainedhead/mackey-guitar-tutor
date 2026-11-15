import type { ArpeggioDirection } from '../bindings/ArpeggioDirection';
import type { ExplorerPanel } from '../bindings/ExplorerPanel';
import type { ExplorerOption } from '../bindings/ExplorerOption';
import type { FretboardView } from '../bindings/FretboardView';
import type { Inversion } from '../bindings/Inversion';
import type { OverlayRequest } from '../bindings/OverlayRequest';
import type { StringSet } from '../bindings/StringSet';
import type { CycleAction } from '../bindings/CycleAction';

const INVERSION_TEXT: Record<Inversion, string> = {
  Root: 'Root position',
  First: '1st inversion',
  Second: '2nd inversion',
  Third: '3rd inversion',
};

/** "S234" is "strings 2 3 4". */
export function setLabel(set: StringSet): string {
  return set.slice(1).split('').join(' ');
}

interface Props {
  panel: ExplorerPanel;
  view: FretboardView;
  onCycle: (action: CycleAction) => void;
  onOption: (option: ExplorerOption) => void;
  onOverlay: (overlay: OverlayRequest) => void;
}

function Stepper({
  label,
  onPrev,
  onNext,
  children,
}: {
  label: string;
  onPrev: () => void;
  onNext: () => void;
  children?: React.ReactNode;
}) {
  return (
    <div className="stepper" role="group" aria-label={label}>
      <button type="button" aria-label={`Previous ${label.toLowerCase()}`} onClick={onPrev}>
        ◀
      </button>
      <span>{children}</span>
      <button type="button" aria-label={`Next ${label.toLowerCase()}`} onClick={onNext}>
        ▶
      </button>
    </div>
  );
}

/** Controls that belong to the current view: voicing steps, triads, arpeggios, capo names, overlay. */
export function ViewControls({ panel, view, onCycle, onOption, onOverlay }: Props) {
  const sel = panel.selection;
  const counter = view.list_position?.label ?? null;
  const noVoicing = view.no_voicing;
  const overlayScale = panel.scale_choices.find((c) => c.selected);
  return (
    <section className="view-controls" aria-label="View controls">
      {view.headline !== '' && (
        <p className="headline" data-testid="headline">
          {view.headline}
        </p>
      )}
      {view.chord_label !== null && (
        <p className="capo-label" data-testid="capo-label">
          {view.chord_label.text}
          <button
            type="button"
            className="link"
            onClick={() => {
              onOption({
                CapoNames: sel.capo_names === 'ShapeFirst' ? 'SoundingFirst' : 'ShapeFirst',
              });
            }}
          >
            Swap names
          </button>
        </p>
      )}

      {sel.view === 'Voicing' && (
        <Stepper
          label="Voicing"
          onPrev={() => {
            onCycle('VoicingPrev');
          }}
          onNext={() => {
            onCycle('VoicingNext');
          }}
        >
          <span data-testid="list-position">{counter ?? '–'}</span>
        </Stepper>
      )}

      {sel.view === 'Triad' && (
        <div className="triad-controls">
          <Stepper
            label="String set"
            onPrev={() => {
              onCycle('StringSetPrev');
            }}
            onNext={() => {
              onCycle('StringSetNext');
            }}
          >
            Strings {setLabel(sel.string_set)}
          </Stepper>
          <Stepper
            label="Inversion"
            onPrev={() => {
              onCycle('InversionPrev');
            }}
            onNext={() => {
              onCycle('InversionNext');
            }}
          >
            {INVERSION_TEXT[sel.inversion]}
          </Stepper>
          <label className="check">
            <input
              type="checkbox"
              checked={sel.show_all_on_set}
              onChange={(e) => {
                onOption({ ShowAllOnSet: e.target.checked });
              }}
            />
            Show all on string set
          </label>
          <Stepper
            label="Position"
            onPrev={() => {
              onCycle('PositionPrev');
            }}
            onNext={() => {
              onCycle('PositionNext');
            }}
          >
            <span data-testid="list-position">{counter ?? '–'}</span>
          </Stepper>
          {view.triad_note !== null && <p className="triad-note">{view.triad_note}</p>}
        </div>
      )}

      {sel.view === 'Arpeggio' && (
        <div className="arpeggio-controls">
          <Stepper
            label="Position"
            onPrev={() => {
              onCycle('PositionPrev');
            }}
            onNext={() => {
              onCycle('PositionNext');
            }}
          >
            <span data-testid="list-position">{counter ?? '–'}</span>
          </Stepper>
          <label>
            Direction
            <select
              value={sel.arpeggio_direction}
              onChange={(e) => {
                onOption({ ArpeggioDirection: e.target.value as ArpeggioDirection });
              }}
            >
              <option value="Up">Up</option>
              <option value="Down">Down</option>
              <option value="UpDown">Up and down</option>
            </select>
          </label>
          <label className="check">
            <input
              type="checkbox"
              checked={sel.show_order}
              onChange={(e) => {
                onOption({ ShowOrder: e.target.checked });
              }}
            />
            Show order
          </label>
        </div>
      )}

      <div className="overlay-controls">
        <label className="check">
          <input
            type="checkbox"
            checked={sel.overlay.on}
            onChange={(e) => {
              onOverlay({
                on: e.target.checked,
                scale: sel.overlay.scale,
                follow: sel.overlay.follow_chords,
              });
            }}
          />
          Scale overlay
        </label>
        <label>
          Scale
          <select
            value={sel.overlay.scale ? panel.scale_choices.findIndex((c) => c.selected) : -1}
            onChange={(e) => {
              const index = Number(e.target.value);
              const scale = panel.scale_choices[index]?.scale ?? null;
              onOverlay({ on: true, scale, follow: false });
            }}
          >
            <option value={-1}>The key&apos;s own scale</option>
            {panel.scale_choices.map((c, i) => (
              <option key={c.label} value={i}>
                {c.label}
              </option>
            ))}
          </select>
        </label>
        <label className="check">
          <input
            type="checkbox"
            checked={sel.overlay.follow_chords}
            onChange={(e) => {
              onOverlay({ on: true, scale: null, follow: e.target.checked });
            }}
          />
          Follow the chords
        </label>
        {sel.overlay.on && sel.view !== 'Triad' && sel.view !== 'Arpeggio' && (
          <Stepper
            label="Pattern"
            onPrev={() => {
              onCycle('PositionPrev');
            }}
            onNext={() => {
              onCycle('PositionNext');
            }}
          >
            {counter ?? overlayScale?.label ?? 'All positions'}
          </Stepper>
        )}
      </div>

      {noVoicing?.kind === 'NoneFound' && (
        <div className="no-voicing" role="status">
          <p>No playable voicing in this tuning</p>
          {noVoicing.value.options.map((opt) =>
            opt === 'AllowStretches' ? (
              <button
                key={opt}
                type="button"
                onClick={() => {
                  onOption({ AllowStretch: true });
                }}
              >
                Allow stretches
              </button>
            ) : (
              <button
                key={opt}
                type="button"
                onClick={() => {
                  onOption({ AllowOmittedFifth: true });
                }}
              >
                Allow omitted fifth
              </button>
            ),
          )}
        </div>
      )}
    </section>
  );
}
