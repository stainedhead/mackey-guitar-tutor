# Product Spec

Status: Draft v0.1 (2026-11-15)

## 1. Vision

Beginners can memorize a few open chords but get lost on the rest of the neck. Mackey Guitar Tutor shows *where* the notes of a key, chord, triad or arpeggio live, lets the player step through them at their own pace, turns that knowledge into deliberate practice, and connects it to songs they actually want to play.

## 2. Users

| Persona | Goal | What they need from us |
|---------|------|------------------------|
| **Beginner** (0 to 12 months) | Play first songs, change chords cleanly | Open chord shapes, slow drills, clear "what finger where" |
| **Improver** (1 to 3 years) | Escape the open position, understand why chords fit | Triads across string sets, CAGED positions, arpeggios, Roman numerals |
| **Teacher** | Hand students a focused routine | Quick-to-type practice sessions that can be shared as text |

## 3. Scope

In scope for v1: macOS and iPad apps (Rust core in a Tauri shell; Windows, Linux, iPhone and Android deferred), for personal use, distributed as an unsigned direct download on the Mac ([platforms.md](platforms.md)), 6-string guitar, right- and left-handed views, standard and alternate tunings (Drop D, DADGAD, open G and others) with capo, scale and mode overlays, a built-in tuner (microphone input for single-note pitch only), local storage.

Out of scope for v1, planned for later phases: CAGED maps, theory tips, progress and streaks, ear training, looper and record-yourself, chord recognition by microphone ("listen and grade"). Not planned for now: Windows, Linux, iPhone and Android builds, App Store release, iCloud sync (later), recorded (non-synthesized) backing tracks, cloud sync, accounts, iOS/iPadOS, bass and 7-string. The architecture keeps these open (see [architecture.md](architecture.md)).

## 4. Feature area A: Fretboard explorer (foundation)

| ID | Requirement |
|----|-------------|
| FB-1 | Render a neck with 6 strings and 0 to 24 frets, inlays, configurable visible fret range, left/right-handed. |
| FB-2 | Set a key (tonic + mode: major, natural minor, and the 7 diatonic modes). Scale overlays including pentatonic and blues are SC-1 to SC-5. |
| FB-3 | Show all notes of the key on the neck, labelled by note name, scale degree, or interval (toggle). |
| FB-4 | Select a chord by symbol (`Cmaj7`) or by degree (`IV`). Show every occurrence of its tones, root highlighted. |
| FB-5 | Show playable **voicings**: open shapes, CAGED shapes, and triads on each adjacent 3-string set, with all inversions. |
| FB-6 | Show **arpeggios** for a chord within a chosen position (fret window of about 4 to 5 frets), with suggested fingering. |
| FB-7 | **Cycle** with keyboard and buttons: next/previous scale degree, next/previous voicing, next/previous inversion, next/previous position up the neck. |
| FB-8 | Play the selection as audio (strummed or arpeggiated) using built-in samples. |
| FB-9 | Tuning and capo setting, which change every displayed fret position (detail in LH-1 to LH-5). |

## 5. Feature area B: Practice sessions

A practice session is a short, named routine: a sequence of items (notes, chords, triads, arpeggios), the transitions between them, and the method used to practice them. Full design in [practice-sessions.md](practice-sessions.md).

| ID | Requirement |
|----|-------------|
| PS-1 | Create a session by typing one line of **quick-entry notation**, e.g. `key G; G > Em >> C > D x4 @ changes 60s`. A live preview shows the parsed result and any errors as you type. |
| PS-2 | Items can be chords (`Em7`), degrees (`vi`), triads (`C:tri inv1 str123`), arpeggios (`Am:arp pos5`), or single notes (`n:F#`). |
| PS-3 | Mark specific **transitions** as focus changes (`>>`) so the drill spends extra time on them. |
| PS-4 | Choose a **method** from presets (one-minute changes, speed trainer, loop, random order, name-the-note, strum pattern) with parameters. |
| PS-5 | Run mode: fretboard shows the current and the *next* item (ghosted), metronome or drum backing, count-in, timer, and progress through the sequence. |
| PS-6 | Log each run (date, tempo reached, changes per minute, self-rating). Show simple history per session and per transition. |
| PS-7 | Sessions are plain text under the hood, so they can be copied, pasted and shared. |
| PS-8 | Generate a session from a song (Feature C) or from the current fretboard selection. |

## 6. Feature area C: Songs (chords and lyrics)

Full design and legal analysis in [song-import.md](song-import.md).

