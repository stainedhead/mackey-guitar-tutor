# ADR 0006: Audio input analysis runs on device

- Status: Proposed
- Date: 2026-11-15

## Context
The tuner (v1), looper, ear training and chord recognition all listen to the player through the microphone or an audio interface.

## Decision
All audio input is processed on device, in real time, in a new `AudioInputKit` package: pitch detection (McLeod / YIN), onset detection, and chroma-based expected-chord verification, with Core ML as the upgrade path. Recordings are stored only locally. Microphone permission is requested the first time a listening feature is opened, with a plain explanation.

## Alternatives
- **Cloud recognition service:** more model capacity, but adds latency, cost, an account, and sends the user's audio off device.
- **Third-party DSP library:** possible for chroma/constant-Q, but the v1 need (a tuner) is small enough to implement and test directly.

## Consequences
Works offline, keeps NF-7 (no content leaves the device), and needs no backend. Chord recognition accuracy is bounded by on-device models, mitigated by verifying an *expected* chord rather than recognising any chord.
