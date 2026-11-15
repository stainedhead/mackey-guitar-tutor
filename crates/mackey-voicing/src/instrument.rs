//! Strings, frets, tunings, capo, fret count and handedness (FR-013, LH-1 to LH-3).

use mackey_theory::{AccidentalPreference, Letter, Note, Pitch, PitchClass};
use serde::{Deserialize, Serialize};

use crate::error::VoicingError;

/// Serialises a checked newtype as its primitive and validates on the way in.
macro_rules! checked_serde {
    ($ty:ty, $prim:ty) => {
        impl Serialize for $ty {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                self.0.serialize(serializer)
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

/// A string number `1..=6`; 1 is the highest-pitched string (D-02).
///
/// ```
/// use mackey_voicing::StringNumber;
/// let low_e = StringNumber::new(6).unwrap();
/// assert_eq!(low_e.index(), 0);
/// assert!(StringNumber::new(7).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, type = "number"))]
pub struct StringNumber(u8);

impl StringNumber {
    /// All six strings from the lowest-pitched (6) to the highest (1).
    pub const LOW_TO_HIGH: [StringNumber; 6] = [
        StringNumber(6),
        StringNumber(5),
        StringNumber(4),
        StringNumber(3),
        StringNumber(2),
        StringNumber(1),
    ];

    /// Validates `1..=6`.
    ///
    /// # Errors
    /// [`VoicingError::StringOutOfRange`] outside `1..=6`.
    pub fn new(number: u8) -> Result<Self, VoicingError> {
        if (1..=6).contains(&number) {
            Ok(Self(number))
        } else {
            Err(VoicingError::StringOutOfRange(number))
        }
    }

    /// The string number `1..=6`.
    #[must_use]
    pub fn number(self) -> u8 {
        self.0
    }

    /// The index into low-to-high arrays: string 6 is 0, string 1 is 5.
    #[must_use]
    pub fn index(self) -> usize {
        usize::from(6 - self.0)
    }

    /// The string at a low-to-high array index.
    #[must_use]
    pub fn from_index(index: usize) -> Option<Self> {
        u8::try_from(index)
            .ok()
            .and_then(|i| 6u8.checked_sub(i))
            .and_then(|n| Self::new(n).ok())
    }
}

impl TryFrom<u8> for StringNumber {
    type Error = VoicingError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<StringNumber> for u8 {
    fn from(value: StringNumber) -> Self {
        value.0
    }
}
checked_serde!(StringNumber, u8);

/// A fret `0..=24`, counted from the nut (D-03).
///
/// ```
/// use mackey_voicing::Fret;
/// assert_eq!(Fret::new(5).unwrap().value(), 5);
/// assert!(Fret::new(25).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, type = "number"))]
pub struct Fret(u8);

impl Fret {
    /// The open-string fret.
    pub const OPEN: Fret = Fret(0);

    /// Validates `0..=24`.
    ///
    /// # Errors
    /// [`VoicingError::FretOutOfRange`] above 24.
    pub fn new(fret: u8) -> Result<Self, VoicingError> {
        if fret <= 24 {
            Ok(Self(fret))
        } else {
            Err(VoicingError::FretOutOfRange(fret))
        }
    }

    /// The fret number.
    #[must_use]
    pub fn value(self) -> u8 {
        self.0
    }

    /// One fret up, if it exists.
    #[must_use]
    pub fn up(self) -> Option<Self> {
        Self::new(self.0 + 1).ok()
    }

    /// One fret down, if it exists.
    #[must_use]
    pub fn down(self) -> Option<Self> {
        self.0.checked_sub(1).map(Self)
    }
}

impl TryFrom<u8> for Fret {
    type Error = VoicingError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Fret> for u8 {
    fn from(value: Fret) -> Self {
        value.0
    }
}
checked_serde!(Fret, u8);

/// A capo position `0..=12` (LH-3). Zero means no capo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, type = "number"))]
pub struct Capo(u8);

impl Capo {
    /// Validates `0..=12`.
    ///
    /// # Errors
    /// [`VoicingError::CapoOutOfRange`] above 12.
    pub fn new(fret: u8) -> Result<Self, VoicingError> {
        if fret <= 12 {
            Ok(Self(fret))
        } else {
            Err(VoicingError::CapoOutOfRange(fret))
        }
    }

    /// The capo fret.
    #[must_use]
    pub fn value(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for Capo {
    type Error = VoicingError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Capo> for u8 {
    fn from(value: Capo) -> Self {
        value.0
    }
}
checked_serde!(Capo, u8);

/// The number of frets on the neck, `19..=24`, default 22 (FB-1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, type = "number"))]
pub struct FretCount(u8);

impl FretCount {
    /// Validates `19..=24`.
    ///
    /// # Errors
    /// [`VoicingError::FretCountOutOfRange`] outside `19..=24`.
    pub fn new(count: u8) -> Result<Self, VoicingError> {
        if (19..=24).contains(&count) {
            Ok(Self(count))
        } else {
            Err(VoicingError::FretCountOutOfRange(count))
        }
    }

    /// The fret count.
    #[must_use]
    pub fn value(self) -> u8 {
        self.0
    }
}

impl Default for FretCount {
    fn default() -> Self {
        Self(22)
    }
}

impl TryFrom<u8> for FretCount {
    type Error = VoicingError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<FretCount> for u8 {
    fn from(value: FretCount) -> Self {
        value.0
    }
}
checked_serde!(FretCount, u8);

/// A playback tempo in beats per minute, `30..=250`, default 70 (G-4).
///
/// ```
/// use mackey_voicing::Bpm;
/// assert_eq!(Bpm::default().value(), 70);
/// assert!(Bpm::new(29).is_err());
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, type = "number"))]
pub struct Bpm(u16);

impl Bpm {
    /// Validates `30..=250`.
    ///
    /// # Errors
    /// [`VoicingError::TempoOutOfRange`] outside `30..=250`.
    pub fn new(bpm: u16) -> Result<Self, VoicingError> {
        if (30..=250).contains(&bpm) {
            Ok(Self(bpm))
        } else {
            Err(VoicingError::TempoOutOfRange(bpm))
        }
    }

    /// The tempo.
    #[must_use]
    pub fn value(self) -> u16 {
        self.0
    }
}

impl Default for Bpm {
    fn default() -> Self {
        Self(70)
    }
}

impl TryFrom<u16> for Bpm {
    type Error = VoicingError;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Bpm> for u16 {
    fn from(value: Bpm) -> Self {
        value.0
    }
}
checked_serde!(Bpm, u16);

/// The nine tuning presets of LH-2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum TuningPreset {
    /// E A D G B E
    Standard,
    /// D A D G B E
    DropD,
    /// D A D G B D
    DoubleDropD,
    /// D A D G A D
    Dadgad,
    /// D G D G B D
    OpenG,
    /// D A D F# A D
    OpenD,
    /// E B E G# B E
    OpenE,
    /// Eb Ab Db Gb Bb Eb
    HalfStepDown,
    /// D G C F A D
    WholeStepDown,
}

/// `(letter, accidental, octave)` for each string, low to high.
type PresetSpec = [(Letter, i8, i8); 6];

impl TuningPreset {
    /// Every preset, in the order of LH-2.
    pub const ALL: [TuningPreset; 9] = [
        TuningPreset::Standard,
        TuningPreset::DropD,
        TuningPreset::DoubleDropD,
        TuningPreset::Dadgad,
        TuningPreset::OpenG,
        TuningPreset::OpenD,
        TuningPreset::OpenE,
        TuningPreset::HalfStepDown,
        TuningPreset::WholeStepDown,
    ];

    /// The name shown in the tuning picker.
    #[must_use]
    pub fn display_name(self) -> &'static str {
        match self {
            TuningPreset::Standard => "Standard",
            TuningPreset::DropD => "Drop D",
            TuningPreset::DoubleDropD => "Double drop D",
            TuningPreset::Dadgad => "DADGAD",
            TuningPreset::OpenG => "Open G",
            TuningPreset::OpenD => "Open D",
            TuningPreset::OpenE => "Open E",
            TuningPreset::HalfStepDown => "Half-step down",
            TuningPreset::WholeStepDown => "Whole-step down",
        }
    }

    fn spec(self) -> PresetSpec {
        use Letter::{A, B, C, D, E, F, G};
        match self {
            TuningPreset::Standard => [
                (E, 0, 2),
                (A, 0, 2),
                (D, 0, 3),
                (G, 0, 3),
                (B, 0, 3),
                (E, 0, 4),
            ],
            TuningPreset::DropD => [
                (D, 0, 2),
                (A, 0, 2),
                (D, 0, 3),
                (G, 0, 3),
                (B, 0, 3),
                (E, 0, 4),
            ],
            TuningPreset::DoubleDropD => [
                (D, 0, 2),
                (A, 0, 2),
                (D, 0, 3),
                (G, 0, 3),
                (B, 0, 3),
                (D, 0, 4),
            ],
            TuningPreset::Dadgad => [
                (D, 0, 2),
                (A, 0, 2),
                (D, 0, 3),
                (G, 0, 3),
                (A, 0, 3),
                (D, 0, 4),
            ],
            TuningPreset::OpenG => [
                (D, 0, 2),
                (G, 0, 2),
                (D, 0, 3),
                (G, 0, 3),
                (B, 0, 3),
                (D, 0, 4),
            ],
            TuningPreset::OpenD => [
                (D, 0, 2),
                (A, 0, 2),
                (D, 0, 3),
                (F, 1, 3),
                (A, 0, 3),
                (D, 0, 4),
            ],
            TuningPreset::OpenE => [
                (E, 0, 2),
                (B, 0, 2),
                (E, 0, 3),
                (G, 1, 3),
                (B, 0, 3),
                (E, 0, 4),
            ],
            TuningPreset::HalfStepDown => [
                (E, -1, 2),
                (A, -1, 2),
                (D, -1, 3),
                (G, -1, 3),
                (B, -1, 3),
                (E, -1, 4),
            ],
            TuningPreset::WholeStepDown => [
                (D, 0, 2),
                (G, 0, 2),
                (C, 0, 3),
                (F, 0, 3),
                (A, 0, 3),
                (D, 0, 4),
            ],
        }
    }

    /// The six open-string pitches, low to high (Open D is D2 A2 D3 F#3 A3 D4).
    #[must_use]
    pub fn strings(self) -> [Pitch; 6] {
        self.spec().map(|(letter, accidental, octave)| {
            match Note::new(letter, accidental).and_then(|n| Pitch::new(n, octave)) {
                Ok(pitch) => pitch,
                // The tables above hold constant, valid notes (checked by the `tunings` tests).
                Err(_) => unreachable!("preset pitches are constants within range"),
            }
        })
    }
}

/// MIDI numbers of standard tuning E2 A2 D3 G3 B3 E4, low to high.
pub(crate) const STANDARD_MIDI: [i32; 6] = [40, 45, 50, 55, 59, 64];

/// A string tuned an unusual distance from the standard pitch (LH-2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum TuningWarning {
    /// Outside "a minor third below to a whole tone above" the standard pitch.
    UnusualTension {
        /// The string.
        string: StringNumber,
        /// Its standard pitch.
        standard: Pitch,
        /// The pitch chosen.
        chosen: Pitch,
    },
}

impl TuningWarning {
    /// The text the tuning panel shows.
    #[must_use]
    pub fn message(&self) -> &'static str {
        "Unusual tension, check your strings"
    }
}

/// Six open-string pitches, low to high, plus a name.
///
/// ```
/// use mackey_voicing::{Tuning, TuningPreset};
/// let open_d = Tuning::preset(TuningPreset::OpenD);
/// let names: Vec<String> = open_d.strings.iter().map(|p| p.name()).collect();
/// assert_eq!(names, ["D2", "A2", "D3", "F#3", "A3", "D4"]);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Tuning {
    /// Display name ("Standard", "Custom").
    pub name: String,
    /// Open-string pitches, string 6 first.
    pub strings: [Pitch; 6],
}

impl Tuning {
    /// A preset tuning.
    #[must_use]
    pub fn preset(preset: TuningPreset) -> Self {
        Self {
            name: preset.display_name().to_owned(),
            strings: preset.strings(),
        }
    }

    /// A custom tuning. Each pitch must lie in C1..=C6; strings far from standard give
    /// warnings but are allowed, and duplicate open strings are allowed (E-FB-6).
    ///
    /// # Errors
    /// [`VoicingError::TuningOutOfRange`] when a pitch is outside MIDI 24..=84.
    ///
    /// ```
    /// use mackey_voicing::{Tuning, TuningPreset};
    /// let mut strings = TuningPreset::Standard.strings();
    /// strings[0] = TuningPreset::OpenD.strings()[3]; // F#3 on the low string: far from E2
    /// let (_, warnings) = Tuning::custom(strings).unwrap();
    /// assert_eq!(warnings.len(), 1);
    /// ```
    pub fn custom(strings: [Pitch; 6]) -> Result<(Self, Vec<TuningWarning>), VoicingError> {
        let mut warnings = Vec::new();
        let standard = TuningPreset::Standard.strings();
        for (i, (chosen, std)) in strings.iter().zip(standard.iter()).enumerate() {
            let number = StringNumber::from_index(i).ok_or(VoicingError::StringOutOfRange(0))?;
            let midi = chosen.midi();
            if !(24..=84).contains(&midi) {
                return Err(VoicingError::TuningOutOfRange {
                    string: number.number(),
                    midi,
                });
            }
            let diff = midi - std.midi();
            if !(-3..=2).contains(&diff) {
                warnings.push(TuningWarning::UnusualTension {
                    string: number,
                    standard: *std,
                    chosen: *chosen,
                });
            }
        }
        Ok((
            Self {
                name: "Custom".to_owned(),
                strings,
            },
            warnings,
        ))
    }

    /// The open pitch of a string (no capo).
    #[must_use]
    pub fn open_pitch(&self, string: StringNumber) -> Pitch {
        self.strings
            .get(string.index())
            .copied()
            .unwrap_or_else(|| TuningPreset::Standard.strings()[0])
    }

    /// MIDI numbers of the open strings, low to high.
    #[must_use]
    pub fn midis(&self) -> [i32; 6] {
        self.strings.map(Pitch::midi)
    }

    /// The uniform offset from standard tuning in semitones (Half-step down is -1), or `None`
    /// when the strings do not all move together.
    #[must_use]
    pub fn shift_from_standard(&self) -> Option<i8> {
        let shifts: Vec<i32> = self
            .midis()
            .iter()
            .zip(STANDARD_MIDI.iter())
            .map(|(m, s)| m - s)
            .collect();
        let first = *shifts.first()?;
        if shifts.iter().all(|s| *s == first) {
            i8::try_from(first).ok()
        } else {
            None
        }
    }

    /// True for standard tuning.
    #[must_use]
    pub fn is_standard(&self) -> bool {
        self.shift_from_standard() == Some(0)
    }

    /// True when the pitches equal the given preset's.
    #[must_use]
    pub fn matches(&self, preset: TuningPreset) -> bool {
        self.midis() == preset.strings().map(Pitch::midi)
    }
}

impl Default for Tuning {
    fn default() -> Self {
        Self::preset(TuningPreset::Standard)
    }
}

/// The instrument: tuning, fret count and capo.
///
/// ```
/// use mackey_voicing::{Fret, Instrument, StringNumber};
/// let guitar = Instrument::default();
/// let g3 = guitar.pitch_at(StringNumber::new(3).unwrap(), Fret::new(0).unwrap()).unwrap();
/// assert_eq!(g3.name(), "G3");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Instrument {
    /// The tuning.
    pub tuning: Tuning,
    /// The number of frets.
    pub frets: FretCount,
    /// The capo.
    pub capo: Capo,
}

impl Instrument {
    /// An instrument with the given tuning, default fret count and no capo.
    #[must_use]
    pub fn with_tuning(tuning: Tuning) -> Self {
        Self {
            tuning,
            ..Self::default()
        }
    }

    /// The lowest playable fret: the capo.
    #[must_use]
    pub fn lowest_fret(&self) -> Fret {
        Fret(self.capo.value())
    }

    /// The highest fret.
    #[must_use]
    pub fn highest_fret(&self) -> Fret {
        Fret(self.frets.value())
    }

    /// MIDI of an open string as it sounds with the capo on (index low to high).
    #[must_use]
    pub fn sounding_open_midis(&self) -> [i32; 6] {
        let capo = i32::from(self.capo.value());
        self.tuning.midis().map(|m| m + capo)
    }

    /// MIDI at a string index (low to high) and absolute fret, with no range checks.
    #[must_use]
    pub(crate) fn midi_at_index(&self, index: usize, fret: u8) -> i32 {
        let open = self.tuning.midis().get(index).copied().unwrap_or(40);
        open + i32::from(fret)
    }

    /// MIDI at a string and absolute fret.
    #[must_use]
    pub fn midi_at(&self, string: StringNumber, fret: Fret) -> i32 {
        self.midi_at_index(string.index(), fret.value())
    }

    fn preference_for(&self, string: StringNumber) -> AccidentalPreference {
        if self.tuning.open_pitch(string).note().accidental.value() < 0 {
            AccidentalPreference::Flats
        } else {
            AccidentalPreference::Sharps
        }
    }

    /// The pitch at a string and absolute fret.
    ///
    /// # Errors
    /// [`VoicingError::BelowCapo`] for a fret below the capo and
    /// [`VoicingError::FretOutOfRange`] above the fret count.
    pub fn pitch_at(&self, string: StringNumber, fret: Fret) -> Result<Pitch, VoicingError> {
        if fret.value() < self.capo.value() {
            return Err(VoicingError::BelowCapo {
                fret,
                capo: self.capo,
            });
        }
        if fret.value() > self.frets.value() {
            return Err(VoicingError::FretOutOfRange(fret.value()));
        }
        Ok(Pitch::from_midi(
            self.midi_at(string, fret),
            self.preference_for(string),
        )?)
    }

    /// The open string as it sounds with the capo on (FB-1.3, AC-TU-5.1).
    ///
    /// ```
    /// use mackey_voicing::{Capo, Instrument, StringNumber};
    /// let guitar = Instrument { capo: Capo::new(2).unwrap(), ..Instrument::default() };
    /// let names: Vec<String> = StringNumber::LOW_TO_HIGH
    ///     .iter()
    ///     .map(|s| guitar.open_sounding_pitch(*s).name())
    ///     .collect();
    /// assert_eq!(names, ["F#2", "B2", "E3", "A3", "C#4", "F#4"]);
    /// ```
    #[must_use]
    pub fn open_sounding_pitch(&self, string: StringNumber) -> Pitch {
        let midi = self.midi_at(string, Fret(self.capo.value()));
        Pitch::from_midi(midi, self.preference_for(string))
            .unwrap_or_else(|_| self.tuning.open_pitch(string))
    }

    /// The pitch class at a string index (low to high) and fret.
    pub(crate) fn pc_at_index(&self, index: usize, fret: u8) -> PitchClass {
        PitchClass::new(self.midi_at_index(index, fret))
    }
}

/// Left-handed and flipped-string display options (LH-1); independent and saved.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Handedness {
    /// Mirror the neck horizontally (nut on the right).
    pub left_handed: bool,
    /// Put string 6 at the top.
    pub flip_strings: bool,
}

