# ADR 0002: Pure, UI-free music theory and voicing core

- Status: Proposed
- Date: 2026-11-15

## Context
Explorer, practice sessions and songs all need the same answers: how a chord is spelled in a key, where its tones sit on the neck, which voicings are playable. Getting these wrong is the fastest way to lose a learner's trust.

## Decision
Put music theory (`MusicTheory`) and voicing search (`VoicingEngine`) in local Swift packages with no UI, I/O or global state. All types are value types; all functions are deterministic. Feature packages (`PracticeKit`, `SongKit`) depend on them; they never depend on features.

## Consequences
- The core can be tested exhaustively and fast, and reused on iPad or a future CLI.
- Layer rules are documented in `CLAUDE.md` so human and agent contributors keep the boundary.
