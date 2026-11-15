//! Pitch classes, letters, accidentals, spelled notes and scientific pitches.

use std::cmp::Ordering;
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::error::TheoryError;

/// Serialises a checked newtype as its primitive and validates on the way in, so a bad number
/// in a settings file or an IPC message is an error, never an invalid value.
macro_rules! checked_serde {
    ($ty:ty, $prim:ty) => {
        impl Serialize for $ty {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                <$prim>::from(*self).serialize(serializer)
            }
        }

        impl<'de> Deserialize<'de> for $ty {
            fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                let raw = <$prim>::deserialize(deserializer)?;
                <$ty>::try_from(raw).map_err(serde::de::Error::custom)
            }
        }
    };
}
pub(crate) use checked_serde;

/// One of the 12 pitch classes, C = 0 to B = 11. Spelling-free: C# and Db are the same value.
///
/// ```
/// use mackey_theory::PitchClass;
/// assert_eq!(PitchClass::new(14).value(), 2);
/// assert_eq!(PitchClass::new(-1).value(), 11);
/// assert_eq!(PitchClass::new(10).transpose(5).value(), 3);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, type = "number"))]
pub struct PitchClass(u8);

impl PitchClass {
    /// Builds a pitch class from any integer, reducing modulo 12 (so `-1` is B).
    #[must_use]
    pub fn new(value: i32) -> Self {
        Self(u8::try_from(value.rem_euclid(12)).unwrap_or(0))
    }

    /// The number `0..=11`.
    #[must_use]
    pub fn value(self) -> u8 {
        self.0
    }

    /// Moves by a number of semitones (negative moves down), wrapping around the octave.
    #[must_use]
    pub fn transpose(self, semitones: i32) -> Self {
        Self::new(i32::from(self.0) + semitones)
    }

    /// Semitones going up from `self` to `other`, `0..=11`.
    ///
    /// ```
    /// use mackey_theory::PitchClass;
    /// assert_eq!(PitchClass::new(9).semitones_to(PitchClass::new(0)), 3);
    /// ```
    #[must_use]
    pub fn semitones_to(self, other: Self) -> u8 {
        Self::new(i32::from(other.0) - i32::from(self.0)).0
    }
}

impl TryFrom<u8> for PitchClass {
    type Error = TheoryError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value < 12 {
            Ok(Self(value))
        } else {
            Err(TheoryError::PitchClassOutOfRange(value.into()))
        }
    }
}

impl From<PitchClass> for u8 {
    fn from(pc: PitchClass) -> Self {
        pc.0
    }
}

checked_serde!(PitchClass, u8);

/// A note letter, C to B.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum Letter {
    /// C
    C,
    /// D
    D,
    /// E
    E,
    /// F
    F,
    /// G
    G,
    /// A
    A,
    /// B
    B,
}

impl Letter {
    /// The seven letters in order, C first.
    pub const ALL: [Letter; 7] = [
        Letter::C,
        Letter::D,
        Letter::E,
        Letter::F,
        Letter::G,
        Letter::A,
        Letter::B,
    ];

    /// Position in the C-based alphabet (C = 0 ... B = 6).
    #[must_use]
    pub fn index(self) -> u8 {
        match self {
            Letter::C => 0,
            Letter::D => 1,
            Letter::E => 2,
            Letter::F => 3,
            Letter::G => 4,
            Letter::A => 5,
            Letter::B => 6,
        }
    }

    /// The letter at an index modulo 7.
    #[must_use]
    pub fn from_index(index: i32) -> Self {
        match index.rem_euclid(7) {
            0 => Letter::C,
            1 => Letter::D,
            2 => Letter::E,
            3 => Letter::F,
            4 => Letter::G,
            5 => Letter::A,
            _ => Letter::B,
        }
    }

