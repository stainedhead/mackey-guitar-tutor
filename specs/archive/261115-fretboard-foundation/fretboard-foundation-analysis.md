# Dev-Flow Process Analysis

**Feature:** fretboard-foundation
**Spec directory:** specs/archive/261115-fretboard-foundation/
**Report generated:** 2026-11-15

---

## 1. Executive Summary

Roadmap phases P0 and P1 were built in one branch (`claude/project-thread-e8va25`): the Cargo workspace and `mackey-theory`; `mackey-voicing` (shapes, search, triads, arpeggios, scale patterns, `FretboardView`); `mackey-audio` (MPM tuner, Karplus-Strong synth, cpal glue); a Tauri 2 shell with a React/SVG UI (Explore, Tuner, Settings, left-handed, tunings, capo); and CI for Linux, macOS, e2e, bindings, universal Mac build and the iOS Simulator. An automated review then found three P0 bugs, which were fixed in a nested review-fix spec.

**Total runtime:** about 3 h 27 min on this branch (first branch commit 2026-11-15T19:11Z, last implementation commit 22:38Z; Steps 11-13 followed). The dashboard's process start is 19:24Z, close to the first spec commit.
**Overall assessment:** Productive and well documented. The main costs were CI-only platform failures that could not be reproduced on Linux, and a review that caught serious bugs the original tests had not.

---

## 2. Step-by-Step Timing

From the dashboard (`fretboard-foundation-status.md`), cross-checked with git.

| Step | Name | Start (Z) | End (Z) | Min | Key outputs |
|---|---|---|---|---|---|
| 1 | Create spec from PRD | 19:24 | 19:36 | 12 | spec, plan, tasks (61 tasks, phases A-E) |
| 2 | Review spec | 19:36 | 19:43 | 7 | "Implementation-ready" after fixes |
| 3 | Implement product | 19:43 | 21:58 | 135 | 3 crates, Tauri app, UI, CI jobs |
| 4 | Documentation | 22:02 | 22:20 | 18 | READMEs, user-docs, screenshots, roadmap, traceability |
| 5 | Code and design review | 22:21 | 22:24 | 3 | 7 findings (3 P0, 4 P2) |
| 6 | Review PRD | 22:24 | 22:24 | 1 | FR-003 raised to P0 |
| 7 | Archive original spec | 22:24 | 22:24 | 1 | moved to `specs/archive/` |
| 8 | Review-fix spec | 22:25 | 22:25 | 1 | nested review spec |
| 9 | Implement fixes | n/a | 22:36 | n/a | FR-001 to FR-007 (FR-005 partial); dashboard start time (22:52) is after the end, so it is a recording error; git shows fixes from 22:29 to 22:36 |
| 10 | Archive fixes spec | 22:37 | 22:38 | 1 | nested into the archive |
| 11 | Final quality pass | see dashboard | | | `make check`, e2e, props, cover all green |
| 12 | This report | see dashboard | | | |
| 13 | Archive confirmation | see dashboard | | | |

**Observations:**
- Step 3 was 60% of the runtime. Steps 5-10 together took about 15 minutes because the review was automated and the fixes were small.
- Step 1 and 2 timings are approximate (one worker did both).

---

## 3. Commit and Push Summary

**Total commits on the branch since `main`:** 47. PR: #3 (draft), opened earlier from this branch; not merged.

Milestone commits (git author time, UTC unless noted):

| Commit | Time | Message |
|---|---|---|
| 6bb53bf | 19:11 | Add dev-flow PRD for the fretboard foundation (P0 and P1) |
| 8056bc0 | | Create and review spec for the fretboard foundation |
| 050d938 | | Add mackey-theory: notes, intervals, scales, keys |
| a5ad296 | | Fix the iOS Simulator build and make it a real CI gate |
| b3dec27 | | Run the Tauri CLI through a root npm script so the iOS build phase can find it |
| c5065fa | | feat(voicing): instruments, curated shapes, voicing search, triads, arpeggios, patterns |
| 9ce41cc | | Add the Explore, Tuner and Settings UI with the mocked command layer |
| d47f20c | | Measure the redraw budget on a production build; link audio frameworks on iOS |
| 9f2ace8 | | Give e2e keyboard tests focus by blurring, not by clicking a fixed point |
| d286541 | 22:19 | Document the fretboard foundation |
| 9ffafcf | 22:23 | Add auto review PRD |
| 2ea778c | 22:24 | Archive the fretboard foundation spec (Step 7) |
| e561709 / 06261f3 / 8b8e4b2 / 2da3dcb | 22:29-22:36 | Fix FR-001, FR-003, FR-002, then FR-004/006/007 and the FR-005 extraction |
| 60e6573 | 22:38 | Archive the auto review fix spec (Step 10) |

