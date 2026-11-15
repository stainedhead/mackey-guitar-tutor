//! Property tests for the invariants the theory engine promises (AC-P0-2, AC-P0-3).
//!
//! `cargo test` runs 256 cases per property; `make props` runs 10,000 in release mode.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]

use mackey_theory::{
    Accidental, AccidentalPreference, Chord, ChordQuality, Degree, Interval, IntervalQuality, Key,
    Letter, Mode, Note, NumeralCase, ParseOptions, Pitch, PitchClass, RomanNumeral, RomanSuffix,
    Scale, Speller, parse_chord, parse_chord_symbol, roman_to_chord,
};
use proptest::prelude::*;

// ---- strategies --------------------------------------------------------------------------

fn letter() -> impl Strategy<Value = Letter> {
    (0..7i32).prop_map(Letter::from_index)
}

/// A note with at most one accidental: what players type and the parser accepts.
fn note() -> impl Strategy<Value = Note> {
    (letter(), -1i8..=1).prop_map(|(l, a)| Note::new(l, a).unwrap())
}

/// Any note including double accidentals.
fn any_note() -> impl Strategy<Value = Note> {
    (letter(), -2i8..=2).prop_map(|(l, a)| Note::new(l, a).unwrap())
}

fn quality() -> impl Strategy<Value = ChordQuality> {
    prop::sample::select(ChordQuality::ALL.to_vec())
}

fn preference() -> impl Strategy<Value = AccidentalPreference> {
    prop_oneof![
        Just(AccidentalPreference::Sharps),
        Just(AccidentalPreference::Flats)
    ]
}

const QUALITIES: [IntervalQuality; 5] = [
    IntervalQuality::Perfect,
    IntervalQuality::Major,
    IntervalQuality::Minor,
    IntervalQuality::Augmented,
    IntervalQuality::Diminished,
];

/// Every valid interval of number 1..=15 up to `max_number`.
fn valid_intervals(max_number: u8) -> Vec<Interval> {
    (1..=max_number)
        .flat_map(|n| {
            QUALITIES
                .iter()
                .filter_map(move |q| Interval::new(*q, n).ok())
        })
        .collect()
}

/// A valid interval of number 1..=15.
fn interval() -> impl Strategy<Value = Interval> {
    prop::sample::select(valid_intervals(15))
}

/// A valid simple interval (unison to octave).
fn simple_interval() -> impl Strategy<Value = Interval> {
    prop::sample::select(valid_intervals(8))
}

fn chord() -> impl Strategy<Value = Chord> {
    (note(), quality(), prop::option::of(note())).prop_map(|(root, quality, bass)| {
        let c = Chord::new(root, quality);
        match bass {
            Some(b) => c.with_bass(b),
            None => c,
        }
    })
}

fn key_of(mode: Mode) -> impl Strategy<Value = Key> {
    prop::sample::select(Key::all_offered(mode))
}

fn any_key() -> impl Strategy<Value = Key> {
    prop::sample::select(Mode::DIATONIC.to_vec()).prop_flat_map(key_of)
}

/// Strings built from the symbol alphabet, so most inputs are near-misses of real symbols.
fn symbol_like() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop::sample::select(vec![
            "A", "B", "C", "D", "E", "F", "G", "H", "a", "b", "m", "M", "7", "9", "5", "6", "1",
            "3", "#", "♯", "♭", "/", "maj", "min", "dim", "aug", "sus", "add", "°", "ø", "Δ", "+",
            "-", " ", "o", "x",
        ]),
        0..10,
    )
    .prop_map(|parts| parts.concat())
}

// ---- A2: pitch classes and pitches ---------------------------------------------------------

