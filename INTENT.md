# INTENT

This file says why Mackey Guitar Tutor exists and what "good" means for it. Read it before writing a requirement, a design or a line of code. When a decision is unclear, choose the option that best serves the intent below. If the intent itself needs to change, change this file first, in its own pull request.

## Purpose

Help a guitarist go from "I know a few open chords" to "I can find what I need anywhere on the neck and play the songs I love in time."

Most beginners learn shapes by rote and get stuck in the first few frets. They do not know *where* the notes of a chord, triad or arpeggio live, *why* those chords belong together in a key, or *how* to practise a hard change until it is clean. Mackey answers those three questions in one place: it shows the notes on the neck, it explains the harmony in plain language, and it turns what the player needs for a real song into a focused, timed practice routine.

## Who it is for

1. **J, first.** For now this is a personal tool. J's own practice is the first and most important test of every feature.
2. **Beginners** (up to about a year of playing) who want to play real songs and change chords cleanly.
3. **Improvers** (one to three years) who want to leave the open position, learn triads, arpeggios and CAGED shapes, and understand why chords fit a key.
4. **Teachers**, later, who want to hand a student a short, exact routine.

It is not, for now, for professional session players, for other instruments, or for an audience that has to be won in an app store.

## Guiding principles

1. **Show, don't tell.** The fretboard is the centre of every screen. Every chord, scale, triad or arpeggio the app talks about can be seen on the neck, and heard, with one action.
2. **Correct music theory, always.** Notes are spelled correctly for the key (Bb in F, not A#). A wrong note, a wrong name or an unplayable fingering is a top-severity bug, because it breaks the learner's trust.
3. **Fast to start practising.** A routine can be typed and running in under 10 seconds. The notation is short, forgiving and readable; nothing typed is ever lost because of a typo.
4. **Practise in time.** Rhythm is not an add-on. The metronome, drums, fretboard and song chart share one sample-accurate clock, and the player can always start slow and step up to full tempo.
5. **Beginner-first, never beginner-only.** Defaults suit a beginner (open shapes, slow tempo, note names). Depth (inversions, modes, CAGED, theory) is one step away, never in the way.
6. **Respect people's work and the law.** Song lyrics and chord sheets belong to their owners. Content enters only by the player's own action and stays on the player's devices. No scraping, no redistribution.
7. **Private and offline.** Everything works without a network or an account. Audio and practice data never leave the device unless the player chooses to move them.
8. **Text is the source of truth.** Practice sessions are notation text and songs are ChordPro, so the player's library is readable, portable and safe from format changes.
9. **One codebase, honest platforms.** A Rust core with a Tauri shell, built for macOS and iPad now. We do not claim support for a platform we have not built and tested.
10. **Accessible by default.** Colour-blind-safe colours with shape cues, full keyboard control, and VoiceOver labels on every fretboard position.
11. **Tested where it matters most.** The theory engine and parsers carry the most tests (property tests and golden files), because they are where mistakes hurt most and are easiest to catch.

## What success looks like

**For the player (J first), after about three months of use:**

- Can name and find every note of a chosen key across the whole neck, not just the first five frets.
- Can play the I, IV, V and vi chords of a common key in at least two positions, including triads on the top three strings.
- Has taken at least five real songs from a pasted chord sheet to a clean play-through at the song's tempo, using the speed trainer.
- Practises in short, focused sessions, and the hardest chord changes in those songs get measurably faster (changes per minute go up).

**For the product:**

- A session can be typed and running in under 10 seconds; a pasted chord sheet becomes a correct chart in under a minute, including any fixes.
- Zero known wrong-note or wrong-spelling bugs in a release.
- Clicks, drums and fretboard changes stay in sync at every tempo; the player never notices a drift.
- The app works the same on the Mac and on the iPad, offline.
- The design docs, the PRD and the code agree: every requirement traces to a design and a test.

## What it is not

- Not a tab or lyrics site, and not a store for other people's content.
- Not a recording studio or a full drum machine. Backing exists to help the player keep time.
- Not a social network. Sharing, if it comes, means sharing a text file.
- Not a replacement for a teacher; it is the practice partner between lessons.

## Where the detail lives

- [docs/prd/](docs/prd/README.md): the product requirements, in full detail. What the app must do.
- [docs/](docs/README.md): architecture, design docs and ADRs. How it will do it.
- [docs/roadmap.md](docs/roadmap.md): the order in which it gets built.
