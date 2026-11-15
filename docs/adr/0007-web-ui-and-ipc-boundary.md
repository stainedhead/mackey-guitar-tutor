# ADR 0007: React and SVG UI with a typed, thin IPC boundary

- Status: Proposed
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
