//! G-1.5 and G-1.6: how a pitch class is spelled for display with and without chord context.
//!
//! The no-context table was calculated separately from this crate: scale tones keep the key's
//! spelling, every other pitch class is flat in flat keys and C major or A minor, and sharp in
//! sharp keys.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]

use mackey_theory::{
    AccidentalPreference, Chord, ChordQuality, DisplayNote, Key, Mode, Note, PitchClass, Speller,
    parse_chord_symbol,
};

fn note(s: &str) -> Note {
    s.parse().unwrap()
}

fn key(tonic: &str, mode: Mode) -> Key {
    Key::new(note(tonic), mode).unwrap()
}

fn chord(s: &str) -> Chord {
    parse_chord_symbol(s).unwrap()
}

/// (tonic, mode, expected name of pitch classes 0..=11 with no chord context)
const NO_CONTEXT: &[(&str, Mode, [&str; 12])] = &[
    (
        "C",
        Mode::Major,
        [
            "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
        ],
    ),
    (
        "G",
        Mode::Major,
        [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ],
    ),
    (
        "D",
        Mode::Major,
        [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ],
    ),
    (
        "A",
        Mode::Major,
        [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ],
    ),
    (
        "E",
        Mode::Major,
        [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ],
    ),
    (
        "B",
        Mode::Major,
        [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ],
    ),
    (
        "F#",
        Mode::Major,
        [
            "C", "C#", "D", "D#", "E", "E#", "F#", "G", "G#", "A", "A#", "B",
        ],
    ),
    (
        "Gb",
        Mode::Major,
        [
            "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "Cb",
        ],
    ),
    (
        "F",
        Mode::Major,
        [
            "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
        ],
    ),
    (
        "Bb",
        Mode::Major,
        [
            "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
        ],
    ),
    (
        "Eb",
        Mode::Major,
        [
            "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
        ],
    ),
    (
        "Ab",
        Mode::Major,
        [
            "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
        ],
    ),
    (
        "Db",
        Mode::Major,
        [
            "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
        ],
    ),
    (
        "A",
        Mode::NaturalMinor,
        [
            "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
        ],
    ),
    (
        "E",
        Mode::NaturalMinor,
        [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ],
    ),
    (
        "B",
        Mode::NaturalMinor,
        [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ],
    ),
    (
        "F#",
        Mode::NaturalMinor,
        [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ],
    ),
    (
        "C#",
        Mode::NaturalMinor,
        [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ],
    ),
    (
        "G#",
        Mode::NaturalMinor,
        [
            "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
        ],
    ),
    (
        "D#",
        Mode::NaturalMinor,
        [
            "C", "C#", "D", "D#", "E", "E#", "F#", "G", "G#", "A", "A#", "B",
        ],
    ),
    (
        "Eb",
        Mode::NaturalMinor,
        [
            "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "Cb",
        ],
    ),
    (
        "D",
        Mode::NaturalMinor,
        [
            "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
        ],
    ),
    (
        "G",
        Mode::NaturalMinor,
        [
            "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
        ],
    ),
    (
        "C",
        Mode::NaturalMinor,
        [
            "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
        ],
    ),
    (
        "F",
        Mode::NaturalMinor,
        [
            "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
        ],
    ),
    (
        "Bb",
        Mode::NaturalMinor,
        [
            "C", "Db", "D", "Eb", "E", "F", "Gb", "G", "Ab", "A", "Bb", "B",
        ],
    ),
];

#[test]
fn g_1_5_every_key_by_all_12_pitch_classes_without_context() {
    assert_eq!(NO_CONTEXT.len(), 26);
    for (tonic, mode, expected) in NO_CONTEXT {
        let speller = Speller::new(key(tonic, *mode));
        for (pc, want) in expected.iter().enumerate() {
            let got = speller
                .spell(PitchClass::new(i32::try_from(pc).unwrap()))
                .name();
            assert_eq!(got, *want, "case {tonic} {mode:?} pitch class {pc}");
        }
    }
}

#[test]
fn g_1_1_f_major_spells_b_flat() {
    let speller = Speller::new(key("F", Mode::Major));
    let notes: Vec<String> = (0..12)
        .map(|pc| speller.spell(PitchClass::new(pc)).name())
        .collect();
    assert_eq!(speller.spell(PitchClass::new(10)).name(), "Bb");
    assert!(
        !notes.contains(&"A#".to_owned()),
        "F major must not show A#: {notes:?}"
    );
}

#[test]
fn ac_fb_2_1_f_major_scale_tones_are_f_g_a_bb_c_d_e() {
    let f = key("F", Mode::Major);
    let speller = Speller::new(f);
    let scale_pcs = [5, 7, 9, 10, 0, 2, 4];
    let got: Vec<String> = scale_pcs
        .iter()
        .map(|pc| speller.spell(PitchClass::new(*pc)).name())
        .collect();
    assert_eq!(got, ["F", "G", "A", "Bb", "C", "D", "E"]);
}