/// Three adjacent strings for triads (FB-6.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum StringSet {
    /// Strings 1, 2, 3.
    S123,
    /// Strings 2, 3, 4.
    S234,
    /// Strings 3, 4, 5.
    S345,
    /// Strings 4, 5, 6.
    S456,
}

impl StringSet {
    /// The four sets from the highest strings down.
    pub const ALL: [StringSet; 4] = [
        StringSet::S123,
        StringSet::S234,
        StringSet::S345,
        StringSet::S456,
    ];

    /// The set's strings, highest-pitched first (123 is strings 1, 2, 3).
    #[must_use]
    pub fn strings(self) -> [StringNumber; 3] {
        let top = match self {
            StringSet::S123 => 1,
            StringSet::S234 => 2,
            StringSet::S345 => 3,
            StringSet::S456 => 4,
        };
        [
            StringNumber(top),
            StringNumber(top + 1),
            StringNumber(top + 2),
        ]
    }

    /// The label "123", "234", ...
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            StringSet::S123 => "123",
            StringSet::S234 => "234",
            StringSet::S345 => "345",
            StringSet::S456 => "456",
        }
    }

    /// The next set, wrapping 456 to 123.
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            StringSet::S123 => StringSet::S234,
            StringSet::S234 => StringSet::S345,
            StringSet::S345 => StringSet::S456,
            StringSet::S456 => StringSet::S123,
        }
    }

    /// The previous set, wrapping 123 to 456.
    #[must_use]
    pub fn prev(self) -> Self {
        match self {
            StringSet::S123 => StringSet::S456,
            StringSet::S234 => StringSet::S123,
            StringSet::S345 => StringSet::S234,
            StringSet::S456 => StringSet::S345,
        }
    }
}

