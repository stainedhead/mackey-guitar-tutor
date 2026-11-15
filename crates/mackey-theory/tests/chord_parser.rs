//! A8, AC-P0-2 and E-FB-1: the chord-symbol parser, its diagnostics and suggestions.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]

use mackey_theory::{
    ChordParseError, Note, ParseOptions, ParseOutcome, Severity, parse_chord, parse_chord_symbol,
};

/// (input, canonical symbol after parsing)
const VALID: &[(&str, &str)] = &[
    ("C", "C"),
    ("Cmaj", "C"),
    ("CM", "C"),
    ("CMaj", "C"),
    ("Cm", "Cm"),
    ("Cmin", "Cm"),
    ("C-", "Cm"),
    ("Cmi", "Cm"),
    ("C7", "C7"),
    ("Cmaj7", "Cmaj7"),
    ("CM7", "Cmaj7"),
    ("CMaj7", "Cmaj7"),
    ("CΔ", "Cmaj7"),
    ("CΔ7", "Cmaj7"),
    ("C△", "Cmaj7"),
    ("C△7", "Cmaj7"),
    ("Cm7", "Cm7"),
    ("Cmin7", "Cm7"),
    ("C-7", "Cm7"),
    ("Cm7b5", "Cm7b5"),
    ("Cø", "Cm7b5"),
    ("Cø7", "Cm7b5"),
    ("Cmin7b5", "Cm7b5"),
    ("C-7b5", "Cm7b5"),
    ("Cdim", "Cdim"),
    ("C°", "Cdim"),
    ("Co", "Cdim"),
    ("Cdim7", "Cdim7"),
    ("C°7", "Cdim7"),
    ("Co7", "Cdim7"),
    ("Caug", "Caug"),
    ("C+", "Caug"),
    ("C+5", "Caug"),
    ("Csus2", "Csus2"),
    ("Csus4", "Csus4"),
    ("Csus", "Csus4"),
    ("C7sus4", "C7sus4"),
    ("C7sus", "C7sus4"),
    ("C6", "C6"),
    ("Cmaj6", "C6"),
    ("Cm6", "Cm6"),
    ("Cmin6", "Cm6"),
    ("C9", "C9"),
    ("Cmaj9", "Cmaj9"),
    ("CM9", "Cmaj9"),
    ("CΔ9", "Cmaj9"),
    ("Cm9", "Cm9"),
    ("Cmin9", "Cm9"),
    ("Cadd9", "Cadd9"),
    ("Cadd2", "Cadd9"),
    ("C11", "C11"),
    ("C13", "C13"),
    ("C5", "C5"),
    // Roots: the player's spelling is kept; Unicode accidentals are read.
    ("A#m", "A#m"),
    ("Bbm", "Bbm"),
    ("C♯m", "C#m"),
    ("B♭7", "Bb7"),
    ("A♭m7♭5", "Abm7b5"),
    ("E#", "E#"),
    ("Fb", "Fb"),
    // Slash bass.
    ("D/F#", "D/F#"),
    ("Am7/G", "Am7/G"),
    ("C/E", "C/E"),
    ("Bb/D", "Bb/D"),
    ("F#m7b5/C", "F#m7b5/C"),
    ("C/E♭", "C/Eb"),
    // Whitespace around the symbol is trimmed.
    ("  Am7  ", "Am7"),
    ("\tG7\n", "G7"),
];

#[test]
fn a8_valid_symbols_parse_to_their_canonical_form() {
    for (input, canonical) in VALID {
        let chord = parse_chord_symbol(input).unwrap_or_else(|e| panic!("case {input:?}: {e}"));
        assert_eq!(chord.symbol(), *canonical, "case {input:?}");
        let outcome = parse_chord(input, ParseOptions::default());
        assert_eq!(outcome.chord, Some(chord), "case {input:?}");
        assert!(
            outcome.diagnostics.is_empty(),
            "case {input:?}: {:?}",
            outcome.diagnostics
        );
    }
}

#[test]
fn a8_every_alias_of_every_quality_works_with_every_root_spelling() {
    let roots = [
        "C", "C#", "Db", "D", "D#", "Eb", "E", "F", "F#", "Gb", "G", "G#", "Ab", "A", "A#", "Bb",
        "B",
    ];
    for root in roots {
        for (input, canonical) in VALID.iter().take(52) {
            // The first 52 rows are the C-rooted quality aliases.
            let swapped = format!("{root}{}", input.strip_prefix('C').unwrap());
            let want = format!("{root}{}", canonical.strip_prefix('C').unwrap());
            assert_eq!(
                parse_chord_symbol(&swapped).unwrap().symbol(),
                want,
                "case {swapped}"
            );
        }
    }
}

