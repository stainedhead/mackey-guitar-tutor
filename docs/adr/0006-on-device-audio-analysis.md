# ADR 0006: Audio input analysis runs on device

- Status: Proposed
- Date: 2026-11-15

## Context
The tuner (v1), looper, ear training and chord recognition all listen to the player through the microphone or an audio interface.

## Decision
All audio input is processed on device, in real time, in the `mackey-audio` crate: a cpal input stream feeds a lock-free ring buffer, and an analysis thread runs pitch detection (McLeod / YIN), onset detection, and chroma-based expected-chord verification. An ONNX model run with ONNX Runtime (the `ort` crate) is the upgrade path, as it runs on macOS (with Core ML acceleration) and on any platform we add later. Recordings are stored only locally. The OS microphone permission (the macOS privacy prompt, declared with `NSMicrophoneUsageDescription`) is requested the first time a listening feature is opened, with a plain explanation.

## Alternatives
- **Cloud recognition service:** more model capacity, but adds latency, cost, an account, and sends the user's audio off device.
- **Web Audio `getUserMedia` in the UI:** easy, but latency is higher and less predictable than a native stream, and analysis in JavaScript is slower.
- **Writing DSP from scratch:** possible, but well-tested crates exist for pitch detection and FFT (for example `pitch-detection`, `realfft`); we wrap them behind our own trait so they can be replaced.

## Consequences
Works offline, keeps NF-7 (no content leaves the device), and needs no backend. Chord recognition accuracy is bounded by on-device models, mitigated by verifying an *expected* chord rather than recognising any chord.
