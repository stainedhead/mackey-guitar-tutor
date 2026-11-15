# Sound Assets

Status: Draft v0.1 (2026-11-15)

Rule: bundle only sounds under CC0, public domain, or permissive licences (MIT) that allow redistribution inside an app. Each asset and its licence is recorded in `assets/CREDITS.md`, even where attribution is not required. Each licence is checked again against its source before it is bundled.

## 1. Candidates

| Need | Candidate | Licence | Format | Notes |
|------|-----------|---------|--------|-------|
| Metronome clicks | Generated in code (sine and noise bursts, wood-block style) | None needed | Synthesized | No asset at all; the voice count ("1, 2, 3, 4") can be recorded by J or generated with the macOS voice once at build time |
| Guitar tones (nylon, steel, clean electric) | **FluidR3_GM** SoundFont, or its trimmed derivative **MuseScore_General** | MIT (keep the copyright and acknowledgment notices) | SF2 | Played by rustysynth. Extract only the guitar presets (GM programs 25 to 28) to keep the bundle small |
| Drum kit (main) | **SM Drums** | Public domain | SFZ + WAV | Multi-sampled acoustic kit, listed at [sfzinstruments.github.io/drums](https://sfzinstruments.github.io/drums) |
| Drum kit (alternatives) | **Unruly Drums**, **Big Rusty Drums**, **Swirly Drums** | CC0-1.0 | SFZ + WAV | Same source; good for style variety (rock vs brushes vs lighter kits) |
| Fallback GM drums | Drum bank of FluidR3_GM / MuseScore_General | MIT | SF2 | Lower quality, but a tiny single file |

Avoided: kits under CC-BY-SA (share-alike terms would attach to the bundle), "freemium" kits, and Freesound sounds unless each one is individually CC0.

## 2. How they are played

- **Guitar and fallback drums:** rustysynth renders SF2 presets inside the audio callback.
- **Drum kits:** SFZ multi-sample kits are not SF2, so `mackey-audio` gets a small one-shot sample player: velocity layers, round-robin alternation, choke groups (open and closed hi-hat), and per-voice gain. That is simpler and better sounding than converting the kits to SF2.
- Samples are decoded to memory at start-up, so the audio thread never touches the disk.

## 3. Size budget

Full kits are hundreds of megabytes. A build script trims each kit to one close-mic mix, 3 to 4 velocity layers and 2 round-robins, and encodes to FLAC. Target: under 40 MB for drums plus guitar in total, so the iPad install stays small.
