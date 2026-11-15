# docs/images

Screenshots of the UI for the root README, the design docs and the user docs. They are generated, not drawn: `ui/e2e/screenshots.spec.ts` drives the UI against the mock command layer (real voicings recorded from the Rust core) and writes the files here. Regenerate with `cd ui && SCREENSHOTS=1 PW_CHANNEL=chrome npx playwright test screenshots` (drop `PW_CHANNEL` to use WebKit).

| File | Shows |
| --- | --- |
| `explore-g-major-desktop.png`, `-ipad.png`, `-ipad-portrait.png`, `-narrow.png` | Explore in G major, by layout |
| `explore-c-voicings-desktop.png`, `-ipad.png`, `-ipad-portrait.png` | C chord, voicing view with the voicing list |
| `explore-left-handed-desktop.png` | The mirrored neck (left-handed) |
| `tuner-flat-desktop.png`, `tuner-in-tune-desktop.png` | Tuner in string mode with a flat A and an in-tune A |
| `settings-desktop.png` | Settings |

These come from the mocked UI in a browser, so they show no window frame. Back to [docs](../README.md).
