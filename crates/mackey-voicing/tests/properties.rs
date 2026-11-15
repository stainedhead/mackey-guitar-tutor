//! B12: property tests across every preset tuning, random custom tunings (including duplicate
//! open strings, E-FB-6), capo 0 to 12, 19 to 24 frets and both handedness settings.
//! AC-FB-5.3 and the AGENTS.md invariants: a chord's pitch classes are the same in every voicing
//! found; no voicing exceeds the span limit or drops the root or third.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]

mod common;

use common::{check_fb_5_4, midis};
use mackey_theory::{AccidentalPreference, Chord, ChordQuality, Pitch, parse_chord_symbol};
use mackey_voicing::{
    Action, ArpeggioDirection, Capo, Context, FretCount, Handedness, Instrument, Inversion,
    PatternKind, Selection, StringSet, Tuning, TuningPreset, ViewRequest, ViewType, VoicingOptions,
    VoicingResult, apply, arpeggio, fretboard_view, scale_patterns, triads, voicings_for,
};
use proptest::prelude::*;

const ROOTS: [&str; 12] = [
    "C", "Db", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
];

fn chord_strategy() -> impl Strategy<Value = Chord> {
    (0..ROOTS.len(), 0..ChordQuality::ALL.len()).prop_map(|(root, quality)| {
        Chord::new(ROOTS[root].parse().unwrap(), ChordQuality::ALL[quality])
    })
}

/// A preset tuning, or a custom one: each string within 7 semitones of standard, which allows
/// duplicate open strings and odd intervals but stays inside C1..=C6.
fn tuning_strategy() -> impl Strategy<Value = Tuning> {
    prop_oneof![
        (0..TuningPreset::ALL.len()).prop_map(|i| Tuning::preset(TuningPreset::ALL[i])),
        proptest::array::uniform6(-7i32..=7).prop_map(|offsets| {
            let standard = TuningPreset::Standard.strings();
            let mut strings = standard;
            for (slot, (base, offset)) in strings.iter_mut().zip(standard.iter().zip(offsets)) {
                *slot = base
                    .add_semitones(offset, AccidentalPreference::Sharps)
                    .unwrap();
            }
            Tuning::custom(strings).unwrap().0
        }),
        // Two identical neighbouring strings (E-FB-6).
        (0usize..5).prop_map(|i| {
            let mut strings = TuningPreset::Standard.strings();
            strings[i + 1] = strings[i];
            Tuning::custom(strings).unwrap().0
        }),
    ]
}

fn instrument_strategy() -> impl Strategy<Value = Instrument> {
    (tuning_strategy(), 0u8..=12, 19u8..=24).prop_map(|(tuning, capo, frets)| Instrument {
        tuning,
        capo: Capo::new(capo).unwrap(),
        frets: FretCount::new(frets).unwrap(),
    })
}

fn options_strategy() -> impl Strategy<Value = VoicingOptions> {
    (any::<bool>(), any::<bool>()).prop_map(|(stretch, omit)| VoicingOptions {
        allow_stretch: stretch,
        allow_omitted_fifth: omit,
        ..VoicingOptions::default()
    })
}

