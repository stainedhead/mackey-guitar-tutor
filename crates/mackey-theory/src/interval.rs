//! Intervals: a quality plus a number, with letter-stepping spelling.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::TheoryError;
use crate::pitch::{Note, PitchClass};

/// Interval quality.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum IntervalQuality {
    /// Perfect (unison, 4th, 5th, octave and their compounds).
    Perfect,
    /// Major (2nd, 3rd, 6th, 7th and compounds).
    Major,
    /// Minor.
    Minor,
    /// Augmented.
    Augmented,
    /// Diminished.
    Diminished,
}

/// Semitones of the perfect or major interval above the root for each simple number 1..=7.
const MAJOR_SCALE_SEMITONES: [u8; 7] = [0, 2, 4, 5, 7, 9, 11];

/// 1, 4 and 5 (and their octave compounds) are the perfect-class numbers.
fn is_perfect_class(number: u8) -> bool {
    matches!(number.saturating_sub(1) % 7, 0 | 3 | 4)
}

/// An interval of number 1 (unison) to 15 (two octaves), compounds included for 9, 11 and 13.
///
/// Invalid pairs ("perfect third", "major fifth", "diminished unison") cannot be built.
///
/// ```
/// use mackey_theory::{Interval, IntervalQuality, Note};
/// let m3 = Interval::new(IntervalQuality::Major, 3).unwrap();
/// let db: Note = "Db".parse().unwrap();
/// assert_eq!(m3.apply(db).unwrap().name(), "F");
/// assert!(Interval::new(IntervalQuality::Perfect, 3).is_err());
/// assert_eq!(Interval::MIN7.degree_label(), "b7");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Interval {
    quality: IntervalQuality,
    number: u8,
}

#[derive(Deserialize)]
struct IntervalRepr {
    quality: IntervalQuality,
    number: u8,
}

impl<'de> Deserialize<'de> for Interval {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let repr = IntervalRepr::deserialize(deserializer)?;
        Interval::new(repr.quality, repr.number).map_err(serde::de::Error::custom)
    }
}

impl Interval {
    /// Perfect unison.
    pub const P1: Interval = Interval::raw(IntervalQuality::Perfect, 1);
    /// Minor second.
    pub const MIN2: Interval = Interval::raw(IntervalQuality::Minor, 2);
    /// Major second.
    pub const MAJ2: Interval = Interval::raw(IntervalQuality::Major, 2);
    /// Minor third.
    pub const MIN3: Interval = Interval::raw(IntervalQuality::Minor, 3);
    /// Major third.
    pub const MAJ3: Interval = Interval::raw(IntervalQuality::Major, 3);
    /// Perfect fourth.
    pub const P4: Interval = Interval::raw(IntervalQuality::Perfect, 4);
    /// Augmented fourth.
    pub const AUG4: Interval = Interval::raw(IntervalQuality::Augmented, 4);
    /// Diminished fifth.
    pub const DIM5: Interval = Interval::raw(IntervalQuality::Diminished, 5);
    /// Perfect fifth.
    pub const P5: Interval = Interval::raw(IntervalQuality::Perfect, 5);
    /// Augmented fifth.
    pub const AUG5: Interval = Interval::raw(IntervalQuality::Augmented, 5);
    /// Minor sixth.
    pub const MIN6: Interval = Interval::raw(IntervalQuality::Minor, 6);
    /// Major sixth.
    pub const MAJ6: Interval = Interval::raw(IntervalQuality::Major, 6);
    /// Diminished seventh.
    pub const DIM7: Interval = Interval::raw(IntervalQuality::Diminished, 7);
    /// Minor seventh.
    pub const MIN7: Interval = Interval::raw(IntervalQuality::Minor, 7);
    /// Major seventh.
    pub const MAJ7: Interval = Interval::raw(IntervalQuality::Major, 7);
    /// Perfect octave.
    pub const P8: Interval = Interval::raw(IntervalQuality::Perfect, 8);
    /// Major ninth.
    pub const MAJ9: Interval = Interval::raw(IntervalQuality::Major, 9);
    /// Perfect eleventh.
    pub const P11: Interval = Interval::raw(IntervalQuality::Perfect, 11);
    /// Major thirteenth.
    pub const MAJ13: Interval = Interval::raw(IntervalQuality::Major, 13);

    /// For constants whose validity is visible at the call site.
    const fn raw(quality: IntervalQuality, number: u8) -> Self {
        Self { quality, number }
    }