    /// Pitch class of the natural note (C 0, D 2, E 4, F 5, G 7, A 9, B 11).
    #[must_use]
    pub fn natural_pc(self) -> PitchClass {
        PitchClass(match self {
            Letter::C => 0,
            Letter::D => 2,
            Letter::E => 4,
            Letter::F => 5,
            Letter::G => 7,
            Letter::A => 9,
            Letter::B => 11,
        })
    }

    /// Steps up (or down, if negative) by `n` letters, wrapping from B to C.
    ///
    /// ```
    /// use mackey_theory::Letter;
    /// assert_eq!(Letter::A.step(2), Letter::C);
    /// assert_eq!(Letter::C.step(-1), Letter::B);
    /// ```
    #[must_use]
    pub fn step(self, n: i8) -> Self {
        Self::from_index(i32::from(self.index()) + i32::from(n))
    }

    /// The letter as text.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Letter::C => "C",
            Letter::D => "D",
            Letter::E => "E",
            Letter::F => "F",
            Letter::G => "G",
            Letter::A => "A",
            Letter::B => "B",
        }
    }

    fn from_char(c: char) -> Option<Self> {
        match c {
            'C' => Some(Letter::C),
            'D' => Some(Letter::D),
            'E' => Some(Letter::E),
            'F' => Some(Letter::F),
            'G' => Some(Letter::G),
            'A' => Some(Letter::A),
            'B' => Some(Letter::B),
            _ => None,
        }
    }
}

/// An accidental from double flat (`-2`) to double sharp (`2`).
///
/// ```
/// use mackey_theory::Accidental;
/// assert_eq!(Accidental::try_from(1).unwrap().symbol(), "#");
/// assert!(Accidental::try_from(3).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, type = "number"))]
pub struct Accidental(i8);

impl Accidental {
    /// No accidental.
    pub const NATURAL: Accidental = Accidental(0);
    /// One sharp.
    pub const SHARP: Accidental = Accidental(1);
    /// One flat.
    pub const FLAT: Accidental = Accidental(-1);
    /// Double sharp.
    pub const DOUBLE_SHARP: Accidental = Accidental(2);
    /// Double flat.
    pub const DOUBLE_FLAT: Accidental = Accidental(-2);

    /// Semitones of offset: `-2..=2`, sharps positive.
    #[must_use]
    pub fn value(self) -> i8 {
        self.0
    }

    /// Text form: `bb`, `b`, empty, `#`, `##`.
    #[must_use]
    pub fn symbol(self) -> &'static str {
        match self.0 {
            -2 => "bb",
            -1 => "b",
            1 => "#",
            2 => "##",
            _ => "",
        }
    }
}

impl TryFrom<i8> for Accidental {
    type Error = TheoryError;

    fn try_from(value: i8) -> Result<Self, Self::Error> {
        if (-2..=2).contains(&value) {
            Ok(Self(value))
        } else {
            Err(TheoryError::AccidentalOutOfRange(value.into()))
        }
    }
}

impl From<Accidental> for i8 {
    fn from(a: Accidental) -> Self {
        a.0
    }
}

checked_serde!(Accidental, i8);

/// Which accidentals to use for a pitch class nothing else spells (G-1.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum AccidentalPreference {
    /// C#, D#, F#, G#, A#
    Sharps,
    /// Db, Eb, Gb, Ab, Bb
    Flats,
}

const SHARP_SPELLING: [(Letter, i8); 12] = [
    (Letter::C, 0),
    (Letter::C, 1),
    (Letter::D, 0),
    (Letter::D, 1),
    (Letter::E, 0),
    (Letter::F, 0),
    (Letter::F, 1),
    (Letter::G, 0),
    (Letter::G, 1),
    (Letter::A, 0),
    (Letter::A, 1),
    (Letter::B, 0),
];

