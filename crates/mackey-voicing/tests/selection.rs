//! FR-020: the selection model and every FB-9 cycling action; AC-FB-2.2, E-FB-4, G-3.3, G-3.4.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]

use mackey_theory::{Degree, Key, Mode, parse_chord_symbol};
use mackey_voicing::{
    Action, Capo, Context, Focus, Handedness, Instrument, Inversion, LabelsMode, Notice, Selection,
    StringSet, Tuning, TuningPreset, ViewType, VoicingCache, VoicingOptions, apply, apply_cached,
    instrument_changed, voicings_for,
};

fn run(sel: &Selection, ins: &Instrument, action: Action) -> Selection {
    let h = Handedness::default();
    apply(
        sel,
        &Context {
            instrument: ins,
            handedness: &h,
        },
        action,
    )
    .selection
}

fn chord_sel(ins: &Instrument, symbol: &str) -> Selection {
    run(
        &Selection::default_g_major(),
        ins,
        Action::SelectChord(parse_chord_symbol(symbol).unwrap()),
    )
}

fn key(tonic: &str, mode: Mode) -> Key {
    Key::new(tonic.parse().unwrap(), mode).unwrap()
}

fn symbol(sel: &Selection) -> String {
    sel.chord.map(|c| c.symbol()).unwrap_or_default()
}

#[test]
fn ac_fb_2_2_changing_key_keeps_the_degree_g_iv_becomes_d_iv_is_g() {
    let ins = Instrument::default();
    let g_major = Selection::default_g_major();
    let s = run(
        &g_major,
        &ins,
        Action::SelectDiatonic(Degree::new(4).unwrap()),
    );
    assert_eq!(symbol(&s), "C", "IV of G is C");
    let s = run(&s, &ins, Action::SelectKey(key("D", Mode::Major)));
    assert_eq!(symbol(&s), "G", "IV of D is G");
    assert_eq!(s.focus, Focus::Chord);
    // A seventh chord keeps being a seventh on the same degree (G-3.3).
    let mut sevenths = run(&g_major, &ins, Action::SetShowSevenths(true));
    sevenths = run(
        &sevenths,
        &ins,
        Action::SelectDiatonic(Degree::new(5).unwrap()),
    );
    assert_eq!(symbol(&sevenths), "D7");
    let moved = run(&sevenths, &ins, Action::SelectKey(key("C", Mode::Major)));
    assert_eq!(symbol(&moved), "G7");
}

#[test]
fn g_3_3_a_chord_outside_the_key_is_kept_with_a_notice() {
    let ins = Instrument::default();
    let h = Handedness::default();
    let ctx = Context {
        instrument: &ins,
        handedness: &h,
    };
    let s = chord_sel(&ins, "Eb");
    let r = apply(&s, &ctx, Action::SelectKey(key("D", Mode::Major)));
    assert_eq!(symbol(&r.selection), "Eb");
    assert!(r.notices.contains(&Notice::ChordKeptKeyChanged));
}

#[test]
fn fb_9_chord_cycling_goes_i_to_vii_and_wraps() {
    let ins = Instrument::default();
    let mut s = Selection::default_g_major();
    let mut seen = Vec::new();
    for _ in 0..8 {
        s = run(&s, &ins, Action::ChordNext);
        seen.push(symbol(&s));
    }
    assert_eq!(seen, ["G", "Am", "Bm", "C", "D", "Em", "F#dim", "G"]);
    for want in ["F#dim", "Em", "D"] {
        s = run(&s, &ins, Action::ChordPrev);
        assert_eq!(symbol(&s), want);
    }
}

#[test]
fn fb_9_voicing_cycling_wraps_the_whole_list_and_switches_to_the_voicing_view() {
    let ins = Instrument::default();
    let mut s = chord_sel(&ins, "C");
    assert_eq!(s.view, ViewType::AllTones);
    let list = voicings_for(
        &parse_chord_symbol("C").unwrap(),
        &ins,
        &VoicingOptions::default(),
    );
    let n = list.voicings().len();
    s = run(&s, &ins, Action::VoicingNext);
    assert_eq!(
        s.view,
        ViewType::Voicing,
        "first press shows the voicing view"
    );
    assert_eq!(s.voicing_index, 0);
    for expected in 1..n {
        s = run(&s, &ins, Action::VoicingNext);
        assert_eq!(s.voicing_index, expected);
    }
    s = run(&s, &ins, Action::VoicingNext);
    assert_eq!(s.voicing_index, 0, "wraps into the start");
    s = run(&s, &ins, Action::VoicingPrev);
    assert_eq!(s.voicing_index, n - 1, "and back to the end");
}

