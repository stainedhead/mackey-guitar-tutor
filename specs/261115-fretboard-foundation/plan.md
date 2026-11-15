# Plan: Fretboard Foundation

**Feature:** fretboard-foundation
**Created:** 2026-11-15
**Status:** Ready for implementation (reviewed in Step 2)
**Spec:** [spec.md](spec.md) · **Tasks:** [tasks.md](tasks.md) · **Architecture:** [architecture.md](architecture.md)

## Development Approach

1. **Inside out, test first.** Build the pure crates first (theory, then voicing), each task starting from failing table-driven, property and snapshot tests (AGENTS.md TDD). The audio DSP is also pure and is built the same way. Only then add the shell and UI, which are thin.
2. **Rust computes, the UI draws.** The first end-to-end slice (task D1) is one command, `fretboard_view`, returning a real `FretboardView`; the UI renders only what it receives. No theory in TypeScript, ever (ADR 0007).
3. **Walking skeleton early.** The roadmap wants the full path (UI → Rust command → neck; Rust → sound) proven in week 1 to retire Tauri iOS risk. Here that means: D1 (Tauri scaffold), E5 and E6 (macOS and iOS CI jobs) start as soon as D1 compiles, in parallel with Phase B and C work, and a draft pull request is opened at that point so CI feedback arrives early.
4. **Decisions are recorded.** Every choice the PRD left open is a numbered decision (D-nn) in [implementation-notes.md](implementation-notes.md); deviations go in the same file.
5. **One branch, one pull request.** All work is on the run's branch; the PR is opened at the end (Step 14 of the orchestration) unless a draft is needed earlier for CI feedback (see 3).
6. **Docs move with code.** Each task updates the READMEs and docs it touches; `make doccheck` is part of `make check`.

## Phase Breakdown

| Phase | Tasks | Outcome | Exit (all must hold) |
|-------|-------|---------|----------------------|
| **A** Workspace scaffold and `mackey-theory` | A1 to A11 | Crates exist and build; theory engine complete | AC-P0-1, AC-P0-2, AC-P0-3, AC-FB-2.1 (spelling part), AC-FB-4.1 (analysis part); `make check` and `make props` green |
| **B** `mackey-voicing` | B1 to B16 | Tunings, shapes, search, triads, arpeggios, patterns, selection, `FretboardView`, playback plan, benchmarks | AC-FB-2.1, 2.2, 4.1 (view), 5.1, 5.2, 5.3, 5.6, 6.1, 6.2, 7.1, 9.1a, 1.6 (data), AC-E-FB-4, AC-SC-3.1, AC-SC-4.1, AC-LH-4.1, AC-LH-5.1; release timing test within budget |
| **C** `mackey-audio` | C1 to C9 | Pure pitch detector, tuner logic, synth; cpal glue | AC-TU-1.1, 1.2, 1.3, 2.1, 3.1, 5.1 (target data), AC-NF-9; no-allocation render test; device-absent path returns an error, not a panic |
| **D** App and UI | D1 to D16 | Tauri shell, commands, settings, bindings, React UI with fretboard, Explore, Tuner, Settings, Vitest, Playwright | AC-P0-5, AC-FB-9.1b, 9.1c, 1.6 (UI), AC-LH-1.1, AC-UX-7, AC-FR-028, AC-NF-24, AC-E-FB-8, AC-E-TU-2 (state renders) |
| **E** Makefile, AGENTS.md, CI | E1 to E9 | New targets documented, CI jobs, docs | AC-P0-4: every job green on the PR; `make check` runs UI steps; `make doccheck` passes |

Run order and parallelism:

- A1 first and alone (it unblocks everything).
- After A2, Phase C (C1 to C9) can proceed in parallel with A3 to A11 and Phase B, because `mackey-audio` needs only `Pitch`, `Note` and `ReferencePitch`.
- B starts after A11 (B needs keys, chords and spelling).
- D1 after B14 and C9 (the app needs all three core crates); D7 (UI scaffold) can start in parallel with D1 once the Node toolchain is chosen; D8 onwards needs D6's generated bindings.
- E1 after D7; E3, E5, E6 as soon as D1 builds; E4 after D15; E8 last.

## Critical Path

`A1 → A2 → A3 → A4 → A5 → A7 → A9 → A11 → B1 → B2 → B3 → B4 → B5 → B9 → B10 → B14 → D1 → D4 → D5 → D6 → D8 → D10 → D11 → D13 → D14 → D15 → E4`

