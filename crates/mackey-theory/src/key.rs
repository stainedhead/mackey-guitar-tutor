//! Keys: offered tonics, key signatures, the circle of fifths and relative keys.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::interval::Interval;
use crate::pitch::{Accidental, AccidentalPreference, Letter, Note, PitchClass};
use crate::scale::{Degree, Mode, Scale};

/// Why a key could not be built.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum KeyError {
    /// The tonic is not one of the keys the app offers (G-1.2, G-1.3), for example C# major.
    #[error("{tonic} {} is not an offered key", mode.display_name())]
    TonicNotOffered {
        /// The refused tonic.
        tonic: Note,
        /// The requested mode.
        mode: Mode,
    },
    /// Only the seven modes of the major scale can be a key (not blues or pentatonics).
    #[error("{} cannot be used as a key", mode.display_name())]
    UnsupportedMode {
        /// The refused mode.
        mode: Mode,
    },
}

const fn n(letter: Letter, accidental: Accidental) -> Note {
    Note { letter, accidental }
}

/// Major keys the picker offers (G-1.2): C G D A E B, F#/Gb, F Bb Eb Ab Db. C# and Cb major are
/// not offered.
pub const OFFERED_MAJOR_TONICS: [Note; 13] = [
    n(Letter::C, Accidental::NATURAL),
    n(Letter::G, Accidental::NATURAL),
    n(Letter::D, Accidental::NATURAL),
    n(Letter::A, Accidental::NATURAL),
    n(Letter::E, Accidental::NATURAL),
    n(Letter::B, Accidental::NATURAL),
    n(Letter::F, Accidental::SHARP),
    n(Letter::G, Accidental::FLAT),
    n(Letter::F, Accidental::NATURAL),
    n(Letter::B, Accidental::FLAT),
    n(Letter::E, Accidental::FLAT),
    n(Letter::A, Accidental::FLAT),
    n(Letter::D, Accidental::FLAT),
];

/// Minor keys the picker offers (G-1.3): A E B F# C# G# D#/Eb D G C F Bb.
pub const OFFERED_MINOR_TONICS: [Note; 13] = [
    n(Letter::A, Accidental::NATURAL),
    n(Letter::E, Accidental::NATURAL),
    n(Letter::B, Accidental::NATURAL),
    n(Letter::F, Accidental::SHARP),
    n(Letter::C, Accidental::SHARP),
    n(Letter::G, Accidental::SHARP),
    n(Letter::D, Accidental::SHARP),
    n(Letter::E, Accidental::FLAT),
    n(Letter::D, Accidental::NATURAL),
    n(Letter::G, Accidental::NATURAL),
    n(Letter::C, Accidental::NATURAL),
    n(Letter::F, Accidental::NATURAL),
    n(Letter::B, Accidental::FLAT),
];

/// A key signature: `-7..=7`, sharps positive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct KeySignature {
    /// Number of accidentals; positive for sharps, negative for flats.
    pub accidentals: i8,
}

impl KeySignature {
    /// How many sharps or flats.
    #[must_use]
    pub fn count(self) -> u8 {
        self.accidentals.unsigned_abs()
    }
}

/// Reported when [`Key::new_normalised`] changed the requested tonic (G-1.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct KeyNotice {
    /// What was asked for.
    pub requested: Note,
    /// What is used instead.
    pub used: Note,
    /// The mode of both.
    pub mode: Mode,
}

impl KeyNotice {
    /// Text for the UI: "D# Dorian is shown as Eb Dorian".
    #[must_use]
    pub fn message(&self) -> String {
        let mode = self.mode.display_name();
        format!("{} {mode} is shown as {} {mode}", self.requested, self.used)
    }
}

/// A key the app can show: a tonic from the offered set and one of the seven modes.
///
/// ```
/// use mackey_theory::{Key, Mode};
/// let f = Key::new("F".parse().unwrap(), Mode::Major).unwrap();
/// let names: Vec<String> = f.scale().degrees().iter().map(|n| n.name()).collect();
/// assert_eq!(names, ["F", "G", "A", "Bb", "C", "D", "E"]);
/// assert!(Key::new("C#".parse().unwrap(), Mode::Major).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Key {
    tonic: Note,
    mode: Mode,
}

