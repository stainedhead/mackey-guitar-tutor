# Tasks: Fretboard Foundation

**Feature:** fretboard-foundation
**Created:** 2026-11-15
**Status:** Ready for implementation (reviewed in Step 2)
**Spec:** [spec.md](spec.md) · **Plan:** [plan.md](plan.md) · **Status tracker:** [status.md](status.md)

## Progress Summary

36/61 tasks complete. Update this file and `status.md` after every task (mandatory).

| Phase | Tasks | Done |
|-------|-------|------|
| A Workspace scaffold and `mackey-theory` | A1 to A11 (11) | 11 |
| B `mackey-voicing` | B1 to B16 (16) | 16 |
| C `mackey-audio` | C1 to C9 (9) | 9 |
| D `src-tauri` and `ui/` | D1 to D16 (16) | 0 |
| E Makefile, AGENTS.md, CI, docs | E1 to E9 (9) | 0 |
| Total | 61 | 36 |

## Conventions

- **Estimate:** S = under 2 hours, M = half a day, L = a day, XL = more than a day (split if you can).
- **TDD:** each task starts with failing tests (names carry the requirement ID, for example `fb_5_4_no_voicing_exceeds_span`, `ac_fb_6_1_c_234_first_inversion`). Table-driven (`const CASES`), proptest for invariants, insta for parser and view snapshots; `cargo insta review`, never accept blindly.
- **Done means:** `make check` green (and `make ui-lint ui-test` once they exist), docs and READMEs updated in the same commit, tasks and status updated, no `#[allow]` without a reason (prefer `#[expect]`).
- **Branch and commits:** work on the run's branch; small imperative commits referencing `specs/261115-fretboard-foundation/`.
- "AC-..." are the product PRD's criteria listed in [spec.md](spec.md#acceptance-criteria).

---

## Phase A: Workspace scaffold and `mackey-theory`

### A1. Cargo scaffold for the three crates and the app crate stub
- **Status:** Done
- **Depends on:** none · **Est:** M · **FR:** FR-001
- **Files:** `Cargo.toml` (members, `[workspace.dependencies]`: `serde`, `thiserror`, `proptest`, `insta`, `criterion`, `ts-rs`; `[profile.dev.package.proptest|insta|similar] opt-level = 3`), `crates/mackey-theory/{Cargo.toml,src/lib.rs,README.md}`, `crates/mackey-voicing/...`, `crates/mackey-audio/...` (each lib with crate docs, `[lints] workspace = true`, feature `ts`), `crates/README.md` (crate table), root `README.md`.
- **Do not add** `src-tauri` to `members` yet (D1 does it, so Phase A–C CI stays green without Tauri system packages).
- **Acceptance (SAC-01):** `make check` passes with empty crates; `make archcheck` passes; `crates/README.md` lists all three; each crate README states purpose, modules, dependencies in and out; `cargo deny check` still passes.
- **Tests:** none beyond the existing xtask tests; a doc test in each `lib.rs`.

