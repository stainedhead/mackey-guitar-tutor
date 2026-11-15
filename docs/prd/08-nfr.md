# 08. Non-Functional Requirements, Data, Settings and Errors

Applies to all phases. These replace and expand NF-1 to NF-11 from the earlier product spec (IDs kept).

## 1. Performance budgets

Measured on the reference devices: a MacBook Air M1 (8 GB) and the iPad Simulator on that Mac for CI; on a real iPad (A14 or newer) once device validation happens.

| ID | Measure | Budget | How it is checked |
|----|---------|--------|-------------------|
| NF-1 | Neck redraw after any cycling action | ≤ 16 ms per frame (p95), no dropped frames in a 100-action burst | Playwright performance trace + Rust benchmark for `fretboard_view` (≤ 2 ms) |
| NF-1.1 | Voicing search for one chord, standard tuning, full neck | ≤ 5 ms (p95) | criterion benchmark |
| NF-1.2 | Voice-leading solver for a 200-chord song, k = 10 | ≤ 20 ms | criterion benchmark |
| NF-2 | Notation parse and preview per keystroke | ≤ 50 ms end to end; parser alone ≤ 1 ms for 1,000 characters | benchmark + Playwright |
| NF-2.1 | Song import parse (2,000 lines) | ≤ 100 ms | benchmark |
| NF-3 | Cold start to interactive neck | Mac ≤ 2 s; iPad ≤ 3 s | manual timed check per release |
| NF-3.1 | Selection playback latency (action to sound) | ≤ 30 ms plus device output latency | instrumented test |
| NF-8 | Transport timing | Click and drum onsets exact to the sample in offline render; drift 0 over 10 minutes; tempo changes on bar lines | offline render tests (AC-RH-1.1) |
| NF-8.1 | Audio callback | No allocation, lock or syscall in the callback; buffer 128 to 256 frames at 48 kHz by default (≤ 5.3 ms) | `assert_no_alloc` in debug builds; code review rule |
| NF-8.2 | Visual sync | Visual events within ± 5 ms of audio events after latency compensation | instrumented test |
| NF-9 | Tuner | ±3 cents accuracy; ≥ 20 readings/s; first reading ≤ 150 ms after a note starts | fixture tests (AC-TU-1.x) |
| NF-12 | Memory | ≤ 400 MB resident on Mac with a song open and drums playing; ≤ 300 MB on iPad | manual check per release |
| NF-13 | App size | ≤ 60 MB download (sound assets ≤ 40 MB) | CI check on the build artifact |
| NF-14 | Library scale | 1,000 songs and 1,000 sessions: list and search ≤ 100 ms | benchmark with generated data |

## 2. Reliability and data safety

- NF-15 No data loss: drafts auto-save every 2 s and on close/backgrounding (G-6.3); SQLite runs in WAL mode; writes are transactional.
- NF-16 Crash safety: on next launch after a crash, the app restores the last area and any draft, and shows "Mackey closed unexpectedly. Your work was restored." No crash reports are sent anywhere (NF-7); a local log is kept for the player to share manually.
- NF-17 Corrupt or unreadable database: the app opens a fresh database, keeps the old file renamed with a date, and offers to rebuild the library from the last export folder if one exists. Because sessions and songs are text, "Export library" is the backup; the app SHOULD remind the player to export once a month (dismissible).
- NF-18 Schema migrations run at start-up, are tested from every previous released version, and take a backup copy before running.

## 3. Accessibility