#[derive(Deserialize)]
struct KeyRepr {
    tonic: Note,
    mode: Mode,
}

impl<'de> Deserialize<'de> for Key {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let repr = KeyRepr::deserialize(deserializer)?;
        Key::new(repr.tonic, repr.mode).map_err(serde::de::Error::custom)
    }
}

impl Key {
    /// The major scale a mode shares its key signature with, spelled by letter.
    fn parent_major(tonic: Note, mode: Mode) -> Result<Note, KeyError> {
        let (Some(degree), Some(offset)) = (mode.parent_degree(), mode.parent_major_offset())
        else {
            return Err(KeyError::UnsupportedMode { mode });
        };
        let steps = i8::try_from(degree - 1).unwrap_or(0);
        let letter = tonic.letter.step(-steps);
        let pc = tonic.pitch_class().transpose(-i32::from(offset));
        Note::from_letter_and_pitch_class(letter, pc)
            .ok_or(KeyError::TonicNotOffered { tonic, mode })
    }

    /// Builds a key exactly as given.
    ///
    /// The mode takes the key signature of its parent major key, and that parent must be one of
    /// the offered major keys (so `D# Dorian`, whose parent is C# major, is refused).
    ///
    /// # Errors
    /// [`KeyError::TonicNotOffered`] or [`KeyError::UnsupportedMode`].
    pub fn new(tonic: Note, mode: Mode) -> Result<Self, KeyError> {
        let parent = Self::parent_major(tonic, mode)?;
        if OFFERED_MAJOR_TONICS.contains(&parent) {
            Ok(Self { tonic, mode })
        } else {
            Err(KeyError::TonicNotOffered { tonic, mode })
        }
    }

    /// Builds a key, replacing a tonic that is not offered by its enharmonic equivalent (G-1.4):
    /// `D# Dorian` becomes `Eb Dorian`, `C# major` becomes `Db major`. The notice says what
    /// changed, if anything.
    ///
    /// ```
    /// use mackey_theory::{Key, Mode};
    /// let (key, notice) = Key::new_normalised("D#".parse().unwrap(), Mode::Dorian).unwrap();
    /// assert_eq!(key.name(), "Eb Dorian");
    /// assert!(notice.is_some());
    /// ```
    ///
    /// # Errors
    /// [`KeyError`] when no spelling of the pitch class gives an offered key.
    pub fn new_normalised(tonic: Note, mode: Mode) -> Result<(Self, Option<KeyNotice>), KeyError> {
        let first = Self::new(tonic, mode);
        let err = match first {
            Ok(key) => return Ok((key, None)),
            Err(e) => e,
        };
        let pc = tonic.pitch_class();
        for preference in [AccidentalPreference::Sharps, AccidentalPreference::Flats] {
            let candidate = Note::from_pitch_class(pc, preference);
            if let Ok(key) = Self::new(candidate, mode) {
                let notice = KeyNotice {
                    requested: tonic,
                    used: candidate,
                    mode,
                };
                return Ok((key, Some(notice)));
            }
        }
        Err(err)
    }

    /// The tonic as spelled (the F#/Gb choice is the tonic's spelling).
    #[must_use]
    pub fn tonic(&self) -> Note {
        self.tonic
    }

    /// The mode.
    #[must_use]
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// The scale of the key.
    #[must_use]
    pub fn scale(&self) -> Scale {
        Scale {
            tonic: self.tonic,
            mode: self.mode,
        }
    }

    /// The key signature of the parent major key: `+n` sharps, `-n` flats.
    ///
    /// ```
    /// use mackey_theory::{Key, Mode};
    /// let key = Key::new("Eb".parse().unwrap(), Mode::Major).unwrap();
    /// assert_eq!(key.signature().accidentals, -3);
    /// ```
    #[must_use]
    pub fn signature(&self) -> KeySignature {
        let parent = Self::parent_major(self.tonic, self.mode).unwrap_or(self.tonic);
        let letter_fifths: i8 = match parent.letter {
            Letter::C => 0,
            Letter::G => 1,
            Letter::D => 2,
            Letter::A => 3,
            Letter::E => 4,
            Letter::B => 5,
            Letter::F => -1,
        };
        KeySignature {
            accidentals: letter_fifths + 7 * parent.accidental.value(),
        }
    }

