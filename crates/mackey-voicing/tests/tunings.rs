//! FR-013: tunings, capo, fret count, handedness (B1, SAC-07).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]

use mackey_voicing::Inversion::{First, Root, Second, Third};
use mackey_voicing::{
    Capo, Fret, FretCount, Instrument, StringNumber, StringSet, Tuning, TuningPreset,
    TuningWarning, VoicingError,
};
use proptest::prelude::*;

const PRESETS: &[(TuningPreset, [&str; 6])] = &[
    (TuningPreset::Standard, ["E2", "A2", "D3", "G3", "B3", "E4"]),
    (TuningPreset::DropD, ["D2", "A2", "D3", "G3", "B3", "E4"]),
    (
        TuningPreset::DoubleDropD,
        ["D2", "A2", "D3", "G3", "B3", "D4"],
    ),
    (TuningPreset::Dadgad, ["D2", "A2", "D3", "G3", "A3", "D4"]),
    (TuningPreset::OpenG, ["D2", "G2", "D3", "G3", "B3", "D4"]),
    (TuningPreset::OpenD, ["D2", "A2", "D3", "F#3", "A3", "D4"]),
    (TuningPreset::OpenE, ["E2", "B2", "E3", "G#3", "B3", "E4"]),
    (
        TuningPreset::HalfStepDown,
        ["Eb2", "Ab2", "Db3", "Gb3", "Bb3", "Eb4"],
    ),
    (
        TuningPreset::WholeStepDown,
        ["D2", "G2", "C3", "F3", "A3", "D4"],
    ),
];

fn names(tuning: &Tuning) -> Vec<String> {
    tuning.strings.iter().map(|p| p.name()).collect()
}

#[test]
fn lh_2_every_preset_has_the_exact_pitches() {
    for (preset, expected) in PRESETS {
        let tuning = Tuning::preset(*preset);
        assert_eq!(names(&tuning), expected.to_vec(), "{preset:?}");
        assert_eq!(tuning.name, preset.display_name());
    }
    assert_eq!(PRESETS.len(), TuningPreset::ALL.len());
}

#[test]
fn lh_2_shift_from_standard() {
    let shift = |p| Tuning::preset(p).shift_from_standard();
    assert_eq!(shift(TuningPreset::Standard), Some(0));
    assert_eq!(shift(TuningPreset::HalfStepDown), Some(-1));
    assert_eq!(shift(TuningPreset::WholeStepDown), Some(-2));
    assert_eq!(shift(TuningPreset::DropD), None);
    assert!(Tuning::preset(TuningPreset::Standard).is_standard());
}

#[test]
fn lh_2_unusual_tension_boundaries_are_inclusive() {
    // Minor third below (-3) to a whole tone above (+2) the standard pitch is fine.
    let standard = TuningPreset::Standard.strings();
    let cases: &[(i32, bool)] = &[(-4, true), (-3, false), (0, false), (2, false), (3, true)];
    for &(offset, warns) in cases {
        let mut strings = standard;
        strings[2] = standard[2]
            .add_semitones(offset, mackey_theory::AccidentalPreference::Sharps)
            .unwrap();
        let (_, warnings) = Tuning::custom(strings).unwrap();
        assert_eq!(!warnings.is_empty(), warns, "offset {offset}");
        if warns {
            let TuningWarning::UnusualTension {
                string,
                standard: s,
                chosen,
            } = warnings[0];
            assert_eq!(string.number(), 4);
            assert_eq!(s, standard[2]);
            assert_eq!(chosen.midi(), standard[2].midi() + offset);
            assert_eq!(warnings[0].message(), "Unusual tension, check your strings");
        }
    }
}

#[test]
fn lh_2_custom_tuning_range_and_duplicates() {
    // A duplicate open string is allowed (E-FB-6).
    let mut strings = TuningPreset::Standard.strings();
    strings[1] = strings[0]
        .add_semitones(5, mackey_theory::AccidentalPreference::Sharps)
        .unwrap();
    strings[2] = strings[1];
    assert!(Tuning::custom(strings).is_ok());
    // Outside C1..=C6 is refused.
    let mut low = TuningPreset::Standard.strings();
    low[0] = low[0]
        .add_semitones(-20, mackey_theory::AccidentalPreference::Sharps)
        .unwrap();
    assert!(matches!(
        Tuning::custom(low),
        Err(VoicingError::TuningOutOfRange { string: 6, .. })
    ));
}

