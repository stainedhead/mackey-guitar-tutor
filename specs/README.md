# Specs

Back to the [root README](../README.md). Rules and workflow: [AGENTS.md](../AGENTS.md#feature-specification-workflow).

Active feature specs live here; completed ones move to `specs/archive/` (via `/dev-flow:archive-spec`).

## Active specs

| Spec | Covers | Status |
|---|---|---|
| [261115-fretboard-foundation/](261115-fretboard-foundation/README.md) | Roadmap P0 and P1: scaffold, `mackey-theory`, `mackey-voicing`, `mackey-audio` (tuner, synth), Tauri app and SVG fretboard UI, CI | Specified and reviewed; implementation not started |

The product PRD lives in `docs/prd/` (with `INTENT.md` at the root). Each roadmap phase becomes a spec: run `/dev-flow:create-spec` on the relevant PRD file to create its `YYMMDD-<feature>/` directory here.

## Naming

`YYMMDD-<feature-name>/`: the prefix is the date the spec was **created** (when `create-spec` ran), not the implementation date.

## Per-spec contents

| File | Purpose |
|---|---|
| `spec.md` | Feature specification (populated from the PRD) |
| `status.md` | Phase progress tracking: **update after every task (mandatory)** |
| `plan.md` | Implementation plan |
| `tasks.md` | Task breakdown |
| `research.md` | Research findings |
| `data-dictionary.md` | Data structures and schemas (Rust types, serde shapes, SQLite tables) |
| `architecture.md` | Architecture and component design (which crates change) |
| `implementation-notes.md` | Decisions and gotchas |
| `<feature>-PRD.md` | Source PRD (moved here by `create-spec`) |
| `<feature>-status.md` | Orchestration dashboard written by `/dev-flow:implm-frm-prd` and the other orchestrators |
| `README.md` | What the spec covers and an index of its files |

## Lifecycle

1. `/dev-flow:create-prd` → `/dev-flow:create-spec` creates `specs/YYMMDD-<feature>/`.
2. Implement with `/dev-flow:implm-from-spec`, updating `status.md` as you go.
3. At 100% in `status.md`, run `/dev-flow:archive-spec` to move it to `specs/archive/`.
