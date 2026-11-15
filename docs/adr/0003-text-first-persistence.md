# ADR 0003: Text is the source of truth for sessions and songs

- Status: Proposed
- Date: 2026-11-15

## Context
Practice sessions are typed in a compact notation and songs arrive as text. Users and teachers want to copy, share and back them up. Parsers will improve over time.

## Decision
Store a session's notation string and a song's ChordPro text as the canonical data. SwiftData records hold that text plus metadata (dates, run logs, voicing overrides). Parsed models are derived on load.

## Consequences
- Parser upgrades apply to all stored data with no migration.
- Export/import is just files (`.mackey`, `.cho`).
- Small cost of re-parsing on load, negligible at expected library sizes.
- Run history keys transitions by chord pair (e.g. `Em>C`), not by item index, so editing a session keeps its history.
