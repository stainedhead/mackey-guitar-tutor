//! FR-018: scale patterns and follow-the-chords (AC-SC-3.1, AC-SC-4.1, SC-3, SC-4).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]

use mackey_theory::{Key, Mode, Scale, parse_chord_symbol};
use mackey_voicing::{
    Capo, FollowReason, Instrument, PatternKind, ScalePattern, Tuning, TuningPreset, full_neck,
    scale_for_chord, scale_patterns,
};

fn scale(tonic: &str, mode: Mode) -> Scale {
    Scale {
        tonic: tonic.parse().unwrap(),
        mode,
    }
}

fn frets_on(p: &ScalePattern, string: u8) -> Vec<u8> {
    p.notes
        .iter()
        .filter(|n| n.string.number() == string)
        .map(|n| n.fret.value())
        .collect()
}

fn box_number(p: &ScalePattern) -> u8 {
    match p.kind {
        PatternKind::PentatonicBox { number } | PatternKind::ThreeNps { number } => number,
        PatternKind::FullNeck => 0,
    }
}

#[test]
fn ac_sc_3_1_a_minor_pentatonic_box_1() {
    let patterns = scale_patterns(&scale("A", Mode::MinorPentatonic), &Instrument::default());
    let one = patterns.iter().find(|p| box_number(p) == 1).unwrap();
    for string in [6, 2, 1] {
        assert_eq!(frets_on(one, string), [5, 8], "string {string}");
    }
    for string in [5, 4, 3] {
        assert_eq!(frets_on(one, string), [5, 7], "string {string}");
    }
    let roots: Vec<(u8, u8)> = one
        .notes
        .iter()
        .filter(|n| n.degree.value() == 1)
        .map(|n| (n.string.number(), n.fret.value()))
        .collect();
    assert_eq!(roots, [(6, 5), (4, 7), (1, 5)]);
}

#[test]
fn sc_3_all_five_a_minor_pentatonic_boxes_match_the_hand_table() {
    // Hand-derived from the open strings E A D G B E and A C D E G. Boxes 4 and 5 sit at the nut
    // (they are the 12th-fret boxes one octave down).
    // (box, [string 6..1 frets])
    const BOXES: &[(u8, [&[u8]; 6])] = &[
        (1, [&[5, 8], &[5, 7], &[5, 7], &[5, 7], &[5, 8], &[5, 8]]),
        (
            2,
            [&[8, 10], &[7, 10], &[7, 10], &[7, 9], &[8, 10], &[8, 10]],
        ),
        (
            3,
            [
                &[10, 12],
                &[10, 12],
                &[10, 12],
                &[9, 12],
                &[10, 13],
                &[10, 12],
            ],
        ),
        (4, [&[0, 3], &[0, 3], &[0, 2], &[0, 2], &[1, 3], &[0, 3]]),
        (5, [&[3, 5], &[3, 5], &[2, 5], &[2, 5], &[3, 5], &[3, 5]]),
    ];
    let patterns = scale_patterns(&scale("A", Mode::MinorPentatonic), &Instrument::default());
    assert_eq!(patterns.len(), 5);
    for (number, rows) in BOXES {
        let p = patterns.iter().find(|p| box_number(p) == *number).unwrap();
        for (i, want) in rows.iter().enumerate() {
            let string = u8::try_from(6 - i).unwrap();
            assert_eq!(&frets_on(p, string), want, "box {number} string {string}");
        }
    }
    // Ordered by box number, not by neck position (FR-001 of the auto review).
    let numbers: Vec<u8> = patterns.iter().map(box_number).collect();
    assert_eq!(numbers, [1, 2, 3, 4, 5]);
}

#[test]
fn sc_3_c_major_three_notes_per_string_pattern_3_hand_table() {
    // Pattern 3 starts on E (the 3rd degree) on string 6: E F G | A B C | D E F | G A B | C D E | F G A.
    let patterns = scale_patterns(&scale("C", Mode::Major), &Instrument::default());
    assert_eq!(patterns.len(), 7);
    let p = patterns.iter().find(|p| box_number(p) == 3).unwrap();
    let want: [&[u8]; 6] = [
        &[0, 1, 3],
        &[0, 2, 3],
        &[0, 2, 3],
        &[0, 2, 4],
        &[1, 3, 5],
        &[1, 3, 5],
    ];
    for (i, w) in want.iter().enumerate() {
        let string = u8::try_from(6 - i).unwrap();
        assert_eq!(&frets_on(p, string), w, "string {string}");
    }
}