proptest! {
    #[test]
    fn a2_transpose_up_then_down_is_the_identity(pc in 0..12i32, n in -200..200i32) {
        let start = PitchClass::new(pc);
        prop_assert_eq!(start.transpose(n).transpose(-n), start);
    }

    #[test]
    fn a2_midi_round_trips(midi in 0..=127i32, pref in preference()) {
        let p = Pitch::from_midi(midi, pref).unwrap();
        prop_assert_eq!(p.midi(), midi);
    }

    #[test]
    fn a2_note_name_round_trips(n in any_note()) {
        prop_assert_eq!(n.name().parse::<Note>().unwrap(), n);
    }

    #[test]
    fn a2_transposing_a_note_up_then_down_keeps_the_pitch_class(n in note(), s in -30..30i32, pref in preference()) {
        let there = n.transpose_semitones(s, pref);
        prop_assert_eq!(there.pitch_class(), n.pitch_class().transpose(s));
        prop_assert_eq!(there.transpose_semitones(-s, pref).pitch_class(), n.pitch_class());
    }

    #[test]
    fn g_1_6_simplified_never_has_a_double_accidental_and_keeps_the_pitch_class(n in any_note()) {
        let s = n.simplified();
        prop_assert!(!s.is_double());
        prop_assert_eq!(s.pitch_class(), n.pitch_class());
    }
}

// ---- A3: intervals -------------------------------------------------------------------------

proptest! {
    #[test]
    fn a3_apply_moves_by_exactly_the_semitones(n in note(), i in interval()) {
        let up = i.apply(n);
        prop_assume!(up.is_ok());
        let up = up.unwrap();
        let semis = n.pitch_class().semitones_to(up.pitch_class());
        prop_assert_eq!(semis, i.semitones() % 12);
    }

    #[test]
    fn a3_apply_then_apply_the_octave_complement_returns_the_start(n in note(), i in interval()) {
        // The octave complement of a simple interval; inversion reduces compound intervals first.
        let inv = i.invert();
        prop_assume!(inv.is_ok());
        let inv = inv.unwrap();
        let up = i.apply(n);
        prop_assume!(up.is_ok());
        let back = inv.apply(up.unwrap());
        prop_assume!(back.is_ok());
        prop_assert_eq!(back.unwrap().pitch_class(), n.pitch_class());
    }

    #[test]
    fn a3_apply_then_apply_down_returns_the_start_note(n in note(), i in interval()) {
        let up = i.apply(n);
        prop_assume!(up.is_ok());
        let down = i.apply_down(up.unwrap());
        prop_assert_eq!(down.ok(), Some(n));
    }

    #[test]
    fn a3_between_recovers_the_simple_interval(n in note(), i in simple_interval()) {
        let up = i.apply(n);
        prop_assume!(up.is_ok());
        let found = Interval::between(n, up.unwrap());
        prop_assert!(found.is_ok());
        prop_assert_eq!(found.unwrap().semitones() % 12, i.semitones() % 12);
    }

    #[test]
    fn a3_inversion_is_an_involution_for_simple_intervals(i in prop::sample::select(valid_intervals(7))) {
        if let Ok(inv) = i.invert() {
            prop_assert_eq!(inv.invert().unwrap(), i);
            prop_assert_eq!(u16::from(inv.semitones()) + u16::from(i.semitones()), 12);
        }
    }
}

// ---- A4/A5: scales and keys ----------------------------------------------------------------

