# Research: Fretboard Foundation

**Feature:** fretboard-foundation
**Created:** 2026-11-15
**Source PRD:** [fretboard-foundation-PRD.md](fretboard-foundation-PRD.md)
**Status:** Seeded with answers from existing knowledge. Items marked **VERIFY** must be confirmed in the task named, against current documentation or by a CI run, because this cloud environment has no macOS machine and versions move.

## Research Questions

### Q1. McLeod Pitch Method or YIN for the tuner?

**Answer: MPM** (decision D-06).

- Both are autocorrelation-family time-domain methods suited to monophonic instruments and a 2048-sample window. For a guitar's low strings the fundamental is often weaker than the second harmonic; both methods handle it better than naive peak picking, and both reach sub-cent accuracy on clean tones with parabolic interpolation.
- MPM's normalised square difference function (NSDF) gives a built-in **clarity** value (the height of the chosen peak, 0 to 1) that maps directly onto our "confidence" and the TU-1.4 "no stable pitch" and E-TU-6 "two strings ringing" rules. It also uses a "first peak above `k` times the maximum" rule (k about 0.93) that avoids octave errors.
- YIN's cumulative mean normalised difference with an absolute threshold (about 0.1 to 0.15) is also robust and slightly simpler, but its confidence is `1 - d'(tau)`, a less direct measure, and it is known to be more prone to picking too-high octaves with decaying, harmonic-rich plucks if the threshold is mistuned.
- Window size: a 2048-sample window at 48 kHz is 42.7 ms. Lags run to `W/2` = 1,024 samples, which covers 60 Hz (800 samples) with room; at 44.1 kHz, 735 samples. The lowest guitar fundamental we need is about 65 Hz (Drop C / drop tunings) to 73 Hz (Drop D), so this is adequate. The detector rejects sample rates below 22,050 Hz.
- Cost: direct evaluation is about `W * maxlag` multiply-adds, roughly 1.6 M per window; at a 512-sample hop (about 94 windows/s) that is about 150 M MAC/s on one thread: fine natively (budget ≤ 2 ms per window, task C8). An FFT-based autocorrelation is the upgrade path if needed, but would add a dependency, so avoid unless profiling demands it.
- Accuracy target ±3 cents (TU-1.1) is 0.17% frequency error. With parabolic interpolation on the NSDF peak, expect well under 1 cent on clean synthetic tones and a few cents on noisy low strings; harmonic-rich synthetic test tones with noise (task C2) are the gate. Smoothing by median-of-3 (TU-1.3) removes the occasional outlier reading.
- Hop and latency (NF-9): first reading needs a full 2048-sample window of tone, so about 43 ms plus one hop and thread scheduling; well under 150 ms. A lower-latency variant (shorter window for high notes) is unnecessary.
- **Open risk:** real guitars have inharmonicity, pick noise and string beating that synthetic tones only approximate. Recorded WAV fixtures are the follow-up AC (Q-1 in spec.md).

### Q2. Which plucked-string synthesis for the steel-like voice?

**Answer: Karplus-Strong with a one-pole loop filter and a pick-position comb** (decision D-07).

- Karplus-Strong: a delay line of length `sample_rate / f0` filled with a noise burst, fed back through an averaging lowpass. It is cheap (one read, one write and a few multiplies per sample per voice), needs no samples and no licensing (the SoundFont is deferred to P1b), and sounds guitar-like. 16 voices cost well under 1% of one core.
- Tuning accuracy matters because the same synth drives the tuner's reference tones (TU-4): plain integer delay lines are out of tune for high notes. Use a **fractional delay** (first-order allpass or linear interpolation) so the pitch error stays under 1 cent; test by running the synth output through the pitch detector (task C4 test).
- Shaping: decay time per string by frequency (higher notes decay faster), `velocity` scales the initial burst, an excitation spectrum lowpassed for a softer pick, and an optional **pick-position comb** (subtract a delayed copy of the excitation at 1/8 to 1/5 of the string) for a brighter steel character. A soft clipper on the mix keeps 16 voices below 1.0.
- Real-time safety: preallocate each voice's delay line for the lowest note (55 Hz at 48 kHz is 873 samples; allocate 1,024 at 48 kHz and scale for 96 kHz if ever needed), seed the excitation from a small xorshift PRNG held in the voice (no `rand`, no allocation). `render` and `apply` must pass `assert_no_alloc`.
- Alternative considered: `rustysynth` with a SoundFont (the architecture's long-term choice). Deferred: licence confirmation for the asset is an open item and P1b owns it.

### Q3. How does `ts-rs` work with Tauri 2, and what about command wrappers?

**Answer:** `ts-rs` generates TypeScript **types** only; it does not know about Tauri commands (D-05).

- Derive `ts_rs::TS` on the types, annotate `#[ts(export, export_to = "...")]`; `cargo test` writes the `.ts` files (an automatically generated `export_bindings_<type>` test per type) into `TS_RS_EXPORT_DIR`, so the "bindings" step is a test run, not a build script.
- Serde attributes (`rename_all`, `tag`, `content`) are honoured by `ts-rs` through its `serde-compat` feature (default on); keep the serde representation simple (externally tagged enums are fine; prefer `#[serde(tag = "kind")]` for enums with data so the TypeScript union is easy to narrow).
- `ts-rs` maps `u8..u32`, `i8..i32` to `number`, `u64/i64` to `bigint` (avoid `u64`/`i64` on the boundary; use `u32`/`f64`; the `t_ms` field in `TunerReading` is `u64` in the data dictionary, so the data dictionary uses `f64` milliseconds for `t_ms` (no `u64` on the boundary)). `Option<T>` → `T | null`. Newtype structs `Fret(u8)` → `number` by default (`#[ts(transparent)]` where needed). Tuples are supported; arrays `[Pitch; 6]` map to `Array<Pitch>` (fixed-length tuple types need `#[ts(as = "...")]`; keep `Vec` in boundary types if that is awkward).
- Tauri's `tauri-specta` 2.x is still a release candidate (this is the PRD's reason for choosing `ts-rs`). Our substitute for its generated `commands` object is a small manifest and generator (architecture, "Bindings generation"): one typed async function per command around `invoke<T>(name, args)`, args passed as an object with camelCase keys (Tauri 2 converts snake_case Rust argument names to camelCase on the JS side by default; the generator emits camelCase), errors thrown as `AppError` (Tauri rejects the promise with the serialised `Err` value).
- **VERIFY (task D6):** exact `ts-rs` major version's export attribute names and the `TS_RS_EXPORT_DIR` environment variable behaviour (`ts-rs` 9 to 11 changed export paths and the `TS::export` API); pin the version in `Cargo.toml` and note any difference here.

