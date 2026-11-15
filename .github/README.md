# .github

Repository automation and agent instructions.

| Path | Purpose |
|---|---|
| `workflows/ci.yml` | On every PR and push to `main`; jobs below. The toolchain comes from `rust-toolchain.toml` |
| `copilot-instructions.md` | Points Copilot at [../AGENTS.md](../AGENTS.md) |

## CI jobs (`workflows/ci.yml`)

| Job | Runner | What it does |
|---|---|---|
| `rust` | macOS and Ubuntu | `make fmt-check`, `make lint` (clippy `-D warnings`), `make archcheck`, `make doccheck`, `make test`, `make cover` (≥ 80% lines); on Linux also `make deny` and `make props` (10 000 proptest cases). Ubuntu first installs `libwebkit2gtk-4.1-dev libasound2-dev libgtk-3-dev libsoup-3.0-dev librsvg2-dev libayatana-appindicator3-dev` (the Tauri crates and cpal link them). The Makefile writes a stub `ui/dist/index.html`, so no Node is needed here |
| `ui` | Ubuntu | Node 22, `make ui-lint` (ESLint, Prettier, `tsc`), `make ui-test` (Vitest) |
| `e2e` | Ubuntu | `playwright install --with-deps webkit`, then `make e2e` (projects `desktop` 1280×800 and `ipad` 1180×820 against the mocked command layer); uploads the report and traces on failure |
| `bindings` | Ubuntu | `make bindings-check`: regenerates `ui/src/bindings/` from the Rust types and fails on any difference |
| `mac-app` | macOS | `make app`: universal (arm64 + x86_64) `.app` and `.dmg`, ad-hoc signed, no Apple secrets; fails unless `lipo -archs` shows both architectures; uploads the `.dmg` |
| `ios-sim` | macOS | `make ios-sim`: `tauri ios init --ci`, then an unsigned debug build for `aarch64-sim`. If that fails it falls back to compiling the Rust library for `aarch64-apple-ios-sim` and emits a warning annotation |

### Assumptions about the macOS jobs (not verifiable from Linux)

- `macos-latest` is Apple silicon with Xcode and iOS Simulator runtimes installed; the `ios-sim` job prints them. XcodeGen is installed with Homebrew because `tauri ios init` needs it.
- The Tauri CLI (invoked from the repo root as `ui/node_modules/.bin/tauri`) runs the `beforeBuildCommand` (`npm run build`) inside `ui/`, as the Linux smoke build of `make app` showed.
- `tauri ios build --debug --target aarch64-sim --ci` can run without a development team for a Simulator target. If it cannot, the fallback keeps the Rust side covered; record the working command in [ADR 0008](../docs/adr/0008-ipad-via-tauri-ios.md) once known.
- Action major versions (`setup-node@v6`, `upload-artifact@v5`) were chosen without registry access; bump them if a job reports a missing tag.

Not yet in place: `release.yml` for ad-hoc signed `.dmg` releases on `v*` tags. CI rules are in [../AGENTS.md](../AGENTS.md#cicd). Back to the [root README](../README.md).
