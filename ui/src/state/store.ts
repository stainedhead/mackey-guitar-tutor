import { useSyncExternalStore } from 'react';
import type { AppError } from '../bindings/AppError';
import type { Degree } from '../bindings/Degree';
import type { CycleAction } from '../bindings/CycleAction';
import type { ExplorerOption } from '../bindings/ExplorerOption';
import type { ExplorerSnapshot } from '../bindings/ExplorerSnapshot';
import type { FretRange } from '../bindings/FretRange';
import type { LabelsMode } from '../bindings/LabelsMode';
import type { Mode } from '../bindings/Mode';
import type { Note } from '../bindings/Note';
import type { OverlayRequest } from '../bindings/OverlayRequest';
import type { Settings } from '../bindings/Settings';
import type { SettingsGroup } from '../bindings/SettingsGroup';
import type { SettingsPatch } from '../bindings/SettingsPatch';
import type { TuningChoice } from '../bindings/TuningChoice';
import type { ViewType } from '../bindings/ViewType';
import type { Commands, TunerEvents } from '../ipc/types';
import { createQueue } from '../ipc/queue';
import { toAppError } from '../ipc/tauri';

/** The screens of the app. */
export type Area = 'explore' | 'tuner' | 'settings';

/** View state only (ADR 0007): what is on screen, never what the music is. */
export interface UiState {
  snapshot: ExplorerSnapshot | null;
  /** The fret range in the neck's viewport; sent with every command for off-range counts. */
  visible: FretRange | null;
  /** The frets the neck is zoomed to (default per layout, "Fit selection", range controls). */
  zoom: FretRange | null;
  area: Area;
  sidebarOpen: boolean;
  /** A brief message under the toolbar (notices, errors). */
  notice: string | null;
  /** What the screen-reader live region says. */
  liveMessage: string;
  settings: Settings | null;
  /** True once "No audio output" was shown (E-FB-8: once per session). */
  audioNoticeShown: boolean;
  /** The most recent command error, for tests and the details disclosure. */
  error: AppError | null;
  /** The chord field text while the user types. */
  draftChord: string | null;
}

const INITIAL: UiState = {
  snapshot: null,
  visible: null,
  zoom: null,
  area: 'explore',
  sidebarOpen: true,
  notice: null,
  liveMessage: '',
  settings: null,
  audioNoticeShown: false,
  error: null,
  draftChord: null,
};

type Listener = () => void;

/** The store and every action the UI can take. All Rust calls go through `commands`. */
export class UiStore {
  private state: UiState = INITIAL;
  private readonly listeners = new Set<Listener>();
  private readonly queue = createQueue();
  private flip = false;

  constructor(
    readonly commands: Commands,
    readonly tunerEvents: TunerEvents,
  ) {}

  getState = (): UiState => this.state;

  subscribe = (listener: Listener): (() => void) => {
    this.listeners.add(listener);
    return () => {
      this.listeners.delete(listener);
    };
  };

  private set(partial: Partial<UiState>): void {
    this.state = { ...this.state, ...partial };
    for (const l of [...this.listeners]) l();
  }

  /** Loads settings and the first snapshot (AC-P0-5). */
  init = async (): Promise<void> => {
    try {
      const [settings, snapshot, notice] = await Promise.all([
        this.commands.getSettings(),
        this.commands.getSnapshot(this.state.visible),
        this.commands.takeSettingsNotice(),
      ]);
      this.set({ settings, snapshot, notice });
    } catch (e) {
      this.fail(e);
    }
  };

  private fail(e: unknown): void {
    const error = toAppError(e);
    this.set({ error, notice: error.message });
  }

  private lastTunerAnnouncement = 0;

  /** Tuner announcements reach the live region at most once a second (NF-5). */
  announceTuner = (text: string, now: number = Date.now()): void => {
    if (now - this.lastTunerAnnouncement < 1000) return;
    this.lastTunerAnnouncement = now;
    this.announce(text);
  };

  private announce(text: string): void {
    this.flip = !this.flip;
    // A trailing no-break space makes a repeated message count as new text for screen readers.
    this.set({ liveMessage: this.flip ? text : `${text}\u00a0` });
  }

