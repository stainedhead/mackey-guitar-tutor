# ui/src/bindings

TypeScript types generated from the Rust types by `ts-rs` (`make bindings`, which runs the crate's `export_bindings_*` tests with `TS_RS_EXPORT_DIR` from `.cargo/config.toml`). **Do not hand-edit**: `make bindings-check` (CI job `bindings`) regenerates and fails if anything differs. To change a type, change the Rust struct and regenerate.

Generated: `AppInfo.ts`, `AppError.ts`.
