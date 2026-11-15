# scripts/screenshots

Captures screenshots of the **real** Mackey app (the Rust core behind a WebKit webview, not the mocked UI) for the root README, `docs/` and `user-docs/`. Linux only and optional: it is not part of `make check` or CI.

Run `make screenshots`. It builds the UI and the app (`cargo build -p mackey-app --features tauri/custom-protocol`), starts a virtual display (Xvfb) with the bare window manager openbox, starts the app in a fresh settings directory per shot, drives it with `xdotool` and writes `docs/images/real-*.png`. Pass shot names to `capture.sh` to redo only some, for example `scripts/screenshots/capture.sh real-tuner`.

Install once: `apt install xvfb xdotool imagemagick x11-utils openbox`.

| File | Purpose |
|---|---|
| `capture.sh` | The list of shots: each one launches the app, sends keys and clicks, then captures. Click coordinates assume the 1280×800 Explore layout |
| `lib.sh` | Helpers: start Xvfb and openbox, launch the app, `click`, `keys`, `pick` (choose an entry in a drop-down by typing), `resize`, `shot` |
| `openbox-rc.xml` | openbox config: no window frames, windows placed at 0,0 |

Notes:

- Software rendering is used (`WEBKIT_DISABLE_COMPOSITING_MODE=1`, `LIBGL_ALWAYS_SOFTWARE=1`). Without a window manager a window resize never reaches the webview, which is why openbox is used.
- There is no microphone, so the Tuner shot shows what the real tuner does with the silent null input: "Too quiet". Its capture runs last because that input delivers samples flat out and keeps the CPU busy.
- Check every image after a run (right notes, nothing overlapping); a UI change can move a control and break the click coordinates.

Part of the repo tooling; it depends on the built `mackey-app` and nothing depends on it. See [docs/images](../../docs/images/README.md) and the [root README](../../README.md); back to the [repo root](../../README.md).
