# docs/images

Screenshots for the root README, the design docs and the user docs. There are two kinds, told apart by the file name prefix:

- **`real-*.png`: the real app.** The built `mackey-app` binary (Rust core, WebKit webview) running under Xvfb on Linux, with openbox as the window manager, driven by `xdotool` keys and clicks. Reproduce with `make screenshots`; the scripts are in [../../scripts/screenshots/](../../scripts/screenshots/README.md). Each shot starts the app with empty settings. Because it is a Linux build, fonts and the window have no macOS chrome and scrollbars follow the GTK theme.
- **`mock-*.png`: the mocked UI.** Playwright drives the UI in a browser against the mocked command layer, which replays states recorded from the real Rust core (so the voicings are real, but nothing runs). Used only where a real shot is not possible: iPad portrait and narrow widths, and the tuner with simulated readings. Regenerate with `cd ui && SCREENSHOTS=1 PW_CHANNEL=chrome npx playwright test screenshots` (drop `PW_CHANNEL` to use WebKit).

All images were made on 2026-11-15.

| File | Kind | Shows |
| --- | --- | --- |
| `real-explore-g-major.png` | real, 1280×800 | Explore in G major, all tones; the `▶ 23` button above the neck counts notes beyond fret 15 |
| `real-explore-c-voicing.png` | real, 1280×800 | IV chord (C), open voicing, with the voicing list |
| `real-explore-c-triad.png` | real, 1280×800 | C major triad, strings 2 to 4, first inversion |
| `real-explore-am-arpeggio.png` | real, 1280×800 | Am arpeggio, position 5 |
| `real-explore-a-minor-pentatonic.png` | real, 1280×800 | A minor, scale overlay on, A minor pentatonic, box 1 of 5 |
| `real-explore-left-handed.png` | real, 1280×800 | G major with Left-handed on |
| `real-explore-drop-d.png` | real, 1280×800 | Drop D tuning, open D chord (voicing view) |
| `real-explore-ipad.png` | real, 1180×820 | The IV chord at the iPad landscape window size (desktop app in a window of that size, not the iPad app) |
| `real-settings.png` | real, 1280×1180 | The whole Settings screen |
| `real-tuner.png` | real, 1280×800 | The tuner on a machine with no microphone: the real tuner listening to silence, "Too quiet" |
| `mock-explore-g-major-ipad-portrait.png`, `mock-explore-c-voicings-ipad-portrait.png` | mock, 820×1180 | Explore on iPad in portrait (tab bar) |
| `mock-explore-g-major-narrow.png` | mock, 420×800 | Explore at Split View width |
| `mock-tuner-flat-desktop.png`, `mock-tuner-in-tune-desktop.png` | mock | The tuner in string mode with a **simulated** flat A and an in-tune A; the readings are scripted, not measured |

Every real shot is checked by eye after a run: the right notes lit, nothing covering a note. Back to [docs](../README.md).
