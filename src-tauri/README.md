# src-tauri (`mackey-app`)

The Tauri 2 shell: a thin adapter that maps UI commands to core calls, owns app state and forwards events. **No music logic lives here** (it belongs in the pure crates under `crates/`); this is the only crate that may depend on `tauri*`. Targets: macOS (universal) and iPad via Tauri iOS.

| Path | Purpose |
|---|---|
| `src/lib.rs` | Crate root; `run` is also the iOS entry point |
| `src/main.rs` | Desktop `main`, calls `mackey_app::run` |
| `src/runtime.rs` | `tauri::Builder` setup and `run` (glue, excluded from coverage) |
| `src/commands.rs` | Commands (`app_info` today), their types (`AppInfo`), the `COMMAND_NAMES` list and its tests |
| `src/error.rs` | `AppError { message, details }`: serialisable, plain message for the user |
| `src/guard.rs` | `guard`: runs a command body under `catch_unwind` so a panic becomes `AppError "Something went wrong"` (AC-NF-24); the release profile keeps `panic = "unwind"` |
| `build.rs` | `tauri_build::build()` |
| `tauri.conf.json` | Identifier `dev.stainedhead.mackey`, window 1280×800 (min 900×600), strict CSP (no remote content), `frontendDist ../ui/dist`, macOS 13.0+, iOS 17.0+, ad-hoc signing (`signingIdentity "-"`) |
| `Info.plist`, `Info.ios.plist` | Merged by Tauri into the macOS and iOS bundles: the `NSMicrophoneUsageDescription` text |
| `capabilities/` | Permission sets; see [capabilities/README.md](capabilities/README.md) |
| `icons/` | App icons; see [icons/README.md](icons/README.md) |

`gen/` (schemas and the iOS Xcode project from `tauri ios init`) is generated and git-ignored.

## Bindings

`AppInfo` and `AppError` derive `ts_rs::TS`; `make bindings` writes them to [`../ui/src/bindings/`](../ui/src/bindings/README.md) and `make bindings-check` fails when they are stale. `COMMAND_NAMES` has a test that the UI's [`ipc/tauri.ts`](../ui/src/ipc/tauri.ts) names every command.

## Building

`make build` and friends need `ui/dist/` to exist at compile time (`tauri::generate_context!`); the Makefile writes a stub when it is missing. On Linux you also need the WebKitGTK and ALSA dev packages (see [.github/README.md](../.github/README.md)). `make app` builds the real app; `make ios-sim` the iOS Simulator build.

Part of the [architecture](../docs/architecture.md); depends on the core crates and nothing depends on it. Back to the [root README](../README.md).
