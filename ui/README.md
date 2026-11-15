# ui

The web UI of Mackey Guitar Tutor: **React 18 + TypeScript (strict) + Vite**, with the fretboard drawn in SVG. It runs inside the Tauri web view (WKWebView on macOS and iPad). It holds **view state only**: everything a test should check is computed in Rust and arrives as data ([ADR 0007](../docs/adr/0007-web-ui-and-ipc-boundary.md)). Never re-implement theory in TypeScript.

Screens: **Explore** (key and chord pickers, chord strip, chord field with inline errors, voicing list with mini diagrams, triad and arpeggio controls, the shared SVG fretboard), **Tuner** and **Settings**. See the screenshots in [docs/images](../docs/images/README.md).

| Path                                                          | Purpose                                                                                                                                                                                                             |
| ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src/main.tsx`, `src/App.tsx`, `src/App.css`                  | Entry, store provider and the shell; all styling (layouts, neck, markers, tuner)                                                                                                                                    |
| `src/theme/tokens.css`                                        | Colour tokens in light and dark: Okabe-Ito role colours with their text colours; contrast is unit-tested                                                                                                            |
| `src/ipc/`                                                    | The only way the UI calls Rust: `tauri.ts` (real `invoke`; rejections become `AppError`), `queue.ts` (commands reach Rust in order), `index.ts` (picks the mock in `vite --mode mock` and wherever Tauri is absent) |
| `src/mocks/`                                                  | `mockCommands.ts` replays `scenarios.json`, states recorded from the real Rust core; `mockTuner.ts` pushes `TunerReading`s (`window.__mackeyMock` in Playwright)                                                    |
| `src/state/`                                                  | `UiStore`: view state and actions over `useSyncExternalStore`; drops snapshots older than the held revision                                                                                                         |
| `src/fretboard/`                                              | `Fretboard` (SVG drawn only from `FretboardView`), geometry, note markers, tooltip, range controls, edge-arrow bar above the neck                                                                                   |
| `src/explore/`, `src/tuner/`, `src/settings/`                 | The screens                                                                                                                                                                                                         |
| `src/layout/`                                                 | Layout detection (`mac`, `ipad-landscape`, `ipad-portrait`, `narrow`), sidebar and shell                                                                                                                            |
| `src/shortcuts.ts`, `useShortcuts.ts`, `gestures.ts`, `a11y/` | Keyboard table (no two commands share a key), swipe and pinch, live region                                                                                                                                          |
| `src/bindings/`                                               | TypeScript generated from Rust (`make bindings`). Never hand-edit; CI fails when stale. See its [README](src/bindings/README.md)                                                                                    |
| `e2e/`                                                        | Playwright specs (projects `desktop`, `ipad`, `ipad-portrait`, `narrow`); see [e2e/README.md](e2e/README.md)                                                                                                        |
| `vite.config.ts`, `playwright.config.ts`                      | Dev server on port 1420, `safari16` build target, Vitest; Playwright projects and the mock web server                                                                                                               |
| `eslint.config.js`, `.prettierrc`, `tsconfig.json`            | Typed strict ESLint (typescript-eslint, react-hooks, jsx-a11y), Prettier, `strict` + `noUncheckedIndexedAccess`                                                                                                     |

## Commands

From the repo root: `make ui-lint`, `make ui-test`, `make e2e`, `make bindings`. Or here: `npm ci`, `npm run dev:mock` (browser with the mock layer), `npm run lint`, `npm run typecheck`, `npm test`, `npm run e2e`.

## Architecture fit

Depends on nothing in the workspace except the generated bindings of [`src-tauri`](../src-tauri/README.md), which embeds `dist/` (`frontendDist`) and serves `npm run dev` during `tauri dev`. Conventions (ARIA labels on every interactive element, shape plus colour cues, call Rust only through `src/ipc/`) are in [AGENTS.md](../AGENTS.md#typescript-and-ui-conventions-ui). Back to the [root README](../README.md).