- NF-5 VoiceOver (Mac and iPad): every fretboard position is an accessible element with label "String 3, fret 2, A, third of F" (role and chord context included); the neck is navigable with VoiceOver arrows by string and fret; changes of selection, transport state, run item and tuner state are announced (tuner announcements throttled to once per second).
- NF-5.1 Keyboard: every action is reachable from the keyboard (07 §4); visible focus ring on every focusable control; no keyboard traps.
- NF-5.2 Colour: palette per G-2 tested for protanopia, deuteranopia and tritanopia; all text meets WCAG 2.2 AA contrast (4.5:1, large text 3:1) in light and dark themes; note dots meet 3:1 against the neck.
- NF-5.3 Text size: UI scales with the system text size (Mac: Settings > Accessibility > Display text size; iPad: Dynamic Type up to AX3) without truncating controls; the neck's labels scale with a minimum of 11 pt.
- NF-5.4 Motion: with Reduce Motion, no pulsing, sliding or animated transitions; replaced by instant changes or cross-fades.
- NF-5.5 Audio cues always have a visual equivalent (count-in numbers, pulse, "Target reached" banner).
- NF-10 Touch: NF-5.1 applies with a hardware keyboard; without one, every function is reachable by touch (UX-5.2).

## 4. Privacy and security

- NF-7 No telemetry, analytics, accounts or network calls in v1. P5 adds web search and one-page fetches, only on an explicit action by the player.
- NF-7.1 Microphone audio is processed in memory and never written to disk, except looper takes the player records (P7).
- NF-7.2 The app requests only the microphone permission, and only when a listening feature first opens.
- NF-7.3 The web view loads only bundled assets (strict Content Security Policy: no remote scripts, no remote styles; `connect-src` limited to the Tauri IPC). Tauri capabilities allow only the commands the UI needs.
- NF-7.4 Imported text is treated as data: it is never executed or rendered as HTML (chart text is escaped).
- NF-11 Bundled sounds are CC0, public domain or MIT, with credits in `assets/CREDITS.md` ([sound-assets.md](../sound-assets.md)).

## 5. Compatibility

- NF-19 macOS 13 Ventura and later, Apple silicon and Intel (universal). iPadOS 17 and later on iPads supported by that version.
- NF-20 Audio: any device CoreAudio exposes on Mac or iPad; sample rates 44.1 and 48 kHz (others resampled); mono or stereo input (input channel selectable).
- NF-21 Files: UTF-8 (with or without BOM), UTF-16, Windows-1252 text; line endings LF, CRLF and CR.

## 6. Settings (complete list for v1)

| Group | Setting | Values (default) |
|-------|---------|------------------|
| Instrument | Handedness | Right (default), Left |
| Instrument | Flip strings | Off |
| Instrument | Frets | 19 to 24 (22) |
| Instrument | Default tuning | Presets and custom (Standard) |
| Instrument | Allow stretches | Off |
| Display | Theme | System, Light, Dark |
| Display | Labels default | Note name |
| Display | Show fret numbers | On |
| Display | Show tips | On |
| Sound | Selection playback instrument | Nylon, Steel (default), Electric clean |
| Sound | Selection playback volume | 0 to 100% (80%) |
| Sound | Mute selection playback | Off |
| Rhythm | Click sound, volumes, count-in, humanise | Per RH-4, RH-5, DR-6.1 |
| Rhythm | Latency offset | −50 to +300 ms (0) |
| Rhythm | Audio buffer size | 128, 256 (default), 512, 1024 frames |
| Audio | Output device / input device | System default |
| Tuner | Reference A4 | 415 to 466 Hz (440) |
| Practice | Default method | Loop |
| Practice | Default tempo | 70 bpm |
| Songs | Default voicing preset | Beginner |
| Songs | Read H as B | Off |
| Data | Export library / Import library / Recently deleted | Actions |

All settings are stored locally, apply immediately, and can be reset per group.

## 7. Global error handling

- NF-22 Every error message says what happened and what to do next, in plain words, without codes in the main text (a "Details" disclosure may show the technical message).
- NF-23 Errors never discard the player's input. Parse errors are inline; I/O errors are banners with a retry action.
- NF-24 The Rust core never panics across the IPC boundary: every command returns a typed result; a panic in a command is caught, logged locally and shown as "Something went wrong" with Details.
