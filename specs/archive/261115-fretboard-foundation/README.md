# specs/archive/261115-fretboard-foundation

Spec for the **fretboard foundation** (roadmap P0 and P1): the Cargo and Tauri scaffold, `mackey-theory`, `mackey-voicing`, `mackey-audio` (tuner and synth), the `src-tauri` app and the React/SVG fretboard UI, with CI for macOS, Linux, the universal Mac build and the iOS Simulator. Created 2026-11-15 from [fretboard-foundation-PRD.md](fretboard-foundation-PRD.md). Back to [specs/](../README.md) and the [root README](../../../README.md).

Authority: [INTENT.md](../../../INTENT.md) → [docs/prd/](../../../docs/prd/README.md) → [docs/](../../../docs/README.md) → this spec. Rules: [AGENTS.md](../../../AGENTS.md).

## Files

| File | Purpose |
|---|---|
| [fretboard-foundation-PRD.md](fretboard-foundation-PRD.md) | Source PRD (moved here by `create-spec`); adapter over `docs/prd/`, numbers the requirements FR-001..028 |
| [spec.md](spec.md) | The specification: goals, non-goals, FRs, NFRs, scope of changes, 38 PRD acceptance criteria plus 19 derived ones (SAC), risks, open questions Q-1..Q-11 |
| [data-dictionary.md](data-dictionary.md) | Concrete Rust types per crate (theory, voicing, audio, app), which derive serde and ts-rs, and the UI view-state types |
| [architecture.md](architecture.md) | Crates, folders, dependency edges, algorithms, Tauri commands and events, audio thread design, UI component tree, bindings generation, CI jobs |
| [plan.md](plan.md) | Development approach, phases A to E, critical path, testing strategy, rollout, AC traceability matrix |
| [tasks.md](tasks.md) | 61 tasks with IDs, dependencies, estimates, files and acceptance criteria |
| [research.md](research.md) | Answers and VERIFY items: MPM vs YIN, Karplus-Strong, ts-rs with Tauri, iOS CI without signing, Playwright WebKit, Linux Tauri CI, mic permission |
| [implementation-notes.md](implementation-notes.md) | Technical decisions D-01..D-33, edge cases, the Step 2 spec review, the manual checklist, deviations and lessons |
| [status.md](status.md) | Phase progress; **update after every task** |
| [fretboard-foundation-status.md](fretboard-foundation-status.md) | Orchestration dashboard for `/dev-flow:implm-frm-prd` (the 14 steps of the run) |

## Reading order

1. [spec.md](spec.md) for what and why, then [architecture.md](architecture.md) and [data-dictionary.md](data-dictionary.md) for how.
2. [tasks.md](tasks.md) for the work, in phase order (A theory, B voicing, C audio, D app and UI, E Makefile and CI); [plan.md](plan.md) for order and risks.
3. [implementation-notes.md](implementation-notes.md) before changing anything the decisions cover.

At 100% in `status.md`, run `/dev-flow:archive-spec` to move this directory to `specs/archive/`.

## Review-fix spec

[261115-fretboard-foundation-auto-review/](261115-fretboard-foundation-auto-review/README.md): the seven fixes (FR-001 to FR-007) from the Step 5 review, with its own PRD, spec, status and implementation notes.
