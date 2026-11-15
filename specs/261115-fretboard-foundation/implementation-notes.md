# Implementation Notes: Fretboard Foundation

**Feature:** fretboard-foundation
**Created:** 2026-11-15

**Purpose.** The running record of decisions, edge cases, deviations and lessons for this spec. Implementers update it as they work: add a numbered entry under Technical Decisions when a choice is made, add edge cases when found, and write a Deviation entry (with the reason) whenever the code departs from `spec.md`, `architecture.md`, `data-dictionary.md` or `tasks.md`. Update the affected spec file in the same change so the files agree. Decisions D-01 to D-33 were made by the spec builder while the user was away (non-interactive run); where one rests on a judgement call it names the open question that lets J overrule it.

## Technical Decisions

| ID | Decision | Why | Revisit when |
|----|----------|-----|--------------|
| D-01 | **Selection, instrument and handedness live in Rust** (`src-tauri` `Explorer` behind a mutex); the UI sends actions and receives an `ExplorerSnapshot`. | G-3.2 (every feature sets the selection through the same core command), AGENTS.md ("anything a test should check is computed in Rust"), and it makes cycling, announcements and notices testable in Rust. | P2/P3 add run and song mode: they reuse `apply`, so the explorer state may become one of several selections (multi-window, UX-4.1). |
| D-02 | Public API string numbers are `1..=6` with 1 = high E (glossary); internal arrays are low to high (index 0 = string 6), `StringNumber::index() = 6 - n`. | Matches the PRD vocabulary and the architecture's "low to high" `Tuning`. A single conversion point avoids off-by-one bugs. | never |
| D-03 | Frets are **absolute from the nut**; an unfretted string is `StringState::Open` and sounds at the capo fret; frets below the capo are unavailable and never highlighted. | Glossary ("fret numbers shown are absolute") and E-FB-4/LH-3. `Open` instead of `Fretted(capo)` keeps the "open string" ring and the open-string bonus meaningful. | never |
| D-04 | **The selected chord is the sounding chord.** Voicings are searched for the sounding chord with the capo applied; the label shows the shape name and the sounding name (`CapoLabel`), emphasis set by `CapoNames`. For the curated library, shape chord = sounding chord minus capo minus the tuning's uniform shift (Half-step down, Whole-step down). AC-LH-5.1 ("Capo 2, chord shape G → G shape (sounds A)") is tested as: sounding chord A, capo 2, open G shape → "G shape (sounds A)". | Keys and songs stay absolute; the PRD's default is shape names with sounding in brackets (LH-5). | J answers Q-7 differently. |
| D-05 | **`ts-rs` for types plus a generated `commands.ts`** built from an in-repo `CommandSpec` manifest; a unit test keeps the manifest, the x-macro command list and the capability permissions in step. | ts-rs generates types only; tauri-specta 2 is pre-release (PRD FR-003); AGENTS.md demands generated command wrappers. | tauri-specta 2 is stable: replace the generator, keep the types. |
| D-06 | **MPM** (McLeod) pitch detection, own implementation, direct NSDF, 2048-sample window, 512 hop, clarity threshold 0.80, k = 0.93. | Built-in clarity maps to confidence; no new dependency; see research.md Q1. | Profiling shows more than 2 ms per window (then FFT) or recorded guitars fail ±3 cents. |
| D-07 | Karplus-Strong synth, 16 preallocated voices, fractional delay, mono, `f32`, soft limiter. | Cheap, licence-free, tuned (also used for reference tones). SoundFont is P1b. | P1b sound-asset work. |
| D-08 | `mackey-audio` depends on `mackey-theory` only; the **app** converts instrument → `TunerTarget`s and `PlaybackPlan` → `SynthCommand`s. | The AGENTS.md table does not allow audio → voicing, and `mackey-rhythm` arrives in P1b. | P1b adds the rhythm dependency. |
| D-09 | Settings: one `settings.json` with a `version` field, `#[serde(default)]` groups, unknown fields ignored, atomic write (temp + rename), corrupt file renamed `.bak-<ts>` and defaults used with a one-time notice, values clamped on load. | NF-23 (never lose input), NF-15 spirit, forward compatibility. | `mackey-store` takes over settings in P2 (PRD open question: resolved yes). |
| D-10 | Curated shapes are `const` tables in Rust (not data files). | Pure crate, no I/O, compile-time checked, trivial to test; the library is small (about 30 shapes plus 5 movable CAGED templates). | The library grows past a few hundred entries. |
| D-11 | Search scoring weights live in one `const` table (span −3/fret, fingers −2, open string within frets 0..=4 +2, root in bass +4, skipped inner string −6, mutes beyond two −2, lowest fret above 7 −1/fret); result cap 24; ordering inside groups by score then lowest fret. AC-FB-5.1/5.2 calibrate it. | Testable, adjustable, documented. | A teacher or J disagrees with the first-listed voicing. |
| D-12 | Triad view for a seventh chord uses the root triad and shows "Triad of Cmaj7: C E G" (FB-6.3); `Inversion::Third` exists but only for the all-tones/voicing code paths, and the triad view clamps to three inversions. | FB-6.1 allows root/1st/2nd only. | A later phase adds four-note drop-2 voicings. |
| D-13 | Tuner time is counted in samples (`ms = samples * 1000 / sr`), never from a wall clock. | Deterministic tests; no clock in a pure module. | never |
| D-14 | UI mocks replay **recorded scenarios generated from the real Rust core** (`ui/src/mocks/scenarios.json`), not hand-written snapshots or TS theory. | Keeps ADR 0007 intact ("never re-implement theory in TypeScript") and the mock honest; stale mock data is caught by `make bindings-check`. | Scenarios grow beyond a maintainable size. |
| D-15 | `make build/test/lint/cover` create `ui/dist/index.html` (stub) if missing. | `tauri::generate_context!` fails without `frontendDist`; avoids building the UI for Rust-only jobs. | The Tauri crate is split or excluded from the default workspace members. |
| D-16 | Coverage exclusions by regex: `src-tauri/src/main.rs`, `src-tauri/build.rs`, `src-tauri/src/bindings.rs`, `crates/mackey-audio/src/io/`, each with a comment (device and runtime glue). Everything else counts. | PRD risk row ("exclude only thin device and Tauri-runtime glue"). | Glue grows logic: move logic into pure code instead of widening the list. |
| D-17 | `catch_unwind(AssertUnwindSafe(..))` guard around every command body; `Mutex::lock` poisoning recovers with `into_inner` and logs. Release profile keeps `panic = "unwind"` (the default). | NF-24. `panic = "abort"` would defeat the guard. | never |
| D-18 | `roman_to_chord` returns `Result<Chord, RomanParseError>` (docs/architecture.md shows `Option<Chord>`); update the architecture doc in task E8. | The UI needs to say why a numeral failed (NF-22). | never |
| D-19 | `Key` stores `tonic: Note` and `mode: Mode`; the F#/Gb and D#/Eb choice is the tonic's spelling; offered tonics are validated at construction; `Key::new_normalised` applies G-1.4 and returns a notice. | Spelling is determined by the key; invalid keys unrepresentable. | never |
| D-20 | `playback_plan` is a pure function in `mackey-voicing` returning `PlannedNote`s with millisecond offsets; the app converts to samples at the output rate. | Testable without audio; keeps `mackey-audio` independent of voicing (D-08). | P1b transport timing needs sample-exact scheduling (then convert in `mackey-rhythm`). |
| D-21 | **Scale pattern algorithm.** Pentatonic box *k* starts at scale degree *k* on string 6 at the lowest fret at or above the capo whose note is that degree (box 1 = the root; open position allowed, so E minor box 1 uses open strings). Notes are then assigned in ascending scale order, two per string (pentatonic) or three per string (7-note), frets derived by pitch arithmetic per string so every tuning works. Boxes are ordered by lowest fret; `P` cycles in that order. 3-notes-per-string patterns use the same rule with degree *k* = 1..=7. AC-SC-3.1 verifies it (A minor: s6 5,8; s5 5,7; s4 5,7; s3 5,7; s2 5,8; s1 5,8). | Generic over tunings (LH-4) with no hand tables; matches the standard boxes. | J prefers fret 12 for E boxes (Q-8). |
| D-22 | Voicing list order: groups Open, Barre and CAGED, Other; inside a group by score. Voicing cycling wraps within the group, then continues into the next group, then wraps to the first (FB-9 table). | FB-5.2, FB-9. | never |
| D-23 | Triad placements are enumerated in fret order; the default is the lowest; `P` moves to the next placement and the counter reads "k of n". An inversion with no playable placement stays selectable and shows `TriadResult::Unavailable` with the notice "Not playable on these strings within 4 frets" (FB-6.4) instead of being skipped. | AC-FB-6.1 expects the lowest placement (C: E s4 f2, G s3 f0, C s2 f1). Showing "unavailable" matches the PRD wording. | User testing finds skipping better. |
| D-24 | Arpeggio positions: start fret *s* with the window `s-1..=s+4` (four-fret window plus one fret each side, AC-FB-7.1: position 5 → frets 4..=9); valid positions are those whose window contains every required chord tone at least once; default per FB-7.4. | AC-FB-7.1 | never |
| D-25 | The explorer area, tuner and settings are separate **areas** in `UiState`; selection state is in Rust but the **active area** is UI view state. | ADR 0007 (UI holds view state). | Home arrives (P4). |
| D-26 | `R` (relative major/minor) toggles only Major ↔ Natural minor; for other modes it is disabled and the live region says "No relative key for E Dorian". | FB-2.4 says "relative major/minor toggle"; the PRD defines nothing for modes. | J wants Dorian ↔ parent major. |
| D-27 | Input devices are identified by the cpal device name string; the saved `input_device` that is no longer present falls back to the default silently with the E-TU-5 notice. | cpal gives no stable ids; acceptable for personal use. | Duplicate device names appear. |
| D-28 | Single-key shortcuts ignore events whose target is `input`, `textarea`, `select`, `[contenteditable]`, and `Space` ignores a focused `button`/`a`/`[role=button]` (so Space activates the button). | UX-6 and normal accessibility; otherwise Space would both press a focused button and trigger Play. | never |
| D-29 | `ExplorerSnapshot` includes `selection` so the UI controls are driven from Rust values, and the UI never infers "which voicing is selected" itself. | G-3.2, ADR 0007. | never |
| D-30 | The Tauri `Info.plist` and `bundle.iOS.infoPlist` carry `NSMicrophoneUsageDescription`; the microphone is requested only when `tuner_start` first opens the input stream. | NF-7.2. | The Swift plugin takes over permission handling. |
| D-31 | Every `ExplorerSnapshot` has a `revision` (u32, incremented on every state change); the UI serialises commands through one promise queue and drops any snapshot older than the one it holds. | Two quick key presses can resolve out of order across IPC; without it the neck could show a stale selection (found in the Step 2 review). | never |
| D-32 | Selecting a chord from focus None (or via the chord field) sets the view type to All tones (so AC-FB-4.1 shows D, F#, A, C); selecting a chord while a chord is focused keeps the current view type. | AC-FB-4.1 says "All tones shows ..." after typing `D7` with no prior view choice; keeping the view on chord-to-chord moves is what cycling expects (F-2). | J finds the reset surprising. |
| D-33 | Extended symbols outside FB-4.3 (`Cmaj13#11`, `C7b9`) are errors this run (Q-9); the error carries no suggestion. | FR-010 copies the FB-4.3 grammar; E-FB-2 and E-FB-1 conflict on `maj13#11`. | J answers Q-9. |
| D-34 | **Licences for Tauri.** `make deny` needed MPL-2.0 for five crates that arrive only via Tauri (`cssparser`, `cssparser-macros`, `dtoa-short`, `option-ext`, `selectors`). Allowed by per-crate `[[licenses.exceptions]]`, not globally. One advisory ignored with a reason: RUSTSEC-2024-0370 (`proc-macro-error`, unmaintained, Linux-only via GTK3 `glib-macros`). | Unavoidable with Tauri 2; MPL-2.0 is file-level weak copyleft and unmodified use from crates.io satisfies it. Linux is not a shipping target. | **J to confirm** the MPL-2.0 exceptions (Q-6). Revisit if the licence policy forbids weak copyleft, or when glib drops `proc-macro-error`. |
| D-35 | UI toolchain pins (exact): React 18.3.1, Vite 7.3.8, Vitest 3.2.7, TypeScript 5.9.3, ESLint 9.39.5, Playwright 1.65.1, Tauri CLI and API 2.12.4, `tauri` crate 2.x (lockfile 2.12.5). Newer majors exist (ESLint 10, TypeScript 7, Vite 8) but `eslint-plugin-jsx-a11y` supports ESLint up to 9 and typescript-eslint supports TypeScript below 6.1. | Boring, mutually compatible set; the task asked for React 18. | jsx-a11y supports ESLint 10, then upgrade together. |
| D-36 | `ts-rs` 12 exports types by its generated `export_bindings_*` tests; the target folder comes from `TS_RS_EXPORT_DIR` in `.cargo/config.toml` (`[env]`, relative to the repo root), so `make bindings` is `cargo test -p mackey-app export_bindings` and any `make test` also rewrites identical files. `make bindings-check` fails when `git status --porcelain ui/src/bindings` is non-empty after regenerating, so it also catches an untracked new binding. | No extra generator binary yet; the manifest-based `commands.ts` generator (D-05) is D6 proper. | D6 adds the command-wrapper generator. |
| D-37 | The UI command layer in this skeleton is hand-written (`ui/src/ipc/`: `Commands` interface, `tauri.ts`, `mock.ts`). A Rust test (`ui_command_layer_names_every_command`) checks that `tauri.ts` names every entry of `COMMAND_NAMES`. The mock is chosen by `vite --mode mock` or by the absence of `window.__TAURI_INTERNALS__`. | Walking-skeleton scope; D6 replaces `tauri.ts` with a generated file. | D6. |
| D-38 | Tauri CLI cwd: `make app` and `make ios-sim` call `ui/node_modules/.bin/tauri` from the repo root (the CLI finds `src-tauri/` below the current folder; `npm --prefix ui run tauri` starts it in `ui/` and fails). The CLI then runs `beforeBuildCommand`/`beforeDevCommand` with cwd `ui/`, so they are `npm run build` and `npm run dev` (verified by a Linux `make app`). | Observed behaviour of CLI 2.12.4. | A newer CLI changes the hook cwd. |
| D-39 | `src-tauri/gen/` is git-ignored entirely; `tauri ios init --ci` regenerates the Xcode project in `make ios-sim` and CI. Icons: placeholder set generated with `tauri icon`; Windows, Android and iOS icon folders removed (deferred platforms; iOS init generates its own). `Info.plist` (macOS) and `Info.ios.plist` (iOS) both hold `NSMicrophoneUsageDescription`; Tauri merges both automatically by name, so no `bundle.iOS.infoPlist` key is needed (amends D-30). | Generated files are never hand-edited; doccheck does not want READMEs in generated folders. | The Swift audio-session plugin needs a committed Xcode project. |
| D-40 | `src-tauri/src/runtime.rs` (the `tauri::Builder` call) is the coverage-excluded glue instead of a `bindings.rs`; exclusion regex is `COVER_IGNORE` in the Makefile: `main.rs`, `runtime.rs`, `build.rs` of `src-tauri`. `make cover` passes locally (93% lines). | Amends D-16. | Glue grows logic. |
| D-41 | Playwright WebKit cannot be downloaded in the sandbox that built this (proxy blocks the CDN), so the specs were verified locally in Chrome via `PW_CHANNEL=chrome make e2e`; CI uses WebKit only. | Environment limit; the escape hatch is documented in `ui/playwright.config.ts`. | WebKit-specific failures in CI: fix there. |

## Edge Cases and Solutions

Seeded from the PRD (E-FB-*, E-TU-*) and from design analysis. Implementers add what they find.

| Case | Required behaviour | Where handled |
|------|--------------------|---------------|
| Unknown chord symbol (`Hm7`) | Selection unchanged; inline error with suggestion `Bm7` | A8, D4, D10 |
| Empty chord field | `ChordFieldStatus::Empty`; selection unchanged | D4 |
| Chord symbol with leading/trailing whitespace | Trimmed; internal whitespace is an error | A8 |
| Very long or non-UTF-8-boundary input | No panic; error with a span on a `char` boundary | A8 (proptest) |
| Double accidental from theory spelling (`Fx`) | Label shows the simple enharmonic; tooltip carries the theory name | A7, B10 |
| Key with 6 accidentals (F#/Gb) | Spelled per the chosen tonic; tooltip offers the enharmonic key | A5, D10 |
| Mode whose parent key is not offered (D# Dorian) | Becomes Eb Dorian with a notice | A5 |
| Capo above the selected position | Move to the first position at or above the capo; "Moved above the capo" | B9 |
| Capo 12 with frets 19 | 7 usable frets; voicing search still honours span rules; "No playable voicing" if none | B5, B10 |
| Capo higher than any valid arpeggio start | Arpeggio view shows the no-position notice, no highlight | B7, B10 |
| Tuning change invalidates the selected voicing | Best voicing selected with "Voicing changed for new tuning" | B9 |
| Custom tuning with two identical open strings | Allowed; fingering counts them once (E-FB-6) | B4, B12 |
| Custom tuning outside −3/+2 semitones | `UnusualTension` warning shown; still allowed | B1, D10 |
| Custom tuning producing pitches out of MIDI range | Rejected at construction with an error (UI keeps the old tuning) | B1 |
| No playable voicing | "No playable voicing in this tuning" with both relax options; never a rule-breaking voicing | B5, D10 |
| Triad unplayable within 4 frets | Shown as unavailable, not as a wrong shape | B6 |
| Voicing list empty while view type is Voicing | `FretboardView.positions` empty, notice, selection unchanged | B9, B10 |
| Visible range smaller than the selection | Edge arrows with counts; "Fit selection" | B10, D9 |
| Visible range inverted (`from > to`) | `FretRange::new` error; UI swaps handles | B10, D9 |
| Left-handed + flip strings together | Independent transforms; tooltips and aria labels unchanged | D8 |
| Narrow window / Split View | Sidebar hidden, neck scrolls, minimum fret widths kept | D13 |
| No audio output device | Visual features work; Play returns `NoAudioOutput`; UI shows the notice once | C5, D5 |
| Audio output disappears mid-session | Stream error flag → next `play_*` returns `NoAudioOutput` | C5, D5 |
| Audio command queue full | Command dropped, counter incremented, never blocks | C5 |
| Play pressed repeatedly | New plan replaces the old (`StopAll` then notes) | D5 |
| Play while focus is on a button | Space activates the button, not Play | D11 |
| Mic permission never asked / denied | Explainer first; denied state with Settings button; reference tones still work | D12 |
| Mic stream delivers digital silence | Treated as denied after 1 s | C6 |
| Input device unplugged while tuning | Fall back to default, notice | C6, D12 |
| Two strings ringing | Strongest pitch if confident, else "Play a single string" | C3 |
| Pitch outside 60 to 1,400 Hz | No reading (`NoStablePitch`) | C2 |
| NaN/inf/empty samples into the detector | `None`, no panic | C2 |
| Sample rate not 44.1/48 kHz | Resampling not provided: `DetectorConfig::new` accepts ≥ 22,050 Hz and uses the device rate directly | C2, C6 |
| Settings file missing / corrupt / unwritable | Defaults; `.bak` for corrupt; write failure returns `AppError` with Details, in-memory settings still applied | D3 |
| Two rapid `set_settings` calls | Serialised by the settings mutex; last write wins; atomic rename prevents a torn file | D3 |
| Panic in a command | `AppError` "Something went wrong" + Details; app keeps running | D2 |
| Command invoked before state is ready | State is created in `setup` before the window loads; commands never see `None` | D1 |
| Window resized under the minimum | Tauri enforces 900×600 on desktop; iPad uses CSS only | D1, D13 |
| Theme changes while open | `data-theme` / `prefers-color-scheme` update; no reload | D7, D13 |
| Reduce Motion | No transitions | D8 |
| Two cycling keys pressed faster than IPC round trips | Commands queued in order; stale snapshots dropped by `revision` | D4, D11, D14 |
| iPad app goes to background or the page is hidden while the tuner is open | `tuner_stop` on `visibilitychange`, restart when visible | D12 |
| Window closed while the tuner runs | `tuner_stop` on exit request; analysis thread joined | D5 |
| Analysis thread panics | Caught; one `AnalysisStopped` notice; app keeps running | C6, D12 |
| `tuner_start` twice / `tuner_stop` without start | Restart / no-op | C6, D5 |
| Fret count lowered in settings (24 to 19) while a position beyond it is selected | Selection indexes clamped; neck redrawn; no panic | B9, D4 |
| Command argument out of range (string 0 or 7, fret above count, capo 13) | `AppError { kind: Invalid }`, state unchanged | D4 |
| Extended chord symbol (`Cmaj13#11`) | E-FB-1 style error without suggestion this run (Q-9) | A8 |
| `make doccheck` after `make e2e` creates report folders | Skipped by xtask (E9) | E9 |
| `make test` on a machine without WebKitGTK/ALSA headers | Fails to build `mackey-app`; README lists the packages; CI installs them | D1, E3 |

## Deviations from Plan

None yet. Record each as: date, what changed, why, which spec files were updated.

## Spec review (Step 2)

**Reviewer:** the same agent that wrote the spec, applying `/dev-flow:review-spec` honestly, on 2026-11-15, by reading every file in this directory against the PRD, AGENTS.md and the repository's tooling (`xtask`, `Makefile`, `ci.yml`). Where the first draft failed a dimension, the gap was fixed in the spec files in the same step and is listed below.

**Verdict before fixes: Needs revision** (no Fail dimensions, five Warnings). **Verdict after fixes: Implementation-ready**, with the residual risks listed at the end (none is a spec gap; each has an owner and a fallback).

| Dimension | First draft | After fixes | Note |
|-----------|-------------|-------------|------|
| Acceptance criteria | Warning | Pass | 38 PRD ACs are numbered and testable, but roughly half of the 28 FRs had no AC of their own (for example FR-002, 004, 005, 009, 011, 013, 014, 022, 025, 028, and the state rules of FR-026 and FR-027). Added 19 derived criteria SAC-01..19 (spec.md) with an owning task each; traceability in plan.md. |
| Technical approach | Warning | Pass | Strategy was coherent (pure core, Rust-owned selection, ts-rs) but three choices were deferred or unsound: (1) ts-rs emits no command wrappers, so "generated wrappers" had no mechanism; (2) out-of-order IPC responses could show a stale neck; (3) E-FB-2 vs E-FB-1 conflict on extended symbols. Fixed: manifest + x-macro generator (architecture, D-05); `revision` plus a command queue (D-31, SAC-18); grammar decision D-33 and Q-9. |
| Component coverage | Warning | Pass | Paths were named per task, but the review of `xtask/src/doccheck.rs` showed `ui/e2e`, `src-tauri/capabilities` and `src-tauri/icons` are not skipped and would fail `make doccheck` without READMEs, and `playwright-report`, `test-results`, `coverage` (git-ignored, present after test runs) would fail it locally. Also missing: `ui/.nvmrc`/`engines`. Added READMEs to the file lists, task E9 (xtask skip list with a test), `.nvmrc`. Corrected the draft's claim that every `ui/src` sub-folder needs a README (doccheck skips `src`). |
| Edge case handling | Warning | Pass | The table covered the PRD's E-* cases but not: concurrent or out-of-order commands, repeated `tuner_start`/`stop`, window close and iPad backgrounding while the tuner runs, analysis-thread panic, fret-count reduction leaving stale indexes, out-of-range command arguments, settings write races, doccheck after e2e. All added to the edge-case table with owning tasks. External API failures: none exist (no network, NF-7), now stated in the spec. Empty/null inputs and permission boundaries (mic, capabilities) were already covered. |
| Open questions | Pass | Pass | Eight flagged with owners and defaults; the review added Q-9 (extended symbols), Q-10 (undo), Q-11 (required checks). |
| Out-of-scope clarity | Warning | Pass | Non-goals were explicit but silent on G-6.1 undo for settings, UX-5.3 shortcut overlay, UX-5.1 pinch zoom, E-FB-2, and network failure handling. Added. |
| Status initialization | Pass | Pass | `status.md` created with Phase 0 In Progress, then advanced after this review; the dashboard Step 1 and Step 2 rows carry real timestamps. |

### Detailed findings and how each was fixed

1. **[Warning] Untested FRs.** See Acceptance criteria. Fix: SAC table in `spec.md`; tasks tagged with their SAC; `plan.md` traceability extended. *(Fixed.)*
2. **[Warning] Command wrappers cannot come from ts-rs.** AGENTS.md says "call Rust only through the generated command wrappers", FR-003 chose ts-rs. Fix: `CommandSpec` manifest, x-macro command list shared by `generate_handler!`, `build.rs` and the generator, and a test that keeps them equal (architecture "Bindings generation", tasks D1, D6). *(Fixed.)*
3. **[Warning] Out-of-order snapshots.** `ExplorerSnapshot.revision`, a single promise queue, tests in D4/D14 (SAC-18). *(Fixed.)*
4. **[Warning] AC-TU-1.1 vs TU-1.1 timing arithmetic.** The AC wants "In tune" within 400 ms of input while TU-1.1 holds 300 ms and NF-9 allows 150 ms to the first reading; 150 + 300 would break the AC. Resolved by design: the first reading is emitted when the first 2,048 samples exist (about 43 ms), the hold timer starts at the first in-tune reading (about 343 ms total) and the median-of-3 uses available readings; the test covers 44.1/48 kHz and block sizes 256, 512 and 1,024 (architecture, task C3). *(Fixed.)*
5. **[Warning] E-FB-2 / E-FB-1 / FB-4.3 inconsistency in the product PRD** (`Cmaj13#11` accepted in one row, a similar symbol rejected in another, neither in the grammar). The spec follows FB-4.3 and logs the conflict (D-33, Q-9, task A8); `docs/prd/09-traceability.md` open items get a row in E8. *(Fixed in the spec; the product PRD itself is not changed by this run without J.)*
6. **[Warning] doccheck and local artefacts.** See Component coverage; task E9 and README entries. *(Fixed.)*
7. **[Warning] Lifecycle gaps for the tuner** (backgrounding, window close, double start, thread panic). Added to architecture, D5, D12, C6. *(Fixed.)*
8. **[Warning] Stale state after fret-count change.** Selection clamping added to B9 and D4. *(Fixed.)*
9. **[Minor] Chord-view default.** AC-FB-4.1 requires "All tones" after typing `D7`; the spec did not say what view a newly selected chord has. D-32. *(Fixed.)*
10. **[Minor] Task numbering.** The draft's progress summary was inconsistent; it now totals 61 tasks (A 11, B 16, C 9, D 16, E 9). *(Fixed.)*
11. **[Minor] `u64` in the TS boundary.** `t_ms` changed to `f64` (research Q3). *(Fixed.)*

### Positive observations

- Every FR traces to product requirement IDs, crates and phases; the PRD's own review items (FB-1.5, E-FB-5..7, E-TU-5..6, FB-8.2 deferral) are each owned by a task.
- Architecture honours the layer table exactly (audio depends on theory only) and puts conversion between voicing and audio in the app (D-08, D-20), avoiding a table change.
- Hand-written expected tables (252 chord rows, curated shapes, key signatures, tunings) are specified rather than generated, which is the right defence for the INTENT "wrong note is top severity" rule.
- Tasks are small enough to be implemented independently, each with files, dependencies, acceptance and tests; the critical path and the riskiest tasks are named; macOS and iOS CI work starts early on purpose.
- Performance budgets are enforced by a release-mode test as well as criterion, so `make props` fails on a regression instead of relying on someone reading benchmark output.

### Residual risks and caveats (not spec gaps)

- **iOS Simulator build without signing (Q-3, task E6)** cannot be proven from the Linux dev environment; three fallbacks are specified; the first passing one is recorded in an ADR note.
- **Licences brought in by Tauri (Q-6, task D1)** may require adding permissive licences to `deny.toml`; anything non-permissive goes to J.
- **Voicing score weights (D-11)** are a first calibration; AC-FB-5.1 and AC-FB-5.2 pin only two orderings. J's practice is the real test.
- **Exact crate and package versions** (`ts-rs`, `cpal`, Tauri, Playwright) were not verified online in this step; each task that adds one verifies the API and pins the version (research.md, VERIFY items).
- **Q-1, Q-2, Q-4, Q-7, Q-9, Q-10, Q-11 need J's confirmation**; defaults are chosen so work is never blocked.

Next step: Step 3 of the orchestration, `/dev-flow:implm-from-spec specs/261115-fretboard-foundation/` (implementation by phase A to E in tasks.md).

## Manual checklist (AC-MAN-1, device-only checks)

To be completed by J (task D16). Leave unchecked until run.

- [ ] Mac (Apple silicon): the app launches; G major notes are on the neck; Space plays a strum; a played string is read by the tuner within ±3 cents; Cmd-T opens the tuner.
- [ ] Mac: first Tuner open shows the explainer, then the OS microphone prompt; deny, then see the "Microphone access is off" state.
- [ ] iPad Simulator: the app launches; the neck renders in landscape and in portrait.
- [ ] Cold start to interactive neck ≤ 2 s on the Mac (NF-3).
- [ ] VoiceOver on the Mac: arrow through the neck positions; labels read "String 3, fret 2, A, third of F"; cycling announces changes.
- [ ] Colour-blind simulation (any tool) of the Explore screen in light and dark.

## Lessons Learned

(To be filled during and after implementation.)
