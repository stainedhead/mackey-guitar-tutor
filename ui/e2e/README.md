# ui/e2e

Playwright end-to-end specs for the UI. They run in **WebKit** (the closest cross-platform proxy for WKWebView) in two projects, `desktop` (1280×800) and `ipad` (1180×820, touch), against `vite --mode mock`, so the command layer is the mock in [`../src/ipc/mock.ts`](../src/ipc/mock.ts) and no Tauri runtime is needed.

| File               | Purpose                                                                                            |
| ------------------ | -------------------------------------------------------------------------------------------------- |
| `skeleton.spec.ts` | Walking skeleton: sidebar, six-string neck placeholder, mocked `app_info`, Explore/Tuner switching |

Run with `make e2e`. One-time setup: `npx --prefix ui playwright install webkit` (add `--with-deps` on Linux). Reports go to `ui/playwright-report/` and `ui/test-results/`, which are git-ignored and skipped by `make doccheck`.

Part of [`ui/`](../README.md); back to the [root README](../../README.md).
