# ui/src/bindings

TypeScript generated from the Rust types by `ts-rs` and from the command manifest in `src-tauri` (`make bindings`, which runs the `export_bindings*` tests with `TS_RS_EXPORT_DIR` from `.cargo/config.toml`). **Do not hand-edit**: `make bindings-check` (CI job `bindings`) regenerates and fails if anything differs, including `../mocks/scenarios.json`. To change a type or a command, change the Rust side and regenerate.

Generated: one file per boundary type (`FretboardView.ts`, `ExplorerSnapshot.ts`, `Settings.ts`, `TunerReading.ts`, `AppError.ts` and so on), `commands.ts` (a typed `Commands` interface and `createCommands(call)`, one method per Tauri command) and `events.ts` (`listenTunerReading`).
