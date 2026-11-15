//! FR-019: the `FretboardView` (AC-FB-2.1, AC-FB-4.1, AC-FB-1.6, AC-LH-5.1, FB-1, G-1.5, G-1.6,
//! G-2) plus text snapshots of eight representative views.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]

use std::fmt::Write as _;

use mackey_theory::{Key, Mode, parse_chord_symbol};
use mackey_voicing::{
    ArpeggioDirection, Capo, Focus, Fret, FretCount, FretRange, FretboardView, Handedness,
    Instrument, LabelsMode, MarkerKind, NoteRole, Selection, Tuning, TuningPreset, ViewRequest,
    ViewType, fretboard_view,
};

fn key(tonic: &str, mode: Mode) -> Key {
    Key::new(tonic.parse().unwrap(), mode).unwrap()
}

fn sel(key_tonic: &str, mode: Mode, chord: Option<&str>, view: ViewType) -> Selection {
    let mut s = Selection::default_g_major();
    s.key = key(key_tonic, mode);
    if let Some(symbol) = chord {
        s.focus = Focus::Chord;
        s.chord = Some(parse_chord_symbol(symbol).unwrap());
    }
    s.view = view;
    s
}

fn build(s: &Selection, ins: &Instrument) -> FretboardView {
    fretboard_view(s, ins, &Handedness::default(), &ViewRequest::default())
}

fn names(v: &FretboardView) -> Vec<String> {
    let mut list: Vec<String> = v
        .positions
        .iter()
        .map(|p| p.labels.note_name.clone())
        .collect();
    list.sort();
    list.dedup();
    list
}

#[test]
fn ac_fb_2_1_f_major_notes_view_has_bb_and_no_sharps() {
    let v = build(
        &sel("F", Mode::Major, None, ViewType::AllTones),
        &Instrument::default(),
    );
    assert_eq!(names(&v), ["A", "Bb", "C", "D", "E", "F", "G"]);
    assert_eq!(v.key_label, "F major");
    assert!(
        v.positions
            .iter()
            .all(|p| !p.labels.note_name.contains('#'))
    );
}

#[test]
fn ac_fb_4_1_d7_in_c_shows_d_fsharp_a_c_with_the_badge() {
    let v = build(
        &sel("C", Mode::Major, Some("D7"), ViewType::AllTones),
        &Instrument::default(),
    );
    assert_eq!(names(&v), ["A", "C", "D", "F#"]);
    assert_eq!(v.badge.as_deref(), Some("Not in key (V/V)"));
    // Roles: D root, F# third, A fifth, C seventh.
    for p in &v.positions {
        let want = match p.labels.note_name.as_str() {
            "D" => NoteRole::Root,
            "F#" => NoteRole::Third,
            "A" => NoteRole::Fifth,
            "C" => NoteRole::Seventh,
            other => panic!("unexpected {other}"),
        };
        assert_eq!(p.role, want, "{}", p.a11y_label);
    }
    // Scale-degree labels are relative to the key tonic, spelled from the note: F# is #4 in C.
    let f_sharp = v
        .positions
        .iter()
        .find(|p| p.labels.note_name == "F#")
        .unwrap();
    assert_eq!(f_sharp.labels.scale_degree, "#4");
    assert_eq!(f_sharp.labels.interval.as_deref(), Some("3"));
}

#[test]
fn fb_4_5_borrowed_chords_say_where_they_come_from() {
    let v = build(
        &sel("C", Mode::Major, Some("Bb"), ViewType::AllTones),
        &Instrument::default(),
    );
    assert_eq!(names(&v), ["Bb", "D", "F"], "G-1.5: the bVII root is Bb");
    assert!(
        v.badge.as_deref().unwrap().contains("bVII"),
        "{:?}",
        v.badge
    );
    let diatonic = build(
        &sel("C", Mode::Major, Some("G"), ViewType::AllTones),
        &Instrument::default(),
    );
    assert_eq!(diatonic.badge, None);
}