The full list is `git log main..HEAD`.

---

## 4. Spec vs. Implementation Comparison

| Phase | Planned | Actual | Notes |
|---|---|---|---|
| Spec and review | 61 tasks in phases A-E | 19:24-19:43Z | on plan |
| Implementation | 60 of 61 tasks | 135 min | D16 (manual Mac and iPad hardware check) needs a device and stays open |
| Documentation | | 18 min | screenshots from the real app under Xvfb were added |
| Review fixes | 7 findings | 6 done, FR-005 partial | `io/input.rs` stays in `COVER_IGNORE` (follow-up recorded in the roadmap and traceability) |

**Skipped:** D16 hardware validation; FB-8.2 instrument choice (deferred to P1b); `release.yml`; the Swift `audio-session` plugin.
**Added:** real-app screenshot tooling (`make screenshots`), the root `package.json` Tauri wrapper, a pace guard and a new `InputNotPaced` tuner notice.

---

## 5. Token / Message Usage

Exact token counts are unavailable. The orchestrator ran the steps sequentially and used one worker agent per workstream in Steps 3 and 9, with a separate reviewer for Step 5. Estimate: high (a large multi-crate build plus two spec cycles).

---

## 6. Process Observations

### What worked well
- Spec-first with a PRD and a spec review gave a stable task list; the architecture rules (archcheck) held with no dependency violations.
- Table-driven, property and snapshot tests plus `make check` kept 14 crates' worth of changes green; final line coverage is 93.8% overall (target 80%).
- The review step is cheap and found real defects; the whole review-to-fix loop took about 15 minutes.
- Recording decisions (D-1 to D-77) as the work went made the docs step quick.

### What caused delays or rework
- **iOS CI:** the Xcode "Build Rust Code" phase could not find the Tauri CLI (`node_modules/.bin/tauri` recorded relative to the repo root, MODULE_NOT_FOUND) in the first CI iOS builds; fixed by running it through a root npm script (D-46, b3dec27). Later the Xcode link failed on undefined CoreAudio/AVFoundation symbols after `mackey-audio` joined the app; fixed with `bundle.iOS.frameworks` (D-77). Neither could be reproduced on Linux, so each cost a CI round trip.
- **e2e focus regression:** keyboard tests gave focus by clicking a fixed point, which hit a different element after the layout changed; fixed by blurring instead (9f2ace8).
- **Disk full:** the build directory filled the container disk during the run (reported by the orchestrator; no commit records it) and needed cleaning before work resumed.
- **WebKit unavailable:** Playwright WebKit cannot be downloaded in the sandbox, so e2e ran in Chrome via `PW_CHANNEL=chrome` (D-49).
- **Review found 3 P0 bugs** that tests had not: the scale-pattern stepper showed and numbered the wrong box (FR-001), the tuner flooded the UI and spun a CPU on an unpaced input (FR-002), and triad placements with an open string at fret 12 or higher were labelled with the wrong inversion (FR-003). FR-003 was raised from P1 to P0 because INTENT.md ranks a wrong note or name as the top severity.
- Dashboard bookkeeping errors (Step 9 start after end).

### Recommendations for future runs
- Run the iOS Simulator build job first and early; treat platform CI as the riskiest task.
- Add property tests that sweep whole ranges (every fret and string) while the voicing code is written, not after review: FR-003 was exactly such a boundary case.
- Test loops that read from devices against a fake clock and ring from the start, so audio glue does not need a coverage exclusion.
- Put the Chrome fallback in the default e2e setup for sandboxed environments.
- Monitor disk (`df`) and clear `target/` of old profiles between phases.

---

## 7. Manual vs. Automated Comparison

**Estimated manual duration:** about 6 to 8 weeks for one senior developer (about 55,000 added lines across three Rust crates, a Tauri shell, a React UI, CI and documentation; excludes meetings and design review).
**Actual automated runtime:** about 3.5 hours from first to last implementation commit.
**Efficiency gain:** order of 10x or more in elapsed time, with the caveat that hardware checks (D16), real-guitar tuner accuracy and sound have not been done by a person, and the code still needs human review.