#[test]
fn g_1_7_the_players_spelling_is_kept_not_respelled() {
    let a_sharp_minor = parse_chord_symbol("A#m").unwrap();
    assert_eq!(a_sharp_minor.root, "A#".parse::<Note>().unwrap());
    assert_ne!(a_sharp_minor.root, "Bb".parse::<Note>().unwrap());
}

/// (input, error kind, span, suggestion) for the FIRST error of each invalid input.
type InvalidCase = (
    &'static str,
    &'static str,
    (usize, usize),
    Option<&'static str>,
);

const INVALID: &[InvalidCase] = &[
    ("", "Empty", (0, 0), None),
    ("   ", "Empty", (0, 0), None),
    ("Hm7", "UnknownRoot", (0, 1), Some("Bm7")),
    ("  Hm7  ", "UnknownRoot", (2, 3), Some("Bm7")),
    ("Hb7", "UnknownRoot", (0, 2), Some("Bb7")),
    ("H", "UnknownRoot", (0, 1), Some("B")),
    ("am", "UnknownRoot", (0, 1), Some("Am")),
    ("xyz", "UnknownRoot", (0, 1), None),
    ("/E", "UnknownRoot", (0, 0), None),
    ("7", "UnknownRoot", (0, 1), None),
    // E-FB-1 / Q-9: extended symbols outside the FB-4.3 grammar are errors without a suggestion.
    ("Cmaj13#11", "UnknownQuality", (1, 9), None),
    ("C#maj13#11b9", "UnknownQuality", (2, 12), None),
    ("C7b9", "UnknownQuality", (1, 4), None),
    ("C7#9", "UnknownQuality", (1, 4), None),
    ("Cmaj7#", "UnknownQuality", (1, 6), None),
    ("C♭♭", "UnknownQuality", (4, 7), None),
    ("Cé", "UnknownQuality", (1, 3), None),
    // Typos within distance 2 get a full replacement symbol.
    ("Cmaj77", "UnknownQuality", (1, 6), Some("Cmaj7")),
    ("Amj7", "UnknownQuality", (1, 4), Some("Amaj7")),
    ("Csus3", "UnknownQuality", (1, 5), Some("Csus2")),
    ("C+7", "UnknownQuality", (1, 3), Some("C+5")),
    ("Cdim9", "UnknownQuality", (1, 5), Some("Cdim7")),
    // Text that cannot belong to a quality.
    ("C maj7", "TrailingInput", (1, 6), None),
    ("Cm7 b5", "TrailingInput", (3, 6), None),
    ("C!", "TrailingInput", (1, 2), None),
    ("C🎸", "TrailingInput", (1, 5), None),
    // Bass notes.
    ("C/", "InvalidBass", (1, 2), None),
    ("C/H", "InvalidBass", (2, 3), None),
    ("C/E/G", "InvalidBass", (2, 5), None),
    ("D/F#x", "InvalidBass", (2, 5), None),
];

fn kind(e: &ChordParseError) -> &'static str {
    match e {
        ChordParseError::Empty => "Empty",
        ChordParseError::UnknownRoot { .. } => "UnknownRoot",
        ChordParseError::UnknownQuality { .. } => "UnknownQuality",
        ChordParseError::InvalidBass { .. } => "InvalidBass",
        ChordParseError::TrailingInput { .. } => "TrailingInput",
    }
}

#[test]
fn e_fb_1_invalid_symbols_report_kind_span_and_suggestion() {
    for (input, want_kind, (start, end), suggestion) in INVALID {
        let err = parse_chord_symbol(input).expect_err(input);
        assert_eq!(kind(&err), *want_kind, "case {input:?} kind");
        let span = err.span();
        assert_eq!(
            (span.start, span.end),
            (*start, *end),
            "case {input:?} span"
        );
        assert_eq!(err.suggestion(), *suggestion, "case {input:?} suggestion");
    }
}

#[test]
fn e_fb_1_the_message_matches_the_prd_wording() {
    let err = parse_chord_symbol("Hm7").unwrap_err();
    assert_eq!(
        err.to_string(),
        "Not a chord symbol I know: Hm7. Did you mean Bm7?"
    );
    let err = parse_chord_symbol("C#maj13#11b9").unwrap_err();
    assert_eq!(err.to_string(), "Not a chord symbol I know: C#maj13#11b9.");
    assert_eq!(
        parse_chord_symbol("").unwrap_err().to_string(),
        "Type a chord symbol such as Am7."
    );
}