    /// How to spell a note that neither a chord nor the scale spells (G-1.5): flats for flat
    /// keys and for keys with no accidentals, sharps for sharp keys.
    #[must_use]
    pub fn preference(&self) -> AccidentalPreference {
        if self.signature().accidentals > 0 {
            AccidentalPreference::Sharps
        } else {
            AccidentalPreference::Flats
        }
    }

    fn step_fifth(self, up: bool) -> Self {
        let moved = if up {
            Interval::P5.apply(self.tonic)
        } else {
            Interval::P5.apply_down(self.tonic)
        };
        moved
            .ok()
            .and_then(|t| Self::new_normalised(t, self.mode).ok())
            .map_or(self, |(key, _)| key)
    }

    /// The next key clockwise on the circle of fifths (a fifth up) in the same mode. Twelve
    /// steps return to the start. F# major goes on to Db (C# is not offered).
    ///
    /// ```
    /// use mackey_theory::{Key, Mode};
    /// let c = Key::new("C".parse().unwrap(), Mode::Major).unwrap();
    /// assert_eq!(c.next_fifth().name(), "G major");
    /// assert_eq!(c.prev_fifth().name(), "F major");
    /// ```
    #[must_use]
    pub fn next_fifth(&self) -> Self {
        self.step_fifth(true)
    }

    /// The next key counter-clockwise (a fifth down) in the same mode.
    #[must_use]
    pub fn prev_fifth(&self) -> Self {
        self.step_fifth(false)
    }

    /// Major <-> natural minor (A minor <-> C major); `None` for every other mode (D-26).
    ///
    /// ```
    /// use mackey_theory::{Key, Mode};
    /// let c = Key::new("C".parse().unwrap(), Mode::Major).unwrap();
    /// assert_eq!(c.relative().unwrap().name(), "A minor");
    /// ```
    #[must_use]
    pub fn relative(&self) -> Option<Self> {
        let (tonic, mode) = match self.mode {
            Mode::Major => (Interval::MAJ6.apply(self.tonic).ok()?, Mode::NaturalMinor),
            Mode::NaturalMinor => (Interval::MIN3.apply(self.tonic).ok()?, Mode::Major),
            _ => return None,
        };
        Self::new(tonic, mode).ok()
    }

    /// The other spelling of a key with six accidentals (F# major <-> Gb major, D# minor <->
    /// Eb minor); `None` for every other key (E-FB-7).
    #[must_use]
    pub fn enharmonic_alternative(&self) -> Option<Self> {
        if self.signature().count() != 6 {
            return None;
        }
        let pc = self.tonic.pitch_class();
        [AccidentalPreference::Sharps, AccidentalPreference::Flats]
            .into_iter()
            .map(|p| Note::from_pitch_class(pc, p))
            .filter(|candidate| *candidate != self.tonic)
            .find_map(|candidate| Self::new(candidate, self.mode).ok())
    }

    /// The degree of a pitch class in the key's scale, if it is a scale tone.
    #[must_use]
    pub fn degree_of(&self, pc: PitchClass) -> Option<Degree> {
        self.scale().degree_of(pc)
    }

    /// Display name: "G major", "E Dorian", "A minor".
    #[must_use]
    pub fn name(&self) -> String {
        self.scale().name()
    }