/// The chord tone in the bass of a triad or voicing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum Inversion {
    /// Root in the bass.
    Root,
    /// Third in the bass.
    First,
    /// Fifth in the bass.
    Second,
    /// Seventh in the bass (seventh chords only).
    Third,
}

impl Inversion {
    /// The label "root", "1st", "2nd", "3rd".
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Inversion::Root => "root",
            Inversion::First => "1st",
            Inversion::Second => "2nd",
            Inversion::Third => "3rd",
        }
    }

    /// The spoken form: "root position", "first inversion", ...
    #[must_use]
    pub fn spoken(self) -> &'static str {
        match self {
            Inversion::Root => "root position",
            Inversion::First => "first inversion",
            Inversion::Second => "second inversion",
            Inversion::Third => "third inversion",
        }
    }

    /// 0 for root up to 3.
    #[must_use]
    pub fn index(self) -> usize {
        match self {
            Inversion::Root => 0,
            Inversion::First => 1,
            Inversion::Second => 2,
            Inversion::Third => 3,
        }
    }

    /// The inversion at an index, if any.
    #[must_use]
    pub fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(Inversion::Root),
            1 => Some(Inversion::First),
            2 => Some(Inversion::Second),
            3 => Some(Inversion::Third),
            _ => None,
        }
    }

    /// The next inversion, wrapping; `Third` only exists for seventh chords.
    #[must_use]
    pub fn next(self, has_seventh: bool) -> Self {
        let count = if has_seventh { 4 } else { 3 };
        Self::from_index((self.index() + 1) % count).unwrap_or(Inversion::Root)
    }

    /// The previous inversion, wrapping.
    #[must_use]
    pub fn prev(self, has_seventh: bool) -> Self {
        let count = if has_seventh { 4 } else { 3 };
        Self::from_index((self.index() + count - 1) % count).unwrap_or(Inversion::Root)
    }

    /// Clamps `Third` to `Second` when the chord has no seventh (or the view is triads).
    #[must_use]
    pub fn clamped(self, allow_third: bool) -> Self {
        if self == Inversion::Third && !allow_third {
            Inversion::Second
        } else {
            self
        }
    }
}