const FLAT_SPELLING: [(Letter, i8); 12] = [
    (Letter::C, 0),
    (Letter::D, -1),
    (Letter::D, 0),
    (Letter::E, -1),
    (Letter::E, 0),
    (Letter::F, 0),
    (Letter::G, -1),
    (Letter::G, 0),
    (Letter::A, -1),
    (Letter::A, 0),
    (Letter::B, -1),
    (Letter::B, 0),
];

/// A spelled note: `F#` and `Gb` are different values with the same pitch class.
///
/// ```
/// use mackey_theory::Note;
/// let fs: Note = "F#".parse().unwrap();
/// let gb: Note = "Gb".parse().unwrap();
/// assert_ne!(fs, gb);
/// assert_eq!(fs.pitch_class(), gb.pitch_class());
/// assert_eq!(fs.name(), "F#");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Note {
    /// The letter name.
    pub letter: Letter,
    /// The accidental, double flat to double sharp.
    pub accidental: Accidental,
}

impl Note {
    /// A natural note.
    #[must_use]
    pub fn natural(letter: Letter) -> Self {
        Self {
            letter,
            accidental: Accidental::NATURAL,
        }
    }

    /// A note from a letter and an accidental count `-2..=2`.
    ///
    /// # Errors
    /// [`TheoryError::AccidentalOutOfRange`] outside `-2..=2`.
    pub fn new(letter: Letter, accidental: i8) -> Result<Self, TheoryError> {
        Ok(Self {
            letter,
            accidental: Accidental::try_from(accidental)?,
        })
    }

    /// The pitch class this note sounds.
    #[must_use]
    pub fn pitch_class(self) -> PitchClass {
        PitchClass::new(i32::from(self.letter.natural_pc().value()) + i32::from(self.accidental.0))
    }

    /// The simple spelling of a pitch class (`C#` or `Db` for 1) per the preference.
    ///
    /// ```
    /// use mackey_theory::{AccidentalPreference, Note, PitchClass};
    /// let pc = PitchClass::new(10);
    /// assert_eq!(Note::from_pitch_class(pc, AccidentalPreference::Flats).name(), "Bb");
    /// assert_eq!(Note::from_pitch_class(pc, AccidentalPreference::Sharps).name(), "A#");
    /// ```
    #[must_use]
    pub fn from_pitch_class(pc: PitchClass, preference: AccidentalPreference) -> Self {
        let table = match preference {
            AccidentalPreference::Sharps => &SHARP_SPELLING,
            AccidentalPreference::Flats => &FLAT_SPELLING,
        };
        let (letter, acc) = table
            .get(usize::from(pc.0))
            .copied()
            .unwrap_or((Letter::C, 0));
        Self {
            letter,
            accidental: Accidental(acc),
        }
    }

    /// Spells a pitch class on a given letter, if that needs at most a double accidental.
    ///
    /// ```
    /// use mackey_theory::{Letter, Note, PitchClass};
    /// let n = Note::from_letter_and_pitch_class(Letter::F, PitchClass::new(7)).unwrap();
    /// assert_eq!(n.name(), "F##");
    /// assert!(Note::from_letter_and_pitch_class(Letter::F, PitchClass::new(10)).is_none());
    /// ```
    #[must_use]
    pub fn from_letter_and_pitch_class(letter: Letter, pc: PitchClass) -> Option<Self> {
        let diff =
            (i32::from(pc.0) - i32::from(letter.natural_pc().value()) + 6).rem_euclid(12) - 6;
        i8::try_from(diff)
            .ok()
            .and_then(|d| Accidental::try_from(d).ok())
            .map(|accidental| Self { letter, accidental })
    }

    /// Text name such as `F#`, `Bb`, `C##`.
    #[must_use]
    pub fn name(self) -> String {
        format!("{}{}", self.letter.as_str(), self.accidental.symbol())
    }

    /// Moves by semitones; the result is spelled by `preference` (no key context here).
    #[must_use]
    pub fn transpose_semitones(self, semitones: i32, preference: AccidentalPreference) -> Self {
        Self::from_pitch_class(self.pitch_class().transpose(semitones), preference)
    }