#[test]
fn ac_tu_5_1_open_strings_with_capo_two() {
    let guitar = Instrument {
        capo: Capo::new(2).unwrap(),
        ..Instrument::default()
    };
    let sounding: Vec<String> = StringNumber::LOW_TO_HIGH
        .iter()
        .map(|s| guitar.open_sounding_pitch(*s).name())
        .collect();
    assert_eq!(sounding, ["F#2", "B2", "E3", "A3", "C#4", "F#4"]);
}

#[test]
fn lh_3_pitch_at_errors_below_the_capo_and_past_the_neck() {
    let guitar = Instrument {
        capo: Capo::new(5).unwrap(),
        ..Instrument::default()
    };
    let s3 = StringNumber::new(3).unwrap();
    assert!(matches!(
        guitar.pitch_at(s3, Fret::new(4).unwrap()),
        Err(VoicingError::BelowCapo { .. })
    ));
    assert_eq!(
        guitar.pitch_at(s3, Fret::new(5).unwrap()).unwrap().name(),
        "C4"
    );
    assert!(matches!(
        guitar.pitch_at(s3, Fret::new(23).unwrap()),
        Err(VoicingError::FretOutOfRange(23))
    ));
}

#[test]
fn newtypes_validate_on_construction_and_deserialisation() {
    assert!(StringNumber::new(0).is_err() && StringNumber::new(7).is_err());
    assert!(Fret::new(24).is_ok() && Fret::new(25).is_err());
    assert!(Capo::new(12).is_ok() && Capo::new(13).is_err());
    assert!(FretCount::new(18).is_err() && FretCount::new(19).is_ok());
    assert!(FretCount::new(24).is_ok() && FretCount::new(25).is_err());
    assert_eq!(FretCount::default().value(), 22);
    assert!(serde_json::from_str::<Capo>("13").is_err());
    assert!(serde_json::from_str::<FretCount>("30").is_err());
    assert_eq!(serde_json::from_str::<Fret>("7").unwrap().value(), 7);
    assert_eq!(StringNumber::from_index(0).unwrap().number(), 6);
    assert_eq!(StringNumber::from_index(5).unwrap().number(), 1);
    assert!(StringNumber::from_index(6).is_none());
    assert_eq!(Fret::new(3).unwrap().up().unwrap().value(), 4);
    assert!(Fret::new(24).unwrap().up().is_none());
    assert!(Fret::OPEN.down().is_none());
}

#[test]
fn fb_6_string_sets_and_inversions() {
    let numbers = |set: StringSet| set.strings().map(StringNumber::number);
    assert_eq!(numbers(StringSet::S123), [1, 2, 3]);
    assert_eq!(numbers(StringSet::S456), [4, 5, 6]);
    let mut set = StringSet::S123;
    let mut seen = Vec::new();
    for _ in 0..4 {
        seen.push(set.label());
        set = set.next();
    }
    assert_eq!(seen, ["123", "234", "345", "456"]);
    assert_eq!(set, StringSet::S123);
    assert_eq!(StringSet::S123.prev(), StringSet::S456);
    assert_eq!(Root.next(false), First);
    assert_eq!(Second.next(false), Root);
    assert_eq!(Second.next(true), Third);
    assert_eq!(Third.next(true), Root);
    assert_eq!(Root.prev(false), Second);
    assert_eq!(Root.prev(true), Third);
    assert_eq!(Third.clamped(false), Second);
}

proptest! {
    #[test]
    fn each_fret_is_one_semitone_higher(string in 1u8..=6, fret in 0u8..22, capo in 0u8..=12, count in 19u8..=24) {
        let fret = fret.max(capo);
        let guitar = Instrument {
            capo: Capo::new(capo).unwrap(),
            frets: FretCount::new(count).unwrap(),
            ..Instrument::default()
        };
        let s = StringNumber::new(string).unwrap();
        if fret < count {
            let here = guitar.pitch_at(s, Fret::new(fret).unwrap()).unwrap();
            let next = guitar.pitch_at(s, Fret::new(fret + 1).unwrap()).unwrap();
            prop_assert_eq!(next.midi(), here.midi() + 1);
        }
    }
}
