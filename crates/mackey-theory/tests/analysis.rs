//! A9 and FB-4.5: is the chord in the key, and if not, what does it do.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]

use mackey_theory::{ChordFunction, Key, Mode, analyze, parse_chord_symbol};

fn key(tonic: &str, mode: Mode) -> Key {
    Key::new(tonic.parse().unwrap(), mode).unwrap()
}

/// (key tonic, mode, chord, in key, numeral, badge)
type Case = (
    &'static str,
    Mode,
    &'static str,
    bool,
    Option<&'static str>,
    Option<&'static str>,
);

const CASES: &[Case] = &[
    // In key: a numeral, no badge.
    ("C", Mode::Major, "C", true, Some("I"), None),
    ("C", Mode::Major, "Dm", true, Some("ii"), None),
    ("C", Mode::Major, "G7", true, Some("V7"), None),
    ("C", Mode::Major, "Bdim", true, Some("vii°"), None),
    ("C", Mode::Major, "Bm7b5", true, Some("viiø7"), None),
    ("C", Mode::Major, "Cmaj7", true, Some("Imaj7"), None),
    ("C", Mode::Major, "Dm7", true, Some("ii7"), None),
    ("C", Mode::Major, "G/B", true, Some("V"), None),
    ("C", Mode::Major, "Csus4", true, None, None),
    ("A", Mode::NaturalMinor, "Am", true, Some("i"), None),
    ("A", Mode::NaturalMinor, "C", true, Some("III"), None),
    ("D", Mode::Dorian, "G", true, Some("IV"), None),
    // AC-FB-4.1: secondary dominants.
    (
        "C",
        Mode::Major,
        "D7",
        false,
        Some("V7"),
        Some("Not in key (V/V)"),
    ),
    (
        "C",
        Mode::Major,
        "D",
        false,
        Some("V"),
        Some("Not in key (V/V)"),
    ),
    (
        "C",
        Mode::Major,
        "D9",
        false,
        None,
        Some("Not in key (V/V)"),
    ),
    (
        "C",
        Mode::Major,
        "E7",
        false,
        Some("V7"),
        Some("Not in key (V/vi)"),
    ),
    (
        "C",
        Mode::Major,
        "A7",
        false,
        Some("V7"),
        Some("Not in key (V/ii)"),
    ),
    (
        "C",
        Mode::Major,
        "B7",
        false,
        Some("V7"),
        Some("Not in key (V/iii)"),
    ),
    (
        "C",
        Mode::Major,
        "C7",
        false,
        Some("V7"),
        Some("Not in key (V/IV)"),
    ),
    (
        "C",
        Mode::Major,
        "A",
        false,
        Some("V"),
        Some("Not in key (V/ii)"),
    ),
    (
        "C",
        Mode::Major,
        "D/F#",
        false,
        Some("V"),
        Some("Not in key (V/V)"),
    ),
    (
        "G",
        Mode::Major,
        "A7",
        false,
        Some("V7"),
        Some("Not in key (V/V)"),
    ),
    (
        "F",
        Mode::Major,
        "G7",
        false,
        Some("V7"),
        Some("Not in key (V/V)"),
    ),
    (
        "A",
        Mode::NaturalMinor,
        "D7",
        false,
        Some("V7"),
        Some("Not in key (V/VII)"),
    ),
    // Borrowed from the parallel mode.
    (
        "G",
        Mode::Major,
        "F",
        false,
        Some("bVII"),
        Some("Not in key (bVII, borrowed from G minor)"),
    ),
    (
        "C",
        Mode::Major,
        "Bb",
        false,
        Some("bVII"),
        Some("Not in key (bVII, borrowed from C minor)"),
    ),
    (
        "C",
        Mode::Major,
        "Ab",
        false,
        Some("bVI"),
        Some("Not in key (bVI, borrowed from C minor)"),
    ),
    (
        "C",
        Mode::Major,
        "Eb",
        false,
        Some("bIII"),
        Some("Not in key (bIII, borrowed from C minor)"),
    ),
    (
        "C",
        Mode::Major,
        "Fm",
        false,
        Some("iv"),
        Some("Not in key (iv, borrowed from C minor)"),
    ),
    (
        "C",
        Mode::Major,
        "Cm",
        false,
        Some("i"),
        Some("Not in key (i, borrowed from C minor)"),
    ),
    (
        "C",
        Mode::Major,
        "Bb7",
        false,
        Some("bVII7"),
        Some("Not in key (bVII7, borrowed from C minor)"),
    ),
    (
        "C",
        Mode::Major,
        "Ddim",
        false,
        Some("ii°"),
        Some("Not in key (ii°, borrowed from C minor)"),
    ),
    (
        "E",
        Mode::Major,
        "D",
        false,
        Some("bVII"),
        Some("Not in key (bVII, borrowed from E minor)"),
    ),
    // Minor keys: the major V and the leading-tone chords come from harmonic minor.
    (
        "A",
        Mode::NaturalMinor,
        "E",
        false,
        Some("V"),
        Some("Not in key (V, borrowed from A harmonic minor)"),
    ),
    (
        "A",
        Mode::NaturalMinor,
        "E7",
        false,
        Some("V7"),
        Some("Not in key (V7, borrowed from A harmonic minor)"),
    ),
    (
        "A",
        Mode::NaturalMinor,
        "G#dim",
        false,
        Some("vii°"),
        Some("Not in key (vii°, borrowed from A harmonic minor)"),
    ),
    (
        "A",
        Mode::NaturalMinor,
        "A",
        false,
        Some("I"),
        Some("Not in key (I, borrowed from A major)"),
    ),
    // No function known.
    (
        "C",
        Mode::Major,
        "C#",
        false,
        Some("#I"),
        Some("Not in key"),
    ),
    (
        "C",
        Mode::Major,
        "F7",
        false,
        Some("IV7"),
        Some("Not in key"),
    ),
    (
        "C",
        Mode::Major,
        "Db",
        false,
        Some("bII"),
        Some("Not in key"),
    ),
    (
        "C",
        Mode::Major,
        "F#",
        false,
        Some("#IV"),
        Some("Not in key"),
    ),
];

