# Product Requirements Document (PRD)

- Status: Draft v1.0 (2026-11-15), for review by J
- Owner: J
- Replaces: `docs/product-spec.md` (its requirement IDs are kept and expanded here)
- Intent: [INTENT.md](../../INTENT.md). If this PRD and the intent disagree, the intent wins and the PRD is fixed.

This PRD says **what** Mackey Guitar Tutor must do, in enough detail that design and implementation do not have to guess. The design docs in [docs/](../README.md) say **how**. Every requirement here traces to a design doc and a test ([09-traceability.md](09-traceability.md)).

## Documents

| # | Document | Covers | Phase |
|---|----------|--------|-------|
| 00 | This file | Conventions, scope, personas, glossary, global rules | All |
| 01 | [01-fretboard.md](01-fretboard.md) | Fretboard explorer, keys, chords, voicings, triads, arpeggios, scale overlays, tunings, left-handed | v1 (P1) |
| 02 | [02-tuner.md](02-tuner.md) | Built-in tuner | v1 (P1) |
| 03 | [03-rhythm.md](03-rhythm.md) | Metronome, drum backing, speed trainer | v1 (P1b) |
| 04 | [04-practice.md](04-practice.md) | Practice sessions, quick-entry notation, run mode, history | v1 (P2) |
| 05 | [05-songs.md](05-songs.md) | Song import, chart, transpose, capo, voicings, practice from song, search | v1 (P3, P4); web import P5 |
| 06 | [06-later-features.md](06-later-features.md) | CAGED maps, theory tips, progress, ear training, looper, mic recognition | P6 to P8 |
| 07 | [07-ux-flows.md](07-ux-flows.md) | Navigation, Mac and iPad layouts, key flows, shortcuts, gestures | All |
| 08 | [08-nfr.md](08-nfr.md) | Performance, timing, accessibility, privacy, data, settings, errors | All |
| 09 | [09-traceability.md](09-traceability.md) | Requirement to design doc to test | All |

## 1. Conventions

**Requirement keywords.** MUST, SHOULD and MAY follow RFC 2119. MUST is required for the requirement's phase to be done. SHOULD is expected unless the PRD owner agrees to drop it. MAY is optional.

**IDs.** `AREA-n` is a requirement (for example `FB-5`). `AREA-n.m` is a sub-requirement. `US-AREA-n` is a user story. `AC-AREA-n.k` is an acceptance criterion. `E-AREA-n` is an edge case or error state. IDs are never reused; a dropped requirement is marked *Withdrawn*.

**Priority and phase.** Each requirement has a phase from [roadmap.md](../roadmap.md). "v1" means phases P0 to P4. Anything later is marked with its phase.

**Acceptance criteria** are written as Given / When / Then, and each becomes at least one automated test unless marked *(manual)*.

