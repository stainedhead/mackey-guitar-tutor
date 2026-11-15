# ADR 0001: Native SwiftUI app for macOS

- Status: Proposed
- Date: 2026-11-15

## Context
The README describes a macOS application. The fretboard must redraw smoothly while the user cycles positions, and practice mode needs tight sync between audio clicks and visuals.

## Decision
Build a native Swift 6 / SwiftUI app targeting macOS 15+, with the fretboard drawn in SwiftUI `Canvas` and audio on AVAudioEngine.

## Alternatives
- **Electron / web (React + SVG):** cross-platform and fast to prototype, but heavier, weaker audio timing, and a non-native feel on Mac.
- **Tauri + web UI:** lighter than Electron, but audio and Share extension still need native glue.
- **Flutter:** good canvas, weaker macOS integration (Share extension, menus).

## Consequences
iPad support later is cheap if UI code avoids AppKit-only APIs. Windows/Linux would need a rewrite of the UI layer; the pure Swift core could still be reused.
