//! AC-P0-1: every chord quality spelled correctly from every one of the 12 root spellings the
//! key pickers offer (C, Db, D, Eb, E, F, F#, G, Ab, A, Bb, B). 21 qualities x 12 roots = 252
//! rows.
//!
//! The expected spellings were produced by a separate letter-and-semitone calculation (not by
//! this crate) and the doubly altered rows and the famous ones (D7 = D F# A C, Bbmaj7 = Bb D F
//! A, G#dim7 = G# B D F) were checked by hand against theory. A row is (root, symbol suffix,
//! tones).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]

use mackey_theory::{Chord, ChordQuality, Note, parse_chord_symbol};

/// (root, symbol suffix, expected spelling of the chord tones from the root up)
const CASES: &[(&str, &str, &[&str])] = &[
    // maj
    ("C", "", &["C", "E", "G"]),
    ("Db", "", &["Db", "F", "Ab"]),
    ("D", "", &["D", "F#", "A"]),
    ("Eb", "", &["Eb", "G", "Bb"]),
    ("E", "", &["E", "G#", "B"]),
    ("F", "", &["F", "A", "C"]),
    ("F#", "", &["F#", "A#", "C#"]),
    ("G", "", &["G", "B", "D"]),
    ("Ab", "", &["Ab", "C", "Eb"]),
    ("A", "", &["A", "C#", "E"]),
    ("Bb", "", &["Bb", "D", "F"]),
    ("B", "", &["B", "D#", "F#"]),
    // min
    ("C", "m", &["C", "Eb", "G"]),
    ("Db", "m", &["Db", "Fb", "Ab"]),
    ("D", "m", &["D", "F", "A"]),
    ("Eb", "m", &["Eb", "Gb", "Bb"]),
    ("E", "m", &["E", "G", "B"]),
    ("F", "m", &["F", "Ab", "C"]),
    ("F#", "m", &["F#", "A", "C#"]),
    ("G", "m", &["G", "Bb", "D"]),
    ("Ab", "m", &["Ab", "Cb", "Eb"]),
    ("A", "m", &["A", "C", "E"]),
    ("Bb", "m", &["Bb", "Db", "F"]),
    ("B", "m", &["B", "D", "F#"]),
    // dom7
    ("C", "7", &["C", "E", "G", "Bb"]),
    ("Db", "7", &["Db", "F", "Ab", "Cb"]),
    ("D", "7", &["D", "F#", "A", "C"]),
    ("Eb", "7", &["Eb", "G", "Bb", "Db"]),
    ("E", "7", &["E", "G#", "B", "D"]),
    ("F", "7", &["F", "A", "C", "Eb"]),
    ("F#", "7", &["F#", "A#", "C#", "E"]),
    ("G", "7", &["G", "B", "D", "F"]),
    ("Ab", "7", &["Ab", "C", "Eb", "Gb"]),
    ("A", "7", &["A", "C#", "E", "G"]),
    ("Bb", "7", &["Bb", "D", "F", "Ab"]),
    ("B", "7", &["B", "D#", "F#", "A"]),
    // maj7
    ("C", "maj7", &["C", "E", "G", "B"]),
    ("Db", "maj7", &["Db", "F", "Ab", "C"]),
    ("D", "maj7", &["D", "F#", "A", "C#"]),
    ("Eb", "maj7", &["Eb", "G", "Bb", "D"]),
    ("E", "maj7", &["E", "G#", "B", "D#"]),
    ("F", "maj7", &["F", "A", "C", "E"]),
    ("F#", "maj7", &["F#", "A#", "C#", "E#"]),
    ("G", "maj7", &["G", "B", "D", "F#"]),
    ("Ab", "maj7", &["Ab", "C", "Eb", "G"]),
    ("A", "maj7", &["A", "C#", "E", "G#"]),
    ("Bb", "maj7", &["Bb", "D", "F", "A"]),
    ("B", "maj7", &["B", "D#", "F#", "A#"]),
    // min7
    ("C", "m7", &["C", "Eb", "G", "Bb"]),
    ("Db", "m7", &["Db", "Fb", "Ab", "Cb"]),
    ("D", "m7", &["D", "F", "A", "C"]),
    ("Eb", "m7", &["Eb", "Gb", "Bb", "Db"]),
    ("E", "m7", &["E", "G", "B", "D"]),
    ("F", "m7", &["F", "Ab", "C", "Eb"]),
    ("F#", "m7", &["F#", "A", "C#", "E"]),
    ("G", "m7", &["G", "Bb", "D", "F"]),
    ("Ab", "m7", &["Ab", "Cb", "Eb", "Gb"]),
    ("A", "m7", &["A", "C", "E", "G"]),
    ("Bb", "m7", &["Bb", "Db", "F", "Ab"]),
    ("B", "m7", &["B", "D", "F#", "A"]),
    // half-dim
    ("C", "m7b5", &["C", "Eb", "Gb", "Bb"]),
    ("Db", "m7b5", &["Db", "Fb", "Abb", "Cb"]),
    ("D", "m7b5", &["D", "F", "Ab", "C"]),
    ("Eb", "m7b5", &["Eb", "Gb", "Bbb", "Db"]),
    ("E", "m7b5", &["E", "G", "Bb", "D"]),
    ("F", "m7b5", &["F", "Ab", "Cb", "Eb"]),
    ("F#", "m7b5", &["F#", "A", "C", "E"]),
    ("G", "m7b5", &["G", "Bb", "Db", "F"]),
    ("Ab", "m7b5", &["Ab", "Cb", "Ebb", "Gb"]),
    ("A", "m7b5", &["A", "C", "Eb", "G"]),
    ("Bb", "m7b5", &["Bb", "Db", "Fb", "Ab"]),
    ("B", "m7b5", &["B", "D", "F", "A"]),
    // dim
    ("C", "dim", &["C", "Eb", "Gb"]),
    ("Db", "dim", &["Db", "Fb", "Abb"]),
    ("D", "dim", &["D", "F", "Ab"]),
    ("Eb", "dim", &["Eb", "Gb", "Bbb"]),
    ("E", "dim", &["E", "G", "Bb"]),
    ("F", "dim", &["F", "Ab", "Cb"]),
    ("F#", "dim", &["F#", "A", "C"]),
    ("G", "dim", &["G", "Bb", "Db"]),
    ("Ab", "dim", &["Ab", "Cb", "Ebb"]),
    ("A", "dim", &["A", "C", "Eb"]),
    ("Bb", "dim", &["Bb", "Db", "Fb"]),
    ("B", "dim", &["B", "D", "F"]),
    // dim7
    ("C", "dim7", &["C", "Eb", "Gb", "Bbb"]),
    ("Db", "dim7", &["Db", "Fb", "Abb", "Cbb"]),
    ("D", "dim7", &["D", "F", "Ab", "Cb"]),
    ("Eb", "dim7", &["Eb", "Gb", "Bbb", "Dbb"]),
    ("E", "dim7", &["E", "G", "Bb", "Db"]),
    ("F", "dim7", &["F", "Ab", "Cb", "Ebb"]),
    ("F#", "dim7", &["F#", "A", "C", "Eb"]),
    ("G", "dim7", &["G", "Bb", "Db", "Fb"]),
    ("Ab", "dim7", &["Ab", "Cb", "Ebb", "Gbb"]),
    ("A", "dim7", &["A", "C", "Eb", "Gb"]),
    ("Bb", "dim7", &["Bb", "Db", "Fb", "Abb"]),
    ("B", "dim7", &["B", "D", "F", "Ab"]),
    // aug
    ("C", "aug", &["C", "E", "G#"]),
    ("Db", "aug", &["Db", "F", "A"]),
    ("D", "aug", &["D", "F#", "A#"]),
    ("Eb", "aug", &["Eb", "G", "B"]),
    ("E", "aug", &["E", "G#", "B#"]),
    ("F", "aug", &["F", "A", "C#"]),
    ("F#", "aug", &["F#", "A#", "C##"]),
    ("G", "aug", &["G", "B", "D#"]),
    ("Ab", "aug", &["Ab", "C", "E"]),
    ("A", "aug", &["A", "C#", "E#"]),
    ("Bb", "aug", &["Bb", "D", "F#"]),
    ("B", "aug", &["B", "D#", "F##"]),
    // sus2
    ("C", "sus2", &["C", "D", "G"]),
    ("Db", "sus2", &["Db", "Eb", "Ab"]),
    ("D", "sus2", &["D", "E", "A"]),
    ("Eb", "sus2", &["Eb", "F", "Bb"]),
    ("E", "sus2", &["E", "F#", "B"]),
    ("F", "sus2", &["F", "G", "C"]),
    ("F#", "sus2", &["F#", "G#", "C#"]),
    ("G", "sus2", &["G", "A", "D"]),
    ("Ab", "sus2", &["Ab", "Bb", "Eb"]),
    ("A", "sus2", &["A", "B", "E"]),
    ("Bb", "sus2", &["Bb", "C", "F"]),
    ("B", "sus2", &["B", "C#", "F#"]),
    // sus4
    ("C", "sus4", &["C", "F", "G"]),
    ("Db", "sus4", &["Db", "Gb", "Ab"]),
    ("D", "sus4", &["D", "G", "A"]),
    ("Eb", "sus4", &["Eb", "Ab", "Bb"]),
    ("E", "sus4", &["E", "A", "B"]),
    ("F", "sus4", &["F", "Bb", "C"]),
    ("F#", "sus4", &["F#", "B", "C#"]),
    ("G", "sus4", &["G", "C", "D"]),
    ("Ab", "sus4", &["Ab", "Db", "Eb"]),
    ("A", "sus4", &["A", "D", "E"]),
    ("Bb", "sus4", &["Bb", "Eb", "F"]),
    ("B", "sus4", &["B", "E", "F#"]),
    // 7sus4
    ("C", "7sus4", &["C", "F", "G", "Bb"]),
    ("Db", "7sus4", &["Db", "Gb", "Ab", "Cb"]),
    ("D", "7sus4", &["D", "G", "A", "C"]),
    ("Eb", "7sus4", &["Eb", "Ab", "Bb", "Db"]),
    ("E", "7sus4", &["E", "A", "B", "D"]),
    ("F", "7sus4", &["F", "Bb", "C", "Eb"]),
    ("F#", "7sus4", &["F#", "B", "C#", "E"]),
    ("G", "7sus4", &["G", "C", "D", "F"]),
    ("Ab", "7sus4", &["Ab", "Db", "Eb", "Gb"]),
    ("A", "7sus4", &["A", "D", "E", "G"]),
    ("Bb", "7sus4", &["Bb", "Eb", "F", "Ab"]),
    ("B", "7sus4", &["B", "E", "F#", "A"]),
    // 6
    ("C", "6", &["C", "E", "G", "A"]),
    ("Db", "6", &["Db", "F", "Ab", "Bb"]),
    ("D", "6", &["D", "F#", "A", "B"]),
    ("Eb", "6", &["Eb", "G", "Bb", "C"]),
    ("E", "6", &["E", "G#", "B", "C#"]),
    ("F", "6", &["F", "A", "C", "D"]),
    ("F#", "6", &["F#", "A#", "C#", "D#"]),
    ("G", "6", &["G", "B", "D", "E"]),
    ("Ab", "6", &["Ab", "C", "Eb", "F"]),
    ("A", "6", &["A", "C#", "E", "F#"]),
    ("Bb", "6", &["Bb", "D", "F", "G"]),
    ("B", "6", &["B", "D#", "F#", "G#"]),
    // m6
    ("C", "m6", &["C", "Eb", "G", "A"]),
    ("Db", "m6", &["Db", "Fb", "Ab", "Bb"]),
    ("D", "m6", &["D", "F", "A", "B"]),
    ("Eb", "m6", &["Eb", "Gb", "Bb", "C"]),
    ("E", "m6", &["E", "G", "B", "C#"]),
    ("F", "m6", &["F", "Ab", "C", "D"]),
    ("F#", "m6", &["F#", "A", "C#", "D#"]),
    ("G", "m6", &["G", "Bb", "D", "E"]),
    ("Ab", "m6", &["Ab", "Cb", "Eb", "F"]),
    ("A", "m6", &["A", "C", "E", "F#"]),
    ("Bb", "m6", &["Bb", "Db", "F", "G"]),
    ("B", "m6", &["B", "D", "F#", "G#"]),
    // 9
    ("C", "9", &["C", "E", "G", "Bb", "D"]),
    ("Db", "9", &["Db", "F", "Ab", "Cb", "Eb"]),
    ("D", "9", &["D", "F#", "A", "C", "E"]),
    ("Eb", "9", &["Eb", "G", "Bb", "Db", "F"]),
    ("E", "9", &["E", "G#", "B", "D", "F#"]),
    ("F", "9", &["F", "A", "C", "Eb", "G"]),
    ("F#", "9", &["F#", "A#", "C#", "E", "G#"]),
    ("G", "9", &["G", "B", "D", "F", "A"]),
    ("Ab", "9", &["Ab", "C", "Eb", "Gb", "Bb"]),
    ("A", "9", &["A", "C#", "E", "G", "B"]),
    ("Bb", "9", &["Bb", "D", "F", "Ab", "C"]),
    ("B", "9", &["B", "D#", "F#", "A", "C#"]),
    // maj9
    ("C", "maj9", &["C", "E", "G", "B", "D"]),
    ("Db", "maj9", &["Db", "F", "Ab", "C", "Eb"]),
    ("D", "maj9", &["D", "F#", "A", "C#", "E"]),
    ("Eb", "maj9", &["Eb", "G", "Bb", "D", "F"]),
    ("E", "maj9", &["E", "G#", "B", "D#", "F#"]),
    ("F", "maj9", &["F", "A", "C", "E", "G"]),
    ("F#", "maj9", &["F#", "A#", "C#", "E#", "G#"]),
    ("G", "maj9", &["G", "B", "D", "F#", "A"]),
    ("Ab", "maj9", &["Ab", "C", "Eb", "G", "Bb"]),
    ("A", "maj9", &["A", "C#", "E", "G#", "B"]),
    ("Bb", "maj9", &["Bb", "D", "F", "A", "C"]),
    ("B", "maj9", &["B", "D#", "F#", "A#", "C#"]),
    // m9
    ("C", "m9", &["C", "Eb", "G", "Bb", "D"]),
    ("Db", "m9", &["Db", "Fb", "Ab", "Cb", "Eb"]),
    ("D", "m9", &["D", "F", "A", "C", "E"]),
    ("Eb", "m9", &["Eb", "Gb", "Bb", "Db", "F"]),
    ("E", "m9", &["E", "G", "B", "D", "F#"]),
    ("F", "m9", &["F", "Ab", "C", "Eb", "G"]),
    ("F#", "m9", &["F#", "A", "C#", "E", "G#"]),
    ("G", "m9", &["G", "Bb", "D", "F", "A"]),
    ("Ab", "m9", &["Ab", "Cb", "Eb", "Gb", "Bb"]),
    ("A", "m9", &["A", "C", "E", "G", "B"]),
    ("Bb", "m9", &["Bb", "Db", "F", "Ab", "C"]),
    ("B", "m9", &["B", "D", "F#", "A", "C#"]),
    // add9
    ("C", "add9", &["C", "E", "G", "D"]),
    ("Db", "add9", &["Db", "F", "Ab", "Eb"]),
    ("D", "add9", &["D", "F#", "A", "E"]),
    ("Eb", "add9", &["Eb", "G", "Bb", "F"]),
    ("E", "add9", &["E", "G#", "B", "F#"]),
    ("F", "add9", &["F", "A", "C", "G"]),
    ("F#", "add9", &["F#", "A#", "C#", "G#"]),
    ("G", "add9", &["G", "B", "D", "A"]),
    ("Ab", "add9", &["Ab", "C", "Eb", "Bb"]),
    ("A", "add9", &["A", "C#", "E", "B"]),
    ("Bb", "add9", &["Bb", "D", "F", "C"]),
    ("B", "add9", &["B", "D#", "F#", "C#"]),
    // 11
    ("C", "11", &["C", "E", "G", "Bb", "D", "F"]),
    ("Db", "11", &["Db", "F", "Ab", "Cb", "Eb", "Gb"]),
    ("D", "11", &["D", "F#", "A", "C", "E", "G"]),
    ("Eb", "11", &["Eb", "G", "Bb", "Db", "F", "Ab"]),
    ("E", "11", &["E", "G#", "B", "D", "F#", "A"]),
    ("F", "11", &["F", "A", "C", "Eb", "G", "Bb"]),
    ("F#", "11", &["F#", "A#", "C#", "E", "G#", "B"]),
    ("G", "11", &["G", "B", "D", "F", "A", "C"]),
    ("Ab", "11", &["Ab", "C", "Eb", "Gb", "Bb", "Db"]),
    ("A", "11", &["A", "C#", "E", "G", "B", "D"]),
    ("Bb", "11", &["Bb", "D", "F", "Ab", "C", "Eb"]),
    ("B", "11", &["B", "D#", "F#", "A", "C#", "E"]),
    // 13
    ("C", "13", &["C", "E", "G", "Bb", "D", "A"]),
    ("Db", "13", &["Db", "F", "Ab", "Cb", "Eb", "Bb"]),
    ("D", "13", &["D", "F#", "A", "C", "E", "B"]),
    ("Eb", "13", &["Eb", "G", "Bb", "Db", "F", "C"]),
    ("E", "13", &["E", "G#", "B", "D", "F#", "C#"]),
    ("F", "13", &["F", "A", "C", "Eb", "G", "D"]),
    ("F#", "13", &["F#", "A#", "C#", "E", "G#", "D#"]),
    ("G", "13", &["G", "B", "D", "F", "A", "E"]),
    ("Ab", "13", &["Ab", "C", "Eb", "Gb", "Bb", "F"]),
    ("A", "13", &["A", "C#", "E", "G", "B", "F#"]),
    ("Bb", "13", &["Bb", "D", "F", "Ab", "C", "G"]),
    ("B", "13", &["B", "D#", "F#", "A", "C#", "G#"]),
    // 5
    ("C", "5", &["C", "G"]),
    ("Db", "5", &["Db", "Ab"]),
    ("D", "5", &["D", "A"]),
    ("Eb", "5", &["Eb", "Bb"]),
    ("E", "5", &["E", "B"]),
    ("F", "5", &["F", "C"]),
    ("F#", "5", &["F#", "C#"]),
    ("G", "5", &["G", "D"]),
    ("Ab", "5", &["Ab", "Eb"]),
    ("A", "5", &["A", "E"]),
    ("Bb", "5", &["Bb", "F"]),
    ("B", "5", &["B", "F#"]),
];

