//! Scales and modes, spelled by letter stepping.

use serde::{Deserialize, Serialize};

use crate::error::TheoryError;
use crate::interval::Interval;
use crate::pitch::{AccidentalPreference, Note, PitchClass};

/// A scale degree `1..=7`.
///
/// ```
/// use mackey_theory::Degree;
/// assert_eq!(Degree::new(5).unwrap().value(), 5);
/// assert!(Degree::new(8).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, type = "number"))]
pub struct Degree(u8);

impl Degree {
    /// Validates `1..=7`.
    ///
    /// # Errors
    /// [`TheoryError::InvalidDegree`] outside `1..=7`.
    pub fn new(value: u8) -> Result<Self, TheoryError> {
        if (1..=7).contains(&value) {
            Ok(Self(value))
        } else {
            Err(TheoryError::InvalidDegree(value))
        }
    }

    /// The 1-based degree number.
    #[must_use]
    pub fn value(self) -> u8 {
        self.0
    }

    /// The 0-based index (degree 1 = 0).
    #[must_use]
    pub fn index(self) -> usize {
        usize::from(self.0 - 1)
    }

    /// All seven degrees in order.
    #[must_use]
    pub fn all() -> [Degree; 7] {
        [
            Degree(1),
            Degree(2),
            Degree(3),
            Degree(4),
            Degree(5),
            Degree(6),
            Degree(7),
        ]
    }
}

impl TryFrom<u8> for Degree {
    type Error = TheoryError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Degree> for u8 {
    fn from(d: Degree) -> Self {
        d.0
    }
}

crate::pitch::checked_serde!(Degree, u8);

/// A scale or mode type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum Mode {
    /// Major (Ionian).
    Major,
    /// Natural minor (Aeolian).
    NaturalMinor,
    /// Harmonic minor.
    HarmonicMinor,
    /// Dorian.
    Dorian,
    /// Phrygian.
    Phrygian,
    /// Lydian.
    Lydian,
    /// Mixolydian.
    Mixolydian,
    /// Locrian.
    Locrian,
    /// Major pentatonic.
    MajorPentatonic,
    /// Minor pentatonic.
    MinorPentatonic,
    /// Blues (minor pentatonic plus the flat fifth).
    Blues,
}

use Interval as I;

const MAJOR: &[Interval] = &[I::P1, I::MAJ2, I::MAJ3, I::P4, I::P5, I::MAJ6, I::MAJ7];
const NATURAL_MINOR: &[Interval] = &[I::P1, I::MAJ2, I::MIN3, I::P4, I::P5, I::MIN6, I::MIN7];
const HARMONIC_MINOR: &[Interval] = &[I::P1, I::MAJ2, I::MIN3, I::P4, I::P5, I::MIN6, I::MAJ7];
const DORIAN: &[Interval] = &[I::P1, I::MAJ2, I::MIN3, I::P4, I::P5, I::MAJ6, I::MIN7];
const PHRYGIAN: &[Interval] = &[I::P1, I::MIN2, I::MIN3, I::P4, I::P5, I::MIN6, I::MIN7];
const LYDIAN: &[Interval] = &[I::P1, I::MAJ2, I::MAJ3, I::AUG4, I::P5, I::MAJ6, I::MAJ7];
const MIXOLYDIAN: &[Interval] = &[I::P1, I::MAJ2, I::MAJ3, I::P4, I::P5, I::MAJ6, I::MIN7];
const LOCRIAN: &[Interval] = &[I::P1, I::MIN2, I::MIN3, I::P4, I::DIM5, I::MIN6, I::MIN7];
const MAJOR_PENTATONIC: &[Interval] = &[I::P1, I::MAJ2, I::MAJ3, I::P5, I::MAJ6];
const MINOR_PENTATONIC: &[Interval] = &[I::P1, I::MIN3, I::P4, I::P5, I::MIN7];
const BLUES: &[Interval] = &[I::P1, I::MIN3, I::P4, I::DIM5, I::P5, I::MIN7];

impl Mode {
    /// Every mode, in declaration order.
    pub const ALL: [Mode; 11] = [
        Mode::Major,
        Mode::NaturalMinor,
        Mode::HarmonicMinor,
        Mode::Dorian,
        Mode::Phrygian,
        Mode::Lydian,
        Mode::Mixolydian,
        Mode::Locrian,
        Mode::MajorPentatonic,
        Mode::MinorPentatonic,
        Mode::Blues,
    ];