    /// True for a double sharp or double flat.
    #[must_use]
    pub fn is_double(self) -> bool {
        self.accidental.0.abs() == 2
    }

    /// The enharmonic note with no double accidental (G-1.6): `F##` becomes `G`, `Bbb` becomes
    /// `A`, `E##` becomes `F#`, `Fbb` becomes `Eb`. Single and natural notes are unchanged. A
    /// double sharp that lands on a black key is spelled with a sharp, a double flat with a flat.
    ///
    /// ```
    /// use mackey_theory::Note;
    /// assert_eq!("F##".parse::<Note>().unwrap().simplified().name(), "G");
    /// assert_eq!("Fbb".parse::<Note>().unwrap().simplified().name(), "Eb");
    /// assert_eq!("Bb".parse::<Note>().unwrap().simplified().name(), "Bb");
    /// ```
    #[must_use]
    pub fn simplified(self) -> Self {
        if !self.is_double() {
            return self;
        }
        let preference = if self.accidental.0 > 0 {
            AccidentalPreference::Sharps
        } else {
            AccidentalPreference::Flats
        };
        Self::from_pitch_class(self.pitch_class(), preference)
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.letter.as_str(), self.accidental.symbol())
    }
}

impl FromStr for Note {
    type Err = TheoryError;

    /// Accepts `A`-`G` plus `#`, `b`, `##`, `bb`, `x`, `♯`, `♭`. Rejects `H`, lowercase letters,
    /// the empty string and mixed accidentals such as `Fb#b`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let bad = || TheoryError::InvalidNote(s.to_owned());
        let mut chars = s.chars();
        let letter = chars.next().and_then(Letter::from_char).ok_or_else(bad)?;
        let mut total = 0i8;
        let mut count = 0u8;
        let mut direction = 0i8;
        for c in chars {
            let (step, dir) = match c {
                '#' | '♯' => (1, 1),
                'x' | '𝄪' => (2, 1),
                'b' | '♭' => (-1, -1),
                '𝄫' => (-2, -1),
                _ => return Err(bad()),
            };
            if direction != 0 && direction != dir {
                return Err(bad());
            }
            direction = dir;
            total = total.saturating_add(step);
            count = count.saturating_add(1);
            if count > 2 {
                return Err(bad());
            }
        }
        Note::new(letter, total).map_err(|_| bad())
    }
}

/// A reference pitch for A4 in Hz, 415 to 466 (TU-3).
///
/// ```
/// use mackey_theory::ReferencePitch;
/// assert_eq!(ReferencePitch::default().hz(), 440.0);
/// assert!(ReferencePitch::new(442.0).is_ok());
/// assert!(ReferencePitch::new(500.0).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, type = "number"))]
pub struct ReferencePitch(f64);

impl ReferencePitch {
    /// Validates a reference pitch.
    ///
    /// # Errors
    /// [`TheoryError::ReferencePitchOutOfRange`] outside `415.0..=466.0` or not finite.
    pub fn new(hz: f64) -> Result<Self, TheoryError> {
        if hz.is_finite() && (415.0..=466.0).contains(&hz) {
            Ok(Self(hz))
        } else {
            Err(TheoryError::ReferencePitchOutOfRange(hz))
        }
    }

    /// The frequency of A4 in Hz.
    #[must_use]
    pub fn hz(self) -> f64 {
        self.0
    }
}

impl Default for ReferencePitch {
    fn default() -> Self {
        Self(440.0)
    }
}

