# src-tauri/capabilities

Tauri 2 capability files: which permissions each window gets. `default.json` grants the `main` window only `core:default` (Tauri's own event and window basics). The app's own commands (`app_info`, ...) are registered in code with `generate_handler!` and need no plugin permission. No filesystem, shell, network or other plugin permission is granted; add one here only with the feature that needs it, and keep this list minimal.

Part of [`src-tauri`](../README.md).