proptest! {
    #[test]
    fn a4_seven_note_scales_use_each_letter_exactly_once(t in note(), mode in prop::sample::select(vec![
        Mode::Major, Mode::NaturalMinor, Mode::HarmonicMinor, Mode::Dorian, Mode::Phrygian, Mode::Lydian, Mode::Mixolydian, Mode::Locrian,
    ])) {
        let degrees = Scale { tonic: t, mode }.degrees();
        prop_assert_eq!(degrees.len(), 7);
        let mut letters: Vec<u8> = degrees.iter().map(|n| n.letter.index()).collect();
        letters.sort_unstable();
        letters.dedup();
        prop_assert_eq!(letters.len(), 7, "{:?}", degrees);
    }

    #[test]
    fn a4_pentatonic_scales_use_distinct_letters(t in note(), mode in prop::sample::select(vec![Mode::MajorPentatonic, Mode::MinorPentatonic])) {
        let degrees = Scale { tonic: t, mode }.degrees();
        prop_assert_eq!(degrees.len(), 5);
        let mut letters: Vec<u8> = degrees.iter().map(|n| n.letter.index()).collect();
        letters.sort_unstable();
        letters.dedup();
        prop_assert_eq!(letters.len(), 5);
    }

    #[test]
    fn a4_scale_degrees_agree_with_degree_of(t in note(), mode in prop::sample::select(Mode::ALL.to_vec())) {
        let scale = Scale { tonic: t, mode };
        for (i, n) in scale.degrees().iter().enumerate() {
            prop_assert_eq!(scale.degree_of(n.pitch_class()).map(|d| usize::from(d.value())), Some(i + 1));
        }
    }

    /// AC-P0-3 part 3: the accidentals in a key's spelling are all one kind and match its signature.
    #[test]
    fn g_1_5_key_accidentals_are_of_one_kind_and_match_the_signature(k in any_key()) {
        let sign = k.signature().accidentals.signum();
        for n in k.scale().degrees() {
            let a = n.accidental.value();
            prop_assert!(a == 0 || a.signum() == sign, "{} has {} in a {}-sig key", k.name(), n, sign);
            prop_assert!(a.abs() <= 1);
        }
        let altered = k.scale().degrees().iter().filter(|n| n.accidental != Accidental::NATURAL).count();
        prop_assert_eq!(altered, usize::from(k.signature().count()), "{}", k.name());
    }

    #[test]
    fn a5_twelve_next_fifths_return_to_the_same_pitch_class(k in any_key()) {
        let mut forward = k;
        let mut backward = k;
        for _ in 0..12 {
            forward = forward.next_fifth();
            backward = backward.prev_fifth();
        }
        prop_assert_eq!(forward.tonic().pitch_class(), k.tonic().pitch_class());
        prop_assert_eq!(backward.tonic().pitch_class(), k.tonic().pitch_class());
        prop_assert_eq!(forward.mode(), k.mode());
    }

    #[test]
    fn a5_relative_of_relative_is_the_same_key(k in prop_oneof![key_of(Mode::Major), key_of(Mode::NaturalMinor)]) {
        let r = k.relative().unwrap();
        prop_assert_eq!(r.relative().unwrap().tonic().pitch_class(), k.tonic().pitch_class());
        prop_assert_eq!(r.signature(), k.signature());
    }
}

// ---- A6/A7: chords and spelling ------------------------------------------------------------

proptest! {
    #[test]
    fn a6_chord_tones_are_the_root_plus_the_formula(c in chord()) {
        let tones = c.tones();
        prop_assert_eq!(tones.len(), c.quality.intervals().len());
        for (tone, interval) in tones.iter().zip(c.quality.intervals()) {
            let expected = c.root.pitch_class().transpose(i32::from(interval.semitones()));
            prop_assert_eq!(tone.note.pitch_class(), expected);
        }
        prop_assert_eq!(tones[0].note, c.root);
    }

    #[test]
    fn a6_transposing_a_chord_up_then_down_keeps_its_pitch_classes(c in chord(), s in -24..24i32, pref in preference()) {
        let round = c.transpose(s, pref).transpose(-s, pref);
        prop_assert_eq!(round.pitch_classes(), c.pitch_classes());
        prop_assert_eq!(round.quality, c.quality);
    }

    #[test]
    fn g_1_5_every_chord_tone_is_spelled_the_way_the_chord_spells_it(k in any_key(), c in chord()) {
        let speller = Speller::new(k).with_chord(c);
        for tone in c.tones() {
            prop_assert_eq!(speller.spell(tone.note.pitch_class()).pitch_class(), tone.note.pitch_class());
            prop_assert_eq!(speller.spell(tone.note.pitch_class()), tone.note);
        }
    }

    #[test]
    fn g_1_6_no_display_label_has_a_double_accidental_and_the_pitch_class_is_kept(k in any_key(), c in prop::option::of(chord()), pc in 0..12i32) {
        let speller = match c { Some(c) => Speller::new(k).with_chord(c), None => Speller::new(k) };
        let shown = speller.display(PitchClass::new(pc));
        prop_assert!(!shown.label.is_double());
        prop_assert_eq!(shown.label.pitch_class(), PitchClass::new(pc));
        prop_assert_eq!(shown.theory.pitch_class(), PitchClass::new(pc));
    }

    #[test]
    fn g_1_5_without_context_scale_tones_use_the_key_spelling(k in any_key()) {
        let speller = Speller::new(k);
        for n in k.scale().degrees() {
            prop_assert_eq!(speller.spell(n.pitch_class()), n);
        }
    }
}

// ---- A8: the parser ------------------------------------------------------------------------

