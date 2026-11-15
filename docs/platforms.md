# Platforms, Distribution and Storage

Status: Draft v0.1 (2026-11-15)

Decisions from J (2026-11-15):

- The app is for J's personal use for now.
- **macOS and iPad from the start.** Windows, Linux, iPhone and Android stay deferred.
- macOS ships as an **unsigned direct download**; the Mac App Store may come later.
- Audio interfaces: **rely on whatever the platform supports** (CoreAudio on both Mac and iPad); no per-device testing.
- Audio stays on the native Rust audio thread; storage is local, with **optional iCloud sync later**.

## 1. Targets

| Target | Shell | Minimum OS | Notes |
|--------|-------|------------|-------|
| macOS | Tauri 2 desktop | macOS 13 | Universal binary (Apple silicon + Intel) |
| iPadOS | Tauri 2 iOS | iPadOS 17 | Landscape-first; iPhone layout not planned |

One Rust workspace and one React UI serve both. The core crates are identical; per-platform code lives only in `src-tauri` (window and menu setup, file pickers) and in `mackey-audio` (audio session setup). See [ADR 0008](adr/0008-ipad-via-tauri-ios.md).

## 2. macOS: unsigned direct download

- CI builds a universal `.dmg` on tagged commits and attaches it to a GitHub Release.
- "Unsigned" in practice means **ad-hoc signed** (`signingIdentity: "-"`): Apple silicon will not run a binary with no signature at all, and ad-hoc signing needs no Apple account.
- First launch of a downloaded build: macOS blocks it, then **System Settings → Privacy & Security → Open Anyway**. Building locally from source avoids the prompt.
- **Known side effect:** macOS ties microphone permission to the app's signature, and an ad-hoc signature changes with every build, so expect the microphone prompt again after each update. Fine for personal use; a Developer ID certificate removes it.
- No Tauri auto-updater for now. Updates are a new download or a local build.
- **Keep App Store readiness cheap:** design as if sandboxed (write only to the app container, open user files only through pickers or drag and drop, declare `NSMicrophoneUsageDescription`), so moving to the Mac App Store later is signing and review work, not a redesign.

## 3. iPad

| Topic | Plan |
|-------|------|
| Install | iPad apps must be signed; there is no unsigned option. Two routes: a free Apple ID "personal team" in Xcode (the install expires after 7 days and must be redone), or the Apple Developer Program ($99/year: 1-year installs, TestFlight, and later App Store and iCloud). See the open decision below. |
| Build | `tauri ios build` produces an Xcode project; requires Xcode and the `aarch64-apple-ios` and `aarch64-apple-ios-sim` Rust targets. |
| Audio session | cpal's iOS backend runs output and input on RemoteIO, but does not set the session category. A small Swift Tauri plugin (`mackey-audio-session`) sets `playAndRecord` with `defaultToSpeaker` and `allowBluetoothA2DP`, `measurement` mode while the tuner is open (disables automatic gain), a preferred I/O buffer of about 5 ms, and handles interruptions (calls, Siri) and route changes (headphones in or out) by pausing the transport. |
| Input devices | Built-in mic, USB-C class-compliant interfaces and headsets, all through CoreAudio. Nothing device-specific. |
| Touch UI | Tap targets at least 44 pt; tap replaces hover (song chords, fretboard notes); swipe on the neck to change position; transport bar sized for thumbs; works in Split View and Stage Manager. A hardware keyboard gets the same shortcuts as the Mac. |
| Files | Import `.cho` and `.txt` through the iOS document picker and "Open in Mackey" from the Files app; paste works as on the Mac. |
| Background | Audio stops when the app goes to the background (no background audio mode for v1). |

## 4. Storage and sync

- Local only for now: SQLite plus text sources (sessions as notation, songs as ChordPro) in each app's own container ([ADR 0003](adr/0003-text-first-persistence.md)).
- Until sync exists, move data between Mac and iPad with **Export / Import** of a library folder (`.mackey` and `.cho` files) via AirDrop or the Files app.
- **iCloud sync later:** because the sources are plain text files, the simplest path is an iCloud Drive document container holding one file per song or session, with SQLite as a rebuildable local index; run history syncs as small append-only JSON files to avoid merge conflicts. **Dependency:** iCloud requires a signed app with the iCloud entitlement on both devices, so it needs the Developer Program and a properly signed Mac build. It lands with or after the App Store move.

## 5. Audio latency

All timing-critical audio (metronome, drums, guitar tones, tuner, looper) runs on the Rust audio thread over CoreAudio on both platforms, never in the web view. The UI only draws positions streamed from the core ([architecture.md §6](architecture.md#6-real-time-audio-design)). Bluetooth output adds a lot of latency (often well over 100 ms); the latency compensation setting in [rhythm.md](rhythm.md) covers it, and the app suggests wired or built-in output when it detects Bluetooth.

## 6. Accessibility (confirmed)

- Colour-blind-safe note palette (tested against protanopia, deuteranopia and tritanopia simulations), and every colour cue also has a shape or label (root = square, chord tone = filled circle, scale tone = ring).
- Full keyboard control on the Mac and on iPad with a hardware keyboard.
- VoiceOver on both: each fretboard position is an element labelled like "String 3, fret 2, A, root"; the transport and run state are announced.
- Respects system text size and Reduce Motion.

## 7. iPad signing and validation (decided)

J's decision (2026-11-15): use a **free Apple ID** first and move to the **Apple Developer Program** later. No accounts are set up yet; the focus is on building.

- Until then, the iPad app is validated in **CI and the iOS Simulator** only: `tauri ios build` for the simulator on every push, Playwright UI tests in an iPad viewport, and manual runs in the Simulator (which uses the Mac's speakers and microphone).
- The Simulator does not reproduce real iPad audio latency, `AVAudioSession` interruptions or touch feel. Those are checked in a short **device validation** step once a free Apple ID is set up (see [roadmap.md](roadmap.md)); timing figures are not claimed for iPad until then.
- The Developer Program comes later, when daily iPad use, TestFlight, iCloud sync or the App Store is wanted.
