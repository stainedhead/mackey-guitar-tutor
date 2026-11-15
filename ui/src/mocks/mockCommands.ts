import type { AppError } from '../bindings/AppError';
import type { ExplorerSnapshot } from '../bindings/ExplorerSnapshot';
import type { ParseOutcome } from '../bindings/ParseOutcome';
import type { Settings } from '../bindings/Settings';
import type { SettingsPatch } from '../bindings/SettingsPatch';
import type { TunerTarget } from '../bindings/TunerTarget';
import type { Commands } from '../bindings/commands';
import scenariosText from './scenarios.json?raw';

/** One state of the app, recorded from the real Rust core. */
export interface ScenarioNode {
  snapshot: ExplorerSnapshot;
  settings: Settings;
}

/**
 * The recorded scenarios: a graph of app states. `edges[from][command key]` is the state a
 * command leads to. Every snapshot in it was produced by the real core, so the mocked UI shows
 * real voicings; the mock itself contains no music logic (ADR 0007).
 */
export interface Scenarios {
  start: string;
  nodes: Record<string, ScenarioNode>;
  edges: Record<string, Record<string, string>>;
  parse: Record<string, ParseOutcome>;
  tuner_targets: TunerTarget[];
}

/** Parses the text of `scenarios.json`. */
export function parseScenarios(text: string): Scenarios {
  return JSON.parse(text) as Scenarios;
}

/** JSON with sorted keys and no spaces: the same text the Rust recorder builds. */
export function canon(value: unknown): string {
  if (Array.isArray(value)) return `[${value.map(canon).join(',')}]`;
  if (typeof value === 'object' && value !== null) {
    const entries = Object.entries(value as Record<string, unknown>).sort(([a], [b]) =>
      a < b ? -1 : a > b ? 1 : 0,
    );
    return `{${entries.map(([k, v]) => `${JSON.stringify(k)}:${canon(v)}`).join(',')}}`;
  }
  return JSON.stringify(value);
}

/** The key of a command in the recorded edges: its name and arguments without `visible`. */
export function commandKey(name: string, args: Record<string, unknown>): string {
  const rest = Object.fromEntries(Object.entries(args).filter(([k]) => k !== 'visible'));
  return `${name}|${canon(rest)}`;
}

/** Test hooks and switches of the mock layer. */
export interface MockOptions {
  /** Make Play fail with "No audio output" (E-FB-8). */
  noAudio?: boolean;
}

/** The mock command layer plus what tests need to inspect or reset. */
export interface MockCommands extends Commands {
  /** Back to the start state. */
  reset: () => void;
  /** Every command that reached the mock, as `name|args`, in order. */
  log: string[];
}

function appError(kind: AppError['kind'], message: string): AppError {
  return { kind, message, details: null };
}

