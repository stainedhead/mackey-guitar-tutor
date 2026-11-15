//! A9: diatonic chords, Roman numerals and `roman_to_chord` (FB-4.1, FB-4.2, SAC-06).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]

use mackey_theory::{
    Accidental, Degree, Key, Mode, Note, NumeralCase, RomanNumeral, RomanParseError, RomanSuffix,
    roman_to_chord,
};

fn key(tonic: &str, mode: Mode) -> Key {
    Key::new(tonic.parse::<Note>().unwrap(), mode).unwrap()
}

fn symbols(k: Key, seventh: bool) -> Vec<String> {
    k.diatonic_chords(seventh)
        .iter()
        .map(|d| d.chord.symbol())
        .collect()
}

fn numerals(k: Key, seventh: bool) -> Vec<String> {
    k.diatonic_chords(seventh)
        .iter()
        .map(|d| d.roman.to_string())
        .collect()
}

/// Suffix of the diatonic triads and sevenths on each degree of a major key.
const MAJOR_TRIADS: [&str; 7] = ["", "m", "m", "", "", "m", "dim"];
const MAJOR_SEVENTHS: [&str; 7] = ["maj7", "m7", "m7", "maj7", "7", "m7", "m7b5"];
/// ... and of a natural minor key.
const MINOR_TRIADS: [&str; 7] = ["m", "dim", "", "m", "m", "", ""];
const MINOR_SEVENTHS: [&str; 7] = ["m7", "m7b5", "maj7", "m7", "m7", "maj7", "7"];

/// (key, the seven scale degrees as spelled in that key), written by hand.
const MAJOR_ROOTS: &[(&str, [&str; 7])] = &[
    ("C", ["C", "D", "E", "F", "G", "A", "B"]),
    ("G", ["G", "A", "B", "C", "D", "E", "F#"]),
    ("D", ["D", "E", "F#", "G", "A", "B", "C#"]),
    ("A", ["A", "B", "C#", "D", "E", "F#", "G#"]),
    ("E", ["E", "F#", "G#", "A", "B", "C#", "D#"]),
    ("B", ["B", "C#", "D#", "E", "F#", "G#", "A#"]),
    ("F#", ["F#", "G#", "A#", "B", "C#", "D#", "E#"]),
    ("F", ["F", "G", "A", "Bb", "C", "D", "E"]),
    ("Bb", ["Bb", "C", "D", "Eb", "F", "G", "A"]),
    ("Eb", ["Eb", "F", "G", "Ab", "Bb", "C", "D"]),
    ("Ab", ["Ab", "Bb", "C", "Db", "Eb", "F", "G"]),
    ("Db", ["Db", "Eb", "F", "Gb", "Ab", "Bb", "C"]),
];

const MINOR_ROOTS: &[(&str, [&str; 7])] = &[
    ("A", ["A", "B", "C", "D", "E", "F", "G"]),
    ("E", ["E", "F#", "G", "A", "B", "C", "D"]),
    ("B", ["B", "C#", "D", "E", "F#", "G", "A"]),
    ("F#", ["F#", "G#", "A", "B", "C#", "D", "E"]),
    ("C#", ["C#", "D#", "E", "F#", "G#", "A", "B"]),
    ("G#", ["G#", "A#", "B", "C#", "D#", "E", "F#"]),
    ("D#", ["D#", "E#", "F#", "G#", "A#", "B", "C#"]),
    ("D", ["D", "E", "F", "G", "A", "Bb", "C"]),
    ("G", ["G", "A", "Bb", "C", "D", "Eb", "F"]),
    ("C", ["C", "D", "Eb", "F", "G", "Ab", "Bb"]),
    ("F", ["F", "G", "Ab", "Bb", "C", "Db", "Eb"]),
    ("Bb", ["Bb", "C", "Db", "Eb", "F", "Gb", "Ab"]),
];

fn expected(roots: &[&str; 7], suffixes: &[&str; 7]) -> Vec<String> {
    roots
        .iter()
        .zip(suffixes)
        .map(|(r, s)| format!("{r}{s}"))
        .collect()
}

#[test]
fn fb_4_2_all_12_major_keys_by_7_degrees_triads_and_sevenths() {
    for (tonic, roots) in MAJOR_ROOTS {
        let k = key(tonic, Mode::Major);
        assert_eq!(
            symbols(k, false),
            expected(roots, &MAJOR_TRIADS),
            "case {tonic} major triads"
        );
        assert_eq!(
            symbols(k, true),
            expected(roots, &MAJOR_SEVENTHS),
            "case {tonic} major sevenths"
        );
    }
}

#[test]
fn fb_4_2_natural_minor_keys_use_their_own_qualities() {
    for (tonic, roots) in MINOR_ROOTS {
        let k = key(tonic, Mode::NaturalMinor);
        assert_eq!(
            symbols(k, false),
            expected(roots, &MINOR_TRIADS),
            "case {tonic} minor triads"
        );
        assert_eq!(
            symbols(k, true),
            expected(roots, &MINOR_SEVENTHS),
            "case {tonic} minor sevenths"
        );
    }
}