    /// The seven modes of the major scale (usable as a key, G-1.4).
    pub const DIATONIC: [Mode; 7] = [
        Mode::Major,
        Mode::Dorian,
        Mode::Phrygian,
        Mode::Lydian,
        Mode::Mixolydian,
        Mode::NaturalMinor,
        Mode::Locrian,
    ];

    /// Intervals above the tonic.
    ///
    /// ```
    /// use mackey_theory::{Interval, Mode};
    /// assert_eq!(Mode::Blues.intervals().len(), 6);
    /// assert_eq!(Mode::Major.intervals()[2], Interval::MAJ3);
    /// ```
    #[must_use]
    pub fn intervals(self) -> &'static [Interval] {
        match self {
            Mode::Major => MAJOR,
            Mode::NaturalMinor => NATURAL_MINOR,
            Mode::HarmonicMinor => HARMONIC_MINOR,
            Mode::Dorian => DORIAN,
            Mode::Phrygian => PHRYGIAN,
            Mode::Lydian => LYDIAN,
            Mode::Mixolydian => MIXOLYDIAN,
            Mode::Locrian => LOCRIAN,
            Mode::MajorPentatonic => MAJOR_PENTATONIC,
            Mode::MinorPentatonic => MINOR_PENTATONIC,
            Mode::Blues => BLUES,
        }
    }

    /// The name used after a tonic: "G major", "G minor" (natural), "E Dorian".
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            Mode::Major => "major",
            Mode::NaturalMinor => "minor",
            Mode::HarmonicMinor => "harmonic minor",
            Mode::Dorian => "Dorian",
            Mode::Phrygian => "Phrygian",
            Mode::Lydian => "Lydian",
            Mode::Mixolydian => "Mixolydian",
            Mode::Locrian => "Locrian",
            Mode::MajorPentatonic => "major pentatonic",
            Mode::MinorPentatonic => "minor pentatonic",
            Mode::Blues => "blues",
        }
    }

    /// True for the seven modes of the major scale (harmonic minor, pentatonics and blues are
    /// not).
    #[must_use]
    pub fn is_diatonic_mode(self) -> bool {
        self.parent_degree().is_some()
    }

    /// Which degree of its parent major scale the mode starts on (Dorian 2 ... Locrian 7).
    #[must_use]
    pub fn parent_degree(self) -> Option<u8> {
        match self {
            Mode::Major => Some(1),
            Mode::Dorian => Some(2),
            Mode::Phrygian => Some(3),
            Mode::Lydian => Some(4),
            Mode::Mixolydian => Some(5),
            Mode::NaturalMinor => Some(6),
            Mode::Locrian => Some(7),
            _ => None,
        }
    }

    /// Semitones from the mode's tonic down to its parent major tonic (Dorian 2, Phrygian 4,
    /// Lydian 5, Mixolydian 7, natural minor 9, Locrian 11, major 0).
    #[must_use]
    pub fn parent_major_offset(self) -> Option<u8> {
        match self {
            Mode::Major => Some(0),
            Mode::Dorian => Some(2),
            Mode::Phrygian => Some(4),
            Mode::Lydian => Some(5),
            Mode::Mixolydian => Some(7),
            Mode::NaturalMinor => Some(9),
            Mode::Locrian => Some(11),
            _ => None,
        }
    }

    /// Major <-> natural minor; `None` for every other mode.
    #[must_use]
    pub fn relative(self) -> Option<Mode> {
        match self {
            Mode::Major => Some(Mode::NaturalMinor),
            Mode::NaturalMinor => Some(Mode::Major),
            _ => None,
        }
    }
}

/// A tonic plus a mode: the spelled notes of a scale.
///
/// ```
/// use mackey_theory::{Mode, Scale};
/// let g_dorian = Scale { tonic: "G".parse().unwrap(), mode: Mode::Dorian };
/// let names: Vec<String> = g_dorian.degrees().iter().map(|n| n.name()).collect();
/// assert_eq!(names, ["G", "A", "Bb", "C", "D", "E", "F"]);
/// assert_eq!(g_dorian.name(), "G Dorian");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Scale {
    /// The first note.
    pub tonic: Note,
    /// The scale type.
    pub mode: Mode,
}