The riskiest tasks, in order of "most likely to take longer than the estimate":

1. **E6** iOS Simulator build in CI (unknown commands on the runner image; start early).
2. **B5** voicing search quality and budget (5 ms) and **B3** the curated shape data (hand-written, easy to get one fret wrong; the consistency tests in B2 guard it).
3. **B9/B10** selection and `FretboardView` (largest surface; many ACs).
4. **D5/C6** audio device glue and the analysis thread (not unit-testable on CI beyond fakes).
5. **D8** the SVG component (geometry, mirroring, accessibility).

## Testing Strategy

| Level | Tools | What |
|-------|-------|------|
| Unit and table-driven | `cargo test` | Hand-written expected tables for spelling (252 chord rows), keys (every offered key), curated shapes (27 + 2 F), tunings, triads, patterns, tuner states. Cases are `const CASES` with a label in the assertion message. |
| Property | `proptest` | AC-P0-3 invariants; FB-5.3 across random chords, tunings and capos; triad and arpeggio invariants; selection never invalid after random actions; parser never panics; detector never panics; pure-sine accuracy. `make props` runs 10,000 cases in release on Linux CI; regressions are committed. |
| Snapshot | `insta` | Parser output (valid and invalid inputs), `FretboardView` JSON for 8 named scenarios. |
| Benchmark and budget | `criterion` plus a release-mode timing test | AC-FB-9.1a (≤ 2 ms view, ≤ 5 ms search), parser ≤ 1 ms per 1,000 characters, detector ≤ 2 ms per window. |
| Real-time rule | `assert_no_alloc` | `Synth::apply` and `render` allocate nothing (debug build test). |
| Integration (Rust) | `cargo test` in `src-tauri` with `FakeAudio` | Explorer commands, settings, panic guard, no-device path, tuner target and throttle logic. |
| UI unit and component | Vitest + Testing Library | Geometry, shortcuts (AC-UX-7), announcements, gestures, tokens (contrast), components against recorded scenarios. |
| End to end (browser) | Playwright (WebKit) with the mocked command layer | AC-P0-5, AC-FB-9.1b/c, AC-FB-1.6, AC-LH-1.1, tuner states, iPad layouts, axe scan. Desktop 1280×800, iPad landscape 1180×820, iPad portrait 820×1180. |
| Build | CI | Linux and macOS Rust; universal `tauri build`; iOS Simulator build. |
| Manual | Checklist in `implementation-notes.md` | AC-MAN-1, AC-E-TU-2 on device, cold start ≤ 2 s, playback latency, VoiceOver pass, colour-blind visual check. |

Not tested here, by design: real guitar recordings (follow-up), real iPad device behaviour (device validation step), release signing.

## Rollout Strategy

- Branch `claude/project-thread-e8va25` (named by the orchestrator); implementation commits per task or small group; each commit message references `specs/261115-fretboard-foundation/`.
- A draft PR is opened once D1 builds (or at the latest at the end of Phase D) so the macOS and iOS jobs run on real runners. Fixes to workflow files are made on the same branch.
- Merge when CI is green (AC-P0-4); no feature flags; no data migration (settings JSON is new and versioned from 1).
- After merge the roadmap marks P0 done and P1 done except FB-8.2, recorded fixtures and the audio-session plugin (carried in `docs/prd/09-traceability.md` open items).
- Archive: at 100% in `status.md`, `/dev-flow:archive-spec` moves this directory to `specs/archive/`.

## Success Metrics