/// (mode, triads, sevenths) of the other five modes on C, from their parent major keys.
const C_MODES: &[(Mode, [&str; 7], [&str; 7])] = &[
    (
        Mode::Dorian,
        ["Cm", "Dm", "Eb", "F", "Gm", "Adim", "Bb"],
        ["Cm7", "Dm7", "Ebmaj7", "F7", "Gm7", "Am7b5", "Bbmaj7"],
    ),
    (
        Mode::Phrygian,
        ["Cm", "Db", "Eb", "Fm", "Gdim", "Ab", "Bbm"],
        ["Cm7", "Dbmaj7", "Eb7", "Fm7", "Gm7b5", "Abmaj7", "Bbm7"],
    ),
    (
        Mode::Lydian,
        ["C", "D", "Em", "F#dim", "G", "Am", "Bm"],
        ["Cmaj7", "D7", "Em7", "F#m7b5", "Gmaj7", "Am7", "Bm7"],
    ),
    (
        Mode::Mixolydian,
        ["C", "Dm", "Edim", "F", "Gm", "Am", "Bb"],
        ["C7", "Dm7", "Em7b5", "Fmaj7", "Gm7", "Am7", "Bbmaj7"],
    ),
    (
        Mode::Locrian,
        ["Cdim", "Db", "Ebm", "Fm", "Gb", "Ab", "Bbm"],
        ["Cm7b5", "Dbmaj7", "Ebm7", "Fm7", "Gbmaj7", "Ab7", "Bbm7"],
    ),
];

#[test]
fn fb_4_2_the_other_modes_use_their_own_qualities() {
    for (mode, triads, sevenths) in C_MODES {
        let k = key("C", *mode);
        assert_eq!(symbols(k, false), *triads, "case C {mode:?} triads");
        assert_eq!(symbols(k, true), *sevenths, "case C {mode:?} sevenths");
    }
}

#[test]
fn fb_4_2_roman_numerals_of_the_strip() {
    let c = key("C", Mode::Major);
    assert_eq!(
        numerals(c, false),
        ["I", "ii", "iii", "IV", "V", "vi", "vii°"]
    );
    assert_eq!(
        numerals(c, true),
        ["Imaj7", "ii7", "iii7", "IVmaj7", "V7", "vi7", "viiø7"]
    );
    let a_minor = key("A", Mode::NaturalMinor);
    assert_eq!(
        numerals(a_minor, false),
        ["i", "ii°", "III", "iv", "v", "VI", "VII"]
    );
    assert_eq!(
        numerals(a_minor, true),
        ["i7", "iiø7", "IIImaj7", "iv7", "v7", "VImaj7", "VII7"]
    );
    let d_dorian = key("D", Mode::Dorian);
    assert_eq!(
        numerals(d_dorian, false),
        ["i", "ii", "III", "IV", "v", "vi°", "VII"]
    );
}

#[test]
fn a9_every_diatonic_chord_uses_only_scale_tones_of_its_key() {
    for mode in Mode::DIATONIC {
        for k in Key::all_offered(mode) {
            let scale = k.scale();
            for seventh in [false, true] {
                for d in k.diatonic_chords(seventh) {
                    for pc in d.chord.pitch_classes().iter() {
                        assert!(
                            scale.contains(pc),
                            "case {} {}: tone {pc:?} not in scale",
                            k.name(),
                            d.chord.symbol()
                        );
                    }
                    assert_eq!(
                        d.chord.root,
                        scale.note_at(d.degree).unwrap(),
                        "case {} degree",
                        k.name()
                    );
                    assert_eq!(d.chord.quality.has_seventh(), seventh, "case {}", k.name());
                }
            }
        }
    }
}

#[test]
fn a9_a_diatonic_chord_round_trips_through_its_numeral() {
    for mode in Mode::DIATONIC {
        for k in Key::all_offered(mode) {
            for seventh in [false, true] {
                for d in k.diatonic_chords(seventh) {
                    let text = d.roman.to_string();
                    let chord = roman_to_chord(&text, &k)
                        .unwrap_or_else(|e| panic!("case {} {text}: {e}", k.name()));
                    assert_eq!(chord, d.chord, "case {} {text}", k.name());
                }
            }
        }
    }
}

/// (key tonic, mode, numeral, expected chord symbol)
const ROMAN_TO_CHORD: &[(&str, Mode, &str, &str)] = &[
    ("G", Mode::Major, "vi", "Em"),
    ("C", Mode::Major, "bVII", "Bb"),
    ("C", Mode::Major, "V7/V", "D7"),
    ("C", Mode::Major, "V/V", "D"),
    ("C", Mode::Major, "ii", "Dm"),
    ("C", Mode::Major, "V7", "G7"),
    ("C", Mode::Major, "IVmaj7", "Fmaj7"),
    ("C", Mode::Major, "viiø7", "Bm7b5"),
    ("C", Mode::Major, "vii°", "Bdim"),
    ("G", Mode::Major, "bVII", "F"),
    ("F", Mode::Major, "bVII", "Eb"),
    ("Bb", Mode::Major, "IV", "Eb"),
    ("C", Mode::Major, "bIII", "Eb"),
    ("C", Mode::Major, "bVI", "Ab"),
    ("C", Mode::Major, "#iv°", "F#dim"),
    ("C", Mode::Major, "V7/vi", "E7"),
    ("C", Mode::Major, "V7/IV", "C7"),
    ("C", Mode::Major, "vii°7/V", "F#dim7"),
    ("D", Mode::Major, "I", "D"),
    ("Eb", Mode::Major, "vi", "Cm"),
    ("A", Mode::NaturalMinor, "III", "C"),
    ("A", Mode::NaturalMinor, "VII", "G"),
    ("A", Mode::NaturalMinor, "iv", "Dm"),
    ("C", Mode::Major, " vi ", "Am"),
    ("C", Mode::Major, "I+", "Caug"),
];

