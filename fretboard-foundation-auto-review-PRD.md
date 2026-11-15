# Fretboard Foundation: Auto Review Fixes PRD

**Created:** 2026-11-15
**Status:** Reviewed (see Review Log)
**Source:** Step 5 (Code and Design Review) of `implm-frm-prd` on branch `claude/project-thread-e8va25`, reviewing `git diff main...HEAD` against `specs/261115-fretboard-foundation/` and `AGENTS.md`.
**Reviewer method:** read of the core, shell and audio paths; executable probes of the theory/voicing engine (since removed); `cargo test` of `mackey-theory` and `mackey-voicing`; `xtask doccheck` and `archcheck`.

## Executive Summary

Overall quality is high. The workspace respects the layer table (`archcheck` and `doccheck` pass), library code has no production `unwrap`/`expect`/`panic!`, the Tauri capability set is `core:default` only, the CSP is tight, the cpal callbacks allocate, lock and log nothing, panics are guarded at the IPC boundary, and the UI does not re-implement theory (the only arithmetic in `ui/src` is geometry and needle drawing). Spelling is correct in every key probed (Gb major, Db major, Eb minor, F# minor, G# minor). Chord voicings were checked for pitch content across Standard, Drop D, DADGAD and Open G for 13 chords: every sounding note belongs to the chord and every bass note is correct.

Three real defects were found, two of which the orchestrator suspected:

1. The scale-pattern stepper shows the open-position pattern (box 4) as "1 of 5" and the docs caption calls it box 1. The core function is right and its test passes, but the shell numbers patterns by sort position, not by box number (FR-001).
2. The tuner floods the UI and spins the CPU when the input device is not paced by a clock. Cause confirmed in code: the readings throttle uses stream time, so an unpaced device passes every third reading at wall-clock speed, and the pump loop never yields while the ring is non-empty (FR-002).
3. Triad placements that use an open string together with a fret of 12 or higher are mislabelled: the open string is the lowest pitch, so the "root position" (or other) triad is actually another inversion. 90 of 716 probed placements were wrong (FR-003).

The rest are medium findings: tuner glue excluded from coverage, a CSP allowance, and doc drift.

Verdict: **not ready to merge until FR-001 to FR-003 are fixed**. Counts: P0 = 3, P1 = 0, P2 = 4. FR-003 is P0 because INTENT.md ranks a wrong note or name as top severity, and a triad labelled with the wrong inversion teaches a wrong voicing.

## Problem Statement

The foundation passes its automated gates, yet a user opening the Explore screen with a pentatonic overlay is shown the wrong box, a beginner using the triad view can be shown a wrong inversion, and a machine with a stuck or unpaced input freezes the app when the tuner opens. INTENT.md ranks correct notes, spelling and fingering as the top severity; the gates did not catch these because the tests exercise the core in isolation and the audio glue is excluded from coverage.

## Goals

- Show the pattern the user asked for, numbered as the PRD defines it (AC-SC-3.1).
- Never mislabel a triad inversion.
- Keep the UI responsive whatever the input device does, and tell the user when the device is unusable.
- Close the test and documentation gaps that let these through.

## Non-Goals

- New features, new tunings, new scales.
- Windows, Linux, iPhone and Android behaviour (Linux is used only to reproduce).
- Redesigning the tuner DSP.

## Functional Requirements

### FR-001 (P0): Scale-pattern stepper shows and numbers the wrong box

**Evidence.**
- `crates/mackey-voicing/src/patterns.rs:194` sorts the patterns by `(lowest_fret, highest_fret)`. For A minor pentatonic the order is therefore box 4, 5, 1, 2, 3 (box 4 sits at frets 0 to 3; see the hand table in `crates/mackey-voicing/tests/patterns.rs`).
- `crates/mackey-voicing/src/selection.rs:548-561` (`step_pattern`) starts at `pattern_index = Some(0)`, so the first `P` shows box 4.
- `crates/mackey-voicing/src/view.rs:914-920` and `:926-935` label it `"{i+1} of {n}"`, so box 4 reads "1 of 5". The label never uses `PatternKind::PentatonicBox { number }`.
- `user-docs/explorer.md:71` captions `docs/images/real-explore-a-minor-pentatonic.png` as "box 1 of 5", but the image draws frets 0 to 3.
- The only test (`tests/patterns.rs`) calls `scale_patterns` and finds box 1 by its `number`, so it never exercises the stepper or the label.

**Why it matters.** PRD SC-3 and AC-SC-3.1 say box 1 has the root on string 6 at the first finger (fret 5 for A). A learner who follows the app learns the wrong box numbers, which is a wrong-fingering defect.

**Acceptance criteria.**
- Given key A minor, scale overlay A minor pentatonic and no pattern selected, when the user presses `P` once, then the `FretboardView` contains exactly the box 1 notes of AC-SC-3.1 (frets 5 and 8 on strings 6, 2, 1; 5 and 7 on strings 5, 4, 3) and the list position label names box 1.
- Given box 1 is shown, when the user presses `P` again, then box 2 is shown, and so on through box 5 and back to "all".
- Given a seven-note mode, when stepping, then the 3-notes-per-string patterns are numbered by their pattern number, not by sort position.
- Given the label, when read by assistive technology, then it states the pattern number and the count ("Box 1 of 5") and the announcement matches the drawn notes.
- The screenshot `docs/images/real-explore-a-minor-pentatonic.png` is regenerated (`make screenshots`) and its caption is true.
- A regression test drives `step_pattern`/`apply` and `build_view` (not only `scale_patterns`) and fails on the current code.

### FR-002 (P0): Tuner floods the UI and spins the CPU on an unpaced input device

**Evidence.**
- `src-tauri/src/tuner.rs:6,22`: the throttle accepts a reading when `reading.t_ms - last >= 30`, where `t_ms` is stream time. A device that delivers samples faster than real time (the ALSA null input, a misbehaving virtual device) still yields about 33 emits per stream-second, which is thousands per wall-second.
- `src-tauri/src/state.rs:382-386` and `commands.rs:313-320` emit each accepted reading with `app.emit` straight from the analysis thread; there is no wall-clock bound and no back-pressure.
- `crates/mackey-audio/src/io/pump.rs:115-137` (`pump_input`) loops while the ring keeps filling to a full scratch buffer, and `crates/mackey-audio/src/io/input.rs:287-337` only sleeps when `n == 0`. With a producer faster than the consumer the thread never sleeps.
- `specs/261115-fretboard-foundation/implementation-notes.md` D-91 records 260% CPU and a frozen UI, and defers the fix to device validation.
- `input.rs` is excluded from `make cover` (`Makefile:46`), so nothing tests the loop.

**Why it matters.** A frozen UI on opening the tuner is a blocker, and the throttle is documented as "about 30 Hz to the UI" (spec TU), which it is not on such devices. Real hardware inputs (a USB device that bursts after a stall, Bluetooth reconnect) can produce the same burst.

**Acceptance criteria.**
- Given a fake input that supplies samples at 50x real time, when the pump and throttle run for one wall-clock second, then at most 35 readings are emitted and the analysis thread yields (measured with a fake clock or by sleep count in a test).
- Given the throttle, when readings arrive, then spacing is enforced in wall-clock time as well as stream time (use `Instant`, injected so tests are deterministic); notices and status changes still pass immediately.
- Given the pump, when the ring stays non-empty, then one iteration processes at most a bounded number of samples (for example about 100 ms of audio) before the worker checks `stop`, commands and sleeps.
- Given an input that delivers more than about 2x its nominal rate over a rolling second, or only exact zeros, when detected, then the tuner emits a notice ("This input is not delivering audio at a normal rate") and stops analysing instead of spinning; the screen shows that message, not "Too quiet".
- Given the ALSA null device on Linux, when the tuner is opened, then CPU stays below 25% of one core and the window stays responsive (manual check recorded in `implementation-notes.md`).
- The loop logic is moved out of the coverage-excluded `input.rs` into a testable function, or the exclusion is narrowed to the cpal calls only.

### FR-003 (P0): Triad placements with an open string at fret 12 or higher have the wrong bass

**Evidence.** `crates/mackey-voicing/src/triads.rs:70-125`. The tones are assigned to the set's low, middle and top strings, and every fret on each string that has the right pitch class is combined (`:98-113`), with only a span check on pressed frets. Open strings (including at the "capo" fret) are excluded from the span, and nothing checks that the three pitches ascend. A probe over 13 chords, 2 tunings, 4 string sets and 3 inversions found 90 of 716 placements whose lowest note was not the inversion's bass. Examples: C Standard S345 Root placement 1 `x(15)(14)0xx` (bass G, not C); Am Standard S123 Root placement 1 `xxx(14)(13)0` (bass E, not A); F#m S456 Root placement 0 `(14)0(10)xxx` (bass A, not F#); G7 Standard S123 Root placement 2 `xxx(12)0(10)` (bass B, not G).

**Why it matters.** The triad ladder is for learning inversions; a placement labelled "Root" that is really a first inversion teaches the wrong voicing. Pitches are right; the label and bass claim are wrong. Existing tests only check the first placement of each combination.

**Acceptance criteria.**
- Given any chord, tuning, capo, string set and inversion, when `triads` returns placements, then for every placement the lowest sounding pitch is the inversion's bass tone and the three pitches ascend from the low string to the top string (a property test with `proptest` over chord qualities, 12 roots, the 9 tunings and capo 0 to 12).
- Given the C Standard S345 root case, when placements are listed, then `x(15)(14)0xx` is absent.
- Given a set with no valid placement, when requested, then `Unavailable` is returned rather than a wrong one.
- The regression test is written first and fails on the current code.

### FR-004 (P2): Pattern stepper has no test at the selection or view level, and the AC mapping points at a missing file

**Evidence.** `specs/261115-fretboard-foundation/spec.md:218` maps AC-SC-3.1 to `mackey-voicing/tests/scale_patterns.rs`, which does not exist (the test is `tests/patterns.rs`). Nothing covers `step_pattern`, the "n of m" label or the announcement, which is how FR-001 shipped.

**Acceptance criteria.**
- Given the traceability tables (`spec.md`, `docs/prd/09-traceability.md`), when they are read, then every test path they name exists (checked by a documented review, or a small `xtask doccheck` rule if cheap).
- Given the new FR-001 tests, when `make test` runs, then AC-SC-3.1 is covered end to end (core, selection, view, announcement) and the e2e mock agrees with the Rust label format.

### FR-005 (P2): Audio glue with real logic is excluded from coverage

**Evidence.** `Makefile:46` ignores `crates/mackey-audio/src/io/{devices,input,output}.rs`. `input.rs` holds the worker loop, device-loss recovery (E-TU-5), permission detection and config/lock commands; only a "starts or errors, never panics" test exists.

**Why it matters.** The exclusion hid FR-002. AGENTS.md allows excluding only thin glue that needs a sound card.

**Acceptance criteria.**
- Given the worker loop, when its decision logic (command handling, stream-failure fallback, silent-stream notice, yield policy) is moved behind a function that takes a fake ring and fake clock, then it is unit tested and `input.rs` is removed from `COVER_IGNORE` or reduced to the cpal build calls.
- Given `make cover` runs in CI, when it finishes, then overall line coverage is still at least 80% and AGENTS.md describes the narrower exclusion.

### FR-006 (P2): CSP allows inline styles

**Evidence.** `src-tauri/tauri.conf.json:26` has `style-src 'self' 'unsafe-inline'` in the production CSP. Scripts are `'self'` only, `object-src 'none'`, `connect-src` is limited to IPC, and capabilities are `core:default`, so the risk is low.

**Acceptance criteria.**
- Given the production build, when the UI is loaded, then the CSP has no `'unsafe-inline'` for styles, or a documented decision (ADR or note) records why React inline `style` attributes need it and what bounds the risk (scripts stay `'self'` only).
- Given any rendered screen, when it is loaded in the app, then there are no CSP violations in the console.

### FR-007 (P2): Documentation drift to correct alongside the fixes

**Evidence.** `user-docs/explorer.md:71` (caption, FR-001); `spec.md:218` (test path, FR-004); `implementation-notes.md` D-91 states the tuner issue is deferred, which becomes false once FR-002 lands; the screenshot set under `docs/images/real-*.png` must be regenerated after FR-001 and FR-002.

**Acceptance criteria.**
- Given FR-001 to FR-006 are merged, when `make doccheck` runs and the docs are read, then no document describes the old behaviour, D-91 is marked resolved with the fix, the Explore screenshot is regenerated, and the root README and `user-docs/` links still resolve.
- Given the spec and PRD traceability tables, when read, then each names the new regression tests.

## Non-Functional Requirements

- NFR-1 Correctness: no regression in the existing 38 PRD ACs and 19 derived SACs; `make check` stays green.
- NFR-2 Real-time safety: no allocation, lock, log or I/O is added to a cpal callback; the analysis thread never spins.
- NFR-3 Responsiveness: tuner events reach the UI at no more than about 30 per wall-clock second under any input.
- NFR-4 Performance: voicing search and triads stay within the budgets in `docs/prd/08-nfr.md` (run the criterion benches before and after FR-003).
- NFR-5 Accessibility: pattern and triad labels are announced and match what is drawn; every cue keeps its shape or label.
- NFR-6 Docs: every behaviour change updates the folder README, `user-docs/` and the spec in the same commit.

## Dependencies and Risks

- FR-001 touches `mackey-voicing` selection and view, the generated TypeScript bindings (`make bindings-check`) and the Playwright mock; keep the label format in one place.
- FR-002 and FR-005 both edit `crates/mackey-audio/src/io/` and `src-tauri/src/tuner.rs`; do them in one workstream.
- FR-003 may remove placements the e2e or snapshot tests expect; review snapshot diffs by hand (`cargo insta review`), never accept blindly.
- Risk: reproducing FR-002 needs a Linux machine with the ALSA null device; the macOS behaviour with a paced input is unaffected but the throttle change applies everywhere.
- Risk: renumbering patterns changes stored selections if a setting persists `pattern_index`; check `settings.rs` and migrate if so.

## Open Questions

1. (Decided in review: box-number order.) Should the stepper list patterns in box-number order (1 to 5) or in neck order starting at the lowest box? This PRD assumes box-number order starting at box 1, as AC-SC-3.1 reads.
2. (Decided in review: list only under its true inversion; do not offer a wrong label.) Is a triad with an open string and a fret of 12 or more ever wanted (it is playable)? This PRD assumes it should be listed only under the inversion it truly is.
3. (Decided in review: show the notice only; no automatic device fallback.) What should the tuner show for an unpaced device: a notice, or fall back to the next input device automatically?
4. (Decided in review: remove it if feasible; if React inline styles need it, keep it with a documented rationale.) Is `'unsafe-inline'` for styles acceptable if documented, or must the UI move to class-based styles only?

## How to Implement

- **TDD is required.** For every FR, write the failing regression test first (it must fail on the current branch for the reason described), then the minimal fix, then refactor. Table-driven tests for labels, `proptest` for the triad invariant, a fake clock and fake ring for the tuner.
- **Brief code review per fix.** After each FR, run `/review-code` on its diff (or a short manual review against `AGENTS.md`) and record the outcome in the spec's `implementation-notes.md` before moving on.
- **Agent teammates.** Use one agent per workstream: A (FR-001, FR-004: voicing selection/view, UI label, screenshot), B (FR-002, FR-005: audio loop, throttle, coverage), C (FR-003: triads property test and fix). A separate reviewer agent checks each result.
- **Git worktrees.** Create one worktree per workstream from the review branch (`git worktree add ../mackey-fr-a -b fix/261115-pattern-stepper`, and similar for B and C) so they proceed in parallel; merge them back one at a time, running `make check` after each merge. Workstreams A and B touch disjoint files; C is independent of both.
- Run `make check` (plus `make e2e` and `make bindings-check` for FR-001) before each commit, and `make screenshots` at the end for FR-001 and FR-007. Do not run `make cover` on small disks; CI does.
- Update `specs/261115-fretboard-foundation/fretboard-foundation-status.md` after each task, and mark D-91 resolved in `implementation-notes.md`.

## Review Log

`/review-prd` run on 2026-11-15 (dev-flow Step 6).

| Dimension | Status | Note |
|---|---|---|
| Problem clarity | Pass | Three user-visible defects named with evidence (file and line) |
| Scope definition | Pass | Goals and non-goals explicit |
| Functional completeness | Pass | FR-001 to FR-007 atomic, each linked to a goal |
| NFR coverage | Pass | Correctness, real-time safety, responsiveness, performance, accessibility, docs |
| Acceptance criteria | Pass after edit | Every FR has Given/When/Then criteria; FR-004 to FR-007 wording tightened |
| Dependency identification | Pass | Bindings, mock, shared audio files, snapshots, persisted settings named |
| Open questions | Pass | Four questions, each now answered with a default decision |

Changes made in review:
- FR-003 raised from P1 to P0 (orchestrator decision): INTENT.md makes a wrong note or name the top severity. Counts are now P0 = 3, P1 = 0, P2 = 4.
- P0 check: FR-001 (wrong box number and pattern), FR-002 (frozen UI) and FR-003 (wrong inversion label) are true blockers; FR-004 to FR-007 are correctly P2.
- Acceptance criteria for FR-004 to FR-007 reworded as Given/When/Then; FR-007 gained a traceability criterion; the FR-005 coverage criterion now names CI as the place `make cover` runs.
- Open questions recorded with defaults so implementation is not blocked.
- Confirmed the PRD carries TDD, per-fix code review, agent teammates and git worktree guidance (How to Implement).

Verdict: **Ready for spec.**