fn check_outcome(input: &str) -> Result<(), TestCaseError> {
    let outcome = parse_chord(input, ParseOptions::default());
    let strict = parse_chord_symbol(input);
    prop_assert_eq!(outcome.chord.is_some(), strict.is_ok());
    if outcome.chord.is_none() {
        prop_assert!(!outcome.diagnostics.is_empty());
    }
    for d in &outcome.diagnostics {
        prop_assert!(
            d.span.start <= d.span.end && d.span.end <= input.len(),
            "span {:?} outside {:?}",
            d.span,
            input
        );
        prop_assert!(
            input.is_char_boundary(d.span.start) && input.is_char_boundary(d.span.end),
            "span {:?} splits a char in {:?}",
            d.span,
            input
        );
        prop_assert!(input.get(d.span.range()).is_some());
    }
    let starts: Vec<usize> = outcome.diagnostics.iter().map(|d| d.span.start).collect();
    let mut sorted = starts.clone();
    sorted.sort_unstable();
    prop_assert_eq!(starts, sorted);
    Ok(())
}

proptest! {
    /// AC-P0-2: any string, including non-ASCII, never panics and always explains a failure.
    #[test]
    fn ac_p0_2_parse_chord_never_panics_on_any_string(s in any::<String>()) {
        check_outcome(&s)?;
    }

    #[test]
    fn ac_p0_2_parse_chord_never_panics_on_symbol_like_strings(s in symbol_like()) {
        check_outcome(&s)?;
    }

    #[test]
    fn ac_p0_2_parse_chord_never_panics_on_non_ascii(s in "\\PC{0,24}") {
        check_outcome(&s)?;
    }

    #[test]
    fn ac_p0_2_parse_chord_never_panics_on_long_input(s in "[A-Gb#m7/ ]{0,400}") {
        check_outcome(&s)?;
    }

    #[test]
    fn a8_every_generated_valid_chord_round_trips_through_its_symbol(c in chord()) {
        let parsed = parse_chord_symbol(&c.symbol());
        prop_assert_eq!(parsed, Ok(c));
    }

    #[test]
    fn a8_whitespace_around_a_symbol_is_ignored(c in chord(), pre in "[ \\t\\n]{0,3}", post in "[ \\t\\n]{0,3}") {
        let text = format!("{pre}{}{post}", c.symbol());
        prop_assert_eq!(parse_chord_symbol(&text), Ok(c));
    }
}

// ---- A9: Roman numerals --------------------------------------------------------------------

fn numeral() -> impl Strategy<Value = RomanNumeral> {
    let combos = vec![
        (NumeralCase::Upper, RomanSuffix::None),
        (NumeralCase::Lower, RomanSuffix::None),
        (NumeralCase::Lower, RomanSuffix::Dim),
        (NumeralCase::Lower, RomanSuffix::HalfDim),
        (NumeralCase::Upper, RomanSuffix::Aug),
        (NumeralCase::Upper, RomanSuffix::Seventh),
        (NumeralCase::Lower, RomanSuffix::Seventh),
        (NumeralCase::Upper, RomanSuffix::Maj7),
        (NumeralCase::Lower, RomanSuffix::Dim7),
    ];
    (1u8..=7, -1i8..=1, prop::sample::select(combos)).prop_map(|(d, a, (case, suffix))| {
        RomanNumeral {
            degree: Degree::new(d).unwrap(),
            accidental: Accidental::try_from(a).unwrap(),
            case,
            suffix,
        }
    })
}

proptest! {
    #[test]
    fn a9_roman_numerals_round_trip_through_text(n in numeral()) {
        prop_assert_eq!(n.to_string().parse::<RomanNumeral>(), Ok(n));
    }

    #[test]
    fn a9_roman_parsing_never_panics(s in any::<String>(), k in any_key()) {
        let _ = s.parse::<RomanNumeral>();
        let _ = roman_to_chord(&s, &k);
    }

    #[test]
    fn a9_every_diatonic_chord_is_made_of_scale_tones(k in any_key(), seventh in any::<bool>(), d in 1u8..=7) {
        let chord = k.diatonic_chord(Degree::new(d).unwrap(), seventh).chord;
        for pc in chord.pitch_classes().iter() {
            prop_assert!(k.scale().contains(pc));
        }
    }
}
