# Mackey Guitar Tutor: Design Docs

Mackey Guitar Tutor is a macOS application that helps beginner and intermediate guitarists learn the fretboard and play real songs. This folder holds the product spec, architecture and phased plan. Nothing is implemented yet; these docs are the starting point for the first build.

| Doc | What it covers |
|-----|----------------|
| [product-spec.md](product-spec.md) | Vision, users, the three feature areas, numbered requirements |
| [architecture.md](architecture.md) | Module layout, domain model, persistence, key algorithms |
| [practice-sessions.md](practice-sessions.md) | Practice session design and the quick-entry notation |
| [rhythm.md](rhythm.md) | Metronome, drum backing and the speed trainer |
| [song-import.md](song-import.md) | Finding, importing and parsing songs, plus legal/ToS constraints |
| [roadmap.md](roadmap.md) | Phased delivery plan with exit criteria |
| [adr/](adr/) | Architecture Decision Records |

## Feature areas at a glance

1. **Fretboard explorer** (foundation). Pick a key and see notes, chords, triads and arpeggios on the neck. Cycle through scale degrees, chord qualities, inversions and positions.
2. **Practice sessions.** Type a short sequence such as `G > Em >> C > D` plus a method such as "one-minute changes" and the app drills it with the fretboard, a metronome and a timer.
3. **Songs.** Bring in a song's chords and lyrics (paste, ChordPro file, share from a browser, later licensed sources), parse it, and show the chart next to fretboard help. Any song can become a practice session in one click.
4. **Rhythm.** A metronome and drum backing that share one clock with the fretboard, plus a speed trainer that starts slow and steps up to the song's real tempo as the player masters it.

Areas 2, 3 and 4 are built on the same music theory engine and fretboard view as area 1, so area 1 ships first. Area 4 is a shared service used by the other three.
