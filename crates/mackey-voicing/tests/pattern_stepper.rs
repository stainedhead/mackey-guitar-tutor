//! FR-001 of the auto review spec (AC-SC-3.1): the stepper shows and numbers real boxes.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures"
)]

use std::collections::BTreeSet;

use mackey_theory::{Key, Mode, Scale};
use mackey_voicing::{
    Action, Context, FretboardView, Handedness, Instrument, Selection, ViewRequest, apply,
    fretboard_view,
};

fn scale(tonic: &str, mode: Mode) -> Scale {
    Scale {
        tonic: tonic.parse().unwrap(),
        mode,
    }
}

fn start(tonic: &str, key_mode: Mode, overlay: Scale) -> Selection {
    let mut s = Selection::default_g_major();
    s.key = Key::new(tonic.parse().unwrap(), key_mode).unwrap();
    s.overlay.on = true;
    s.overlay.scale = Some(overlay);
    s
}

fn press(s: &Selection, action: Action) -> (Selection, String) {
    let ins = Instrument::default();
    let h = Handedness::default();
    let ctx = Context {
        instrument: &ins,
        handedness: &h,
    };
    let applied = apply(s, &ctx, action);
    (applied.selection, applied.announcement)
}

fn view(s: &Selection) -> FretboardView {
    fretboard_view(
        s,
        &Instrument::default(),
        &Handedness::default(),
        &ViewRequest::default(),
    )
}

fn cells(v: &FretboardView) -> BTreeSet<(u8, u8)> {
    v.positions
        .iter()
        .map(|p| (p.string.number(), p.fret.value()))
        .collect()
}

fn box_1_a_minor_pentatonic() -> BTreeSet<(u8, u8)> {
    let mut want = BTreeSet::new();
    for string in [6, 2, 1] {
        want.extend([(string, 5), (string, 8)]);
    }
    for string in [5, 4, 3] {
        want.extend([(string, 5), (string, 7)]);
    }
    want
}

#[test]
fn fr001_ac_sc_3_1_first_p_press_shows_box_1_and_names_it() {
    let s = start("A", Mode::NaturalMinor, scale("A", Mode::MinorPentatonic));
    let (s, announcement) = press(&s, Action::PositionNext);
    let v = view(&s);
    assert_eq!(cells(&v), box_1_a_minor_pentatonic());
    let label = v.list_position.expect("a pattern label is shown");
    assert_eq!(label.label, "Box 1 of 5");
    assert_eq!((label.index, label.count), (1, 5));
    assert!(announcement.contains("Box 1 of 5"), "{announcement}");
}

#[test]
fn fr001_ac_sc_3_1_p_cycles_in_box_order_then_back_to_all() {
    let mut s = start("A", Mode::NaturalMinor, scale("A", Mode::MinorPentatonic));
    for n in 1..=5 {
        s = press(&s, Action::PositionNext).0;
        let label = view(&s).list_position.unwrap().label;
        assert_eq!(label, format!("Box {n} of 5"));
    }
    s = press(&s, Action::PositionNext).0;
    assert_eq!(s.overlay.pattern_index, None);
    assert!(view(&s).list_position.is_none());
}

#[test]
fn fr001_ac_sc_3_1_shift_p_from_all_goes_to_the_last_box() {
    let s = start("A", Mode::NaturalMinor, scale("A", Mode::MinorPentatonic));
    let s = press(&s, Action::PositionPrev).0;
    assert_eq!(view(&s).list_position.unwrap().label, "Box 5 of 5");
}

#[test]
fn fr001_seven_note_patterns_are_numbered_by_pattern_number() {
    let mut s = start("C", Mode::Major, scale("C", Mode::Major));
    for n in 1..=7 {
        s = press(&s, Action::PositionNext).0;
        let label = view(&s).list_position.unwrap().label;
        assert_eq!(label, format!("Pattern {n} of 7"));
    }
}