    /// Validates a quality and number.
    ///
    /// # Errors
    /// [`TheoryError::InvalidInterval`] for number 0 or above 15, a perfect third, a major
    /// fifth, or a diminished unison.
    pub fn new(quality: IntervalQuality, number: u8) -> Result<Self, TheoryError> {
        let err = || TheoryError::InvalidInterval { quality, number };
        if !(1..=15).contains(&number) {
            return Err(err());
        }
        let quality_ok = if is_perfect_class(number) {
            matches!(
                quality,
                IntervalQuality::Perfect | IntervalQuality::Augmented | IntervalQuality::Diminished
            )
        } else {
            !matches!(quality, IntervalQuality::Perfect)
        };
        if !quality_ok || Self::semitones_of(quality, number) < 0 {
            return Err(err());
        }
        Ok(Self { quality, number })
    }

    fn base_semitones(number: u8) -> i32 {
        let n = number.saturating_sub(1);
        let simple = usize::from(n % 7);
        i32::from(MAJOR_SCALE_SEMITONES.get(simple).copied().unwrap_or(0)) + 12 * i32::from(n / 7)
    }

    fn semitones_of(quality: IntervalQuality, number: u8) -> i32 {
        let adjust = match quality {
            IntervalQuality::Perfect | IntervalQuality::Major => 0,
            IntervalQuality::Minor => -1,
            IntervalQuality::Augmented => 1,
            IntervalQuality::Diminished => {
                if is_perfect_class(number) {
                    -1
                } else {
                    -2
                }
            }
        };
        Self::base_semitones(number) + adjust
    }

    /// The interval of a number that spans `semitones`, if one exists.
    fn from_number_semitones(number: u8, semitones: i32) -> Option<Self> {
        let diff = semitones - Self::base_semitones(number);
        let quality = if is_perfect_class(number) {
            match diff {
                0 => IntervalQuality::Perfect,
                1 => IntervalQuality::Augmented,
                -1 => IntervalQuality::Diminished,
                _ => return None,
            }
        } else {
            match diff {
                0 => IntervalQuality::Major,
                -1 => IntervalQuality::Minor,
                1 => IntervalQuality::Augmented,
                -2 => IntervalQuality::Diminished,
                _ => return None,
            }
        };
        Self::new(quality, number).ok()
    }

    /// The quality.
    #[must_use]
    pub fn quality(self) -> IntervalQuality {
        self.quality
    }

    /// The number (1 = unison, 3 = third, 8 = octave, 9 = ninth ...).
    #[must_use]
    pub fn number(self) -> u8 {
        self.number
    }

    /// Size in semitones.
    ///
    /// ```
    /// use mackey_theory::Interval;
    /// assert_eq!(Interval::MIN7.semitones(), 10);
    /// assert_eq!(Interval::MAJ9.semitones(), 14);
    /// ```
    #[must_use]
    pub fn semitones(self) -> u8 {
        u8::try_from(Self::semitones_of(self.quality, self.number)).unwrap_or(0)
    }

    /// The note this interval above `note`, spelled by letter stepping: the letter moves
    /// `number - 1` places and the accidental makes up the semitones.
    ///
    /// ```
    /// use mackey_theory::{Interval, Note};
    /// let n = |s: &str| s.parse::<Note>().unwrap();
    /// assert_eq!(Interval::MAJ3.apply(n("F#")).unwrap(), n("A#"));
    /// assert_eq!(Interval::DIM5.apply(n("B")).unwrap(), n("F"));
    /// ```
    ///
    /// # Errors
    /// [`TheoryError::SpellingOutOfRange`] if the result needs a triple accidental.
    pub fn apply(self, note: Note) -> Result<Note, TheoryError> {
        let steps = i8::try_from(self.number - 1).unwrap_or(0);
        let letter = note.letter.step(steps);
        let pc = note.pitch_class().transpose(i32::from(self.semitones()));
        Note::from_letter_and_pitch_class(letter, pc).ok_or(TheoryError::SpellingOutOfRange)
    }