proptest! {
    /// AC-FB-5.3: for any chord, tuning, capo and fret count, every voicing found obeys FB-5.4:
    /// the span limit, at most four fingers, only chord tones, the root or third present (every
    /// required tone), the bass note as the lowest pitch, frets above the capo; the list is
    /// deterministic, capped and free of duplicates; "none found" offers only unused options.
    #[test]
    fn ac_fb_5_3_every_voicing_found_obeys_fb_5_4(
        chord in chord_strategy(),
        ins in instrument_strategy(),
        options in options_strategy(),
    ) {
        let result = voicings_for(&chord, &ins, &options);
        prop_assert_eq!(&result, &voicings_for(&chord, &ins, &options), "deterministic");
        match &result {
            VoicingResult::NoneFound { options: offered } => {
                prop_assert!(!offered.is_empty() || options.allow_stretch && options.allow_omitted_fifth);
            }
            VoicingResult::Found { .. } => {
                let list = result.voicings();
                prop_assert!(!list.is_empty() && list.len() <= options.max_results);
                let mut seen = std::collections::HashSet::new();
                for v in list {
                    check_fb_5_4(&chord, &ins, v, options.allow_stretch).map_err(|e| {
                        TestCaseError::fail(format!("{} {}: {e}", chord.symbol(), v.fret_string(ins.capo)))
                    })?;
                    prop_assert!(seen.insert(v.strings), "duplicate {}", v.fret_string(ins.capo));
                    // The same pitch classes whatever the voicing: only chord tones, every
                    // required role present (check_fb_5_4), never a foreign note.
                    let sounding = midis(&ins, &v.strings);
                    prop_assert!(sounding.iter().flatten().count() >= 2);
                }
            }
        }
    }

    /// FR-003 of the auto review: the labelled inversion is the one sounded. Every placement
    /// ascends in pitch from the lowest string to the top string, so its lowest pitch is the
    /// inversion's bass tone (an open string above fret 12 must not undercut the bass).
    #[test]
    fn fr003_every_triad_placement_ascends_and_has_the_labelled_bass(
        chord in chord_strategy(),
        ins in instrument_strategy(),
        set in 0usize..4,
        inversion in 0usize..3,
    ) {
        let tones = chord.tones();
        if tones.len() < 3 || chord.quality == ChordQuality::Power { return Ok(()); }
        let inversion = [Inversion::Root, Inversion::First, Inversion::Second][inversion];
        let bass = tones[inversion.index()].note.pitch_class().value();
        for v in triads(&chord, &ins, StringSet::ALL[set], inversion).placements() {
            let sounding: Vec<i32> = midis(&ins, &v.strings).into_iter().flatten().collect();
            prop_assert_eq!(sounding.len(), 3);
            prop_assert!(
                sounding.windows(2).all(|w| w[0] < w[1]),
                "{} {:?} {}: pitches do not ascend",
                chord.symbol(), inversion, v.fret_string(ins.capo)
            );
            let lowest = sounding.iter().min().copied().unwrap();
            prop_assert_eq!(
                u8::try_from(lowest.rem_euclid(12)).unwrap(), bass,
                "{} {:?} {}: wrong bass", chord.symbol(), inversion, v.fret_string(ins.capo)
            );
        }
    }

    /// FB-6: every triad placement holds exactly the triad's three pitch classes, one per string,
    /// the inversion's tone lowest, within four frets, above the capo.
    #[test]
    fn fb_6_every_triad_placement_is_exactly_the_triad(
        chord in chord_strategy(),
        ins in instrument_strategy(),
        set in 0usize..4,
        inversion in 0usize..3,
    ) {
        let tones = chord.tones();
        if tones.len() < 3 || chord.quality == ChordQuality::Power { return Ok(()); }
        let inversion = [Inversion::Root, Inversion::First, Inversion::Second][inversion];
        let want: Vec<u8> = tones.iter().take(3).map(|t| t.note.pitch_class().value()).collect();
        let bass = want[inversion.index()];
        for v in triads(&chord, &ins, StringSet::ALL[set], inversion).placements() {
            let sounding: Vec<i32> = midis(&ins, &v.strings).into_iter().flatten().collect();
            prop_assert_eq!(sounding.len(), 3);
            let mut pcs: Vec<u8> = sounding.iter().map(|m| u8::try_from(m.rem_euclid(12)).unwrap()).collect();
            prop_assert_eq!(pcs[0], bass);
            pcs.sort_unstable();
            let mut expected = want.clone();
            expected.sort_unstable();
            prop_assert_eq!(pcs, expected);
            let pressed: Vec<u8> = v.strings.iter().filter_map(|s| s.fretted()).collect();
            prop_assert!(pressed.iter().all(|f| *f > ins.capo.value()));
            if let (Some(lo), Some(hi)) = (pressed.iter().min(), pressed.iter().max()) {
                prop_assert!(hi - lo <= 4);
            }
        }
    }

    /// FB-7: an arpeggio holds only chord tones, inside the window, ascending in pitch, with a
    /// play order that visits every note.
    #[test]
    fn fb_7_arpeggio_notes_are_chord_tones_in_the_window(
        chord in chord_strategy(),
        ins in instrument_strategy(),
        start in 0u8..=20,
        direction in 0usize..3,
    ) {
        let direction = [ArpeggioDirection::Up, ArpeggioDirection::Down, ArpeggioDirection::UpDown][direction];
        let a = arpeggio(&chord, &ins, start, direction);
        let pcs: Vec<u8> = chord.tones().iter().map(|t| t.note.pitch_class().value()).collect();
        let lo = start.saturating_sub(1).max(ins.capo.value());
        let mut last = i32::MIN;
        for n in &a.notes {
            let fret = n.fret.value();
            prop_assert!(fret >= lo && fret <= (start + 4).min(ins.frets.value()));
            let midi = ins.tuning.midis()[6 - usize::from(n.string.number())] + i32::from(fret);
            prop_assert!(pcs.contains(&u8::try_from(midi.rem_euclid(12)).unwrap()));
            prop_assert!(midi >= last);
            last = midi;
        }
        let visited: std::collections::HashSet<usize> = a.sequence.iter().copied().collect();
        prop_assert_eq!(visited.len(), a.notes.len());
    }

    /// SC-3: scale patterns stay in the scale and above the capo in any tuning.
    #[test]
    fn sc_3_patterns_stay_in_the_scale(
        root in 0..ROOTS.len(),
        mode in 0usize..4,
        ins in instrument_strategy(),
    ) {
        let mode = [
            mackey_theory::Mode::Major,
            mackey_theory::Mode::NaturalMinor,
            mackey_theory::Mode::MajorPentatonic,
            mackey_theory::Mode::MinorPentatonic,
        ][mode];
        let scale = mackey_theory::Scale { tonic: ROOTS[root].parse().unwrap(), mode };
        for p in scale_patterns(&scale, &ins) {
            let per = if matches!(p.kind, PatternKind::PentatonicBox { .. }) { 2 } else { 3 };
            for string in 1..=6u8 {
                let on_string: Vec<_> = p.notes.iter().filter(|n| n.string.number() == string).collect();
                prop_assert_eq!(on_string.len(), per);
                for n in on_string {
                    prop_assert!(n.fret.value() >= ins.capo.value() && n.fret.value() <= ins.frets.value());
                    let midi = ins.tuning.midis()[6 - usize::from(string)] + i32::from(n.fret.value());
                    prop_assert_eq!(scale.degree_of(mackey_theory::PitchClass::new(midi)), Some(n.degree));
                }
            }
        }
    }

    /// FB-1, G-1.6, G-2: after any run of actions, every drawn position is on the neck, above
    /// the capo, sounds the pitch the instrument says, and carries clean labels; handedness is
    /// only echoed.
    #[test]
    fn fb_1_views_are_consistent_after_any_action_sequence(
        ins in instrument_strategy(),
        left in any::<bool>(),
        flip in any::<bool>(),
        steps in proptest::collection::vec((0u8..14, 0usize..7), 1..12),
    ) {
        let hand = Handedness { left_handed: left, flip_strings: flip };
        let ctx = Context { instrument: &ins, handedness: &hand };
        let chords = ["C", "Am7", "F#dim", "Bb", "Esus4", "D7", "C5"];
        let mut s = Selection::default_g_major();
        for (kind, which) in steps {
            let action = match kind {
                0 => Action::ChordNext,
                1 => Action::VoicingNext,
                2 => Action::InversionNext,
                3 => Action::StringSetNext,
                4 => Action::PositionNext,
                5 => Action::CycleView,
                6 => Action::CycleLabels,
                7 => Action::ToggleOverlay,
                8 => Action::KeyNext,
                9 => Action::RelativeToggle,
                10 => Action::SetView(ViewType::Triad),
                11 => Action::SetView(ViewType::Arpeggio),
                12 => Action::ClearFocus,
                _ => Action::SelectChord(parse_chord_symbol(chords[which]).unwrap()),
            };
            s = apply(&s, &ctx, action).selection;
        }
        let v = fretboard_view(&s, &ins, &hand, &ViewRequest::default());
        prop_assert_eq!((v.mirrored, v.flipped), (left, flip));
        prop_assert_eq!(v.fret_count, ins.frets.value());
        prop_assert_eq!(v.capo, ins.capo.value());
        let mut seen = std::collections::HashSet::new();
        for p in &v.positions {
            prop_assert!(p.fret.value() >= ins.capo.value() && p.fret.value() <= ins.frets.value());
            prop_assert!(seen.insert((p.string, p.fret)), "duplicate position");
            let want: Pitch = ins.pitch_at(p.string, p.fret).unwrap();
            prop_assert_eq!(Pitch::new(p.note, p.octave).unwrap().midi(), want.midi());
            prop_assert_eq!(p.is_open, p.fret.value() == ins.capo.value());
            let name = &p.labels.note_name;
            prop_assert!(!name.contains("bb") && !name.contains('x') && !name.is_empty(), "label {name}");
            prop_assert!(p.a11y_label.starts_with("String "));
        }
        prop_assert!(!v.announcement.is_empty());
    }
}

/// FR-003 example from the review: C on strings 3, 4, 5 in root position must not offer
/// `x(15)(14)0xx`, whose lowest note is the open G (a second inversion).
#[test]
fn fr003_c_standard_s345_root_has_no_open_g_undercut() {
    let c = Chord::new("C".parse().unwrap(), ChordQuality::Major);
    let ins = Instrument::default();
    let list = triads(&c, &ins, StringSet::S345, Inversion::Root);
    let shown: Vec<String> = list
        .placements()
        .iter()
        .map(|v| v.fret_string(ins.capo))
        .collect();
    assert!(!shown.iter().any(|s| s.contains("(15)(14)0")), "{shown:?}");
}
