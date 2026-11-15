# Mackey Guitar Tutor: Design Docs

Mackey Guitar Tutor is an app for macOS and iPad (Rust core, Tauri shell) that helps beginner and intermediate guitarists learn the fretboard and play real songs. This folder holds the product requirements (PRD), architecture, design docs and phased plan. Order of authority: [INTENT.md](../INTENT.md), then the [PRD](prd/README.md), then the design docs. The fretboard foundation (Explore, Tuner, Settings) is built; the rest is designed and planned in the [roadmap](roadmap.md).

| Doc | What it covers |
|-----|----------------|
| [../INTENT.md](../INTENT.md) | Why the product exists, who it is for, principles, what success looks like |
| [prd/](prd/README.md) | **Product requirements in full detail:** user stories, functional requirements, edge cases, acceptance criteria, UX flows, NFRs, traceability |
| [images/](images/README.md) | Screenshots of the real app (Explore, Tuner, Settings) and a few of the mocked UI, with how each was made |
| [architecture.md](architecture.md) | Module layout, domain model, persistence, key algorithms |
| [practice-sessions.md](practice-sessions.md) | Practice session design and the quick-entry notation |
| [rhythm.md](rhythm.md) | Metronome, drum backing and the speed trainer |
| [learning-features.md](learning-features.md) | Tuner, scales overlay, left-handed and tunings, CAGED maps, theory tips, progress, ear training, looper, mic recognition |
| [platforms.md](platforms.md) | macOS and iPad targets, unsigned direct download, storage and iCloud later, latency, accessibility |
| [sound-assets.md](sound-assets.md) | Freely licensed drum kits and guitar tones, and how they are played |
| [song-import.md](song-import.md) | Finding, importing and parsing songs, plus legal/ToS constraints |
| [roadmap.md](roadmap.md) | Phased delivery plan with exit criteria |
| [adr/](adr/) | Architecture Decision Records (0001 to 0008: Rust core with a Tauri shell, theory core, persistence, song import, drums, on-device audio, the UI boundary, iPad via Tauri iOS) |

## Feature areas at a glance

1. **Fretboard explorer** (foundation). Pick a key and see notes, chords, triads and arpeggios on the neck. Cycle through scale degrees, chord qualities, inversions and positions.
2. **Practice sessions.** Type a short sequence such as `G > Em >> C > D` plus a method such as "one-minute changes" and the app drills it with the fretboard, a metronome and a timer.
3. **Songs.** Bring in a song's chords and lyrics (paste, ChordPro file, share from a browser, later licensed sources), parse it, and show the chart next to fretboard help. Any song can become a practice session in one click.
4. **Rhythm.** A metronome and drum backing that share one clock with the fretboard, plus a speed trainer that starts slow and steps up to the song's real tempo as the player masters it.

5. **Learning features.** A tuner, scale and mode overlays, left-handed mode and alternate tunings in v1; CAGED maps, theory tips, progress and streaks, ear training, a looper and microphone chord checking in later phases.

Areas 2 to 5 are built on the same music theory engine and fretboard view as area 1, so area 1 ships first. Area 4 is a shared service used by the other three.