    /// The note this interval below `note`.
    ///
    /// ```
    /// use mackey_theory::{Interval, Note};
    /// let n = |s: &str| s.parse::<Note>().unwrap();
    /// assert_eq!(Interval::P5.apply_down(n("Db")).unwrap(), n("Gb"));
    /// ```
    ///
    /// # Errors
    /// [`TheoryError::SpellingOutOfRange`] if the result needs a triple accidental.
    pub fn apply_down(self, note: Note) -> Result<Note, TheoryError> {
        let steps = i8::try_from(self.number - 1).unwrap_or(0);
        let letter = note.letter.step(-steps);
        let pc = note.pitch_class().transpose(-i32::from(self.semitones()));
        Note::from_letter_and_pitch_class(letter, pc).ok_or(TheoryError::SpellingOutOfRange)
    }

    /// The upward interval from `from` to `to`, within an octave (unison to octave).
    ///
    /// ```
    /// use mackey_theory::{Interval, Note};
    /// let n = |s: &str| s.parse::<Note>().unwrap();
    /// assert_eq!(Interval::between(n("C"), n("Eb")).unwrap(), Interval::MIN3);
    /// assert_eq!(Interval::between(n("B"), n("F")).unwrap(), Interval::DIM5);
    /// ```
    ///
    /// # Errors
    /// [`TheoryError::SpellingOutOfRange`] when the pair is more than doubly augmented or
    /// diminished.
    pub fn between(from: Note, to: Note) -> Result<Self, TheoryError> {
        let letters = (i32::from(to.letter.index()) - i32::from(from.letter.index())).rem_euclid(7);
        let number = u8::try_from(letters + 1).unwrap_or(1);
        let semis = i32::from(from.pitch_class().semitones_to(to.pitch_class()));
        [
            (number, semis),
            (number, semis + 12),
            (number + 7, semis),
            (number + 7, semis + 12),
        ]
        .into_iter()
        .find_map(|(n, s)| Self::from_number_semitones(n, s))
        .ok_or(TheoryError::SpellingOutOfRange)
    }

    /// The inversion: the interval that completes the octave (major <-> minor, augmented <->
    /// diminished, perfect stays; a third becomes a sixth). Compound intervals are reduced
    /// to their simple form first.
    ///
    /// ```
    /// use mackey_theory::Interval;
    /// assert_eq!(Interval::MAJ3.invert().unwrap(), Interval::MIN6);
    /// assert_eq!(Interval::P5.invert().unwrap(), Interval::P4);
    /// ```
    ///
    /// # Errors
    /// [`TheoryError::InvalidInterval`] when the inversion does not exist (the inversion of
    /// an augmented octave would be a diminished unison).
    pub fn invert(self) -> Result<Self, TheoryError> {
        let (number, semis) = if self.number <= 8 {
            (self.number, i32::from(self.semitones()))
        } else {
            let octaves = (self.number - 1) / 7;
            (
                (self.number - 1) % 7 + 1,
                i32::from(self.semitones()) - 12 * i32::from(octaves),
            )
        };
        Self::from_number_semitones(9 - number, 12 - semis).ok_or(TheoryError::InvalidInterval {
            quality: self.quality,
            number: self.number,
        })
    }

    /// The chord-formula label: `R`, `b3`, `3`, `#4`, `b5`, `5`, `b7`, `7`, `9`, `b9`, `#11`,
    /// `13` ... (accidentals are measured against the major scale degree).
    ///
    /// ```
    /// use mackey_theory::Interval;
    /// assert_eq!(Interval::P1.degree_label(), "R");
    /// assert_eq!(Interval::MIN3.degree_label(), "b3");
    /// assert_eq!(Interval::DIM7.degree_label(), "bb7");
    /// assert_eq!(Interval::MAJ13.degree_label(), "13");
    /// ```
    #[must_use]
    pub fn degree_label(self) -> String {
        if self.number == 1 && self.quality == IntervalQuality::Perfect {
            return "R".to_owned();
        }
        let diff =
            Self::semitones_of(self.quality, self.number) - Self::base_semitones(self.number);
        let prefix = match diff {
            -2 => "bb",
            -1 => "b",
            1 => "#",
            _ => "",
        };
        format!("{prefix}{}", self.number)
    }

    /// The pitch class `self` above a pitch class (no spelling).
    #[must_use]
    pub fn above(self, pc: PitchClass) -> PitchClass {
        pc.transpose(i32::from(self.semitones()))
    }
}

