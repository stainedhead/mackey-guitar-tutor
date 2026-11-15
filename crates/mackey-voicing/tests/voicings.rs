//! FR-015: voicing search, scoring, grouping, fingering, "no playable voicing" (B4, B5, SAC-09).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]

mod common;

use common::check_fb_5_4;
use mackey_theory::{Chord, ChordQuality, parse_chord_symbol};
use mackey_voicing::{
    Capo, Fret, Instrument, Origin, RelaxOption, StringState, Tuning, TuningPreset, VoicingGroup,
    VoicingKind, VoicingOptions, VoicingResult, assign_fingering, voicings_for,
};

fn chord(symbol: &str) -> Chord {
    parse_chord_symbol(symbol).unwrap()
}

fn instrument(preset: TuningPreset, capo: u8) -> Instrument {
    Instrument {
        tuning: Tuning::preset(preset),
        capo: Capo::new(capo).unwrap(),
        ..Instrument::default()
    }
}

fn frets_of(ins: &Instrument, c: &str) -> Vec<String> {
    voicings_for(&chord(c), ins, &VoicingOptions::default())
        .voicings()
        .iter()
        .map(|v| v.fret_string(ins.capo))
        .collect()
}

#[test]
fn ac_fb_5_1_c_voicing_view_starts_with_open_c_and_its_fingers() {
    let ins = Instrument::default();
    let result = voicings_for(&chord("C"), &ins, &VoicingOptions::default());
    let first = &result.voicings()[0];
    assert_eq!(first.fret_string(ins.capo), "x32010");
    assert_eq!(
        first.fingering.fingers,
        [None, Some(3), Some(2), Some(0), Some(1), Some(0)]
    );
    assert_eq!(first.name, "Open C");
    assert_eq!(first.group, VoicingGroup::Open);
    assert_eq!(first.origin, Origin::Curated);
}

#[test]
fn ac_fb_5_2_f_list_has_small_f_and_the_full_barre_and_both_obey_fb_5_4() {
    let ins = Instrument::default();
    let f = chord("F");
    let result = voicings_for(&f, &ins, &VoicingOptions::default());
    let list = result.voicings();
    for wanted in ["xx3211", "133211"] {
        let v = list
            .iter()
            .find(|v| v.fret_string(ins.capo) == wanted)
            .unwrap_or_else(|| panic!("{wanted}"));
        check_fb_5_4(&f, &ins, v, false).unwrap();
        assert!(!v.fingering.barres.is_empty(), "{wanted} is barred");
    }
    let full = list
        .iter()
        .find(|v| v.fret_string(ins.capo) == "133211")
        .unwrap();
    assert_eq!(full.fingering.barres.len(), 1);
    let barre = full.fingering.barres[0];
    assert_eq!(
        (barre.fret.value(), barre.from.number(), barre.to.number()),
        (1, 6, 1)
    );
}

#[test]
fn fb_5_2_groups_are_ordered_and_named() {
    let ins = Instrument::default();
    for symbol in ["C", "G", "F", "Bm", "A7", "Dm7", "Bb"] {
        let result = voicings_for(&chord(symbol), &ins, &VoicingOptions::default());
        let groups: Vec<VoicingGroup> = result.voicings().iter().map(|v| v.group).collect();
        let mut sorted = groups.clone();
        sorted.sort_by_key(|g| match g {
            VoicingGroup::Open => 0,
            VoicingGroup::BarreAndCaged => 1,
            VoicingGroup::Other => 2,
        });
        assert_eq!(
            groups, sorted,
            "{symbol}: Open, then Barre and CAGED, then Other"
        );
        for v in result.voicings() {
            assert_ne!(v.name, "");
            match v.group {
                VoicingGroup::Open => assert!(
                    v.strings.iter().any(|s| matches!(s, StringState::Open))
                        || v.kind == VoicingKind::Open,
                    "{symbol} {}",
                    v.fret_string(ins.capo)
                ),
                VoicingGroup::BarreAndCaged => assert!(
                    matches!(v.kind, VoicingKind::Caged { .. } | VoicingKind::Barre),
                    "{symbol}"
                ),
                VoicingGroup::Other => {}
            }
        }
    }
    let list = voicings_for(&chord("G"), &ins, &VoicingOptions::default());
    assert!(
        list.voicings()
            .iter()
            .any(|v| v.name == "E-shape barre, fret 3"),
        "named like the PRD example"
    );
}

