# 01. Fretboard Explorer

Phase: P1 (v1). Design: [architecture.md §3.2, §4.1](../architecture.md), [learning-features.md §2, §3](../learning-features.md). Global rules: [README.md §5](README.md#5-global-rules).

## 1. Purpose

Let the player pick a key and see, hear and step through its notes, chords, triads, arpeggios and scales anywhere on the neck, in any supported tuning, right- or left-handed.

## 2. User stories

| ID | Story |
|----|-------|
| US-FB-1 | As a beginner, I pick G major and see every G major note on the neck, so I learn where the key lives. |
| US-FB-2 | As a beginner, I step through the chords of the key (I, ii, iii ...) and see the open shape of each, so I know what to play. |
| US-FB-3 | As an improver, I see a chord's triads on each string set in every inversion, so I can play it anywhere. |
| US-FB-4 | As an improver, I see an arpeggio inside one position with fingering, so I can practise it without moving my hand. |
| US-FB-5 | As a player, I overlay the minor pentatonic under the chord, so I can see safe notes for soloing. |
| US-FB-6 | As a left-handed player, I mirror the neck so it matches my guitar. |
| US-FB-7 | As a player in Drop D or DADGAD, I see shapes that are correct for my tuning. |
| US-FB-8 | As a player, I hear the chord or arpeggio I am looking at, so I can check my ear. |
| US-FB-9 | As a keyboard user, I do everything without the mouse. |

## 3. Functional requirements

### FB-1 Neck rendering

- FB-1.1 The neck MUST show 6 strings, frets 0 to the instrument's fret count (19 to 24, default 22), the nut, and inlays at frets 3, 5, 7, 9, 15, 17, 19, 21 (single) and 12, 24 (double).
- FB-1.2 Fret numbers MUST be shown below the neck (above when strings are flipped) at frets 3, 5, 7, 9, 12, 15, 17, 19, 21, 24, and at the first visible fret.
- FB-1.3 String labels (open-string note names for the current tuning and capo) MUST be shown at the nut side.
- FB-1.4 Fret spacing MUST follow the real 12th-root-of-2 ratio, with a minimum fret width of 28 pt on iPad and 24 px on Mac; if the visible range cannot fit at the minimum, the neck scrolls horizontally.
- FB-1.5 The visible range is set by dragging range handles or by "Fit selection", which zooms to show every highlighted note plus one fret either side.
- FB-1.6 Hover (Mac) or tap (iPad) on any fret position MUST show a tooltip with: note name, octave (scientific pitch, C4 = middle C), scale degree in the key, interval to the chord root if a chord is selected, and its role.
- FB-1.7 Clicking or tapping a position plays that note (see FB-8) unless sound is off.

### FB-2 Key and mode

- FB-2.1 The key picker MUST list tonics per G-1.2/G-1.3 and modes: Major (Ionian), Natural minor (Aeolian), Dorian, Phrygian, Lydian, Mixolydian, Locrian, Harmonic minor.
- FB-2.2 The key MUST be shown at all times in the toolbar as text, for example "G major" or "E Dorian".
- FB-2.3 Changing the key follows G-3.3.
- FB-2.4 Shortcuts: next/previous tonic around the circle of fifths (Mac: `]` / `[`; iPad keyboard the same), relative major/minor toggle (`R`).

### FB-3 Key notes view

- FB-3.1 With focus "none", the neck MUST show every note of the key at every fret position in the visible range, with the tonic as root shape and other degrees as chord-tone circles coloured by degree role (1, 3, 5, 7 coloured; 2, 4, 6 drawn as scale tones).
- FB-3.2 Labels follow G-2.2. Scale degree labels for modes are relative to the mode's tonic (E Dorian shows E as 1, G as b3).

### FB-4 Chord selection

- FB-4.1 A chord is selected by: (a) the diatonic chord strip (I to vii° as buttons with Roman numerals and chord names); (b) typing a chord symbol in the chord field; (c) typing a Roman numeral; (d) clicking a chord anywhere else in the app (song chart, practice preview).
- FB-4.2 The diatonic chord strip MUST show triads by default with a toggle for sevenths (Imaj7, ii7, iii7, IVmaj7, V7, vi7, viiø7 in major).
- FB-4.3 The chord field MUST accept every symbol in the chord-symbol grammar ([song-import.md §4.3](../song-import.md#43-chords-over-lyrics)): major, m, 7, maj7, m7, m7b5, dim, dim7, aug, sus2, sus4, 7sus4, 6, m6, 9, maj9, m9, add9, 11, 13, 5, slash bass, and their common spellings.
- FB-4.4 An unrecognised symbol MUST NOT clear the current selection. The field shows the error inline (E-FB-1).
- FB-4.5 A chord outside the key is allowed; its notes are spelled per G-1.5 and a badge says "Not in key" with its function where known (for example "bVII, borrowed from G minor").
- FB-4.6 "All tones" view: every occurrence of each chord tone in the visible range, roles per G-2.

### FB-5 Voicings

- FB-5.1 "Voicing" view MUST list voicings found by the voicing engine for the current tuning and capo, ordered by score, and show the selected one on the neck with fingers, open strings and mutes.
- FB-5.2 Voicings are grouped: **Open** (uses at least one open string, within frets 0 to 4), **Barre and CAGED** (named by shape: C, A, G, E, D), **Other**. Each list entry shows a mini chord diagram, its lowest fret and its name ("E-shape barre, fret 3").
- FB-5.3 The open shapes for these chords MUST match the standard teaching shapes exactly: C, A, G, E, D, Am, Em, Dm, A7, B7, C7, D7, E7, G7, Amaj7, Cmaj7, Dmaj7, Fmaj7, Am7, Dm7, Em7, Asus2, Asus4, Dsus2, Dsus4, Esus4, plus F (small 4-string and full barre). These come from the curated library, not search.
- FB-5.4 Every voicing shown MUST: contain the root and the third (or the sus note for sus chords; or root and fifth for power chords); have a fret span of at most 4 frets (5 when "Allow stretches" is on); need at most 4 fingers counting a barre as one finger; and have the required bass note for slash chords and inversions.
- FB-5.5 Fingering MUST be shown with finger numbers 1 to 4. Barres are drawn as a bar across the strings they cover.
- FB-5.6 If no voicing meets the constraints, the list shows "No playable voicing in this tuning" and offers "Allow stretches" and "Allow omitted fifth" (E-FB-3).

### FB-6 Triads

- FB-6.1 "Triad" view MUST show the chord's triad on a chosen string set (123, 234, 345, 456) in a chosen inversion (root, 1st, 2nd), using only three adjacent strings, one note per string.
- FB-6.2 For any string set the app MUST be able to show all three inversions in order up the neck at once ("Show all on string set"), each inversion labelled.
- FB-6.3 Seventh chords in triad view use the chord's root triad and show a note: "Triad of Cmaj7: C E G".
- FB-6.4 For tunings where adjacent strings are not a fourth or third apart, triads are still found by search; if a triad is not playable within a 4-fret span on that string set, that inversion shows as unavailable, not as a wrong shape.

### FB-7 Arpeggios

- FB-7.1 "Arpeggio" view MUST show every chord tone inside a chosen position window (start fret 0 to 20; window of 4 frets with one-fret stretches either side), with a suggested finger per note.
- FB-7.2 Fingering rule: one finger per fret in the window (index on the start fret), stretching with finger 1 or 4 for the extra frets.
- FB-7.3 The play order is shown as small numbers when "Show order" is on: lowest string upward, then down (direction: up, down, up-down).
- FB-7.4 The default window is the position that contains the selected voicing; otherwise the lowest position that contains the root on string 6 or 5.

### FB-8 Sound

- FB-8.1 "Play" (Space when the transport is stopped, or the Play button) plays the current selection: voicing as a strum (low to high, 30 ms between strings) or arpeggio as eighth notes at the current tempo; key notes as an ascending scale from the lowest tonic in the visible range.
- FB-8.2 Instrument sound: nylon, steel (default) or clean electric, from the bundled SoundFont ([sound-assets.md](../sound-assets.md)).
- FB-8.3 Sound MUST start within 30 ms of the action (see NF budgets).
- FB-8.4 A mute toggle silences selection playback without affecting the metronome.

### FB-9 Cycling

| Action | Mac key | iPad gesture / button | Behaviour |
|--------|---------|-----------------------|-----------|
| Next / previous chord in key | `↓` / `↑` | Swipe up / down on chord strip; ▲▼ buttons | Moves through I to vii and wraps |
| Next / previous voicing | `→` / `←` | Swipe left / right on the neck; ◀▶ buttons | Wraps within the current group, then into the next group |
| Next / previous inversion | `I` / `Shift-I` | Inversion segmented control | Root, 1st, 2nd (and 3rd for sevenths), wraps |
| Next / previous string set | `S` / `Shift-S` | String-set segmented control | 123, 234, 345, 456, wraps |
| Next / previous position | `P` / `Shift-P` | Two-finger swipe on the neck | Moves the window up or down to the next position containing the chord |
| Cycle view type | `V` | View segmented control | All tones, Voicing, Triad, Arpeggio |
| Cycle labels | `L` | Labels menu | Per G-2.2 |
| Toggle scale overlay | `O` | Overlay toggle | See SC-1 |
| Play | `Space` | Play button | FB-8.1 |

- FB-9.1 Each cycling action MUST update the neck within one frame (NF-1) and announce the new state to VoiceOver ("E minor, first inversion, strings 2 3 4").
- FB-9.2 The current position in each list is shown as "3 of 7".

### SC Scales and modes overlay

- SC-1 Overlay scales: major, natural minor, major pentatonic, minor pentatonic, blues (minor pentatonic plus b5), harmonic minor, and the 7 diatonic modes. Default overlay when first turned on: the key's own scale; for a minor or minor-7 chord in "Follow the chords" mode, that chord root's minor pentatonic.
- SC-2 Chord tones are drawn per G-2; scale tones that are not chord tones are drawn as hollow rings.
- SC-3 Position patterns: the five minor/major pentatonic boxes (box 1 has the root on string 6 at the first finger) and seven 3-notes-per-string major-mode patterns. Cycle with `P` / `Shift-P`. "Show all positions" shows the full neck.
- SC-4 "Follow the chords": in song and run mode, the overlay changes to fit each chord when the chord changes. The rule for which scale: if the chord is diatonic, use the key's scale; if not, use the chord root's mode that contains the chord (major chord: Mixolydian for dominant 7, Ionian otherwise; minor chord: Dorian; diminished: Locrian).
- SC-5 Labels follow G-2.2; scale degrees in overlay are relative to the overlay scale's tonic.

### LH Handedness, tunings and capo

- LH-1 "Left-handed" mirrors the neck horizontally (nut on the right). "Flip strings" puts string 6 at the top. Both are independent and saved.
- LH-2 Tuning presets: Standard (E A D G B E), Drop D (D A D G B E), Double drop D (D A D G B D), DADGAD, Open G (D G D G B D), Open D (D A D F# A D), Open E (E B E G# B E), Half-step down (Eb Ab Db Gb Bb Eb), Whole-step down (D G C F A D). Custom: each string set by note and octave within a minor third below to a whole tone above the standard pitch for that string; outside that range the app warns "Unusual tension, check your strings" but allows it.
- LH-3 Capo 0 to 12. Frets below the capo are dimmed and not selectable. A capo indicator is drawn across the neck.
- LH-4 Every voicing, triad, arpeggio and scale pattern MUST be computed for the current tuning and capo. The curated shape library exists for Standard, Drop D and Half-step down / Whole-step down (same shapes, transposed names); other tunings use search only, and their list shows "Found by search" on each voicing.
- LH-5 Chord names under a capo: by default the neck shows **shape names** (what the fingers play) with the **sounding name** in brackets, for example "G shape (sounds A)". A toggle swaps the emphasis.

## 4. Edge cases and error states

| ID | Situation | Required behaviour |
|----|-----------|--------------------|
| E-FB-1 | Player types an unknown chord symbol (`Hm7`, `C#maj13#11b9`) | Field turns red, message "Not a chord symbol I know: Hm7. Did you mean Bm7?" (suggest when edit distance ≤ 2 or H→B); selection unchanged. |
| E-FB-2 | Chord symbol valid but unusual (`Cmaj13#11`) | Accepted; all-tones view shows all tones; voicing list may be short; extensions above the 9th are allowed to omit the 5th and 11th as normal. |
| E-FB-3 | No playable voicing | FB-5.6. Never draw a voicing that breaks FB-5.4. |
| E-FB-4 | Capo above the selected position | Position moves to the first available position at or above the capo; notice "Moved above the capo". |
| E-FB-5 | Visible range smaller than the selected voicing | "Fit selection" is offered; highlighted notes outside the range are indicated by an arrow at the neck edge. |
| E-FB-6 | Custom tuning with two identical open strings | Allowed (some open tunings do this); voicings must not double-count those strings in fingering. |
| E-FB-7 | Key with 6 accidentals | Spelled per G-1.2; tooltip offers the enharmonic key. |
| E-FB-8 | Audio device unavailable | Visual features work; Play shows "No audio output" once and offers Settings. |
| E-FB-9 | Very narrow window (Mac < 700 px wide) or iPad Split View one-third | Neck scrolls; side panels collapse into a menu; the neck never shrinks below FB-1.4 minimums. |

## 5. Acceptance criteria

| ID | Given | When | Then |
|----|-------|------|------|
| AC-FB-2.1 | Key F major | Notes view | Labels show F G A Bb C D E; no A#. |
| AC-FB-2.2 | Key G major, IV chord selected | Key changed to D major | G chord is selected (IV of D). |
| AC-FB-4.1 | Key C | Type `D7` | All tones shows D, F#, A, C; badge "Not in key (V/V)". |
| AC-FB-5.1 | Standard tuning | Select C, voicing view | First voicing is the open C (x32010) with fingers x-3-2-0-1-0. |
| AC-FB-5.2 | Standard tuning | Select F, voicing view | List contains small F (xx3211) and full barre F (133211); both obey FB-5.4. |
| AC-FB-5.3 | Any tuning and chord | Voicing search runs | No voicing has span > 4 frets (allow stretches off) or more than 4 fingers. (Property test.) |
| AC-FB-6.1 | Key C, chord C, string set 234, 1st inversion | Triad view | Shows E (string 4 fret 2), G (string 3 fret 0), C (string 2 fret 1). |
| AC-FB-6.2 | Chord G, string set 123 | Show all on string set | Three inversions appear in fret order, labelled root, 1st, 2nd. |
| AC-FB-7.1 | Chord Am, position 5 | Arpeggio view | Shows every A, C and E between frets 4 and 9 inclusive, finger 1 on fret 5. |
| AC-FB-9.1 | Any selection | Each cycling key pressed 100 times | Every frame renders in under 16 ms (benchmark); VoiceOver announces each change. |
| AC-SC-3.1 | A minor pentatonic overlay | Box 1 | Notes at frets 5 and 8 on strings 6, 2 and 1, and frets 5 and 7 on strings 5, 4 and 3; roots (A) at string 6 fret 5, string 4 fret 7 and string 1 fret 5. |
| AC-SC-4.1 | Song in C, chord changes from C to A7 | Follow the chords on | Overlay changes from C major to A Mixolydian. |
| AC-LH-1.1 | Left-handed on | Any view | The nut is drawn on the right and the frets are mirrored; tooltips are unchanged. |
| AC-LH-4.1 | Drop D | Select D, voicing view | The list includes a D chord using all six strings with string 6 open (D); every note shown is the correct pitch for D A D G B E. |
| AC-LH-5.1 | Capo 2, chord shape G | Chord label | Reads "G shape (sounds A)". |

## 6. Out of scope for this feature

Scales beyond SC-1 (for example melodic minor, whole tone, bebop), partial capos, 7-string and bass, tablature output, chord-melody arrangements.