#[test]
fn ac_fb_1_6_tooltip_has_note_octave_degree_interval_and_role() {
    let v = build(
        &sel("C", Mode::Major, Some("Am7"), ViewType::AllTones),
        &Instrument::default(),
    );
    // String 5, fret 0 is the open A (A2), the root of Am7, 6th degree of C.
    let open_a = v
        .positions
        .iter()
        .find(|p| p.string.number() == 5 && p.fret.value() == 0)
        .unwrap();
    assert_eq!(open_a.tooltip.note_octave, "A2");
    assert_eq!(open_a.tooltip.degree, "6");
    assert_eq!(open_a.tooltip.interval.as_deref(), Some("R"));
    assert_eq!(open_a.tooltip.role, "Root");
    // High E string fret 12 is E5; the 5th of Am7 and the 3rd degree of C.
    let e5 = v
        .positions
        .iter()
        .find(|p| p.string.number() == 1 && p.fret.value() == 12)
        .unwrap();
    assert_eq!(e5.tooltip.note_octave, "E5");
    assert_eq!(e5.tooltip.degree, "3");
    assert_eq!(e5.tooltip.interval.as_deref(), Some("5"));
    assert_eq!(e5.tooltip.role, "5th");
    // Key G, chord G: string 3 open is G3, degree 1, interval R, role Root.
    let g = build(
        &sel("G", Mode::Major, Some("G"), ViewType::AllTones),
        &Instrument::default(),
    );
    let open_g = g
        .positions
        .iter()
        .find(|p| p.string.number() == 3 && p.fret.value() == 0)
        .unwrap();
    assert_eq!(
        (
            open_g.tooltip.note_octave.as_str(),
            open_g.tooltip.degree.as_str(),
            open_g.tooltip.interval.as_deref(),
            open_g.tooltip.role.as_str()
        ),
        ("G3", "1", Some("R"), "Root")
    );
    // Without a chord there is no interval.
    let plain = build(
        &sel("C", Mode::Major, None, ViewType::AllTones),
        &Instrument::default(),
    );
    assert!(plain.positions.iter().all(|p| p.tooltip.interval.is_none()));
}

#[test]
fn g_1_6_double_accidentals_never_appear_as_labels_but_stay_in_the_tooltip() {
    // Fdim7 is spelled F Ab Cb Ebb; the fretboard shows the simple names B and D.
    let v = build(
        &sel("C", Mode::Major, Some("Fdim7"), ViewType::AllTones),
        &Instrument::default(),
    );
    for p in &v.positions {
        assert!(
            !p.labels.note_name.contains("bb") && !p.labels.note_name.contains('x'),
            "{}",
            p.labels.note_name
        );
    }
    let ebb = v
        .positions
        .iter()
        .find(|p| p.tooltip.theory_spelling.is_some());
    let ebb = ebb.expect("Ebb is drawn as D with the theory spelling in the tooltip");
    assert_eq!(ebb.labels.note_name, "D");
    assert_eq!(ebb.tooltip.theory_spelling.as_deref(), Some("Ebb"));
}

#[test]
fn b16_spelling_table_by_key_and_chord() {
    // (key, mode, chord or none, expected distinct labels). Every row checked against G-1.5.
    const CASES: &[(&str, Mode, Option<&str>, &[&str])] = &[
        ("C", Mode::Major, None, &["A", "B", "C", "D", "E", "F", "G"]),
        (
            "G",
            Mode::Major,
            None,
            &["A", "B", "C", "D", "E", "F#", "G"],
        ),
        (
            "F",
            Mode::Major,
            None,
            &["A", "Bb", "C", "D", "E", "F", "G"],
        ),
        (
            "Bb",
            Mode::Major,
            None,
            &["A", "Bb", "C", "D", "Eb", "F", "G"],
        ),
        (
            "E",
            Mode::Major,
            None,
            &["A", "B", "C#", "D#", "E", "F#", "G#"],
        ),
        (
            "Db",
            Mode::Major,
            None,
            &["Ab", "Bb", "C", "Db", "Eb", "F", "Gb"],
        ),
        (
            "A",
            Mode::NaturalMinor,
            None,
            &["A", "B", "C", "D", "E", "F", "G"],
        ),
        (
            "D",
            Mode::NaturalMinor,
            None,
            &["A", "Bb", "C", "D", "E", "F", "G"],
        ),
        (
            "E",
            Mode::NaturalMinor,
            None,
            &["A", "B", "C", "D", "E", "F#", "G"],
        ),
        ("C", Mode::Major, Some("D7"), &["A", "C", "D", "F#"]),
        ("C", Mode::Major, Some("Bb"), &["Bb", "D", "F"]),
        ("C", Mode::Major, Some("E7"), &["B", "D", "E", "G#"]),
        ("C", Mode::Major, Some("Ab"), &["Ab", "C", "Eb"]),
        ("C", Mode::Major, Some("F#m7b5"), &["A", "C", "E", "F#"]),
        ("F", Mode::Major, Some("Gm7"), &["Bb", "D", "F", "G"]),
        ("G", Mode::Major, Some("A#m"), &["A#", "C#", "E#"]),
    ];
    for (tonic, mode, chord, want) in CASES {
        let v = build(
            &sel(tonic, *mode, *chord, ViewType::AllTones),
            &Instrument::default(),
        );
        let got = names(&v);
        let mut want: Vec<String> = want.iter().map(ToString::to_string).collect();
        want.sort();
        assert_eq!(got, want, "{tonic} {mode:?} {chord:?}");
    }
}