#[test]
fn g_1_5_gb_major_has_cb_and_d_sharp_minor_has_e_sharp() {
    assert_eq!(
        Speller::new(key("Gb", Mode::Major))
            .spell(PitchClass::new(11))
            .name(),
        "Cb"
    );
    assert_eq!(
        Speller::new(key("D#", Mode::NaturalMinor))
            .spell(PitchClass::new(5))
            .name(),
        "E#"
    );
}

#[test]
fn g_1_5_a_chromatic_note_in_g_major_is_sharp_in_c_major_flat() {
    assert_eq!(
        Speller::new(key("G", Mode::Major))
            .spell(PitchClass::new(10))
            .name(),
        "A#"
    );
    assert_eq!(
        Speller::new(key("C", Mode::Major))
            .spell(PitchClass::new(10))
            .name(),
        "Bb"
    );
}

#[test]
fn g_1_5_the_third_of_d7_in_c_is_f_sharp() {
    let c = key("C", Mode::Major);
    let speller = Speller::new(c).with_chord(chord("D7"));
    assert_eq!(speller.spell(PitchClass::new(6)).name(), "F#");
    // ... while without the chord, the same pitch class is the flat spelling.
    assert_eq!(Speller::new(c).spell(PitchClass::new(6)).name(), "Gb");
}

#[test]
fn g_1_5_the_root_of_the_flat_seven_chord_in_c_is_b_flat() {
    let c = key("C", Mode::Major);
    let bb = mackey_theory::roman_to_chord("bVII", &c).unwrap();
    assert_eq!(
        Speller::new(c)
            .with_chord(bb)
            .spell(PitchClass::new(10))
            .name(),
        "Bb"
    );
}

#[test]
fn g_1_5_chord_spelling_beats_the_scale_and_the_key_preference() {
    // A# is not Bb, even in F major whose scale has Bb: the chord the player typed decides.
    let f = key("F", Mode::Major);
    let speller = Speller::new(f).with_chord(chord("A#m"));
    assert_eq!(speller.spell(PitchClass::new(10)).name(), "A#");
    assert_eq!(speller.spell(PitchClass::new(1)).name(), "C#");
    assert_eq!(speller.spell(PitchClass::new(5)).name(), "E#");
    // Notes outside the chord still follow the scale.
    assert_eq!(speller.spell(PitchClass::new(7)).name(), "G");
}

#[test]
fn g_1_5_the_slash_bass_keeps_its_spelling() {
    let speller = Speller::new(key("C", Mode::Major)).with_chord(chord("D/F#"));
    assert_eq!(speller.spell(PitchClass::new(6)).name(), "F#");
}

#[test]
fn g_1_6_double_sharps_are_simplified_for_labels_and_kept_for_the_tooltip() {
    // D#7 (V7 of G# minor) has F## as its third; the fretboard label is G.
    let g_sharp_minor = key("G#", Mode::NaturalMinor);
    let speller = Speller::new(g_sharp_minor).with_chord(chord("D#7"));
    let shown = speller.display(PitchClass::new(7));
    assert_eq!(shown.label.name(), "G");
    assert_eq!(shown.theory.name(), "F##");
    assert!(shown.is_simplified());
    let plain = speller.display(PitchClass::new(3));
    assert_eq!(plain.label.name(), "D#");
    assert!(!plain.is_simplified());
}

#[test]
fn g_1_6_display_note_table() {
    // (theory spelling, label)
    for (theory, label) in [
        ("F##", "G"),
        ("Bbb", "A"),
        ("C##", "D"),
        ("B##", "C#"),
        ("Fbb", "Eb"),
        ("F#", "F#"),
        ("Bb", "Bb"),
    ] {
        let d = DisplayNote::of(note(theory));
        assert_eq!(d.label.name(), label, "case {theory}");
        assert_eq!(d.theory.name(), theory, "case {theory}");
    }
}

#[test]
fn g_1_6_no_label_in_any_key_chord_or_pitch_class_has_a_double_accidental() {
    let qualities = ChordQuality::ALL;
    for mode in [Mode::Major, Mode::NaturalMinor] {
        for k in Key::all_offered(mode) {
            for root_pc in 0..12 {
                for pref in [AccidentalPreference::Sharps, AccidentalPreference::Flats] {
                    let root = Note::from_pitch_class(PitchClass::new(root_pc), pref);
                    for quality in qualities {
                        let speller = Speller::new(k).with_chord(Chord::new(root, quality));
                        for pc in 0..12 {
                            let shown = speller.display(PitchClass::new(pc));
                            assert!(
                                !shown.label.is_double(),
                                "case {} {root}{}: pitch class {pc} -> {}",
                                k.name(),
                                quality.symbol(),
                                shown.label
                            );
                            assert_eq!(shown.label.pitch_class(), PitchClass::new(pc));
                        }
                    }
                }
            }
        }
    }
}
