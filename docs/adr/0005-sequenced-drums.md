# ADR 0005: Drum backing is sequenced from patterns, not recorded loops

- Status: Proposed
- Date: 2026-11-15

## Context
Players need to start slow and build up to full speed, so drum backing must play at any tempo from very slow to fast, stay in time with the metronome and practice visuals, and follow song sections.

## Decision
Represent drum grooves as step patterns (JSON) and play them through a General MIDI drum SoundFont with rustysynth (a pure-Rust SoundFont synthesizer) inside the cpal audio callback, scheduled from the shared `TransportClock`. Drum kits are CC0 or public-domain multi-sample kits played by a small one-shot sampler, with a General MIDI SoundFont (MIT) as fallback; see [sound-assets.md](../sound-assets.md).

## Alternatives
- **Recorded audio loops with time-stretching:** more realistic at the native tempo, but artefacts at large stretch ratios (e.g. 50% speed), larger app size, and loops must be licensed or recorded.
- **Web Audio in the UI:** convenient, but timing would depend on the web view, which is throttled when the window is hidden and would differ again on other platforms' web views later.
- **Third-party drum engine:** adds a dependency and licensing for a feature that a small sequencer covers.

## Consequences
- Clean sound at every tempo, tiny assets, easy variations, fills and per-part mutes.
- Realism depends on the SoundFont and humanisation; a better kit can be swapped in later.
- The pattern format doubles as the future user-pattern format.