#[test]
fn fb_9_inversion_cycles_root_first_second_and_third_only_for_sevenths() {
    let ins = Instrument::default();
    let mut s = chord_sel(&ins, "C");
    let mut seen = vec![s.inversion];
    for _ in 0..3 {
        s = run(&s, &ins, Action::InversionNext);
        seen.push(s.inversion);
    }
    assert_eq!(
        seen,
        [
            Inversion::Root,
            Inversion::First,
            Inversion::Second,
            Inversion::Root
        ]
    );
    let mut s = chord_sel(&ins, "G7");
    let mut seen = vec![s.inversion];
    for _ in 0..4 {
        s = run(&s, &ins, Action::InversionNext);
        seen.push(s.inversion);
    }
    assert_eq!(
        seen,
        [
            Inversion::Root,
            Inversion::First,
            Inversion::Second,
            Inversion::Third,
            Inversion::Root
        ]
    );
    s = run(&s, &ins, Action::InversionPrev);
    assert_eq!(s.inversion, Inversion::Third, "Shift-I wraps backwards");
}

#[test]
fn fb_9_string_set_cycles_123_234_345_456() {
    let ins = Instrument::default();
    let mut s = chord_sel(&ins, "C");
    let mut seen = vec![s.string_set];
    for _ in 0..4 {
        s = run(&s, &ins, Action::StringSetNext);
        seen.push(s.string_set);
    }
    assert_eq!(
        seen,
        [
            StringSet::S123,
            StringSet::S234,
            StringSet::S345,
            StringSet::S456,
            StringSet::S123
        ]
    );
    s = run(&s, &ins, Action::StringSetPrev);
    assert_eq!(s.string_set, StringSet::S456);
}

#[test]
fn fb_9_view_and_labels_cycle_in_order_and_skip_unavailable_labels() {
    let ins = Instrument::default();
    let mut s = chord_sel(&ins, "Em");
    let mut views = vec![s.view];
    for _ in 0..4 {
        s = run(&s, &ins, Action::CycleView);
        views.push(s.view);
    }
    assert_eq!(
        views,
        [
            ViewType::AllTones,
            ViewType::Voicing,
            ViewType::Triad,
            ViewType::Arpeggio,
            ViewType::AllTones
        ]
    );
    // All tones with a chord: note, degree, interval, none. Finger appears with a shape view.
    let mut labels = vec![s.labels];
    for _ in 0..4 {
        s = run(&s, &ins, Action::CycleLabels);
        labels.push(s.labels);
    }
    assert_eq!(
        labels,
        [
            LabelsMode::NoteName,
            LabelsMode::ScaleDegree,
            LabelsMode::Interval,
            LabelsMode::None,
            LabelsMode::NoteName
        ]
    );
    s = run(&s, &ins, Action::SetView(ViewType::Voicing));
    let mut with_finger = Vec::new();
    for _ in 0..5 {
        s = run(&s, &ins, Action::CycleLabels);
        with_finger.push(s.labels);
    }
    assert!(with_finger.contains(&LabelsMode::Finger));
    // Without a chord there is no interval or finger label.
    let mut s = Selection::default_g_major();
    let mut plain = vec![s.labels];
    for _ in 0..3 {
        s = run(&s, &ins, Action::CycleLabels);
        plain.push(s.labels);
    }
    assert_eq!(
        plain,
        [
            LabelsMode::NoteName,
            LabelsMode::ScaleDegree,
            LabelsMode::None,
            LabelsMode::NoteName
        ]
    );
}

