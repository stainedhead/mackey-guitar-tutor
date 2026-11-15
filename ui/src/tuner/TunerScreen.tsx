import { useCallback, useEffect, useRef, useState } from 'react';
import type { InputDevice } from '../bindings/InputDevice';
import type { TunerMode } from '../bindings/TunerMode';
import type { TunerReading } from '../bindings/TunerReading';
import type { TunerTarget } from '../bindings/TunerTarget';
import { toAppError } from '../ipc/tauri';
import { useStore } from '../state/context';
import { useUiState } from '../state/store';
import { describe, targetText } from './describe';
import { Needle } from './Needle';

type Phase = 'explain' | 'starting' | 'running' | 'denied' | 'nodevice' | 'error';

const MIC_EXPLAINER =
  'Mackey uses the microphone only to hear your guitar. Nothing is recorded or sent anywhere.';

function notice(reading: TunerReading | null): string | null {
  switch (reading?.notice) {
    case 'DeviceDisconnectedUsingDefault':
      return 'The input device was unplugged. Using the default input.';
    case 'DeviceChanged':
      return 'The input device changed.';
    case 'InputNotPaced':
      return 'This input is not delivering audio at a normal rate. Choose another input.';
    case 'AnalysisStopped':
      return 'The tuner stopped unexpectedly. Close and reopen it.';
    default:
      return null;
  }
}