/// `(chord, tuning, capo, expected first voicing)`: hand-checked note by note.
const FIRST: &[(&str, TuningPreset, u8, &str)] = &[
    ("C", TuningPreset::Standard, 0, "x32010"),
    ("D", TuningPreset::Standard, 0, "xx0232"),
    ("G", TuningPreset::Standard, 0, "320003"),
    ("Bm", TuningPreset::Standard, 0, "x20402"),
    ("F#m", TuningPreset::Standard, 0, "20422x"),
    ("Bb", TuningPreset::Standard, 0, "x10331"),
    ("E7", TuningPreset::Standard, 0, "020100"),
    ("Cmaj7", TuningPreset::Standard, 0, "x32000"),
    ("Gsus4", TuningPreset::Standard, 0, "3300xx"),
    ("Bdim", TuningPreset::Standard, 0, "x20401"),
    ("Caug", TuningPreset::Standard, 0, "x32110"),
    ("A5", TuningPreset::Standard, 0, "x022xx"),
    ("D9", TuningPreset::Standard, 0, "(10)9(10)9(10)0"),
    ("G6", TuningPreset::Standard, 0, "320000"),
    ("C", TuningPreset::Standard, 2, "x10331"),
    ("G", TuningPreset::Standard, 3, "022100"),
    ("D", TuningPreset::DropD, 0, "000232"),
    ("G", TuningPreset::DropD, 0, "x(10)9787"),
    ("Dm", TuningPreset::DropD, 0, "000231"),
    ("C", TuningPreset::DoubleDropD, 0, "x32012"),
    ("D", TuningPreset::Dadgad, 0, "000204"),
    ("G", TuningPreset::OpenG, 0, "x00000"),
    ("D", TuningPreset::OpenD, 0, "000000"),
    ("E", TuningPreset::OpenE, 0, "000000"),
    ("Eb", TuningPreset::HalfStepDown, 0, "022100"),
    ("D", TuningPreset::WholeStepDown, 0, "022100"),
    ("A", TuningPreset::Standard, 2, "320003"),
    ("E", TuningPreset::HalfStepDown, 1, "022100"),
    ("D/F#", TuningPreset::Standard, 0, "2002xx"),
    ("C/G", TuningPreset::Standard, 0, "332010"),
    ("G/B", TuningPreset::Standard, 0, "x2000x"),
    ("Am7", TuningPreset::Standard, 5, "020000"),
];

#[test]
fn fb_5_1_first_voicing_table_over_tunings_and_capos() {
    assert!(FIRST.len() >= 30);
    for (symbol, preset, capo, expected) in FIRST {
        let ins = instrument(*preset, *capo);
        let c = chord(symbol);
        let result = voicings_for(&c, &ins, &VoicingOptions::default());
        let list = result.voicings();
        assert_eq!(
            list[0].fret_string(ins.capo),
            *expected,
            "{symbol} {preset:?} capo {capo}"
        );
        for v in list {
            check_fb_5_4(&c, &ins, v, false).unwrap_or_else(|e| {
                panic!(
                    "{symbol} {preset:?} capo {capo} {}: {e}",
                    v.fret_string(ins.capo)
                )
            });
        }
        assert!(list.len() <= 24);
    }
}

#[test]
fn lh_4_search_voicings_are_marked_in_non_curated_tunings_only() {
    let standard = voicings_for(
        &chord("Bm"),
        &Instrument::default(),
        &VoicingOptions::default(),
    );
    assert!(
        standard
            .voicings()
            .iter()
            .all(|v| v.name != "Found by search")
    );
    let dadgad = instrument(TuningPreset::Dadgad, 0);
    let result = voicings_for(&chord("Bm"), &dadgad, &VoicingOptions::default());
    assert!(
        result
            .voicings()
            .iter()
            .all(|v| v.origin == Origin::Search && v.name == "Found by search")
    );
}

#[test]
fn ac_fb_5_6_no_playable_voicing_offers_both_relaxations_and_never_breaks_fb_5_4() {
    // Every string tuned to E: C major needs C, E and G, which are 8 frets apart at best.
    let e3 = TuningPreset::Standard.strings()[5]
        .add_semitones(-12, mackey_theory::AccidentalPreference::Sharps)
        .unwrap();
    let (tuning, _) = Tuning::custom([e3; 6]).unwrap();
    let ins = Instrument::with_tuning(tuning);
    let c = chord("C");
    let result = voicings_for(&c, &ins, &VoicingOptions::default());
    assert_eq!(
        result,
        VoicingResult::NoneFound {
            options: vec![RelaxOption::AllowStretches, RelaxOption::AllowOmittedFifth]
        }
    );
    assert_eq!(
        result.none_message(),
        Some("No playable voicing in this tuning")
    );
    assert_eq!(result.voicings(), []);
    assert_eq!(RelaxOption::AllowStretches.label(), "Allow stretches");
    assert_eq!(
        RelaxOption::AllowOmittedFifth.label(),
        "Allow omitted fifth"
    );
    // Switching one on removes it from the offer.
    let result = voicings_for(
        &c,
        &ins,
        &VoicingOptions {
            allow_stretch: true,
            ..Default::default()
        },
    );
    assert_eq!(
        result,
        VoicingResult::NoneFound {
            options: vec![RelaxOption::AllowOmittedFifth]
        }
    );
}

