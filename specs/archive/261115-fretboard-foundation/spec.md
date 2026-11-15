# Spec: Fretboard Foundation (roadmap P0 and P1)

**Feature:** fretboard-foundation
**Created:** 2026-11-15
**Status:** Specified (reviewed in Step 2, see [implementation-notes.md](implementation-notes.md#spec-review-step-2))
**Source PRD:** [`specs/archive/261115-fretboard-foundation/fretboard-foundation-PRD.md`](fretboard-foundation-PRD.md)
**Authority:** `INTENT.md` → `docs/prd/` → `docs/` → this spec. Where this spec and `docs/prd/` differ, `docs/prd/` wins and this spec is fixed. FR numbers are the dev-flow working PRD's; AC numbers are the product PRD's.

## Executive Summary

Mackey Guitar Tutor has design documents and repository tooling but no product code. This feature builds the foundation every later phase stands on: a Tauri 2 app (macOS universal, iPad via the iOS Simulator) with a React/TypeScript UI whose SVG fretboard is drawn entirely from a `FretboardView` computed in Rust; the pure `mackey-theory` and `mackey-voicing` crates (notes, keys, chords, voicings, triads, arpeggios, scale patterns, tunings, capo, selection and cycling); and `mackey-audio` with a plucked-string synthesiser and a tuner whose pitch detector is proven against synthetic signals. It ends with CI that builds and tests all of it on Linux and macOS, including a universal Mac build and an iOS Simulator build.

## Problem Statement

Beginners and improvers learn chord shapes by rote and stay in the first few frets (INTENT: Purpose). They cannot see where the notes of a key, chord, triad, arpeggio or scale sit on the neck, cannot step through them quickly, and have no in-app way to tune. Nothing in the repository yet draws a neck, spells a note or opens an audio device.

## Goals

- **G1 Walking skeleton on both targets.** A Tauri 2 app whose UI calls a typed Rust command and draws the neck; builds as a universal macOS app and for the iOS Simulator in CI; plays a sound through the Rust audio path.
- **G2 Correct theory core.** `mackey-theory` spells every note, interval, scale, mode, key and chord correctly (G-1), parses the chord-symbol grammar and Roman numerals, and is proven by table-driven and property tests.
- **G3 Fretboard explorer.** Pick a key, step I to vii°, and see open shapes, barre/CAGED shapes, triads per string set and inversion, an arpeggio in position, scale overlays and pentatonic boxes; left- or right-handed; any preset tuning; with a capo (FB-1..9, SC-1..5, LH-1..5).
- **G4 Tuner.** Chromatic and string-aware, within ±3 cents from the default input device (TU-1..7), pitch detection proven on synthetic signals.

## Non-Goals

Taken from the PRD and kept binding:

- Rhythm and the transport bar (P1b), practice sessions (P2), songs (P3 to P5), CAGED maps, tips, progress, ear training, looper, mic recognition (P6 to P8).
- `mackey-store`, SQLite, library export/import, Recently deleted and drafts (G-6). Settings persist as one JSON file owned by `src-tauri` (FR-028).
- Bundled SoundFont playback and the nylon/steel/electric choice (FB-8.2 is deferred to P1b). One synthesised steel-like voice ships.
- Release automation (`release.yml`, ad-hoc signed `.dmg`), unless time allows after everything else is green.
- Real iPad validation, Apple signing, App Store, iCloud.
- Windows, Linux, iPhone and Android as products (Linux CI compiles and tests only).
- First-launch onboarding F-1, multiple windows (UX-4.1), menus mirroring every command (UX-4.2).
- The `audio-session` Swift plugin (stretch goal; default is the next run, see Open Questions Q-4).
- Recorded guitar fixtures for the tuner (follow-up AC; synthetic fixtures gate this run).
- Home, Sessions, Songs and the transport bar in the sidebar. The sidebar has Explore, Tools (Tuner) and Settings only.
- Memory (NF-12), app size (NF-13) and visual sync (NF-8.2) measurements.
- Undo and redo of setting and selection changes (G-6.1 applies to sessions, songs and settings; with no session editor yet it is deferred to P2, see Q-10). Settings changes apply immediately and can be reverted by the player, but `Cmd-Z` does nothing in this run.
- The hold-Cmd iPad shortcut overlay (UX-5.3) and pinch-zoom on the iPad neck (UX-5.1) are SHOULD items: pinch-zoom is built in task D9 as the last item and may be cut; the overlay is not built.
- Extended chord symbols beyond the FB-4.3 list (E-FB-2 gives `Cmaj13#11` as an example of an accepted unusual symbol): deferred, see Q-9. Chords are limited to the 21 qualities of FR-010.
- Network or external-service failure handling: there is no network or external API in this run (NF-7), so no such failure paths exist. The failure paths that do exist are the local audio, microphone, settings-file and panic paths listed in implementation-notes.md.

## User Requirements

Stories are `US-FB-1..9` and `US-TU-1..3` in [docs/prd/01-fretboard.md §2](../../../docs/prd/01-fretboard.md) and [02-tuner.md §2](../../../docs/prd/02-tuner.md). Personas: J (primary), Bea (beginner), Ivan (improver).

### Functional Requirements

Detail and edge cases live in the cited product requirement and are part of the FR. "Builder notes" record decisions this spec makes where the product PRD leaves a choice; each has a decision ID in [implementation-notes.md](implementation-notes.md#technical-decisions).

#### Scaffold and boundary

| ID | Requirement | Product reqs | Crate / path | Phase |
|----|-------------|--------------|--------------|-------|
| FR-001 | Workspace members `crates/mackey-theory`, `crates/mackey-voicing`, `crates/mackey-audio`, `src-tauri` (crate `mackey-app`), each with a `README.md`; `make archcheck` passes. The xtask layer table and the AGENTS.md table already contain these crates; `mackey-audio` depends on `mackey-theory` only in this run. `CLAUDE.md` already exists. | architecture §2 | `Cargo.toml`, `crates/*`, `src-tauri/` | A |
| FR-002 | Tauri 2 app in `src-tauri/`; React + TypeScript (strict) + Vite UI in `ui/`. Window minimum 900×600, default 1280×800. Strict CSP, no remote content; capabilities allow only the app's own commands. | UX-4, NF-7.3 | `src-tauri/tauri.conf.json`, `src-tauri/capabilities/`, `ui/` | D |
| FR-003 | Boundary types derive `serde` and (behind a `ts` feature) `ts_rs::TS`; bindings generate into `ui/src/bindings/` (never hand-edited) including typed command wrappers; `make bindings` regenerates; CI fails if stale. One-line note added to ADR 0007 (ts-rs chosen because tauri-specta 2 is pre-release). | ADR 0007 | `ui/src/bindings/`, `Makefile`, `docs/adr/0007-*.md` | D, E |
| FR-004 | Every command returns `Result<T, AppError>`; `AppError` serialisable with plain message and optional details; panics inside a command are caught and returned as "Something went wrong". | NF-22, NF-24 | `src-tauri/src/error.rs`, `guard.rs` | D |
| FR-005 | Make targets `ui-lint`, `ui-test`, `e2e`, `bindings`, `app`, `ios-sim` each with an AGENTS.md row; `make check` includes UI lint, type check and unit tests. | AGENTS.md | `Makefile`, `AGENTS.md` | E |
| FR-006 | CI jobs: UI lint+type check+Vitest; Playwright (WebKit; 1280×800 and 1180×820); stale bindings; universal `tauri build` (unsigned); `tauri ios build` for the Simulator (unsigned compile check). Linux Rust jobs install WebKitGTK and ALSA dev packages. All required. iPad walking-skeleton exit = green Simulator job + Playwright iPad viewport; audible output/mic in the Simulator is manual (AC-MAN-1). | roadmap P0 | `.github/workflows/ci.yml` | E |

#### Theory core: `mackey-theory`

| ID | Requirement | Product reqs |
|----|-------------|--------------|
| FR-007 | `PitchClass`, `Letter`, `Note` (spelled), `Pitch` (MIDI and frequency with reference A4), `Interval` (quality + number), validated at construction. | architecture §3.1 |
| FR-008 | Scales and modes: Major, Natural minor, Harmonic minor, Dorian, Phrygian, Lydian, Mixolydian, Locrian, Major pentatonic, Minor pentatonic, Blues; correct degree spelling for the tonic. | FB-2.1, SC-1 |
| FR-009 | Keys per G-1.2 and G-1.3 (F#/Gb and D#/Eb choices), modes with parent key signature and enharmonic tonic substitution (G-1.4), next/previous around the circle of fifths, relative major/minor, enharmonic alternative for a 6-accidental key. | G-1, FB-2.4, E-FB-7 |
| FR-010 | `ChordQuality` and chord-symbol parser for the FB-4.3 grammar plus alternative spellings (`min`, `-`, `M7`, `Δ`, `ø`, `°`, `+`); positioned errors; suggestion when edit distance ≤ 2 or root `H`; player's root spelling kept. | FB-4.3, E-FB-1, G-1.7 |
| FR-011 | Diatonic chords (triads and sevenths) with Roman numerals; Roman-numeral parsing (`vi` in G → Em); "not in key" analysis with function (secondary dominants `V/x`, borrowed `bVII`). | FB-4.2, FB-4.5 |
| FR-012 | Chromatic spelling per G-1.5; display spelling per G-1.6 (no double accidentals as fretboard labels). | G-1.5, G-1.6 |

#### Voicing and fretboard: `mackey-voicing`

| ID | Requirement | Product reqs |
|----|-------------|--------------|
| FR-013 | Nine tuning presets and custom tunings with "unusual tension" warning; `Instrument` (tuning, frets 19..=24, capo 0..=12); `Handedness`. | LH-1..3, G-5 |
| FR-014 | Curated open shapes exactly as FB-5.3 (small and full barre F), CAGED barre shapes, standard tuning plus transposed for Half/Whole-step down; Drop D curated where different. | FB-5.3, LH-4 |
| FR-015 | Voicing search obeying FB-5.4; scoring; groups Open / Barre and CAGED / Other; fingering and barres; "Found by search" outside curated tunings; identical open strings not double-counted; "No playable voicing" result with the stretch and omitted-fifth options. | FB-5.1..6, LH-4, E-FB-3, E-FB-6 |
| FR-016 | Triads on string sets 123/234/345/456 in root, first, second inversion; "show all on string set"; seventh-chord note. | FB-6.1..4 |
| FR-017 | Arpeggio in position (4-fret window ±1), suggested fingers, play order up/down/up-down, default-window rule. | FB-7.1..4 |
| FR-018 | Scale patterns: full neck, five pentatonic boxes, seven 3-notes-per-string patterns; next/previous position; "Follow the chords" scale choice as a pure function. | SC-1..5 |
| FR-019 | `FretboardView`: one serialisable value with every visible position (string, fret, spelled note, octave, role, labels for all modes, finger), open/muted markers, barres, dimmed frets below the capo, capo indicator, inlays, fret numbers, string labels, list position, notices, accessibility label per position, tooltip data; key-notes view for focus "none"; off-range counts per edge; "fit selection" range. | FB-1, FB-3, G-2, G-3.2, NF-5 |
| FR-020 | Selection model (G-3) and pure cycling actions (FB-9); key change keeps the degree; tuning/capo change keeps the chord and reselects the best voicing with a notice. | G-3, FB-9, E-FB-4 |

#### App and UI

| ID | Requirement | Product reqs |
|----|-------------|--------------|
| FR-021 | Shared SVG fretboard component: strings, frets, nut, inlays, numbers, string labels, 12th-root-of-2 spacing with minimum widths and horizontal scroll, G-2 shapes and Okabe-Ito tokens in light/dark, left-hand mirroring and string flip, capo dimming, tooltip (hover/tap), range controls, "Fit selection", edge arrows; every position an accessible element. | FB-1, LH-1, LH-3, E-FB-5, NF-5 |
| FR-022 | Explore screen: toolbar (key, tuning and capo, labels, view type), diatonic chord strip with sevenths toggle, chord field with inline errors, voicing list with mini diagrams, triad/arpeggio/scale controls, counters ("3 of 7"), "Not in key" badge, capo shape/sounding names. | FB-2, FB-4, FB-5.2, FB-9.2, LH-5 |
| FR-023 | Keyboard shortcuts (FB-9 and 07 §4 Explore rows, `Cmd-T`), disabled while a text field has focus; no two commands in an area share a key; ARIA live announcements; iPad swipes and buttons; 44 pt targets. | FB-9, UX-5.1, UX-6, UX-7 |
| FR-024 | Sound: Play = strum (30 ms apart), arpeggio in eighths at 70 bpm, or ascending key scale; click a position plays it; mute toggle; one synthesised steel-like voice via cpal; callback obeys real-time rules; "No audio output" shown once. FB-8.2 deferred. | FB-8.1, FB-8.3, FB-8.4, FB-1.7, E-FB-8 |
| FR-025 | Sidebar (Explore, Tools→Tuner, Settings), iPad landscape and portrait layouts via CSS breakpoints, narrow-window behaviour. | UX-1, UX-5, E-FB-9 |

#### Tuner

| ID | Requirement | Product reqs |
|----|-------------|--------------|
| FR-026 | Pure pitch-detection module in `mackey-audio` (MPM): 60 to 1,400 Hz, ≥ 20 readings/s from a 2048-sample window, median-of-3 smoothing, cents vs reference A4, confidence, level, too quiet/too loud. | TU-1, TU-3, E-TU-3, E-TU-4, NF-9 |
| FR-027 | Tuner screen (`Cmd-T`): chromatic display, "In tune" ±3 cents held 300 ms (colour + check + text), "Play a single string" after 500 ms; string mode with targets for tuning and capo, auto-select, Tighten/Loosen, tap-to-lock, ticks, "Far from any string"; reference tones; device-disconnect fallback; two-strings-ringing behaviour; device picker and level meter; mic explanation and permission-denied states; `NSMicrophoneUsageDescription`; opening the tuner stops selection playback; `tuner_reading` event at about 30 Hz. | TU-1..7, E-TU-1..6 |

#### Settings

| ID | Requirement | Product reqs |
|----|-------------|--------------|
| FR-028 | Instrument, Display, Sound, Tuner settings used by this run (handedness, flip strings, frets, default tuning, allow stretches, theme, labels default, show fret numbers, playback volume and mute, input device, reference A4), G-4 defaults, applied immediately, persisted as JSON in the app data directory, reset per group. Instrument selector hidden until P1b; "Show tips" waits for P6. | 08 §6 |

### Builder clarifications (binding for implementers)

1. **Selection state lives in Rust.** `src-tauri` owns the `Selection`, `Instrument` and `Handedness` behind a mutex; the UI sends actions and receives a snapshot (`FretboardView` plus panel data). The UI never recomputes theory (D-01).
2. **String numbers** in public APIs are 1..=6 with 1 the high string (glossary). Arrays are stored low to high (index 0 = string 6) as in architecture §3.2 (D-02).
3. **Fret numbers are absolute from the nut.** An unfretted string under a capo is `Open` and sounds at the capo fret (D-03).
4. **The selected chord is the sounding chord.** With a capo, a voicing's label shows the shape name and the sounding name ("G shape (sounds A)") (D-04).
5. **Chord field** accepts chord symbols and Roman numerals (FB-4.1 b, c); items (d) clicking a chord elsewhere belong to later phases.

## Non-Functional Requirements

| Area | Requirement | Check |
|------|-------------|-------|
| Performance | `fretboard_view` ≤ 2 ms; voicing search for one chord on the full neck ≤ 5 ms p95 (release) (NF-1, NF-1.1). Neck redraw ≤ 16 ms per cycling action. Playback ≤ 30 ms after action plus device latency (NF-3.1). Cold start ≤ 2 s on Mac (NF-3). | criterion benches plus a release-mode timing test; Playwright timing test with mocked commands; the last two manual |
| Reliability | No panics across IPC (NF-24). Parsers never panic (property test). Errors never discard input (NF-23). Audio callback never allocates, locks or does I/O (NF-8.1). No audio device: all visual features work. | `catch_unwind` guard test; proptest; `assert_no_alloc` in debug tests; no-device test |
| Security and privacy | No network, telemetry or accounts (NF-7). Mic audio in memory only, permission requested only when the tuner first opens (NF-7.1, NF-7.2). Strict CSP, minimal capabilities (NF-7.3). `unsafe_code` forbidden; `cargo deny` clean. | archcheck, deny, CSP test |
| Accessibility | Colour-blind-safe palette plus shapes (G-2, NF-5.2); keyboard control with visible focus (NF-5.1); VoiceOver labels and announcements (NF-5); Reduce Motion (NF-5.4); text size, 11 pt minimum neck labels (NF-5.3). | Vitest, Playwright, manual VoiceOver |
| Observability | `tracing` in `src-tauri` and `mackey-audio` (never in the audio callback); command errors logged with full chain to a local log file; nothing sent anywhere. | unit test on the log layer |
| Quality | `make check` and CI green on macOS and Linux; line coverage ≥ 80% overall; new pure-crate code fully covered by tests naming their requirement ID; table-driven, property and insta tests per AGENTS.md. | `make cover` |
| Compatibility | macOS 13+ universal; iPadOS 17+ (Simulator); 44.1 and 48 kHz. | CI builds; manual |

## System Architecture

Full design in [architecture.md](architecture.md); data shapes in [data-dictionary.md](data-dictionary.md).

Affected layers and components:

| Layer | Crate or folder | Change |
|-------|-----------------|--------|
| Pure core | `crates/mackey-theory` | New. Notes, intervals, scales, keys, chords, parser, Roman numerals, spelling. |
| Pure core | `crates/mackey-voicing` | New. Tunings, curated shapes, search, triads, arpeggios, scale patterns, selection, `FretboardView`, playback plan. Depends on theory only. |
| I/O | `crates/mackey-audio` | New. Pure `dsp` (pitch detector, tuner logic, Karplus-Strong synth) plus device glue (cpal output and input, rtrb queues, analysis thread). Depends on theory only. |
| Shell | `src-tauri` (`mackey-app`) | New. Commands, state, settings, errors, events, Tauri config and capabilities. |
| UI | `ui/` | New. React + TypeScript + Vite; fretboard SVG, Explore, Tuner, Settings. |
| Repo | `Makefile`, `AGENTS.md`, `.github/workflows/ci.yml`, `Cargo.toml`, `xtask/` | Modified. |

## Scope of Changes

Files to create:

- `crates/mackey-theory/{Cargo.toml,README.md,src/**,tests/**,benches/**}`
- `crates/mackey-voicing/{Cargo.toml,README.md,src/**,tests/**,benches/**}`
- `crates/mackey-audio/{Cargo.toml,README.md,src/**,tests/**,benches/**}`
- `src-tauri/{Cargo.toml,build.rs,tauri.conf.json,Info.plist,capabilities/default.json,icons/**,README.md,src/**}`
- `ui/{package.json,package-lock.json,tsconfig.json,vite.config.ts,eslint.config.js,.prettierrc,index.html,playwright.config.ts,README.md,src/**,tests/**,e2e/**}`; `ui/src/**` and `src-tauri/src/**` are skipped by `doccheck` (it skips `src`, `tests`, `benches`, `node_modules`, `dist`, `gen`, ...), but `ui/e2e/`, `src-tauri/capabilities/` and `src-tauri/icons/` are not and each needs a `README.md`; `ui/src/bindings/README.md` is added anyway to say the folder is generated
- `specs/archive/261115-fretboard-foundation/*` (this spec)

Files to modify:

- `Cargo.toml` (members, workspace dependencies, `[profile.dev.package.*]` for proptest/insta)
- `Makefile` (new targets and a `ui/dist` stub rule)
- `AGENTS.md` (Commands table rows; CI/CD "Not yet in place" paragraph)
- `.github/workflows/ci.yml`, `.github/README.md`
- `README.md`, `crates/README.md`, `docs/adr/0007-web-ui-and-ipc-boundary.md` (ts-rs note), `docs/prd/09-traceability.md` (open items), `docs/roadmap.md` (phase status), `docs/architecture.md` (command and event list, `roman_to_chord` returns `Result`), `user-docs/` (Step 4)
- `.gitignore` (already covers `ui/dist`, `node_modules`, `src-tauri/gen/*`)
- `xtask/src/doccheck.rs` and `xtask/README.md`: skip generated report folders `playwright-report`, `test-results` and `coverage` so a local test run does not fail `make doccheck` (task E9; the repo's own TDD rules apply to the xtask change)
- `ui/.nvmrc` and `ui/package.json` `engines` (Node 22 LTS), so CI and contributors agree

New dependencies (exact versions pinned at implementation; the "latest stable at that time" rule applies):

| Area | Crates / packages |
|------|-------------------|
| Rust workspace | `serde` (derive), `thiserror` 2, `ts-rs`, `proptest`, `insta` (yaml), `criterion`, `rtrb`, `cpal`, `tracing`, `tracing-subscriber`, `tracing-appender`, `tauri` 2, `tauri-build` 2, `assert_no_alloc` (dev) |
| UI | `react`, `react-dom`, `zustand`, `@tauri-apps/api` 2, `vite`, `@vitejs/plugin-react`, `typescript`, `vitest`, `@testing-library/react`, `jsdom`, `eslint` (flat config) with `typescript-eslint`, `eslint-plugin-react-hooks`, `eslint-plugin-jsx-a11y`, `prettier`, `@playwright/test`, `@tauri-apps/cli` 2 |

`deny.toml` may need additions (for example a licence such as `MPL-2.0` or `BSL-1.0` if a Tauri transitive dependency needs it); any addition is recorded in an ADR note and must stay permissive (OQ-6).

## Breaking Changes

None for users. Internal contract changes:

- `docs/architecture.md` lists `roman_to_chord(s, key) -> Option<Chord>`; this spec returns `Result<Chord, RomanParseError>` so the UI can show why (D-18). Update the doc in Step 4.
- `make check` gains UI steps; contributors need Node 22 LTS and `npm` for it.
- Linux Rust jobs need system packages; `make build/test/lint/cover` create a stub `ui/dist/index.html` when missing so `tauri::generate_context!` compiles (D-15).

## Success Criteria

1. A beginner picks G major, steps I to vii°, and for each chord sees the open shape, CAGED shapes, triads on each string set and an arpeggio in position, in any preset tuning, left- or right-handed (roadmap P1 exit), and A minor pentatonic boxes 1 to 5 cycle cleanly.
2. The tuner reads within ±3 cents on the synthetic fixtures (AC-TU-1.x, AC-NF-9).
3. CI is green on the pull request: Rust on macOS and Linux, UI lint/type/Vitest, Playwright desktop and iPad, stale bindings, universal build, iOS Simulator build.
4. `make check` passes locally; coverage ≥ 80%.

### Acceptance Criteria

Each maps to at least one automated test unless marked manual. Owners are tasks in [tasks.md](tasks.md); the full matrix is in [plan.md](plan.md#acceptance-criteria-traceability).

| AC | Statement (short) | FRs | Primary test home |
|----|-------------------|-----|-------------------|
| AC-P0-1 | Every chord quality in all 12 keys spelled per a hand-written table | FR-010, 012 | `mackey-theory/tests/chord_spelling.rs` |
| AC-P0-2 | Parser accepts every quality and alternative spelling with every root and slash bass; `Hm7` suggests `Bm7`; never panics | FR-010 | `mackey-theory/tests/chord_parser.rs`, proptest |
| AC-P0-3 | Transpose, interval and key-accidental properties | FR-007, 009, 012 | `mackey-theory/tests/properties.rs` |
| AC-P0-4 | `ci.yml` defines all listed jobs and each exits 0 on the PR | FR-006 | `.github/workflows/ci.yml` (verified by the green PR) |
| AC-P0-5 | UI calls `fretboard_view` via bindings, draws 6 strings and 22 frets | FR-003, 021 | `ui/e2e/neck.spec.ts` |
| AC-FB-2.1 | F major notes view labels F G A Bb C D E | FR-008, 009, 019 | `mackey-voicing/tests/view_key_notes.rs` |
| AC-FB-2.2 | G major IV, key to D major, G selected | FR-020 | `mackey-voicing/src/selection/tests` |
| AC-FB-4.1 | Key C, `D7`: D F# A C; "Not in key (V/V)" | FR-010, 011, 019 | `mackey-theory/tests/analysis.rs`, view test |
| AC-FB-5.1 | C voicing view first is x32010, fingers x-3-2-0-1-0 | FR-014, 015 | `mackey-voicing/tests/voicings_standard.rs` |
| AC-FB-5.2 | F list contains xx3211 and 133211 | FR-014, 015 | same |
| AC-FB-5.3 | No voicing breaks span 4 / 4 fingers (any tuning, chord) | FR-015 | `mackey-voicing/tests/properties.rs` |
| AC-FB-6.1 | C, strings 234, 1st inversion: E s4 f2, G s3 f0, C s2 f1 | FR-016 | `mackey-voicing/tests/triads.rs` |
| AC-FB-6.2 | G, string set 123, show all: three inversions in fret order | FR-016 | same |
| AC-FB-7.1 | Am position 5: every A, C, E in frets 4..=9, finger 1 on fret 5 | FR-017 | `mackey-voicing/tests/arpeggios.rs` |
| AC-FB-9.1a | Benchmarks: `fretboard_view` ≤ 2 ms, search ≤ 5 ms | FR-019, 015 | `benches/`, release timing test |
| AC-FB-9.1b | Each cycling key 100 times, redraw ≤ 16 ms | FR-023 | `ui/e2e/cycling-perf.spec.ts` |
| AC-FB-9.1c | Each change writes its announcement to the live region | FR-023 | Vitest + `ui/e2e/live-region.spec.ts` |
| AC-FB-1.6 | Tooltip string 3 fret 0, G and G: "G3", "1", "R", "Root" | FR-019, 021 | view test + `ui/e2e/tooltip.spec.ts` |
| AC-FB-5.6 | No-voicing result with both options; nothing breaking FB-5.4 returned | FR-015 | `mackey-voicing/tests/no_voicing.rs` |
| AC-E-FB-4 | Capo 5, position 2 → first position at or above capo, "Moved above the capo" | FR-020 | selection tests |
| AC-E-FB-8 | No audio device: all explorer commands succeed; Play errors once | FR-024 | `src-tauri` tests with a fake audio port |
| AC-SC-3.1 | A minor pentatonic box 1 frets and roots | FR-018 | `mackey-voicing/tests/scale_patterns.rs` |
| AC-SC-4.1 | C → A7 follow-the-chords: C major → A Mixolydian | FR-018 | same |
| AC-LH-1.1 | Left-handed: nut right, mirrored, tooltips unchanged | FR-021 | `ui/e2e/left-handed.spec.ts` |
| AC-LH-4.1 | Drop D, D chord: six-string D with string 6 open, pitches correct | FR-014, 015 | `mackey-voicing/tests/tunings.rs` |
| AC-LH-5.1 | Capo 2, G shape: "G shape (sounds A)" | FR-019, 022 | `mackey-voicing/tests/capo.rs` |
| AC-TU-1.1 | 440.0 Hz → A4, 0 ± 1 cent, "In tune" within 400 ms | FR-026, 027 | `mackey-audio/tests/tuner_fixtures.rs` |
| AC-TU-1.2 | Synthetic guitar tones ±20/±5 cents within ±3 cents | FR-026 | same |
| AC-NF-9 | First reading ≤ 150 ms after onset; ≥ 20 readings/s | FR-026 | same |
| AC-TU-1.3 | 73.42 Hz → D2 | FR-026 | same |
| AC-TU-2.1 | DADGAD, 146.83 Hz → string 4 (D), in tune | FR-027 | `mackey-audio/tests/tuner_strings.rs` |
| AC-TU-3.1 | A4 = 442, 440 Hz input → A4, about −8 cents | FR-026 | `tuner_fixtures.rs` |
| AC-TU-5.1 | Capo 2 standard targets F#2 B2 E3 A3 C#4 F#4 | FR-027 | `tuner_strings.rs` + `mackey-app` target test |
| AC-E-TU-2 | Permission denied: message and settings button, no crash | FR-027 | manual on device; Playwright covers the rendered state |
| AC-UX-7 | No two commands in the Explore area share a shortcut | FR-023 | `ui/src/shortcuts.test.ts` |
| AC-FR-028 | Settings save, reload; "reset Display" resets only Display | FR-028 | `src-tauri/src/settings.rs` tests |
| AC-NF-24 | Panicking command returns `AppError` "Something went wrong" | FR-004 | `src-tauri/src/guard.rs` tests |
| AC-MAN-1 | Manual: Mac launch, G major, Space strum, tuner reads; iPad Simulator renders landscape and portrait | FR-001..027 | manual checklist in `implementation-notes.md` |

#### Derived acceptance criteria

The PRD's ACs leave several FRs without a testable statement (found in the Step 2 review). These builder-added criteria close the gaps; each is automated and owned by a task. They use the prefix `SAC` (spec acceptance criterion) so they are never confused with the product PRD's IDs; if J promotes one to the product PRD it takes an `AC-` ID there.

| SAC | FR | Statement | Task |
|-----|----|-----------|------|
| SAC-01 | 001 | `make archcheck` and `make doccheck` pass with the four new members; each has a README | A1, D1 |
| SAC-02 | 002 | `tauri.conf.json` has window 1280×800 (min 900×600), a CSP with no remote origin, and `capabilities/default.json` lists exactly `allow-<command>` for every registered command and nothing else | D1 |
| SAC-03 | 004 | `AppError` serialises as `{ kind, message, details }`; each core error maps to the plain message in the table; no panic text appears in `message` | D2 |
| SAC-04 | 005 | Every new `.PHONY` target has an AGENTS.md row (`make doccheck`), and `make check` runs UI lint, type check, Vitest and the bindings check | E1, E2 |
| SAC-05 | 009 | `next_fifth` x12 returns to the start for every offered major and minor key; `relative` round trips; D# Dorian becomes Eb Dorian with a notice; a 6-accidental key offers its enharmonic | A5 |
| SAC-06 | 011 | `vi` in G gives Em; Roman numerals round trip through `Display`/`FromStr`; diatonic triads and sevenths match a hand-written table for all 12 major keys; `bVII` in C is Bb, borrowed | A9 |
| SAC-07 | 013 | The nine presets have the exact pitches of LH-2; a custom string -3 or +2 semitones from standard is accepted silently and -4 or +3 warns | B1 |
| SAC-08 | 014 | Every FB-5.3 curated shape matches a hand-written fret string and sounds only chord tones | B2 |
| SAC-09 | 015 | In every voicing of a slash chord the lowest sounding pitch class is the bass; voicings in non-curated tunings are `Origin::Search` | B5 |
| SAC-10 | 019 | For any selection every `ViewPosition` pitch equals `Instrument::pitch_at`, no label has a double accidental, positions below the capo are never highlighted, off-range counts and `fit_range` agree with the highlighted frets | B10, B12 |
| SAC-11 | 020 | Any sequence of up to 50 random actions leaves a valid selection; each `Next` action applied list-length times returns to the start | B9 |
| SAC-12 | 022 | The chord field shows the E-FB-1 message with a "use suggestion" button and leaves the neck unchanged; "No playable voicing" shows both relax buttons | D10 |
| SAC-13 | 024 | Strum offsets are 0, 30, 60 ms ... low to high; arpeggio and scale notes are 428 or 429 ms apart at 70 bpm; mute silences `play_selection` and `play_note` | B11, D5 |
| SAC-14 | 025 | Explore renders without horizontal page scroll at 1280×800, 1180×820 landscape, 820×1180 portrait and 680×800; the neck scrolls inside its pane; the sidebar is hidden at 680 px | D13, D15 |
| SAC-15 | 026 | Tuner rules: `NoStablePitch` after 500 ms with no pitch, `TooQuiet` after 3 s low level, `TooLoud` on clipping, `far_from_any` beyond 3 semitones, lock overrides auto-select, ticks persist per session | C3 |
| SAC-16 | 027 | Each tuner state of FR-027 renders from a `TunerReading` (Vitest and Playwright); opening the tuner stops playback; hiding the page (iPad background) calls `tuner_stop` | D5, D12, D15 |
| SAC-17 | 028 | A settings change (theme, handedness, frets, labels default) takes effect without restart | D3, D13 |
| SAC-18 | 003, 023 | Out-of-order command responses never overwrite a newer snapshot (snapshots carry a `revision`) | D4, D14 |
| SAC-19 | 006 | The macOS job fails unless `lipo -archs` shows both `arm64` and `x86_64`; no workflow step references an Apple secret | E5 |

Quality gates (every phase): `make check` green; new Rust code has tests naming the requirement ID in the test name; no `#[allow]` without a reason; READMEs and docs updated in the same change; `status.md` updated after every task.

## Risks and Mitigation

| Risk | Impact | Mitigation |
|------|--------|-----------|
| No macOS machine in the dev environment; macOS and iOS jobs are proven only in CI | Late discovery of Tauri, signing or Xcode problems | Keep macOS-specific code minimal; open the draft PR early so CI runs; Phase E tasks E5 and E6 start as soon as D1 builds; fallbacks documented in research.md (Q-3) |
| Tauri iOS build without signing may not produce an `.app` via `tauri ios build` alone | AC-P0-4 job fails | Fallback chain in task E6: `tauri ios build` → `xcodebuild` with `CODE_SIGNING_ALLOWED=NO` on the generated project → `cargo build --target aarch64-apple-ios-sim` compile check; record in an ADR note |
| iOS microphone without the audio-session plugin | Tuner silent in Simulator | Tuner DSP tested without devices; manual check; plugin is first item next run (Q-4) |
| Synthetic tuner fixtures optimistic vs real guitars | False confidence in ±3 cents | Harmonic-rich decaying tones with noise and slight inharmonicity; recorded WAV follow-up AC (Q-1) |
| Coverage ≥ 80% with Tauri and cpal glue | `make cover` fails | Logic in pure functions; exclude only thin device/Tauri-runtime files by regex with a comment (D-16) |
| `tauri::generate_context!` needs `ui/dist` at compile time | Linux jobs fail | `ui/dist/index.html` stub rule in the Makefile (D-15) |
| Voicing search speed budget (5 ms) | Benchmark fails | Prune per string; bitmask pitch-class checks; cache by (chord, instrument, options) in the app state; curated first |
| Wrong note or spelling is a top-severity bug (INTENT) | Trust | Hand-written expected tables, properties on every tuning, pitch-class check on every curated shape in tests |
| `unsafe_code = forbid` vs cpal and Tauri | None expected | cpal and Tauri expose safe APIs; the workspace lint is not relaxed in this run |
| Playwright WebKit on Linux flakiness | CI noise | `--with-deps`, retries 1, traces on failure, no network |

## Timeline and Milestones

Sized in tasks, not calendar days; one developer with agents. Phases are in [plan.md](plan.md).

| Milestone | Content | Exit |
|-----------|---------|------|
| M-A | Phase A: workspace scaffold and `mackey-theory` | AC-P0-1..3, `make check` green |
| M-B | Phase B: `mackey-voicing` | AC-FB-*, AC-SC-*, AC-LH-*, AC-E-FB-4, benchmarks within budget |
| M-C | Phase C: `mackey-audio` | AC-TU-*, AC-NF-9, no-alloc test |
| M-D | Phase D: app and UI | AC-P0-5, AC-FB-9.1b/c, AC-FB-1.6, AC-LH-1.1, AC-UX-7, AC-FR-028, AC-NF-24, AC-E-FB-8 |
| M-E | Phase E: Makefile, AGENTS.md and CI | AC-P0-4: every CI job green on the PR |

Phase E task E3 (Linux jobs) can start as soon as the first workspace member builds; E5 and E6 as soon as D1 compiles.

## Open Questions

| ID | Question | Owner | Resolution path |
|----|----------|-------|-----------------|
| Q-1 | Are synthetic tuner fixtures enough for the P1 exit, or must recorded guitar WAVs pass first? | J | Default: synthetic is enough. Recorded WAVs in `crates/mackey-audio/tests/fixtures/` become a follow-up AC before device validation. |
| Q-2 | Is one synthesised voice acceptable until P1b (FB-8.2 deferred)? | J | Default: yes; `docs/prd/09-traceability.md` open items already note it. |
| Q-3 | Which unsigned iOS Simulator build command works on the GitHub macOS runner? | Build (task E6) | Try the fallback chain in research.md; record the working command in an ADR note. |
| Q-4 | Swift `audio-session` plugin in this run or next? | Build | Default: next run. Decide at the end of Phase E; do it this run only if iOS CI is green before Phase D ends. |
| Q-5 | How is microphone permission state detected without Objective-C bindings? | Build (task C6) | Default: cpal stream-build failure or a second of exact-zero input maps to `Denied`; first-open explainer persisted in settings; revisit with the Swift plugin. |
| Q-6 | Do Tauri 2 transitive dependencies need a licence outside `deny.toml`'s allow list (MPL-2.0, BSL-1.0)? | Build (task A1/D1) | Run `make deny` as soon as `tauri` is added; extend `deny.toml` for permissive licences only and record in the task notes; ask J if anything non-permissive is required. |
| Q-7 | Capo and chord naming: is the selected chord the sounding or the shape name? | Build, J to confirm | Default D-04: sounding name selected; shape name in the label; toggle swaps emphasis. |
| Q-8 | Should E-root pentatonic boxes use open strings (fret 0) or fret 12 for box 1? | Build | Default D-21: lowest root fret on string 6 at or above the capo, so open position for E; revisit after J tries it. |
| Q-9 | E-FB-2 says `Cmaj13#11` must be accepted, but E-FB-1 uses `C#maj13#11b9` as an unknown symbol, FB-4.3's grammar has neither `maj13` nor alterations, and FR-010 copies FB-4.3. Which wins? | J | Default: FB-4.3's grammar only (21 qualities); anything else returns the E-FB-1 error without a suggestion. Add `maj13` and alteration suffixes (`b5 #5 b9 #9 #11 b13`) in a later run. Log as an open item in `docs/prd/09-traceability.md`. |
| Q-10 | G-6.1 (undo for settings) and the PRD working file's silence on it: is deferring undo to P2 acceptable? | J | Default: yes (non-goal above). |
| Q-11 | FR-006 says all new CI jobs are required checks, but branch protection is repository configuration (`/repo:add-branch-restricts`), outside this run. | J | Job names are stable (`rust (macos-latest)`, `rust (ubuntu-latest)`, `ui`, `e2e`, `bindings`, `mac-app`, `ios-sim`); J marks them required after the first green run. The run only guarantees they exist and pass. |

## References

- Source PRD: [`specs/archive/261115-fretboard-foundation/fretboard-foundation-PRD.md`](fretboard-foundation-PRD.md)
- [INTENT.md](../../../INTENT.md), [AGENTS.md](../../../AGENTS.md), [docs/architecture.md](../../../docs/architecture.md), [docs/roadmap.md](../../../docs/roadmap.md)
- Product PRD: [README](../../../docs/prd/README.md) (G-1..G-6), [01-fretboard](../../../docs/prd/01-fretboard.md), [02-tuner](../../../docs/prd/02-tuner.md), [07-ux-flows](../../../docs/prd/07-ux-flows.md), [08-nfr](../../../docs/prd/08-nfr.md), [09-traceability](../../../docs/prd/09-traceability.md)
- [ADR 0006](../../../docs/adr/0006-on-device-audio-analysis.md), [ADR 0007](../../../docs/adr/0007-web-ui-and-ipc-boundary.md), [ADR 0008](../../../docs/adr/0008-ipad-via-tauri-ios.md)
- [docs/learning-features.md §1 to §3](../../../docs/learning-features.md), [docs/platforms.md](../../../docs/platforms.md)