    /// Every offered key of one mode, in circle-of-fifths order starting at C or A.
    #[must_use]
    pub fn all_offered(mode: Mode) -> Vec<Key> {
        let mut keys = Vec::new();
        let start = if mode == Mode::NaturalMinor {
            Note::natural(Letter::A)
        } else {
            Note::natural(Letter::C)
        };
        let Ok(first) = Self::new(start, mode) else {
            return keys;
        };
        let mut key = first;
        for _ in 0..12 {
            keys.push(key);
            key = key.next_fifth();
        }
        keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(s: &str) -> Note {
        s.parse().unwrap()
    }

    fn key(tonic: &str, mode: Mode) -> Key {
        Key::new(note(tonic), mode).unwrap()
    }

    fn spelled(k: Key) -> String {
        k.scale()
            .degrees()
            .iter()
            .map(|n| n.name())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// (tonic, signature, spelled major scale) for the 13 offered major keys.
    const MAJOR_CASES: &[(&str, i8, &str)] = &[
        ("C", 0, "C D E F G A B"),
        ("G", 1, "G A B C D E F#"),
        ("D", 2, "D E F# G A B C#"),
        ("A", 3, "A B C# D E F# G#"),
        ("E", 4, "E F# G# A B C# D#"),
        ("B", 5, "B C# D# E F# G# A#"),
        ("F#", 6, "F# G# A# B C# D# E#"),
        ("Gb", -6, "Gb Ab Bb Cb Db Eb F"),
        ("F", -1, "F G A Bb C D E"),
        ("Bb", -2, "Bb C D Eb F G A"),
        ("Eb", -3, "Eb F G Ab Bb C D"),
        ("Ab", -4, "Ab Bb C Db Eb F G"),
        ("Db", -5, "Db Eb F Gb Ab Bb C"),
    ];

    /// (tonic, signature, spelled natural minor scale) for the 13 offered minor keys.
    const MINOR_CASES: &[(&str, i8, &str)] = &[
        ("A", 0, "A B C D E F G"),
        ("E", 1, "E F# G A B C D"),
        ("B", 2, "B C# D E F# G A"),
        ("F#", 3, "F# G# A B C# D E"),
        ("C#", 4, "C# D# E F# G# A B"),
        ("G#", 5, "G# A# B C# D# E F#"),
        ("D#", 6, "D# E# F# G# A# B C#"),
        ("Eb", -6, "Eb F Gb Ab Bb Cb Db"),
        ("D", -1, "D E F G A Bb C"),
        ("G", -2, "G A Bb C D Eb F"),
        ("C", -3, "C D Eb F G Ab Bb"),
        ("F", -4, "F G Ab Bb C Db Eb"),
        ("Bb", -5, "Bb C Db Eb F Gb Ab"),
    ];

    #[test]
    fn g_1_2_every_offered_major_key() {
        assert_eq!(OFFERED_MAJOR_TONICS.len(), MAJOR_CASES.len());
        for (tonic, sig, scale) in MAJOR_CASES {
            let k = key(tonic, Mode::Major);
            assert_eq!(
                k.signature().accidentals,
                *sig,
                "case {tonic} major signature"
            );
            assert_eq!(spelled(k), *scale, "case {tonic} major scale");
        }
    }

    #[test]
    fn g_1_3_every_offered_minor_key() {
        assert_eq!(OFFERED_MINOR_TONICS.len(), MINOR_CASES.len());
        for (tonic, sig, scale) in MINOR_CASES {
            let k = key(tonic, Mode::NaturalMinor);
            assert_eq!(
                k.signature().accidentals,
                *sig,
                "case {tonic} minor signature"
            );
            assert_eq!(spelled(k), *scale, "case {tonic} minor scale");
        }
    }

    #[test]
    fn g_1_2_offered_sets_match_the_picker_lists() {
        let majors: Vec<String> = OFFERED_MAJOR_TONICS.iter().map(|n| n.name()).collect();
        assert_eq!(
            majors,
            [
                "C", "G", "D", "A", "E", "B", "F#", "Gb", "F", "Bb", "Eb", "Ab", "Db"
            ]
        );
        let minors: Vec<String> = OFFERED_MINOR_TONICS.iter().map(|n| n.name()).collect();
        assert_eq!(
            minors,
            [
                "A", "E", "B", "F#", "C#", "G#", "D#", "Eb", "D", "G", "C", "F", "Bb"
            ]
        );
    }

    #[test]
    fn g_1_2_unoffered_keys_are_rejected() {
        for t in ["C#", "Cb", "G#", "D#", "A#", "E#", "Fb"] {
            let err = Key::new(note(t), Mode::Major).unwrap_err();
            assert!(
                matches!(err, KeyError::TonicNotOffered { .. }),
                "case {t} major"
            );
        }
        for t in ["Ab", "Db", "Gb", "A#", "Cb", "E#", "Fb"] {
            assert!(
                Key::new(note(t), Mode::NaturalMinor).is_err(),
                "case {t} minor"
            );
        }
    }

    #[test]
    fn g_1_2_offered_sets_agree_with_validation() {
        let all_names = [
            "C", "C#", "Db", "D", "D#", "Eb", "E", "F", "F#", "Gb", "G", "G#", "Ab", "A", "A#",
            "Bb", "B", "Cb", "E#", "Fb", "B#",
        ];
        for name in all_names {
            let n = note(name);
            assert_eq!(
                Key::new(n, Mode::Major).is_ok(),
                OFFERED_MAJOR_TONICS.contains(&n),
                "case {name} major"
            );
            assert_eq!(
                Key::new(n, Mode::NaturalMinor).is_ok(),
                OFFERED_MINOR_TONICS.contains(&n),
                "case {name} minor"
            );
        }
    }

    #[test]
    fn g_1_4_modes_use_the_parent_key_signature() {
        // (tonic, mode, signature, spelling)
        let cases: &[(&str, Mode, i8, &str)] = &[
            ("G", Mode::Dorian, -1, "G A Bb C D E F"),
            ("D", Mode::Dorian, 0, "D E F G A B C"),
            ("E", Mode::Phrygian, 0, "E F G A B C D"),
            ("F", Mode::Lydian, 0, "F G A B C D E"),
            ("G", Mode::Mixolydian, 0, "G A B C D E F"),
            ("B", Mode::Locrian, 0, "B C D E F G A"),
            ("Eb", Mode::Dorian, -5, "Eb F Gb Ab Bb C Db"),
            ("F#", Mode::Mixolydian, 5, "F# G# A# B C# D# E"),
        ];
        for (tonic, mode, sig, scale) in cases {
            let k = key(tonic, *mode);
            assert_eq!(k.signature().accidentals, *sig, "case {tonic} {mode:?}");
            assert_eq!(spelled(k), *scale, "case {tonic} {mode:?}");
        }
    }

    #[test]
    fn g_1_4_d_sharp_dorian_becomes_e_flat_dorian_with_a_notice() {
        let (k, notice) = Key::new_normalised(note("D#"), Mode::Dorian).unwrap();
        assert_eq!(k.name(), "Eb Dorian");
        let notice = notice.unwrap();
        assert_eq!(notice.message(), "D# Dorian is shown as Eb Dorian");
        assert_eq!((notice.requested, notice.used), (note("D#"), note("Eb")));
    }

    #[test]
    fn g_1_4_normalisation_table() {
        // (tonic, mode, expected name, notice expected)
        let cases: &[(&str, Mode, &str, bool)] = &[
            ("C#", Mode::Major, "Db major", true),
            ("Cb", Mode::Major, "B major", true),
            ("Ab", Mode::NaturalMinor, "G# minor", true),
            ("A#", Mode::NaturalMinor, "Bb minor", true),
            ("Db", Mode::NaturalMinor, "C# minor", true),
            ("G", Mode::Major, "G major", false),
            ("F#", Mode::Major, "F# major", false),
            ("Gb", Mode::Major, "Gb major", false),
            ("A#", Mode::Dorian, "Bb Dorian", true),
            ("Gb", Mode::Lydian, "Gb Lydian", false),
        ];
        for (tonic, mode, name, notice) in cases {
            let (k, n) = Key::new_normalised(note(tonic), *mode).unwrap();
            assert_eq!(k.name(), *name, "case {tonic} {mode:?}");
            assert_eq!(n.is_some(), *notice, "case {tonic} {mode:?} notice");
        }
    }

    #[test]
    fn g_1_4_only_the_seven_modes_can_be_keys() {
        for mode in [
            Mode::HarmonicMinor,
            Mode::MajorPentatonic,
            Mode::MinorPentatonic,
            Mode::Blues,
        ] {
            assert_eq!(
                Key::new(note("C"), mode),
                Err(KeyError::UnsupportedMode { mode }),
                "case {mode:?}"
            );
            assert!(Key::new_normalised(note("C"), mode).is_err());
        }
    }

    #[test]
    fn g_1_5_preference_follows_the_signature() {
        // flats for flat keys and C major, sharps for sharp keys
        for (tonic, mode, sharps) in [
            ("C", Mode::Major, false),
            ("A", Mode::NaturalMinor, false),
            ("G", Mode::Major, true),
            ("E", Mode::NaturalMinor, true),
            ("F", Mode::Major, false),
            ("D", Mode::NaturalMinor, false),
            ("F#", Mode::Major, true),
            ("Gb", Mode::Major, false),
            ("E", Mode::Dorian, true),
        ] {
            let expected = if sharps {
                AccidentalPreference::Sharps
            } else {
                AccidentalPreference::Flats
            };
            assert_eq!(
                key(tonic, mode).preference(),
                expected,
                "case {tonic} {mode:?}"
            );
        }
    }

    fn walk(start: Key, steps: usize, forward: bool) -> Vec<String> {
        let mut names = Vec::new();
        let mut k = start;
        for _ in 0..steps {
            k = if forward {
                k.next_fifth()
            } else {
                k.prev_fifth()
            };
            names.push(k.tonic().name());
        }
        names
    }

    #[test]
    fn a5_circle_of_fifths_major() {
        let c = key("C", Mode::Major);
        assert_eq!(
            walk(c, 12, true),
            [
                "G", "D", "A", "E", "B", "F#", "Db", "Ab", "Eb", "Bb", "F", "C"
            ]
        );
        assert_eq!(
            walk(c, 12, false),
            [
                "F", "Bb", "Eb", "Ab", "Db", "Gb", "B", "E", "A", "D", "G", "C"
            ]
        );
    }

    #[test]
    fn a5_circle_of_fifths_minor() {
        let a = key("A", Mode::NaturalMinor);
        assert_eq!(
            walk(a, 12, true),
            [
                "E", "B", "F#", "C#", "G#", "D#", "Bb", "F", "C", "G", "D", "A"
            ]
        );
        assert_eq!(
            walk(a, 12, false),
            [
                "D", "G", "C", "F", "Bb", "Eb", "G#", "C#", "F#", "B", "E", "A"
            ]
        );
    }

    #[test]
    fn a5_circle_honours_the_chosen_f_sharp_or_g_flat() {
        let gb = key("Gb", Mode::Major);
        assert_eq!(gb.next_fifth().name(), "Db major");
        assert_eq!(gb.prev_fifth().name(), "B major");
        let fs = key("F#", Mode::Major);
        assert_eq!(fs.prev_fifth().name(), "B major");
        assert_eq!(key("B", Mode::Major).next_fifth().name(), "F# major");
        assert_eq!(key("Db", Mode::Major).prev_fifth().name(), "Gb major");
    }

    #[test]
    fn a5_twelve_fifths_return_to_the_start_in_every_mode_and_key() {
        for mode in Mode::DIATONIC {
            for start in Key::all_offered(mode) {
                let mut forward = start;
                let mut backward = start;
                for _ in 0..12 {
                    forward = forward.next_fifth();
                    backward = backward.prev_fifth();
                }
                // Twelve fifths always come back to the same pitch class. The F#/Gb (D#/Eb)
                // choice is not remembered across the far side of the circle, so a key with six
                // accidentals may return in its other spelling.
                for (label, end) in [("forward", forward), ("backward", backward)] {
                    assert_eq!(
                        end.tonic().pitch_class(),
                        start.tonic().pitch_class(),
                        "case {} {label}",
                        start.name()
                    );
                    if start.signature().count() != 6 {
                        assert_eq!(end, start, "case {} {label}", start.name());
                    }
                }
            }
        }
    }

    #[test]
    fn a5_all_offered_lists_twelve_distinct_pitch_classes() {
        for mode in Mode::DIATONIC {
            let keys = Key::all_offered(mode);
            assert_eq!(keys.len(), 12, "case {mode:?}");
            let mut pcs: Vec<u8> = keys
                .iter()
                .map(|k| k.tonic().pitch_class().value())
                .collect();
            pcs.sort_unstable();
            pcs.dedup();
            assert_eq!(pcs.len(), 12, "case {mode:?}");
        }
        assert_eq!(Key::all_offered(Mode::Blues), Vec::<Key>::new());
    }

    #[test]
    fn a5_relative_keys() {
        // (key, mode, relative name)
        let cases: &[(&str, Mode, Option<&str>)] = &[
            ("C", Mode::Major, Some("A minor")),
            ("A", Mode::NaturalMinor, Some("C major")),
            ("Gb", Mode::Major, Some("Eb minor")),
            ("F#", Mode::Major, Some("D# minor")),
            ("D#", Mode::NaturalMinor, Some("F# major")),
            ("Eb", Mode::NaturalMinor, Some("Gb major")),
            ("Bb", Mode::Major, Some("G minor")),
            ("B", Mode::Major, Some("G# minor")),
            ("E", Mode::Dorian, None),
            ("G", Mode::Mixolydian, None),
        ];
        for (tonic, mode, relative) in cases {
            let got = key(tonic, *mode).relative().map(|k| k.name());
            assert_eq!(got.as_deref(), *relative, "case {tonic} {mode:?}");
        }
    }

    #[test]
    fn a5_enharmonic_alternative_only_for_six_accidentals() {
        // (key, mode, alternative)
        let cases: &[(&str, Mode, Option<&str>)] = &[
            ("F#", Mode::Major, Some("Gb major")),
            ("Gb", Mode::Major, Some("F# major")),
            ("D#", Mode::NaturalMinor, Some("Eb minor")),
            ("Eb", Mode::NaturalMinor, Some("D# minor")),
            ("C", Mode::Major, None),
            ("B", Mode::Major, None),
            ("Db", Mode::Major, None),
            ("A", Mode::NaturalMinor, None),
        ];
        for (tonic, mode, alt) in cases {
            let got = key(tonic, *mode).enharmonic_alternative().map(|k| k.name());
            assert_eq!(got.as_deref(), *alt, "case {tonic} {mode:?}");
        }
    }

    #[test]
    fn a5_degree_of_and_signature_count() {
        let g = key("G", Mode::Major);
        assert_eq!(g.degree_of(PitchClass::new(6)).map(Degree::value), Some(7));
        assert_eq!(g.degree_of(PitchClass::new(5)), None);
        assert_eq!(g.signature().count(), 1);
        assert_eq!(key("Eb", Mode::Major).signature().count(), 3);
        assert_eq!(g.mode(), Mode::Major);
    }

    #[test]
    fn a5_key_validates_on_deserialize() {
        let k: Key =
            serde_json::from_str(r#"{"tonic":{"letter":"G","accidental":0},"mode":"Major"}"#)
                .unwrap();
        assert_eq!(k.name(), "G major");
        assert!(
            serde_json::from_str::<Key>(
                r#"{"tonic":{"letter":"C","accidental":1},"mode":"Major"}"#
            )
            .is_err()
        );
        assert_eq!(
            serde_json::from_str::<Key>(&serde_json::to_string(&k).unwrap()).unwrap(),
            k
        );
    }

    #[test]
    fn a5_key_error_messages_name_the_key() {
        let e = Key::new(note("C#"), Mode::Major).unwrap_err();
        assert_eq!(e.to_string(), "C# major is not an offered key");
        let e = Key::new(note("C"), Mode::Blues).unwrap_err();
        assert_eq!(e.to_string(), "blues cannot be used as a key");
    }

    #[test]
    fn g_1_5_scale_accidentals_are_of_one_kind_and_match_the_signature() {
        for mode in Mode::DIATONIC {
            for k in Key::all_offered(mode) {
                let accidentals: Vec<i8> = k
                    .scale()
                    .degrees()
                    .iter()
                    .map(|n| n.accidental.value())
                    .filter(|a| *a != 0)
                    .collect();
                let sign = k.signature().accidentals.signum();
                for a in &accidentals {
                    assert_eq!(a.signum(), sign, "case {} accidental {a}", k.name());
                    assert_eq!(a.abs(), 1, "case {} has a double accidental", k.name());
                }
                if sign == 0 {
                    assert!(accidentals.is_empty(), "case {}", k.name());
                }
            }
        }
    }
}
