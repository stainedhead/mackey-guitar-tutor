# Spec: Fretboard Foundation Auto Review Fixes

**Created:** 2026-11-15 · **Source PRD:** [fretboard-foundation-auto-review-PRD.md](fretboard-foundation-auto-review-PRD.md)

## Executive Summary

The fretboard foundation passes its gates but a code review found three blockers: the pattern stepper numbers boxes by sort position, the tuner floods the UI on an unpaced input, and triad placements can carry the wrong inversion label. Four P2 items close the test, coverage, CSP and documentation gaps that let these through.

## Problem Statement

A user stepping through A minor pentatonic is shown box 4 as "1 of 5"; a beginner can be shown a mislabelled triad; a stuck input freezes the app. INTENT.md ranks wrong notes, names and fingerings as top severity.

## Goals and Non-Goals

Goals: show the pattern requested, numbered as AC-SC-3.1 defines; never mislabel an inversion; keep the UI responsive whatever the input does; close the test and documentation gaps. Non-goals: new features, tunings or scales; non-Mac platforms; redesigning the tuner DSP.

## Functional Requirements

| ID | Pri | Requirement | Regression test |
|---|---|---|---|
| FR-001 | P0 | Pattern stepper starts at box 1 (root on string 6), labels by box number ("Box 1 of 5"), cycles in box order | `mackey-voicing` selection and view tests; UI label test |
| FR-002 | P0 | Tuner events are wall-clock throttled; the analysis loop is bounded and yields; an unpaced input gets a notice | `src-tauri` throttle test with fake clock; `mackey-audio` loop test with fake ring |
| FR-003 | P0 | Triads ascend low to high with the bass of the labelled inversion; no wrong placements | `proptest` over chords, string sets, tunings, capos |
| FR-004 | P2 | Traceability names existing tests; AC-SC-3.1 covered end to end | Covered by FR-001 tests plus a path check |
| FR-005 | P2 | Audio loop decision logic is testable and out of the coverage exclusion | Unit tests with a fake ring and clock |
| FR-006 | P2 | CSP has no `'unsafe-inline'` for styles, or the reason it stays is documented | Config check and note |
| FR-007 | P2 | Docs describe the new behaviour (D-91 resolved, screenshots, captions) | `make doccheck`, manual read |

Each FR's Given/When/Then acceptance criteria are in the [PRD](fretboard-foundation-auto-review-PRD.md#functional-requirements) and are the acceptance criteria of this spec.

## Non-Functional Requirements

See PRD NFR-1 to NFR-6: no regressions, no allocation or locking in cpal callbacks, at most about 30 tuner events per wall-clock second, voicing budgets kept, labels announced and matching the drawing, docs updated with each change.

## Architecture and Scope of Changes

`mackey-voicing` (patterns, selection, view, triads), `mackey-audio` (`io/pump.rs`, `io/input.rs`), `mackey-app` (`tuner.rs`, `state.rs`), `ui/` (label, mock), `tauri.conf.json`, `Makefile`, `user-docs/`, `docs/images/`. No crate or dependency additions, so no archcheck change.

## Breaking Changes

Pattern order and numbering change (box order); some triad placements disappear. If settings persist a pattern index it is migrated (checked in task FR-001). Generated bindings change only if a type does.

## Success Criteria

All seven FRs done or deferred with a reason; `make check`, UI tests and e2e green; CI green on the PR; regenerated Explore screenshot shows frets 5 to 8.

## Risks and Mitigation

Snapshots or e2e expecting old triads or order: review diffs by hand. Tuner fix cannot be verified on real hardware here: fake clock and ring tests, plus an ALSA null-device check recorded in the notes.

## Timeline

One work session, P0 first, as stable commit groups.

## References

Source PRD: [fretboard-foundation-auto-review-PRD.md](fretboard-foundation-auto-review-PRD.md). Original spec: [archive](../spec.md).