### Q4. Tauri 2 iOS build in CI without signing

**Answer:** uncertain; the task (E6) tries a fallback chain and records the one that works (spec Q-3).

What is known:
- `tauri ios init` generates `src-tauri/gen/apple` (an Xcode project) and requires macOS with Xcode, the Rust targets `aarch64-apple-ios`, `aarch64-apple-ios-sim` (and `x86_64-apple-ios` for Intel Macs; not needed on Apple-silicon runners), and CocoaPods only if plugins need them. `rust-toolchain.toml` already lists both iOS targets.
- `tauri ios build` is designed to produce a signed IPA and expects a development team (`APPLE_DEVELOPMENT_TEAM` or `bundle.iOS.developmentTeam`). Simulator builds do not need signing, so the build for the Simulator target (`--target aarch64-sim`) may succeed without a team; the `--debug` flag avoids release signing paths. **VERIFY** on the runner.
- GitHub's `macos-latest` image ships Xcode and iOS Simulator runtimes; `xcrun simctl list runtimes` shows which. A Simulator build does not need a booted simulator, only the SDK.

Fallback chain, in order (task E6 records which one passes):
1. `npx tauri ios init --ci` then `npx tauri ios build --debug --target aarch64-sim --ci` (the PRD's literal wording).
2. `npx tauri ios init --ci`, then `xcodebuild -project src-tauri/gen/apple/<name>.xcodeproj -scheme <name>_iOS -sdk iphonesimulator -configuration Debug -destination 'generic/platform=iOS Simulator' CODE_SIGNING_ALLOWED=NO build` (Xcode builds the Rust static library through the project's build phase; the frontend must already be built: run `npm run build` first).
3. Compile-only: `cargo build --target aarch64-apple-ios-sim -p mackey-app --lib` plus the UI build. This proves the Rust side compiles for iOS (including `cpal`'s iOS backend and our `cfg`s) but not the Xcode project; if chosen, record a Known Limitation in the ADR note and keep the job name.
- Microphone key: on iOS, `NSMicrophoneUsageDescription` must be in the generated `Info.plist`; Tauri merges `bundle.iOS.infoPlist` (path to a plist) into it. Do not edit generated files.
- Frontend assets: the build runs `beforeBuildCommand` (`npm --prefix ui run build`) itself; with fallback 2 run it before `xcodebuild`.
- **Audio session:** cpal's iOS backend does not set the `AVAudioSession` category, so the Simulator may play but record nothing without the Swift plugin (spec Q-4); the tuner then shows the denied/no-signal state rather than crashing (C6 heuristic).

### Q5. Playwright WebKit on Linux CI

**Answer:** works; the engine is a patched WebKit build (not Safari), run headless with its own bundled system libraries.

- `npx playwright install --with-deps webkit` installs the browser and the apt packages it needs (about 30 libs: GStreamer, libwoff, libharfbuzz-icu, libenchant, etc.) on Ubuntu runners. Pin the Playwright version in `package-lock.json` so the Ubuntu image's supported release matches (Playwright supports specific Ubuntu LTS versions; `ubuntu-latest` on GitHub tracks the supported one; if CI moves to an unsupported release, pin `ubuntu-24.04`).
- It is not the iOS WebKit and not WKWebView on macOS, but it is the closest cross-platform proxy; the PRD accepts this ("tauri-driver cannot drive WKWebView").
- iPad emulation: a project with `viewport: { width: 1180, height: 820 }`, `hasTouch: true`, `isMobile: false` (Playwright's `devices['iPad (gen 7) landscape']` descriptor has a different viewport; prefer an explicit config so the numbers in the spec hold); portrait 820×1180. Touch gestures: dispatch `TouchEvent`/`PointerEvent` through `page.touchscreen.tap` and `page.dispatchEvent`; swipes are simulated with pointer events in a helper (Playwright has no swipe API).
- Timing tests: `page.evaluate` with `performance.now()` and `requestAnimationFrame`; the budget of 16 ms is tight on shared CI runners, so the test measures the **app's own work** (keydown handled → DOM updated → next frame) with the mock layer, discards 5 warm-up iterations, and fails on p100 > 16 ms; if CI proves flaky, relax to p95 and record the change as a deviation (the AC says each redraw).
- Mock layer: a Vite mode (`vite --mode mock`) that aliases the command module to the mock implementation, so no Tauri runtime is needed. The recorded `scenarios.json` comes from the real Rust core (D-14).
- Accessibility: `@axe-core/playwright` works in WebKit; colour-contrast checks in light and dark themes; SVG `role="img"` elements need accessible names.
- **VERIFY (D15):** `page.keyboard.press` with `Meta+T` on WebKit/Linux (the browser may swallow it; the app listens on `keydown` and tests dispatch the event on `document`).

### Q6. Tauri 2 on Linux CI for the whole workspace

**Answer:** needs system packages and a populated `frontendDist`.

- Packages (Ubuntu 22.04/24.04): `libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `libsoup-3.0-dev`, `libjavascriptcoregtk-4.1-dev`, `librsvg2-dev`, `libayatana-appindicator3-dev` (only if the tray plugin is used; not here), `libssl-dev`, `pkg-config`, plus `libasound2-dev` for `cpal`'s ALSA backend. `patchelf` is only for AppImage bundling.
- `tauri::generate_context!` reads `tauri.conf.json` and embeds `frontendDist` at compile time and **fails the build if the directory does not exist**. Hence the stub rule (D-15): `ui/dist/index.html` is created by `make` when missing. Real builds run `npm run build` first (the Tauri CLI's `beforeBuildCommand`).
- `tauri-build` needs `icons/` referenced in the config to exist (generate once with `npx tauri icon <png>`; commit the icons).
- Unit tests of `src-tauri` do not open windows; they link GTK and WebKit but never start a display, so no `xvfb` is needed unless a test constructs a `tauri::test` mock runtime window (`tauri::test::mock_builder` uses the mock runtime and needs no display).
- `cargo deny`: Tauri brings in transitive licences such as MPL-2.0 (`cssparser`, `selectors`, `option-ext`), `Unicode-DFS-2016`, `BSD-3-Clause`; check `make deny` immediately after adding it. **VERIFY (Q-6)**; add only permissive licences to `deny.toml` and note each, escalating to J for anything non-permissive (MPL-2.0 is file-level weak copyleft; J decides if it is acceptable).

### Q7. How is microphone permission detected without Objective-C?

**Answer:** it cannot be queried directly through cpal; use heuristics and the first-use explainer (spec Q-5).

- macOS shows the system prompt the first time an input stream starts (`NSMicrophoneUsageDescription` must be present or the process is killed). cpal reports a build or play error in some denied states and in others streams zeros. Heuristic: stream-build error → `PermissionDenied`; one second of exact digital silence after the stream starts → `PermissionDenied` (a real mic has noise, even if tiny). False positives are acceptable (it also catches a muted hardware input) because the message offers the Settings button.
- A proper query (`AVCaptureDevice.authorizationStatus(for: .audio)`) needs Objective-C or Swift: that is the Swift plugin territory (spec Q-4), and on iPad `AVAudioSession.recordPermission`.
- The mic explainer screen is shown by the UI before the first `tuner_start` (E-TU-1); `settings.tuner.mic_explained` records that Continue was pressed.

### Q8. Voicing search design details to settle before coding

Seeded from architecture §4.1 and FB-5.4:

- Candidate count: how many voicings does a typical chord yield? Expect 10 to 60 before the cap; cap at 24 and sort groups by score. Benchmark with E, A, C, G, F, Bm7b5, C13, and a slash chord.
- Scoring weights (D-11): span (−3 per fret), fingers (−2 per finger), open strings within the first 4 frets (+2 each, FB-5.2 Open group), root in bass (+4), skipped inner string (−6), lowest fret above 7 (−1 per fret over 7), muted strings (−2 per mute beyond the first two). Weights live in one `const` table; the AC-FB-5.1 expectation (open C first) and AC-FB-5.2 (both F voicings present) are the calibration tests.
- Barre detection: the open-E-shape F (`133211`) needs the barre at fret 1 over six strings; a finger count of 4 results (barre + 3 fingers).
- Power chord (`5`): two or three notes, root and fifth only, so "third required" does not apply (FB-5.4 states this).
- Slash chords: bass must be the lowest sounding string; if the bass is not a chord tone it is an added note (`C/D`), allowed.
- Curated library vs search duplicates: key by `strings` array.

### Q9. UI performance of the SVG neck

- 6 strings × 24 frets = 144 hit areas plus up to 144 markers: well within SVG's comfort (ADR 0007). Re-render per snapshot: memoise rows and use stable keys (`string-fret`); avoid re-creating the whole neck for a label change (derive static geometry once per `fret_count`/layout).
- 16 ms budget for the UI part: React's reconciliation of ~150 elements is about 1 to 3 ms. The IPC round trip in the real app is under 1 ms locally plus the Rust work (view ≤ 2 ms, search cached). The Playwright budget uses the mock layer, so it measures only the UI.
- Reduce Motion: no CSS transitions on markers when `prefers-reduced-motion: reduce`.

## Industry Standards

- Scientific pitch notation (C4 = middle C = MIDI 60; A4 = 440 Hz = MIDI 69), equal temperament: `f = A4 * 2^((m - 69)/12)`; cents = `1200 * log2(f / f_target)`.
- Okabe-Ito colour-blind-safe palette (G-2 tokens). WCAG 2.2 AA contrast (4.5:1 text, 3:1 non-text).
- WAI-ARIA: `role="img"` or `role="button"` with `aria-label`, `aria-live="polite"` region for announcements, roving `tabindex` for composite widgets (the neck), `prefers-reduced-motion` media query.
- Apple Human Interface Guidelines: 44 pt minimum tap target (UX-5.1).
- MIDI note and General MIDI are not used in this run.

## Existing Implementations

- **MPM:** the original paper (McLeod and Wyvill, 2005); the open-source `pitch-detection` Rust crate (MPM, YIN) as a reference implementation. We write our own (small, no dependency, deterministic scratch buffers) and may compare outputs in tests during development only.
- **Plucked-string synthesis:** Karplus and Strong (1983); Jaffe and Smith (1983) extensions (fractional delay, loop filter, decay stretching).
- **Chord symbol parsing:** ChordPro's chord grammar, `tonal` (JS) chord detection tables, Hooktheory conventions. Only the symbol grammar is borrowed; no code.
- **Voicing search:** chord-diagram generators (for example `chord-fingering` libraries) use DFS with span pruning; ours adds curated-first and scoring.

## API Documentation

- Tauri 2: commands (`#[tauri::command]`, `generate_handler!`), `AppHandle::emit`, capabilities and permissions (`tauri-build` `AppManifest`), `tauri.conf.json` v2 schema (`app.windows`, `app.security.csp`, `build`, `bundle.macOS`, `bundle.iOS`). **VERIFY** exact keys against the pinned Tauri version.
- `cpal`: `Host::default_output_device`, `default_input_device`, `build_output_stream`, `build_input_stream`, `SupportedStreamConfig`, error enums (`BuildStreamError`, `PlayStreamError`, `StreamError::DeviceNotAvailable` for disconnect). **VERIFY** the 0.16/0.17 API (callback signatures changed with timestamps and `OutputCallbackInfo`).
- `rtrb`: `RingBuffer::new(capacity)` → `(Producer, Consumer)`; `push`, `pop`, `slots`; wait-free.
- `assert_no_alloc`: `#[global_allocator] static A: AllocDisabler = AllocDisabler;` in the test binary; `assert_no_alloc(|| { ... })`; only active with the `debug_assertions` build.
- `ts-rs`, `proptest`, `insta`, `criterion`: standard; see AGENTS.md TDD rules.
- Vite, Vitest, Testing Library, Playwright, Zustand: standard documentation; pin exact versions at implementation.

## Best Practices

- Keep functions pure and data tables `const`; hand-write expected tables, never generate them from the code under test.
- Prefer returning `Option`/`Result` over panicking; `unreachable!` only with a proof comment.
- Time in the tuner is counted in samples, never read from a clock, so tests are deterministic (D-13).
- Never lock, allocate or log in an audio callback; communicate over wait-free queues.
- The UI never parses or interprets theory strings; it displays strings the core provides.
- Generated files carry a header and are rewritten byte-identically (stable ordering, trailing newline) so `git diff --exit-code` is a reliable staleness check.

## Open Questions

Tracked with owners in [spec.md](spec.md#open-questions) (Q-1 to Q-8). Research-level items still to verify, each in the named task:

| Item | Owner | Task |
|------|-------|------|
| `ts-rs` version, attributes and `u64` handling | Build | D6 |
| `cpal` current API and iOS behaviour | Build | C5, C6 |
| Tauri 2 config keys, `AppManifest`, mock runtime | Build | D1 |
| Unsigned iOS Simulator build command | Build | E6 |
| Licences brought in by Tauri's dependency tree | Build (escalate to J if non-permissive) | D1 |
| `Meta+T` handling in Playwright WebKit | Build | D15 |

## References

- McLeod, P. and Wyvill, G., "A Smarter Way to Find Pitch", ICMC 2005.
- de Cheveigné, A. and Kawahara, H., "YIN, a fundamental frequency estimator for speech and music", JASA 2002.
- Karplus, K. and Strong, A., "Digital synthesis of plucked-string and drum timbres", Computer Music Journal 1983; Jaffe, D. and Smith, J., "Extensions of the Karplus-Strong plucked-string algorithm", CMJ 1983.
- Okabe, M. and Ito, K., "Color Universal Design" palette; WCAG 2.2.
- Tauri 2 docs (tauri.app), `ts-rs` README, Playwright docs (browsers, touch, accessibility testing), `cpal`/`rtrb` docs.
- Repository: [ADR 0006](../../../docs/adr/0006-on-device-audio-analysis.md), [ADR 0007](../../../docs/adr/0007-web-ui-and-ipc-boundary.md), [ADR 0008](../../../docs/adr/0008-ipad-via-tauri-ios.md), [docs/platforms.md](../../../docs/platforms.md), [docs/learning-features.md](../../../docs/learning-features.md).
