# Learning Features

Status: Draft v0.2 (2026-11-15). Requirements are specified in full in [PRD 01](prd/01-fretboard.md) (scales, tunings), [PRD 02](prd/02-tuner.md) (tuner) and [PRD 06](prd/06-later-features.md) (later features); this doc holds the design notes. If they differ, the PRD wins.

These features build on the fretboard, practice, song and rhythm layers. Agreed phasing: the **tuner**, the **scales and modes overlay** and **left-handed mode with alternate tunings** ship in v1; the rest follow in later phases (see [roadmap.md](roadmap.md)).

| Feature | IDs | Phase |
|---------|-----|-------|
| Built-in tuner | TU-* | v1 (P1) |
| Scales and modes overlay | SC-* | v1 (P1) |
| Left-handed mode and alternate tunings | LH-* | v1 (P1) |
| CAGED and position maps | CG-* | P6 |
| Theory tips | TT-* | P6 |
| Progress, streaks and drill suggestions | PR-* | P6 |
| Ear training | ET-* | P7 |
| Looper and record yourself | LP-* | P7 |
| Chord and note recognition by microphone | MR-* | P8 |

## 1. Built-in tuner (v1)

| ID | Requirement |
|----|-------------|
| TU-1 | Chromatic tuner from the built-in microphone on Mac or iPad, or any audio interface CoreAudio supports: detected note, octave, cents offset (needle and numeric), in tune within ±3 cents. |
| TU-2 | Tuning-aware string mode: shows the target string for the current tuning (standard, Drop D, DADGAD, open G, custom) and auto-selects the nearest string. |
| TU-3 | Reference pitch A4 adjustable 415 to 466 Hz (default 440). |
| TU-4 | Reference tones: play each open string through the sampler to tune by ear. |
| TU-5 | Capo aware: with a capo set, shows the sounding notes. |

Design: pitch detection with the McLeod Pitch Method (or YIN) on a cpal input stream (CoreAudio) feeding an analysis thread, 2048-sample window, smoothing over about 3 frames. Runs entirely on device. The microphone permission prompt appears only when the user first opens the tuner. See [ADR 0006](adr/0006-on-device-audio-analysis.md).

## 2. Scales and modes overlay (v1)