**Platforms.** Unless a requirement says otherwise, it applies to both macOS and iPad. iPad behaviour is verified in the iOS Simulator until device validation ([platforms.md §7](../platforms.md#7-ipad-signing-and-validation-decided)).

## 2. Scope

### 2.1 In scope for v1 (P0 to P4)

- macOS 13+ (universal) and iPadOS 17+ apps from one Rust + Tauri codebase.
- Personal use; macOS shipped as an ad-hoc signed direct download; iPad built and tested in CI and the iOS Simulator.
- Standard 6-string guitar, 19 to 24 frets, right- or left-handed, standard and alternate tunings, capo.
- Fretboard explorer: keys, modes, chords, voicings, triads, arpeggios, scale overlays, cycling.
- Tuner.
- Metronome, drum backing, speed trainer.
- Practice sessions with quick-entry notation, run mode and history.
- Songs from pasted text and ChordPro files, chart view, transpose, capo, key detection, voicing sets, auto-scroll, practice from song.
- Local storage; export and import of the library as files.
- Accessibility: colour-blind-safe colours with shape cues, full keyboard control, VoiceOver.

### 2.2 Later phases (planned)

- P5: web import (drag and drop of page content, deep link, Share extension) and song search, behind a legal review.
- P6: CAGED and position maps, theory tips, progress, streaks and drill suggestions.
- P7: ear training, looper and record-yourself.
- P8: chord and note recognition by microphone.
- Later: Mac App Store, Apple Developer Program signing, iCloud sync, user drum patterns and MIDI drum import, Windows, Linux, iPhone and Android, bass and 7-string.

### 2.3 Out of scope (not planned)

- Hosting, caching or redistributing song lyrics or chord sheets; any crawler or background scraping.
- Accounts, a backend service, analytics or telemetry.
- Recorded (non-synthesized) backing tracks; full song arrangement or a DAW.
- Tablature editing or standard notation scores (tab blocks in imported songs are shown as plain text only).
- Social features.

## 3. Personas

| Persona | Description | Top needs | Default experience |
|---------|-------------|-----------|--------------------|
| **J (primary)** | The owner and first user. Experienced in software, learning to find positions across the neck and to play songs in time. | Clear neck views, fast practice entry, songs from pasted sheets, start-slow tempo work | All features visible; beginner defaults |
| **Bea, beginner** | Six months of playing; knows G, C, D, Em, Am; changes are slow. | Open shapes, slow tempo, one-minute changes, a song at a time | Open voicings first, note names, 60 bpm |
| **Ivan, improver** | Two years; stuck in open position; wants triads and arpeggios up the neck. | Triads on string sets, inversions, positions, Roman numerals, modes | Same app; uses deeper options |
| **Tess, teacher** *(later)* | Gives students short routines. | Quick notation she can paste into a message | Notation copy and paste |

## 4. Glossary

| Term | Meaning in this product |
|------|------------------------|
| String number | 1 is the highest-pitched string (high E in standard tuning), 6 the lowest. |
| Fret | 0 is the open string. Fret *n* is *n* semitones above the open string (plus the capo). |
| Capo fret | With a capo at *c*, fret numbers shown are absolute (from the nut); frets below *c* are unavailable and drawn dimmed. Voicings are found relative to the capo. "Shape" names are relative to the capo, "sounding" names are absolute. |
| Key | A tonic plus a mode. Determines note spelling. |
| Chord tone | A note that belongs to the selected chord. |
| Scale tone | A note in the selected scale that is not a chord tone. |
| Voicing | One concrete way to play a chord: a fret (or mute) per string. |
| Shape | A named voicing pattern (open C shape, E-shape barre, and so on). |
| Triad | A three-note voicing (root, third, fifth) on three adjacent strings, in root position or first or second inversion. |
| Inversion | Which chord tone is lowest: root position (0), first (third in bass), second (fifth in bass), third (seventh in bass, seventh chords only). |
| Position | A fret window. "Position *n*" is the 4-fret window starting at fret *n* (frets *n* to *n*+3), with a one-fret stretch allowed either side. |
| Arpeggio | The chord tones played one at a time, ascending and/or descending, inside a position. |
| String set | Three adjacent strings: 123 (top), 234, 345, 456 (bottom). |
| Item | One step in a practice session (a chord, triad, arpeggio or note). |
| Transition | The change from one item to the next. A *focus transition* is one marked with `>>`. |
| Clean | Played correctly and in time, judged by the player (or, from P8, by the microphone). |
| bpm | Beats per minute, where the beat is the meter's beat unit (quarter note in 4/4, dotted quarter in 6/8 and 12/8). |
| Transport | The shared clock that drives clicks, drums, run mode and chart scrolling. |

## 5. Global rules

These rules apply to every feature. Feature documents refer to them by number.

### G-1 Note spelling

- G-1.1 Every displayed note name MUST be spelled from the current key's key signature. Example: in F major the fourth degree is **Bb**, never A#.
- G-1.2 Major keys offered: C, G, D, A, E, B, F#/Gb, F, Bb, Eb, Ab, Db. The picker shows "F#/Gb" as one entry and asks which spelling to use (default F#). C# major and Cb major are not offered (use Db and B).
- G-1.3 Minor keys offered: A, E, B, F#, C#, G#, D#, Eb, D, G, C, F, Bb. D# and Eb are both offered as "D#/Eb".
- G-1.4 Modes take the key signature of their parent major key. If the parent key would be one not offered in G-1.2, the enharmonic tonic is used (for example D# Dorian is shown as Eb Dorian).
- G-1.5 Chromatic notes (not in the key) are spelled by their function in the chord they belong to (for example in C major, the third of D7 is **F#**, and the root of the bVII chord is **Bb**). With no chord context, flat keys and C major use flats, and sharp keys use sharps.
- G-1.6 Double sharps and double flats MAY appear in theory detail views (interval names) but MUST NOT appear as fretboard labels; the enharmonic simple name is shown instead, with the theory spelling in the tooltip.
- G-1.7 Chord symbols entered by the player keep the player's spelling (`A#m` stays `A#m`), unless the player runs "Respell for key".

### G-2 Fretboard visual language

| Element | Shape | Colour token (Okabe-Ito based, colour-blind safe) | Label |
|---------|-------|-------------------------------------------------|-------|
| Root | Filled rounded square | `--note-root` (vermillion #D55E00) | Per labels mode |
| Third | Filled circle | `--note-third` (blue #0072B2) | Per labels mode |
| Fifth | Filled circle | `--note-fifth` (bluish green #009E73) | Per labels mode |
| Seventh | Filled circle | `--note-seventh` (orange #E69F00) | Per labels mode |
| Other chord tones (6, 9, 11, 13, sus) | Filled circle | `--note-ext` (reddish purple #CC79A7) | Per labels mode |
| Scale tone, not in chord | Hollow ring | `--note-scale` (neutral grey) | Per labels mode, smaller text |
| Single selected note (no chord) | Filled diamond | `--note-root` | Note name |
| Next item (ghost) in run mode | Dashed outline, 40% opacity | Same as its role | None |
| Anchor (finger stays put) | Solid outline ring around the dot | `--anchor` (black / white by theme) | Finger number |
| Open string to play | Ring above the nut | Role colour | "0" or label |
| Muted string | "×" above the nut | `--muted` | None |

- G-2.1 Colour is never the only cue: role is also shown by shape, and on hover/tap a label names the role ("Root", "3rd", ...).
- G-2.2 Labels mode is one of: **Note name** (default), **Scale degree** (1, b3, 5 ...; relative to the key tonic), **Interval** (R, 3, 5, b7 ...; relative to the chord root), **Finger** (1 to 4, only when a voicing is shown), **None**.
- G-2.3 Light and dark themes both meet the contrast in [08-nfr.md](08-nfr.md#3-accessibility).

### G-3 Selection model

- G-3.1 There is exactly one **current selection** per window, made of: key; focus (none, chord, scale, or note); for a chord, the chord and a view type (all tones, voicing, triad or arpeggio); the voicing, inversion, string-set and position indexes; and the scale overlay (on or off, and which scale).
- G-3.2 The fretboard always draws the current selection. Every feature (explorer, practice, songs) sets the selection through the same core command and never draws the neck differently.
- G-3.3 Changing the key keeps the focus type and moves the selection to the same scale degree in the new key (for example the IV chord in G becomes the IV chord in D).
- G-3.4 Changing tuning or capo keeps the selected chord or scale and recomputes positions; if the selected voicing no longer exists, the best-scored voicing is selected and a brief notice says "Voicing changed for new tuning".

### G-4 Defaults (first launch)

| Setting | Default |
|---------|---------|
| Key | G major |
| Tuning | Standard (E A D G B E) |
| Capo | 0 |
| Handedness | Right-handed; nut on the left; string 1 (high E) at the top and string 6 at the bottom, as in tab. "Flip strings" puts string 6 at the top, as seen when looking down at the guitar |
| Frets on the instrument | 22 |
| Visible fret range | Mac: 0 to 15. iPad landscape: 0 to 12. iPad portrait: 0 to 7 (scrolls) |
| Labels | Note name |
| Tempo | 70 bpm, 4/4 |
| Metronome sound | Wood block, accent on beat 1 |
| Count-in | 1 bar |
| Drum backing | Off |
| Theme | Follows the system |

### G-5 Numbers and limits

| Value | Range | Step |
|-------|-------|------|
| Tempo | 30 to 250 bpm | 1 (also ±5 shortcut) |
| Frets on instrument | 19 to 24 | 1 |
| Capo | 0 to 12 | 1 |
| Reference pitch A4 | 415 to 466 Hz | 1 Hz |
| Practice session length | 1 to 64 items after expanding repeats; up to 1,000 characters of notation | |
| Song size | up to 2,000 lines and 200 KB of text | |
| Library | at least 1,000 songs and 1,000 sessions without slowdown | |

### G-6 Undo and safety

- G-6.1 Every edit to a session, song or setting MUST be undoable (Cmd-Z on Mac, three-finger swipe or the Undo button on iPad) for at least the last 50 actions in the current window.
- G-6.2 Deleting a song or session moves it to **Recently deleted** for 30 days, from where it can be restored.
- G-6.3 Unsaved edits are auto-saved as drafts every 2 seconds and on window close or app backgrounding. Nothing the player typed is lost on a crash.
