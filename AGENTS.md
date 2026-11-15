# AGENTS.md

Rules for AI agents and human contributors working in `mackey-guitar-tutor`: **Mackey Guitar Tutor**, an app for macOS and iPad that helps beginner and intermediate guitarists learn the fretboard and play real songs. The stack is a **Rust core** (Cargo workspace) with a **Tauri 2 shell** and a **React + TypeScript UI** whose fretboard is drawn in **SVG**. Targets are macOS (universal binary) and iPad (Tauri iOS); Windows, Linux, iPhone and Android are deferred, so keep the core portable but do not build or test for them.

Before changing code, read `INTENT.md` (purpose, scope, key decisions), the root `README.md`, the `README.md` in the folders you touch, and `docs/architecture.md`. Order of authority: `INTENT.md` → the PRD in `docs/prd/` (numbered requirements, acceptance criteria, NFRs, traceability) → the design docs in `docs/` → the active spec under `specs/`. Decisions are in `docs/adr/`.

## Architecture: pure core, thin shell

Dependencies point inward only. The layout (from [docs/architecture.md §2](docs/architecture.md#2-module-layout)):

```
crates/
  mackey-theory/    notes, intervals, scales, chords, keys, chord-symbol parser      (pure)
  mackey-voicing/   voicing search, scale patterns, positions, voice-leading         (pure)
  mackey-practice/  practice-notation parser, session model, run engine              (pure)
  mackey-songs/     ChordPro + chords-over-lyrics parsers, chart model, transposer   (pure)
  mackey-rhythm/    transport clock, metronome, drum sequencer, speed trainer        (pure)
  mackey-audio/     cpal streams, synth, mixer, input analysis, recording            (only crate that opens audio devices or the mic)
  mackey-store/     SQLite, import/export                                            (only crate that touches the database)
  mackey-progress/  stats, streaks, drill suggestions
  mackey-sources/   SongSource trait, providers                                      (only crate that touches the network)
src-tauri/          mackey-app: Tauri commands, events, windows, menus (thin adapter; no music logic)
src-tauri/plugins/audio-session/   Swift AVAudioSession plugin (iPad only)
ui/                 React + TypeScript + Vite: fretboard SVG, chart, editors, transport bar
xtask/              repo automation (archcheck, doccheck); not shipped
```

Rules:

- **Pure crates** (`theory`, `voicing`, `practice`, `songs`, `rhythm`) do no I/O, spawn no threads, keep no global state and are deterministic. Parsers are pure functions from `&str` to a result with diagnostics. Nothing macOS-specific.
- Allowed direct dependencies between workspace crates:

  | Crate | May depend on |
  |---|---|
  | `mackey-theory` | nothing in the workspace |
  | `mackey-voicing` | theory |
  | `mackey-practice` | theory, voicing |
  | `mackey-songs` | theory, voicing |
  | `mackey-rhythm` | theory |
  | `mackey-audio` | theory, rhythm |
  | `mackey-store` | theory, voicing, practice, songs |
  | `mackey-progress` | theory, voicing, practice, songs, store |
  | `mackey-sources` | theory, voicing, songs |
  | `mackey-app` | any core crate; nothing depends on it |
- I/O crates belong to one owner each: `cpal`/`coreaudio-*` → `mackey-audio`, `rusqlite` → `mackey-store`, `reqwest`/`ureq`/`hyper` → `mackey-sources`, `tauri*` → `mackey-app`. Pure crates also may not use async runtimes or thread pools (`tokio`, `rayon`, `crossbeam`, ...).
- `src-tauri` maps commands to core calls, owns app state behind a mutex (never touched by the audio thread), and forwards events. No music logic lives there.
- **The UI holds view state only.** Anything a test should check (which frets light up, which voicing is chosen, the `FretboardView`) is computed in Rust and sent to the UI as data. Never re-implement theory in TypeScript ([ADR 0007](docs/adr/0007-web-ui-and-ipc-boundary.md)).
- Platform-specific code lives only in `src-tauri` and the `audio-session` plugin (and audio session setup in `mackey-audio`).
- Wire dependencies explicitly in `src-tauri`; no global singletons, `lazy_static`/`OnceLock` state or hidden side effects in library crates.
- `make archcheck` (part of `make check` and CI) enforces the crate table and I/O ownership from `cargo metadata`. **A new crate must be added to the workspace `members`, the layer table in `xtask/src/archcheck.rs` and the table above in the same change**, or archcheck fails. Dev-dependencies are exempt.

## Commands

Use the `Makefile`; do not invent ad-hoc invocations. The toolchain comes from `rust-toolchain.toml`.

| Command | Purpose |
|---|---|
| `make build` | `cargo build --workspace --all-targets` |
| `make test` | All unit, integration and doc tests in the workspace |
| `make props` | Property tests with `PROPTEST_CASES` (default 10 000) in release mode; CI runs it on Linux. Commit any new `proptest-regressions/` file with the fix |
| `make lint` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` (lint levels in `[workspace.lints]`, test allowances in `clippy.toml`) |
| `make fmt` | `cargo fmt --all` (config in `rustfmt.toml`) |
| `make fmt-check` | `cargo fmt --all --check`, as CI runs it |
| `make cover` | Line coverage with `cargo llvm-cov`; target **≥ 80%** overall, new code fully covered |
| `make archcheck` | Check the crate layer rules above (`cargo xtask archcheck`) |
| `make doccheck` | Check that every `.PHONY` target has a row in this table and every folder has a `README.md` (`cargo xtask doccheck`) |
| `make deny` | `cargo deny check`: licences (permissive only), advisories, bans, sources (`deny.toml`) |
| `make tools` | One-time install of `cargo-deny` and `cargo-llvm-cov` |
| `make check` | fmt + lint + archcheck + doccheck + deny + test: must pass before every commit |

UI and app targets (`ui-lint`, `ui-test`, `e2e`, `bindings`, `app`, `ios-sim`) are added with the Tauri scaffold in roadmap phase P0; add each to this table when it lands.

## Development: TDD

1. Write a failing test first, then the minimal code to pass, then refactor (red-green-refactor).
2. **Table-driven tests** for theory and parsers: a `const CASES: &[(...)]` (or array) of inputs and expected outputs, iterated with a label in the assertion message so a failure names its case (for example every chord quality in all 12 keys, spelled correctly).
3. **Property tests (`proptest`)** for every invariant the theory engine promises: transpose up then down is the identity; interval add then subtract returns the start; a chord's pitch classes are the same in every voicing; spelling matches the key's accidentals; voicing search never exceeds the span limit or drops the root or third. Write `Arbitrary`/strategy functions next to the type in a `#[cfg(test)]` module (or behind a `proptest` feature if other crates need them). Commit `proptest-regressions/` files.
4. **Snapshot tests (`insta`)** for parser output: chord sheets, ChordPro and practice notation in; normalised output, parsed model and diagnostics out. Review changes with `cargo insta review`; never accept snapshots blindly. Fixtures live in the crate's `tests/fixtures/` and must be written for the project or public domain.
5. **Benchmarks (`criterion`)** guard the voicing search and parsers against the performance budgets in `docs/prd/08-nfr.md`.
6. Unit tests live in a `#[cfg(test)] mod tests` beside the code; cross-module tests go in the crate's `tests/`. Prefer fakes (a hand-written struct implementing the trait) over mocking frameworks.
7. Every bug fix starts with a regression test.
8. Run `make check` before any commit.

## Rust toolchain and conventions

- Use the latest stable Rust (currently **1.100**) and the **2024 edition**. `rust-toolchain.toml` is the single source; keep `rust-version` in `Cargo.toml` in step, and bump both when a new stable ships.
- One Cargo workspace. Shared metadata in `[workspace.package]`, every third-party version once in `[workspace.dependencies]` (crates use `dep = { workspace = true }`), lints once in `[workspace.lints]` (each crate sets `[lints] workspace = true`). Commit `Cargo.lock`.
- Crate names are `mackey-<domain>`, kebab-case, under `crates/`. Keep crates small and named for what they provide; no `utils`/`common`/`helpers` crates or modules.
- Format with `rustfmt` (`rustfmt.toml`). Lint with `clippy` at `-D warnings`, `pedantic` on. Fix findings; an `#[allow(...)]` or `#[expect(...)]` needs a comment saying why, and prefer `#[expect]` so a stale allowance is noticed.
- `unsafe_code` is forbidden workspace-wide. If `mackey-audio` or the iOS plugin bridge ever needs `unsafe`, that crate overrides the lint, documents every block with a `// SAFETY:` comment, and the decision gets an ADR.
- Public items need doc comments (`missing_docs`); include an example in the docs for public parsing and theory functions (doc tests run in `make test`).
- Prefer strong types over primitives: `PitchClass`, `Interval`, `Fret`, `StringIndex`, `Bpm`, not bare `u8`/`f32`. Make invalid states unrepresentable; validate at construction (`TryFrom`, `FromStr`).
- Types that cross into the UI derive `serde::Serialize`/`Deserialize` and generated TypeScript bindings (tauri-specta, or ts-rs). Keep them in the core crates, not in `src-tauri`.
- Prefer iterators and borrowing to cloning; `#[must_use]` on pure functions whose result matters is welcome but not required.

### Error handling

- **Library crates** define their own error enums with `thiserror` (one per crate, or per module where the failure modes differ, for example `ChordParseError`). Variants carry context (the input span, the offending token) rather than strings. Never expose another crate's error type in a public signature without wrapping it (`#[from]` on a variant is fine).
- **No panics in library code**: no `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!` or unchecked indexing outside tests (clippy enforces most of this). Return `Result`. A truly impossible case may use `unreachable!` with a comment proving it.
- Parsers report **diagnostics**, not just the first error: return the parsed value plus a list of positioned warnings/errors so the UI can underline them.
- `anyhow` is allowed only in `xtask`, build scripts and tests. `src-tauri` commands return a serialisable error type (`Result<T, AppError>`) that the UI can render; log the full chain with `tracing`, show the user a plain message.
- Add context when propagating across a boundary (`map_err` into a variant that names the operation), the Rust equivalent of `fmt.Errorf("doing x: %w", err)`.

### Real-time audio rules (`mackey-audio`)

- The cpal output callback is the clock. It never allocates, locks, blocks, logs or does I/O. Messages arrive through SPSC ring buffers (`rtrb`); synth voices and patterns are pre-allocated.
- Analysis (tuner, onsets, chord verification) runs on its own thread from an input ring buffer, never in the callback.
- All audio analysis stays on the device ([ADR 0006](docs/adr/0006-on-device-audio-analysis.md)).

## TypeScript and UI conventions (`ui/`)

- TypeScript `strict`, React function components and hooks, Vite. ESLint and Prettier; `tsc --noEmit` must pass.
- The fretboard is one shared SVG component driven by the `FretboardView` from Rust. Every interactive element has an ARIA label; every colour cue also has a shape or label (root = square, chord tone = filled circle, scale tone = ring), and the palette is colour-blind safe.
- Call Rust only through the generated command wrappers; never hand-edit generated bindings. CI fails if the bindings are stale.
- Vitest for UI logic; Playwright (WebKit, desktop and iPad viewports) against the UI with a mocked command layer.

## Assets and licences

- Sounds must be CC0, public domain or MIT, recorded in `docs/sound-assets.md` with source and licence. Fixtures (chord sheets) must be written for the project or public domain; never commit copyrighted lyrics.
- Rust dependencies must pass `make deny` (permissive licences only). Song import follows [ADR 0004](docs/adr/0004-user-initiated-song-import.md): no scraping.

## Documentation: keep it current

Documentation is part of the change. A change is not done until the affected docs are updated in the same commit/PR.

| File | Audience | Purpose | Update when |
|---|---|---|---|
| `README.md` (root) | Everyone | Overview, quick start, links to all docs | Features, commands or setup change |
| `INTENT.md` (root) | Everyone | One-page purpose, scope and key decisions | Purpose, scope or a key decision changes |
| `docs/prd/` | Product/engineering | The PRD: scope, personas, global rules, per-feature requirements and acceptance criteria, UX flows, NFRs, traceability (requirement → crate → test → phase) | A requirement is added, changed or removed; keep `09-traceability.md` in step |
| `docs/architecture.md` | Engineers | Crate layout, domain model, boundary, algorithms, quality practice | Structure, interfaces or dependencies change |
| `docs/adr/NNNN-*.md` | Engineers | One record per architectural decision (indexed from `docs/README.md`) | Any significant decision is made or reversed (new ADR; mark the old one superseded) |
| `docs/roadmap.md` | Everyone | Phases and exit criteria | A phase starts, finishes or changes scope |
| `docs/*.md` feature designs | Engineers | Practice sessions, rhythm, learning features, platforms, song import, sound assets | That feature's design changes |
| `user-docs/` | Users | Getting started, how to use each feature | User-visible behaviour or settings change |
| `specs/` | Engineers | Active feature specs; finished ones go to `specs/archive/` | Per the specification workflow below |

## README.md at every level

- Keep a `README.md` at the repo root **and in every folder of the hierarchy** (`crates/`, each `crates/<crate>/`, `src-tauri/`, `ui/`, `xtask/`, `docs/`, `user-docs/`, `specs/`, `.github/`, `.claude/`). Inside a crate or package, the crate README covers `src/`, `tests/` and `benches/`; `docs/adr/` is indexed from `docs/README.md`.
- Each folder README states: the folder's purpose, its key files or modules, how it fits the architecture (what it depends on and what depends on it), and links to related docs. A crate README is also its `readme` in `Cargo.toml`.
- When you add, remove, move or change the responsibility of a folder or crate, update its README and its parent's README in the same change. Create the README when you create the folder. `make doccheck` fails on a missing one.
- **Read the READMEs first** when learning or navigating the codebase: start at the root `README.md`, then the README of each folder you work in. Treat them, together with `docs/`, as the source of truth for intent; if code and docs disagree, fix whichever is wrong.

## CI/CD

Every push to `main` and every pull request runs the pipeline.

| Trigger | Workflow | What it does |
|---|---|---|
| Pull request, push to `main` | `.github/workflows/ci.yml` | On Linux and macOS: `make fmt-check`, `make lint`, `make archcheck`, `make doccheck`, `make test`, `make cover`; on Linux also `make deny` and `make props` |

Rules:

- **CI must be green.** A red `main` is fixed before any other work.
- **CI mirrors `make check`.** Anything CI enforces must also be runnable locally through the `Makefile`; when you add a CI step, add or extend a make target.
- **macOS is the real target.** Linux runs only because the pure crates are portable and Linux runners are cheap; it is not platform support.
- **Not yet in place** (added in roadmap phase P0 with the Tauri scaffold): UI lint, type check, Vitest and Playwright; the stale-bindings check; `tauri build` for a universal Mac binary; `tauri ios build` for the iOS Simulator (unsigned compile check); and `release.yml`, which on a `v*` tag attaches an ad-hoc signed `.dmg` to a GitHub Release. Releases are annotated semver tags; never build or publish release artifacts by hand. Record signing or notarisation changes in an ADR.
- Keep workflow files documented in `.github/README.md`, and update it when they change.

## Git workflow

- Direct pushes to `main` are allowed for repo setup, docs and small fixes. Run `make check` before pushing, and watch CI after (see CI/CD above).
- **Implementation work goes on its own branch** (one per spec or roadmap phase, for example `feat/<YYMMDD-feature>`), committed as it progresses, with **one pull request at the end of the work** that is reviewed and merged when CI is green.
- Small, focused commits with imperative-mood messages; reference the spec directory (`specs/YYMMDD-<feature>/`) when the work belongs to one.

## Dev-Flow Skills

The `dev-flow` and `repo` plugins from the `stainedhead/shared-plugins` marketplace are enabled for this repo in `.claude/settings.json`, so they are available as `/dev-flow:<name>` and `/repo:<name>` in every session, including cloud sessions.

The following skills are available for the developer workflow. Invoke them with the listed slash command.

| Skill | Invocation | Purpose |
|---|---|---|
| `init-dev-flow` | `/init-dev-flow` | One-time repo setup; updates AGENTS.md |
| `create-prd` | `/create-prd [title]` | Interactive PRD authoring; writes to repo root |
| `review-prd` | `/review-prd [file]` | PRD quality review |
| `create-spec` | `/create-spec [prd-file]` | Spec creation from PRD; YYMMDD-prefixed directory |
| `review-spec` | `/review-spec [spec-dir]` | Spec quality review |
| `archive-spec` | `/archive-spec [spec-dir]` | Move completed spec to specs/archive/ |
| `implm-frm-prd` | `/implm-frm-prd [prd-file]` | Implement from PRD (11 steps) |
| `implm-frm-change-dtls` | `/implm-frm-change-dtls [ticket-or-desc]` | Implement from ticket or description (12 steps) |
| `implm-from-spec` | `/implm-from-spec [spec-dir]` | Full 11-step orchestrated implementation |
| `create-review` | `/create-review` | Code and design review artifact |
| `review-code` | `/review-code` | Code review of current branch |
| `write-flow-analys` | `/write-flow-analys [spec-dir]` | Process analysis report (final step of implm-from-spec) |

Repository administration (`repo` plugin): `/repo:add-branch-restricts`, `/repo:add-collabs`, `/repo:label-repo`. `/repo:init-repo` has already been applied by hand; do not re-run it over these files.

## Feature Specification Workflow

### Specs Directory Structure

All feature development uses the `specs/` directory for planning and tracking. Each feature gets its own subdirectory named `YYMMDD-<feature-name>/`.

**Directory Structure:**
```
specs/
└── YYMMDD-<feature-name>/
    ├── spec.md                  # Feature specification and requirements
    ├── status.md                # **CRITICAL**: Phase progress tracking (update after each task)
    ├── plan.md                  # Implementation plan and architecture decisions
    ├── tasks.md                 # Task breakdown and progress tracking
    ├── research.md              # Research findings, API docs, examples
    ├── data-dictionary.md       # Data structures, types, schemas
    ├── architecture.md          # System architecture and component design
    └── implementation-notes.md  # Implementation details, gotchas, decisions
```

### Progressive Documentation Build

Documents are created progressively as the feature develops:

**Phase 0: Initial Research (PRD/Feature Research)**
- Input: Product Requirement Document, RFC, or feature research
- Purpose: Understand the problem, gather requirements, identify constraints
- **Update status.md**: Mark Phase 0 as "In Progress"

**Phase 1: Specification (spec.md)**
- Define what the feature does, user requirements, acceptance criteria, goals and non-goals
- **Update status.md**: Mark Phase 0 complete, Phase 1 in progress

**Phase 2: Research & Data Modeling (research.md, data-dictionary.md)**
- Gather documentation, explore existing code, define domain entities and data structures
- **Update status.md**: Mark Phase 1 complete, Phase 2 in progress

**Phase 3: Architecture & Planning (architecture.md, plan.md)**
- Design implementation approach, identify affected crates and layers, document component architecture
- **Update status.md**: Mark Phase 2 complete, Phase 3 in progress

**Phase 4: Task Breakdown (tasks.md)**
- Break down work into concrete, testable tasks with dependencies and estimates
- **Update status.md**: Mark Phase 3 complete, Phase 4 in progress

**Phase 5: Implementation (code + implementation-notes.md)**
- Follow TDD (Red-Green-Refactor), record decisions, document edge cases
- **Update status.md**: After EACH task completion — MANDATORY

**Phase 6: Completion & Archival**
- Update product documentation, move spec to `specs/archive/`, capture lessons learned
- **Verify status.md**: Must show 100% completion before archiving

**MANDATORY**: Update `status.md` after completing each task or phase. This file is the single source of truth for progress tracking.

### Specs Workflow Rules

- **Create the spec directory** before starting any new feature work
- **Update progressively** — specs are living documents, not written once
- **Update status.md ALWAYS** after completing each task, phase, or milestone — this is MANDATORY
- **Reference from commits** — link to the spec directory in commit messages
- **Archive completed specs** — move to `specs/archive/` when 100% complete in status.md
- **Version control** — commit specs alongside code for team visibility
