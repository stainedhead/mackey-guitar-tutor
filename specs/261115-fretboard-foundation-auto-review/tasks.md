# Tasks: Fretboard Foundation Auto Review Fixes

**Created:** 2026-11-15 · **Status:** In progress

## Progress Summary

0/7 tasks complete. Update this file and [status.md](status.md) after every task.

## Conventions

TDD: write the regression test first and see it fail on the current code; then the minimal fix; then a brief self-review recorded in [implementation-notes.md](implementation-notes.md). Test names carry the FR and AC id.

## Tasks

| ID | Pri | Task | Depends on | Regression test (fails first) | Done |
|---|---|---|---|---|---|
| FR-001 | P0 | Stepper defaults to box 1, label "Box N of M", box-number order; fix caption, regenerate screenshot | none | `fr001_ac_sc_3_1_first_press_shows_box_1` in `mackey-voicing` (selection and view), plus UI label test | [ ] |
| FR-002 | P0 | Wall-clock throttle, bounded pump iteration with yield, unpaced-input notice | none | `fr002_unpaced_input_emits_at_most_35_readings_per_second` (fake clock), `fr002_pump_iteration_is_bounded` | [ ] |
| FR-003 | P0 | Triads ascend with the labelled bass; unavailable when none | none | `fr003_triad_lowest_note_matches_inversion` (proptest) and the C Standard S345 case | [ ] |
| FR-004 | P2 | Traceability test paths exist; AC-SC-3.1 covered end to end | FR-001 | path-existence check over the traceability tables | [ ] |
| FR-005 | P2 | Move loop decision logic behind a fake-ring function; narrow coverage exclusion | FR-002 | unit tests of the extracted function | [ ] |
| FR-006 | P2 | Remove or justify `'unsafe-inline'` for styles | none | config assertion test (CSP string) | [ ] |
| FR-007 | P2 | Doc drift: D-91 resolved, captions, screenshots, READMEs | FR-001 to FR-006 | `make doccheck` | [ ] |
