//! FR-017: arpeggios in position (AC-FB-7.1, FB-7.2 to FB-7.4, E-FB-4 for positions).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]

use mackey_theory::{Chord, parse_chord_symbol};
use mackey_voicing::{
    ArpeggioDirection, Capo, Instrument, Tuning, TuningPreset, VoicingOptions, arpeggio,
    arpeggio_positions, default_arpeggio_position, voicings_for,
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

/// An independent oracle: every (string number, fret) in the window whose pitch class is a chord
/// tone, computed from the tuning's open MIDI numbers (frets are absolute from the nut).
fn oracle(c: &Chord, ins: &Instrument, start: u8) -> Vec<(u8, u8)> {
    let pcs: Vec<i32> = c
        .tones()
        .iter()
        .map(|t| i32::from(t.note.pitch_class().value()))
        .collect();
    let lo = start.saturating_sub(1).max(ins.capo.value());
    let hi = (start + 4).min(ins.frets.value());
    let open = ins.tuning.midis();
    let mut out = Vec::new();
    for (index, base) in open.iter().enumerate() {
        for fret in lo..=hi {
            if pcs.contains(&(base + i32::from(fret)).rem_euclid(12)) {
                out.push((u8::try_from(6 - index).unwrap(), fret));
            }
        }
    }
    out.sort_unstable();
    out
}

#[test]
fn ac_fb_7_1_am_position_5_holds_every_a_c_e_between_frets_4_and_9() {
    let ins = Instrument::default();
    let a = arpeggio(&chord("Am"), &ins, 5, ArpeggioDirection::Up);
    let mut got: Vec<(u8, u8)> = a
        .notes
        .iter()
        .map(|n| (n.string.number(), n.fret.value()))
        .collect();
    got.sort_unstable();
    // Hand check, standard tuning: s6 E: 5 (A); s5 A: 4? no, 5 (D) no; A is open 0. Frets 4..=9:
    //   s6 (E): 5=A, 8=C  | s5 (A): 7=E | s4 (D): 7=A,  s4 fret 10 is out | s3 (G): 5=C, 9=E
    //   s2 (B): 5=E, 6? no | s1 (E): 5=A, 8=C
    let expected = vec![
        (1, 5),
        (1, 8),
        (2, 5),
        (3, 5),
        (3, 9),
        (4, 7),
        (5, 7),
        (6, 5),
        (6, 8),
    ];
    assert_eq!(got, expected);
    assert_eq!(got, oracle(&chord("Am"), &ins, 5));
    assert!(
        a.notes.iter().any(|n| n.fret.value() == 5 && n.finger == 1),
        "finger 1 on fret 5"
    );
    assert!(a.notes.iter().all(|n| (1..=4).contains(&n.finger)));
}

#[test]
fn fb_7_2_one_finger_per_fret_with_index_on_the_start_fret() {
    let ins = Instrument::default();
    for (symbol, start) in [("Am", 5u8), ("C", 3), ("G", 7), ("E7", 9), ("F#m", 2)] {
        let a = arpeggio(&chord(symbol), &ins, start, ArpeggioDirection::Up);
        for n in &a.notes {
            let fret = n.fret.value();
            if n.is_open {
                assert_eq!(n.finger, 0, "{symbol} {start}: open string");
            } else {
                let want = (i16::from(fret) - i16::from(start) + 1).clamp(1, 4);
                assert_eq!(i16::from(n.finger), want, "{symbol} {start} fret {fret}");
            }
        }
        assert!(
            a.notes
                .iter()
                .filter(|n| n.fret.value() == start)
                .all(|n| n.finger == 1),
            "{symbol} {start}: index finger on the start fret"
        );
    }
}

#[test]
fn fb_7_3_directions_and_play_order_numbers() {
    let ins = Instrument::default();
    let up = arpeggio(&chord("Am"), &ins, 5, ArpeggioDirection::Up);
    let down = arpeggio(&chord("Am"), &ins, 5, ArpeggioDirection::Down);
    let both = arpeggio(&chord("Am"), &ins, 5, ArpeggioDirection::UpDown);
    let n = up.notes.len();
    assert_eq!(n, 9);
    assert_eq!(up.sequence, (0..n).collect::<Vec<_>>());
    assert_eq!(down.sequence, (0..n).rev().collect::<Vec<_>>());
    assert_eq!(
        both.sequence.len(),
        2 * n - 1,
        "the top note is not repeated"
    );
    assert_eq!(both.sequence[n - 1], n - 1);
    assert_eq!(both.sequence.last(), Some(&0));
    assert_eq!(up.order_numbers(), (1..=9).collect::<Vec<u8>>());
    assert_eq!(down.order_numbers(), (1..=9).rev().collect::<Vec<u8>>());
    assert_eq!(both.order_numbers(), (1..=9).collect::<Vec<u8>>());
    // Ascending in pitch.
    let midi = |i: usize| {
        let note = &up.notes[i];
        ins.tuning.midis()[6 - usize::from(note.string.number())] + i32::from(note.fret.value())
    };
    assert!((1..n).all(|i| midi(i - 1) <= midi(i)));
}

#[test]
fn fb_7_1_property_window_matches_the_oracle_across_tunings_capos_and_chords() {
    let symbols = [
        "C", "Am", "G7", "Bm7b5", "Fmaj7", "Dsus4", "Eb", "F#dim", "Bbaug", "A9",
    ];
    let mut checked = 0;
    for preset in TuningPreset::ALL {
        for capo in [0u8, 3, 7] {
            let ins = instrument(preset, capo);
            for symbol in symbols {
                let c = chord(symbol);
                for start in capo..=20 {
                    let a = arpeggio(&c, &ins, start, ArpeggioDirection::Up);
                    let mut got: Vec<(u8, u8)> = a
                        .notes
                        .iter()
                        .map(|n| (n.string.number(), n.fret.value()))
                        .collect();
                    got.sort_unstable();
                    assert_eq!(
                        got,
                        oracle(&c, &ins, start),
                        "{symbol} {preset:?} capo {capo} start {start}"
                    );
                    for n in &a.notes {
                        assert_eq!(n.is_open, n.fret.value() == capo);
                    }
                    checked += 1;
                }
            }
        }
    }
    assert!(checked > 1_000);
}

#[test]
fn fb_7_4_positions_hold_the_required_tones_and_respect_the_capo() {
    let ins = Instrument::default();
    let c = chord("Am");
    let positions = arpeggio_positions(&c, &ins);
    assert!(positions.windows(2).all(|w| w[0] < w[1]), "ascending");
    for start in &positions {
        let notes: Vec<u8> = oracle(&c, &ins, *start)
            .iter()
            .map(|(s, f)| {
                u8::try_from(
                    (ins.tuning.midis()[6 - usize::from(*s)] + i32::from(*f)).rem_euclid(12),
                )
                .unwrap()
            })
            .collect();
        for pc in [9u8, 0, 4] {
            assert!(
                notes.contains(&pc),
                "position {start} lacks pitch class {pc}"
            );
        }
    }
    let capoed = instrument(TuningPreset::Standard, 5);
    let positions = arpeggio_positions(&c, &capoed);
    assert!(
        positions.iter().all(|p| *p >= 5),
        "E-FB-4: nothing below the capo"
    );
    assert!(positions.len() <= 16);
}

#[test]
fn fb_7_4_default_position_follows_the_selected_voicing() {
    let ins = Instrument::default();
    let c = chord("Am");
    let list = voicings_for(&c, &ins, &VoicingOptions::default());
    let open_am = &list.voicings()[0];
    let p = default_arpeggio_position(&c, &ins, Some(open_am)).unwrap();
    // The open Am voicing sits at the nut: its position's window must reach the pressed frets.
    let pressed: Vec<u8> = open_am.strings.iter().filter_map(|s| s.fretted()).collect();
    for f in &pressed {
        assert!(
            (p.saturating_sub(1)..=p + 4).contains(f),
            "position {p} holds fret {f}"
        );
    }
    // Without a voicing: the lowest valid position with the root on string 6 or 5.
    let lowest = default_arpeggio_position(&c, &ins, None).unwrap();
    assert!(arpeggio_positions(&c, &ins).contains(&lowest));
    // A capo above every sensible position moves it above the capo (E-FB-4).
    let capoed = instrument(TuningPreset::Standard, 7);
    let moved = default_arpeggio_position(&c, &capoed, None).unwrap();
    assert!(moved >= 7);
}