#[test]
fn a9_roman_to_chord_table() {
    for (tonic, mode, numeral, symbol) in ROMAN_TO_CHORD {
        let chord = roman_to_chord(numeral, &key(tonic, *mode))
            .unwrap_or_else(|e| panic!("case {tonic} {numeral}: {e}"));
        assert_eq!(
            chord.symbol(),
            *symbol,
            "case {numeral} in {tonic} {mode:?}"
        );
    }
}

#[test]
fn a9_numerals_round_trip_through_display_and_from_str() {
    let combos = [
        (NumeralCase::Upper, RomanSuffix::None),
        (NumeralCase::Lower, RomanSuffix::None),
        (NumeralCase::Lower, RomanSuffix::Dim),
        (NumeralCase::Lower, RomanSuffix::HalfDim),
        (NumeralCase::Upper, RomanSuffix::Aug),
        (NumeralCase::Upper, RomanSuffix::Seventh),
        (NumeralCase::Lower, RomanSuffix::Seventh),
        (NumeralCase::Upper, RomanSuffix::Maj7),
        (NumeralCase::Lower, RomanSuffix::Dim7),
    ];
    let mut count = 0;
    for degree in Degree::all() {
        for acc in [-1, 0, 1] {
            for (case, suffix) in combos {
                let n = RomanNumeral {
                    degree,
                    accidental: Accidental::try_from(acc).unwrap(),
                    case,
                    suffix,
                };
                let text = n.to_string();
                assert_eq!(text.parse::<RomanNumeral>().unwrap(), n, "case {text}");
                assert!(n.quality().is_some(), "case {text}");
                count += 1;
            }
        }
    }
    assert_eq!(count, 7 * 3 * 9);
}

#[test]
fn a9_numeral_display_examples() {
    for (text, shown) in [
        ("bVII", "bVII"),
        ("vii°", "vii°"),
        ("viiø7", "viiø7"),
        ("V7", "V7"),
        ("IVmaj7", "IVmaj7"),
        ("#iv", "#iv"),
        ("viio", "vii°"),
        ("ii7", "ii7"),
        ("♭III", "bIII"),
    ] {
        assert_eq!(
            text.parse::<RomanNumeral>().unwrap().to_string(),
            shown,
            "case {text}"
        );
    }
}

#[test]
fn a9_numeral_errors_name_the_problem_and_position() {
    let c = key("C", Mode::Major);
    let cases: &[(&str, &str)] = &[
        ("", "Empty"),
        ("   ", "Empty"),
        ("VIII", "DegreeOutOfRange"),
        ("IIII", "DegreeOutOfRange"),
        ("Iv", "NotANumeral"),
        ("xyz", "NotANumeral"),
        ("b", "NotANumeral"),
        ("V°", "UnsupportedSuffix"),
        ("vii+", "UnsupportedSuffix"),
        ("Vmaj9", "UnsupportedSuffix"),
        ("imaj7", "UnsupportedSuffix"),
        ("V/", "Empty"),
        ("V/xyz", "NotANumeral"),
    ];
    for (input, want) in cases {
        let err = roman_to_chord(input, &c).unwrap_err();
        let got = match err {
            RomanParseError::Empty => "Empty",
            RomanParseError::NotANumeral { .. } => "NotANumeral",
            RomanParseError::DegreeOutOfRange { .. } => "DegreeOutOfRange",
            RomanParseError::UnsupportedSuffix { .. } => "UnsupportedSuffix",
        };
        assert_eq!(got, *want, "case {input:?}");
        assert_ne!(err.to_string(), "");
    }
    match roman_to_chord("V/xyz", &c).unwrap_err() {
        RomanParseError::NotANumeral { span, found } => {
            assert_eq!((span.start, span.end, found.as_str()), (2, 3, "x"));
        }
        other => panic!("unexpected {other:?}"),
    }
    match roman_to_chord("V7x", &c).unwrap_err() {
        RomanParseError::UnsupportedSuffix { span, found } => {
            assert_eq!((span.start, span.end, found.as_str()), (1, 3, "7x"));
        }
        other => panic!("unexpected {other:?}"),
    }
}

#[test]
fn a9_plain_drops_the_suffix() {
    let n: RomanNumeral = "V7".parse().unwrap();
    assert_eq!(n.plain().to_string(), "V");
}
