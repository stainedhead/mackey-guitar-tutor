# ui/e2e

Playwright end-to-end specs for the UI. They run in **WebKit** (the closest cross-platform proxy for WKWebView) against a production build of `vite --mode mock` (`dist-mock/`, served by `vite preview`; see D-64), so the command layer is the mock in [`../src/mocks/`](../src/mocks) replaying states recorded from the real Rust core; no Tauri runtime is needed. Projects: `desktop` (1280×800), `ipad` (1180×820, touch), `ipad-portrait` (820×1180, touch) and `narrow` (420×800, Split View width).

| File                  | Purpose                                                                                                                                                                                                          |
| --------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `explore.spec.ts`     | AC-P0-5 (six strings, 22 frets), AC-LH-1.1 (left-handed mirror), AC-FB-9.1b (100 cycles, redraw time), AC-FB-9.1c (live region), AC-FB-1.6 (tooltip), chord flow, E-FB-1, shortcuts ignored while typing, E-FB-8 |
| `tuner.spec.ts`       | Tuner states from mocked `tuner_reading` events: explainer, flat, in tune, too quiet, no stable pitch, string targets, unsubscribe on leaving                                                                    |
| `layouts.spec.ts`     | iPad landscape and portrait, narrow, no horizontal page scroll, Reduce Motion                                                                                                                                    |
| `screenshots.spec.ts` | Not a test: with `SCREENSHOTS=1` it regenerates the images in [`../../docs/images/`](../../docs/images/README.md)                                                                                                |

Run with `make e2e`. One-time setup: `npx --prefix ui playwright install --with-deps webkit`. Where WebKit cannot be downloaded, `PW_CHANNEL=chrome make e2e` runs the same specs in an installed Chrome. Screenshots: `cd ui && SCREENSHOTS=1 PW_CHANNEL=chrome npx playwright test screenshots`. Reports go to `ui/playwright-report/` and `ui/test-results/` (git-ignored, skipped by `make doccheck`).

The mock only knows recorded paths (see `src-tauri/src/bindings.rs`); a spec that steps outside them gets an error notice. Part of [`ui/`](../README.md); back to the [root README](../../README.md).