impl Scale {
    /// The scale notes in order, each spelled by stepping letters from the tonic. In the rare
    /// case a spelling would need a triple accidental (only from exotic tonics such as `F##`),
    /// that note is spelled with sharps instead.
    #[must_use]
    pub fn degrees(&self) -> Vec<Note> {
        self.mode
            .intervals()
            .iter()
            .map(|i| {
                i.apply(self.tonic).unwrap_or_else(|_| {
                    Note::from_pitch_class(
                        i.above(self.tonic.pitch_class()),
                        AccidentalPreference::Sharps,
                    )
                })
            })
            .collect()
    }

    /// True if the pitch class is a scale tone.
    #[must_use]
    pub fn contains(&self, pc: PitchClass) -> bool {
        self.degree_of(pc).is_some()
    }

    /// The 1-based degree of a pitch class in the scale, if it is a scale tone.
    ///
    /// ```
    /// use mackey_theory::{Mode, PitchClass, Scale};
    /// let c = Scale { tonic: "C".parse().unwrap(), mode: Mode::Major };
    /// assert_eq!(c.degree_of(PitchClass::new(7)).unwrap().value(), 5);
    /// assert!(c.degree_of(PitchClass::new(1)).is_none());
    /// ```
    #[must_use]
    pub fn degree_of(&self, pc: PitchClass) -> Option<Degree> {
        let offset = self.tonic.pitch_class().semitones_to(pc);
        self.mode
            .intervals()
            .iter()
            .position(|i| i.semitones() % 12 == offset)
            .and_then(|p| u8::try_from(p + 1).ok())
            .and_then(|n| Degree::new(n).ok())
    }

    /// The note at a 1-based degree of a seven-note scale (`None` for scales with fewer notes).
    #[must_use]
    pub fn note_at(&self, degree: Degree) -> Option<Note> {
        self.degrees().get(degree.index()).copied()
    }

