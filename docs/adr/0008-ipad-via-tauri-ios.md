# ADR 0008: iPad from the start, through Tauri 2's iOS target

- Status: Accepted (J asked for iPad now, 2026-11-15)
- Date: 2026-11-15

## Context
J wants the app on iPad as well as the Mac from the start. The core is Rust and the UI is React in a web view ([ADR 0001](0001-rust-core-tauri-shell.md), [ADR 0007](0007-web-ui-and-ipc-boundary.md)).

## Decision
Build the iPad app from the same workspace with Tauri 2's iOS target. Same core crates and same React UI, with a responsive, touch-first layout. Platform-specific code is limited to a Swift Tauri plugin for `AVAudioSession` and iOS file import. Every phase's exit criteria apply to both Mac and iPad.

## Alternatives
- **Native SwiftUI iPad app over the Rust core (UniFFI):** best iPad feel, but a second UI codebase to maintain.
- **Defer iPad:** rejected by J.

## Risks and mitigations
| Risk | Mitigation |
|------|------------|
| Tauri's iOS support is younger than its desktop support, and fewer plugins work on mobile | P0 walking skeleton runs on iPad in week 1; keep plugin use minimal; anything missing becomes a small in-repo Swift plugin |
| cpal does not configure `AVAudioSession` | Own Swift plugin for session category, mode, buffer size, interruptions and route changes |
| iPad apps cannot be unsigned | Build and test in CI and the iOS Simulator first; free Apple ID for device installs later; Developer Program after that ([platforms.md §7](../platforms.md#7-ipad-signing-and-validation-decided)) |
| Hover-based interactions do not exist on touch | UI rule: every hover action has a tap equivalent; Playwright runs UI tests in a touch-emulated iPad viewport |
| No WebDriver for WKWebView, so tauri-driver cannot drive the real app on Mac or iPad | UI tests run in Playwright's WebKit against a mocked command layer; a short manual smoke checklist on both devices per release |

## Consequences
One codebase covers both devices. P0 grows slightly to set up the iOS build. The UI must be designed responsive and touch-first from the first screen, which also helps the Mac UI.
