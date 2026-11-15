# ADR 0007: React and SVG UI with a typed, thin IPC boundary

- Status: Accepted (implemented in the fretboard foundation)
- Date: 2026-11-15

## Context
With a Tauri shell ([ADR 0001](0001-rust-core-tauri-shell.md)), the UI runs in the macOS web view (WKWebView) and talks to the Rust core over IPC. The fretboard needs smooth 60 fps updates, accurate hit-testing, accessibility labels and crisp rendering at any window size. Real-time state (playhead, tuner) streams from the core.

## Decision
- **UI stack:** TypeScript, React and Vite, with a small state store (Zustand). Components are plain React; no heavy UI framework.
- **Fretboard in SVG:** about 150 to 300 elements (strings, frets, inlays, dots and labels) is well within SVG's comfort zone, gives free hit-testing and ARIA labels, and scales cleanly. Canvas is kept as a fallback for dense animations (for example CAGED transitions) if profiling shows a need.
- **Rust computes, the UI draws:** the core returns a `FretboardView` (positions, colours, labels, ghosts) rather than the UI working out music theory. The UI has no copy of theory logic.
- **Typed boundary:** Tauri commands for request/response and Tauri events for streams. TypeScript types and command wrappers are generated from Rust (tauri-specta, or ts-rs if needed), and CI fails if the generated bindings are stale.
- **Timing:** streamed positions carry sample time, tempo and output latency; the UI extrapolates in `requestAnimationFrame` instead of trusting event arrival time.

## Alternatives
- **Svelte or Solid:** smaller and fast, but React is the most familiar choice for contributors and has the richest testing tools.
- **Rust UI (Leptos or Dioxus) compiled to WebAssembly:** one language end to end, but a smaller ecosystem and fewer contributors who know it.
- **Timing in Web Audio:** rejected for the same reason; see [ADR 0005](0005-sequenced-drums.md).
- **Theory logic duplicated in TypeScript for snappier UI:** rejected; two implementations would drift. IPC round trips are sub-millisecond locally.

## Consequences
- UI logic is testable with Vitest and Playwright in a normal browser against a mocked command layer, as well as end to end with tauri-driver.
- Contributors need Rust and TypeScript; the boundary keeps each side's work separate.

## Implementation notes (2026-11-15)

- **`ts-rs` was chosen over `tauri-specta`.** `tauri-specta` for Tauri 2 is still a release candidate. `ts-rs` generates types only, so command signatures are written once in a manifest (`src-tauri/src/manifest.rs`: name, argument names and TypeScript types, result type) and `make bindings` generates `ui/src/bindings/commands.ts` from it. A unit test checks that the manifest lists exactly the registered commands and the capability permissions. `make bindings-check` fails CI on any stale generated file. (spec decisions D-05, D-44)
- **Types are exported by `ts-rs` tests** (`export_bindings_*`); the target folder comes from `TS_RS_EXPORT_DIR` in `.cargo/config.toml`, so any `make test` also rewrites identical files.
- **The UI mock replays recorded states, not hand-written ones.** `src-tauri/src/bindings.rs` drives the real `AppState` through scripted scenarios and records them as `ui/src/mocks/scenarios.json`, keeping "never re-implement theory in TypeScript" true for the mock too (D-14, D-69). A path the recording does not contain gets an explicit error.
- **Revisions.** Every snapshot carries a revision; the UI applies a response only if it is newer than the last applied one, so out-of-order replies cannot show a stale neck.
- Zustand was not needed: a small store with a last-revision-wins rule is enough (`ui/src/state/`).