    /// "G major", "E Dorian", "G minor".
    #[must_use]
    pub fn name(&self) -> String {
        format!("{} {}", self.tonic, self.mode.display_name())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scale(tonic: &str, mode: Mode) -> Scale {
        Scale {
            tonic: tonic.parse().unwrap(),
            mode,
        }
    }

    fn spelled(tonic: &str, mode: Mode) -> String {
        scale(tonic, mode)
            .degrees()
            .iter()
            .map(|n| n.name())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// (mode, [expected spelling in C, G, F, Bb, E]), written by hand from the interval formulas.
    const TONICS: [&str; 5] = ["C", "G", "F", "Bb", "E"];
    const CASES: &[(Mode, [&str; 5])] = &[
        (
            Mode::Major,
            [
                "C D E F G A B",
                "G A B C D E F#",
                "F G A Bb C D E",
                "Bb C D Eb F G A",
                "E F# G# A B C# D#",
            ],
        ),
        (
            Mode::NaturalMinor,
            [
                "C D Eb F G Ab Bb",
                "G A Bb C D Eb F",
                "F G Ab Bb C Db Eb",
                "Bb C Db Eb F Gb Ab",
                "E F# G A B C D",
            ],
        ),
        (
            Mode::HarmonicMinor,
            [
                "C D Eb F G Ab B",
                "G A Bb C D Eb F#",
                "F G Ab Bb C Db E",
                "Bb C Db Eb F Gb A",
                "E F# G A B C D#",
            ],
        ),
        (
            Mode::Dorian,
            [
                "C D Eb F G A Bb",
                "G A Bb C D E F",
                "F G Ab Bb C D Eb",
                "Bb C Db Eb F G Ab",
                "E F# G A B C# D",
            ],
        ),
        (
            Mode::Phrygian,
            [
                "C Db Eb F G Ab Bb",
                "G Ab Bb C D Eb F",
                "F Gb Ab Bb C Db Eb",
                "Bb Cb Db Eb F Gb Ab",
                "E F G A B C D",
            ],
        ),
        (
            Mode::Lydian,
            [
                "C D E F# G A B",
                "G A B C# D E F#",
                "F G A B C D E",
                "Bb C D E F G A",
                "E F# G# A# B C# D#",
            ],
        ),
        (
            Mode::Mixolydian,
            [
                "C D E F G A Bb",
                "G A B C D E F",
                "F G A Bb C D Eb",
                "Bb C D Eb F G Ab",
                "E F# G# A B C# D",
            ],
        ),
        (
            Mode::Locrian,
            [
                "C Db Eb F Gb Ab Bb",
                "G Ab Bb C Db Eb F",
                "F Gb Ab Bb Cb Db Eb",
                "Bb Cb Db Eb Fb Gb Ab",
                "E F G A Bb C D",
            ],
        ),
        (
            Mode::MajorPentatonic,
            [
                "C D E G A",
                "G A B D E",
                "F G A C D",
                "Bb C D F G",
                "E F# G# B C#",
            ],
        ),
        (
            Mode::MinorPentatonic,
            [
                "C Eb F G Bb",
                "G Bb C D F",
                "F Ab Bb C Eb",
                "Bb Db Eb F Ab",
                "E G A B D",
            ],
        ),
        (
            Mode::Blues,
            [
                "C Eb F Gb G Bb",
                "G Bb C Db D F",
                "F Ab Bb Cb C Eb",
                "Bb Db Eb Fb F Ab",
                "E G A Bb B D",
            ],
        ),
    ];

    #[test]
    fn a4_every_mode_in_five_tonics() {
        for (mode, expected) in CASES {
            for (tonic, want) in TONICS.iter().zip(expected) {
                assert_eq!(spelled(tonic, *mode), *want, "case {tonic} {mode:?}");
            }
        }
    }

    #[test]
    fn a4_all_eleven_modes_are_covered_by_the_table() {
        let covered: Vec<Mode> = CASES.iter().map(|(m, _)| *m).collect();
        for mode in Mode::ALL {
            assert!(covered.contains(&mode), "mode {mode:?} missing from CASES");
        }
    }

    #[test]
    fn fb_2_f_major_notes_are_f_g_a_bb_c_d_e() {
        assert_eq!(spelled("F", Mode::Major), "F G A Bb C D E");
    }

    #[test]
    fn a4_names() {
        assert_eq!(scale("G", Mode::Major).name(), "G major");
        assert_eq!(scale("G", Mode::NaturalMinor).name(), "G minor");
        assert_eq!(scale("E", Mode::Dorian).name(), "E Dorian");
        assert_eq!(scale("A", Mode::Blues).name(), "A blues");
    }

    #[test]
    fn a4_degree_of_and_contains() {
        let c = scale("C", Mode::Major);
        for (pc, degree) in [
            (0, Some(1)),
            (2, Some(2)),
            (11, Some(7)),
            (1, None),
            (10, None),
        ] {
            let got = c.degree_of(PitchClass::new(pc)).map(Degree::value);
            assert_eq!(got, degree, "case pc {pc}");
            assert_eq!(
                c.contains(PitchClass::new(pc)),
                degree.is_some(),
                "case pc {pc}"
            );
        }
        let blues = scale("C", Mode::Blues);
        assert_eq!(
            blues.degree_of(PitchClass::new(6)).map(Degree::value),
            Some(4)
        );
        assert_eq!(blues.note_at(Degree::new(7).unwrap()), None);
        assert_eq!(
            c.note_at(Degree::new(7).unwrap())
                .map(Note::name)
                .as_deref(),
            Some("B")
        );
    }

    #[test]
    fn a4_mode_facts() {
        assert_eq!(Mode::Dorian.parent_major_offset(), Some(2));
        assert_eq!(Mode::Locrian.parent_major_offset(), Some(11));
        assert_eq!(Mode::HarmonicMinor.parent_major_offset(), None);
        assert!(Mode::NaturalMinor.is_diatonic_mode());
        assert!(!Mode::Blues.is_diatonic_mode());
        assert!(!Mode::HarmonicMinor.is_diatonic_mode());
        assert_eq!(Mode::Major.relative(), Some(Mode::NaturalMinor));
        assert_eq!(Mode::Dorian.relative(), None);
        assert_eq!(Mode::DIATONIC.len(), 7);
    }

    #[test]
    fn a4_degree_validates() {
        assert!(Degree::new(0).is_err());
        assert!(Degree::new(8).is_err());
        assert_eq!(Degree::try_from(3u8).unwrap().index(), 2);
        assert!(serde_json::from_str::<Degree>("9").is_err());
        assert_eq!(
            serde_json::to_string(&Degree::new(4).unwrap()).unwrap(),
            "4"
        );
    }

    #[test]
    fn a4_exotic_tonic_falls_back_to_a_simple_spelling() {
        // F## harmonic minor needs a triple accidental on its fifth; it is spelled with sharps.
        let degrees = scale("F##", Mode::HarmonicMinor).degrees();
        assert_eq!(degrees.len(), 7);
    }
}
