# Mackey Guitar Tutor: Design Docs

Mackey Guitar Tutor is a desktop application for macOS (Rust core, Tauri shell) that helps beginner and intermediate guitarists learn the fretboard and play real songs. This folder holds the product spec, architecture and phased plan. Nothing is implemented yet; these docs are the starting point for the first build.

| Doc | What it covers |
|-----|----------------|
| [product-spec.md](product-spec.md) | Vision, users, the three feature areas, numbered requirements |
| [architecture.md](architecture.md) | Module layout, domain model, persistence, key algorithms |
| [practice-sessions.md](practice-sessions.md) | Practice session design and the quick-entry notation |
| [rhythm.md](rhythm.md) | Metronome, drum backing and the speed trainer |
| [learning-features.md](learning-features.md) | Tuner, scales overlay, left-handed and tunings, CAGED maps, theory tips, progress, ear training, looper, mic recognition |
| [song-import.md](song-import.md) | Finding, importing and parsing songs, plus legal/ToS constraints |
| [roadmap.md](roadmap.md) | Phased delivery plan with exit criteria |
| [adr/](adr/) | Architecture Decision Records (0001: Rust core with a Tauri shell, macOS first) |

## Feature areas at a glance

1. **Fretboard explorer** (foundation). Pick a key and see notes, chords, triads and arpeggios on the neck. Cycle through scale degrees, chord qualities, inversions and positions.
2. **Practice sessions.** Type a short sequence such as `G > Em >> C > D` plus a method such as "one-minute changes" and the app drills it with the fretboard, a metronome and a timer.
3. **Songs.** Bring in a song's chords and lyrics (paste, ChordPro file, share from a browser, later licensed sources), parse it, and show the chart next to fretboard help. Any song can become a practice session in one click.
4. **Rhythm.** A metronome and drum backing that share one clock with the fretboard, plus a speed trainer that starts slow and steps up to the song's real tempo as the player masters it.

5. **Learning features.** A tuner, scale and mode overlays, left-handed mode and alternate tunings in v1; CAGED maps, theory tips, progress and streaks, ear training, a looper and microphone chord checking in later phases.

Areas 2 to 5 are built on the same music theory engine and fretboard view as area 1, so area 1 ships first. Area 4 is a shared service used by the other three.