#[test]
fn a9_analysis_table() {
    for (tonic, mode, chord, in_key, numeral, badge) in CASES {
        let k = key(tonic, *mode);
        let a = analyze(&k, &parse_chord_symbol(chord).unwrap());
        let label = format!("case {chord} in {tonic} {mode:?}");
        assert_eq!(a.in_key, *in_key, "{label}: in_key");
        if let Some(numeral) = numeral {
            assert_eq!(
                a.roman.map(|r| r.to_string()).as_deref(),
                Some(*numeral),
                "{label}: numeral"
            );
        } else {
            assert_eq!(a.roman, None, "{label}: numeral");
        }
        assert_eq!(a.badge().as_deref(), *badge, "{label}: badge");
        assert_eq!(
            a.function.is_none(),
            *in_key,
            "{label}: a function only for chords outside the key"
        );
    }
}

#[test]
fn ac_fb_4_1_d7_in_c_is_a_secondary_dominant_of_v() {
    let c = key("C", Mode::Major);
    let a = analyze(&c, &parse_chord_symbol("D7").unwrap());
    assert!(!a.in_key);
    match a.function {
        Some(ChordFunction::SecondaryDominant { target }) => assert_eq!(target.to_string(), "V"),
        other => panic!("expected a secondary dominant, got {other:?}"),
    }
    assert_eq!(a.badge().as_deref(), Some("Not in key (V/V)"));
}

#[test]
fn fb_4_5_b_flat_seven_in_g_is_borrowed_from_g_minor() {
    let g = key("G", Mode::Major);
    let a = analyze(&g, &parse_chord_symbol("F").unwrap());
    match a.function {
        Some(ChordFunction::Borrowed { from, roman }) => {
            assert_eq!(from, Mode::NaturalMinor);
            assert_eq!(roman.to_string(), "bVII");
        }
        other => panic!("expected borrowed, got {other:?}"),
    }
    assert_eq!(
        a.badge().as_deref(),
        Some("Not in key (bVII, borrowed from G minor)")
    );
}

#[test]
fn fb_4_5_a_dominant_ninth_is_also_a_secondary_dominant() {
    let c = key("C", Mode::Major);
    let a = analyze(&c, &parse_chord_symbol("D9").unwrap());
    assert_eq!(a.badge().as_deref(), Some("Not in key (V/V)"));
}

#[test]
fn a9_in_key_chords_have_no_badge_and_no_function() {
    for mode in Mode::DIATONIC {
        for k in Key::all_offered(mode) {
            for seventh in [false, true] {
                for d in k.diatonic_chords(seventh) {
                    let a = analyze(&k, &d.chord);
                    assert!(a.in_key, "case {} {}", k.name(), d.chord.symbol());
                    assert_eq!(
                        a.roman,
                        Some(d.roman),
                        "case {} {}",
                        k.name(),
                        d.chord.symbol()
                    );
                    assert_eq!(a.function, None);
                    assert_eq!(a.badge(), None);
                }
            }
        }
    }
}

#[test]
fn a9_every_secondary_dominant_of_every_major_key_is_named() {
    // V7 of each of ii, iii, IV, V, vi in all 12 major keys is a secondary dominant.
    for k in Key::all_offered(Mode::Major) {
        let chords = k.diatonic_chords(false);
        for (degree, target) in [(2, "ii"), (3, "iii"), (4, "IV"), (5, "V"), (6, "vi")] {
            let target_root = chords[degree - 1].chord.root;
            let dominant = mackey_theory::Interval::P5.apply(target_root).unwrap();
            let chord = mackey_theory::Chord::new(dominant, mackey_theory::ChordQuality::Dominant7);
            let a = analyze(&k, &chord);
            assert!(!a.in_key, "case {} {}", k.name(), chord.symbol());
            assert_eq!(
                a.badge(),
                Some(format!("Not in key (V/{target})")),
                "case {} {}",
                k.name(),
                chord.symbol()
            );
        }
    }
}
