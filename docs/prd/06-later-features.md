# 06. Later Learning Features

Phases: P6 to P8. Design: [learning-features.md §4 to §9](../learning-features.md). These requirements are complete enough to estimate and design against; each is reviewed again, with exact wording and thresholds confirmed, at the start of its phase.

## CG CAGED and position maps (P6)

| ID | Requirement |
|----|-------------|
| CG-1 | For any major or minor triad chord, "CAGED map" MUST show all five shapes for that chord across the neck at once, each in its own colour-blind-safe colour with its letter drawn on the shape, overlapping notes shared. |
| CG-2 | Step through shapes in neck order (`P` / `Shift-P`); the transition animates the fingers that move (or cross-fades with Reduce Motion). |
| CG-3 | Each shape links to its arpeggio (FB-7) and its pentatonic box (SC-3): "Show arpeggio" and "Show box" switch the view while keeping the window. |
| CG-4 | Position map: for a chosen 4-fret window in the current key, list every diatonic chord, triad and arpeggio playable inside it, with a small diagram; clicking selects it. |
| CG-5 | "Practice this" writes notation, for example `key G; G:chord caged:E > G:chord caged:D > G:chord caged:C > G:chord caged:A > G:chord caged:G @loop`. |
| AC-CG-1 | Given G major, the map shows: E shape barred at fret 3; D shape at frets 5 to 8 (root on string 4, fret 5); C shape at frets 7 to 10 (root on string 5, fret 10); A shape barred at fret 10 (root on string 5, fret 10); G shape at frets 12 to 15 (root on string 6, fret 15), plus the open G shape (golden test). |

## TT Theory tips (P6)

| ID | Requirement |
|----|-------------|
| TT-1 | For the selected chord in a key, the tips panel MUST show: Roman numeral, function (tonic, predominant, dominant), and "built from" (which scale degrees make it), in at most two sentences. Example: "D is the V chord in G: built from the 5th, 7th and 2nd notes of G major. It pulls back to G." |
| TT-2 | Non-diatonic chords: explain secondary dominants (V/x), borrowed chords (bIII, bVI, bVII, iv in major) and the dominant in minor (V in a minor key). Anything not covered says "Outside the key" without guessing. |
| TT-3 | Substitutions: relative major/minor, adding 7th/sus/add9 colours, and (Improver setting) tritone substitution for dominant chords, each with "Hear it" (plays it in place) and "See it". |
| TT-4 | Tips never block: a collapsible side panel, on by default, with a "Hide tips" setting. Text is generated from rules, not a network service. |
| AC-TT-1 | Given key C and chord A7, the tip reads that A7 is V/ii (secondary dominant leading to Dm). |

## PR Progress, streaks and suggestions (P6)

| ID | Requirement |
|----|-------------|
| PR-1 | Dashboard: practice minutes per day (last 30 days) and per week (last 12 weeks), current streak, total sessions and songs practised. Practice minutes count only time the transport is playing in run mode or song play-along. |
| PR-2 | Trends: per transition, changes per minute over time (from `@changes`); per session and song, clean tempo over time. |
| PR-3 | Suggestions (up to 3 on the home screen): ranked by (a) transitions with the lowest changes per minute relative to the player's median, (b) songs whose last clean tempo is below 80% of the song tempo, (c) sessions not practised in 7 days. Each suggestion opens a ready-to-run session. |
| PR-4 | Streak: a day counts when at least 5 practice minutes are logged. The streak survives up to 2 missed days in any 7 if the weekly goal (default 4 days) is met. |
| PR-5 | All data local; "Export practice data (CSV)" with one row per run. |

## ET Ear training (P7)

| ID | Requirement |
|----|-------------|
| ET-1 | Interval drill: plays two notes (melodic, then optionally harmonic); the player names the interval from buttons (m2 to P8) or taps the second note on the neck given the first. Interval set and direction are configurable. |
| ET-2 | Chord-quality drill: major, minor, dominant 7, major 7, minor 7, sus4, diminished; player picks the quality. |
| ET-3 | Progression drill: in a chosen key, plays a 4-chord progression from diatonic chords; player picks each Roman numeral. |
| ET-4 | "Play it back": the app plays a note or chord and the player plays it on the guitar; graded by MR (P8) or by tapping the neck before P8. |
| ET-5 | Rounds of 10 or 20; difficulty steps up after 80% correct in two rounds and down after under 50%; results stored for PR. |
| ET-6 | Notation: `@ear intervals M3 P5 m7`, `@ear chords maj min 7`, `@ear progressions`. |

## LP Looper and record yourself (P7)

| ID | Requirement |
|----|-------------|
| LP-1 | Record from the input device over the backing with a count-in; the take is aligned to the transport using measured round-trip latency (calibrated once per device with a loopback or clap test). |
| LP-2 | Play back the take with or without backing; loop any bar range; adjust take volume. |
| LP-3 | Timing view: detected onsets drawn on the beat grid with early/late in ms; a summary "Average 18 ms late, spread 25 ms". |
| LP-4 | Layering: record a loop of N bars, then keep it playing while practising over it; up to 4 layers. |
| LP-5 | Takes saved locally (FLAC) against the song or session; export as WAV or M4A. Storage use is shown and old takes can be bulk-deleted. |

## MR Microphone recognition (P8)

| ID | Requirement |
|----|-------------|
| MR-1 | In run mode and play-along, "Listen" grades each expected chord or note: green check (match), amber "close" (some strings wrong; which ones, where detectable), red (no match), within 300 ms of the chord's start. |
| MR-2 | In `@changes`, clean changes are counted automatically when MR-1 matches within the first beat after the change; the player can still correct the count. |
| MR-3 | Calibration step per input device: noise floor and level. |
| MR-4 | Never required: every feature works without the microphone. |
| AC-MR-1 | On the recorded test set (acoustic and electric, at least 20 chords × 10 takes), at least 90% correct verdicts and under 5% false "match" verdicts. |