#[test]
fn fb_5_4_allow_stretches_finds_what_the_four_fret_span_could_not() {
    // Strings a semitone apart: C7 cannot be fingered inside 4 frets but can inside 5.
    let strings: Vec<_> = (0..6)
        .map(|i| {
            mackey_theory::Pitch::from_midi(40 + i, mackey_theory::AccidentalPreference::Sharps)
                .unwrap()
        })
        .collect();
    let (tuning, _) = Tuning::custom(strings.try_into().unwrap()).unwrap();
    let ins = Instrument::with_tuning(tuning);
    let c7 = chord("C7");
    let plain = voicings_for(&c7, &ins, &VoicingOptions::default());
    assert!(matches!(plain, VoicingResult::NoneFound { .. }));
    let stretched = voicings_for(
        &c7,
        &ins,
        &VoicingOptions {
            allow_stretch: true,
            ..Default::default()
        },
    );
    assert_ne!(stretched.voicings(), []);
    for v in stretched.voicings() {
        check_fb_5_4(&c7, &ins, v, true).unwrap();
        assert_eq!(v.span, 5);
    }
}

#[test]
fn fb_5_4_allow_omitted_fifth_lets_a_triad_drop_its_fifth() {
    let ins = Instrument::default();
    let c = chord("C");
    let has_fifth = |v: &mackey_voicing::Voicing| {
        common::roles(&c, &ins, v).contains(&mackey_theory::ToneRole::Fifth)
    };
    let strict = voicings_for(
        &c,
        &ins,
        &VoicingOptions {
            max_results: 200,
            ..Default::default()
        },
    );
    assert!(
        strict.voicings().iter().all(has_fifth),
        "a triad keeps its fifth by default"
    );
    let relaxed = voicings_for(
        &c,
        &ins,
        &VoicingOptions {
            allow_omitted_fifth: true,
            max_results: 200,
            ..Default::default()
        },
    );
    assert!(relaxed.voicings().iter().any(|v| !has_fifth(v)));
    for v in relaxed.voicings() {
        check_fb_5_4(&c, &ins, v, false).unwrap();
    }
}

#[test]
fn fb_5_4_seventh_chords_may_leave_out_the_fifth_but_not_the_seventh() {
    let ins = Instrument::default();
    let c7 = chord("C7");
    let result = voicings_for(&c7, &ins, &VoicingOptions::default());
    // The curated open C7 (x32310) has no G.
    let open = result
        .voicings()
        .iter()
        .find(|v| v.fret_string(ins.capo) == "x32310")
        .unwrap();
    assert!(!common::roles(&c7, &ins, open).contains(&mackey_theory::ToneRole::Fifth));
    assert!(
        result
            .voicings()
            .iter()
            .all(|v| common::roles(&c7, &ins, v).contains(&mackey_theory::ToneRole::Seventh))
    );
}

#[test]
fn fb_5_4_slash_chords_have_the_required_bass() {
    let ins = Instrument::default();
    for symbol in ["D/F#", "C/G", "G/B", "Am/C", "C/E", "F/A"] {
        let c = chord(symbol);
        let result = voicings_for(&c, &ins, &VoicingOptions::default());
        assert!(!result.voicings().is_empty(), "{symbol}");
        for v in result.voicings() {
            check_fb_5_4(&c, &ins, v, false)
                .unwrap_or_else(|e| panic!("{symbol} {}: {e}", v.fret_string(ins.capo)));
        }
    }
}

#[test]
fn lh_3_under_a_capo_frets_are_absolute_and_open_means_open_at_the_capo() {
    let ins = instrument(TuningPreset::Standard, 3);
    let c = chord("C");
    let result = voicings_for(&c, &ins, &VoicingOptions::default());
    for v in result.voicings() {
        for s in &v.strings {
            if let Some(f) = s.fretted() {
                assert!(f > 3, "pressed frets are above the capo");
            }
        }
        check_fb_5_4(&c, &ins, v, false).unwrap();
    }
    // Chord C under capo 3 is the A shape: x02220 relative, absolute 5 5 5 on strings 4 3 2.
    let a_shape = result
        .voicings()
        .iter()
        .find(|v| v.fret_string(ins.capo) == "x02220")
        .unwrap();
    assert_eq!(
        a_shape.strings[3],
        StringState::Fretted {
            fret: Fret::new(5).unwrap()
        }
    );
    assert_eq!(a_shape.strings[1], StringState::Open);
}

