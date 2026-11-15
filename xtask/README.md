# xtask

Repo automation written in Rust, run with `cargo xtask <task>` (alias in `.cargo/config.toml`) and wrapped by `make`. It is a workspace member so it shares the toolchain and lints, but it is never shipped and nothing depends on it.

| Task | Make target | What it checks |
|---|---|---|
| `archcheck` | `make archcheck` | Reads `cargo metadata` and enforces the crate layer table in [AGENTS.md](../AGENTS.md#architecture-pure-core-thin-shell): allowed workspace dependencies, one owner for each I/O crate family (`cpal`, `rusqlite`, `reqwest`, `tauri`, ...), no runtimes or thread pools in pure crates, and every workspace crate registered in the table. Dev-dependencies are exempt |
| `doccheck` | `make doccheck` | Every `.PHONY` target in the `Makefile` has a `make <target>` row in the AGENTS.md Commands table, and every folder has a `README.md` (code folders such as `src/` and `tests/`, build output and `docs/adr/` are exempt) |

| File | Purpose |
|---|---|
| `src/main.rs` | Task dispatch and the `cargo metadata` call |
| `src/archcheck.rs` | The layer table (`LAYERS`, `EXCLUSIVE`, `IMPURE`) and its tests |
| `src/doccheck.rs` | The docs checks and their tests |

When you add a crate, add it to `LAYERS` in `src/archcheck.rs` in the same change. Back to the [root README](../README.md).