| Metric | Target |
|--------|--------|
| ACs with passing automated tests | 37 of 38 PRD ACs (AC-MAN-1 is manual; AC-E-TU-2's rendered state is automated, the OS behaviour is manual) plus all 19 SAC |
| `make check` | Green locally and in CI |
| Coverage | ≥ 80% lines overall; new pure-crate code ~100% |
| Performance | `fretboard_view` ≤ 2 ms; search ≤ 5 ms p95 (release); cycling redraw ≤ 16 ms (mock) |
| Tuner | ±3 cents on all 24 synthetic string cases; first reading ≤ 150 ms |
| Known wrong-note or wrong-spelling bugs | 0 |
| CI jobs required and green | 7 (rust on macOS and Linux, ui, e2e, bindings, mac-app, ios-sim) |

## Acceptance Criteria Traceability

Every AC from the PRD, the task that implements it and the task that tests it. "Impl / test" are task IDs in [tasks.md](tasks.md).

| AC | FRs | Implemented in | Tested in |
|----|-----|----------------|-----------|
| AC-P0-1 | 010, 012 | A6, A7 | A10 |
| AC-P0-2 | 010 | A8 | A8 |
| AC-P0-3 | 007, 009, 012 | A2, A3, A5, A7 | A2, A3, A5, A7 |
| AC-P0-4 | 006 | E3, E4, E5, E6 | CI on the PR |
| AC-P0-5 | 003, 021 | D1, D6, D8 | D15 |
| AC-FB-2.1 | 008, 009, 019 | A4, A7, B10 | A7, B10, B16 |
| AC-FB-2.2 | 020 | B9 | B9, D4 |
| AC-FB-4.1 | 010, 011, 019 | A8, A9, B10 | A9, B10 |
| AC-FB-5.1 | 014, 015 | B2, B5 | B2, B5 |
| AC-FB-5.2 | 014, 015 | B2, B3, B5 | B5 |
| AC-FB-5.3 | 015 | B4, B5 | B5, B12 |
| AC-FB-6.1 | 016 | B6 | B6 |
| AC-FB-6.2 | 016 | B6 | B6 |
| AC-FB-7.1 | 017 | B7 | B7 |
| AC-FB-9.1a | 015, 019 | B5, B10 | B13 |
| AC-FB-9.1b | 023 | D8, D11 | D15 |
| AC-FB-9.1c | 023 | B9, D11 | D11, D15 |
| AC-FB-1.6 | 019, 021 | B10, D9 | B10, D15 |
| AC-FB-5.6 | 015 | B5 | B5 |
| AC-E-FB-4 | 020 | B9 | B9, D4 |
| AC-E-FB-8 | 024 | C5, D5 | D5 |
| AC-SC-3.1 | 018 | B8 | B8 |
| AC-SC-4.1 | 018 | B8 | B8 |
| AC-LH-1.1 | 021 | D8 | D15 |
| AC-LH-4.1 | 014, 015 | B3, B5 | B3, B5 |
| AC-LH-5.1 | 019, 022 | B10, D10 | B10 |
| AC-TU-1.1 | 026, 027 | C2, C3 | C3 |
| AC-TU-1.2 | 026 | C2 | C2 |
| AC-NF-9 | 026 | C3 | C3 |
| AC-TU-1.3 | 026 | C2 | C2 |
| AC-TU-2.1 | 027 | C3 | C3 |
| AC-TU-3.1 | 026 | C2 | C2 |
| AC-TU-5.1 | 027 | C7, D5 | C3, D5 |
| AC-E-TU-2 | 027 | D5, D12 | D12 (state), manual (OS) |
| AC-UX-7 | 023 | D11 | D11 |
| AC-FR-028 | 028 | D3 | D3 |
| AC-NF-24 | 004 | D2 | D2 |
| AC-MAN-1 | all | D16, E6 | manual |

Builder-added criteria (spec.md, "Derived acceptance criteria"), covering FRs the PRD's ACs leave untested: SAC-01 (A1, D1), SAC-02 (D1), SAC-03 (D2), SAC-04 (E1, E2), SAC-05 (A5), SAC-06 (A9), SAC-07 (B1), SAC-08 (B2), SAC-09 (B5), SAC-10 (B10, B12), SAC-11 (B9), SAC-12 (D10), SAC-13 (B11, D5), SAC-14 (D13, D15), SAC-15 (C3), SAC-16 (D5, D12, D15), SAC-17 (D3, D13), SAC-18 (D4, D14), SAC-19 (E5).

Non-AC requirements and the task that carries them (so none is orphaned): E-FB-1 (A8, D10), E-FB-2 (A8, B10), E-FB-3 (B5), E-FB-5 (B10, D9), E-FB-6 (B4, B12), E-FB-7 (A5, D10), E-FB-9 (D13), E-TU-1 (D12), E-TU-3/4 (C3, D12), E-TU-5 (C6), E-TU-6 (C3), FB-1.7 (D8, B11), FB-8.4 (C4, D5), FB-3 (B10), TU-4 (C4, D5, D12), TU-6 (C6, D12), TU-7 (D5), G-1.6 (A7, B16), NF-5 (B10, D8, D15), NF-7.3 (D1), NF-8.1 (C4).
