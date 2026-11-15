# 04. Practice Sessions

Phase: P2 (v1). Design: [practice-sessions.md](../practice-sessions.md) (grammar in §2.2), [architecture.md §3.3](../architecture.md#33-practice-sessions). Depends on 01 (fretboard) and 03 (rhythm).

## 1. Purpose

Let the player describe what they need to drill for a song in a few seconds of typing (items, transitions, method, tempo and backing) and then practise it with the fretboard, clock and progress tracking.

## 2. User stories

| ID | Story |
|----|-------|
| US-PS-1 | As a player, I type `G > Em >> C > D` and press Play, and the neck shows each chord in time. |
| US-PS-2 | As a player, I mark Em to C as my hard change so the drill works on it more. |
| US-PS-3 | As a beginner, I run one-minute changes on my hard change and see my count go up over the week. |
| US-PS-4 | As an improver, I drill triads on the top strings with chosen inversions. |
| US-PS-5 | As a player, I use the speed trainer with drums to bring a progression up to tempo. |
| US-PS-6 | As a player who does not know the notation yet, I build a session by clicking, and watch the notation fill in. |
| US-PS-7 | As a teacher *(later)*, I copy a session as one line of text and send it. |

## 3. The session

- PS-0.1 A session has: name (1 to 80 characters, default "Untitled session" plus a number), notation source text, created and updated times, and derived parse result.
- PS-0.2 The notation source is the only stored definition (ADR 0003). Everything else is derived by parsing.
- PS-0.3 Sessions are listed in the sidebar sorted by last practised, then last edited; searchable by name and by chord content.

## 4. Quick-entry notation (behaviour)

The grammar is defined in [practice-sessions.md §2.2](../practice-sessions.md#22-grammar-ebnf). This section fixes the meaning.

### PS-1 Header

- PS-1.1 `key <tonic> [mode]` sets the key; mode words: `major` (default), `minor`, `dorian`, `phrygian`, `lydian`, `mixolydian`, `locrian`, `harmonic minor`; also `m` after the tonic (`key Em`).
- PS-1.2 Without a header key, the key is inferred from the items (same algorithm as song key detection) and shown as "Key: G major (detected)". Roman numerals then require a key and give error E-PS-3.
- PS-1.3 Settings: `capo N` (0 to 12), `tempo N` (30 to 250), `time N/M` (meters in RH-2), `tuning <preset-name>` (`standard`, `dropd`, `ddropd`, `dadgad`, `openg`, `opend`, `opene`, `halfdown`, `wholedown`). Header settings override the app defaults while the session runs and are restored after.

### PS-2 Items

- PS-2.1 **Chord:** any chord symbol (FB-4.3). Shown as its best voicing (open first for beginners) unless modifiers say otherwise.
- PS-2.2 **Roman numeral:** `I` to `VII` and `i` to `vii`, with optional `b`/`#` prefix, quality suffixes (`°`, `o`, `ø`, `+`, `7`, `maj7`, `m7`) and secondary dominants (`V/V`). Upper case means major, lower case minor, unless a suffix says otherwise. Resolved in the key (`vi` in G is Em).
- PS-2.3 **Triad:** `<chord>:tri`. Modifiers `str123 | str234 | str345 | str456` (default: the previous triad's string set; `str123` for the first triad) and `inv0 | inv1 | inv2` (default: root position for the first item; after that, the inversion on the same string set with the lowest voice-leading cost from the previous triad, where cost is the sum over the three strings of the absolute fret distance; ties go to the lower position).
- PS-2.4 **Arpeggio:** `<chord>:arp`. Modifiers `posN` (start fret, default: position of the previous item or the chord's lowest position), `up | down | updown` (default up), and subdivision from the method (default 8ths).
- PS-2.5 **Note:** `n:<note>` for a single pitch class (all occurrences are shown) or `n:<note><octave>` for one pitch (for example `n:A2`).
- PS-2.6 **Voicing modifiers on chords:** `open`, `barre`, `caged:C|A|G|E|D`, `posN`. If the requested voicing does not exist, a warning is shown and the best voicing is used.
- PS-2.7 **Duration:** default 1 bar per item. `2bars`, `1bar`, `2b` (beats) override. A duration shorter than 1 beat is an error.
- PS-2.8 **Labels:** `"Chorus"` before a step or group labels it in the preview and run mode.

### PS-3 Transitions, groups and repeats

- PS-3.1 `>` is a normal transition. `>>` is a focus transition from the previous item to the next. `|` is a bar separator and behaves like `>`.
- PS-3.2 `( ... )` groups; `xN` (N 1 to 32) repeats the step or group. The whole sequence repeats according to the method.
- PS-3.3 Repeats are expanded for running; the expanded length must be 1 to 64 items (G-5), otherwise E-PS-6.
- PS-3.4 Transitions inside a repeat include the wrap-around (in `(G > C) x4`, C to G is a transition, and it is a focus transition only if marked `(G > C >>) x4`, which means "C to G is focus").
- PS-3.5 Transition identity for history is the ordered chord-name pair, for example `Em>C`, so the same change is tracked across sessions and songs.

### PS-4 Methods

| Method | Notation | Parameters (defaults) | Run behaviour |
|--------|----------|-----------------------|---------------|
| Loop (default) | `@loop` `@loop N` | N passes, default unlimited | Play the sequence in time and repeat. |
| One-minute changes | `@changes` `@changes 60s` | Duration 30 to 300 s (60) | For each transition (focus first, then the others in order, each once), alternate between the two items for the duration, with click at the tempo. Player taps Return on each clean change; count shown live. |
| Speed trainer | `@ladder 60-96 +4 [pass\|clean\|8bars] [-4 miss]` | per ST-1 | Loop with tempo changes per the speed trainer. |
| Random | `@random` | none | Items in random order; no item repeats immediately; focus transitions occur twice as often. Next item shown one bar early. |
| Name the note | `@name [show\|find] [random]` | `show` (app shows a fret, player names it) | `show`: a position lights; player types the note name or taps it on a note keypad; `find`: app names a note, player taps every place on the neck. 20 questions per round. |
| Strum pattern | `@strum D-DU-UDU` | Pattern over one bar of 8ths (D = down, U = up, - = rest, x = muted strum) | Pattern drawn above the beat grid; current stroke highlighted. |

- PS-4.1 Backing (any method): `click` (default), `click 8ths|16ths|triplets|swing`, `gap:N`, `drums:<style>[/<variation>]`, `fills:N`, `nodrums`, `noclick`. Style names are the DR-1 list in lower case without spaces (`rock`, `pop`, `ballad`, `folk`, `country`, `shuffle`, `funk`, `reggae`, `bossa`, `waltz`, `sixeight`, `halftime`).
- PS-4.2 Tempo in the method line (`80bpm`) overrides the header tempo.

### PS-5 Editor

- PS-5.1 One-line field (wraps visually; Enter does not insert a newline, it starts the run when the parse has no errors).
- PS-5.2 Live preview under the field, updated within 50 ms of each keystroke (debounced 30 ms): a row of chips (one per item before repeat expansion; repeat count shown as a badge), transition arrows (focus arrows thick and coloured), the method and backing summary, and the expanded length.
- PS-5.3 Moving the text cursor over an item or clicking its chip shows that item on the neck.
- PS-5.4 Errors: the faulty token is underlined (red for error, amber for warning) with a one-line message under the field; up to 3 messages shown, "+N more" expands. The parser recovers at the next `>`, `>>`, `|`, `)` or `@`; everything parseable is previewed.
- PS-5.5 Autocomplete (Tab accepts, Esc dismisses): chord qualities after a root, modifiers after `:`, method names after `@`, style names after `drums:`.
- PS-5.6 **Builder**: a form with the key's diatonic chord palette, an "Add" for triad/arpeggio/note, a focus toggle on each arrow, method picker and backing picker. Every change writes notation into the field; the field remains the source of truth, and editing the text updates the builder.
- PS-5.7 "Copy as text" copies the notation; pasting notation anywhere a session can be created creates one.

### PS-6 Run mode

- PS-6.1 Enter run mode with Enter in the editor, the Run button, or Space when the session is selected.
- PS-6.2 Screen: the neck shows the **current** item in full colour, the **next** item as ghosts (G-2), and anchors where a finger stays on the same string and fret between the two. Above the neck: current and next chips, the item label, bar:beat, timer, method status (for example "Changes: 23 in 0:41"), and speed-trainer progress when active.
- PS-6.3 The next item is ghosted for the whole of the current item when its duration is 1 bar or less, otherwise for its last bar.
- PS-6.4 Item changes happen exactly on bar lines of the transport (TR-1), with visuals per RH-8.
- PS-6.5 Controls: Space pause/resume, Return clean, Backspace miss, `←`/`→` previous/next item (paused only), `R` restart, `-`/`=` tempo, `D` drums on/off, Esc end run.
- PS-6.6 Ending a run (Esc, method completes, or 60 minutes with no input) shows the **Review** panel: duration, passes, tempo start and end, last clean tempo, per-transition counts for changes, and a 1 to 5 self-rating (optional). "Again" restarts; "Again +step" restarts one speed-trainer step higher; "Done" saves and closes.

### PS-7 History

- PS-7.1 Every run longer than 20 seconds is saved: session id, start time, duration, method, start/end/clean tempos, per-transition clean-change counts (changes method), passes, rating.
- PS-7.2 The session's detail view shows: last 10 runs, a chart of clean tempo over time, and for each transition the best and the latest changes per minute.
- PS-7.3 Editing the notation keeps history (PS-3.5 keys by chord pair). Deleting a session keeps its runs in Recently deleted for 30 days (G-6.2).

### PS-8 Sessions from elsewhere

- PS-8.1 From the explorer: "Practice this" on the current selection writes notation, for example the triad cycle on string set 123 for C becomes `key C; C:tri str123 inv0 > C:tri str123 inv1 > C:tri str123 inv2 @loop`.
- PS-8.2 From a song: see SG-9 in [05-songs.md](05-songs.md).

## 5. Edge cases and error states

| ID | Input or situation | Severity | Message (exact wording) and behaviour |
|----|--------------------|----------|----------------------------------------|
| E-PS-1 | Unknown chord `Hm7` | Error | "Unknown chord: Hm7 (did you mean Bm7?)". Item skipped in preview; run blocked until fixed or removed. |
| E-PS-2 | Ambiguous `EM` | Warning | "EM read as E major. Write Em for minor or Emaj7 for major seventh." |
| E-PS-3 | Roman numeral with no key | Error | "Add a key to use Roman numerals, for example: key G;" |
| E-PS-4 | Unknown modifier `str124` | Error | "String sets are str123, str234, str345 or str456." |
| E-PS-5 | Unknown method `@chnges` | Error | "Unknown method: chnges (did you mean changes?)". |
| E-PS-6 | Expanded length over 64 | Error | "That is 96 items after repeats. The limit is 64." |
| E-PS-7 | Tempo out of range | Warning | Clamped; "Tempo limited to 30 to 250 bpm." |
| E-PS-8 | `>>` at the start or end | Warning | "Focus arrow needs an item on both sides." Ignored. |
| E-PS-9 | Empty notation | Info | Preview shows "Type chords like G > C > D, or use the builder." Run disabled. |
| E-PS-10 | `@changes` with no transitions (single item) | Error | "One-minute changes needs at least two items." |
| E-PS-11 | Requested voicing not available in the tuning | Warning | "No open C in DADGAD; using the best voicing." |
| E-PS-12 | Drum style meter conflicts with `time` | Warning | "waltz is in 3/4; the session is 4/4. Using the click instead." |
| E-PS-13 | Notation over 1,000 characters | Error | Input stops accepting characters; message "Sessions are limited to 1,000 characters." |

## 6. Acceptance criteria

The examples in [practice-sessions.md §6](../practice-sessions.md#6-acceptance-examples) are acceptance tests. In addition:

| ID | Given | When | Then |
|----|-------|------|------|
| AC-PS-1.1 | Editor empty | Type `key G; G > Em >> C > D x4 @changes 60s` | Preview: 4 chips, D with x4 badge, thick arrow Em→C, "One-minute changes, 60 s", no errors; parse under 50 ms. |
| AC-PS-2.1 | `key C; C:tri str123 > F:tri str123 > G:tri str123` | Parse | C root position (strings 3-2-1 at frets 5-5-3), F second inversion (5-6-5), G first inversion (4-3-3, chosen over 7-8-7 by the tie-break). |
| AC-PS-3.1 | `(G > C) x4 @loop 1` | Run | 8 items played, 8 bars, transitions G>C and C>G each counted 4 times. |
| AC-PS-4.1 | `G >> C @changes 30s` | Run | Two transitions run (G>C focus first, then C>G), 30 s each, with a running count. |
| AC-PS-5.1 | Type `G > Hm7 > C` | Preview | G and C chips, an error under Hm7 with the E-PS-1 message, run blocked. |
| AC-PS-5.2 | Builder: click I, IV, V in G, mark IV→V focus | Field | Contains `key G; G > C >> D` (header added because the builder knows the key). |
| AC-PS-6.1 | Running `G > C`, 80 bpm, 4/4 | Bar 1 beat 1 | Neck shows G solid and C ghosted; at bar 2 beat 1 (sample-accurate, with latency offset) C becomes solid. |
| AC-PS-7.1 | A 2-minute run of `@changes` | Done | History shows one run with counts per transition; a second session containing `Em>C` shows the same transition history. |
| AC-PS-8.1 | Explorer: Am arpeggio at position 5 | Practice this | Session notation `key Am; Am:arp pos5 up @loop` is created and parses without errors. |