| ID | Requirement |
|----|-------------|
| SG-1 | Import by **pasting text** in the common "chords over lyrics" layout; auto-detect chord lines vs lyric lines. |
| SG-2 | Import **ChordPro** (`.cho`, `.chopro`, `.crd`) files and inline `[G]lyrics` text; export to ChordPro. |
| SG-3 | Import from a web page the user is viewing, via drag and drop, the clipboard, or a "paste URL" action (a browser extension or macOS Share extension can feed the same path later), user-initiated, one page at a time, stored only locally. |
| SG-4 | **Search** for songs: show web search results that open in the user's browser; import happens only through SG-1 to SG-3. Licensed providers can plug in later behind the same interface. |
| SG-5 | Display the song as a chart: lyrics with chords aligned above, sections (verse, chorus), and a chord palette of every chord used. |
| SG-6 | Click any chord to see it on the fretboard. The app suggests a **voicing set** that minimises hand movement between consecutive chords. |
| SG-7 | Transpose, set capo (with "capo to play in open shapes" suggestion), and detect the likely key so Roman numerals can be shown. |
| SG-8 | Auto-scroll at the song's tempo (or the speed trainer's current tempo), with optional drum backing that follows the sections; current line and current chord highlighted, with the fretboard following along. |
| SG-9 | One click "Practice this song": builds a practice session from the song's chord progression, flagging its hardest transitions as focus changes. |
| SG-10 | Users can edit an imported song (fix a mis-parsed chord, move a chord) before saving. |

## 7. Feature area D: Rhythm (metronome, drum backing, speed trainer)

Players need to practise in cadence. A metronome and drum backing run at an adjustable tempo so the player can start slow and work up to the right speed. Full design in [rhythm.md](rhythm.md).

| ID | Requirement |
|----|-------------|
| RH-1 to RH-7 | Metronome: 30 to 250 bpm, tap tempo, time signatures and accents, subdivisions and swing, click sounds, count-in, visual pulse, gap click. |
| DR-1 to DR-7 | Drum backing: style library with variations, follows tempo with no pitch change, fills, intensity, mixer and part mutes, follows song sections; user patterns later. |
| ST-1 to ST-5 | Speed trainer: start, target and step tempo; advance every N bars, every pass or on a clean pass; step back on a miss; progress and last clean tempo remembered. |

Rhythm is available everywhere: in the explorer (for arpeggio and scale drills), in practice sessions (`@ladder 60-96 +4 drums:rock`), and in song view, where the target tempo is the song's tempo.

## 8. Feature area E: Learning features

Full design in [learning-features.md](learning-features.md).

| Feature | IDs | Phase |
|---------|-----|-------|
| Built-in tuner (chromatic and per-string, any tuning) | TU-1 to TU-5 | v1 |
| Scales and modes overlay (pentatonic, major/minor, modes, boxes, follow-the-chords) | SC-1 to SC-5 | v1 |
| Left-handed mode and alternate tunings (Drop D, DADGAD, open G, custom, capo) | LH-1 to LH-5 | v1 |
| CAGED and position maps | CG-1 to CG-5 | Later (P6) |
| Theory tips (Roman numerals, function, why it fits, substitutions) | TT-1 to TT-4 | Later (P6) |
| Progress, streaks and drill suggestions | PR-1 to PR-5 | Later (P6) |
| Ear training | ET-1 to ET-5 | Later (P7) |
| Looper and record yourself over the drums | LP-1 to LP-5 | Later (P7) |
| Chord and note recognition by microphone | MR-1 to MR-4 | Later (P8) |

## 9. Non-functional requirements

| ID | Requirement |
|----|-------------|
| NF-1 | Fretboard redraw under 16 ms (60 fps) while cycling. |
| NF-2 | Quick-entry parse and preview under 50 ms per keystroke. |
| NF-3 | Fully usable offline; network only for search and URL import. |
| NF-4 | Keyboard-first: every cycling and run-mode action has a shortcut. |
| NF-5 | Accessibility (confirmed): VoiceOver labels for every fretboard position on Mac and iPad, full keyboard control (including iPad with a hardware keyboard), colour-blind-safe palette with shape cues, respects system text size and Reduce Motion. Detail in [platforms.md §6](platforms.md#6-accessibility-confirmed). |
| NF-6 | Music theory engine at 95%+ line coverage; parsers covered by a fixture corpus of real-world chord sheets. |
| NF-8 | Rhythm timing is sample-accurate: under 1 ms drift over 10 minutes, tempo changes land on bar lines. |
| NF-7 | No song content or audio leaves the device; no telemetry without opt-in. |
| NF-10 | Touch-first on iPad: every hover action has a tap equivalent; tap targets at least 44 pt. |
| NF-11 | Bundled sounds are CC0, public domain or MIT only ([sound-assets.md](sound-assets.md)). |
| NF-9 | Tuner reads within ±3 cents and updates at least 20 times per second. |

## 10. Decisions and open questions

Decided (J, 2026-11-15):

- **Stack and platforms:** Rust core, Tauri shell, macOS and iPad now; Windows, Linux, iPhone and Android later.
- **Distribution:** personal use for now; unsigned (ad-hoc signed) direct download on the Mac; App Store maybe later. The design stays sandbox-friendly so that move is cheap.
- **Audio interfaces:** rely on whatever CoreAudio supports on Mac and iPad.
- **Audio and storage:** all audio on the native Rust thread; local storage, optional iCloud sync later.
- **Sound assets:** open-source or licence-free only (CC0, public domain, MIT).
- **Accessibility:** colour-blind-safe colours, keyboard control and VoiceOver.

Still open:

1. **iPad signing route:** free Apple ID (reinstall every 7 days) or the Apple Developer Program ($99/year). See [platforms.md §7](platforms.md#7-open-decision).
2. **Sharing:** are teacher-to-student shared sessions in scope (files only, or a service)?