#[test]
fn ac_lh_5_1_capo_2_chord_shape_g_reads_g_shape_sounds_a() {
    let ins = Instrument {
        capo: Capo::new(2).unwrap(),
        ..Instrument::default()
    };
    let v = build(&sel("G", Mode::Major, Some("A"), ViewType::Voicing), &ins);
    let label = v.chord_label.expect("a capo label");
    assert_eq!(label.text, "G shape (sounds A)");
    assert_eq!((label.shape.as_str(), label.sounding.as_str()), ("G", "A"));
    assert_eq!(v.capo, 2);
    assert_eq!(v.dimmed_below, 2, "frets below the capo are dimmed");
    // The toggle swaps the emphasis.
    let mut swapped = sel("G", Mode::Major, Some("A"), ViewType::Voicing);
    swapped.capo_names = mackey_voicing::CapoNames::SoundingFirst;
    let v = build(&swapped, &ins);
    assert_eq!(v.chord_label.unwrap().text, "A (G shape)");
    // No capo, no label.
    let none = build(
        &sel("G", Mode::Major, Some("A"), ViewType::Voicing),
        &Instrument::default(),
    );
    assert!(none.chord_label.is_none());
}

#[test]
fn fb_1_neck_furniture_fret_count_inlays_numbers_and_string_labels() {
    for frets in [19u8, 22, 24] {
        let ins = Instrument {
            frets: FretCount::new(frets).unwrap(),
            ..Instrument::default()
        };
        let v = build(&Selection::default_g_major(), &ins);
        assert_eq!(v.fret_count, frets);
        let inlays: Vec<(u8, bool)> = v
            .inlays
            .iter()
            .map(|i| (i.fret.value(), i.double))
            .collect();
        let expected: Vec<(u8, bool)> = [3u8, 5, 7, 9, 12, 15, 17, 19, 21, 24]
            .into_iter()
            .filter(|f| *f <= frets)
            .map(|f| (f, f == 12 || f == 24))
            .collect();
        assert_eq!(inlays, expected, "{frets} frets");
        assert_eq!(
            v.fret_numbers,
            expected.iter().map(|(f, _)| *f).collect::<Vec<_>>(),
            "FB-1.2 fret numbers"
        );
    }
    // FB-1.3: open-string note names for the tuning and capo.
    let ins = Instrument {
        tuning: Tuning::preset(TuningPreset::DropD),
        capo: Capo::new(2).unwrap(),
        ..Instrument::default()
    };
    let v = build(&Selection::default_g_major(), &ins);
    let labels: Vec<String> = v.strings.iter().map(|s| s.note.name()).collect();
    // Drop D D A D G B E raised two semitones by the capo: E B E A C# F# (spelled in G major).
    let mut sorted = labels.clone();
    sorted.sort();
    assert_eq!(v.strings.len(), 6);
    assert_eq!(labels[0], "E", "string 6: D plus capo 2");
    assert_eq!(labels[5], "F#", "string 1: E plus capo 2");
    assert_eq!(v.strings[0].string.number(), 6);
}

#[test]
fn fb_1_5_visible_range_adds_off_range_counts_and_fit_selection_pads_one_fret() {
    let ins = Instrument::default();
    let s = sel("C", Mode::Major, Some("C"), ViewType::Voicing);
    let full = build(&s, &ins);
    assert_eq!((full.off_range.below, full.off_range.above), (0, 0));
    // The open C voicing spans frets 0 to 3; fit selection shows them plus one fret either side
    // (clamped to the nut).
    assert_eq!(
        (full.fit_range.from.value(), full.fit_range.to.value()),
        (0, 4)
    );
    let req = ViewRequest {
        visible: Some(FretRange {
            from: Fret::new(5).unwrap(),
            to: Fret::new(10).unwrap(),
        }),
        ..ViewRequest::default()
    };
    let narrow = fretboard_view(&s, &ins, &Handedness::default(), &req);
    assert!(narrow.off_range.below > 0, "the open C sits below fret 5");
    assert_eq!(narrow.off_range.above, 0);
    assert!(
        narrow.fret_numbers.contains(&5),
        "first visible fret is numbered"
    );
}

