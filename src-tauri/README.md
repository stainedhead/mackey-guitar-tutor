# src-tauri (`mackey-app`)

The Tauri 2 shell: a thin adapter that maps UI commands to core calls, owns app state and forwards events. **No music logic lives here** (it belongs in the pure crates under `crates/`); this is the only crate that may depend on `tauri*`. Targets: macOS (universal) and iPad via Tauri iOS.

| Path | Purpose |
|---|---|
| `src/lib.rs` | Crate root; `run` is also the iOS entry point |
| `src/main.rs` | Desktop `main`, calls `mackey_app::run` |
| `src/runtime.rs` | `tauri::Builder` setup, state, logging and `run` (glue, excluded from coverage) |
| `src/state.rs` | `AppState`: the explorer, settings and audio behind three mutexes (always locked in that order, never touched by the audio thread; a poisoned lock is recovered). Every command body lives here and runs without Tauri |
| `src/explorer.rs` | `Explorer`: selection, instrument, handedness, `VoicingCache` and revision counter; builds the `ExplorerSnapshot` the UI draws (view, panel, notices, live-region announcement) |
| `src/settings.rs` | `Settings` (Instrument, Display, Sound, Tuner), JSON load and atomic save in the app data folder, corrupt-file backup, reset per group (FR-028) |
| `src/ports.rs` | `AudioPort`: `RealAudio` opens devices lazily on first use, `FakeAudio` records calls for tests |
| `src/tuner.rs` | `Throttle`: forwards about every third reading, so the UI gets `tuner_reading` at about 30 Hz |
| `src/commands.rs`, `src/commands_list.rs` | 32 thin `#[tauri::command]` wrappers and the x-macro list that registers them and builds `COMMAND_NAMES` |
| `src/manifest.rs` | The table (name, arguments, result) that generates the TypeScript command wrappers |
| `src/bindings.rs` | Test-only generators behind `make bindings`: `commands.ts`, `events.ts` and `scenarios.json` (the recorded states the mock UI replays) |
| `src/error.rs` | `AppError { kind, message, details }` with `From` conversions from every core error; plain message for the user |
| `src/guard.rs` | `guard`: runs a command body under `catch_unwind` so a panic becomes `AppError "Something went wrong"` (AC-NF-24); the release profile keeps `panic = "unwind"` |
| `src/logging.rs` | `tracing` to a daily rolling file in the log folder (three kept) |
| `tests/state.rs` | Integration tests through `AppState` with `FakeAudio`: AC-P0-5, AC-FB-2.2, E-FB-8, strum timing, tuner start and stop, settings persistence, concurrency |
| `build.rs` | `tauri_build::build()` |
| `tauri.conf.json` | Identifier `dev.stainedhead.mackey`, window 1280×800 (min 900×600), strict CSP (no remote content), `frontendDist ../ui/dist`, macOS 13.0+, iOS 17.0+, ad-hoc signing (`signingIdentity "-"`) |
| `Info.plist`, `Info.ios.plist` | Merged by Tauri into the macOS and iOS bundles: the `NSMicrophoneUsageDescription` text |
| `capabilities/` | Permission sets; see [capabilities/README.md](capabilities/README.md) |
| `icons/` | App icons; see [icons/README.md](icons/README.md) |

## Commands and events

Explorer: `get_snapshot`, `fretboard_view`, `select_key`, `select_diatonic_chord`, `select_chord`, `parse_chord`, `select_note`, `select_tuning`, `set_capo`, `set_view_type`, `set_labels_mode`, `set_scale_overlay`, `set_option`, `cycle`, `voicings_for`, `fit_selection`. Audio: `play_selection`, `play_note`, `stop_playback`, `list_input_devices`, `open_system_settings`. Tuner: `tuner_targets`, `tuner_start`, `tuner_stop`, `tuner_set_mode`, `tuner_reference_tone`. Settings: `get_settings`, `set_settings`, `reset_settings`, `take_settings_notice`. Also `app_info`. Every command returns `Result<T, AppError>`.

One event, `tuner_reading` (a `TunerReading`), is emitted from the analysis thread through a throttle while the tuner runs.

`gen/` (schemas and the iOS Xcode project from `tauri ios init`) is generated and git-ignored.

## Bindings

Boundary types derive `ts_rs::TS` (the core crates through their `ts` feature); `make bindings` writes them, plus `commands.ts`, `events.ts` and the mock `scenarios.json`, to [`../ui/src/bindings/`](../ui/src/bindings/README.md) and [`../ui/src/mocks/`](../ui/src/mocks), and `make bindings-check` fails when any is stale. A test checks that the manifest names exactly the registered commands.

## Building

`make build` and friends need `ui/dist/` to exist at compile time (`tauri::generate_context!`); the Makefile writes a stub when it is missing. On Linux you also need the WebKitGTK and ALSA dev packages (see [.github/README.md](../.github/README.md)). `make app` builds the real app; `make ios-sim` the iOS Simulator build.

Part of the [architecture](../docs/architecture.md); depends on the core crates and nothing depends on it. Back to the [root README](../README.md).