  /** Runs an explorer command in order and applies its snapshot unless a newer one is shown. */
  private run(
    call: (visible: FretRange | null) => Promise<ExplorerSnapshot>,
    announce = true,
  ): Promise<void> {
    return this.queue
      .enqueue(() => call(this.state.visible))
      .then((snapshot) => {
        const current = this.state.snapshot;
        if (current && snapshot.revision < current.revision) return; // stale (SAC-18)
        this.set({
          snapshot,
          draftChord: null,
          notice: snapshot.notices.length > 0 ? snapshot.notices.join(' · ') : null,
          error: null,
        });
        if (announce && snapshot.announcement !== '') this.announce(snapshot.announcement);
      })
      .catch((e: unknown) => {
        this.fail(e);
      });
  }

  cycle = (action: CycleAction): Promise<void> => this.run((v) => this.commands.cycle(action, v));
  selectDiatonic = (degree: Degree): Promise<void> =>
    this.run((v) => this.commands.selectDiatonicChord(degree, v));
  selectChord = (text: string): Promise<void> =>
    this.run((v) => this.commands.selectChord(text, v));
  selectKey = (tonic: Note, mode: Mode): Promise<void> =>
    this.run((v) => this.commands.selectKey(tonic, mode, v));
  selectNote = (string: number, fret: number): Promise<void> =>
    this.run((v) => this.commands.selectNote(string, fret, v));
  selectTuning = (tuning: TuningChoice): Promise<void> =>
    this.run((v) => this.commands.selectTuning(tuning, v));
  setCapo = (capo: number): Promise<void> => this.run((v) => this.commands.setCapo(capo, v));
  setViewType = (view: ViewType): Promise<void> =>
    this.run((v) => this.commands.setViewType(view, v));
  setLabels = (labels: LabelsMode): Promise<void> =>
    this.run((v) => this.commands.setLabelsMode(labels, v));
  setOverlay = (overlay: OverlayRequest): Promise<void> =>
    this.run((v) => this.commands.setScaleOverlay(overlay, v));
  setOption = (option: ExplorerOption): Promise<void> =>
    this.run((v) => this.commands.setOption(option, v));
  refresh = (): Promise<void> => this.run((v) => this.commands.getSnapshot(v), false);

  /** The neck's viewport changed: remember it and refresh the off-range counts. */
  setVisible = (visible: FretRange): Promise<void> => {
    const current = this.state.visible;
    if (current?.from === visible.from && current.to === visible.to) return Promise.resolve();
    this.set({ visible });
    return this.refresh();
  };

  /** Zooms the neck to a fret range; the neck reports the viewport back through `setVisible`. */
  setZoom = (zoom: FretRange): void => {
    const current = this.state.zoom;
    if (current?.from === zoom.from && current.to === zoom.to) return;
    this.set({ zoom });
  };

  setDraftChord = (draftChord: string | null): void => {
    this.set({ draftChord });
  };

  setArea = (area: Area): void => {
    this.set({ area });
  };

  toggleSidebar = (): void => {
    this.set({ sidebarOpen: !this.state.sidebarOpen });
  };

  setNotice = (notice: string | null): void => {
    this.set({ notice });
  };

  /** A sound command: "No audio output" shows once, then Play stays quiet about it (E-FB-8). */
  private sound(call: () => Promise<void>): Promise<void> {
    return this.queue
      .enqueue(call)
      .then(() => {
        this.set({ error: null });
      })
      .catch((e: unknown) => {
        const error = toAppError(e);
        if (error.kind === 'NoAudioOutput') {
          if (this.state.audioNoticeShown) return;
          this.set({ audioNoticeShown: true, error, notice: error.message });
          return;
        }
        this.fail(e);
      });
  }

  play = (): Promise<void> => this.sound(() => this.commands.playSelection(this.state.visible));
  playNote = (string: number, fret: number): Promise<void> =>
    this.sound(() => this.commands.playNote(string, fret));

  /** Applies a settings change at once; the explorer refreshes because instrument settings feed it. */
  updateSettings = async (patch: Partial<SettingsPatch>): Promise<void> => {
    const full: SettingsPatch = {
      instrument: null,
      display: null,
      sound: null,
      tuner: null,
      ...patch,
    };
    try {
      const settings = await this.queue.enqueue(() => this.commands.setSettings(full));
      this.set({ settings });
      await this.refresh();
    } catch (e) {
      this.fail(e);
    }
  };

  resetSettings = async (group: SettingsGroup): Promise<void> => {
    try {
      const settings = await this.queue.enqueue(() => this.commands.resetSettings(group));
      this.set({ settings });
      await this.refresh();
    } catch (e) {
      this.fail(e);
    }
  };
}

/** Subscribes a component to a slice of the store. */
export function useUiState<T>(store: UiStore, select: (state: UiState) => T): T {
  return useSyncExternalStore(store.subscribe, () => select(store.getState()));
}
