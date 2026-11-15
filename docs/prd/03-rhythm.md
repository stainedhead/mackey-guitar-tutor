# 03. Rhythm: Metronome, Drum Backing and Speed Trainer

Phase: P1b (v1). Design: [rhythm.md](../rhythm.md), [architecture.md §6](../architecture.md#6-real-time-audio-design), [ADR 0005](../adr/0005-sequenced-drums.md), [sound-assets.md](../sound-assets.md).

## 1. Purpose

Help the player practise in cadence, starting slow and stepping up to the right tempo, with a click or a drummer that never drifts from the fretboard and chart.

## 2. User stories

| ID | Story |
|----|-------|
| US-RH-1 | As a player, I start a click at 60 bpm in 4/4 and hear beat 1 accented. |
| US-RH-2 | As a player, I tap the tempo of a song I hear to set the metronome. |
| US-RH-3 | As a player, I practise over a rock beat instead of a click, because it feels like playing a song. |
| US-RH-4 | As a learner, I set 60 to 96 bpm in steps of 4, and the tempo goes up only when I say my last pass was clean. |
| US-RH-5 | As a player with Bluetooth headphones, I set a latency offset so the screen matches what I hear. |
| US-RH-6 | As a player, the click drops out for two bars so I can test whether I keep time myself. |

## 3. Transport

- TR-1 There is one transport per window with states **Stopped**, **Counting in**, **Playing**, **Paused**. Every rhythm feature, run mode and song auto-scroll uses it.
- TR-2 The transport bar is always visible at the bottom of the window (Mac) or bottom of the screen (iPad) and shows: play/pause, tempo (number field, − and + buttons), tap tempo, meter, click on/off, drums style (or Off), speed-trainer status, bar:beat counter, elapsed time.
- TR-3 Space (Mac, and iPad with keyboard) toggles play/pause when focus is not in a text field. Esc stops and resets to bar 1.
- TR-4 Starting from Stopped always runs the count-in (RH-5) unless count-in is set to 0 bars.
- TR-5 Pausing holds the position; resuming continues from the next beat after a count-in of 1 bar (if count-in is on).
- TR-6 Tempo changes made while playing take effect at the next bar line (also ST-1.4).
- TR-7 The transport MUST stop when the app loses audio (device removed, iPad interruption), and show why.

## 4. Metronome

- RH-1 Tempo 30 to 250 bpm. Inputs: number field (typing commits on Enter or focus loss), − / + buttons (±1, long-press or Shift for ±5), keyboard `-`/`=` (±1) and `Shift-` (±5), and a slider on iPad.
- RH-1.1 Tap tempo (`T` or the Tap button): the tempo is the mean of the intervals of the last 4 taps; taps more than 2 seconds apart reset the count; the result is rounded to the nearest whole bpm and clamped to the range.
- RH-2 Meters: 2/4, 3/4, 4/4, 5/4, 6/8, 7/8, 9/8, 12/8. Accent on beat 1 by default. Groupings for odd meters are selectable: 5/4 as 3+2 or 2+3; 7/8 as 2+2+3, 2+3+2, 3+2+2; secondary group starts get a medium accent. 6/8, 9/8 and 12/8 count in dotted quarters (2, 3, 4 beats).
- RH-3 Subdivisions: none, 8ths, triplets, 16ths. Swing: off, or 50% to 75% (66% = triplet swing) applied to 8ths.
- RH-4 Sounds: wood block (default), beep, cowbell, hi-hat, voice count. Volumes: accent, beat, subdivision, each 0 to 100%, defaults 100 / 80 / 50.
- RH-5 Count-in: 0, 1 (default) or 2 bars, using the click even when drums are on, with a large visual count "1, 2, 3, 4" on screen.
- RH-6 Visual pulse: beat dots in the transport bar (accent dot larger) and an optional pulsing border around the neck; MUST respect Reduce Motion (pulse becomes a colour change without movement).
- RH-7 Gap click: "play N bars, mute M bars" with N, M in 1 to 8 (defaults 2 and 2); the visual pulse keeps going during the gap unless "Hide during gap" is on.

## 5. Drum backing

- DR-1 Styles (each with at least a **verse** and a **chorus** variation and one fill): Rock, Pop, Ballad, Folk strum, Country train, Shuffle / blues, Funk, Reggae (one drop), Bossa nova, Waltz (3/4), 6/8 ballad, Half-time. Each style declares its meter; selecting a style whose meter differs from the transport asks "Switch to 6/8?" (Yes changes the meter; No keeps the meter and the style is filtered out).
- DR-2 Drums follow the transport tempo with no change in pitch or sound quality from 30 to 250 bpm.
- DR-3 Fills: off, every 4 bars, every 8 bars (default). A fill replaces the last bar of the period; a crash cymbal plays on the next downbeat. In song view, fills land on the last bar before a new section regardless of the period.
- DR-4 Intensity: sparse (kick and hi-hat only, fewer hits), normal, full (adds ghost notes and open hats). Default normal.
- DR-5 Mixer: drums volume (default 80%), click over drums (default off), per-voice mutes for kick, snare, hats, toms, cymbals.
- DR-6 In song view, each section MAY have a variation set (verse or chorus); by default, sections labelled Chorus use chorus, everything else uses verse.
- DR-6.1 Humanise: off or on (default on): velocity varies ±8%, timing ±4 ms, never on the count-in.
- DR-7 *(Later)* User patterns in the text-grid format and MIDI drum file import.

## 6. Speed trainer

- ST-1 Parameters: start tempo, target tempo, step (1 to 20 bpm), advance rule, step back on miss (on/off), step-back amount (defaults to step).
- ST-1.1 Defaults: start 60, target = song tempo if known else 100, step 4, advance "on clean pass", step back off.
- ST-1.2 Start must be less than target; if equal, the trainer runs at that tempo and reports "Already at target".
- ST-1.3 Advance rules: **Every N bars** (N 1 to 64, default 8); **Every pass** (each time a practice sequence or song section loop completes); **On clean pass** (the player presses Return / taps "Clean" during or within 2 seconds after the pass end).
- ST-1.4 A tempo step always applies at the next bar line.
- ST-2 "Miss" (Backspace / tap "Miss") marks the current pass as not clean; with step back on, the tempo drops by the step-back amount at the next bar line, never below the start tempo.
- ST-3 On reaching the target, the trainer holds the target tempo, shows "Target reached" and plays a short chime (respecting the mute setting).
- ST-4 Progress display: "72 of 96 bpm" and a bar showing (current − start) / (target − start).
- ST-5 The highest tempo at which a pass was marked clean is saved as **last clean tempo** for the session or song; next time the trainer offers to start at last clean tempo minus two steps (not below the start tempo).

## 7. Latency

- RH-8 Output latency compensation: automatic (from the device's reported latency) plus a manual offset from −50 to +300 ms (default 0). Visual events are delayed by the total so they line up with what is heard.
- RH-8.1 A calibration screen plays a click and flashes in time; the player adjusts the offset until they look and sound together.
- RH-8.2 When the output is Bluetooth, a one-time notice says "Bluetooth adds delay. Wired headphones or the built-in speaker work best. You can adjust the offset in Settings."

## 8. Edge cases and error states

| ID | Situation | Required behaviour |
|----|-----------|--------------------|
| E-RH-1 | Tempo typed outside range | Clamped to 30 or 250, with a brief "Tempo limited to 30 to 250 bpm". |
| E-RH-2 | Non-number typed | Field reverts to the last valid value. |
| E-RH-3 | Audio device changes while playing | Transport stops; notice "Audio output changed"; Play resumes on the new device. |
| E-RH-4 | iPad interruption (call, Siri, alarm) | Transport pauses; resumes only on the player's Play. |
| E-RH-5 | App goes to the background (iPad) or window minimised (Mac) | iPad: pauses. Mac: keeps playing (the player may be reading another window). |
| E-RH-6 | System under load (audio buffer underrun) | Audio thread never blocks; a dropout counter is logged locally; if more than 3 dropouts happen in a minute, suggest a larger buffer in Settings. |
| E-RH-7 | Style meter conflicts with a song's meter | Song meter wins; styles that do not match are hidden in song view. |

## 9. Acceptance criteria

| ID | Given | When | Then |
|----|-------|------|------|
| AC-RH-1.1 | 120 bpm, 4/4 | Play for 10 minutes (offline render) | Every click onset is within 1 sample of its ideal position; total drift 0. |
| AC-RH-1.2 | Taps at 500 ms intervals | 4 taps | Tempo is 120. |
| AC-RH-2.1 | 7/8 as 2+2+3 | Play one bar | Accents on eighths 1 (strong), 3 and 5 (medium). |
| AC-RH-3.1 | 100 bpm, 8ths, swing 66% | Render | Off-beat eighth falls at 2/3 of the beat. |
| AC-TR-6.1 | Playing at 72 bpm, mid-bar | Tempo set to 76 | First beat at 76 bpm is the next downbeat. |
| AC-DR-2.1 | Rock, verse | Render at 40 and 200 bpm | Same sample set; spectral centroid of kick hits differs by less than 1%. |
| AC-DR-3.1 | Fills every 8 bars | Play 16 bars | Fills in bars 8 and 16; crash on bars 9 and 17. |
| AC-ST-1.1 | 60 to 72, step 4, on clean pass | Three clean passes | Tempos 60, 64, 68, 72; then "Target reached". |
| AC-ST-2.1 | At 68, step back on | Miss | Next bar at 64. |
| AC-ST-5.1 | Last clean tempo 84, step 4, start 60 | Open the trainer next time | Offers to start at 76. |
| AC-RH-8.1 | Offset 150 ms | Playing | Visual beat event timestamps are 150 ms (± 2 ms) after the audio sample time of the click (instrumented test). |

## 10. Out of scope

Tempo maps (gradual tempo changes inside a song), polyrhythms, user-recorded click sounds, MIDI clock sync with external gear.