impl fmt::Display for Interval {
    /// `M3`, `m7`, `P5`, `A4`, `d5`.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let q = match self.quality {
            IntervalQuality::Perfect => "P",
            IntervalQuality::Major => "M",
            IntervalQuality::Minor => "m",
            IntervalQuality::Augmented => "A",
            IntervalQuality::Diminished => "d",
        };
        write!(f, "{q}{}", self.number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(s: &str) -> Note {
        s.parse().unwrap()
    }

    /// (note, interval, expected note above)
    const APPLY_CASES: &[(&str, Interval, &str)] = &[
        ("Db", Interval::MAJ3, "F"),
        ("F#", Interval::MAJ3, "A#"),
        ("B", Interval::DIM5, "F"),
        ("C", Interval::P5, "G"),
        ("F", Interval::MAJ3, "A"),
        ("Bb", Interval::MAJ7, "A"),
        ("Eb", Interval::MIN7, "Db"),
        ("G#", Interval::DIM7, "F"),
        ("F#", Interval::MAJ7, "E#"),
        ("G#", Interval::MAJ3, "B#"),
        ("B", Interval::MAJ3, "D#"),
        ("F##", Interval::MAJ3, "A##"),
        ("Gb", Interval::MAJ3, "Bb"),
        ("E", Interval::MIN6, "C"),
        ("C", Interval::AUG4, "F#"),
        ("Cb", Interval::P5, "Gb"),
        ("C", Interval::MAJ9, "D"),
        ("C", Interval::P11, "F"),
        ("A", Interval::MAJ13, "F#"),
        ("Bb", Interval::DIM7, "Abb"),
        ("D", Interval::AUG5, "A#"),
        ("C", Interval::P8, "C"),
        ("C", Interval::P1, "C"),
        ("Eb", Interval::MIN2, "Fb"),
    ];

    #[test]
    fn a3_apply_by_letter_stepping() {
        for (start, interval, expected) in APPLY_CASES {
            let got = interval.apply(note(start)).unwrap();
            assert_eq!(got.name(), *expected, "case {start} + {interval}");
        }
    }

    #[test]
    fn a3_apply_down_mirrors_apply() {
        for (below, interval, above) in APPLY_CASES {
            assert_eq!(
                interval.apply_down(note(above)).unwrap().name(),
                *below,
                "case {above} - {interval}"
            );
        }
    }

    #[test]
    fn a3_apply_reports_triple_accidentals() {
        assert_eq!(
            Interval::AUG5.apply(note("F##")),
            Err(TheoryError::SpellingOutOfRange)
        );
        assert_eq!(
            Interval::AUG5.apply_down(note("F##")).map(Note::name),
            Ok("B".to_owned())
        );
        assert_eq!(
            Interval::AUG5.apply_down(note("Bbb")),
            Err(TheoryError::SpellingOutOfRange)
        );
    }

    /// (quality, number, valid)
    const NEW_CASES: &[(IntervalQuality, u8, bool)] = &[
        (IntervalQuality::Perfect, 5, true),
        (IntervalQuality::Major, 5, false),
        (IntervalQuality::Perfect, 3, false),
        (IntervalQuality::Major, 3, true),
        (IntervalQuality::Minor, 4, false),
        (IntervalQuality::Perfect, 0, false),
        (IntervalQuality::Perfect, 16, false),
        (IntervalQuality::Perfect, 15, true),
        (IntervalQuality::Diminished, 1, false),
        (IntervalQuality::Augmented, 1, true),
        (IntervalQuality::Diminished, 2, true),
        (IntervalQuality::Diminished, 8, true),
        (IntervalQuality::Perfect, 11, true),
        (IntervalQuality::Major, 13, true),
        (IntervalQuality::Perfect, 12, true),
        (IntervalQuality::Major, 8, false),
    ];

    #[test]
    fn a3_new_rejects_invalid_pairs() {
        for (quality, number, valid) in NEW_CASES {
            assert_eq!(
                Interval::new(*quality, *number).is_ok(),
                *valid,
                "case {quality:?} {number}"
            );
        }
    }

    /// (interval, semitones, degree label, short name)
    const SIZE_CASES: &[(Interval, u8, &str, &str)] = &[
        (Interval::P1, 0, "R", "P1"),
        (Interval::MIN2, 1, "b2", "m2"),
        (Interval::MAJ2, 2, "2", "M2"),
        (Interval::MIN3, 3, "b3", "m3"),
        (Interval::MAJ3, 4, "3", "M3"),
        (Interval::P4, 5, "4", "P4"),
        (Interval::AUG4, 6, "#4", "A4"),
        (Interval::DIM5, 6, "b5", "d5"),
        (Interval::P5, 7, "5", "P5"),
        (Interval::AUG5, 8, "#5", "A5"),
        (Interval::MIN6, 8, "b6", "m6"),
        (Interval::MAJ6, 9, "6", "M6"),
        (Interval::DIM7, 9, "bb7", "d7"),
        (Interval::MIN7, 10, "b7", "m7"),
        (Interval::MAJ7, 11, "7", "M7"),
        (Interval::P8, 12, "8", "P8"),
        (Interval::MAJ9, 14, "9", "M9"),
        (Interval::P11, 17, "11", "P11"),
        (Interval::MAJ13, 21, "13", "M13"),
    ];

    #[test]
    fn a3_sizes_labels_and_names() {
        for (interval, semis, label, short) in SIZE_CASES {
            assert_eq!(interval.semitones(), *semis, "case {short}");
            assert_eq!(interval.degree_label(), *label, "case {short}");
            assert_eq!(interval.to_string(), *short, "case {short}");
        }
        let flat9 = Interval::new(IntervalQuality::Minor, 9).unwrap();
        let sharp11 = Interval::new(IntervalQuality::Augmented, 11).unwrap();
        let sharp9 = Interval::new(IntervalQuality::Augmented, 9).unwrap();
        let aug_unison = Interval::new(IntervalQuality::Augmented, 1).unwrap();
        assert_eq!(flat9.degree_label(), "b9");
        assert_eq!(sharp11.degree_label(), "#11");
        assert_eq!(sharp9.degree_label(), "#9");
        assert_eq!(aug_unison.degree_label(), "#1");
        assert_eq!(Interval::P5.quality(), IntervalQuality::Perfect);
        assert_eq!(Interval::P5.number(), 5);
    }

    /// (from, to, expected interval)
    const BETWEEN_CASES: &[(&str, &str, Interval)] = &[
        ("C", "E", Interval::MAJ3),
        ("C", "Eb", Interval::MIN3),
        ("F#", "A", Interval::MIN3),
        ("Db", "F", Interval::MAJ3),
        ("B", "F", Interval::DIM5),
        ("C", "C", Interval::P1),
        ("E", "C", Interval::MIN6),
        ("G", "F", Interval::MIN7),
        ("C", "F#", Interval::AUG4),
    ];

    #[test]
    fn a3_between_notes() {
        for (from, to, expected) in BETWEEN_CASES {
            assert_eq!(
                Interval::between(note(from), note(to)).unwrap(),
                *expected,
                "case {from}-{to}"
            );
        }
        let d8 = Interval::new(IntervalQuality::Diminished, 8).unwrap();
        assert_eq!(Interval::between(note("C"), note("Cb")).unwrap(), d8);
        let a7 = Interval::new(IntervalQuality::Augmented, 7).unwrap();
        assert_eq!(Interval::between(note("C"), note("B#")).unwrap(), a7);
        assert!(Interval::between(note("Cb"), note("C#")).is_err());
    }

    /// (interval, inversion)
    const INVERT_CASES: &[(Interval, Interval)] = &[
        (Interval::MAJ3, Interval::MIN6),
        (Interval::MIN3, Interval::MAJ6),
        (Interval::P5, Interval::P4),
        (Interval::P4, Interval::P5),
        (Interval::MAJ7, Interval::MIN2),
        (Interval::MAJ2, Interval::MIN7),
        (Interval::AUG4, Interval::DIM5),
        (Interval::DIM5, Interval::AUG4),
        (Interval::P1, Interval::P8),
        (Interval::P8, Interval::P1),
        (Interval::MAJ9, Interval::MIN7),
        (Interval::P11, Interval::P5),
    ];

    #[test]
    fn a3_invert() {
        for (interval, inverse) in INVERT_CASES {
            assert_eq!(interval.invert().unwrap(), *inverse, "case {interval}");
        }
        let aug8 = Interval::new(IntervalQuality::Augmented, 8).unwrap();
        assert!(aug8.invert().is_err());
        let p15 = Interval::new(IntervalQuality::Perfect, 15).unwrap();
        assert_eq!(p15.invert().unwrap(), Interval::P8);
    }

    #[test]
    fn a3_interval_validates_on_deserialize() {
        let ok: Interval = serde_json::from_str(r#"{"quality":"Major","number":3}"#).unwrap();
        assert_eq!(ok, Interval::MAJ3);
        assert!(serde_json::from_str::<Interval>(r#"{"quality":"Perfect","number":3}"#).is_err());
    }
}
