# mackey-theory

Pure music theory for Mackey Guitar Tutor: notes and intervals, scales and modes, keys, chords, the chord-symbol parser, Roman numerals, chord-function analysis and display spelling. It is the bottom of the core: it depends on nothing in the workspace and every other core crate builds on it. No I/O, no threads, no global state; every function is deterministic ([ADR 0002](../../docs/adr/0002-pure-music-theory-core.md)).

Spelling is the top-severity bug class here ([INTENT.md](../../INTENT.md), PRD rule G-1): a note's name comes from its key, chord or scale context, never from a fixed table. F major has Bb, the third of D7 is F#, `A#m` stays `A#m`, and fretboard labels never show a double sharp or flat (the theory spelling is kept for the tooltip).

## Modules (`src/`)

| Module | What it provides |
|---|---|
| `pitch` | `PitchClass`, `Letter`, `Accidental`, `Note` (spelled, `FromStr`), `Pitch` (MIDI both ways, frequency), `ReferencePitch` (A4 415 to 466 Hz), `AccidentalPreference` |
| `interval` | `Interval` (validated quality and number), spelling by letter stepping (`apply`, `apply_down`), `between`, `invert`, `degree_label` |
| `scale` | `Mode` (11 modes incl. pentatonics, blues, harmonic minor), `Scale`, `Degree` |
| `key` | `Key` (offered tonics, G-1.2 to G-1.4), `KeySignature`, circle of fifths, `relative`, `enharmonic_alternative`, `KeyNotice` |
| `chord` | `ChordQuality` (21 qualities), `Chord`, `ChordTone`, `ToneRole`, `PitchClassSet` |
| `parser` | `parse_chord` (diagnostics with spans and suggestions, E-FB-1), `parse_chord_symbol` (strict) |
| `roman` | `RomanNumeral`, `roman_to_chord`, diatonic chords (`Key::diatonic_chord(s)`) |
| `analysis` | `analyze` and `ChordAnalysis::badge` ("Not in key (V/V)", "bVII, borrowed from G minor") |
| `spelling` | `Speller` (chord tone, then scale tone, then key preference; G-1.5) and `DisplayNote` (G-1.6) |
| `error` | `TheoryError` |

## Tests (`tests/`, `benches/`)

- Unit tests sit beside the code in `#[cfg(test)] mod tests`; names carry the requirement ID (`g_1_1_...`, `ac_fb_4_1_...`).
- `tests/chord_spelling.rs`: AC-P0-1, 21 qualities x 12 roots against a hand-checked table.
- `tests/chord_parser.rs`: table-driven parser cases and insta snapshots of the diagnostics (`tests/snapshots/`, fixtures in `tests/fixtures/`).
- `tests/roman.rs`, `tests/analysis.rs`: Roman numerals, diatonic chords and function analysis.
- `tests/properties.rs`: proptest invariants (AC-P0-2, AC-P0-3). `make props` raises the case count.
- `benches/parser.rs`: criterion benchmark for the parser budget (NF-2).

## Features

`ts`: derive `ts_rs::TS` on the boundary types. `cargo test -p mackey-theory --features ts` writes the TypeScript bindings (to `bindings/` in this folder by default, or the folder in `TS_RS_EXPORT_DIR`); the Tauri phase copies them to `ui/src/bindings/`.

## Fits the architecture

Depends on: nothing in the workspace (`serde`, `thiserror`, optional `ts-rs`). Depended on by: `mackey-voicing`, `mackey-rhythm`, `mackey-audio`, `mackey-practice`, `mackey-songs`, `mackey-store`, `mackey-progress`, `mackey-sources` and `mackey-app` (see the layer table in [AGENTS.md](../../AGENTS.md#architecture-pure-core-thin-shell)).

Related docs: [docs/architecture.md §3.1](../../docs/architecture.md), [PRD G-1](../../docs/prd/README.md), [spec data dictionary](../../specs/archive/261115-fretboard-foundation/data-dictionary.md), [crates/README.md](../README.md).
