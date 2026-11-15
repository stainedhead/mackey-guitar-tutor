# PRD: Fretboard Foundation (roadmap P0 and P1)

**Created:** 2026-11-15
**Jira:** N/A
**Status:** Reviewed, ready for spec (review log at the end)
**Owner:** J
**Source of truth:** this file is the dev-flow working PRD for one implementation run. It is an adapter over the product PRD in [`docs/prd/`](docs/prd/README.md) and [`INTENT.md`](INTENT.md): it selects the requirements for this run, numbers them `FR-nnn` for the dev-flow tools, and points at the detailed requirement (`FB-*`, `SC-*`, `LH-*`, `TU-*`, `G-*`, `NF-*`, `UX-*`) for exact behaviour. **Where this file and `docs/prd/` differ, `docs/prd/` wins and this file is fixed.** Acceptance criteria IDs (`AC-*`) are the product PRD's.

## Problem Statement

A beginner or improving guitarist (J first, then the Bea and Ivan personas in [docs/prd/README.md §3](docs/prd/README.md#3-personas)) learns chord shapes by rote and stays stuck in the first few frets. They cannot see *where* the notes of a key, chord, triad, arpeggio or scale sit on the neck, cannot step through them quickly, and have no in-app way to get in tune first. The repository today has the design (INTENT, PRD, architecture, ADRs) and the Rust workspace tooling (`xtask`, `make check`, CI), but **no product code**: no theory engine, no fretboard, no app shell, no UI and no audio path.

This run builds the foundation every later phase stands on: the Tauri 2 app shell for macOS and iPad (iOS Simulator), the pure theory and voicing core, the shared SVG fretboard driven by a `FretboardView` computed in Rust, the explorer (keys, chords, voicings, triads, arpeggios, scale overlays, cycling), left-handed mode, tunings and capo, and the tuner.

## Goals

- **G1. Walking skeleton on both targets.** A Tauri 2 app whose React UI calls a typed Rust command and draws the neck, builds as a universal macOS app and for the iOS Simulator in CI, and can play a sound through the Rust audio path (roadmap P0).
- **G2. Correct theory core.** `mackey-theory` spells every note, interval, scale, mode, key and chord correctly (G-1), parses the chord-symbol grammar and Roman numerals, and is proven by table-driven and property tests (roadmap P0 exit).
- **G3. Fretboard explorer.** A player can pick a key, step I to vii°, and for each chord see the open shape, barre/CAGED shapes, triads on each string set in each inversion and an arpeggio in position, with scale overlays and pentatonic boxes, left- or right-handed, in any preset tuning and with a capo (roadmap P1: FB-1 to FB-9, SC-1 to SC-5, LH-1 to LH-5).
- **G4. Tuner.** A chromatic and string-aware tuner that reads within ±3 cents from the default input device (TU-1 to TU-7), with its pitch detection proven against synthetic signals.

## Non-Goals

- Rhythm (P1b: transport clock, metronome, drums, speed trainer; RH-*, DR-*, ST-*, TR-*). The transport bar is not built in this run.
- Practice sessions (P2, PS-*), songs (P3/P4, SG-*), web import (P5), CAGED maps, tips, progress, ear training, looper, mic recognition (P6 to P8).
- Persistence beyond settings: `mackey-store` (SQLite), library export/import, Recently deleted and drafts (G-6) are P2/P3. Settings in this run persist through a small JSON file owned by `src-tauri` (see FR-028).
- Bundled SoundFont sample playback. Sound assets need licence confirmation at source (open item in [09-traceability.md](docs/prd/09-traceability.md#open-items)); this run synthesises guitar-like tones (FR-024) and leaves the SoundFont for P1b, where the asset work is scheduled.
- Release automation (`release.yml`, ad-hoc signed `.dmg` on tags). It needs nothing from J but is not needed to prove the walking skeleton; it is a P0 item deferred to the next run unless time allows.
- Device validation on a real iPad, Apple signing, App Store, iCloud (need J's Apple ID; see [platforms.md §7](docs/platforms.md#7-ipad-signing-and-validation-decided)).
- Windows, Linux, iPhone and Android as products. Linux CI compiles and tests the code only because runners are cheap.
- First-launch onboarding flow F-1, multi-window (UX-4.1), menus mirroring every command (UX-4.2): P4 polish.

## Functional Requirements

Each FR names the product requirements it implements; the detail and edge cases are in the linked section and are part of the requirement.

### Scaffold and boundary (P0)

**FR-001:** Cargo workspace members `crates/mackey-theory`, `crates/mackey-voicing`, `crates/mackey-audio` and `src-tauri` (crate `mackey-app`), each with a `README.md`, registered in the workspace `members`, the `xtask` layer table and the AGENTS.md table, so `make archcheck` passes. The existing table already allows these edges; in this run `mackey-audio` depends on `mackey-theory` only (`mackey-rhythm` arrives in P1b). `CLAUDE.md` already exists (it includes AGENTS.md), so the roadmap P0 `CLAUDE.md` item is done ([architecture.md §2](docs/architecture.md#2-module-layout)).

**FR-002:** A Tauri 2 app in `src-tauri/` with a React + TypeScript (strict) + Vite UI in `ui/`. Window minimum 900 × 600, default 1280 × 800 (UX-4). Strict CSP with no remote content; capabilities allow only the app's own commands (NF-7.3).

**FR-003:** Types crossing the IPC boundary derive `serde` and generate TypeScript bindings with `ts-rs` (ADR 0007 already permits it; add a one-line note to the ADR saying it was chosen because tauri-specta 2 is pre-release) into `ui/src/bindings/` (generated, never hand-edited). A make target regenerates them and CI fails if they are stale ([ADR 0007](docs/adr/0007-web-ui-and-ipc-boundary.md)).

**FR-004:** Every Tauri command returns `Result<T, AppError>` where `AppError` is serialisable with a plain message and optional details; panics inside a command are caught and returned as "Something went wrong" (NF-22, NF-24).

**FR-005:** New make targets `ui-lint`, `ui-test`, `e2e`, `bindings`, `app` and `ios-sim`, each with a row in the AGENTS.md Commands table (doccheck). `make check` includes the UI lint, type check and unit tests.

**FR-006:** CI gains jobs for: UI lint + type check + Vitest; Playwright (WebKit, desktop 1280 × 800 and iPad 1180 × 820 viewports) against the UI with a mocked command layer; the stale-bindings check; `tauri build` of a universal macOS app (unsigned, no Apple account); and `tauri ios build` for the iOS Simulator (unsigned compile check). Linux Rust jobs install the WebKitGTK and ALSA development packages so the whole workspace compiles there. All of these jobs are required checks. For the iPad, the P0 walking-skeleton exit in this run is met by the green Simulator compile job plus the Playwright iPad-viewport render; audible output and microphone input in the Simulator are checked manually (AC-MAN-1) and are not a CI gate.

### Theory core: `mackey-theory` (P0, G-1)

**FR-007:** Types `PitchClass`, `Letter`, `Note` (letter + accidental, spelled), `Pitch` (note + octave, MIDI and frequency conversion with a reference A4), `Interval` (quality + number), validated at construction ([architecture.md §3.1](docs/architecture.md#31-music-theory)).

**FR-008:** Scales and modes: Major (Ionian), Natural minor (Aeolian), Harmonic minor, Dorian, Phrygian, Lydian, Mixolydian, Locrian, Major pentatonic, Minor pentatonic, Blues, each spelling its degrees correctly for its tonic (FB-2.1, SC-1).

**FR-009:** Keys: the major and minor tonics offered in G-1.2 and G-1.3 (with the F#/Gb and D#/Eb choices), modes taking their parent key signature with enharmonic tonic substitution (G-1.4), next/previous tonic around the circle of fifths and relative major/minor (FB-2.4), and the enharmonic alternative for a key with 6 accidentals for the tooltip (E-FB-7).

**FR-010:** Chord qualities and a chord-symbol parser covering the grammar in FB-4.3 (major, m, 7, maj7, m7, m7b5, dim, dim7, aug, sus2, sus4, 7sus4, 6, m6, 9, maj9, m9, add9, 11, 13, 5, slash bass, and common alternative spellings such as `min`, `-`, `M7`, `Δ`, `ø`, `°`, `+`). Unknown symbols return a positioned error with a suggestion when the edit distance is ≤ 2 or the root is `H` (E-FB-1). The player's spelling is kept (G-1.7).

**FR-011:** Diatonic chords of a key as triads and sevenths, with Roman numerals (I, ii, iii, IV, V, vi, vii° and Imaj7 ... viiø7 in major; the mode's own qualities otherwise), Roman-numeral parsing (`vi` in G → Em), and a "not in key" analysis with function where known (secondary dominants `V/x`, borrowed chords such as `bVII`) (FB-4.2, FB-4.5, AC-FB-4.1).

**FR-012:** Chromatic spelling per G-1.5 and display spelling per G-1.6 (no double sharps or flats as fretboard labels).

### Voicing and fretboard: `mackey-voicing` (P1)

**FR-013:** `Tuning` presets (all nine in LH-2) and custom tunings (each string by note and octave, with the "unusual tension" warning outside the allowed range), `Instrument` (tuning, frets 19 to 24, capo 0 to 12) and `Handedness` (left-handed, flip strings) (LH-1 to LH-3, G-5).

**FR-014:** Curated open-shape library exactly matching FB-5.3 (including the small and full-barre F), and CAGED barre shapes (C, A, G, E, D shapes, movable), defined for standard tuning and transposed for Half-step and Whole-step down; Drop D has its own curated entries where they differ (LH-4).

**FR-015:** Voicing search for any chord, tuning and capo, obeying FB-5.4 (root and third, or sus note, or root and fifth for power chords; span ≤ 4 frets, or 5 with "Allow stretches"; at most 4 fingers with a barre counting as one; required bass for slash chords and inversions), scored and grouped Open / Barre and CAGED / Other (FB-5.1, FB-5.2), with fingering and barre detection (FB-5.5), "Found by search" marking for non-curated tunings (LH-4), custom tunings with two identical open strings handled without double-counting in fingering (E-FB-6) and the "No playable voicing" result with the stretch and omitted-fifth options (FB-5.6, E-FB-3).

**FR-016:** Triads on each string set (123, 234, 345, 456) in root, first and second inversion, one note per string on three adjacent strings, including "show all on string set" in fret order and the seventh-chord note (FB-6.1 to FB-6.4).

**FR-017:** Arpeggio in position: every chord tone in a 4-fret window with one-fret stretches either side, suggested fingers (one finger per fret, index on the start fret), play order for up, down and up-down, and the default-window rule (FB-7.1 to FB-7.4).

**FR-018:** Scale overlay patterns: full neck, the five pentatonic boxes (box 1 has the root on string 6 under the first finger) and seven 3-notes-per-string patterns for the major modes, cycled with next/previous position (SC-1 to SC-3, SC-5). "Follow the chords" scale choice (SC-4) is implemented as a pure function and tested; it is wired into run and song mode in P2/P3.

**FR-019:** `FretboardView`: one serialisable value computed from the current selection, instrument and handedness, listing for every visible position its string, fret, spelled note, octave, role (root, third, fifth, seventh, extension, scale tone, single note), label for each labels mode (G-2.2), finger, open/muted markers, barres, dimmed frets below the capo, capo indicator, inlays, fret numbers, string labels, list position ("3 of 7"), notices ("Voicing changed for new tuning", "Moved above the capo") and the accessibility label for each position ("String 3, fret 2, A, third of F", NF-5). With focus "none" the value is the key-notes view: every note of the key, tonic as root shape, degrees 1, 3, 5, 7 coloured and 2, 4, 6 as scale tones (FB-3.1, FB-3.2). Highlighted notes outside the visible range are reported per edge so the UI can draw an arrow (E-FB-5), and a "fit selection" fret range is computed (FB-1.5). The UI draws only from this value (G-3.2).

**FR-020:** Selection model per G-3: one current selection (key, focus, chord and view type, voicing, inversion, string-set and position indexes, scale overlay, labels mode); key change keeps the scale degree (G-3.3, AC-FB-2.2); tuning or capo change keeps the chord and reselects the best voicing with a notice (G-3.4, E-FB-4). Cycling actions (FB-9 table) are pure functions on the selection that wrap as specified.

### App and UI (P1)

**FR-021:** The shared SVG fretboard component: 6 strings, frets per instrument, nut, inlays, fret numbers, string labels, 12th-root-of-2 spacing with minimum fret widths and horizontal scrolling (FB-1.1 to FB-1.4), the G-2 shapes and Okabe-Ito colour tokens in light and dark themes, left-handed mirroring and string flip (LH-1, AC-LH-1.1), capo dimming (LH-3), a tooltip on hover (Mac) or tap (iPad) with the FB-1.6 contents, visible-range controls with "Fit selection" (FB-1.5) and edge arrows for off-range notes (E-FB-5). Every position is an accessible element labelled from `FretboardView` (NF-5).

**FR-022:** The Explore screen: toolbar with key picker (tonic + mode, the key always shown as text, FB-2.2), tuning and capo picker, labels mode and view type; the diatonic chord strip with the sevenths toggle (FB-4.2); the chord field with inline errors and suggestions (FB-4.4, E-FB-1); the voicing list with mini diagrams grouped as FB-5.2; triad string-set and inversion controls; arpeggio position control and "Show order"; scale overlay toggle and scale picker; "3 of 7" counters (FB-9.2); "Not in key" badge (FB-4.5); capo shape/sounding names (LH-5).

**FR-023:** Keyboard shortcuts from FB-9 and the Explore rows of [07 §4](docs/prd/07-ux-flows.md#4-keyboard-shortcuts-mac-and-ipad-with-a-keyboard) (`↑ ↓ ← →`, `I`, `S`, `P` with Shift, `V`, `L`, `O`, `[`, `]`, `R`, `Space`, `Cmd-T`), disabled while a text field has focus (UX-6), with no two commands in one area sharing a key (UX-7, tested). Every change is announced through an ARIA live region (FB-9.1). On iPad: swipe on the neck for voicings, buttons for every cycling action (FB-9 table), 44 pt targets (UX-5.1).

**FR-024:** Sound: Play (Space or button) plays the selection as a strum (low to high, 30 ms apart), an arpeggio in eighth notes at 70 bpm, or the key's scale ascending (FB-8.1); clicking a position plays its note (FB-1.7); a mute toggle (FB-8.4). **FB-8.2 (nylon, steel, electric from a bundled SoundFont) is deferred to P1b**, where the sound-asset licence work is scheduled; this run ships one synthesised steel-like voice. Tones are synthesised in `mackey-audio` (plucked-string synthesis) on a cpal output stream; the callback follows the real-time rules (no allocation, locks or I/O; commands arrive over an `rtrb` queue). "No audio output" is shown once when no device is available (E-FB-8).

**FR-025:** Sidebar with the areas Explore and Tools (Tuner) for this run (UX-1 subset), iPad landscape and portrait layouts from UX-5 using CSS breakpoints, and the narrow-window behaviour of E-FB-9.

### Tuner (P1)

**FR-026:** Pitch detection in `mackey-audio` (McLeod Pitch Method or YIN), as a pure analysis module testable without a device: 60 to 1,400 Hz, ≥ 20 readings per second from a 2048-sample window, median-of-3 smoothing, cents from the nearest equal-tempered pitch relative to the reference A4, confidence, input level, "too quiet" and "too loud" detection (TU-1, TU-3, E-TU-3, E-TU-4, NF-9).

**FR-027:** Tuner screen (Tools → Tuner, `Cmd-T`): chromatic display (note, octave, cents needle, Hz), "In tune" within ±3 cents held 300 ms shown by colour, check mark and text, "Play a single string" after 500 ms without a pitch; string mode (default) with the six targets for the current tuning and capo, auto-select of the nearest string, Tighten/Loosen arrows, tap-to-lock, per-string ticks, "Far from any string" (TU-1, TU-2, TU-5); reference tones per string using the synthesised voice (TU-4); fallback to the default input with a notice when the device disconnects (E-TU-5); the strongest pitch, or the TU-1.4 message when confidence is low, with two strings ringing (E-TU-6); input device picker and level meter (TU-6); the microphone explanation screen and the permission-denied state (E-TU-1, E-TU-2); `NSMicrophoneUsageDescription` set. Opening the tuner stops selection playback (TU-7). Readings reach the UI as a `tuner_reading` event at about 30 Hz.

### Settings (P1 subset)

**FR-028:** The Instrument, Display, Sound and Tuner settings from [08 §6](docs/prd/08-nfr.md#6-settings-complete-list-for-v1) that this run's features use (handedness, flip strings, frets, default tuning, allow stretches, theme, labels default, show fret numbers, selection playback volume and mute, input device, reference A4; the playback instrument selector is hidden until P1b and "Show tips" waits for P6), with the G-4 defaults, applied immediately, persisted as JSON in the app data directory by `src-tauri`, and reset per group.

## Non-Functional Requirements

- **Performance:** `fretboard_view` ≤ 2 ms and voicing search for one chord on the full neck ≤ 5 ms p95 in release builds, guarded by criterion benchmarks (NF-1, NF-1.1). Neck redraw ≤ 16 ms per cycling action (NF-1; checked by a Playwright timing test with the mocked command layer). Selection playback starts ≤ 30 ms after the action plus device latency (NF-3.1; manual check). Cold start to interactive neck ≤ 2 s on Mac (NF-3; manual).
- **Reliability:** No panics across the IPC boundary (NF-24); parsers never panic on any input (property test); errors never discard input (NF-23). The audio callback never allocates, locks or does I/O (NF-8.1; enforced by code structure, review and an `assert_no_alloc` check around the render function in debug tests). If no audio device is present, every visual feature still works (E-FB-8).
- **Security and privacy:** No network calls, telemetry or accounts (NF-7). Microphone audio is processed in memory only and never written to disk (NF-7.1); the permission is requested only when the tuner first opens (NF-7.2). Strict CSP and minimal Tauri capabilities (NF-7.3). `unsafe_code` stays forbidden; `cargo deny` passes with permissive licences only.
- **Accessibility:** Colour-blind-safe palette with shape cues (G-2, NF-5.2), full keyboard control with visible focus (NF-5.1), VoiceOver labels on every fretboard position and announced changes (NF-5), Reduce Motion respected (NF-5.4), system text size respected with a 11 pt minimum on neck labels (NF-5.3).
- **Observability:** `tracing` in `src-tauri` and `mackey-audio` (never in the audio callback); command errors are logged with their full chain to a local log file in the app data directory; nothing is sent anywhere (NF-7, NF-22, NF-24).
- **Quality:** `make check` and CI green on macOS and Linux; line coverage ≥ 80% overall, new pure-crate code fully covered by tests that name their requirement ID (for example `fb_5_4_no_voicing_exceeds_span`, [09-traceability.md](docs/prd/09-traceability.md)); table-driven tests for theory, property tests for the invariants listed in AGENTS.md, insta snapshots for parser output.
- **Not measured this run:** memory (NF-12), app size (NF-13) and visual sync (NF-8.2, needs the transport from P1b).
- **Compatibility:** macOS 13+ universal; iPadOS 17+ (Simulator only in this run); 44.1 and 48 kHz devices (NF-19, NF-20).

## Acceptance Criteria

Each criterion maps to at least one automated test unless marked *(manual)*.

- [ ] **AC-P0-1** (G2) Table-driven tests: every chord quality in FR-010 in all 12 keys is spelled as in a hand-written expected-spelling table in `crates/mackey-theory/tests/` (roadmap P0 exit).
- [ ] **AC-P0-2** (G2) The chord parser accepts every quality and alternative spelling listed in FR-010 / FB-4.3 with every root and an optional slash bass; `Hm7` returns an error suggesting `Bm7` (E-FB-1); the parser never panics on arbitrary input (property test).
- [ ] **AC-P0-3** (G2) Property tests: transpose up then down is the identity; interval add then subtract returns the start; spelling in every offered key uses only that key's accidental type.
- [ ] **AC-P0-4** (G1) `.github/workflows/ci.yml` defines jobs for Rust on macOS and Linux, UI lint/type check/Vitest, Playwright (desktop and iPad viewports), stale bindings, universal `tauri build` and `tauri ios build` for the Simulator, and each exits 0 on the pull request for this run.
- [ ] **AC-P0-5** (G1) The UI calls `fretboard_view` through generated bindings and draws the neck; a Playwright test checks 6 strings and 22 frets with the default settings (G-4).
- [ ] **AC-FB-2.1** Key F major, notes view: labels F G A Bb C D E, no A#.
- [ ] **AC-FB-2.2** Key G major, IV selected, key changed to D major: G is selected.
- [ ] **AC-FB-4.1** Key C, type `D7`: all tones D, F#, A, C; badge "Not in key (V/V)".
- [ ] **AC-FB-5.1** Standard tuning, C, voicing view: first voicing is x32010 with fingers x-3-2-0-1-0.
- [ ] **AC-FB-5.2** Standard tuning, F: list contains xx3211 and 133211, both obeying FB-5.4.
- [ ] **AC-FB-5.3** Any tuning and chord: no voicing exceeds span 4 (stretches off) or 4 fingers (property test).
- [ ] **AC-FB-6.1** Key C, chord C, string set 234, first inversion: E (string 4 fret 2), G (string 3 fret 0), C (string 2 fret 1).
- [ ] **AC-FB-6.2** Chord G, string set 123, show all: three inversions in fret order, labelled root, 1st, 2nd.
- [ ] **AC-FB-7.1** Chord Am, position 5: every A, C and E between frets 4 and 9 inclusive, finger 1 on fret 5.
- [ ] **AC-FB-9.1a** Criterion benchmark: `fretboard_view` ≤ 2 ms and voicing search ≤ 5 ms for one chord on the full neck (release).
- [ ] **AC-FB-9.1b** Playwright: each cycling key pressed 100 times, each redraw completes in ≤ 16 ms (mocked command layer).
- [ ] **AC-FB-9.1c** Each cycling change writes its announcement ("E minor, first inversion, strings 2 3 4") to the ARIA live region.
- [ ] **AC-FB-1.6** The tooltip for string 3 fret 0 with key G and chord G contains "G3", "1" (degree), "R" (interval) and "Root".
- [ ] **AC-FB-5.6** A chord and tuning with no voicing under FB-5.4 returns the "No playable voicing in this tuning" result with both relaxation options; nothing that breaks FB-5.4 is returned.
- [ ] **AC-E-FB-4** Capo 5, selection in position 2: the selection moves to the first position at or above the capo with the notice "Moved above the capo".
- [ ] **AC-E-FB-8** With no audio output device, `fretboard_view` and every explorer command still succeed and Play returns the "No audio output" error once.
- [ ] **AC-SC-3.1** A minor pentatonic box 1: frets 5 and 8 on strings 6, 2 and 1; frets 5 and 7 on strings 5, 4 and 3; roots at string 6 fret 5, string 4 fret 7, string 1 fret 5.
- [ ] **AC-SC-4.1** Key C, chord changes C → A7 with follow-the-chords: the overlay scale changes from C major to A Mixolydian (pure function test).
- [ ] **AC-LH-1.1** Left-handed on: nut drawn on the right, frets mirrored, tooltips unchanged (Playwright).
- [ ] **AC-LH-4.1** Drop D, chord D: the list includes a six-string D with string 6 open (D), every pitch correct for D A D G B E.
- [ ] **AC-LH-5.1** Capo 2, chord shape G: label "G shape (sounds A)".
- [ ] **AC-TU-1.1** Synthetic 440.0 Hz, A4 = 440: A4, 0 ± 1 cent, "In tune" within 400 ms of simulated input.
- [ ] **AC-TU-1.2** Synthetic guitar-like tones (harmonic-rich, decaying) for each open string detuned −20, −5, +5, +20 cents: offset within ±3 cents of truth. Recorded guitar fixtures replace the synthetic set when J records them *(follow-up)*.
- [ ] **AC-NF-9** First reading ≤ 150 ms after a synthetic note starts, and ≥ 20 readings per second.
- [ ] **AC-TU-1.3** Synthetic 73.42 Hz: D2.
- [ ] **AC-TU-2.1** DADGAD, 146.83 Hz: string 4 (D), in tune.
- [ ] **AC-TU-3.1** A4 = 442, input 440 Hz: A4, about −8 cents.
- [ ] **AC-TU-5.1** Capo 2, standard: targets F#2 B2 E3 A3 C#4 F#4.
- [ ] **AC-E-TU-2** Permission denied: message and settings button, no crash *(manual on device)*.
- [ ] **AC-UX-7** No two commands in the Explore area share a shortcut (unit test over the shortcut table).
- [ ] **AC-FR-028** Settings saved, reloaded from the JSON file, and "reset Display" restores only the Display group to G-4 defaults (unit test).
- [ ] **AC-NF-24** A command that panics returns `AppError` "Something went wrong" instead of crashing (unit test in `src-tauri`).
- [ ] **AC-MAN-1** *(manual)* On a Mac: app launches, G major shows on the neck, Space plays a strum, the tuner reads a played string. In the iPad Simulator: the app launches and the neck renders in landscape and portrait.

## Dependencies and Risks

| Item | Type | Notes |
|------|------|-------|
| Tauri 2 (stable), `@tauri-apps/cli`, `@tauri-apps/api` | Dependency | Pin exact versions in `Cargo.toml` workspace deps and `package-lock.json`. |
| `cpal`, `rtrb` | Dependency | Audio I/O; Linux needs ALSA headers (CI installs `libasound2-dev`). |
| `ts-rs` | Dependency | Bindings generation; chosen over tauri-specta because tauri-specta 2 is still pre-release. Record in ADR 0007's notes. |
| `thiserror`, `serde`, `proptest`, `insta`, `criterion` | Dependency | Standard picks from AGENTS.md. |
| GitHub Actions macOS runners with Xcode | Dependency | Needed for `tauri build` and `tauri ios build`; no Apple account required for unsigned Simulator builds. |
| Tauri iOS generated project (`src-tauri/gen/apple`) | Risk | Generated by `tauri ios init`, which needs macOS. Mitigation: CI runs `tauri ios init` before `tauri ios build`; the generated project is not committed in this run. |
| iOS microphone and audio session without the Swift plugin | Risk | cpal on iOS does not set the `AVAudioSession` category, so input may not work in the Simulator. Mitigation: the tuner's analysis is tested without a device; the `audio-session` Swift plugin is a stretch goal for this run, otherwise first item of the next run, before device validation. |
| No macOS machine in the cloud dev environment | Risk | Development happens on Linux; macOS-only steps are proven only in CI. Mitigation: keep macOS-specific code minimal; iterate on CI logs. |
| WebKit in Playwright on Linux | Risk | Playwright WebKit runs on Linux with system deps; CI installs them with `playwright install --with-deps webkit`. |
| Synthetic tuner fixtures | Risk | Real guitar recordings may behave differently (inharmonicity, noise). Mitigation: harmonic-rich decaying test tones with noise; recorded fixtures as a follow-up. |
| Coverage ≥ 80% with an app crate | Risk | `src-tauri` command glue and cpal stream code are hard to unit-test. Mitigation: keep logic in pure functions; exclude only the thin device and Tauri-runtime glue from coverage, with a comment saying why. |

## Open Questions

| Question | Owner | Resolution path |
|----------|-------|-----------------|
| Are synthetic tuner fixtures enough for the P1 exit, or must recorded guitar WAVs pass first? | J | Default: synthetic is enough for this run; recorded WAVs (`crates/mackey-audio/tests/fixtures/`) become a follow-up AC before device validation. |
| Is one synthesised voice acceptable until P1b (FB-8.2 deferred)? | J | Default: yes; recorded in `docs/prd/09-traceability.md` open items. |
| SoundFont or synthesised tones for selection playback long term? | J / P1b | Synthesised in this run; P1b's asset work decides, per [sound-assets.md](docs/sound-assets.md). |
| Swift `audio-session` plugin in this run or the next? | Build | Default: next run, unless the Simulator CI build is green early in this run. Decide by the end of the implementation step. |
| Should the explorer's settings move into `mackey-store` when it lands in P2? | Build | Resolved: yes, in P2; the JSON file is a stopgap owned by `src-tauri`. |

## Review log

- 2026-11-15, `/dev-flow:review-prd` (independent reviewer). Verdict: **Needs revision**, no Fail dimensions. Warnings: functional completeness (FB-1.5, FB-3, E-FB-5..7, E-TU-5..6 unassigned; FR-024 drifted from FB-8.2), acceptance criteria (AC-P0-4 not testable, AC-FB-9.1 bundled, no ACs for FB-1.6, FB-5.6, E-FB-4, E-FB-8, NF-9, FR-028). Minor: citation slips, settings list, unmeasured NFRs, open-question defaults.
- Fixes applied in the same pass: every uncovered item assigned to an FR; FB-8.2 explicitly deferred to P1b; AC-P0-4 rewritten; AC-FB-9.1 split in three; eight ACs added; citations, settings list and open questions corrected, with defaults for the three owner questions. FR bundling kept, since each clause cites the ID it implements. Result: **Ready for spec.**