#[test]
fn fb_9_position_cycles_arpeggio_positions_and_wraps() {
    let ins = Instrument::default();
    let mut s = chord_sel(&ins, "Am");
    s = run(&s, &ins, Action::SetView(ViewType::Arpeggio));
    let mut seen = Vec::new();
    for _ in 0..40 {
        s = run(&s, &ins, Action::PositionNext);
        seen.push(s.position.unwrap());
    }
    // Strictly ascending, then wraps to the lowest.
    let wrap = seen.windows(2).position(|w| w[0] > w[1]).expect("wraps");
    assert_eq!(
        (seen[wrap], seen[wrap + 1]),
        (20, 0),
        "highest position to lowest: {seen:?}"
    );
    assert!(
        seen[..=wrap].windows(2).all(|w| w[0] + 1 == w[1]),
        "one step at a time"
    );
    let back = run(&s, &ins, Action::PositionPrev);
    assert_ne!(back.position, s.position);
}

#[test]
fn fb_4_4_roman_numerals_select_and_invalid_input_changes_nothing() {
    let ins = Instrument::default();
    let h = Handedness::default();
    let ctx = Context {
        instrument: &ins,
        handedness: &h,
    };
    let s = Selection::default_g_major();
    let v7 = apply(&s, &ctx, Action::SelectRoman("V7".into()));
    assert_eq!(symbol(&v7.selection), "D7");
    let flat7 = apply(&s, &ctx, Action::SelectRoman("bVII".into()));
    assert_eq!(symbol(&flat7.selection), "F");
    let bad = apply(&v7.selection, &ctx, Action::SelectRoman("zz".into()));
    assert_eq!(
        bad.selection, v7.selection,
        "an invalid numeral changes nothing"
    );
    assert!(!bad.announcement.is_empty());
}

#[test]
fn g_3_4_tuning_change_keeps_the_chord_and_reselects_the_best_voicing_with_a_notice() {
    let ins = Instrument::default();
    let h = Handedness::default();
    let mut s = chord_sel(&ins, "G");
    s = run(&s, &ins, Action::VoicingNext);
    s = run(&s, &ins, Action::VoicingNext);
    assert_eq!(s.voicing_index, 1);
    let drop_d = Instrument {
        tuning: Tuning::preset(TuningPreset::DropD),
        ..Instrument::default()
    };
    let r = instrument_changed(
        &s,
        &Context {
            instrument: &ins,
            handedness: &h,
        },
        &Context {
            instrument: &drop_d,
            handedness: &h,
        },
    );
    assert_eq!(symbol(&r.selection), "G");
    assert_eq!(r.selection.voicing_index, 0, "the best-scored voicing");
    assert!(r.notices.contains(&Notice::VoicingChangedForTuning));
    assert!(r.announcement.contains("Voicing changed for new tuning"));
    assert!(r.selection.is_valid(&Context {
        instrument: &drop_d,
        handedness: &h
    }));
}

#[test]
fn g_3_4_an_unchanged_voicing_keeps_its_index_without_a_notice() {
    let ins = Instrument::default();
    let h = Handedness::default();
    let mut s = chord_sel(&ins, "C");
    s = run(&s, &ins, Action::VoicingNext);
    s = run(&s, &ins, Action::VoicingNext);
    let same = ins.clone();
    let r = instrument_changed(
        &s,
        &Context {
            instrument: &ins,
            handedness: &h,
        },
        &Context {
            instrument: &same,
            handedness: &h,
        },
    );
    assert_eq!(r.selection.voicing_index, 1);
    assert!(r.notices.is_empty());
}

#[test]
fn e_fb_4_a_capo_above_the_position_moves_it_up_with_a_notice() {
    let ins = Instrument::default();
    let h = Handedness::default();
    let mut s = chord_sel(&ins, "Am");
    s = run(&s, &ins, Action::SetView(ViewType::Arpeggio));
    s = run(&s, &ins, Action::PositionNext);
    let before = s.position.unwrap();
    assert!(before < 7);
    let capoed = Instrument {
        capo: Capo::new(7).unwrap(),
        ..Instrument::default()
    };
    let r = instrument_changed(
        &s,
        &Context {
            instrument: &ins,
            handedness: &h,
        },
        &Context {
            instrument: &capoed,
            handedness: &h,
        },
    );
    assert!(
        r.selection.position.unwrap() >= 7,
        "first position at or above the capo"
    );
    assert!(r.notices.contains(&Notice::MovedAboveCapo));
    assert_eq!(Notice::MovedAboveCapo.text(), "Moved above the capo");
    assert!(r.announcement.contains("Moved above the capo"));
}

