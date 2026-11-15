# .github

Repository automation and agent instructions.

| Path | Purpose |
|---|---|
| `workflows/ci.yml` | On every PR and push to `main`, on macOS and Linux: `make fmt-check`, `make lint` (clippy `-D warnings`), `make archcheck`, `make doccheck`, `make test`, `make cover` (≥ 80% lines); on Linux also `make deny` and `make props` (10 000 proptest cases). The toolchain comes from `rust-toolchain.toml` |
| `copilot-instructions.md` | Points Copilot at [../AGENTS.md](../AGENTS.md) |

Not yet in place (roadmap phase P0, with the Tauri scaffold): the UI jobs (lint, type check, Vitest, Playwright), the stale-bindings check, `tauri build` for a universal Mac binary, `tauri ios build` for the iOS Simulator, and `release.yml` for ad-hoc signed `.dmg` releases on `v*` tags. CI rules are in [../AGENTS.md](../AGENTS.md#cicd). Back to the [root README](../README.md).
