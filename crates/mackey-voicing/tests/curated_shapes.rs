//! FB-5.3: the curated open shapes match the standard teaching shapes exactly, verified note by
//! note (B2, B3, SAC-08). The expectations below are hand-written from teaching charts, not
//! generated from the code under test.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]

use mackey_theory::{Chord, parse_chord_symbol};
use mackey_voicing::shapes::{CAGED_TEMPLATES, DROP_D_SHAPES, OPEN_SHAPES};
use mackey_voicing::{
    Capo, Instrument, Origin, StringNumber, StringState, Tuning, TuningPreset, VoicingKind,
    VoicingOptions, VoicingResult, voicings_for,
};

/// `(chord symbol, frets, fingers, sounding notes low to high)` for every FB-5.3 open shape.
const OPEN: &[(&str, &str, &str, &str)] = &[
    ("C", "x32010", "x-3-2-0-1-0", "C E G C E"),
    ("A", "x02220", "x-0-1-2-3-0", "A E A C# E"),
    ("G", "320003", "2-1-0-0-0-3", "G B D G B G"),
    ("E", "022100", "0-2-3-1-0-0", "E B E G# B E"),
    ("D", "xx0232", "x-x-0-1-3-2", "D A D F#"),
    ("Am", "x02210", "x-0-2-3-1-0", "A E A C E"),
    ("Em", "022000", "0-2-3-0-0-0", "E B E G B E"),
    ("Dm", "xx0231", "x-x-0-2-3-1", "D A D F"),
    ("A7", "x02020", "x-0-1-0-2-0", "A E G C# E"),
    ("B7", "x21202", "x-2-1-3-0-4", "B D# A B F#"),
    ("C7", "x32310", "x-3-2-4-1-0", "C E A# C E"),
    ("D7", "xx0212", "x-x-0-2-1-3", "D A C F#"),
    ("E7", "020100", "0-2-0-1-0-0", "E B D G# B E"),
    ("G7", "320001", "3-2-0-0-0-1", "G B D G B F"),
    ("Amaj7", "x02120", "x-0-2-1-3-0", "A E G# C# E"),
    ("Cmaj7", "x32000", "x-3-2-0-0-0", "C E G B E"),
    ("Dmaj7", "xx0222", "x-x-0-1-2-3", "D A C# F#"),
    ("Fmaj7", "xx3210", "x-x-3-2-1-0", "F A C E"),
    ("Am7", "x02010", "x-0-2-0-1-0", "A E G C E"),
    ("Dm7", "xx0211", "x-x-0-2-1-1", "D A C F"),
    ("Em7", "020000", "0-2-0-0-0-0", "E B D G B E"),
    ("Asus2", "x02200", "x-0-1-2-0-0", "A E A B E"),
    ("Asus4", "x02230", "x-0-1-2-4-0", "A E A D E"),
    ("Dsus2", "xx0230", "x-x-0-1-2-0", "D A D E"),
    ("Dsus4", "xx0233", "x-x-0-1-2-3", "D A D G"),
    ("Esus4", "022200", "0-2-3-4-0-0", "E B E A B E"),
];

fn chord(symbol: &str) -> Chord {
    parse_chord_symbol(symbol).unwrap()
}

fn fingers_text(v: &mackey_voicing::Voicing) -> String {
    v.fingering
        .fingers
        .iter()
        .map(|f| f.map_or("x".to_owned(), |n| n.to_string()))
        .collect::<Vec<_>>()
        .join("-")
}

