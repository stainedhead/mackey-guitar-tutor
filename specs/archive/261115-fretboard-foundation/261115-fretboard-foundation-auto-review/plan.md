# Plan: Fretboard Foundation Auto Review Fixes

**Created:** 2026-11-15 · **Status:** Planning

## Development Approach

P0 first: FR-001, FR-003, FR-002; then FR-004 to FR-007. TDD for each. The PRD recommends worktrees and agent teammates per workstream; this run is one sequential session by a single worker, so fixes are committed as stable groups on the run branch.

## Phase Breakdown

Group 1 FR-001 (voicing, UI, screenshot). Group 2 FR-003. Group 3 FR-002 and FR-005 (audio loop and throttle). Group 4 FR-004, FR-006, FR-007.

## Critical Path

FR-002 and FR-005 share files. Screenshots are regenerated last, once.

## Testing Strategy

Table-driven label tests, `proptest` for triads, fake clock and fake ring for the tuner, Vitest and Playwright for the label. `make check` before each push; `make cover` runs in CI only.

## Rollout Strategy

Part of the open draft PR; merged by the user.

## Success Metrics

All FR acceptance criteria met or deferred with reasons; CI green.
