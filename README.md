# mackey-guitar-tutor
macos application that helps guitarists learn the neck and play songs (Rust core, Tauri shell; macOS and iPad)

## Design

Product spec, architecture and roadmap live in [docs/](docs/README.md): a fretboard explorer for keys, chords, triads and arpeggios; quick-entry practice sessions; and song chord/lyric import with fretboard help.

Start with [INTENT.md](INTENT.md) for purpose and scope, then the PRD in [docs/prd/](docs/prd/README.md).

## Core crates

The Rust core is built crate by crate ([crates/](crates/README.md)). So far: `mackey-theory` (notes, intervals, scales, keys, chords, chord-symbol parser, Roman numerals and analysis, spelling).

## Contributing

Rules for people and AI agents are in [AGENTS.md](AGENTS.md): crate layer rules, TDD with proptest and insta, Rust error handling, docs and the dev-flow spec workflow (`specs/`).

| Command | Purpose |
|---|---|
| `make tools` | One-time install of `cargo-deny` and `cargo-llvm-cov` |
| `make check` | fmt, clippy, layer and docs checks, licences and tests: run before every commit |
| `make cover` | Test coverage (≥ 80%) |

The toolchain is pinned in `rust-toolchain.toml` (rustup installs it on first use). Folder guides: [crates/](crates/README.md), [xtask/](xtask/README.md), [specs/](specs/README.md), [user-docs/](user-docs/README.md), [.github/](.github/README.md), [.claude/](.claude/README.md).
