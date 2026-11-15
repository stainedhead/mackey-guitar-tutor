# 05. Songs

Phase: P3 and P4 (v1) for SG-1, SG-2 and SG-5 to SG-11; P5 for SG-3 and SG-4 (gated on legal review). Design: [song-import.md](../song-import.md), [architecture.md §3.4, §4.2 to §4.4](../architecture.md). Legal constraints: [song-import.md §2](../song-import.md#2-legal-and-terms-of-service-constraints-read-first), [ADR 0004](../adr/0004-user-initiated-song-import.md).

## 1. Purpose

Get a song the player wants to play into the app from text they already have, show a clean chart with fretboard help for every chord, and turn its hard parts into practice.

## 2. User stories

| ID | Story |
|----|-------|
| US-SG-1 | As a player, I paste a chord sheet I found and get a clean chart with chords above the right words. |
| US-SG-2 | As a player, I open a `.cho` file and it just works. |
| US-SG-3 | As a player, when the parser gets a chord wrong, I fix it in seconds before saving. |
| US-SG-4 | As a beginner, I tap a chord in the chart and see how to play it, with shapes that need the least hand movement through the song. |
| US-SG-5 | As a beginner, the song is in Bb and the app suggests capo 3 with G shapes. |
| US-SG-6 | As a player, the chart scrolls in time with drums at a slower tempo, and the neck shows the current and next chord. |
| US-SG-7 | As a player, one click gives me a practice session for this song's hardest changes. |
| US-SG-8 | As a player, I write my own song's chart. |

## 3. Song data

- SG-0.1 A song has: title (required, 1 to 200 characters; default from the first line or "Untitled song"), artist (optional), key (optional, detected), capo (0 to 12, default 0), tempo (optional, 30 to 250), meter (optional, default 4/4), tuning (optional, default the app's), sections, provenance, and its ChordPro text (the source of truth).
- SG-0.2 Provenance is one of: pasted, file (name only), written in app, web (URL and date; P5), provider (P5 and later). It is shown in the song's info panel.
- SG-0.3 Unknown ChordPro directives are preserved and round-tripped unchanged.

## 4. Import (v1)

### SG-1 Paste chords-over-lyrics text

- SG-1.1 "New song from text" (Cmd-Shift-V on Mac; "Paste" button on iPad) opens the **Import review** with the clipboard text; pasting into an empty library list does the same.
- SG-1.2 The text is classified line by line per [song-import.md §4.3](../song-import.md#43-chords-over-lyrics) into blank, section header, chord line, lyric line, tab block and inline label-plus-chords.
- SG-1.3 Chord lines are aligned to the following lyric line by column: a chord whose first character is at column *c* is anchored before the lyric character at column *c*. Tabs are expanded to 8-column stops before measuring; full-width characters count as 2 columns.
- SG-1.4 A chord line followed by a blank line, a section header or another chord line becomes an instrumental line (chords with no lyrics).
- SG-1.5 Lines before the first section header or chord line that look like metadata ("Title - Artist", "Capo 2", "Key: G", "Tempo: 96", "Tuning: Drop D") fill the matching fields and are removed from the body.
- SG-1.6 Page junk lines are removed when they match known patterns ("Print", "Transpose", "Rate this tab", "Tabbed by", "Difficulty:", URLs on their own line) and are listed in the review as "Removed lines" so the player can restore any of them.
- SG-1.7 Every token that looks like a chord but does not parse is kept as an **unparsed chord** (shown with a dashed amber outline), never dropped.

### SG-2 ChordPro

- SG-2.1 Open `.cho`, `.chopro`, `.chordpro`, `.crd`, `.pro` files by menu, drag and drop (Mac), the Files picker or "Open in Mackey" (iPad). `.txt` files are detected as ChordPro or chords-over-lyrics.
- SG-2.2 Supported directives: `title`/`t`, `subtitle`/`st`, `artist`, `key`, `capo`, `tempo`, `time`, `tuning` (Mackey extension, also `x_mackey_tuning`), `comment`/`c`, `start_of_chorus`/`soc`, `end_of_chorus`/`eoc`, `start_of_verse`/`sov`, `end_of_verse`/`eov`, `start_of_bridge`/`sob`, `end_of_bridge`/`eob`, `start_of_tab`/`sot`, `end_of_tab`/`eot`, `chorus` (repeat the last chorus), `define` (custom chord shape), and `x_mackey_*` extensions. Inline `[Chord]` anywhere in a lyric line.
- SG-2.3 `define` shapes become pinned voicings for that chord in this song.
- SG-2.4 Export: "Export ChordPro" writes the canonical ChordPro; "Copy as ChordPro" copies it; "Export library" writes every song and session to a folder.
- SG-2.5 Round trip: parse then export then parse gives an equal model (property test).

### SG-10 Import review and editing

- SG-10.1 The review shows the parsed chart on the left and the original text on the right, with linked highlighting (hover a chart line to highlight its source lines).
- SG-10.2 The player can: edit a chord (click it, type), move a chord along its lyric (drag on Mac; long-press and drag on iPad; `Alt-←/→` moves by one character), add a chord at a position, delete a chord, change a line's type (chord, lyric, header, tab), merge or split sections, rename sections, and edit metadata.
- SG-10.3 A summary bar shows: number of chords parsed, number unparsed (with "Next unparsed" button), detected key with confidence, and removed lines count.
- SG-10.4 "Save" stores the song; "Save" is allowed with unparsed chords (they are treated as "no chord" in playback and practice, and stay marked).
- SG-10.5 The same editor is used to edit a saved song later ("Edit chart") and to write a new song from scratch (SG-11).

### SG-11 Write a chart

- SG-11.1 "New blank song" opens the editor with an empty verse; the player types lyrics and inserts chords with `[` (Mac) or the chord palette (iPad), or types ChordPro directly in a source view toggle.

## 5. Chart view

### SG-5 Display

- SG-5.1 Lyrics with chords drawn above their anchor positions; chords never overlap: if two chords would collide, the lyric is padded with spaces.
- SG-5.2 Section labels in the margin; repeated choruses (ChordPro `chorus`) shown in full with a "Chorus (repeat)" label.
- SG-5.3 Chord palette at the top: every distinct chord in order of first appearance, each with a mini diagram of its chosen voicing (SG-6).
- SG-5.4 Toggles: lyrics on/off (chords-only lead sheet, one row of chords per line with bar separators where known), Roman numerals under chord names, capo shapes vs sounding chords, font size (5 steps), two columns (Mac, wide windows).
- SG-5.5 Tab blocks are shown as monospaced text and are not interpreted.

### SG-6 Fretboard help and voicing sets

- SG-6.1 Clicking or tapping a chord in the chart or palette selects it on the neck (G-3.2).
- SG-6.2 The song has a **voicing set**: one voicing per distinct chord, chosen by the voice-leading path solver ([architecture.md §4.2](../architecture.md#42-voice-leading-path-songs-and-practice)) over the chord sequence of the whole song.
- SG-6.3 Presets: **Beginner** (open voicings only when they exist, otherwise the easiest by score), **Smooth** (minimum total movement, any voicings), **One position** (all voicings inside a 5-fret window chosen by the player). Default Beginner.
- SG-6.4 The player can pin a voicing for a chord (click a palette diagram, cycle with `←`/`→`, then "Pin"); the solver re-runs with pinned chords fixed.
- SG-6.5 Each transition in the song gets a difficulty score (the solver's edge cost), used by SG-9.

### SG-7 Transpose, capo and key

- SG-7.1 Transpose up/down by semitones (`Cmd-↑`/`Cmd-↓` on Mac); chords are respelled for the new key (G-1). The original key is kept and "Reset" returns to it.
- SG-7.2 Key detection runs on import and after edits ([architecture.md §4.4](../architecture.md#44-key-detection)); shows the top key with confidence High (≥ 0.8), Medium (0.5 to 0.8) or Low (< 0.5). Low confidence shows "Key unclear" and the top two candidates; Roman numerals are hidden until the player picks one.
- SG-7.3 **Capo suggestion:** when the song's key has 3 or more flats or 4 or more sharps, or when any chord needs a barre in the Beginner preset, suggest the capo (1 to 7) that lets the most chords be played as open shapes, breaking ties by lower capo. Shown as "Try capo 3 with G shapes" with one-tap apply.
- SG-7.4 Capo display follows LH-5 (shape names with sounding names on toggle).

### SG-8 Play-along

- SG-8.1 Auto-scroll uses the transport. Tempo is the song tempo if set, else 80 bpm; the speed trainer may run with the song tempo as target.
- SG-8.2 Bar timing: each chord lasts until the next chord. Default duration is 1 bar per chord for chord lines whose chords have no explicit timing; the player can set beats per chord in the editor (for example 2 beats) per line or for the song. ChordPro bar lines `|` set chord durations by bar.
- SG-8.3 The current line is highlighted and kept at 1/3 of the view height; the current chord is highlighted; the neck shows the current chord solid and the next ghosted (as PS-6.2).
- SG-8.4 Drums follow sections per DR-6.
- SG-8.5 Loop a section: select a section and press `L` (or the loop button); the speed trainer's "every pass" rule counts loops of that section.
- SG-8.6 Unparsed chords are skipped silently in play-along; the neck shows "No chord" during them.

### SG-9 Practice this song

- SG-9.1 "Practice this song" creates a session named "<title>: practice" whose notation is generated from the song: each section becomes a labelled group with its chord sequence; consecutive identical sections collapse to `xN`.
- SG-9.2 The 3 transitions with the highest difficulty score (SG-6.5), with ties broken by how often they occur, are marked `>>`.
- SG-9.3 Voicings from the song's voicing set are carried as modifiers where they are not the default (`posN`, `caged:X`, `barre`).
- SG-9.4 The method is `@ladder <start>-<song tempo> +4 clean` when the song has a tempo (start = 60% of song tempo, rounded down to a multiple of 4, minimum 40), otherwise `@changes 60s` on the focus transitions.
- SG-9.5 The generated notation must parse without errors (property test over the fixture corpus).

## 6. Web import and search (P5, gated)

- SG-3 Import from a web page: the player drags selected text or a page URL onto the app, or uses "Import from clipboard", or (later) a browser extension or Share extension that opens `mackey://import`. The app processes the text locally exactly like SG-1. For a URL, the app fetches that one page once, extracts the largest monospaced or `<pre>` block, and shows the review; nothing is fetched in the background and nothing is re-fetched automatically.
- SG-3.1 Before the first web import, the app shows a one-time notice: "Song lyrics and chord sheets are owned by their creators. Import only for your own practice. Mackey keeps the song on this device only."
- SG-4 Search: a search field queries a web search API for "<query> chords"; results show title, site and snippet; opening a result opens the user's browser. Import happens only through SG-3.
- SG-4.1 *Gate:* SG-3 and SG-4 ship only after the legal review and distribution decision in [roadmap.md P5](../roadmap.md). Until then the menu items are not present.

## 7. Edge cases and error states

| ID | Situation | Required behaviour |
|----|-----------|--------------------|
| E-SG-1 | The word "A" or "Am" in a lyric line | Not a chord unless the line passes the chord-line rule with context (single-token lines need a lyric line after and a chord line or header before). |
| E-SG-2 | Chord line longer than its lyric line | Lyric padded with spaces; chords keep their columns. |
| E-SG-3 | Two chord lines in a row | First becomes an instrumental line. |
| E-SG-4 | Mixed ChordPro and chords-over-lyrics | Treated as ChordPro if the ChordPro rules match; chord-over-lyrics lines inside it are detected and offered "Convert these 6 lines?" |
| E-SG-5 | Text with no chords at all | Review shows "No chords found. Is this the right text?" Save is allowed (lyrics-only song). |
| E-SG-6 | Text over 2,000 lines or 200 KB | Rejected with "This text is too long for one song (limit 2,000 lines)." |
| E-SG-7 | Binary or non-UTF-8 file | Tries UTF-16 and Windows-1252; if still invalid, "Could not read this file as text." |
| E-SG-8 | Chord with unusual spelling (`Bbmaj7/D`, `C(add9)`, `G/B`, `N.C.`) | Parsed; `N.C.` is "no chord". |
| E-SG-9 | Chords written with H (German) | Option "Read H as B" in the review, auto-suggested when an `H` chord appears. |
| E-SG-10 | Song uses a tuning different from the app | On open, "This song uses Drop D. Switch tuning?" (Switch / Keep). |
| E-SG-11 | Duplicate import (same title and artist exists) | Ask "Replace, keep both, or cancel?" |
| E-SG-12 | Network failure during URL import (P5) | "Could not load the page. Paste the text instead?" with a paste button. |

## 8. Acceptance criteria

| ID | Given | When | Then |
|----|-------|------|------|
| AC-SG-1.1 | Fixture `basic-verse.txt` (chords over lyrics, 2 sections) | Paste | Chart matches the golden ChordPro snapshot exactly. |
| AC-SG-1.2 | Fixture corpus (at least 50 sheets written for the project or public domain) | Parse all | ≥ 98% of chords are anchored at the correct character; 0 chords dropped. |
| AC-SG-1.3 | Sheet with a header "Capo 2" | Paste | Capo field = 2; line removed from the body. |
| AC-SG-2.1 | Any ChordPro fixture | Parse, export, parse | Equal models (property test). |
| AC-SG-6.1 | Song G C D Em in Beginner preset, standard tuning | Voicing set | All four are the open shapes. |
| AC-SG-6.2 | Song with F and Bb, Beginner | Voicing set | F is the small F (xx3211) or the barre F (133211), whichever gives the lower total path cost; no voicing breaks FB-5.4. |
| AC-SG-7.1 | Song in Bb (Bb, Eb, F, Gm) | Capo suggestion | "Try capo 3 with G shapes" (G, C, D, Em). |
| AC-SG-7.2 | Song in G (G, C, D, Em), transpose +2 | Chart | Chords become A, D, E, F#m; key shows A major. |
| AC-SG-8.1 | Song at 90 bpm, 1 bar per chord | Play | Each chord change lands on a bar line; the neck changes at the same sample-time as the drums' downbeat (with latency offset). |
| AC-SG-9.1 | Song: verse G C G D, chorus Em C G D (x2) | Practice this song | Notation `key G; "Verse" (G > C > G > D) > "Chorus" (Em > C > G > D) x2 ...` with 3 `>>` on the highest-cost transitions; parses with no errors. |
| AC-SG-10.1 | Review with 2 unparsed chords | "Next unparsed" twice | Focus moves to each in order; fixing both clears the count. |

## 9. Out of scope

Tab playback, lyric synchronisation to recordings, chord detection from audio recordings of songs, sharing songs with other users, any automatic download of songs.
