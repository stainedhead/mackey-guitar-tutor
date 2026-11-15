# mackey-voicing

Pure fretboard logic for Mackey Guitar Tutor: tunings, capo and handedness, chord voicings (curated open and CAGED shapes plus a bounded search), fingering and barres, triads, arpeggios in position, scale patterns, the selection model with its keyboard cycling, and the `FretboardView` the UI draws. It depends only on `mackey-theory`. No I/O, no threads, no global state; every function is deterministic. The UI never re-implements any of this ([ADR 0007](../../docs/adr/0007-web-ui-and-ipc-boundary.md)).

Conventions: string numbers run 1 to 6 with 1 the high E; arrays are low to high. Frets are absolute from the nut; an open string rings at the capo fret.

## Modules (`src/`)

| Module | What it provides |
|---|---|
| `instrument` | `StringNumber`, `Fret`, `Capo`, `FretCount`, `TuningPreset`, `Tuning`, `Instrument`, `Handedness`, `StringSet`, `Inversion` (FR-013) |
| `voicing` | `Voicing`, `StringState`, `Fingering`, `Barre`, `VoicingOptions`, `VoicingResult` (with the "no playable voicing" relax options) |
| `shapes` | Curated open shapes, CAGED templates and Drop D shapes, verified against the real tuning and capo (FR-014) |
| `rules`, `search`, `fingering` | FB-5.4 rules and scoring, `voicings_for`, `assign_fingering` (FR-015) |
| `triads` | `triads`, `triad_ladder`, `triad_note` (FR-016) |
| `arpeggio` | `arpeggio`, `arpeggio_positions`, `default_arpeggio_position` (FR-017) |
| `patterns` | `scale_patterns`, `full_neck`, `scale_for_chord` (FR-018) |
| `selection` | `Selection`, `Action`, `apply`, `instrument_changed`, `VoicingCache`, VoiceOver text (FR-020) |
| `view` | `FretboardView` and `fretboard_view` (FR-019) |

## Entry points for the app

`fretboard_view_cached(selection, instrument, handedness, request, cache)` builds the view; `apply_cached(selection, ctx, action, cache)` handles every key press and click; `instrument_changed_cached` handles a tuning, capo or fret-count change.

## Tests (`tests/`, `benches/`)

- Tests carry requirement IDs in their names (`ac_fb_6_1_...`, `fb_5_4_...`).
- `tests/common`: an independent FB-5.4 checker used by the property tests.
- `tests/curated_shapes.rs`, `voicings.rs`, `tunings.rs`, `triads.rs`, `arpeggios.rs`: hand-verified tables and properties.
- `benches/search.rs`, `benches/view.rs`: criterion benchmarks for the NF-2 budgets (search at most 5 ms, view at most 2 ms).

## Features

`ts`: derive `ts_rs::TS` on the boundary types (also enables `mackey-theory/ts`).

## Fits the architecture

Depends on: `mackey-theory` (plus `serde`, `thiserror`, optional `ts-rs`). Depended on by: `mackey-practice`, `mackey-songs`, `mackey-store`, `mackey-progress`, `mackey-sources` and `mackey-app` (see the layer table in [AGENTS.md](../../AGENTS.md#architecture-pure-core-thin-shell)).

Related docs: [docs/architecture.md](../../docs/architecture.md), [PRD fretboard](../../docs/prd/01-fretboard.md), [spec](../../specs/261115-fretboard-foundation/spec.md), [crates/README.md](../README.md).