### A2. `mackey-theory`: pitch classes, notes, pitches
- **Status:** Done
- **Depends on:** A1 · **Est:** M · **FR:** FR-007
- **Files:** `src/pitch.rs`, `src/error.rs`, `tests/properties.rs` (start).
- **Acceptance:** `PitchClass`, `Letter`, `Accidental`, `Note` (`FromStr` accepts "F#", "Bb", "C##", rejects "H", "", "Fb#b"), `Pitch` (MIDI both ways, A4 = 69, C4 = 60, B#3 = 60, Cb4 = 59; frequency with a reference A4; octave and MIDI range errors), `ReferencePitch` (415..=466), `AccidentalPreference`. All constructors validate.
- **Tests:** table-driven `CASES` for names, MIDI and frequency (A4 440 → 440.0, C4 → 261.6256, A4 442 case); proptest: `Pitch::from_midi(m).midi() == m`; `PitchClass::transpose(n).transpose(-n)` is identity.

### A3. `mackey-theory`: intervals
- **Status:** Done
- **Depends on:** A2 · **Est:** M · **FR:** FR-007
- **Files:** `src/interval.rs`.
- **Acceptance:** `Interval::new` rejects invalid quality/number pairs; `semitones`, `apply` (letter stepping), `between`, `invert`, `degree_label` ("R", "b3", "3", "5", "b7"); applying M3 to Db gives F, to F# gives A#, d5 to B gives F.
- **Tests:** table of (note, interval, expected note) including double accidentals; proptest (AC-P0-3): `apply(i)` then `apply(i.invert())` of the octave complement returns the start pitch class; for any note and any valid interval the semitone distance equals `semitones()`.

### A4. `mackey-theory`: scales and modes
- **Status:** Done
- **Depends on:** A3 · **Est:** M · **FR:** FR-008
- **Files:** `src/scale.rs`.
- **Acceptance:** all 11 modes in `Mode`, degrees spelled for any offered tonic (G Dorian = G A Bb C D E F; F Lydian = F G A B C D E; E harmonic minor = E F# G A B C D#; blues = C Eb F Gb G Bb); `degree_of`, `contains`.
- **Tests:** `CASES` of (tonic, mode, expected spelling) for every mode in at least 5 tonics (sharp, flat and natural); property: degrees of a 7-note scale use each letter exactly once; pentatonic and blues use distinct letters.

### A5. `mackey-theory`: keys, signatures, circle of fifths
- **Status:** Done
- **Depends on:** A4 · **Est:** L · **FR:** FR-009 · **SAC-05**
- **Files:** `src/key.rs`.
- **Acceptance:** offered major tonics C G D A E B F#/Gb F Bb Eb Ab Db and minor tonics A E B F# C# G# D#/Eb D G C F Bb (G-1.2, G-1.3); C# major and Cb major rejected; `Key::new_normalised` maps D# Dorian → Eb Dorian with a notice (G-1.4); `signature`; `next_fifth`/`prev_fifth` wrap through all 12 for major and minor honouring the chosen F#/Gb spelling; `relative` (major ↔ natural minor, `None` otherwise); `enharmonic_alternative` is `Some` only for keys with 6 accidentals (E-FB-7); `degree_of`.
- **Tests:** table of every offered key with its signature and spelled scale (hand-written); property (AC-P0-3 part 3): the accidentals appearing in a key's scale spelling are all of one kind (all sharps or all flats) and match the signature sign; `next_fifth` applied 12 times returns to the start.

### A6. `mackey-theory`: chord qualities and chords
- **Status:** Done
- **Depends on:** A3 · **Est:** M · **FR:** FR-010
- **Files:** `src/chord.rs`.
- **Acceptance:** all 21 qualities with intervals, symbols, required tones, roles; `Chord::tones` spells by interval from the root (D7 → D F# A C, Bbmaj7 → Bb D F A, A#m keeps A#, G#dim7 → G# B D F); slash bass; `symbol()` canonical text.
- **Tests:** unit tests for formula sanity; this feeds A8.

### A7. `mackey-theory`: spelling (G-1.5, G-1.6)
- **Status:** Done
- **Depends on:** A5, A6 · **Est:** M · **FR:** FR-012
- **Files:** `src/spelling.rs`.
- **Acceptance:** `Speller` resolves chord tone, then scale tone, then key preference; display simplification removes double accidentals (Fx → G) and reports the theory spelling; AC-FB-2.1 data (F major notes F G A Bb C D E, no A#); in C major with no context the pitch class 10 is Bb (flats for C), in G major it is A# only if chromatic with no chord context (sharp key); the third of D7 in C is F#; the root of bVII in C is Bb.
- **Tests:** table-driven cases for every key × all 12 pitch classes with no context (expected sharps/flats hand-written); property: no `Speller::display` result has a double accidental; the pitch class is preserved.

### A8. `mackey-theory`: chord-symbol parser with diagnostics
- **Status:** Done
- **Depends on:** A6 · **Est:** L · **FR:** FR-010
- **Files:** `src/parser.rs`, `tests/chord_parser.rs`, `tests/fixtures/chord_symbols.txt` (one symbol per line, written for the project), `tests/snapshots/`.
- **Acceptance (AC-P0-2, E-FB-1):** accepts every quality and alias in the data dictionary grammar with every root (including accidentals and Unicode `♯ ♭`) and an optional slash bass; the player's root spelling is kept (`A#m`, `Bbm`); `Hm7` → error with suggestion `Bm7`; extended symbols outside the FB-4.3 grammar (`Cmaj13#11`, `C#maj13#11b9`, `C7b9`) return the E-FB-1 error with a positioned span and no suggestion (spec Q-9 defers them; log the deviation from E-FB-2 in `implementation-notes.md`); `Csus` = sus4; distance-≤2 suggestions (`Cmaj77` → `Cmaj7`, `Amj7` → `Amaj7`); empty or whitespace-only input → `Empty`; trailing garbage → `TrailingInput` with its span; `parse_chord` returns diagnostics (all of them, positioned), `parse_chord_symbol` the strict `Result`.
- **Tests:** table-driven `CASES` (input → expected symbol or error kind and span); insta snapshots of `ParseOutcome` for a fixture list including 20 invalid inputs; proptest: `parse_chord(any_string)` never panics (strategy: arbitrary `String`, plus strings from a symbol-biased alphabet, plus non-ASCII); proptest: for every generated valid chord, `parse(chord.symbol()) == chord`.

### A9. `mackey-theory`: diatonic chords, Roman numerals, analysis
- **Status:** Done
- **Depends on:** A5, A6, A8 · **Est:** L · **FR:** FR-011 · **SAC-06**
- **Files:** `src/roman.rs`, `src/analysis.rs`, `tests/roman.rs`, `tests/analysis.rs`.
- **Acceptance:** `diatonic_chords` for all 7 modes in major and minor keys with triads and sevenths (Imaj7 ii7 iii7 IVmaj7 V7 vi7 viiø7 in major; natural minor and the other modes use their own qualities); `RomanNumeral` `Display`/`FromStr` round trip; `roman_to_chord("vi", G)` = Em; `bVII` in C = Bb; `V7/V` in C = D7; `analyze(C, D7)` = not in key, `SecondaryDominant { target: V }`, badge "Not in key (V/V)" (AC-FB-4.1); `analyze(C, Bb)` = borrowed `bVII` "borrowed from C minor" (match the PRD example wording for G in the test: "bVII, borrowed from G minor"); in-key chords return `in_key: true` and no badge.
- **Tests:** tables for all 12 major keys × 7 degrees × (triad, seventh); property: every `diatonic_chord`'s tones are scale tones of the key.

### A10. `mackey-theory`: AC-P0-1 spelling table and benchmarks
- **Status:** Done
- **Depends on:** A7, A8 · **Est:** M · **FR:** FR-010, FR-012
- **Files:** `tests/chord_spelling.rs` (hand-written `CASES`: for each of the 21 qualities, the full expected spelling in at least all 12 chromatic roots using the root spellings sharp-or-flat offered by the key pickers — C, Db, D, Eb, E, F, F#, G, Ab, A, Bb, B = 12 keys), `benches/parser.rs` (criterion: parse 1,000 symbols; NF-2 parser budget ≤ 1 ms per 1,000 characters).
- **Acceptance (AC-P0-1):** 12 roots × 21 qualities = 252 rows, hand-written (not generated from the code under test), each failing with a label naming root and quality; benchmark committed and within budget on the dev machine.

### A11. `mackey-theory`: ts feature, docs, coverage gate for the crate
- **Status:** Done
- **Depends on:** A9, A10 · **Est:** S · **FR:** FR-003, FR-007..012
- **Files:** `#[cfg_attr(feature = "ts", derive(ts_rs::TS))]` on boundary types listed in the data dictionary; `README.md` (modules, depends on nothing, depended on by voicing/audio/app); doc examples on every public parsing and theory function.
- **Acceptance:** `cargo test -p mackey-theory --features ts` exports TypeScript files into a temp dir without error; `make check` green; crate line coverage 100% of new code (checked with `cargo llvm-cov -p mackey-theory`); `make props` passes with 10,000 cases; commit any `proptest-regressions/`.

---

## Phase B: `mackey-voicing`

### B1. Instrument: strings, frets, tunings, capo, handedness
- **Status:** Done
- **Depends on:** A11 · **Est:** M · **FR:** FR-013 · **SAC-07**
- **Files:** `crates/mackey-voicing/src/instrument.rs`, `tests/tunings.rs`.
- **Acceptance:** `StringNumber`, `Fret`, `Capo` (0..=12), `FretCount` (19..=24), nine presets with the exact pitches of LH-2 (Open D = D2 A2 D3 F#3 A3 D4, Open E = E2 B2 E3 G#3 B3 E4, Half-step down = Eb2 Ab2 Db3 Gb3 Bb3 Eb4, etc.), `Tuning::custom` with `UnusualTension` warnings outside "minor third below to whole tone above" the standard pitch of that string (boundaries inclusive: -3 and +2 semitones are fine, -4 and +3 warn), duplicate open strings allowed, `shift_from_standard`, `Instrument::pitch_at` (error below capo), `open_sounding_pitch` with capo.
- **Tests:** table of every preset's six pitches; open-string sounding names with capo 2 (AC-TU-5.1 data F#2 B2 E3 A3 C#4 F#4); proptest: `pitch_at(s, f+1) == pitch_at(s, f)` + 1 semitone; boundary cases for warnings.

### B2. Curated open-shape library (FB-5.3)
- **Status:** Done
- **Depends on:** B1 · **Est:** L · **FR:** FR-014 · **SAC-08**
- **Files:** `src/shapes/open.rs`, `src/shapes/mod.rs`.
- **Acceptance:** `const` tables for C, A, G, E, D, Am, Em, Dm, A7, B7, C7, D7, E7, G7, Amaj7, Cmaj7, Dmaj7, Fmaj7, Am7, Dm7, Em7, Asus2, Asus4, Dsus2, Dsus4, Esus4 and F (small `xx3211` and full barre `133211`) with strings, fingers, names ("Open C"). Hand-written from teaching charts (C `x32010` fingers `x-3-2-0-1-0`).
- **Tests:** `fb_5_3_curated_shapes_exact` (hand-written expected fret strings for all 27 shapes + 2 F); consistency test that each shape's sounding pitch classes equal its chord's tones and the shape obeys FB-5.4 (root and third or sus note present, span ≤ 4, ≤ 4 fingers); where a standard teaching shape legitimately omits the fifth the test allows it (fifth optional).

### B3. CAGED barre shapes, Half/Whole-step-down reuse, Drop D entries
- **Status:** Done
- **Depends on:** B2 · **Est:** L · **FR:** FR-014
- **Files:** `src/shapes/caged.rs`, `src/shapes/drop_d.rs`, `src/shapes/mod.rs` (lookup by chord, tuning shift, capo).
- **Acceptance:** movable C, A, G, E, D barre shapes (major and minor where a standard form exists: E-shape, A-shape, C-shape and D-shape forms, G-shape) transposable up the neck to any root with its fret; Half-step down and Whole-step down reuse the standard shapes with the chord transposed by the tuning shift (Eb in Half-step down = the open E shape); Drop D entries (six-string D `000232`... verify pitch classes in test; Dm, D7, Dsus2, Dsus4, Dmaj7 with string 6 open) for AC-LH-4.1; capo translation (relative → absolute).
- **Tests:** property over all roots and shapes: transposed shape pitch classes equal the chord's; table for AC-LH-4.1 (Drop D, D: list includes a six-string D with string 6 open (D), every pitch correct for D A D G B E); Half-step-down table (Eb, Ab, Db open shapes).

### B4. Fingering assignment and barre detection
- **Status:** Done
- **Depends on:** B1 · **Est:** M · **FR:** FR-015
- **Files:** `src/fingering.rs`.
- **Acceptance:** finger numbers 1..=4 by ascending fret; barre detection (lowest fretted fret repeated on 2+ strings; 133211 → barre at fret 1 over strings 6..1... verify); a barre counts as one finger; more than 4 fingers → `None` (candidate rejected); identical open strings (E-FB-6) never double-counted; open strings → finger 0; muted → `None`; hand-written fingerings for the curated shapes are used as-is when present.
- **Tests:** table of (frets, expected fingers, expected barres) including `x32010` → `x-3-2-0-1-0`, `133211`, `xx3211`, `x02220` (A), a custom tuning with two identical open strings; property: result never exceeds 4 fingers.

### B5. Voicing search, scoring, grouping, no-voicing result
- **Status:** Done
- **Depends on:** B2, B3, B4 · **Est:** XL (split search core / scoring and grouping / curated merge if needed) · **FR:** FR-015 · **SAC-09**
- **Files:** `src/search.rs`, `tests/voicings_standard.rs`, `tests/no_voicing.rs`, `tests/capo.rs`, `tests/properties.rs`.
- **Acceptance:** `voicings_for(chord, instrument, options) -> VoicingResult` per the architecture: curated first, search fills gaps, grouped Open / Barre and CAGED / Other, sorted by score, capped; obeys FB-5.4 everywhere (root and third, sus note, root and fifth for `Power`, span ≤ 4 or 5 with stretches, ≤ 4 fingers, required bass for slash chords); `Origin::Search` marked in non-curated tunings; capo applied (frets absolute, open = capo); `NoneFound` with both relax options when no voicing exists, never a voicing that breaks FB-5.4 (AC-FB-5.6); AC-FB-5.1 (C voicing view first is `x32010`, fingers `x-3-2-0-1-0`); AC-FB-5.2 (F list contains `xx3211` and `133211`); AC-LH-5.1 labels via `CapoLabel`.
- **Tests:** table for ≥ 30 (chord, tuning, capo) → expected first voicing and presence of named shapes; proptest AC-FB-5.3: for random chords (all roots × qualities), random presets and random capo 0..=12, every returned voicing has span ≤ 4 (≤ 5 with stretches), ≤ 4 fingers, root and third (or sus note or fifth for power), and all its pitch classes belong to the chord; proptest: a chord's pitch classes are the same in every voicing found; a slash-bass chord's lowest sounding pitch class is the bass.
- **Notes:** write the benchmark harness skeleton here (B13 finishes it).

### B6. Triads on string sets
- **Status:** Done
- **Depends on:** B1, A6 · **Est:** M · **FR:** FR-016
- **Files:** `src/triads.rs`, `tests/triads.rs`.
- **Acceptance:** `triads(chord, instrument, set, inversion) -> TriadResult`: placements in fret order, one note per string on three adjacent strings; seventh chords use the root triad with a note ("Triad of Cmaj7: C E G"); `Third` inversion only for sevenths (and not in triad view; the view clamps); `Unavailable` with a reason when no placement within span 4 (FB-6.4); "show all on string set" returns the three inversions in fret order, labelled root, 1st, 2nd (AC-FB-6.2: G, set 123).
- **Tests:** AC-FB-6.1 (C, set 234, first inversion lowest placement: E s4 f2, G s3 f0, C s2 f1); AC-FB-6.2; table of all 12 roots × major/minor × 4 sets × 3 inversions in standard tuning, with pitch-class and bass checks; proptest across all presets: every returned triad has exactly the three chord tones, on adjacent strings, bass tone = the inversion's tone, span ≤ 4.

### B7. Arpeggio in position
- **Status:** Done
- **Depends on:** B1, A6 · **Est:** M · **FR:** FR-017
- **Files:** `src/arpeggio.rs`, `tests/arpeggios.rs`.
- **Acceptance:** window start `0..=20` (clamped to the fret count and capo), notes for every chord tone in `start-1..=start+4`, fingers per FB-7.2, valid positions (windows containing all required tones), default position per FB-7.4, play order for Up, Down, UpDown (UpDown does not repeat the top note), `show order` numbering. AC-FB-7.1: Am position 5 has every A, C, E between frets 4 and 9 inclusive, finger 1 on fret 5.
- **Tests:** AC-FB-7.1; table for C, G, Am, Em, Dm7 in several positions; property: every returned note is a chord tone within the window; fingers ∈ 1..=4 (0 for open); order is a permutation of the notes.

### B8. Scale patterns and follow-the-chords
- **Status:** Done
- **Depends on:** B1, A5 · **Est:** L · **FR:** FR-018
- **Files:** `src/patterns.rs`, `tests/scale_patterns.rs`.
- **Acceptance:** `scale_patterns(scale, instrument)` returns FullNeck plus five pentatonic boxes (for the two pentatonics and, as a bonus, blues returns FullNeck only) or seven 3nps patterns for the major-family modes (Ionian, Dorian, Phrygian, Lydian, Mixolydian, Aeolian, Locrian; harmonic minor and blues: FullNeck only, documented), ordered by lowest fret from the first pattern at or above the capo; box 1 has the root on string 6 under the first finger; AC-SC-3.1 (A minor pentatonic box 1: frets 5 and 8 on strings 6, 2 and 1; frets 5 and 7 on strings 5, 4 and 3; roots at s6 f5, s4 f7, s1 f5); all boxes use only scale tones and fit within 4 frets (pentatonic) or 5 (3nps); `scale_for_chord` per SC-4 (AC-SC-4.1: C → A7 in key C gives A Mixolydian; Dm in C gives C major since diatonic; Bm in C gives B Dorian... check the table in the data dictionary) and the SC-1 default for minor chords in follow mode.
- **Tests:** AC-SC-3.1, AC-SC-4.1; every box of every pentatonic root in standard tuning checked against a hand-written table for 3 roots (A, G, E open-position case); property across tunings: notes ⊆ scale pitch classes; each string has 2 (pentatonic) or 3 (3nps) notes except at the neck's ends; no duplicates.

### B9. Selection model and cycling
- **Status:** Done
- **Depends on:** B5, B6, B7, B8, A9 · **Est:** XL · **FR:** FR-020 · **SAC-11**
- **Also:** selecting a chord from focus None sets the view type to All tones; selecting a chord while a chord is already focused keeps the current view type (D-32). A change of fret count (settings) clamps `position`, `voicing_index` and the visible selection to valid values (no stale index after 24 → 19 frets).
- **Files:** `src/selection.rs`, `tests/selection.rs`.
- **Acceptance:** `Selection`, `Action`, `apply`, `Applied`, `Notice`, announcements ("E minor, first inversion, strings 2 3 4"); all FB-9 actions with wrap-around exactly as the table says (chord I..vii wraps; voicing wraps within the group then into the next; inversion wraps root, 1st, 2nd and 3rd for sevenths; string sets wrap; position moves to the next position containing the chord; view type All tones → Voicing → Triad → Arpeggio; labels cycle skips `Finger` without a voicing; `O` toggles overlay; `[`/`]` circle of fifths; `R` relative); AC-FB-2.2 (G major, IV, key D major → G selected); G-3.4 (tuning/capo change keeps the chord, reselects the best voicing with notice "Voicing changed for new tuning"); AC-E-FB-4 (capo 5, selection in position 2 → first position at or above the capo, "Moved above the capo"); invalid chord text never mutates the selection (FB-4.4); `SelectRoman` in the current key.
- **Tests:** table-driven action sequences with expected selections and announcements; property: any sequence of up to 50 random actions never panics and keeps the selection valid (indexes in range); applying `Next` n times (n = list length) returns to the start for every cycling action.

### B10. `FretboardView` builder
- **Status:** Done
- **Depends on:** B9, A7 · **Est:** XL · **FR:** FR-019 · **SAC-10**
- **Files:** `src/view.rs`, `tests/view_key_notes.rs`, `tests/view_tooltips.rs`, insta snapshots.
- **Acceptance:** all fields in the data dictionary: positions with role, labels for every mode, finger, order, tooltip, a11y label ("String 3, fret 2, A, third of F"); key-notes view for focus None (tonic Root, degrees 1/3/5/7 coloured roles, 2/4/6 ScaleTone; AC-FB-2.1: F major shows F G A Bb C D E, no A#); chord views; voicing view with open ring markers and muted markers and barres; triad and arpeggio views; overlay with chord tones solid and other scale tones `ScaleTone` (SC-2); dimmed frets below the capo and no highlights there; inlays and fret numbers (FB-1.1, FB-1.2); string labels with capo (FB-1.3); off-range counts and `fit_range` (FB-1.5, E-FB-5); `list_position` "3 of 7"; notices; badge ("Not in key (V/V)"); capo label (AC-LH-5.1); tooltip contents (AC-FB-1.6: string 3 fret 0, key G chord G → "G3", "1", "R", "Root"); no double accidentals in labels (G-1.6) with `theory_spelling` in the tooltip; `mirrored`/`flipped` echoed.
- **Tests:** AC-FB-2.1, AC-FB-4.1 (view side: tones D F# A C, badge), AC-FB-1.6, AC-LH-5.1; insta JSON snapshots for 8 named scenarios (G major notes, C open voicing, F barre, C triad 234 first inversion, Am arpeggio pos 5, A minor pentatonic box 1, Drop D D, capo 2 G shape); property: every position's fret ≥ capo and ≤ fret count, pitch of the position equals `Instrument::pitch_at`, labels never contain "x" or double accidentals.

### B11. Playback plan
- **Status:** Done
- **Depends on:** B10 · **Est:** S · **FR:** FR-024
- **Files:** `src/playback.rs`.
- **Acceptance:** strum: low to high, 30 ms apart, only sounding strings; arpeggio: view order, eighths at 70 bpm (428 ms, rounded from 428.57; delays cumulative from exact arithmetic then rounded once); key notes (focus None): ascending scale from the lowest tonic in the visible range (one octave plus tonic, 8 notes); single note; empty plan for nothing to play.
- **Tests:** table for each kind; property: delays are non-decreasing; every pitch's frequency is within 20 to 20,000 Hz.

### B12. Property-test sweep over all tunings
- **Status:** Done
- **Depends on:** B5, B6, B7, B8, B10 · **Est:** M · **FR:** FR-013..019
- **Files:** `tests/properties.rs`; strategies in a `#[cfg(test)] mod strategies` (or `proptest` feature if other crates need them).
- **Acceptance:** one proptest each for: voicings (AC-FB-5.3), triads, arpeggios, patterns, `FretboardView` consistency, for random chord × random preset × random custom tunings within allowed ranges × capo 0..=12 × frets 19..=24 × both handedness; custom tunings with two identical open strings do not double-count fingers (E-FB-6); commit `proptest-regressions/` if any appear; `make props` passes in release.

### B13. Criterion benchmarks and release-mode timing tests (AC-FB-9.1a)
- **Status:** Done
- **Depends on:** B5, B10 · **Est:** M · **FR:** FR-015, FR-019
- **Files:** `benches/search.rs`, `benches/view.rs`, `tests/perf_budget.rs`.
- **Acceptance:** benches for voicing search (one chord, full neck, standard tuning, 22 frets) and `fretboard_view` (all view types); `tests/perf_budget.rs` runs each 200 times with `std::time::Instant` and asserts p95 ≤ 5 ms and ≤ 2 ms **only when `cfg!(not(debug_assertions))`** (so `make props`, which uses `--release`, enforces it; the debug `make test` skips it with a comment). If the budget fails, optimise before continuing (see the risk table in spec.md).
- **Tests:** the timing test itself; bench results recorded in `implementation-notes.md`.

### B14. `mackey-voicing`: ts feature, README, docs, coverage
- **Status:** Done
- **Depends on:** B10, B11 · **Est:** S · **FR:** FR-003, FR-019
- **Files:** `#[cfg_attr(feature = "ts", derive(ts_rs::TS))]` on every boundary type; `README.md`; doc examples on public functions.
- **Acceptance:** `cargo test -p mackey-voicing --features ts` exports without error; coverage of new code at or near 100% by `cargo llvm-cov -p mackey-voicing`.

### B15. Traceability: test names carry requirement IDs
- **Status:** Done
- **Depends on:** B14 · **Est:** S · **FR:** all
- **Files:** `docs/prd/09-traceability.md` rows (Step 4 finalises); a grep check in the task notes that every AC in spec.md has at least one test whose name contains its ID in snake_case (`ac_fb_6_1`, `ac_sc_3_1`).
- **Acceptance:** a script-free check recorded in `implementation-notes.md` listing AC → test name for Phases A and B.

### B16. Spelling-in-views regression table (INTENT top-severity bug class)
- **Status:** Done
- **Depends on:** B10 · **Est:** M · **FR:** FR-012, FR-019
- **Files:** `tests/view_spelling.rs`.
- **Acceptance:** hand-written table: for 12 keys × 3 views (key notes, chord, scale overlay) the full set of expected labels per string-fret on strings 6 and 1 over frets 0..=12 (open-position and 12-fret check), proving no `A#` in F, no `Gb` in D, `F#` third of D7 in C, `Bb` root of bVII in C.

---

## Phase C: `mackey-audio`

### C1. Crate skeleton, dependencies and error type
- **Status:** Done
- **Depends on:** A1 · **Est:** S · **FR:** FR-001, FR-024, FR-026
- **Files:** `crates/mackey-audio/Cargo.toml` (`cpal`, `rtrb`, `tracing`, `thiserror`, `serde`; dev: `proptest`, `criterion`, `assert_no_alloc`), `src/{lib.rs,error.rs,dsp/mod.rs,io/mod.rs}`.
- **Acceptance:** archcheck passes (`cpal` only here, depends on theory only); `make deny` passes with the new dependencies (extend `deny.toml` only per Q-6); on Linux CI, ALSA headers are required: task E4 installs them, and until then the crate compiles locally with `libasound2-dev` (document in README).

### C2. MPM pitch detector (pure)
- **Status:** Done
- **Depends on:** C1, A2 · **Est:** L · **FR:** FR-026
- **Files:** `src/dsp/pitch.rs`, `src/dsp/cents.rs`, `tests/common/signals.rs` (synthetic signal generators), `tests/tuner_fixtures.rs` (detector part).
- **Acceptance:** `PitchDetector::detect` over a 2048-sample window for 60..=1400 Hz; sub-sample interpolation; returns `None` for silence, noise and for clarity under the threshold; `CentsOffset`, `nearest_pitch` (A4 reference); the synthetic generators produce (a) pure sines, (b) guitar-like tones (fundamental plus 6 harmonics with 1/n amplitude, exponential decay, tiny inharmonicity `f_n = n f0 sqrt(1 + B n^2)` with B ≈ 1e-4, plus −50 dBFS white noise from a seeded PRNG).
- **Tests:** AC-TU-1.1 data (440.0 Hz, 0 ± 1 cent), AC-TU-1.3 (73.42 Hz → D2), AC-TU-3.1 (A4 = 442, 440 Hz → about −8 cents); AC-TU-1.2 for the six open strings of standard tuning at −20, −5, +5, +20 cents (24 cases) within ±3 cents; table of sample rates 44,100 and 48,000; proptest: for any frequency in 70..=1300 Hz and sample rate in {44100, 48000}, a pure sine is detected within 1 cent; the detector never panics on arbitrary `f32` slices of any length ≥ 1 (including NaN, ±inf: returns `None`).

### C3. Level meter and tuner state machine (pure)
- **Status:** Done
- **Depends on:** C2 · **Est:** L · **FR:** FR-026, FR-027 · **SAC-15**
- **Files:** `src/dsp/level.rs`, `src/dsp/tuner.rs`, `tests/tuner_strings.rs`.
- **Acceptance:** `Tuner::push_block` produces `TunerReading`s per hop with: median-of-3 cents smoothing (resets on pitch change), in tune within ±3 cents held for 300 ms, `NoStablePitch` after 500 ms without a pitch, `TooQuiet` after 3 s of low level, `TooLoud` on clipping, `LowConfidence` for two-string chords (strongest pitch kept when confidence is adequate), string mode (auto-select nearest target by cents; tap-to-lock; `Tighten`/`Loosen` direction; `far_from_any` beyond 3 semitones; per-string ticks), chromatic mode; capo handled by the app through `TunerTarget`s; A4 reference.
- **Tests:** AC-TU-2.1 (DADGAD, 146.83 Hz → string 4, in tune), AC-TU-5.1 (targets from `Instrument` pitch list F#2 B2 E3 A3 C#4 F#4, built in test by transposing the standard pitches by 2 semitones), AC-NF-9 (first reading within 150 ms of onset: feed 20 ms of silence then a tone block-wise at 48 kHz; the first `Pitch` reading's `t_ms` − onset ≤ 150; ≥ 20 readings per second of tone), AC-TU-1.1 "In tune" within 400 ms of simulated input (budget: first reading when 2,048 samples are available, hold timer from the first in-tune reading, so about 343 ms; the test asserts ≤ 400 ms at 44.1 and 48 kHz and block sizes 256, 512 and 1,024 to cover device buffer sizes); table-driven status transitions with sample counters; E-TU-6 two-tone input (110 Hz + 165 Hz at similar level) → either strongest pitch or `LowConfidence`, never a wrong in-tune.

### C4. Karplus-Strong synth (pure) and the real-time rules test
- **Status:** Done
- **Depends on:** C1 · **Est:** L · **FR:** FR-024
- **Files:** `src/dsp/synth.rs`, `tests/synth.rs`.
- **Acceptance:** `Synth::new(sr, voices)` preallocates; `apply(SynthCommand)`, `render(&mut [f32])`; `NoteOn` with `delay_samples`; voice stealing; mute and volume (0..=1 linear from percent); `ReferenceToneOn` loops with re-pluck every 1.5 s until off (TU-4); output peak ≤ 1.0 for 16 simultaneous voices (soft limiter); decay to silence; no NaN.
- **Tests:** `render` under `assert_no_alloc::assert_no_alloc(|| ...)` in debug builds (NF-8.1) with a custom allocator test binary (`#[global_allocator] static A: AllocDisabler`, `harness = true` in its own integration test file so the allocator does not affect other tests); pitch accuracy: render a 440 Hz and an 82.41 Hz note and run the detector (C2) on the output: within 3 cents; strum timing: 6 notes at 30 ms offsets start at the right sample (sample-exact onsets within 1 sample, measured by first non-zero sample).

### C5. Cpal output glue and device-absent path
- **Status:** Done
- **Depends on:** C4 · **Est:** M · **FR:** FR-024
- **Files:** `src/io/output.rs`, `src/io/devices.rs`.
- **Acceptance:** `OutputHandle::start()` (default output, f32, 44.1/48 kHz, mono duplicated across channels) with the callback draining the `rtrb` queue then `Synth::render`; `NoOutputDevice` when none; queue-full drop counter; no logging or allocation in the callback; tracing for open/close; stream error callback sets an atomic flag the app polls.
- **Tests:** unit tests against a `trait OutputBackend` fake (the glue's testable part: command draining into the synth; device selection logic from a list); on CI Linux without a sound card the real `start()` must return `Err(NoOutputDevice)` or succeed, never panic (test accepts both).

### C6. Cpal input glue, analysis thread, device list, permission handling
- **Status:** Done
- **Depends on:** C3, C5 · **Est:** L · **FR:** FR-027
- **Files:** `src/io/input.rs`, `src/io/devices.rs`.
- **Acceptance:** `list_input_devices()`; `InputHandle::start(device: Option<&str>, cfg, on_reading) -> Result<InputHandle, AudioError>`; callback copies into the ring (mono mixdown), the analysis thread consumes hop-sized blocks and calls `Tuner::push_block`; rate limit to about 30 Hz handled by the app (C6 provides all readings, the app throttles); device removal → fallback to default with `TunerNotice::DeviceDisconnectedUsingDefault` (E-TU-5); permission: stream-build failure or one second of exact zeros while the OS reports the stream running → `AudioError::PermissionDenied` (Q-5); `stop()` joins the thread; `start` while running stops the previous handle, `stop` without `start` is a no-op; the analysis thread body runs under `catch_unwind` and a panic yields one `TunerNotice::AnalysisStopped` reading; audio never written anywhere.
- **Tests:** unit tests on the pure parts via fakes: ring buffer → block assembly → tuner, mono mixdown, fallback decision function, zero-input permission heuristic; no real device needed in CI.

### C7. Tuner target helper and A4/string utilities
- **Status:** Done
- **Depends on:** C3 · **Est:** S · **FR:** FR-027
- **Files:** `src/dsp/tuner.rs` (`targets_from_pitches(&[Pitch;6], a4) -> Vec<TunerTarget>`).
- **Acceptance:** targets carry string number 1..=6 (index 0 = string 6), pitch, frequency from `ReferencePitch`; used by the app (D5).
- **Tests:** table for standard, DADGAD, capo 2, A4 = 442.

### C8. Benchmarks and coverage for `mackey-audio`
- **Status:** Done
- **Depends on:** C3, C4 · **Est:** S · **FR:** FR-026
- **Files:** `benches/pitch.rs`; `Makefile`'s cover target (E7) excludes `crates/mackey-audio/src/io/` with a comment.
- **Acceptance:** detector cost per window recorded in `implementation-notes.md` (budget: ≤ 2 ms per 2048-sample window on the dev machine so the thread keeps up at 94 windows/s); `dsp` module coverage 100% of new code.

### C9. `mackey-audio`: ts feature, README, docs
- **Status:** Done
- **Depends on:** C7 · **Est:** S · **FR:** FR-003
- **Files:** `#[cfg_attr(feature = "ts", derive(ts_rs::TS))]` on `TunerReading` and friends, `InputDevice`, `MicPermission`; README (modules, depends on theory, depended on by app; the real-time rules; the `io` exclusion).
- **Acceptance:** `cargo test -p mackey-audio --features ts` exports; `make check` green.

---

## Phase D: `src-tauri` and `ui/`

### D1. Tauri 2 scaffold, config, capabilities, command registry
- **Progress:** Scaffold done (infrastructure worker): `mackey-app` builds on Linux, config, capabilities, plists, icons, `app_info` command; the command registry is the `COMMAND_NAMES` list and its tests. Pending: `fretboard_view`, the capability/permission-per-command test, logging.
- **Depends on:** B14, C9 · **Est:** L · **FR:** FR-002, FR-003, FR-004 (partly)
- **Files:** `src-tauri/{Cargo.toml,build.rs,tauri.conf.json,Info.plist,capabilities/default.json,icons/**,README.md,src/{main.rs,lib.rs,commands_list.rs,commands/mod.rs}}`, root `Cargo.toml` (add `src-tauri` to `members`, add `tauri`, `tauri-build`, `tracing-subscriber`, `tracing-appender` to `[workspace.dependencies]`), `Makefile` `ui/dist` stub rule (D-15), `ui/package.json` minimal so `tauri.conf.json`'s `beforeBuildCommand` works, `ui/.nvmrc` (22) and `engines`, `src-tauri/capabilities/README.md`, `src-tauri/icons/README.md` (doccheck does not skip these folders).
- **Acceptance (SAC-02):** `cargo build -p mackey-app` succeeds on Linux with WebKitGTK and ALSA dev packages; window 1280×800, min 900×600; CSP per the architecture; capabilities list only app command permissions; `NSMicrophoneUsageDescription` in `Info.plist`; one command `fretboard_view` returning an empty-selection view works (walking skeleton); archcheck passes (`tauri` only in `mackey-app`); `make deny` passes or `deny.toml` is extended for permissive licences only (Q-6).
- **Tests:** unit test that `COMMAND_NAMES` has no duplicates and that `capabilities/default.json` mentions exactly the permissions `allow-<name>` for each name (parse the JSON in the test).

### D2. `AppError`, panic guard, logging
- **Progress:** Partly done: `AppError { message, details }`, `guard` with tests (AC-NF-24). Pending: `AppErrorKind`, core-error `From` conversions, mutex poisoning, `tracing` file logging.
- **Depends on:** D1 · **Est:** M · **FR:** FR-004 · **SAC-03**
- **Files:** `src-tauri/src/{error.rs,guard.rs,logging.rs}`.
- **Acceptance:** `AppError`/`AppErrorKind` with `From` conversions mapping each core error to a plain message (E-FB-1 text for chord parse errors, "No audio output" for `NoOutputDevice`, "Microphone access is off" for `PermissionDenied`); `guard(|| ...)` wraps a closure in `catch_unwind(AssertUnwindSafe)` and returns `AppError { kind: Internal, message: "Something went wrong", details: Some(panic message) }`, logging the full chain with `tracing`; poisoned mutex handled (recover with `into_inner`, log); logging layer writes to `app_data_dir/logs/mackey.log` (rotating daily, 3 kept) with a test using a temp dir.
- **Tests:** AC-NF-24 (a command closure that panics returns `AppError` "Something went wrong"); mutex poisoning test; table of error → message mappings; no panic payload text leaks into `message`.

### D3. Settings: types, JSON store, reset per group
- **Depends on:** D2 · **Est:** M · **FR:** FR-028
- **Files:** `src-tauri/src/settings.rs`, `commands/settings.rs`.
- **Acceptance:** the types in the data dictionary with G-4/08 §6 defaults; `load(dir)`, `save(dir, &Settings)` (atomic temp + rename), `reset(group)`, `apply(patch)` with clamping and validation (frets 19..=24, A4 415..=466, volume 0..=100); corrupt file renamed `.bak-<ts>` and defaults used with a one-time notice; unknown fields ignored; missing file → defaults; version field.
- **Tests:** AC-FR-028 (save, reload, "reset Display" restores only Display to defaults while Instrument stays changed); corrupt-file and out-of-range tests; round trip property for arbitrary valid settings; commands `get_settings`, `set_settings`, `reset_settings`.

### D4. Explorer state and commands
- **Depends on:** D2, D3, B14 · **Est:** L · **FR:** FR-019, FR-020, FR-022 · **SAC-18**
- **Files:** `src-tauri/src/{state.rs,explorer.rs,commands/explorer.rs}`.
- **Acceptance:** `Explorer` owns `Selection`, `Instrument`, `Handedness`, last `VoicingResult` cache; commands from the architecture table (`get_snapshot`, `fretboard_view`, `select_key`, `select_diatonic_chord`, `select_chord`, `parse_chord`, `select_note`, `select_tuning`, `set_capo`, `set_view_type`, `set_labels_mode`, `set_scale_overlay`, `set_option`, `cycle`, `voicings_for`, `fit_selection`); panel assembly (key label, tonic and mode choices, diatonic chips, chord field status, instrument panel, scale choices); invalid chord text returns `Ok(snapshot)` with `ChordFieldStatus::Error` and the selection unchanged (FB-4.4, NF-23); settings feed handedness, fret count, default tuning, labels default and allow stretches into the explorer; every snapshot carries an incrementing `revision` (SAC-18, D-31); commands take the explorer mutex for the whole read-modify-snapshot step so concurrent calls serialise; invalid arguments (string 0 or 7, fret above the fret count, capo 13) return `AppError { kind: Invalid }` without changing state.
- **Tests:** integration tests via the `Explorer` type (no Tauri runtime): AC-FB-2.2 through the command path, AC-E-FB-4, `select_chord("Hm7")` returns the E-FB-1 message with suggestion and unchanged selection, Roman numeral `vi` in G selects Em; no music logic in this crate (review checklist item: grep for theory math).

### D5. Audio commands, `AudioPort`, tuner commands and event emitter
- **Depends on:** D4, C9 · **Est:** L · **FR:** FR-024, FR-027 · **SAC-13, SAC-16**
- **Files:** `src-tauri/src/{ports.rs,commands/audio.rs,commands/tuner.rs}`.
- **Acceptance:** `AudioPort` trait with `RealAudio` and `FakeAudio`; `play_selection`, `play_note`, `stop_playback`, `list_input_devices`, `tuner_start`, `tuner_stop`, `tuner_set_mode`, `tuner_reference_tone`, `open_system_settings`; Play on a machine with no output returns `AppErrorKind::NoAudioOutput` and every explorer command still works (AC-E-FB-8); mute and volume from settings applied via `SetMute`/`SetVolume`; opening the tuner stops selection playback (TU-7); tuner targets built from the instrument with the capo (AC-TU-5.1) and A4 from settings; `tuner_reading` emitted at about 30 Hz (throttle on the analysis callback using the reading's `t_ms`); the explorer mutex is never locked from the audio or analysis thread; `tuner_start` while running restarts, `tuner_stop` without start is a no-op, and the window-close event stops the tuner.
- **Tests:** with `FakeAudio::NoDevice`: AC-E-FB-8; with `Recording`: strum order and delays for a C voicing (6 notes → 5 with the muted low E, 30 ms spacing in samples at 48 kHz = 1,440), `play_note` frequency for string 3 fret 0 = 196.0 Hz; target list for capo 2 equals F#2 B2 E3 A3 C#4 F#4; throttle unit test.

### D6. Bindings generator, fixtures and stale-bindings check
- **Progress:** Skeleton done: `ts-rs` export of `AppInfo` and `AppError` into `ui/src/bindings/`, `make bindings`, `make bindings-check`, CI `bindings` job (D-44). Pending: command-wrapper generator, `events.ts`, `scenarios.json`.
- **Depends on:** D4, D5 · **Est:** L · **FR:** FR-003
- **Files:** `src-tauri/src/{commands/manifest.rs,bindings.rs}`, `ui/src/bindings/{README.md,*.ts (generated)}`, `ui/src/mocks/scenarios.json` (generated), `Makefile` (`bindings`, `bindings-check` in E1).
- **Acceptance:** all boundary types export with `ts-rs`; `commands.ts` has one typed async wrapper per command and `events.ts` the `tuner_reading` subscription helper; generated files carry a "GENERATED, do not edit" header; `scenarios.json` produced by driving the real `Explorer` through the scripted sequences listed in D14; running the generator twice yields identical bytes; `COMMAND_NAMES` equals the manifest names (test).
- **Tests:** the generator tests themselves (run only with `--features ts`). Staleness cannot be checked inside a normal `cargo test`, so it is checked by `make bindings-check` (E1), which regenerates and runs `git diff --exit-code`.

### D7. UI scaffold: Vite, TS strict, lint, test runners, tokens, command layer
- **Progress:** Scaffold done: Vite 7, React 18, TS strict, typed ESLint + Prettier, Vitest, `src/ipc/` command layer with mock selection (D-43, D-45). Pending: tokens, store, contrast tests.
- **Depends on:** D1 · **Est:** L · **FR:** FR-002, FR-021
- **Files:** `ui/package.json`, `ui/package-lock.json` (committed), `ui/tsconfig.json` (`strict`, `noUncheckedIndexedAccess`), `ui/vite.config.ts` (port 1420, `build.target: 'safari16'`, mock mode), `ui/eslint.config.js` (typescript-eslint strict, react-hooks, jsx-a11y), `ui/.prettierrc`, `ui/index.html`, `ui/src/{main.tsx,App.tsx,commands/index.ts,state/store.ts,theme/tokens.css}`, READMEs in `ui/` and `ui/src/bindings/` (`ui/src/**` is skipped by doccheck; `ui/e2e/` gets its README in D15).
- **Acceptance:** `npm ci && npm run lint && npm run typecheck && npm test` pass; tokens define the Okabe-Ito colours of G-2 (`--note-root #D55E00`, `--note-third #0072B2`, `--note-fifth #009E73`, `--note-seventh #E69F00`, `--note-ext #CC79A7`, scale grey, `--anchor`, `--muted`) in light and dark with `prefers-color-scheme` and `data-theme`; text contrast tests (WCAG AA 4.5:1; note dots 3:1 on the neck) computed in a Vitest test from the token values; command layer selects real or mock implementation; pinned exact versions.
- **Tests:** token contrast unit test; command-layer selection test.

### D8. Fretboard SVG component
- **Depends on:** D6, D7 · **Est:** XL (split: geometry + neck; markers + barres; tooltip) · **FR:** FR-021
- **Files:** `ui/src/fretboard/*` (see architecture).
- **Acceptance:** draws from `FretboardView` only: 6 strings, `fret_count` frets, nut, inlays, fret numbers, string labels, 12th-root-of-2 spacing with minimum fret width (24 px Mac, 28 pt iPad) and horizontal scroll when the visible range cannot fit; markers by role (root rounded square, third/fifth/seventh/extension filled circle, scale ring, single-note diamond) with Okabe-Ito tokens; open-string rings above the nut; muted "×"; barres as bars; capo dimming and capo bar; left-handed mirroring (nut on the right) and string flip as SVG transforms of the same geometry; every `ViewPosition` an element with `role="img"`/`button`, `aria-label` = `a11y_label`, `tabIndex` roving, visible focus ring; click calls `play_note`; AC-P0-5 (6 strings and 22 frets with defaults); labels ≥ 11 pt.
- **Tests:** Vitest on `geometry.ts` (fret x-positions for 22 frets sum to the scale length ratios, minimum width rule, mirror maths); component tests with Testing Library on fixtures from `scenarios.json`: every position has an aria-label; shapes by role; mirrored layout puts the nut on the right.

### D9. Tooltip, edge arrows, range controls, fit selection
- **Depends on:** D8 · **Est:** M · **FR:** FR-021
- **Files:** `Tooltip.tsx`, `EdgeArrows.tsx`, `RangeControls.tsx`.
- **Acceptance:** hover (Mac) and tap/long-press (iPad) show the tooltip from `ViewPosition.tooltip` (AC-FB-1.6: "G3", "1", "R", "Root"; theory spelling when present); range handles (drag and keyboard) and a "Fit selection" button that sets the visible range from `view.fit_range`; edge arrows when `view.off_range.below/above > 0` (E-FB-5) with an accessible name ("3 highlighted notes beyond fret 12"); visible range default by layout (0–15 Mac, 0–12 iPad landscape, 0–7 iPad portrait); pinch zoom 5 to 24 frets on iPad.
- **Tests:** Vitest component tests; Playwright `tooltip.spec.ts` for AC-FB-1.6.

### D10. Explore screen controls
- **Depends on:** D8, D4 · **Est:** XL (split by panel) · **FR:** FR-022 · **SAC-12**
- **Files:** `ui/src/explore/*`.
- **Acceptance:** toolbar (key picker with tonic + mode and the F#/Gb choice, key always shown as text; tuning and capo picker with custom tuning entry and the unusual-tension warning; labels mode; view type); diatonic chord strip with Roman numerals and a sevenths toggle; chord field with inline error "Not a chord symbol I know: Hm7. Did you mean Bm7?" in red with a text icon (not colour alone) and a "use suggestion" button, selection unchanged; voicing list grouped Open / Barre and CAGED / Other with mini diagrams and "Found by search"; triad string-set and inversion controls and "Show all on string set"; arpeggio position control, direction and "Show order"; scale overlay toggle and scale picker with "Follow the chords"; counters "3 of 7"; "Not in key" badge; capo shape/sounding names with the emphasis toggle; "No playable voicing in this tuning" with "Allow stretches" and "Allow omitted fifth" buttons; all controls call commands and render the snapshot; every control has a label and 44 pt targets on iPad.
- **Tests:** Vitest component tests per panel against fixtures; Playwright smoke (G major → chord strip → voicings).

### D11. Shortcuts, live region, iPad gestures
- **Depends on:** D10 · **Est:** M · **FR:** FR-023
- **Files:** `ui/src/{shortcuts.ts,useShortcuts.ts,gestures.ts,a11y/*}`, `ui/src/commands/queue.ts` (a single promise queue so commands reach Rust in key order; stale snapshots are dropped by `revision`).
- **Acceptance:** the shortcut table covers FB-9 and 07 §4 Explore rows (`↑ ↓ ← →`, `I`/`Shift-I`, `S`/`Shift-S`, `P`/`Shift-P`, `V`, `L`, `O`, `[`, `]`, `R`, `Space`) and `Cmd-T`; disabled while an input, textarea, select or contenteditable has focus (UX-6); `Space` activates Play only when focus is not on a button (otherwise the button's own activation) (UX-6 refinement, D-28); each cycling change writes `view.announcement` to an `aria-live="polite"` region (AC-FB-9.1c); iPad: swipe left/right on the neck → voicing prev/next, two-finger swipe → position, swipe up/down on the chord strip → chord prev/next, buttons for every FB-9 action, 44 pt targets; Reduce Motion respected (no animated transitions).
- **Tests:** AC-UX-7 (no two commands in one area share a key; table-driven over `shortcuts.ts`); Vitest: shortcut ignored in a text field; announcement written for each action; gesture classifier unit tests (thresholds, direction, two-finger).

### D12. Tuner screen
- **Depends on:** D5, D7 · **Est:** L · **FR:** FR-027
- **Files:** `ui/src/tuner/*`.
- **Acceptance:** `Cmd-T` and sidebar entry; MicExplainer (E-TU-1 text: "Mackey uses the microphone only to hear your guitar. Nothing is recorded or sent anywhere." + Continue) shown until `settings.tuner.mic_explained`; MicDenied state (E-TU-2: "Microphone access is off", Open Settings button, reference tones still work); chromatic display (note, octave, cents needle −50..+50, Hz small), in-tune state by colour **and** check mark **and** "In tune" text, "Play a single string" when `NoStablePitch`, "Too quiet: move closer or turn up your input", "Too loud: turn down your input", string mode with six targets, auto-selected string, Tighten/Loosen arrows with cents, tap-to-lock, per-string ticks, "Far from any string: check the tuning setting", reference tone buttons (3 s, toggle), device picker, level meter, disconnect notice; tuner announcements throttled to once per second to the live region (NF-5); subscribes to `tuner_reading` on mount and unsubscribes on unmount calling `tuner_stop`; also calls `tuner_stop` when `document.visibilityState` becomes `hidden` and restarts on `visible` (iPad backgrounding; SAC-16); shows "The tuner stopped unexpectedly. Close and reopen it." on `AnalysisStopped`.
- **Tests:** Vitest with `mockTuner` series for each state; Playwright `tuner.spec.ts` (explainer → readings → in-tune state → too quiet; denied state).

### D13. Sidebar, layouts, narrow behaviour, settings screen
- **Depends on:** D10, D12, D3 · **Est:** L · **FR:** FR-025, FR-028 · **SAC-14, SAC-17**
- **Files:** `ui/src/layout/*`, `ui/src/settings/SettingsScreen.tsx`.
- **Acceptance:** sidebar with Explore, Tools (Tuner) and Settings; `Cmd-0` toggles it; Mac layout: toolbar, main, neck below main (default); iPad landscape: sidebar, main, neck across the bottom 40%; iPad portrait: tab bar, main top half, neck bottom half showing 7 frets and scrolling; narrow window (< 700 px or Split View one-third): sidebar hidden, panels collapse into a menu, the neck scrolls and never shrinks below the FB-1.4 minimums (E-FB-9); settings screen for the FR-028 settings with per-group reset buttons, applied immediately, theme applied via `data-theme`, `Cmd-,` opens it.
- **Tests:** Playwright `ipad-layouts.spec.ts` at 1180×820 and 820×1180 and `narrow.spec.ts` at 680×800; Vitest for `useLayout`.

### D14. Vitest suite, mock command layer, recorded scenarios
- **Depends on:** D6, D8..D13 · **Est:** M · **FR:** FR-021..027 · **SAC-18**
- **Files:** `ui/src/mocks/{mockCommands.ts,mockTuner.ts}`, tests next to sources.
- **Acceptance:** `mockCommands` replays `scenarios.json` (sequences recorded from the real core): `g-major-notes`, `chord-strip-walk` (I..vii), `c-voicings-cycle` (≥ 20 voicing steps with wrap), `triads-234-cycle`, `am-arpeggio-position-5`, `a-minor-pent-boxes`, `left-handed-g`, `capo-5-moved`, `hm7-error`, `no-voicing`; mock never contains theory; `mockTuner` emits a scripted `TunerReading` series (listening → pitch flat → in tune → too quiet → no stable pitch); Vitest coverage on `ui/src` reported (`vitest --coverage`), target ≥ 80% lines on non-generated files.
- **Tests:** the Vitest suite; a test that every scenario step deserialises into the generated types (type-level check via `satisfies`).

### D15. Playwright suite (WebKit, desktop and iPad)
- **Progress:** Scaffold done: Playwright config (WebKit; `desktop` and `ipad` projects, mock web server), `e2e/README.md`, one skeleton spec. Pending: the feature specs, portrait project, axe.
- **Depends on:** D14 · **Est:** L · **FR:** FR-006, FR-021..027
- **Files:** `ui/e2e/README.md` (doccheck does not skip `e2e`), `ui/playwright.config.ts` (projects `desktop` 1280×800 and `ipad` 1180×820 with `hasTouch`, a third `ipad-portrait` 820×1180; webServer `vite --mode mock`), `ui/e2e/*.spec.ts`.
- **Acceptance:** AC-P0-5 (neck draws 6 strings and 22 frets), AC-FB-9.1b (each cycling key pressed 100 times, each redraw ≤ 16 ms measured from `keydown` dispatch to the next `requestAnimationFrame` after the DOM update, p100 ≤ 16 ms on the mock layer, with the first 5 warm-up presses discarded and the budget stated in the test), AC-FB-9.1c (announcements for each cycling action), AC-FB-1.6 (tooltip), AC-LH-1.1 (nut on the right, frets mirrored, tooltips unchanged), shortcut tests (key ignored in the chord field), tuner states, iPad layouts, axe accessibility scan (`@axe-core/playwright`, no serious violations on Explore and Tuner in light and dark), colour-blind check is the token test in D7.
- **Tests:** the specs; traces on failure; one retry in CI.

### D16. Manual checklist and walking-skeleton proof (AC-MAN-1)
- **Depends on:** D15, E6 · **Est:** S · **FR:** FR-001..027
- **Files:** `implementation-notes.md` (manual checklist section), `user-docs/` stub deferred to Step 4.
- **Acceptance:** the checklist for AC-MAN-1 (Mac launch, G major shown, Space strum, tuner reads a played string; iPad Simulator launch and landscape/portrait render) is written with steps and expected results and left unchecked with a note that J runs it; a Linux-side smoke test launches the app crate in a headless test that calls `fretboard_view` through the command function and asserts 6 strings and 22 frets.

---

## Phase E: Makefile, AGENTS.md, CI, docs

### E1. Makefile targets
- **Progress:** Done except `dev`: `ui-lint ui-test e2e bindings bindings-check app ios-sim`, stub rule, `ui/node_modules` rule, `check` extended (D-15, D-46).
- **Depends on:** D7 · **Est:** M · **FR:** FR-005 · **SAC-04**
- **Files:** `Makefile`.
- **Acceptance:** `.PHONY` gains `ui-lint ui-test e2e bindings bindings-check app dev ios-sim`; `ui/node_modules` file target runs `npm ci --prefix ui` when `package-lock.json` is newer; `ui-lint` = eslint + prettier `--check` + `tsc --noEmit`; `ui-test` = `vitest run`; `e2e` = `npx playwright test` (installs WebKit when `CI` is unset? no: documented as a one-time `npx playwright install webkit`); `bindings` and `bindings-check` per the architecture; `app` = `npx tauri build --target universal-apple-darwin` on macOS (`APP_ARGS ?=`; on other hosts it builds for the host with `--no-bundle` and says so); `dev` = `npx tauri dev`; `ios-sim` = `npx tauri ios init` if `src-tauri/gen/apple` is missing, then the simulator build (E6 sets the exact command); the stub rule `ui/dist/index.html`; `make check` = existing steps plus `ui-lint ui-test` and `bindings-check` when `node` is installed, otherwise a clear error naming what to install.
- **Tests:** `make -n` dry runs in the task notes; `make doccheck` passes after E2.

### E2. AGENTS.md updates
- **Progress:** Done: AGENTS.md Commands table, CI table and "Not yet in place" updated.
- **Depends on:** E1 · **Est:** S · **FR:** FR-005
- **Files:** `AGENTS.md`.
- **Acceptance:** Commands table gains a row per new `.PHONY` target (`ui-lint`, `ui-test`, `e2e`, `bindings`, `bindings-check`, `app`, `dev`, `ios-sim`) and the sentence "UI and app targets ... are added with the Tauri scaffold" is replaced by the facts; CI/CD "Not yet in place" paragraph updated (what now exists, what remains: `release.yml`); crate table unchanged (already allows these edges) with a note that `mackey-audio` uses theory only for now; `make doccheck` passes.

### E3. CI: Rust jobs with system packages, stub dist
- **Progress:** Done: Linux packages step; stub `ui/dist` via the Makefile.
- **Depends on:** D1 · **Est:** M · **FR:** FR-006
- **Files:** `.github/workflows/ci.yml`.
- **Acceptance:** the `rust` matrix job installs Linux packages (`sudo apt-get update && sudo apt-get install -y libwebkit2gtk-4.1-dev libgtk-3-dev libsoup-3.0-dev libjavascriptcoregtk-4.1-dev librsvg2-dev libasound2-dev libssl-dev pkg-config`) before the Rust steps; macOS needs nothing; `make cover` excludes the thin glue by regex (E7); all steps use `make`.

### E4. CI: UI, Playwright and stale-bindings jobs
- **Progress:** Done: jobs `ui`, `e2e`, `bindings`. Cannot be run on GitHub from the sandbox.
- **Depends on:** D15, E1 · **Est:** M · **FR:** FR-006
- **Files:** `.github/workflows/ci.yml`.
- **Acceptance:** jobs `ui` (setup-node 22 with npm cache, `make ui-lint`, `make ui-test`), `e2e` (`npx playwright install --with-deps webkit` then `make e2e`; uploads `ui/playwright-report` and traces on failure), `bindings` (Rust toolchain, system packages, `make bindings-check`); each exits 0 on the PR.

### E5. CI: universal macOS build
- **Progress:** Written, not run (needs macOS): job `mac-app` with the `lipo -archs` check.
- **Depends on:** D1, E1 · **Est:** M · **FR:** FR-006 · **SAC-19**
- **Files:** `.github/workflows/ci.yml`.
- **Acceptance:** job `mac-app` on `macos-latest`: Rust (targets from `rust-toolchain.toml` include both Apple triples), setup-node, `make app` producing a universal `.app` (unsigned; no Apple secrets referenced), artefact uploaded; verify with `lipo -archs` that both `arm64` and `x86_64` are present (step fails otherwise).

### E6. CI: iOS Simulator build
- **Progress:** Written, not run (needs macOS): job `ios-sim` with the compile-only fallback; record the working command in ADR 0008 after the first run.
- **Depends on:** D1, E1 · **Est:** L (unknown-risk task: start early) · **FR:** FR-006
- **Files:** `.github/workflows/ci.yml`, `Makefile` (`ios-sim`), `docs/adr/0008-ipad-via-tauri-ios.md` (one-paragraph note with the working command).
- **Acceptance:** job `ios-sim` on `macos-latest` runs `make ios-sim` successfully with no Apple account; the fallback chain from research.md Q-3 is tried in order and the first that works is recorded; the job selects a simulator runtime that exists on the runner image (`xcrun simctl list runtimes`); if `NSMicrophoneUsageDescription` must be merged into the generated `Info.plist` for iOS it is configured in `tauri.conf.json`'s `bundle.iOS` (`infoPlist`) not by editing generated files.

### E7. Coverage configuration
- **Progress:** Done: `COVER_IGNORE` in the Makefile (D-48).
- **Depends on:** D5 · **Est:** S · **FR:** FR-006
- **Files:** `Makefile` (`cover`: `--ignore-filename-regex` for `src-tauri/src/main.rs`, `src-tauri/build.rs`, `crates/mackey-audio/src/io/`, `src-tauri/src/bindings.rs`, each excluded with a comment saying why).
- **Acceptance:** `make cover` ≥ 80% overall; the exclusion list is documented in `AGENTS.md`'s Commands row for `make cover`; new pure-crate code is fully covered (check per crate with `cargo llvm-cov -p`).

### E8. Docs and ADR updates that belong to the code change
- **Depends on:** E2 · **Est:** M · **FR:** FR-003, FR-005
- **Files:** `docs/adr/0007-web-ui-and-ipc-boundary.md` (one line: ts-rs chosen; tauri-specta 2 is pre-release; commands.ts generated from a manifest), `docs/architecture.md` (commands and events, `roman_to_chord` returns `Result`), `docs/prd/09-traceability.md` (open items: FB-8.2 deferral, recorded fixtures follow-up, iOS plugin), `docs/roadmap.md` (P0/P1 status), `.github/README.md` (jobs), `ui/README.md`, `src-tauri/README.md`, root `README.md` (commands table: new targets; layout), `crates/README.md`.
- **Acceptance:** `make doccheck` passes; each README states purpose, key files and architecture fit; Step 4 (documentation and user docs) later adds `user-docs/`.

---

### E9. `xtask doccheck`: skip generated report folders
- **Progress:** Done (with a unit-test extension).
- **Depends on:** none (do it with E1) · **Est:** S · **FR:** FR-005
- **Files:** `xtask/src/doccheck.rs` (add `playwright-report`, `test-results`, `coverage` to `SKIP_DIRS`; extend its unit test first), `xtask/README.md`.
- **Why:** these folders are git-ignored but exist in the working tree after `make e2e` or `make ui-test`, and `doccheck` walks the working tree, so a local `make check` would fail with "has no README.md".
- **Acceptance:** after running `make e2e ui-test`, `make doccheck` still passes; the xtask test asserts the three names are skipped.

## Dependency summary

```
A1 → A2 → A3 → A4 → A5 ─┬→ A7 ┐
              A3 → A6 ──┤       ├→ A9 → A10 → A11
                   A6 → A8 ─────┘
A11 → B1 → B2 → B3 → B4 → B5 → B9 → B10 → B11 → B14 → B15
          B1 → B6, B7, B8 → B9;  B5/B10 → B12, B13;  B10 → B16
A1 → C1 → C2 → C3 → C7 → C9;  C1 → C4 → C5 → C6;  C3, C4 → C8
B14 + C9 → D1 → D2 → D3, D4 → D5 → D6 → D8 → D9, D10 → D11 → D13 → D14 → D15 → D16
D1 → D7 → D8, D12;  D5 → D12
D7 → E1 → E2;  E9 with E1;  D1 → E3, E5, E6;  D15 + E1 → E4;  D5 → E7;  E2 → E8
```

Phases A, B and C are largely independent after A1 (C needs only `mackey-theory`'s pitch types from A2); an implementer with parallel capacity can run C alongside B. Phase E tasks E3, E5 and E6 should start as soon as D1 builds, because macOS and iOS behaviour can only be proven in CI.
