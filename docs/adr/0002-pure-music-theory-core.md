# ADR 0002: Pure, I/O-free Rust crates for music theory and parsing

- Status: Proposed
- Date: 2026-11-15

## Context
Explorer, practice sessions and songs all need the same answers: how a chord is spelled in a key, where its tones sit on the neck, which voicings are playable. Getting these wrong is the fastest way to lose a learner's trust. The same logic must run on desktop now and on mobile later.

## Decision
Put music theory (`mackey-theory`), voicing search (`mackey-voicing`), the practice-notation parser (`mackey-practice`) and song parsing (`mackey-songs`) in separate crates in one Cargo workspace. These crates do no I/O, spawn no threads, keep no global state and are deterministic. Feature crates depend on the theory crates; theory crates never depend on features, the app shell or the UI. Types that cross into the UI derive serde and generated TypeScript bindings.

## Consequences
- The core can be tested exhaustively and fast (unit, property and snapshot tests), benchmarked, and reused on mobile or in a CLI (for example a `mackey` command that checks a chord sheet).
- Layer rules are documented in `CLAUDE.md`, and can be enforced in CI with a dependency check (for example `cargo deny` bans plus a simple crate-graph test).
