//! FR-016: triads on adjacent string sets (AC-FB-6.1, AC-FB-6.2, FB-6.3, FB-6.4) and the
//! property that every placement holds exactly the triad, with the inversion's tone lowest.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]

mod common;

use common::midis;
use mackey_theory::{Chord, parse_chord_symbol};
use mackey_voicing::{
    Capo, Instrument, Inversion, StringSet, StringState, TriadResult, Tuning, TuningPreset,
    triad_ladder, triad_note, triads,
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

#[test]
fn ac_fb_6_1_c_major_string_set_234_first_inversion_is_e_g_c() {
    let ins = Instrument::default();
    let list = triads(&chord("C"), &ins, StringSet::S234, Inversion::First);
    let first = &list.placements()[0];
    // E string 4 fret 2, G string 3 open, C string 2 fret 1; strings 6, 5 and 1 stay silent.
    assert_eq!(first.fret_string(ins.capo), "xx201x");
    assert_eq!(first.strings[2].fretted(), Some(2));
    assert_eq!(first.strings[3], StringState::Open);
    assert_eq!(first.strings[4].fretted(), Some(1));
    assert_eq!(first.strings[5], StringState::Muted);
}

#[test]
fn ac_fb_6_2_g_on_string_set_123_lists_root_first_second_in_fret_order() {
    let ins = Instrument::default();
    let ladder = triad_ladder(&chord("G"), &ins, StringSet::S123);
    let order: Vec<Inversion> = ladder.iter().map(|(i, _)| *i).collect();
    assert_eq!(
        order,
        [Inversion::Root, Inversion::First, Inversion::Second]
    );
    let lowest: Vec<u8> = ladder
        .iter()
        .map(|(_, v)| {
            v.strings
                .iter()
                .filter_map(|s| match s {
                    StringState::Fretted { fret } => Some(fret.value()),
                    StringState::Open => Some(0),
                    StringState::Muted => None,
                })
                .min()
                .unwrap_or(0)
        })
        .collect();
    assert!(
        lowest.windows(2).all(|w| w[0] <= w[1]),
        "fret order: {lowest:?}"
    );
    assert_eq!(ladder[1].1.fret_string(ins.capo), "xxx433");
    assert_eq!(ladder[2].1.fret_string(ins.capo), "xxx787");
    // Root position G B D: string 3 open (G), string 2 open (B), string 1 fret 10 (D).
    assert_eq!(ladder[0].1.fret_string(ins.capo), "xxx00(10)");
}

#[test]
fn fb_6_3_seventh_chords_show_their_triad_and_say_so() {
    let ins = Instrument::default();
    let list = triads(&chord("Cmaj7"), &ins, StringSet::S234, Inversion::First);
    assert_eq!(
        list.placements()[0].fret_string(ins.capo),
        "xx201x",
        "Cmaj7 uses the C E G triad"
    );
    assert_eq!(
        triad_note(&chord("Cmaj7")).as_deref(),
        Some("Triad of Cmaj7: C E G")
    );
    assert_eq!(
        triad_note(&chord("Am7")).as_deref(),
        Some("Triad of Am7: A C E")
    );
    assert_eq!(triad_note(&chord("C")), None);
}

#[test]
fn fb_6_4_unplayable_triads_say_why() {
    let ins = Instrument::default();
    // A power chord has no third, so no triad; a seventh chord has no third inversion triad.
    assert!(matches!(
        triads(&chord("C5"), &ins, StringSet::S123, Inversion::Root),
        TriadResult::Unavailable { .. }
    ));
    assert!(matches!(
        triads(&chord("C"), &ins, StringSet::S123, Inversion::Third),
        TriadResult::Unavailable { .. }
    ));
    // Under a capo at the 20th fret the 4-fret window runs off a 24-fret neck for nothing, but a
    // capo at 12 leaves a normal triad.
    let high = instrument(TuningPreset::Standard, 12);
    assert_ne!(
        triads(&chord("C"), &high, StringSet::S234, Inversion::Root)
            .placements()
            .len(),
        0
    );
}

#[test]
fn fb_6_1_triad_placements_are_fret_ordered_and_distinct() {
    let ins = Instrument::default();
    for set in StringSet::ALL {
        for inversion in [Inversion::Root, Inversion::First, Inversion::Second] {
            let list = triads(&chord("D"), &ins, set, inversion);
            let lows: Vec<u8> = list
                .placements()
                .iter()
                .map(|v| {
                    // Lowest sounding fret, an open string counting as the capo fret (0 here).
                    v.strings
                        .iter()
                        .filter_map(|s| match s {
                            StringState::Fretted { fret } => Some(fret.value()),
                            StringState::Open => Some(0),
                            StringState::Muted => None,
                        })
                        .min()
                        .unwrap_or(0)
                })
                .collect();
            assert!(
                lows.windows(2).all(|w| w[0] <= w[1]),
                "{set:?} {inversion:?}: {lows:?}"
            );
            let mut shapes: Vec<_> = list
                .placements()
                .iter()
                .map(|v| v.fret_string(ins.capo))
                .collect();
            let before = shapes.len();
            shapes.sort();
            shapes.dedup();
            assert_eq!(before, shapes.len(), "{set:?} {inversion:?}: duplicates");
        }
    }
}

/// Every triad on every string set and inversion, in every key and quality, under every preset
/// tuning and a few capos: exactly the three triad pitch classes sound, one per string, the
/// inversion's tone is the lowest note, and the pressed frets span at most four.
#[test]
fn fb_6_1_property_every_triad_placement_holds_exactly_the_triad() {
    let roots = [
        "C", "Db", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
    ];
    let qualities = ["", "m", "dim", "aug", "7", "m7", "maj7"];
    let mut checked = 0u32;
    for preset in TuningPreset::ALL {
        for capo in [0u8, 2, 5] {
            let ins = instrument(preset, capo);
            for root in roots {
                for quality in qualities {
                    let symbol = format!("{root}{quality}");
                    let c = chord(&symbol);
                    let tones = c.tones();
                    let want: Vec<u8> = tones
                        .iter()
                        .take(3)
                        .map(|t| t.note.pitch_class().value())
                        .collect();
                    for set in StringSet::ALL {
                        for (k, inversion) in [Inversion::Root, Inversion::First, Inversion::Second]
                            .into_iter()
                            .enumerate()
                        {
                            for v in triads(&c, &ins, set, inversion).placements() {
                                let notes = midis(&ins, &v.strings);
                                let sounding: Vec<i32> = notes.iter().flatten().copied().collect();
                                assert_eq!(
                                    sounding.len(),
                                    3,
                                    "{symbol} {preset:?} capo {capo} {set:?}: 3 strings"
                                );
                                let mut pcs: Vec<u8> = sounding
                                    .iter()
                                    .map(|m| u8::try_from(m.rem_euclid(12)).unwrap())
                                    .collect();
                                let bass_pc = pcs[0];
                                pcs.sort_unstable();
                                let mut expected = want.clone();
                                expected.sort_unstable();
                                assert_eq!(
                                    pcs,
                                    expected,
                                    "{symbol} {preset:?} capo {capo} {set:?} {inversion:?}: {}",
                                    v.fret_string(ins.capo)
                                );
                                assert_eq!(
                                    bass_pc, want[k],
                                    "{symbol} {inversion:?}: lowest note is the inversion's tone"
                                );
                                let pressed: Vec<u8> =
                                    v.strings.iter().filter_map(|s| s.fretted()).collect();
                                if let (Some(lo), Some(hi)) =
                                    (pressed.iter().min(), pressed.iter().max())
                                {
                                    assert!(hi - lo <= 4, "{symbol}: span");
                                }
                                checked += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    assert!(checked > 5_000, "only {checked} placements checked");
}

#[test]
fn fb_6_1_hand_table_of_triads_on_string_set_123() {
    // Strings 3, 2, 1 are G, B, E open. Each row is the lowest placement, worked out by hand.
    const CASES: &[(&str, Inversion, &str)] = &[
        ("G", Inversion::Root, "xxx00(10)"),
        ("Em", Inversion::First, "xxx000"),
        ("Em", Inversion::Root, "xxx987"),
        ("C", Inversion::Second, "xxx010"),
        ("Am", Inversion::First, "xxx555"),
        ("Am", Inversion::Second, "xxx9(10)8"),
    ];
    let ins = Instrument::default();
    for (symbol, inversion, head) in CASES {
        let list = triads(&chord(symbol), &ins, StringSet::S123, *inversion);
        let got = list.placements().first().map(|v| v.fret_string(ins.capo));
        assert_eq!(got.as_deref(), Some(*head), "{symbol} {inversion:?}");
    }
}