impl TryFrom<f64> for ReferencePitch {
    type Error = TheoryError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<ReferencePitch> for f64 {
    fn from(r: ReferencePitch) -> Self {
        r.0
    }
}

checked_serde!(ReferencePitch, f64);

/// A note in an octave (scientific pitch notation): C4 is middle C = MIDI 60, A4 = 69.
///
/// The octave number belongs to the letter, so `B#3` is MIDI 60 and `Cb4` is MIDI 59. The MIDI
/// number is always within `0..=127`.
///
/// ```
/// use mackey_theory::{Note, Pitch};
/// let a4 = Pitch::new("A".parse().unwrap(), 4).unwrap();
/// assert_eq!(a4.midi(), 69);
/// let b_sharp_3 = Pitch::new("B#".parse::<Note>().unwrap(), 3).unwrap();
/// assert_eq!(b_sharp_3.midi(), 60);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Pitch {
    note: Note,
    octave: i8,
}

#[derive(Deserialize)]
struct PitchRepr {
    note: Note,
    octave: i8,
}

impl<'de> Deserialize<'de> for Pitch {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let repr = PitchRepr::deserialize(deserializer)?;
        Pitch::new(repr.note, repr.octave).map_err(serde::de::Error::custom)
    }
}

impl Pitch {
    /// Validates that the pitch lies within MIDI `0..=127`.
    ///
    /// # Errors
    /// [`TheoryError::MidiOutOfRange`] otherwise.
    pub fn new(note: Note, octave: i8) -> Result<Self, TheoryError> {
        let midi = Self::midi_of(note, octave);
        if (0..=127).contains(&midi) {
            Ok(Self { note, octave })
        } else {
            Err(TheoryError::MidiOutOfRange(midi))
        }
    }

    fn midi_of(note: Note, octave: i8) -> i32 {
        12 * (i32::from(octave) + 1)
            + i32::from(note.letter.natural_pc().value())
            + i32::from(note.accidental.0)
    }

    /// The MIDI note (0 to 127) spelled by `preference`.
    ///
    /// # Errors
    /// [`TheoryError::MidiOutOfRange`] outside `0..=127`.
    ///
    /// ```
    /// use mackey_theory::{AccidentalPreference, Pitch};
    /// let p = Pitch::from_midi(58, AccidentalPreference::Flats).unwrap();
    /// assert_eq!(p.name(), "Bb3");
    /// ```
    pub fn from_midi(midi: i32, preference: AccidentalPreference) -> Result<Self, TheoryError> {
        if !(0..=127).contains(&midi) {
            return Err(TheoryError::MidiOutOfRange(midi));
        }
        let note = Note::from_pitch_class(PitchClass::new(midi), preference);
        let octave =
            i8::try_from(midi.div_euclid(12) - 1).map_err(|_| TheoryError::MidiOutOfRange(midi))?;
        Ok(Self { note, octave })
    }

    /// The spelled note.
    #[must_use]
    pub fn note(self) -> Note {
        self.note
    }

    /// The octave number (the octave of the letter).
    #[must_use]
    pub fn octave(self) -> i8 {
        self.octave
    }

    /// MIDI note number: `12 * (octave + 1) + pitch class`, with C-1 = 0.
    #[must_use]
    pub fn midi(self) -> i32 {
        Self::midi_of(self.note, self.octave)
    }

    /// Frequency in Hz in equal temperament against a reference A4.
    ///
    /// ```
    /// use mackey_theory::{AccidentalPreference, Pitch, ReferencePitch};
    /// let c4 = Pitch::from_midi(60, AccidentalPreference::Sharps).unwrap();
    /// assert!((c4.frequency(ReferencePitch::default()) - 261.6256).abs() < 1e-3);
    /// ```
    #[must_use]
    pub fn frequency(self, a4: ReferencePitch) -> f64 {
        a4.hz() * 2f64.powf(f64::from(self.midi() - 69) / 12.0)
    }

    /// Moves by semitones, spelling the result by `preference`.
    ///
    /// # Errors
    /// [`TheoryError::MidiOutOfRange`] when the result leaves `0..=127`.
    pub fn add_semitones(
        self,
        semitones: i32,
        preference: AccidentalPreference,
    ) -> Result<Self, TheoryError> {
        Self::from_midi(self.midi() + semitones, preference)
    }

