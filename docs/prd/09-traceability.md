# 09. Traceability

Every requirement maps to where it is designed and how it is verified. When code exists, each test name includes the requirement ID (for example `fb_5_4_no_voicing_exceeds_span`), and CI produces a report listing requirements with no passing test.

Test types: **U** unit, **P** property (proptest), **G** golden file (insta), **B** benchmark (criterion), **R** offline audio render, **UI** Playwright (Mac and iPad viewports), **M** manual check (Mac, iPad Simulator, device once available).

## Requirements to design and tests

| Requirement IDs | PRD | Design | Crate / layer | Tests | Phase |
|-----------------|-----|--------|---------------|-------|-------|
| G-1 note spelling | [00 §5](README.md#g-1-note-spelling) | [architecture §3.1](../architecture.md#31-music-theory) | mackey-theory | U, P (every key × every chord quality), G | P0 |
| G-2 visual language | [00 §5](README.md#g-2-fretboard-visual-language) | [architecture §5](../architecture.md#5-ui-composition-and-the-core-boundary), [platforms §6](../platforms.md#6-accessibility-confirmed) | ui, mackey-voicing (`FretboardView`) | UI (screenshot), M (colour-blind simulation) | P1 |
| G-3 selection model | [00 §5](README.md#g-3-selection-model) | [architecture §5](../architecture.md#5-ui-composition-and-the-core-boundary) | src-tauri state, ui | U, UI | P1 |
| G-4, G-5 defaults and limits | [00 §5](README.md#g-4-defaults-first-launch) | [architecture §3](../architecture.md#3-domain-model) | all | U | P0 to P4 |
| G-6 undo, recently deleted, drafts | [00 §5](README.md#g-6-undo-and-safety) | [ADR 0003](../adr/0003-text-first-persistence.md) | mackey-store, ui | U, UI | P2, P3 |
| FB-1 neck rendering | [01](01-fretboard.md#fb-1-neck-rendering) | [ADR 0007](../adr/0007-web-ui-and-ipc-boundary.md) | ui | UI, B (NF-1) | P1 |
| FB-2, FB-3 key and notes | [01](01-fretboard.md#fb-2-key-and-mode) | [architecture §3.1](../architecture.md#31-music-theory) | mackey-theory | U, P | P1 |
| FB-4 chord selection and symbols | [01](01-fretboard.md#fb-4-chord-selection) | [song-import §4.3](../song-import.md#43-chords-over-lyrics) | mackey-theory | U, P (fuzz), G | P0, P1 |
| FB-5 voicings | [01](01-fretboard.md#fb-5-voicings) | [architecture §4.1](../architecture.md#41-voicing-search-mackey-voicing) | mackey-voicing | U (curated shapes), P (FB-5.4 invariants), B | P1 |
| FB-6 triads, FB-7 arpeggios | [01](01-fretboard.md#fb-6-triads) | [architecture §4.1](../architecture.md#41-voicing-search-mackey-voicing) | mackey-voicing | U, P, G | P1 |
| FB-8 sound | [01](01-fretboard.md#fb-8-sound) | [sound-assets](../sound-assets.md) | mackey-audio | R, M | P1 |
| FB-9 cycling | [01](01-fretboard.md#fb-9-cycling) | [07 §4](07-ux-flows.md#4-keyboard-shortcuts-mac-and-ipad-with-a-keyboard) | ui, src-tauri | UI, B | P1 |
| SC-1 to SC-5 scales | [01](01-fretboard.md#sc-scales-and-modes-overlay) | [learning-features §2](../learning-features.md#2-scales-and-modes-overlay-v1) | mackey-theory, mackey-voicing | U, G (pentatonic boxes) | P1 |
| LH-1 to LH-5 handedness, tunings, capo | [01](01-fretboard.md#lh-handedness-tunings-and-capo) | [learning-features §3](../learning-features.md#3-left-handed-mode-and-alternate-tunings-v1) | mackey-voicing, ui | U, P (every tuning), UI | P1 |
| TU-1 to TU-7 tuner | [02](02-tuner.md) | [learning-features §1](../learning-features.md#1-built-in-tuner-v1), [ADR 0006](../adr/0006-on-device-audio-analysis.md), [platforms §3](../platforms.md#3-ipad) | mackey-audio, audio-session plugin | U (synthetic tones), G (recorded fixtures), M | P1 |
| TR-1 to TR-7 transport | [03 §3](03-rhythm.md#3-transport) | [rhythm §5.1](../rhythm.md#51-one-clock-many-listeners), [architecture §6](../architecture.md#6-real-time-audio-design) | mackey-rhythm, mackey-audio | U, R | P1b |
| RH-1 to RH-8 metronome and latency | [03 §4, §7](03-rhythm.md#4-metronome) | [rhythm §2, §5](../rhythm.md) | mackey-rhythm, mackey-audio | U, R, M (calibration) | P1b |
| DR-1 to DR-6 drums | [03 §5](03-rhythm.md#5-drum-backing) | [rhythm §3, §5.2](../rhythm.md), [ADR 0005](../adr/0005-sequenced-drums.md), [sound-assets](../sound-assets.md) | mackey-rhythm, mackey-audio | U (patterns), R | P1b |
| ST-1 to ST-5 speed trainer | [03 §6](03-rhythm.md#6-speed-trainer) | [rhythm §4](../rhythm.md#4-speed-trainer-start-slow-build-up) | mackey-rhythm | U, R | P1b |
| PS-0, PS-1 to PS-4 notation and methods | [04 §3, §4](04-practice.md#4-quick-entry-notation-behaviour) | [practice-sessions §2, §3](../practice-sessions.md) | mackey-practice | U, G (parse results and diagnostics), P (fuzz: never panics) | P2 |
| PS-5 editor | [04](04-practice.md#ps-5-editor) | [practice-sessions §2.3](../practice-sessions.md#23-editor-experience) | ui, mackey-practice | UI, B (NF-2) | P2 |
| PS-6 run mode | [04](04-practice.md#ps-6-run-mode) | [practice-sessions §4](../practice-sessions.md#4-run-mode) | mackey-practice, mackey-rhythm, ui | U, R, UI | P2 |
| PS-7 history | [04](04-practice.md#ps-7-history) | [architecture §3.5](../architecture.md#35-persistence) | mackey-store | U | P2 |
| PS-8 sessions from elsewhere | [04](04-practice.md#ps-8-sessions-from-elsewhere) | [practice-sessions §5](../practice-sessions.md#5-from-songs-and-the-explorer) | mackey-practice | U, P (generated notation parses) | P2, P4 |
| SG-0, SG-1, SG-2 import and ChordPro | [05 §3, §4](05-songs.md#4-import-v1) | [song-import §4](../song-import.md#4-parsing) | mackey-songs | G (fixture corpus), P (round trip), U | P3 |
| SG-10, SG-11 review and editor | [05](05-songs.md#sg-10-import-review-and-editing) | [song-import §3](../song-import.md#3-import-channels) | ui, mackey-songs | UI | P3 |
| SG-5 chart | [05](05-songs.md#sg-5-display) | [song-import §5](../song-import.md#5-chart-view) | ui | UI (screenshot) | P3 |
| SG-6 voicing sets | [05](05-songs.md#sg-6-fretboard-help-and-voicing-sets) | [architecture §4.2](../architecture.md#42-voice-leading-path-songs-and-practice) | mackey-voicing | U, P, B | P3 |
| SG-7 transpose, capo, key | [05](05-songs.md#sg-7-transpose-capo-and-key) | [architecture §4.4](../architecture.md#44-key-detection) | mackey-theory, mackey-songs | U, P (transpose round trip) | P3 |
| SG-8 play-along | [05](05-songs.md#sg-8-play-along) | [rhythm §5.1](../rhythm.md#51-one-clock-many-listeners) | mackey-rhythm, ui | R, UI | P3 |
| SG-9 practice this song | [05](05-songs.md#sg-9-practice-this-song) | [practice-sessions §5](../practice-sessions.md#5-from-songs-and-the-explorer) | mackey-songs, mackey-practice | U, P | P4 |
| SG-3, SG-4 web import and search | [05 §6](05-songs.md#6-web-import-and-search-p5-gated) | [song-import §2, §3](../song-import.md), [ADR 0004](../adr/0004-user-initiated-song-import.md) | mackey-sources | U, M; legal review gate | P5 |
| CG, TT, PR | [06](06-later-features.md) | [learning-features §4 to §6](../learning-features.md) | mackey-voicing, mackey-theory, mackey-progress | U, G | P6 |
| ET, LP | [06](06-later-features.md) | [learning-features §7, §8](../learning-features.md) | mackey-audio, mackey-practice | U, R, M | P7 |
| MR | [06](06-later-features.md) | [learning-features §9](../learning-features.md#9-chord-and-note-recognition-by-microphone-p8), [ADR 0006](../adr/0006-on-device-audio-analysis.md) | mackey-audio | G (recorded test set), M | P8 |
| UX-1 to UX-10 | [07](07-ux-flows.md) | [platforms §3](../platforms.md#3-ipad) | ui | UI, M | P1 to P4 |
| NF-1 to NF-24 | [08](08-nfr.md) | [architecture §6, §7](../architecture.md#6-real-time-audio-design), [platforms](../platforms.md) | all | B, R, UI, M | All |

## Earlier IDs

The earlier product spec's IDs are all kept: FB-1 to FB-9, PS-1 to PS-8, SG-1 to SG-10, RH-1 to RH-7, DR-1 to DR-7, ST-1 to ST-5, TU, SC, LH, CG, TT, PR, ET, LP, MR and NF-1 to NF-11. New IDs: G-1 to G-6, TR-1 to TR-7, RH-8, SG-0, SG-11, UX-1 to UX-10, NF-12 to NF-24.

## Open items

| Item | Owner | Needed by |
|------|-------|-----------|
| Teacher-to-student sharing: files only, or a service? | J | Before any sharing work (not planned for v1) |
| Legal review of web import | J | Start of P5 |
| Confirm each sound asset's licence at source | Build | P1b |
