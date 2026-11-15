//! FR-024 (B11): what Play sounds. Strum, arpeggio, key scale, single note, empty plans.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]

use mackey_theory::{Mode, ReferencePitch, parse_chord_symbol};
use mackey_voicing::{
    Action, ArpeggioDirection, Bpm, Capo, Context, Fret, FretRange, Handedness, Instrument,
    PlanKind, PlaybackPlan, Selection, StringNumber, ViewType, apply, playback_plan,
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

fn chord_sel(ins: &Instrument, symbol: &str, view: ViewType) -> Selection {
    let s = run(
        &Selection::default_g_major(),
        ins,
        Action::SelectChord(parse_chord_symbol(symbol).unwrap()),
    );
    run(&s, ins, Action::SetView(view))
}

fn names(plan: &PlaybackPlan) -> Vec<String> {
    plan.notes
        .iter()
        .map(|n| format!("{}{}", n.pitch.note().name(), n.pitch.octave()))
        .collect()
}

#[test]
fn fb_8_1_strum_is_low_to_high_30_ms_apart_over_sounding_strings_only() {
    let ins = Instrument::default();
    let plan = playback_plan(
        &chord_sel(&ins, "C", ViewType::Voicing),
        &ins,
        None,
        Bpm::default(),
    );
    assert_eq!(plan.kind, PlanKind::Strum);
    // Open C is x32010: C3 E3 G3 C4 E4 with string 6 muted.
    assert_eq!(names(&plan), ["C3", "E3", "G3", "C4", "E4"]);
    let delays: Vec<u32> = plan.notes.iter().map(|n| n.delay_ms).collect();
    assert_eq!(delays, [0, 30, 60, 90, 120]);
    // All tones view strums the selected voicing too.
    let all = playback_plan(
        &chord_sel(&ins, "C", ViewType::AllTones),
        &ins,
        None,
        Bpm::default(),
    );
    assert_eq!(all, plan);
}

#[test]
fn fb_8_1_strum_under_a_capo_sounds_the_capoed_pitches() {
    let ins = Instrument {
        capo: Capo::new(2).unwrap(),
        ..Instrument::default()
    };
    // Chord A with the G shape: sounding A major, A2 not played (string 6 fret 5 = A2).
    let plan = playback_plan(
        &chord_sel(&ins, "A", ViewType::Voicing),
        &ins,
        None,
        Bpm::default(),
    );
    assert_eq!(names(&plan), ["A2", "C#3", "E3", "A3", "C#4", "A4"]);
}

#[test]
fn fb_8_1_triad_view_strums_the_selected_placement() {
    let ins = Instrument::default();
    let mut s = chord_sel(&ins, "C", ViewType::Triad);
    s.string_set = mackey_voicing::StringSet::S234;
    s.inversion = mackey_voicing::Inversion::First;
    let plan = playback_plan(&s, &ins, None, Bpm::default());
    assert_eq!(plan.kind, PlanKind::Strum);
    assert_eq!(names(&plan), ["E3", "G3", "C4"]);
}

#[test]
fn fb_8_1_arpeggio_is_eighth_notes_at_70_bpm_with_cumulative_exact_delays() {
    let ins = Instrument::default();
    let mut s = chord_sel(&ins, "Am", ViewType::Arpeggio);
    s.position = Some(5);
    let plan = playback_plan(&s, &ins, None, Bpm::default());
    assert_eq!(plan.kind, PlanKind::Arpeggio);
    assert_eq!(plan.notes.len(), 9);
    // Hand table: A2 E3 A3 C4 E4 A4... from frets 4..=9 in standard tuning, ascending:
    // s6 f5 A2, s6 f8 C3, s5 f7 E3, s4 f7 A3, s3 f5 C4, s3 f9 E4, s2 f5 E4, s1 f5 A4, s1 f8 C5.
    assert_eq!(
        names(&plan),
        ["A2", "C3", "E3", "A3", "C4", "E4", "E4", "A4", "C5"]
    );
    let delays: Vec<u32> = plan.notes.iter().map(|n| n.delay_ms).collect();
    // 30000 / 70 = 428.57: delays are i * 428.57 truncated once, not i * 428.
    assert_eq!(delays, [0, 428, 857, 1285, 1714, 2142, 2571, 3000, 3428]);
    assert!(plan.notes.iter().all(|n| n.duration_ms == 428));
}

#[test]
fn fb_8_1_arpeggio_directions_follow_the_view_order() {
    let ins = Instrument::default();
    let mut s = chord_sel(&ins, "Am", ViewType::Arpeggio);
    s.position = Some(5);
    s.arpeggio_direction = ArpeggioDirection::Down;
    let down = playback_plan(&s, &ins, None, Bpm::default());
    let mut up_names = {
        s.arpeggio_direction = ArpeggioDirection::Up;
        names(&playback_plan(&s, &ins, None, Bpm::default()))
    };
    up_names.reverse();
    assert_eq!(names(&down), up_names);
    s.arpeggio_direction = ArpeggioDirection::UpDown;
    let both = playback_plan(&s, &ins, None, Bpm::new(120).unwrap());
    assert_eq!(both.notes.len(), 17);
    assert_eq!(both.notes[1].delay_ms, 250, "120 bpm eighths are 250 ms");
}

#[test]
fn fb_8_1_key_notes_ascend_one_octave_from_the_lowest_tonic_in_range() {
    let ins = Instrument::default();
    // G major from the lowest G: string 6 fret 3 is G2.
    let plan = playback_plan(&Selection::default_g_major(), &ins, None, Bpm::default());
    assert_eq!(plan.kind, PlanKind::Scale);
    assert_eq!(
        names(&plan),
        ["G2", "A2", "B2", "C3", "D3", "E3", "F#3", "G3"]
    );
    assert_eq!(plan.notes.len(), 8);
    assert_eq!(plan.notes[7].delay_ms, 7 * 30_000 / 70);
    // Visible range 5..=12 starts at the lowest G in it: string 4 fret 5 is G3? string 4 is D3:
    // D + 5 = G3 is lower than string 3's open G3? Equal pitch; string 5 fret 10 = G3 too.
    let range = FretRange {
        from: Fret::new(5).unwrap(),
        to: Fret::new(12).unwrap(),
    };
    let plan = playback_plan(
        &Selection::default_g_major(),
        &ins,
        Some(range),
        Bpm::default(),
    );
    assert_eq!(names(&plan)[0], "G3");
    // F major spells Bb: F2 is string 6 fret 1.
    let mut f = Selection::default_g_major();
    f.key = mackey_theory::Key::new("F".parse().unwrap(), Mode::Major).unwrap();
    let plan = playback_plan(&f, &ins, None, Bpm::default());
    assert_eq!(
        names(&plan),
        ["F2", "G2", "A2", "Bb2", "C3", "D3", "E3", "F3"]
    );
}

#[test]
fn fb_8_1_pentatonic_overlay_plays_its_own_five_notes_plus_the_tonic() {
    let ins = Instrument::default();
    let mut s = Selection::default_g_major();
    s.overlay.on = true;
    s.overlay.scale = Some(mackey_theory::Scale {
        tonic: "A".parse().unwrap(),
        mode: Mode::MinorPentatonic,
    });
    let plan = playback_plan(&s, &ins, None, Bpm::default());
    assert_eq!(names(&plan), ["A2", "C3", "D3", "E3", "G3", "A3"]);
}

#[test]
fn fb_1_7_a_clicked_note_plays_alone() {
    let ins = Instrument::default();
    let s = run(
        &Selection::default_g_major(),
        &ins,
        Action::SelectNote {
            string: StringNumber::new(2).unwrap(),
            fret: Fret::new(1).unwrap(),
        },
    );
    let plan = playback_plan(&s, &ins, None, Bpm::default());
    assert_eq!(plan.kind, PlanKind::SingleNote);
    assert_eq!(names(&plan), ["C4"]);
    assert_eq!(plan.notes[0].delay_ms, 0);
}

#[test]
fn fb_8_1_nothing_to_play_is_an_empty_plan_not_an_error() {
    // A tuning with no playable C: an empty strum.
    let e3 = mackey_voicing::TuningPreset::Standard.strings()[5]
        .add_semitones(-12, mackey_theory::AccidentalPreference::Sharps)
        .unwrap();
    let (tuning, _) = mackey_voicing::Tuning::custom([e3; 6]).unwrap();
    let ins = Instrument::with_tuning(tuning);
    let plan = playback_plan(
        &chord_sel(&ins, "C", ViewType::Voicing),
        &ins,
        None,
        Bpm::default(),
    );
    assert_eq!(plan.notes, []);
    // A visible range with no tonic in it plays no scale.
    let std = Instrument::default();
    let range = FretRange {
        from: Fret::new(1).unwrap(),
        to: Fret::new(1).unwrap(),
    };
    let none = playback_plan(
        &Selection::default_g_major(),
        &std,
        Some(range),
        Bpm::default(),
    );
    assert_eq!(none.notes, []);
}

#[test]
fn bpm_is_validated_and_serialises_as_a_number() {
    assert!(Bpm::new(29).is_err() && Bpm::new(251).is_err());
    assert_eq!(Bpm::new(30).unwrap().value(), 30);
    assert_eq!(serde_json::to_string(&Bpm::default()).unwrap(), "70");
    assert!(serde_json::from_str::<Bpm>("500").is_err());
}

/// Delays never decrease and every pitch is audible, across kinds, tempos, tunings and capos.
#[test]
fn fb_8_1_property_delays_are_non_decreasing_and_frequencies_audible() {
    let a4 = ReferencePitch::default();
    let mut plans = 0;
    for preset in mackey_voicing::TuningPreset::ALL {
        for capo in [0u8, 4, 9] {
            let ins = Instrument {
                tuning: mackey_voicing::Tuning::preset(preset),
                capo: Capo::new(capo).unwrap(),
                ..Instrument::default()
            };
            for symbol in ["C", "Am7", "F#dim", "Bb", "E7"] {
                for view in [
                    ViewType::AllTones,
                    ViewType::Voicing,
                    ViewType::Triad,
                    ViewType::Arpeggio,
                ] {
                    for bpm in [30u16, 70, 250] {
                        let plan = playback_plan(
                            &chord_sel(&ins, symbol, view),
                            &ins,
                            None,
                            Bpm::new(bpm).unwrap(),
                        );
                        check(&plan, a4);
                        plans += 1;
                    }
                }
            }
            check(
                &playback_plan(&Selection::default_g_major(), &ins, None, Bpm::default()),
                a4,
            );
        }
    }
    assert!(plans > 500);
}

fn check(plan: &PlaybackPlan, a4: ReferencePitch) {
    assert!(
        plan.notes
            .windows(2)
            .all(|w| w[0].delay_ms <= w[1].delay_ms)
    );
    for n in &plan.notes {
        let hz = n.pitch.frequency(a4);
        assert!((20.0..=20_000.0).contains(&hz), "{hz} Hz");
        assert!(n.duration_ms > 0);
    }
}
