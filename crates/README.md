# crates

The Rust core: one crate per domain, all in the root Cargo workspace. No crates exist yet; roadmap phase P0 creates them. The planned crates, what each may depend on and which may do I/O are in [AGENTS.md](../AGENTS.md#architecture-pure-core-thin-shell) and [docs/architecture.md §2](../docs/architecture.md#2-module-layout).

To add a crate:

1. `cargo new --lib crates/mackey-<domain>` and add the path to `members` in the root `Cargo.toml`.
2. In its `Cargo.toml`, inherit `edition`, `rust-version`, `license`, `repository` and `publish` from the workspace, take every dependency as `{ workspace = true }`, and set `[lints] workspace = true`.
3. Add it to the layer table in `xtask/src/archcheck.rs` and in `AGENTS.md` (`make archcheck` fails otherwise).
4. Give it a `README.md` (purpose, modules, dependencies in and out) and add a row below.

| Crate | Purpose |
|---|---|
| None yet | |

Back to the [root README](../README.md).