#[test]
fn fb_5_x_muted_strings_are_marked_and_the_nut_barre_is_drawn() {
    let ins = Instrument::default();
    let v = build(&sel("C", Mode::Major, Some("C"), ViewType::Voicing), &ins);
    assert_eq!(v.markers.len(), 1);
    assert_eq!(v.markers[0].string.number(), 6);
    assert_eq!(v.markers[0].kind, MarkerKind::Muted);
    assert_eq!((v.barres).len(), 0);
    // The F barre (133211) draws one bar.
    let mut f = sel("C", Mode::Major, Some("F"), ViewType::Voicing);
    f.voicing_index = 0;
    let list = build(&f, &ins).voicing_list;
    let barre_at = list
        .iter()
        .position(|e| {
            e.strings
                .iter()
                .map(|s| s.fretted().unwrap_or(0))
                .eq([1, 3, 3, 2, 1, 1])
        })
        .expect("133211 is listed");
    f.voicing_index = barre_at;
    let v = build(&f, &ins);
    assert_eq!(v.barres.len(), 1);
    assert_eq!(v.barres[0].fret.value(), 1);
    assert_eq!((v.barres[0].from.number(), v.barres[0].to.number()), (6, 1));
}

#[test]
fn g_2_every_position_has_an_a11y_label_a_role_and_a_shape_cue() {
    let ins = Instrument::default();
    for view in [
        ViewType::AllTones,
        ViewType::Voicing,
        ViewType::Triad,
        ViewType::Arpeggio,
    ] {
        let v = build(&sel("G", Mode::Major, Some("Em"), view), &ins);
        assert!(!v.positions.is_empty(), "{view:?}");
        for p in &v.positions {
            assert_ne!((p.a11y_label).len(), 0);
            assert!(p.a11y_label.starts_with("String "), "{}", p.a11y_label);
            assert_ne!((p.tooltip.role).len(), 0);
        }
        assert_ne!((v.announcement).len(), 0);
    }
    // Root, third, fifth and extension read differently (shape cue) via distinct roles.
    let v = build(
        &sel("C", Mode::Major, Some("Cadd9"), ViewType::AllTones),
        &ins,
    );
    let roles: std::collections::HashSet<_> = v.positions.iter().map(|p| p.role).collect();
    assert!(roles.contains(&NoteRole::Root) && roles.contains(&NoteRole::Extension));
}

#[test]
fn g_2_2_labels_modes_change_the_text_and_finger_falls_back_to_names() {
    let ins = Instrument::default();
    let mut s = sel("C", Mode::Major, Some("Am"), ViewType::Voicing);
    s.labels = LabelsMode::Finger;
    let v = build(&s, &ins);
    assert_eq!(v.labels_mode, LabelsMode::Finger);
    assert!(v.positions.iter().all(|p| p.labels.finger.is_some()));
    // All tones has no fingering: finger falls back to note names.
    s.view = ViewType::AllTones;
    let v = build(&s, &ins);
    assert_eq!(v.labels_mode, LabelsMode::NoteName);
    s.labels = LabelsMode::Interval;
    let v = build(&s, &ins);
    assert_eq!(v.labels_mode, LabelsMode::Interval);
    assert!(v.positions.iter().all(|p| p.labels.interval.is_some()));
}

#[test]
fn e_fb_3_no_playable_voicing_is_reported_never_drawn() {
    // Every string tuned to E: C major needs C, E and G, which are 8 frets apart at best.
    let e3 = TuningPreset::Standard.strings()[5]
        .add_semitones(-12, mackey_theory::AccidentalPreference::Sharps)
        .unwrap();
    let (tuning, _) = Tuning::custom([e3; 6]).unwrap();
    let ins = Instrument::with_tuning(tuning);
    let v = build(&sel("C", Mode::Major, Some("C"), ViewType::Voicing), &ins);
    assert_eq!((v.voicing_list).len(), 0);
    let none = v.no_voicing.expect("the none-found result is carried");
    assert_eq!(
        none.none_message(),
        Some("No playable voicing in this tuning")
    );
    assert!(v.positions.is_empty(), "nothing is drawn");
    assert!(
        v.notices
            .iter()
            .any(|n| n.text().contains("No playable voicing"))
    );
}

#[test]
fn fb_3_handedness_is_reported_for_the_ui_to_mirror() {
    let ins = Instrument::default();
    let s = Selection::default_g_major();
    let h = Handedness {
        left_handed: true,
        flip_strings: true,
    };
    let v = fretboard_view(&s, &ins, &h, &ViewRequest::default());
    assert!(v.mirrored && v.flipped);
    let right = build(&s, &ins);
    assert!(!right.mirrored && !right.flipped);
    // The content never changes with handedness.
    assert_eq!(v.positions, right.positions);
}