/// Every pattern, every scale and tuning: all notes belong to the scale, each string has 2 (box)
/// or 3 (3nps) notes, the degree matches the scale, and strings rise in pitch.
#[test]
fn sc_3_property_patterns_are_wellformed_in_every_tuning_and_capo() {
    let modes = [
        Mode::Major,
        Mode::NaturalMinor,
        Mode::Dorian,
        Mode::Mixolydian,
        Mode::MajorPentatonic,
        Mode::MinorPentatonic,
    ];
    let tonics = ["C", "Db", "E", "F#", "A", "Bb"];
    let mut count = 0;
    for preset in TuningPreset::ALL {
        for capo in [0u8, 2, 5] {
            let ins = Instrument {
                tuning: Tuning::preset(preset),
                capo: Capo::new(capo).unwrap(),
                ..Instrument::default()
            };
            for mode in modes {
                for tonic in tonics {
                    let sc = scale(tonic, mode);
                    for p in scale_patterns(&sc, &ins) {
                        let per = match p.kind {
                            PatternKind::PentatonicBox { .. } => 2,
                            _ => 3,
                        };
                        let mut last_pitch = i32::MIN;
                        for n in &p.notes {
                            assert!(n.fret.value() >= capo, "{tonic} {mode:?}: below capo");
                            let idx = 6 - usize::from(n.string.number());
                            let midi = ins.tuning.midis()[idx] + i32::from(n.fret.value());
                            let pc = mackey_theory::PitchClass::new(midi);
                            assert_eq!(
                                sc.degree_of(pc),
                                Some(n.degree),
                                "{tonic} {mode:?} {preset:?}"
                            );
                            assert!(midi > last_pitch, "{tonic} {mode:?}: strictly ascending");
                            last_pitch = midi;
                        }
                        for string in 1..=6u8 {
                            assert_eq!(
                                frets_on(&p, string).len(),
                                per,
                                "{tonic} {mode:?} {preset:?} capo {capo} {:?} string {string}",
                                p.kind
                            );
                        }
                        assert!(p.lowest_fret <= p.highest_fret);
                        count += 1;
                    }
                }
            }
        }
    }
    assert!(count > 1_000);
}

#[test]
fn sc_3_full_neck_holds_every_scale_note_from_the_capo_up() {
    let ins = Instrument {
        capo: Capo::new(2).unwrap(),
        ..Instrument::default()
    };
    let sc = scale("F", Mode::Major);
    let all = full_neck(&sc, &ins);
    assert_eq!(all.kind, PatternKind::FullNeck);
    let mut expected = 0;
    for string in 1..=6u8 {
        for fret in 2..=ins.frets.value() {
            let midi = ins.tuning.midis()[6 - usize::from(string)] + i32::from(fret);
            if sc.degree_of(mackey_theory::PitchClass::new(midi)).is_some() {
                expected += 1;
            }
        }
    }
    assert_eq!(all.notes.len(), expected);
    assert!(all.notes.iter().all(|n| n.fret.value() >= 2));
}

#[test]
fn sc_3_scales_without_positions_use_the_full_neck() {
    let ins = Instrument::default();
    assert_eq!(scale_patterns(&scale("A", Mode::Blues), &ins), []);
    assert_eq!(scale_patterns(&scale("A", Mode::HarmonicMinor), &ins), []);
}

#[test]
fn ac_sc_4_1_song_in_c_a7_follows_to_a_mixolydian() {
    let key = Key::new("C".parse().unwrap(), Mode::Major).unwrap();
    let c = scale_for_chord(&key, &parse_chord_symbol("C").unwrap());
    assert_eq!(c.scale.name(), "C major");
    assert_eq!(c.reason, FollowReason::Diatonic);
    let a7 = scale_for_chord(&key, &parse_chord_symbol("A7").unwrap());
    assert_eq!(a7.scale.name(), "A Mixolydian");
    assert_eq!(a7.reason, FollowReason::DominantMixolydian);
}

#[test]
fn sc_4_rule_table_for_chords_outside_the_key() {
    // Key of C major. (chord, scale name, reason)
    const CASES: &[(&str, &str, FollowReason)] = &[
        ("D7", "D Mixolydian", FollowReason::DominantMixolydian),
        ("E", "E major", FollowReason::MajorIonian),
        ("Bb", "Bb major", FollowReason::MajorIonian),
        ("Cm", "C Dorian", FollowReason::MinorDorian),
        ("F#m", "F# Dorian", FollowReason::MinorDorian),
        ("F#dim", "F# Locrian", FollowReason::DiminishedLocrian),
        ("Bdim", "C major", FollowReason::Diatonic),
        ("Dm", "C major", FollowReason::Diatonic),
        ("G7", "C major", FollowReason::Diatonic),
    ];
    let key = Key::new("C".parse().unwrap(), Mode::Major).unwrap();
    for (symbol, name, reason) in CASES {
        let got = scale_for_chord(&key, &parse_chord_symbol(symbol).unwrap());
        assert_eq!(got.scale.name(), *name, "{symbol}");
        assert_eq!(got.reason, *reason, "{symbol}");
    }
}
