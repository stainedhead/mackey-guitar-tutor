# Status: Fretboard Foundation

**Feature:** fretboard-foundation
**Created:** 2026-11-15
**Last updated:** 2026-11-15T22:18Z
**Spec:** [spec.md](spec.md) · **Tasks:** [tasks.md](tasks.md) · **Orchestration dashboard:** [fretboard-foundation-status.md](fretboard-foundation-status.md)

**Update this file after every task and every phase. It is the single source of truth for progress.**

## Overall Progress

| Phase | Name | Status | Notes |
|-------|------|--------|-------|
| 0 | Initial research (PRD, review, research questions) | Complete | PRD reviewed (review-prd) before this run; research seeded in [research.md](research.md) |
| 1 | Specification ([spec.md](spec.md)) | Complete | 28 FRs, 38 PRD ACs + 19 SAC, 11 open questions with owners |
| 2 | Research and data modelling ([research.md](research.md), [data-dictionary.md](data-dictionary.md)) | Complete | VERIFY items assigned to tasks |
| 3 | Architecture and planning ([architecture.md](architecture.md), [plan.md](plan.md)) | Complete | |
| 4 | Task breakdown ([tasks.md](tasks.md)) | Complete | 61 tasks in phases A to E |
| 5 | Implementation (code and [implementation-notes.md](implementation-notes.md)) | Complete | 60 of 61 tasks; D16 (manual Mac and iPad checklist) needs a device and is deferred |
| 6 | Completion and archival | Complete | Archived 2026-11-15 by the orchestrator (Step 7) with D16 outstanding; review fixes are in `261115-fretboard-foundation-auto-review/` |

Spec review (Step 2): done 2026-11-15, verdict **Implementation-ready** after fixes; findings in [implementation-notes.md](implementation-notes.md#spec-review-step-2).

## Implementation Progress (Phase 5)

60 of 61 tasks complete (D16 needs a device). Per-phase detail is in [tasks.md](tasks.md#progress-summary).

| Implementation phase | Tasks | Done | Status |
|----------------------|-------|------|--------|
| A Workspace scaffold and `mackey-theory` | 11 | 11 | Complete |
| B `mackey-voicing` | 16 | 16 | Complete |
| C `mackey-audio` | 9 | 9 | Complete |
| D `src-tauri` and `ui/` | 16 | 15 | In Progress (D16, the manual Mac and iPad checklist, needs a device) |
| E Makefile, AGENTS.md, CI, docs | 9 | 9 | Complete |

## Phase 0 checklist

- [x] Spec directory created (`specs/archive/261115-fretboard-foundation/`)
- [x] Research questions identified (research.md Q1 to Q9)
- [x] Phase files initialised and populated (spec, research, data dictionary, architecture, plan, tasks, implementation notes)
- [x] PRD and orchestration dashboard moved into the spec directory

## Blockers

None. Open questions that need J (defaults chosen, work not blocked): Q-1, Q-2, Q-4, Q-7, Q-9, Q-10, Q-11 in [spec.md](spec.md#open-questions).

## Recent Activity

| Date (UTC) | Activity |
|------------|----------|
| 2026-11-15T22:18Z | Step 4 documentation: real-app screenshots (`make screenshots`, D-89), edge-arrow overlap fixed (D-90), root README, architecture section 8, ADR 0007 and 0008 notes, roadmap status, traceability test references, user-docs (getting-started, explorer, tuner, settings, troubleshooting), docs/images README |
| 2026-11-15T21:40Z | Phase D (D1 to D15): `mackey-app` state, 32 commands, settings, tuner event, bindings and recorded mock scenarios; the Explore, Tuner and Settings UI with the shared SVG fretboard; Vitest (91 tests) and Playwright specs (desktop, iPad, iPad portrait, narrow); screenshots in `docs/images/`. D16 waits for a Mac |
| 2026-11-15T21:30Z | Phase B complete: `mackey-voicing` (instrument, curated and CAGED shapes, search, fingering, triads, arpeggios, patterns, selection, FretboardView, playback plan); about 125 tests incl. properties at 10,000 cases, 94% workspace line coverage, search 0.3 ms, view under 1 ms |
| 2026-11-15T21:00Z | Phase C complete (mackey-audio worker): MPM detector, tuner state machine, Karplus-Strong synth with allocation-free render, playback plans, cpal glue (`OutputHandle`, `TunerInput`), 94 tests plus doc tests, detector 0.12 ms per window; decisions D-80 to D-88; deny.toml gains BSD-1-Clause (dev-dependency `assert_no_alloc`); Makefile COVER_IGNORE narrowed to the three cpal files |
| 2026-11-15T20:08Z | Phase A complete: mackey-theory ts feature, README, docs, 99% line coverage; make check, make cover, make props green |
| 2026-11-15T20:05Z | mackey-theory chord-symbol parser with diagnostics and snapshots (A8), diatonic chords, Roman numerals and function analysis (A9); 30 proptest properties pass at 10,000 cases |
| 2026-11-15T20:01Z | mackey-theory chords, display spelling and the 252-row AC-P0-1 table (A6, A7, A10); parser benchmark 76 microseconds per 1,000 valid symbols |
| 2026-11-15T19:58Z | Phase A: workspace scaffold and mackey-theory notes, intervals, scales, keys (A1 to A5) done; voicing and audio crate scaffolds stay with B1 and C1 |
| 2026-11-15T20:05Z | Walking-skeleton infrastructure (Step 3, infrastructure worker): `ui/` scaffold, `src-tauri` (`mackey-app`) with `AppError` and panic guard, ts-rs bindings with stale check, Makefile UI/app targets, AGENTS.md, CI jobs (`ui`, `e2e`, `bindings`, `mac-app`, `ios-sim`), deny.toml MPL-2.0 exceptions (D-42 to D-49). Scaffold parts of D1, D2, D6, D7, D15, E1 to E7 and E9 done; macOS and iOS jobs written but unrun. Task counts above are updated by the integrating step |
| 2026-11-15T19:42Z | Step 2 spec review complete; gaps fixed (19 SAC added, command-wrapper generator, revision counter, E9 xtask task, edge cases, non-goals, Q-9 to Q-11); Phases 0 to 4 marked complete |
| 2026-11-15T19:27Z | Step 1 create-spec: spec directory created from the PRD; all phase files written |