#[test]
fn e_fb_6_identical_open_strings_are_not_double_counted_in_fingering() {
    // Strings 6 and 5 both tuned to A2: pressing fret 2 on both is one finger (one note, one fingertip).
    let std = TuningPreset::Standard.strings();
    let mut strings = std;
    strings[0] = std[1];
    let (tuning, _) = Tuning::custom(strings).unwrap();
    let midis = tuning.midis();
    let f = |n: u8| StringState::Fretted {
        fret: Fret::new(n).unwrap(),
    };
    let states = [
        f(2),
        f(2),
        f(2),
        StringState::Muted,
        StringState::Muted,
        StringState::Muted,
    ];
    let fingering = assign_fingering(&states, midis).unwrap();
    assert_eq!(fingering.fingers[0], Some(1));
    assert_eq!(fingering.fingers[1], Some(1));
    // String 4 (D) at the same fret is a different pitch: with the doubled A strings it is a barre.
    assert_eq!(fingering.fingers[2], Some(1));
    let states = [f(2), f(2), f(4), f(4), f(5), f(5)];
    // Six pressed notes of four distinct fret/pitch groups need more than four fingers: rejected.
    assert!(assign_fingering(&states, midis).is_none());
}

/// `(frets, expected fingers, expected barre (fret, from, to))`, standard tuning.
#[allow(clippy::type_complexity)]
const FINGERING: &[(&str, &str, Option<(u8, u8, u8)>)] = &[
    ("x32010", "x32010", None),
    ("133211", "134211", Some((1, 6, 1))),
    ("xx3211", "xx3211", Some((1, 2, 1))),
    ("x02220", "x01110", Some((2, 4, 2))),
    ("320003", "210003", None),
    ("x24432", "x13421", Some((2, 5, 1))),
    ("xxxxxx", "xxxxxx", None),
    ("000000", "000000", None),
    ("x0x0x0", "x0x0x0", None),
    ("1x2x3x", "1x2x3x", None),
];

fn parse_states(text: &str) -> [StringState; 6] {
    let mut out = [StringState::Muted; 6];
    for (slot, c) in out.iter_mut().zip(text.chars()) {
        *slot = match c {
            'x' => StringState::Muted,
            '0' => StringState::Open,
            d => StringState::Fretted {
                fret: Fret::new(u8::try_from(d.to_digit(10).unwrap()).unwrap()).unwrap(),
            },
        };
    }
    out
}

#[test]
fn fb_5_5_fingering_table_with_barres() {
    let midis = TuningPreset::Standard
        .strings()
        .map(mackey_theory::Pitch::midi);
    for (frets, fingers, barre) in FINGERING {
        let got = assign_fingering(&parse_states(frets), midis).unwrap();
        let text: String = got
            .fingers
            .iter()
            .map(|f| f.map_or("x".to_owned(), |n| n.to_string()))
            .collect();
        assert_eq!(text, *fingers, "{frets}");
        let got_barre = got
            .barres
            .first()
            .map(|b| (b.fret.value(), b.from.number(), b.to.number()));
        assert_eq!(got_barre, *barre, "{frets}");
    }
}

#[test]
fn fb_5_4_five_fingers_are_rejected() {
    let midis = TuningPreset::Standard
        .strings()
        .map(mackey_theory::Pitch::midi);
    // Five different frets on five strings need five fingers.
    assert!(assign_fingering(&parse_states("x12345"), midis).is_none());
    // Four different pressed frets are exactly four fingers (open strings need none): accepted.
    let ok = assign_fingering(&parse_states("1023x4"), midis).unwrap();
    assert_eq!(ok.fingers.iter().flatten().filter(|&&f| f > 0).count(), 4);
}

#[test]
fn fb_5_1_results_are_capped_and_deterministic() {
    let ins = Instrument::default();
    let a = voicings_for(&chord("Am"), &ins, &VoicingOptions::default());
    let b = voicings_for(&chord("Am"), &ins, &VoicingOptions::default());
    assert_eq!(a, b);
    assert_eq!(a.voicings().len(), 24);
    let small = voicings_for(
        &chord("Am"),
        &ins,
        &VoicingOptions {
            max_results: 6,
            ..Default::default()
        },
    );
    assert_eq!(small.voicings().len(), 6);
    assert_eq!(frets_of(&ins, "Am")[0], "x02210");
}

#[test]
fn lh_4_every_quality_has_a_voicing_in_standard_tuning() {
    let ins = Instrument::default();
    for q in ChordQuality::ALL {
        for root in ["C", "F#", "Bb"] {
            let c = Chord::new(root.parse().unwrap(), q);
            let result = voicings_for(&c, &ins, &VoicingOptions::default());
            assert!(!result.voicings().is_empty(), "{root}{}", q.symbol());
        }
    }
}