| ID | Requirement |
|----|-------------|
| SC-1 | Overlay a scale under the current chord or key: major, natural minor, major and minor pentatonic, blues, harmonic minor, and the 7 diatonic modes. |
| SC-2 | Chord tones drawn solid, other scale tones drawn hollow, so a beginner sees "safe" target notes inside the scale. |
| SC-3 | Position boxes: the 5 pentatonic boxes and 3-notes-per-string major patterns; cycle boxes with the same next/previous position keys as chords. |
| SC-4 | "Follow the chords" mode in song and practice views: the overlay switches with each chord (e.g. A minor pentatonic over Am, with the chord's tones lit). |
| SC-5 | Labels by note name, scale degree or interval, using the existing labels toggle. |

The theory engine already models scales and modes; this adds pentatonic, blues and harmonic minor to `Mode` in `mackey-theory`, plus a `ScalePattern` library in `mackey-voicing` for boxes.

## 3. Left-handed mode and alternate tunings (v1)

| ID | Requirement |
|----|-------------|
| LH-1 | Left-handed view mirrors the neck horizontally; a separate "flip strings" option puts low E at the top or bottom to match how the player looks down at the guitar. |
| LH-2 | Tuning presets: Standard, Drop D, Double Drop D, DADGAD, Open G, Open D, Open E, Half-step down, Whole-step down; plus custom (any pitch per string). |
| LH-3 | Capo 0 to 12, with partial capo as a later option. |
| LH-4 | Every voicing, scale box, triad and arpeggio is recomputed for the tuning, so nothing assumes standard tuning. The curated shape library is per tuning, with search filling the gaps for less common tunings. |
| LH-5 | Songs can store a tuning (`{tuning: D A D G B E}` in ChordPro metadata) and the app offers to switch when a song is opened. |

## 4. CAGED and position maps (P6)

| ID | Requirement |
|----|-------------|
| CG-1 | For any major or minor chord, show all five CAGED shapes along the neck at once, each in its own colour, with overlaps shaded. |
| CG-2 | Step through shapes in order (C, A, G, E, D, then back to C an octave up) with animation showing which fingers move. |
| CG-3 | Link each shape to its arpeggio and its pentatonic box (SC-3), so chord, arpeggio and scale positions line up. |
| CG-4 | Position map: pick a 4-fret window and see every diatonic chord, triad and arpeggio of the key playable inside it. |
| CG-5 | "Practice this" emits notation, e.g. `G:chord caged:C > G:chord caged:A > G:chord caged:G ...`. |

## 5. Theory tips (P6)

| ID | Requirement |
|----|-------------|
| TT-1 | For any chord in a key or song: its Roman numeral, function (tonic, subdominant, dominant), and why it belongs (which scale notes it is built from). |
| TT-2 | Borrowed and secondary chords are explained when detected (e.g. bVII in a major key, V/V). |
| TT-3 | Common substitutions: relative minor/major, sus and add colours, 7ths, tritone substitution for advanced players, with "hear it" and "see it" buttons. |
| TT-4 | Tips are short (one or two sentences) and shown in a side panel, never blocking. Beginners can turn them off. |

Implementation: rules in `mackey-theory` (harmonic function and substitution tables) produce structured facts; a small template layer turns facts into sentences. No network or LLM dependency in v1, which keeps it offline and consistent.

## 6. Progress, streaks and drill suggestions (P6)

| ID | Requirement |
|----|-------------|
| PR-1 | Dashboard: practice minutes per day and week, current streak, sessions and songs practised. |
| PR-2 | Per transition (e.g. `Em>C`): changes per minute over time. Per song and session: highest clean tempo over time. |
| PR-3 | Drill suggestions: "Your slowest change this week is C>D. 5 minutes of one-minute changes?" Ranked by low changes-per-minute, low clean tempo vs target, and time since last practised. |
| PR-4 | Gentle streaks: a missed day does not reset to zero if a weekly goal is met, so streaks motivate without punishing. |
| PR-5 | All data local; export as CSV. |

Builds on the `RunRecord` data from practice sessions and the speed trainer.

## 7. Ear training (P7)

| ID | Requirement |
|----|-------------|
| ET-1 | Interval drills: hear two notes, name the interval or click the second note on the neck from the first. |
| ET-2 | Chord-quality drills: major, minor, 7, maj7, m7, sus, dim. |
| ET-3 | Progression drills in a key: hear I-IV-V-I style progressions and identify the degrees. |
| ET-4 | "Find it on the neck": hear a note or chord, then play it; graded by microphone recognition once MR-* exists, by clicking the fretboard before that. |
| ET-5 | Difficulty adapts to accuracy; results feed the progress dashboard. |

Ear-training drills use the same sampler as playback and can be typed as practice notation, e.g. `@ear intervals M3 P5 m7`.

## 8. Looper and record yourself (P7)

| ID | Requirement |
|----|-------------|
| LP-1 | Record the guitar input over the drum backing and click, aligned to the transport clock, with a count-in. |
| LP-2 | Play back the take with or without the backing; loop a section. |
| LP-3 | Timing view: detected note onsets drawn against the beat grid, showing early/late against the click (onset detection on the recorded audio). |
| LP-4 | Layered loops: record a chord loop, then practise lead or arpeggios over it. |
| LP-5 | Takes saved locally per song or session, with an option to export as audio. |

Design: an input tap writes to a ring buffer; the transport clock time-stamps the start, and the round-trip latency (from the device, or a one-off loopback calibration) is subtracted so takes line up with the beat.

## 9. Chord and note recognition by microphone (P8)

| ID | Requirement |
|----|-------------|
| MR-1 | In run mode, listen and confirm the expected chord or note was played (green check / amber "close" / red), with the strings that sounded wrong highlighted where possible. |
| MR-2 | Count clean changes automatically in one-minute changes and speed-trainer "clean pass" mode, instead of the player pressing Return. |
| MR-3 | Works with acoustic (built-in mic) and electric (audio interface); a calibration step sets input level and noise floor. |
| MR-4 | Never required: every feature still works by self-report. |

Design: monophonic notes reuse the tuner's pitch detector. Chords use a constant-Q / chroma feature compared with templates for the expected chord and its likely confusions; because the app knows what *should* be played, this is verification of an expected chord, which is much more reliable than open-ended chord recognition. An ONNX model trained on guitar recordings, run with ONNX Runtime, is the upgrade path if template matching is not accurate enough.
