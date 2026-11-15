# mackey-audio

Audio for Mackey Guitar Tutor: a McLeod pitch detector and the tuner state machine, a plucked-string synth with a playback-plan builder, and the cpal glue that runs them. It is the only crate that opens audio devices or the microphone. It depends on `mackey-theory` only (`Pitch`, `Note`, `ReferencePitch`); `mackey-app` depends on it. The app converts the instrument (tuning plus capo) into `TunerTarget`s and chord voicings or scale patterns into MIDI-note lists, so this crate never sees `mackey-voicing` (D-08).

Design: [specs/261115-fretboard-foundation/architecture.md](../../specs/261115-fretboard-foundation/architecture.md) (Audio), [docs/prd/02-tuner.md](../../docs/prd/02-tuner.md), [docs/learning-features.md](../../docs/learning-features.md), [ADR 0006](../../docs/adr/0006-on-device-audio-analysis.md) (all analysis stays on the device).

## Modules (`src/`)

| Module | What it provides |
|---|---|
| `dsp::pitch` | `DetectorConfig`, `PitchDetector` (MPM: direct NSDF, key maxima, parabolic interpolation; 60 to 1,400 Hz; 2048-sample window, 512 hop) |
| `dsp::cents` | `cents_between`, `cents_from`, `nearest_pitch` (nearest equal-tempered pitch, reference A4 415 to 466 Hz) |
| `dsp::level` | `LevelMeter`, `Level` (RMS, peak, clipping) |
| `dsp::tuner` | `Tuner` state machine (`push_block` returns `TunerReading`s), `TunerConfig`, `TunerTarget`, `targets_from_pitches`, `TunerStatus`, `TunerNotice` |
| `dsp::synth` | `Synth` (16 preallocated Karplus-Strong voices, allocation-free `render`), `SynthCommand` |
| `dsp::plan` | `PlaybackPlan` (strum 30 ms apart, arpeggio and scale in eighths at a tempo) and `commands()` into `SynthCommand`s |
| `io::select` | Device and stream-configuration choice from plain lists (`choose_input_device`, `choose_config`), `InputDevice`, `MicPermission` |
| `io::pump` | What the output callback and the analysis thread do with their rings (tested with fakes) |
| `io::{devices, output, input}` | The cpal calls and threads: `list_input_devices`, `OutputHandle`, `InputHandle`, `TunerInput` |
| `error` | `AudioError` (no cpal types in public signatures) |

## Using it from the app

```rust,ignore
// Tuner: one switch holding at most one input; start replaces, stop without start is a no-op.
let tuner = TunerInput::new();
let info = tuner.start(Some(&saved_device_id), cfg, move |reading| emit("tuner_reading", reading))?;
tuner.with(|h| h.lock(Some(3)));          // tap a string; lock(None) returns to auto-select
tuner.with(|h| h.permission());           // MicPermission::Denied after a second of exact zeros
tuner.stop();

// Sound: commands go through a lock-free queue; the callback renders the synth.
let out = OutputHandle::start()?;          // Err(NoOutputDevice) when there is none
let plan = PlaybackPlan::strum(&midi_low_to_high)?;
out.send_all(plan.commands(out.sample_rate(), a4, first_id));
out.send(SynthCommand::StopAll)?;
```

`TunerReading`, `TunerConfig`, `TunerTarget` and the other boundary types derive serde and, with `--features ts`, ts-rs bindings (`cargo test -p mackey-audio --features ts`).

## Real-time rules (AGENTS.md)

The output callback only drains an `rtrb` queue and calls `Synth::render`: no allocation, locks, logging or I/O (`tests/synth_noalloc.rs` proves it with `assert_no_alloc`). The input callback mixes down to mono and pushes into an `rtrb` ring. Analysis runs on its own thread (`mackey-tuner`), which also owns the input stream because cpal streams are not `Send` on every platform. Nothing is ever written to disk.

## Tests (`tests/`, `benches/`)

- Unit tests sit beside the code; names carry the requirement ID (`ac_tu_3_1_...`, `nf_8_1_...`).
- `tests/tuner_fixtures.rs`: AC-TU-1.1, 1.2 (24 detuned open strings at 44.1 and 48 kHz), 1.3, 3.1, AC-NF-9, E-TU-6, proptest on the detector and tuner.
- `tests/tuner_strings.rs`: AC-TU-2.1 (DADGAD), AC-TU-5.1 (capo 2), auto-select, lock.
- `tests/synth.rs`: synth pitch within 3 cents (measured with the detector), strum onsets sample-exact, determinism.
- `tests/synth_noalloc.rs`: `render` and `apply` never allocate (debug builds; the process aborts on an allocation).
- `tests/common/mod.rs`: synthetic signal generators (sines, guitar-like tones with inharmonic partials, decay and -50 dBFS noise).
- `benches/pitch.rs`: detector cost per window (about 0.12 ms, budget 2 ms).

## Coverage

`make cover` excludes only `io/devices.rs`, `io/input.rs` and `io/output.rs` (cpal calls and threads that need a sound card, `COVER_IGNORE` in the `Makefile`). Everything with a decision in it is in `dsp` and `io::{select, pump}` and is measured. On Linux the crate needs the ALSA headers (`libasound2-dev`) to compile.