/** The tuner screen (FR-027): renders `TunerReading`s from Rust; decides nothing about pitch. */
export function TunerScreen() {
  const store = useStore();
  const settings = useUiState(store, (s) => s.settings);
  const explained = settings?.tuner.mic_explained ?? false;
  const [phase, setPhase] = useState<Phase>(explained ? 'starting' : 'explain');
  const [reading, setReading] = useState<TunerReading | null>(null);
  const [targets, setTargets] = useState<TunerTarget[]>([]);
  const [devices, setDevices] = useState<InputDevice[]>([]);
  const [device, setDevice] = useState<string | null>(null);
  const [mode, setMode] = useState<'chromatic' | 'strings'>('chromatic');
  const [locked, setLocked] = useState<number | null>(null);
  const [tone, setTone] = useState<number | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const running = useRef(false);
  const unsubscribe = useRef<(() => void) | null>(null);
  const toneTimer = useRef<number | null>(null);

  const stop = useCallback(async () => {
    unsubscribe.current?.();
    unsubscribe.current = null;
    if (running.current) {
      running.current = false;
      await store.commands.tunerStop();
    }
  }, [store]);

  const start = useCallback(
    async (deviceId: string | null) => {
      setPhase('starting');
      setMessage(null);
      // Reference tones need the strings even when the microphone is unavailable (E-TU-2).
      void store.commands.tunerTargets().then(setTargets);
      try {
        await stop();
        const unsub = await store.tunerEvents.listen((r) => {
          setReading(r);
        });
        unsubscribe.current = unsub;
        const info = await store.commands.tunerStart(deviceId);
        running.current = true;
        setTargets(info.targets);
        setDevice(info.device.id);
        setPhase(info.permission === 'Denied' ? 'denied' : 'running');
        setDevices(await store.commands.listInputDevices());
      } catch (e) {
        unsubscribe.current?.();
        unsubscribe.current = null;
        const error = toAppError(e);
        if (error.kind === 'MicrophoneDenied') setPhase('denied');
        else if (error.kind === 'NoInputDevice') setPhase('nodevice');
        else setPhase('error');
        setMessage(error.message);
      }
    },
    [store, stop],
  );

  // Start when the explainer was already accepted; stop on leaving; pause when hidden (SAC-16).
  useEffect(() => {
    if (explained) void start(settings?.tuner.input_device ?? null);
    const onVisibility = (): void => {
      if (document.visibilityState === 'hidden') void stop();
      else if (explained) void start(device);
    };
    document.addEventListener('visibilitychange', onVisibility);
    return () => {
      document.removeEventListener('visibilitychange', onVisibility);
      void stop();
      if (toneTimer.current !== null) window.clearTimeout(toneTimer.current);
      void store.commands.tunerReferenceTone(1, false);
    };
    // The tuner starts once per mount (or when the explainer is accepted).
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [explained]);

  const view = describe(reading, targets.length > 0);
  useEffect(() => {
    if (view.speech !== '') store.announceTuner(view.speech);
  }, [store, view.speech]);

  const accept = async (): Promise<void> => {
    if (!settings) return;
    await store.updateSettings({ tuner: { ...settings.tuner, mic_explained: true } });
  };

  const changeMode = (next: 'chromatic' | 'strings', lock: number | null): void => {
    setMode(next);
    setLocked(lock);
    const wire: TunerMode = next === 'chromatic' ? 'Chromatic' : { Strings: { locked: lock } };
    void store.commands.tunerSetMode(wire);
  };

  const playTone = (string: number): void => {
    if (toneTimer.current !== null) window.clearTimeout(toneTimer.current);
    if (tone === string) {
      setTone(null);
      void store.commands.tunerReferenceTone(string, false);
      return;
    }
    setTone(string);
    void store.commands.tunerReferenceTone(string, true).catch((e: unknown) => {
      setTone(null);
      setMessage(toAppError(e).message);
    });
    toneTimer.current = window.setTimeout(() => {
      setTone(null);
      void store.commands.tunerReferenceTone(string, false);
    }, 3000);
  };

  const pickDevice = (id: string): void => {
    setDevice(id);
    if (settings) void store.updateSettings({ tuner: { ...settings.tuner, input_device: id } });
    void start(id);
  };

  const body = (() => {
    if (phase === 'explain') {
      return (
        <section className="mic-explainer" aria-label="Microphone">
          <p>{MIC_EXPLAINER}</p>
          <button type="button" className="primary" onClick={() => void accept()}>
            Continue
          </button>
        </section>
      );
    }
    return null;
  })();

  const note = notice(reading);
  const level = reading ? Math.min(1, reading.level.peak * 2) : 0;
  const cents = reading?.target?.cents ?? reading?.cents ?? null;
  const showTargets = targets.length > 0;
  return (
    <div className="tuner" data-testid="tuner">
      <h2>Tuner</h2>
      {body}
      {phase === 'denied' && (
        <section className="mic-denied" role="alert">
          <p>
            <span aria-hidden="true">⚠ </span>Microphone access is off
          </p>
          <p>
            Turn on the microphone for Mackey in System Settings to use the tuner. The reference
            tones below still work.
          </p>
          <button
            type="button"
            onClick={() => void store.commands.openSystemSettings('Microphone')}
          >
            Open Settings
          </button>
        </section>
      )}
      {phase === 'nodevice' && (
        <section className="mic-denied" role="alert">
          <p>
            <span aria-hidden="true">⚠ </span>No microphone found
          </p>
          <p>Connect a microphone or audio interface. The reference tones below still work.</p>
        </section>
      )}
      {phase === 'error' && message !== null && <p role="alert">{message}</p>}
      {note !== null && (
        <p className="tuner-notice" role="status">
          {note}
        </p>
      )}

      {(phase === 'running' || phase === 'starting') && (
        <>
          <div className="segmented" role="radiogroup" aria-label="Tuner mode">
            <button
              type="button"
              role="radio"
              aria-checked={mode === 'chromatic'}
              onClick={() => {
                changeMode('chromatic', null);
              }}
            >
              Chromatic
            </button>
            <button
              type="button"
              role="radio"
              aria-checked={mode === 'strings'}
              onClick={() => {
                changeMode('strings', null);
              }}
            >
              Strings
            </button>
          </div>
          <div className={`readout tone-${view.tone}`} data-testid="readout">
            <div className="note-big" data-testid="tuner-note">
              {view.tone === 'good' && (
                <span className="check-mark" aria-hidden="true">
                  ✓{' '}
                </span>
              )}
              {view.headline}
            </div>
            {view.detail !== null && (
              <div className="detail" data-testid="tuner-detail">
                {view.detail}
              </div>
            )}
            {reading?.status === 'Pitch' && reading.frequency_hz !== null && (
              <div className="hz">{reading.frequency_hz.toFixed(2)} Hz</div>
            )}
            <Needle
              cents={reading?.status === 'Pitch' ? cents : null}
              inTune={reading?.in_tune ?? false}
            />
          </div>
          <div
            className="level"
            role="meter"
            aria-label="Input level"
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={Math.round(level * 100)}
          >
            <div className="level-fill" style={{ width: `${(level * 100).toString()}%` }} />
          </div>
        </>
      )}

      {showTargets && (
        <section className="targets" aria-label="Strings">
          <h3>Strings</h3>
          <ul>
            {[...targets].reverse().map((t) => {
              const current = reading?.target?.string === t.string && mode === 'strings';
              const done = reading?.ticks.includes(t.string) ?? false;
              return (
                <li key={t.string}>
                  <button
                    type="button"
                    className={current ? 'target current' : 'target'}
                    aria-pressed={locked === t.string}
                    aria-label={`String ${t.string.toString()}, ${targetText(t)}${done ? ', tuned' : ''}${locked === t.string ? ', locked' : ''}`}
                    onClick={() => {
                      changeMode('strings', locked === t.string ? null : t.string);
                    }}
                  >
                    <span className="target-name">{targetText(t)}</span>
                    <span className="target-string">string {t.string}</span>
                    {done && (
                      <span className="tick" aria-hidden="true">
                        ✓
                      </span>
                    )}
                  </button>
                  <button
                    type="button"
                    className="tone-button"
                    aria-pressed={tone === t.string}
                    aria-label={`Play reference tone ${targetText(t)}`}
                    onClick={() => {
                      playTone(t.string);
                    }}
                  >
                    {tone === t.string ? '■' : '♪'}
                  </button>
                </li>
              );
            })}
          </ul>
        </section>
      )}

      {devices.length > 0 && (
        <label className="device-picker">
          Input
          <select
            value={device ?? ''}
            onChange={(e) => {
              pickDevice(e.target.value);
            }}
          >
            {devices.map((d) => (
              <option key={d.id} value={d.id}>
                {d.name}
                {d.is_default ? ' (default)' : ''}
              </option>
            ))}
          </select>
        </label>
      )}
      {message !== null && phase === 'running' && <p role="status">{message}</p>}
    </div>
  );
}