fn names(chord: Chord) -> Vec<String> {
    chord.tones().iter().map(|t| t.note.name()).collect()
}

#[test]
fn ac_p0_1_table_has_252_rows_21_qualities_by_12_roots() {
    assert_eq!(CASES.len(), 252);
    let mut qualities: Vec<&str> = CASES.iter().map(|(_, q, _)| *q).collect();
    qualities.dedup();
    assert_eq!(qualities.len(), 21);
}

#[test]
fn ac_p0_1_every_quality_in_every_root_is_spelled_by_function() {
    for (root, suffix, expected) in CASES {
        let symbol = format!("{root}{suffix}");
        let chord = parse_chord_symbol(&symbol)
            .unwrap_or_else(|e| panic!("case {root} {suffix:?}: {symbol} did not parse: {e}"));
        assert_eq!(
            chord.symbol(),
            symbol,
            "case {root} {suffix:?}: canonical symbol"
        );
        assert_eq!(
            names(chord),
            *expected,
            "case {root} {suffix:?}: tone spelling"
        );
    }
}

#[test]
fn ac_p0_1_qualities_in_the_table_are_all_21_distinct_chord_qualities() {
    let mut seen: Vec<ChordQuality> = CASES
        .iter()
        .map(|(root, suffix, _)| {
            parse_chord_symbol(&format!("{root}{suffix}"))
                .unwrap()
                .quality
        })
        .collect();
    seen.sort_by_key(|q| q.symbol());
    seen.dedup();
    assert_eq!(seen.len(), 21);
    for quality in ChordQuality::ALL {
        assert!(
            seen.contains(&quality),
            "quality {quality:?} is missing from the table"
        );
    }
}

/// Spot checks written straight from the PRD and the task text.
#[test]
fn g_1_5_named_examples() {
    let spell = |s: &str| names(parse_chord_symbol(s).unwrap());
    assert_eq!(spell("D7"), ["D", "F#", "A", "C"]);
    assert_eq!(spell("Bbmaj7"), ["Bb", "D", "F", "A"]);
    assert_eq!(spell("A#m"), ["A#", "C#", "E#"]);
    assert_eq!(spell("G#dim7"), ["G#", "B", "D", "F"]);
    assert_eq!(spell("Fmaj7"), ["F", "A", "C", "E"]);
}

#[test]
fn g_1_7_the_players_root_spelling_is_kept() {
    for (input, root) in [
        ("A#m", "A#"),
        ("Bbm", "Bb"),
        ("Dbmaj7", "Db"),
        ("C#7", "C#"),
        ("Gb", "Gb"),
    ] {
        let chord = parse_chord_symbol(input).unwrap();
        assert_eq!(chord.root, root.parse::<Note>().unwrap(), "case {input}");
        assert_eq!(chord.symbol(), input, "case {input}");
    }
}