    /// Name with octave, such as `G3`.
    #[must_use]
    pub fn name(self) -> String {
        format!("{}{}", self.note.name(), self.octave)
    }
}

impl PartialOrd for Pitch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Pitch {
    fn cmp(&self, other: &Self) -> Ordering {
        self.midi()
            .cmp(&other.midi())
            .then(self.note.cmp(&other.note))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(s: &str) -> Note {
        s.parse().unwrap()
    }

    /// (input, expected letter, expected accidental) for accepted names.
    const NOTE_CASES: &[(&str, Letter, i8)] = &[
        ("C", Letter::C, 0),
        ("F#", Letter::F, 1),
        ("Bb", Letter::B, -1),
        ("C##", Letter::C, 2),
        ("Ebb", Letter::E, -2),
        ("Fx", Letter::F, 2),
        ("B♭", Letter::B, -1),
        ("E♯", Letter::E, 1),
        ("B𝄫", Letter::B, -2),
        ("F𝄪", Letter::F, 2),
    ];

    const NOTE_REJECTS: &[&str] = &[
        "H", "", "Fb#b", "c", "bb", "C###", "Cbbb", "F#b", " C", "C ", "Cm", "♯",
    ];

    #[test]
    fn a2_note_names_parse_with_labels() {
        for (input, letter, acc) in NOTE_CASES {
            let n = note(input);
            assert_eq!(
                (n.letter, n.accidental.value()),
                (*letter, *acc),
                "case {input}"
            );
        }
    }

    #[test]
    fn a2_note_names_reject_bad_input() {
        for input in NOTE_REJECTS {
            assert!(
                input.parse::<Note>().is_err(),
                "case {input:?} should be rejected"
            );
        }
    }

    #[test]
    fn a2_note_name_round_trips_through_display() {
        for (input, ..) in NOTE_CASES {
            let n = note(input);
            assert_eq!(note(&n.to_string()), n, "case {input}");
        }
    }

    /// (note, octave, midi)
    const MIDI_CASES: &[(&str, i8, i32)] = &[
        ("A", 4, 69),
        ("C", 4, 60),
        ("B#", 3, 60),
        ("Cb", 4, 59),
        ("C", -1, 0),
        ("G", 9, 127),
        ("Bb", 3, 58),
        ("E", 2, 40),
        ("F#", 3, 54),
        ("C##", 4, 62),
    ];

    #[test]
    fn a2_pitch_midi_table() {
        for (name, octave, midi) in MIDI_CASES {
            let p = Pitch::new(note(name), *octave).unwrap();
            assert_eq!(p.midi(), *midi, "case {name}{octave}");
        }
    }

    #[test]
    fn a2_pitch_range_is_enforced() {
        assert_eq!(
            Pitch::new(note("G#"), 9),
            Err(TheoryError::MidiOutOfRange(128))
        );
        assert_eq!(
            Pitch::new(note("Cb"), -1),
            Err(TheoryError::MidiOutOfRange(-1))
        );
        assert_eq!(
            Pitch::from_midi(128, AccidentalPreference::Sharps),
            Err(TheoryError::MidiOutOfRange(128))
        );
        assert_eq!(
            Pitch::from_midi(-1, AccidentalPreference::Flats),
            Err(TheoryError::MidiOutOfRange(-1))
        );
    }

    #[test]
    fn a2_pitch_from_midi_uses_preference() {
        let sharp = Pitch::from_midi(61, AccidentalPreference::Sharps).unwrap();
        let flat = Pitch::from_midi(61, AccidentalPreference::Flats).unwrap();
        assert_eq!(
            (sharp.name(), flat.name()),
            ("C#4".to_owned(), "Db4".to_owned())
        );
        assert_eq!(sharp.octave(), 4);
        assert_eq!(sharp.note(), note("C#"));
    }

    #[test]
    fn a2_pitch_add_semitones_crosses_octaves() {
        let b3 = Pitch::new(note("B"), 3).unwrap();
        assert_eq!(
            b3.add_semitones(1, AccidentalPreference::Sharps)
                .unwrap()
                .name(),
            "C4"
        );
        assert_eq!(
            b3.add_semitones(-12, AccidentalPreference::Sharps)
                .unwrap()
                .name(),
            "B2"
        );
        assert!(b3.add_semitones(100, AccidentalPreference::Sharps).is_err());
    }

    #[test]
    fn a2_pitch_orders_by_midi_then_note() {
        let b_sharp3 = Pitch::new(note("B#"), 3).unwrap();
        let c4 = Pitch::new(note("C"), 4).unwrap();
        let d4 = Pitch::new(note("D"), 4).unwrap();
        assert!(c4 < d4);
        assert_eq!(b_sharp3.midi(), c4.midi());
        assert_ne!(b_sharp3.cmp(&c4), Ordering::Equal);
        assert_eq!(c4.partial_cmp(&c4), Some(Ordering::Equal));
    }

    /// (midi, a4, expected Hz)
    const FREQ_CASES: &[(i32, f64, f64)] = &[
        (69, 440.0, 440.0),
        (60, 440.0, 261.6256),
        (57, 440.0, 220.0),
        (69, 442.0, 442.0),
        (69, 415.0, 415.0),
        (40, 440.0, 82.4069),
        (81, 440.0, 880.0),
    ];

    #[test]
    fn a2_frequency_table() {
        for (midi, a4, hz) in FREQ_CASES {
            let p = Pitch::from_midi(*midi, AccidentalPreference::Sharps).unwrap();
            let got = p.frequency(ReferencePitch::new(*a4).unwrap());
            assert!(
                (got - hz).abs() < 1e-3,
                "case midi {midi} a4 {a4}: {got} vs {hz}"
            );
        }
    }

    #[test]
    fn a2_reference_pitch_validates() {
        for hz in [415.0, 440.0, 466.0] {
            assert!(ReferencePitch::new(hz).is_ok(), "case {hz}");
        }
        for hz in [414.9, 466.1, f64::NAN, f64::INFINITY, 0.0] {
            assert!(ReferencePitch::new(hz).is_err(), "case {hz}");
        }
        assert_eq!(ReferencePitch::default().hz(), 440.0);
    }

    #[test]
    fn a2_checked_types_validate_on_deserialize() {
        assert_eq!(serde_json::from_str::<PitchClass>("7").unwrap().value(), 7);
        assert!(serde_json::from_str::<PitchClass>("12").is_err());
        assert!(serde_json::from_str::<Accidental>("3").is_err());
        assert_eq!(
            serde_json::from_str::<Accidental>("-2").unwrap(),
            Accidental::DOUBLE_FLAT
        );
        assert!(serde_json::from_str::<ReferencePitch>("500.0").is_err());
        assert!(
            serde_json::from_str::<Pitch>(r#"{"note":{"letter":"G","accidental":1},"octave":9}"#)
                .is_err()
        );
        let p = Pitch::new(note("A"), 4).unwrap();
        let json = serde_json::to_string(&p).unwrap();
        assert_eq!(serde_json::from_str::<Pitch>(&json).unwrap(), p);
        let rp = serde_json::to_string(&ReferencePitch::default()).unwrap();
        assert_eq!(rp, "440.0");
        assert_eq!(serde_json::to_string(&PitchClass::new(3)).unwrap(), "3");
    }

    #[test]
    fn a2_pitch_class_helpers() {
        assert_eq!(PitchClass::new(-13).value(), 11);
        assert_eq!(PitchClass::try_from(11u8).unwrap().value(), 11);
        assert_eq!(
            PitchClass::try_from(12u8),
            Err(TheoryError::PitchClassOutOfRange(12))
        );
        assert_eq!(PitchClass::new(0).semitones_to(PitchClass::new(11)), 11);
        assert_eq!(PitchClass::new(11).semitones_to(PitchClass::new(0)), 1);
        assert_eq!(u8::from(PitchClass::new(4)), 4);
    }

    #[test]
    fn a2_letters_step_and_wrap() {
        for (letter, n, expected) in [
            (Letter::B, 1, Letter::C),
            (Letter::C, -1, Letter::B),
            (Letter::E, 3, Letter::A),
            (Letter::A, 7, Letter::A),
            (Letter::D, -9, Letter::B),
        ] {
            assert_eq!(letter.step(n), expected, "case {letter:?} {n}");
        }
        assert_eq!(Letter::ALL.map(Letter::index), [0, 1, 2, 3, 4, 5, 6]);
    }

    /// (letter, pitch class, expected name) for spelling a pitch class on a letter.
    const LETTER_PC_CASES: &[(Letter, i32, Option<&str>)] = &[
        (Letter::F, 6, Some("F#")),
        (Letter::G, 6, Some("Gb")),
        (Letter::F, 7, Some("F##")),
        (Letter::C, 11, Some("Cb")),
        (Letter::B, 0, Some("B#")),
        (Letter::C, 9, None),
        (Letter::F, 9, None),
        (Letter::C, 10, Some("Cbb")),
    ];

    #[test]
    fn a2_letter_and_pitch_class_spelling() {
        for (letter, pc, expected) in LETTER_PC_CASES {
            let got =
                Note::from_letter_and_pitch_class(*letter, PitchClass::new(*pc)).map(Note::name);
            assert_eq!(got.as_deref(), *expected, "case {letter:?} {pc}");
        }
    }

    /// (note, simplified)
    const SIMPLIFY_CASES: &[(&str, &str)] = &[
        ("F##", "G"),
        ("C##", "D"),
        ("D##", "E"),
        ("E##", "F#"),
        ("G##", "A"),
        ("A##", "B"),
        ("B##", "C#"),
        ("Bbb", "A"),
        ("Dbb", "C"),
        ("Ebb", "D"),
        ("Fbb", "Eb"),
        ("Abb", "G"),
        ("Gbb", "F"),
        ("Cbb", "Bb"),
        ("Bb", "Bb"),
        ("F#", "F#"),
        ("C", "C"),
    ];

    #[test]
    fn g_1_6_double_accidentals_simplify() {
        for (input, expected) in SIMPLIFY_CASES {
            let s = note(input).simplified();
            assert_eq!(s.name(), *expected, "case {input}");
            assert!(!s.is_double(), "case {input}");
            assert_eq!(s.pitch_class(), note(input).pitch_class(), "case {input}");
        }
    }

    #[test]
    fn a2_note_transpose_uses_preference() {
        assert_eq!(
            note("C")
                .transpose_semitones(1, AccidentalPreference::Sharps)
                .name(),
            "C#"
        );
        assert_eq!(
            note("C")
                .transpose_semitones(1, AccidentalPreference::Flats)
                .name(),
            "Db"
        );
        assert_eq!(
            note("A")
                .transpose_semitones(-12, AccidentalPreference::Flats)
                .name(),
            "A"
        );
        assert_eq!(
            Note::new(Letter::C, 3),
            Err(TheoryError::AccidentalOutOfRange(3))
        );
        assert_eq!(Note::natural(Letter::D).name(), "D");
    }

    #[test]
    fn a2_every_pitch_class_has_both_simple_spellings() {
        for value in 0..12 {
            let pc = PitchClass::new(value);
            for pref in [AccidentalPreference::Sharps, AccidentalPreference::Flats] {
                let n = Note::from_pitch_class(pc, pref);
                assert_eq!(n.pitch_class(), pc, "case {value} {pref:?}");
                assert!(!n.is_double(), "case {value} {pref:?}");
            }
        }
    }
}
