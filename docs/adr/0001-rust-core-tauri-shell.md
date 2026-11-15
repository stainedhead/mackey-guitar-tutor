# ADR 0001: Rust core with a Tauri shell

- Status: Accepted (chosen by J, 2026-11-15)
- Date: 2026-11-15
- Supersedes: the earlier draft of this ADR, which proposed a native SwiftUI app for macOS

## Context
The app needs exact music theory, fast parsers and voicing search, and a sample-accurate audio clock shared by the metronome, drums, tuner and practice visuals. The fretboard must redraw smoothly while the player cycles positions. The first release targets macOS and iPad. Windows, Linux, iPhone and Android are possible later, and we want to keep that door open without paying for it now.

## Decision
- Write the core in **Rust**: music theory, chord and practice-notation parsers, ChordPro and chords-over-lyrics parsing, voicing search, the transport clock, drum sequencer, synth, and all audio input analysis.
- Ship it inside a **Tauri 2** shell on **macOS** (universal build) and on **iPad** through Tauri's iOS target ([ADR 0008](0008-ipad-via-tauri-ios.md)). The UI is TypeScript and React in the system web view; the fretboard is drawn in SVG ([ADR 0007](0007-web-ui-and-ipc-boundary.md)).
- Audio runs natively in Rust through cpal, never through Web Audio, so timing does not depend on the web view.
- Windows, Linux, iPhone and Android are deferred (J, 2026-11-15). We do not build, test or package for them yet, but we avoid choices that would block them: the core crates stay platform-neutral and macOS-specific code sits behind small traits in `src-tauri` and `mackey-audio`.

## Alternatives
- **Native SwiftUI on macOS:** best Mac feel and an easy path to iPad, but Apple only and a second codebase for anyone else.
- **Rust core with a SwiftUI shell via UniFFI:** same Rust core, native Mac UI, but still Apple only.
- **Electron:** one codebase, but a bundled Chromium makes it much heavier, and audio would still need native modules.
- **Flutter:** good canvas rendering, but a Dart UI over a Rust core adds a second bridge (FFI) and weaker desktop integration.

## Consequences
- One codebase for Mac and iPad. Adding Windows, Linux or phones later is packaging, a CI matrix entry and platform testing, not a rewrite.
- On macOS the UI renders in WKWebView (WebKit), so we test against Safari's engine. Avoiding leading-edge CSS keeps a later move to WebView2 (Windows) and WebKitGTK (Linux) easy.
- Real-time audio rules (no allocation or locks in the callback) are enforced by design in `mackey-audio`; see [architecture.md §6](../architecture.md#6-real-time-audio-design).
- Platform integration that a native app gets for free (share sheets, menus, file associations) is done through Tauri plugins, deep links and per-OS packaging.
