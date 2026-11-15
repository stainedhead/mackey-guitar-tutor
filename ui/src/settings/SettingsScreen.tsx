import type { LabelsMode } from '../bindings/LabelsMode';
import type { SettingsGroup } from '../bindings/SettingsGroup';
import type { Theme } from '../bindings/Theme';
import type { TuningChoice } from '../bindings/TuningChoice';
import { useStore } from '../state/context';
import { useUiState } from '../state/store';

function Group({
  title,
  group,
  onReset,
  children,
}: {
  title: string;
  group: SettingsGroup;
  onReset: (g: SettingsGroup) => void;
  children: React.ReactNode;
}) {
  return (
    <fieldset className="settings-group">
      <legend>{title}</legend>
      {children}
      <button
        type="button"
        onClick={() => {
          onReset(group);
        }}
      >
        Reset {title}
      </button>
    </fieldset>
  );
}

/** The Settings screen (FR-028): applied immediately, with a reset button per group. */
export function SettingsScreen() {
  const store = useStore();
  const settings = useUiState(store, (s) => s.settings);
  const snapshot = useUiState(store, (s) => s.snapshot);
  if (!settings) return <p className="loading">Loading</p>;
  const presets = (snapshot?.panel.instrument.tuning_choices ?? []).filter(
    (t) => 'Preset' in t.choice,
  );
  const instrument = settings.instrument;
  const display = settings.display;
  const reset = (g: SettingsGroup): void => void store.resetSettings(g);
  return (
    <div className="settings" data-testid="settings">
      <h2>Settings</h2>
      <Group title="Instrument" group="Instrument" onReset={reset}>
        <label className="check">
          <input
            type="checkbox"
            checked={instrument.handedness.left_handed}
            onChange={(e) => {
              void store.updateSettings({
                instrument: {
                  ...instrument,
                  handedness: { ...instrument.handedness, left_handed: e.target.checked },
                },
              });
            }}
          />
          Left-handed
        </label>
        <label className="check">
          <input
            type="checkbox"
            checked={instrument.handedness.flip_strings}
            onChange={(e) => {
              void store.updateSettings({
                instrument: {
                  ...instrument,
                  handedness: { ...instrument.handedness, flip_strings: e.target.checked },
                },
              });
            }}
          />
          Flip strings (string 6 at the top)
        </label>
        <label>
          Frets
          <select
            value={instrument.frets}
            onChange={(e) => {
              void store.updateSettings({
                instrument: { ...instrument, frets: Number(e.target.value) },
              });
            }}
          >
            {[19, 20, 21, 22, 23, 24].map((n) => (
              <option key={n} value={n}>
                {n}
              </option>
            ))}
          </select>
        </label>
        <label>
          Default tuning
          <select
            value={JSON.stringify(instrument.default_tuning)}
            onChange={(e) => {
              void store.updateSettings({
                instrument: {
                  ...instrument,
                  default_tuning: JSON.parse(e.target.value) as TuningChoice,
                },
              });
            }}
          >
            {presets.map((t) => (
              <option key={t.label} value={JSON.stringify(t.choice)}>
                {t.label}
              </option>
            ))}
          </select>
        </label>
        <label className="check">
          <input
            type="checkbox"
            checked={instrument.allow_stretches}
            onChange={(e) => {
              void store.updateSettings({
                instrument: { ...instrument, allow_stretches: e.target.checked },
              });
            }}
          />
          Allow stretches (5-fret span)
        </label>
      </Group>
      <Group title="Display" group="Display" onReset={reset}>
        <label>
          Theme
          <select
            value={display.theme}
            onChange={(e) => {
              void store.updateSettings({
                display: { ...display, theme: e.target.value as Theme },
              });
            }}
          >
            <option value="System">Follow the system</option>
            <option value="Light">Light</option>
            <option value="Dark">Dark</option>
          </select>
        </label>
        <label>
          Labels
          <select
            value={display.labels_default}
            onChange={(e) => {
              void store.updateSettings({
                display: { ...display, labels_default: e.target.value as LabelsMode },
              });
            }}
          >
            <option value="NoteName">Note names</option>
            <option value="ScaleDegree">Scale degrees</option>
            <option value="None">No labels</option>
          </select>
        </label>
        <label className="check">
          <input
            type="checkbox"
            checked={display.show_fret_numbers}
            onChange={(e) => {
              void store.updateSettings({
                display: { ...display, show_fret_numbers: e.target.checked },
              });
            }}
          />
          Show fret numbers
        </label>
      </Group>
      <Group title="Sound" group="Sound" onReset={reset}>
        <label>
          Volume
          <input
            type="range"
            min={0}
            max={100}
            value={settings.sound.volume}
            aria-valuetext={`${settings.sound.volume.toString()} percent`}
            onChange={(e) => {
              void store.updateSettings({
                sound: { ...settings.sound, volume: Number(e.target.value) },
              });
            }}
          />
        </label>
        <label className="check">
          <input
            type="checkbox"
            checked={settings.sound.mute}
            onChange={(e) => {
              void store.updateSettings({ sound: { ...settings.sound, mute: e.target.checked } });
            }}
          />
          Mute
        </label>
      </Group>
      <Group title="Tuner" group="Tuner" onReset={reset}>
        <label>
          A4 reference (Hz)
          <input
            type="number"
            min={415}
            max={466}
            step={1}
            value={settings.tuner.reference_a4}
            onChange={(e) => {
              const hz = Number(e.target.value);
              if (hz >= 415 && hz <= 466)
                void store.updateSettings({ tuner: { ...settings.tuner, reference_a4: hz } });
            }}
          />
        </label>
      </Group>
    </div>
  );
}