#[test]
fn e_fb_1_diagnostics_carry_the_error_message_and_suggestion() {
    let outcome = parse_chord("Hm7", ParseOptions::default());
    assert!(outcome.chord.is_none());
    let d = &outcome.diagnostics[0];
    assert_eq!(d.severity, Severity::Error);
    assert_eq!(
        d.message,
        "Not a chord symbol I know: Hm7. Did you mean Bm7?"
    );
    assert_eq!(d.suggestion.as_deref(), Some("Bm7"));
}

#[test]
fn a8_csus_is_sus4() {
    assert_eq!(
        parse_chord_symbol("Csus").unwrap(),
        parse_chord_symbol("Csus4").unwrap()
    );
}

#[test]
fn a8_every_problem_is_reported_in_input_order() {
    let outcome = parse_chord("Xmaj77/Q", ParseOptions::default());
    assert!(outcome.chord.is_none());
    let spans: Vec<(usize, usize)> = outcome
        .diagnostics
        .iter()
        .map(|d| (d.span.start, d.span.end))
        .collect();
    assert_eq!(spans, [(0, 1), (1, 6), (7, 8)]);
    let kinds: Vec<&str> = outcome
        .diagnostics
        .iter()
        .map(|d| {
            if d.message.contains("bass") {
                "bass"
            } else {
                "symbol"
            }
        })
        .collect();
    assert_eq!(kinds, ["symbol", "symbol", "bass"]);
    // The strict form returns the first of them.
    assert_eq!(
        kind(&parse_chord_symbol("Xmaj77/Q").unwrap_err()),
        "UnknownRoot"
    );
}

#[test]
fn a8_spans_are_byte_offsets_on_char_boundaries() {
    let input = "  C♯maj13#11  ";
    let outcome = parse_chord(input, ParseOptions::default());
    let d = &outcome.diagnostics[0];
    assert_eq!(&input[d.span.range()], "maj13#11");
    assert_eq!((d.span.start, d.span.end), (6, 14));
}

#[test]
fn a8_read_h_as_b_is_off_by_default_and_a_warning_when_on() {
    let on = ParseOptions { read_h_as_b: true };
    let outcome = parse_chord("Hm7", on);
    assert_eq!(outcome.chord.unwrap().symbol(), "Bm7");
    assert_eq!(outcome.diagnostics.len(), 1);
    assert_eq!(outcome.diagnostics[0].severity, Severity::Warning);
    let bass = parse_chord("C/H", on);
    assert_eq!(bass.chord.unwrap().symbol(), "C/B");
    assert_eq!(bass.diagnostics[0].severity, Severity::Warning);
    assert_eq!(
        (bass.diagnostics[0].span.start, bass.diagnostics[0].span.end),
        (2, 3)
    );
    assert!(parse_chord("Hm7", ParseOptions::default()).chord.is_none());
}

#[test]
fn a8_from_str_is_the_strict_parser() {
    let chord: mackey_theory::Chord = "Bbmaj7".parse().unwrap();
    assert_eq!(chord.symbol(), "Bbmaj7");
    assert!("Hm7".parse::<mackey_theory::Chord>().is_err());
}

fn fixture_lines(text: &str) -> Vec<&str> {
    text.lines().filter(|l| !l.trim().is_empty()).collect()
}

#[test]
fn a8_snapshot_invalid_symbol_diagnostics() {
    let lines = fixture_lines(include_str!("fixtures/invalid_chord_symbols.txt"));
    assert!(lines.len() >= 20, "AC: at least 20 invalid inputs");
    let outcomes: Vec<(&str, ParseOutcome)> = lines
        .iter()
        .map(|l| (*l, parse_chord(l, ParseOptions::default())))
        .collect();
    for (input, outcome) in &outcomes {
        assert!(outcome.chord.is_none(), "case {input:?} must not parse");
        assert!(
            !outcome.diagnostics.is_empty(),
            "case {input:?} must explain itself"
        );
    }
    insta::assert_yaml_snapshot!("invalid_chord_symbols", outcomes);
}

#[test]
fn a8_snapshot_valid_symbols_normalise() {
    let lines = fixture_lines(include_str!("fixtures/chord_symbols.txt"));
    let normalised: Vec<(&str, String)> = lines
        .iter()
        .map(|l| {
            (
                *l,
                parse_chord_symbol(l)
                    .unwrap_or_else(|e| panic!("{l}: {e}"))
                    .symbol(),
            )
        })
        .collect();
    insta::assert_yaml_snapshot!("valid_chord_symbols", normalised);
}