#[test]
fn g_3_4_a_smaller_fret_count_leaves_no_stale_index() {
    let ins = Instrument::default();
    let h = Handedness::default();
    let mut s = chord_sel(&ins, "Am");
    s = run(&s, &ins, Action::SetView(ViewType::Arpeggio));
    for _ in 0..30 {
        s = run(&s, &ins, Action::PositionNext);
    }
    let short = Instrument {
        frets: mackey_voicing::FretCount::new(19).unwrap(),
        ..Instrument::default()
    };
    let new = Context {
        instrument: &short,
        handedness: &h,
    };
    let r = instrument_changed(
        &s,
        &Context {
            instrument: &ins,
            handedness: &h,
        },
        &new,
    );
    assert!(r.selection.is_valid(&new));
}

/// A deterministic pseudo-random walk over every action on several instruments: the selection
/// never becomes invalid, the announcement is never empty, and the cached and uncached paths
/// agree.
#[test]
fn fb_9_property_any_action_sequence_keeps_the_selection_valid() {
    let h = Handedness::default();
    let instruments = [
        Instrument::default(),
        Instrument {
            tuning: Tuning::preset(TuningPreset::DropD),
            capo: Capo::new(3).unwrap(),
            ..Instrument::default()
        },
        Instrument {
            frets: mackey_voicing::FretCount::new(19).unwrap(),
            ..Instrument::default()
        },
    ];
    let chords = ["C", "Am7", "F#dim", "Bb", "Esus4", "D7", "C5"];
    let actions = |step: u32, chord: &str| -> Action {
        match step % 27 {
            0 => Action::ChordNext,
            1 => Action::ChordPrev,
            2 => Action::VoicingNext,
            3 => Action::VoicingPrev,
            4 => Action::InversionNext,
            5 => Action::InversionPrev,
            6 => Action::StringSetNext,
            7 => Action::StringSetPrev,
            8 => Action::PositionNext,
            9 => Action::PositionPrev,
            10 => Action::CycleView,
            11 => Action::CycleLabels,
            12 => Action::ToggleOverlay,
            13 => Action::KeyNext,
            14 => Action::KeyPrev,
            15 => Action::RelativeToggle,
            16 => Action::SetFollowChords(step % 2 == 0),
            17 => Action::SetShowSevenths(step % 3 == 0),
            18 => Action::SetShowAllOnSet(step % 2 == 0),
            19 => Action::SetShowOrder(step % 2 == 1),
            20 => Action::SetAllowStretch(step % 2 == 0),
            21 => Action::SetAllowOmittedFifth(step % 3 == 1),
            22 => Action::SelectChord(parse_chord_symbol(chord).unwrap()),
            23 => Action::ClearFocus,
            24 => Action::SetView(ViewType::Triad),
            25 => Action::SetOverlayScale(None),
            _ => Action::SelectDiatonic(Degree::new(u8::try_from(step % 7 + 1).unwrap()).unwrap()),
        }
    };
    let mut seed = 12_345_u32;
    for ins in &instruments {
        let ctx = Context {
            instrument: ins,
            handedness: &h,
        };
        let mut s = Selection::default_g_major();
        let mut cache = VoicingCache::new();
        for _ in 0..400 {
            seed = seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            let step = seed >> 8;
            let chord = chords[(step as usize / 27) % chords.len()];
            let action = actions(step, chord);
            let plain = apply(&s, &ctx, action.clone());
            let cached = apply_cached(&s, &ctx, action.clone(), &mut cache);
            assert_eq!(plain.selection, cached.selection, "{action:?}");
            assert!(
                plain.selection.is_valid(&ctx),
                "{action:?} -> {:?}",
                plain.selection
            );
            assert!(!plain.announcement.is_empty(), "{action:?}");
            s = plain.selection;
        }
    }
}
