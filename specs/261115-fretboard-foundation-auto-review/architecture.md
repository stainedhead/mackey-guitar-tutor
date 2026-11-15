# Architecture: Fretboard Foundation Auto Review Fixes

**Created:** 2026-11-15 · **Status:** Draft

## Architecture Overview

No structural change. The layer rules in [docs/architecture.md](../../docs/architecture.md) hold: label and order logic stay in `mackey-voicing` (UI shows data); the tuner pacing policy is a pure, testable function in `mackey-audio` or `mackey-app` with an injected clock; `mackey-app` stays an adapter.

## Component Architecture and Layer Responsibilities

`mackey-voicing`: patterns, selection, view, triads. `mackey-audio`: input worker loop and pump. `mackey-app`: throttle and events. `ui/`: renders the label it is given.

## Data Flow, Sequence Diagrams, Integration Points

Unchanged: input ring to analysis thread to throttle to `app.emit` to UI.

## Architectural Decisions

FR-006 outcome is recorded in [implementation-notes.md](implementation-notes.md).