/** Creates a mock command layer over recorded scenarios. */
export function createMockCommands(scenarios: Scenarios, options: MockOptions = {}): MockCommands {
  let node = scenarios.start;
  let revision = 0;
  let settings: Settings = structuredClone(scenarios.nodes[scenarios.start]?.settings) as Settings;
  const log: string[] = [];

  const at = (id: string): ScenarioNode => {
    const found = scenarios.nodes[id];
    // eslint-disable-next-line @typescript-eslint/only-throw-error -- commands reject with plain AppError data, like Tauri
    if (!found) throw appError('Internal', `The mock has no state ${id}`);
    return found;
  };

  const snapshot = (): ExplorerSnapshot => {
    revision += 1;
    return { ...at(node).snapshot, revision };
  };

  const step = (name: string, args: Record<string, unknown>): Promise<ExplorerSnapshot> => {
    const key = commandKey(name, args);
    log.push(key);
    const target = scenarios.edges[node]?.[key];
    if (target === undefined) {
      // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors -- AppError is plain data, like Tauri's
      return Promise.reject(
        appError(
          'Invalid',
          `The mock has no recorded answer for ${key}. Record it in bindings.rs.`,
        ),
      );
    }
    node = target;
    const next = at(node).snapshot;
    // Transient notices belong to the action that produced them.
    return Promise.resolve({ ...next, revision: ++revision });
  };

  const noop = (name: string): Promise<void> => {
    log.push(name);
    return Promise.resolve();
  };

  const play = (name: string): Promise<void> => {
    log.push(name);
    return options.noAudio === true
      ? // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors -- AppError is plain data, like Tauri's
        Promise.reject(appError('NoAudioOutput', 'No audio output'))
      : Promise.resolve();
  };

  const merge = (patch: SettingsPatch): Settings => {
    const next = structuredClone(settings);
    if (patch.instrument) next.instrument = patch.instrument;
    if (patch.display) next.display = patch.display;
    if (patch.sound) next.sound = { ...patch.sound, volume: Math.min(100, patch.sound.volume) };
    if (patch.tuner) next.tuner = patch.tuner;
    return next;
  };

  const commands: MockCommands = {
    log,
    reset: () => {
      node = scenarios.start;
      revision = 0;
      settings = structuredClone(at(node).settings);
      log.length = 0;
    },
    appInfo: () =>
      Promise.resolve({ name: 'Mackey Guitar Tutor', version: '0.0.0-mock', os: 'mock' }),
    getSnapshot: () => Promise.resolve(snapshot()),
    fretboardView: () => Promise.resolve(snapshot().view),
    selectKey: (tonic, mode) => step('select_key', { tonic, mode }),
    selectDiatonicChord: (degree) => step('select_diatonic_chord', { degree }),
    selectChord: (text) => step('select_chord', { text }),
    parseChord: (text) =>
      Promise.resolve(scenarios.parse[text] ?? { chord: null, diagnostics: [] }),
    selectNote: (string, fret) => step('select_note', { string, fret }),
    selectTuning: (tuning) => step('select_tuning', { tuning }),
    setCapo: (capo) => step('set_capo', { capo }),
    setViewType: (view) => step('set_view_type', { view }),
    setLabelsMode: (labels) => step('set_labels_mode', { labels }),
    setScaleOverlay: (overlay) => step('set_scale_overlay', { overlay }),
    setOption: (option) => step('set_option', { option }),
    cycle: (action) => step('cycle', { action }),
    voicingsFor: () => Promise.resolve({ kind: 'NoneFound', value: { options: [] } }),
    fitSelection: () => Promise.resolve(at(node).snapshot.view.fit_range),
    playSelection: () => play('play_selection'),
    playNote: (string, fret) => play(`play_note|${string.toString()}|${fret.toString()}`),
    stopPlayback: () => noop('stop_playback'),
    listInputDevices: () =>
      Promise.resolve([
        { id: 'mock-built-in', name: 'Built-in Microphone', is_default: true },
        { id: 'mock-interface', name: 'USB Audio Interface', is_default: false },
      ]),
    tunerTargets: () => Promise.resolve(scenarios.tuner_targets),
    tunerStart: () => {
      log.push('tuner_start');
      return Promise.resolve({
        permission: 'Granted',
        device: { id: 'mock-built-in', name: 'Built-in Microphone', is_default: true },
        targets: scenarios.tuner_targets,
      });
    },
    tunerStop: () => noop('tuner_stop'),
    tunerSetMode: (mode) => noop(`tuner_set_mode|${canon(mode)}`),
    tunerReferenceTone: (string, on) =>
      noop(`tuner_reference_tone|${string.toString()}|${on.toString()}`),
    openSystemSettings: () => noop('open_system_settings'),
    getSettings: () => Promise.resolve(structuredClone(settings)),
    setSettings: (patch) => {
      const key = commandKey('set_settings', { patch });
      log.push(key);
      settings = merge(patch);
      const target = scenarios.edges[node]?.[key];
      if (target !== undefined) node = target;
      return Promise.resolve(structuredClone(settings));
    },
    resetSettings: (group) => {
      log.push(`reset_settings|${group}`);
      const defaults = at(scenarios.start).settings;
      const next = structuredClone(settings);
      if (group === 'Instrument') next.instrument = structuredClone(defaults.instrument);
      if (group === 'Display') next.display = structuredClone(defaults.display);
      if (group === 'Sound') next.sound = structuredClone(defaults.sound);
      if (group === 'Tuner') next.tuner = structuredClone(defaults.tuner);
      settings = next;
      return Promise.resolve(structuredClone(settings));
    },
    takeSettingsNotice: () => Promise.resolve(null),
  };
  return commands;
}

/** The mock the app uses in `vite --mode mock` and in plain browsers. */
export const mockCommands: MockCommands = createMockCommands(
  parseScenarios(scenariosText),
  typeof location === 'undefined'
    ? {}
    : { noAudio: new URLSearchParams(location.search).has('noaudio') },
);