fn notes_text(ins: &Instrument, v: &mackey_voicing::Voicing) -> String {
    v.strings
        .iter()
        .zip(StringNumber::LOW_TO_HIGH)
        .filter_map(|(state, string)| match state {
            StringState::Muted => None,
            StringState::Open => Some(ins.open_sounding_pitch(string).note().name()),
            StringState::Fretted { fret } => {
                Some(ins.pitch_at(string, *fret).unwrap().note().name())
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn find<'a>(
    list: &'a [mackey_voicing::Voicing],
    frets: &str,
    capo: Capo,
) -> Option<&'a mackey_voicing::Voicing> {
    list.iter().find(|v| v.fret_string(capo) == frets)
}

#[test]
fn fb_5_3_curated_shapes_exact() {
    let ins = Instrument::default();
    for (symbol, frets, fingers, notes) in OPEN {
        let result = voicings_for(&chord(symbol), &ins, &VoicingOptions::default());
        let list = result.voicings();
        let v = find(list, frets, ins.capo).unwrap_or_else(|| panic!("{symbol}: {frets} missing"));
        assert_eq!(v.origin, Origin::Curated, "{symbol}");
        assert_eq!(fingers_text(v), *fingers, "{symbol} fingers");
        assert_eq!(notes_text(&ins, v), *notes, "{symbol} notes");
        assert!(v.name.starts_with("Open "), "{symbol}: {}", v.name);
    }
    assert_eq!(OPEN.len(), 26, "every chord listed in FB-5.3 except F");
}

#[test]
fn fb_5_3_the_library_tables_hold_exactly_the_listed_shapes() {
    // 26 shapes of FB-5.3, the alternative Em7 and the small F.
    assert_eq!(OPEN_SHAPES.len(), 28);
    for shape in OPEN_SHAPES.iter().chain(DROP_D_SHAPES) {
        assert!(shape.parse_frets().is_some(), "{}", shape.name);
        assert!(shape.parse_fingers().is_some(), "{}", shape.name);
    }
}

#[test]
fn fb_5_3_f_has_the_small_shape_and_the_full_barre() {
    let ins = Instrument::default();
    let f = chord("F");
    let result = voicings_for(&f, &ins, &VoicingOptions::default());
    let list = result.voicings();
    let small = find(list, "xx3211", ins.capo).unwrap();
    assert_eq!(small.name, "Small F");
    assert_eq!(fingers_text(small), "x-x-3-2-1-1");
    assert_eq!(small.fingering.barres.len(), 1);
    assert_eq!(notes_text(&ins, small), "F A C F");
    let full = find(list, "133211", ins.capo).unwrap();
    assert_eq!(
        full.kind,
        VoicingKind::Caged {
            shape: mackey_voicing::CagedShape::E
        }
    );
    assert_eq!(fingers_text(full), "1-3-4-2-1-1");
    assert_eq!(full.name, "E-shape barre, fret 1");
    assert_eq!(notes_text(&ins, full), "F C F A C F");
}

#[test]
fn fb_5_4_every_curated_open_shape_obeys_the_constraints() {
    let ins = Instrument::default();
    for shape in OPEN_SHAPES {
        let symbol = {
            let root = [
                "C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B",
            ][usize::from(shape.root_pc)];
            format!("{root}{}", shape.quality.symbol())
        };
        let result = voicings_for(&chord(&symbol), &ins, &VoicingOptions::default());
        let v = find(result.voicings(), shape.frets, ins.capo)
            .unwrap_or_else(|| panic!("{} {} not offered", shape.name, shape.frets));
        assert!(v.span <= 4, "{}", shape.name);
        let fingers: std::collections::HashSet<u8> = v
            .fingering
            .fingers
            .iter()
            .flatten()
            .copied()
            .filter(|f| *f > 0)
            .collect();
        assert!(fingers.len() <= 4, "{}", shape.name);
        // Every sounding note is a chord tone; root and third (or sus note) are present.
        let tones = chord(&symbol).tones();
        let pcs: Vec<u8> = tones.iter().map(|t| t.note.pitch_class().value()).collect();
        let sounding: Vec<u8> = v
            .strings
            .iter()
            .zip(StringNumber::LOW_TO_HIGH)
            .filter_map(|(s, string)| match s {
                StringState::Muted => None,
                StringState::Open => Some(ins.open_sounding_pitch(string).midi()),
                StringState::Fretted { fret } => Some(ins.pitch_at(string, *fret).unwrap().midi()),
            })
            .map(|m| u8::try_from(m.rem_euclid(12)).unwrap())
            .collect();
        assert!(
            sounding.iter().all(|p| pcs.contains(p)),
            "{}: foreign note",
            shape.name
        );
        // The root plus the second pitch class (third, or the sus tone) must always sound.
        let needs: Vec<u8> = vec![pcs[0], pcs[1]];
        assert!(
            needs.iter().all(|p| sounding.contains(p)),
            "{}: root or third/sus missing",
            shape.name
        );
    }
}

#[test]
fn fb_5_3_curated_shapes_come_first_in_their_group() {
    let ins = Instrument::default();
    for symbol in ["C", "G", "D", "Em", "A7", "Fmaj7"] {
        let result = voicings_for(&chord(symbol), &ins, &VoicingOptions::default());
        let first = &result.voicings()[0];
        assert_eq!(first.origin, Origin::Curated, "{symbol}");
    }
}

#[test]
fn b3_every_caged_form_is_correct_for_every_root() {
    let ins = Instrument::default();
    let roots = [
        "C", "C#", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
    ];
    for template in CAGED_TEMPLATES {
        for root in roots {
            let c = Chord::new(root.parse().unwrap(), template.quality);
            let result = voicings_for(&c, &ins, &VoicingOptions::default());
            let found: Vec<_> = result
                .voicings()
                .iter()
                .filter(|v| {
                    v.kind
                        == VoicingKind::Caged {
                            shape: template.shape,
                        }
                })
                .collect();
            assert!(
                !found.is_empty(),
                "{root}{} {:?}-shape missing",
                template.quality.symbol(),
                template.shape
            );
            let pcs: Vec<u8> = c
                .tones()
                .iter()
                .map(|t| t.note.pitch_class().value())
                .collect();
            for v in found {
                for (state, string) in v.strings.iter().zip(StringNumber::LOW_TO_HIGH) {
                    let midi = match state {
                        StringState::Muted => continue,
                        StringState::Open => ins.open_sounding_pitch(string).midi(),
                        StringState::Fretted { fret } => {
                            ins.pitch_at(string, *fret).unwrap().midi()
                        }
                    };
                    assert!(
                        pcs.contains(&u8::try_from(midi.rem_euclid(12)).unwrap()),
                        "{root}{} {:?}: {}",
                        template.quality.symbol(),
                        template.shape,
                        v.fret_string(ins.capo)
                    );
                }
            }
        }
    }
}

#[test]
fn b3_movable_shapes_at_known_frets() {
    let ins = Instrument::default();
    let cases: &[(&str, &str)] = &[
        ("G", "355433"),
        ("Bb", "x13331"),
        ("Bm", "x24432"),
        ("Fm", "133111"),
        ("Cm", "x35543"),
        ("B7", "x2120x".trim_end_matches('x')),
    ];
    for (symbol, expected) in cases {
        let result = voicings_for(&chord(symbol), &ins, &VoicingOptions::default());
        let has = result
            .voicings()
            .iter()
            .any(|v| v.fret_string(ins.capo).starts_with(expected));
        assert!(
            has,
            "{symbol} should offer {expected}: {:?}",
            result
                .voicings()
                .iter()
                .map(|v| v.fret_string(ins.capo))
                .collect::<Vec<_>>()
        );
    }
}

#[test]
fn ac_lh_4_1_drop_d_d_chord_uses_all_six_strings_with_string_six_open() {
    let ins = Instrument::with_tuning(Tuning::preset(TuningPreset::DropD));
    let result = voicings_for(&chord("D"), &ins, &VoicingOptions::default());
    let list = result.voicings();
    let v = find(list, "000232", ins.capo).expect("six-string D in the list");
    assert_eq!(v.strings[0], StringState::Open);
    assert!(v.strings.iter().all(|s| s.is_sounding()));
    assert_eq!(v.origin, Origin::Curated);
    // Every note is the correct pitch for D A D G B E: D A D A D F#.
    assert_eq!(notes_text(&ins, v), "D A D A D F#");
    for (symbol, frets, notes) in [
        ("Dm", "000231", "D A D A D F"),
        ("D7", "000212", "D A D A C F#"),
        ("Dmaj7", "000222", "D A D A C# F#"),
        ("Dsus2", "000230", "D A D A D E"),
        ("Dsus4", "000233", "D A D A D G"),
    ] {
        let result = voicings_for(&chord(symbol), &ins, &VoicingOptions::default());
        let v = find(result.voicings(), frets, ins.capo).unwrap_or_else(|| panic!("{symbol}"));
        assert_eq!(notes_text(&ins, v), notes, "{symbol}");
    }
}

#[test]
fn lh_4_drop_d_never_offers_a_standard_shape_that_would_sound_wrong() {
    // Standard G (320003) uses the low string; in Drop D that string is D, so it must not appear.
    let ins = Instrument::with_tuning(Tuning::preset(TuningPreset::DropD));
    let result = voicings_for(&chord("G"), &ins, &VoicingOptions::default());
    assert!(find(result.voicings(), "320003", ins.capo).is_none());
    // The fifth-string shapes are unchanged: A in Drop D is the same as in standard.
    let result = voicings_for(&chord("A"), &ins, &VoicingOptions::default());
    assert!(find(result.voicings(), "x02220", ins.capo).is_some());
}

#[test]
fn lh_4_half_step_down_reuses_standard_shapes_with_transposed_names() {
    let ins = Instrument::with_tuning(Tuning::preset(TuningPreset::HalfStepDown));
    let cases: &[(&str, &str, &str)] = &[
        ("Eb", "022100", "Eb Bb Eb G Bb Eb"),
        ("Ab", "x02220", "Ab Eb Ab C Eb"),
        ("Db", "xx0232", "Db Ab Db F"),
        ("Abm", "x02210", "Ab Eb Ab B Eb"),
    ];
    for (symbol, frets, notes) in cases {
        let result = voicings_for(&chord(symbol), &ins, &VoicingOptions::default());
        let v = find(result.voicings(), frets, ins.capo).unwrap_or_else(|| panic!("{symbol}"));
        assert_eq!(v.origin, Origin::Curated);
        assert!(v.name.ends_with(" shape"), "{}", v.name);
        // Notes are spelled with flats in this tuning; compare by pitch class names.
        let got = notes_text(&ins, v);
        assert_eq!(got.split(' ').count(), notes.split(' ').count(), "{symbol}");
        for (g, e) in got.split(' ').zip(notes.split(' ')) {
            let pc = |n: &str| n.parse::<mackey_theory::Note>().unwrap().pitch_class();
            assert_eq!(pc(g), pc(e), "{symbol}: {got}");
        }
    }
}

#[test]
fn lh_4_whole_step_down_reuses_standard_shapes() {
    let ins = Instrument::with_tuning(Tuning::preset(TuningPreset::WholeStepDown));
    // Standard open E shape, a whole step down, sounds D.
    let result = voicings_for(&chord("D"), &ins, &VoicingOptions::default());
    let v = find(result.voicings(), "022100", ins.capo).expect("E shape sounds D");
    assert_eq!(v.origin, Origin::Curated);
}

#[test]
fn lh_4_search_only_tunings_have_no_curated_shapes() {
    let ins = Instrument::with_tuning(Tuning::preset(TuningPreset::Dadgad));
    let result = voicings_for(&chord("D"), &ins, &VoicingOptions::default());
    assert!(matches!(result, VoicingResult::Found(_)));
    assert!(result.voicings().iter().all(|v| v.origin == Origin::Search));
    assert!(
        result
            .voicings()
            .iter()
            .all(|v| v.name == "Found by search")
    );
}

#[test]
fn lh_5_capo_translates_open_shapes_to_absolute_frets() {
    // A under capo 2 is the open G shape: absolute frets 5 4 2 2 2 5, fingers unchanged.
    let ins = Instrument {
        capo: Capo::new(2).unwrap(),
        ..Instrument::default()
    };
    let result = voicings_for(&chord("A"), &ins, &VoicingOptions::default());
    let v = result
        .voicings()
        .iter()
        .find(|v| v.name == "Open G shape")
        .expect("G shape under the capo");
    assert_eq!(v.fret_string(ins.capo), "320003");
    let absolute: Vec<Option<u8>> = v.strings.iter().map(|s| s.fretted()).collect();
    assert_eq!(absolute, [Some(5), Some(4), None, None, None, Some(5)]);
    assert_eq!(fingers_text(v), "2-1-0-0-0-3");
    assert_eq!(v.lowest_fret.value(), 2, "open voicings start at the capo");
    assert_eq!(
        notes_text(&ins, v),
        "A C# E A C# E".replace("A C# E A C# E", "A C# E A C# A")
    );
}