#[test]
fn fb_7_arpeggio_view_numbers_notes_when_order_is_on() {
    let ins = Instrument::default();
    let mut s = sel("C", Mode::Major, Some("Am"), ViewType::Arpeggio);
    s.show_order = true;
    s.arpeggio_direction = ArpeggioDirection::Up;
    let v = build(&s, &ins);
    let mut order: Vec<u8> = v.positions.iter().filter_map(|p| p.order).collect();
    order.sort_unstable();
    assert_eq!(
        order,
        (1..=u8::try_from(v.positions.len()).unwrap()).collect::<Vec<_>>()
    );
    s.show_order = false;
    assert!(build(&s, &ins).positions.iter().all(|p| p.order.is_none()));
}

// ---- Snapshots: a compact text rendering of the neck (one row per string, one cell per fret).

fn render(v: &FretboardView) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "{} | {}", v.headline, v.announcement);
    if let Some(label) = &v.chord_label {
        let _ = writeln!(out, "capo label: {}", label.text);
    }
    if let Some(badge) = &v.badge {
        let _ = writeln!(out, "badge: {badge}");
    }
    let _ = writeln!(
        out,
        "fit {}..{} off {}/{}",
        v.fit_range.from.value(),
        v.fit_range.to.value(),
        v.off_range.below,
        v.off_range.above
    );
    for string in 1..=6u8 {
        let mut row = format!("{string} ");
        for fret in 0..=v.fret_count.min(14) {
            let cell = v
                .positions
                .iter()
                .find(|p| p.string.number() == string && p.fret.value() == fret)
                .map_or_else(
                    || ".".to_owned(),
                    |p| {
                        let shape = match p.role {
                            NoteRole::Root => '#',
                            NoteRole::ScaleTone => 'o',
                            _ => '*',
                        };
                        format!("{shape}{}", p.labels.note_name)
                    },
                );
            let _ = write!(row, "{cell:<4}");
        }
        let _ = writeln!(out, "{}", row.trim_end());
    }
    if !v.markers.is_empty() {
        let muted: Vec<String> = v
            .markers
            .iter()
            .map(|m| m.string.number().to_string())
            .collect();
        let _ = writeln!(out, "muted: {}", muted.join(" "));
    }
    out
}

#[test]
fn snapshots_of_eight_views() {
    let std = Instrument::default();
    let capo2 = Instrument {
        capo: Capo::new(2).unwrap(),
        ..Instrument::default()
    };
    let drop_d = Instrument {
        tuning: Tuning::preset(TuningPreset::DropD),
        ..Instrument::default()
    };
    let mut overlay = sel("A", Mode::NaturalMinor, None, ViewType::AllTones);
    overlay.overlay.on = true;
    overlay.overlay.scale = Some(mackey_theory::Scale {
        tonic: "A".parse().unwrap(),
        mode: Mode::MinorPentatonic,
    });
    overlay.overlay.pattern_index = Some(0);
    let mut triad = sel("C", Mode::Major, Some("C"), ViewType::Triad);
    triad.string_set = mackey_voicing::StringSet::S234;
    triad.inversion = mackey_voicing::Inversion::First;
    let views: Vec<(&str, FretboardView)> = vec![
        ("g_major_notes", build(&Selection::default_g_major(), &std)),
        (
            "f_major_notes",
            build(&sel("F", Mode::Major, None, ViewType::AllTones), &std),
        ),
        (
            "c_d7_all_tones",
            build(&sel("C", Mode::Major, Some("D7"), ViewType::AllTones), &std),
        ),
        (
            "c_open_voicing",
            build(&sel("C", Mode::Major, Some("C"), ViewType::Voicing), &std),
        ),
        ("c_triad_234_first", build(&triad, &std)),
        (
            "am_arpeggio",
            build(&sel("C", Mode::Major, Some("Am"), ViewType::Arpeggio), &std),
        ),
        ("a_minor_pentatonic_box1", build(&overlay, &std)),
        (
            "capo2_a_voicing",
            build(&sel("G", Mode::Major, Some("A"), ViewType::Voicing), &capo2),
        ),
    ];
    for (name, view) in &views {
        insta::assert_snapshot!(*name, render(view));
    }
    // A ninth view in Drop D guards the tuning plumbing.
    insta::assert_snapshot!(
        "drop_d_d_voicing",
        render(&build(
            &sel("D", Mode::Major, Some("D"), ViewType::Voicing),
            &drop_d
        ))
    );
}
