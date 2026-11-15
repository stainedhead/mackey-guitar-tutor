//! Chord qualities, chords and pitch-class sets.

use serde::{Deserialize, Serialize};

use crate::interval::Interval;
use crate::pitch::{AccidentalPreference, Note, PitchClass};

/// The job a chord tone does, which decides how the fretboard draws it and which tones a
/// voicing may not drop (G-2, FB-5.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum ToneRole {
    /// The chord root.
    Root,
    /// The third (major or minor).
    Third,
    /// The fifth (perfect, diminished or augmented).
    Fifth,
    /// The seventh.
    Seventh,
    /// The suspended note, 6th, 9th, 11th or 13th.
    Extension,
}

impl ToneRole {
    /// The role of a chord-formula interval.
    #[must_use]
    pub fn of_interval(interval: Interval) -> Self {
        match interval.number() {
            1 => ToneRole::Root,
            3 => ToneRole::Third,
            5 => ToneRole::Fifth,
            7 => ToneRole::Seventh,
            _ => ToneRole::Extension,
        }
    }
}

/// The 21 chord qualities of the chord-symbol grammar (FB-4.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum ChordQuality {
    /// Major triad (`C`).
    Major,
    /// Minor triad (`Cm`).
    Minor,
    /// Dominant seventh (`C7`).
    Dominant7,
    /// Major seventh (`Cmaj7`).
    Major7,
    /// Minor seventh (`Cm7`).
    Minor7,
    /// Half-diminished seventh (`Cm7b5`).
    HalfDiminished7,
    /// Diminished triad (`Cdim`).
    Diminished,
    /// Diminished seventh (`Cdim7`).
    Diminished7,
    /// Augmented triad (`Caug`).
    Augmented,
    /// Suspended second (`Csus2`).
    Sus2,
    /// Suspended fourth (`Csus4`).
    Sus4,
    /// Dominant seventh suspended fourth (`C7sus4`).
    Dominant7Sus4,
    /// Major sixth (`C6`).
    Sixth,
    /// Minor sixth (`Cm6`).
    MinorSixth,
    /// Dominant ninth (`C9`).
    Dominant9,
    /// Major ninth (`Cmaj9`).
    Major9,
    /// Minor ninth (`Cm9`).
    Minor9,
    /// Added ninth (`Cadd9`).
    Add9,
    /// Dominant eleventh (`C11`).
    Dominant11,
    /// Dominant thirteenth (`C13`).
    Dominant13,
    /// Power chord (`C5`).
    Power,
}

use Interval as I;

impl ChordQuality {
    /// All qualities, in a stable order.
    pub const ALL: [ChordQuality; 21] = [
        ChordQuality::Major,
        ChordQuality::Minor,
        ChordQuality::Dominant7,
        ChordQuality::Major7,
        ChordQuality::Minor7,
        ChordQuality::HalfDiminished7,
        ChordQuality::Diminished,
        ChordQuality::Diminished7,
        ChordQuality::Augmented,
        ChordQuality::Sus2,
        ChordQuality::Sus4,
        ChordQuality::Dominant7Sus4,
        ChordQuality::Sixth,
        ChordQuality::MinorSixth,
        ChordQuality::Dominant9,
        ChordQuality::Major9,
        ChordQuality::Minor9,
        ChordQuality::Add9,
        ChordQuality::Dominant11,
        ChordQuality::Dominant13,
        ChordQuality::Power,
    ];

    /// The chord formula above the root. `Dominant13` is P1 M3 P5 m7 M9 M13 (the 11th is
    /// omitted by convention); `Dominant11` keeps the third: P1 M3 P5 m7 M9 P11.
    ///
    /// ```
    /// use mackey_theory::ChordQuality;
    /// assert_eq!(ChordQuality::Dominant7.intervals().len(), 4);
    /// ```
    #[must_use]
    pub fn intervals(self) -> &'static [Interval] {
        match self {
            ChordQuality::Major => &[I::P1, I::MAJ3, I::P5],
            ChordQuality::Minor => &[I::P1, I::MIN3, I::P5],
            ChordQuality::Dominant7 => &[I::P1, I::MAJ3, I::P5, I::MIN7],
            ChordQuality::Major7 => &[I::P1, I::MAJ3, I::P5, I::MAJ7],
            ChordQuality::Minor7 => &[I::P1, I::MIN3, I::P5, I::MIN7],
            ChordQuality::HalfDiminished7 => &[I::P1, I::MIN3, I::DIM5, I::MIN7],
            ChordQuality::Diminished => &[I::P1, I::MIN3, I::DIM5],
            ChordQuality::Diminished7 => &[I::P1, I::MIN3, I::DIM5, I::DIM7],
            ChordQuality::Augmented => &[I::P1, I::MAJ3, I::AUG5],
            ChordQuality::Sus2 => &[I::P1, I::MAJ2, I::P5],
            ChordQuality::Sus4 => &[I::P1, I::P4, I::P5],
            ChordQuality::Dominant7Sus4 => &[I::P1, I::P4, I::P5, I::MIN7],
            ChordQuality::Sixth => &[I::P1, I::MAJ3, I::P5, I::MAJ6],
            ChordQuality::MinorSixth => &[I::P1, I::MIN3, I::P5, I::MAJ6],
            ChordQuality::Dominant9 => &[I::P1, I::MAJ3, I::P5, I::MIN7, I::MAJ9],
            ChordQuality::Major9 => &[I::P1, I::MAJ3, I::P5, I::MAJ7, I::MAJ9],
            ChordQuality::Minor9 => &[I::P1, I::MIN3, I::P5, I::MIN7, I::MAJ9],
            ChordQuality::Add9 => &[I::P1, I::MAJ3, I::P5, I::MAJ9],
            ChordQuality::Dominant11 => &[I::P1, I::MAJ3, I::P5, I::MIN7, I::MAJ9, I::P11],
            ChordQuality::Dominant13 => &[I::P1, I::MAJ3, I::P5, I::MIN7, I::MAJ9, I::MAJ13],
            ChordQuality::Power => &[I::P1, I::P5],
        }
    }

    /// The canonical symbol suffix: "", "m", "7", "maj7", "m7", "m7b5", "dim", "dim7", "aug",
    /// "sus2", "sus4", "7sus4", "6", "m6", "9", "maj9", "m9", "add9", "11", "13", "5".
    #[must_use]
    pub fn symbol(self) -> &'static str {
        match self {
            ChordQuality::Major => "",
            ChordQuality::Minor => "m",
            ChordQuality::Dominant7 => "7",
            ChordQuality::Major7 => "maj7",
            ChordQuality::Minor7 => "m7",
            ChordQuality::HalfDiminished7 => "m7b5",
            ChordQuality::Diminished => "dim",
            ChordQuality::Diminished7 => "dim7",
            ChordQuality::Augmented => "aug",
            ChordQuality::Sus2 => "sus2",
            ChordQuality::Sus4 => "sus4",
            ChordQuality::Dominant7Sus4 => "7sus4",
            ChordQuality::Sixth => "6",
            ChordQuality::MinorSixth => "m6",
            ChordQuality::Dominant9 => "9",
            ChordQuality::Major9 => "maj9",
            ChordQuality::Minor9 => "m9",
            ChordQuality::Add9 => "add9",
            ChordQuality::Dominant11 => "11",
            ChordQuality::Dominant13 => "13",
            ChordQuality::Power => "5",
        }
    }

    /// The role of each tone, aligned with [`intervals`](Self::intervals).
    #[must_use]
    pub fn roles(self) -> Vec<ToneRole> {
        self.intervals()
            .iter()
            .copied()
            .map(ToneRole::of_interval)
            .collect()
    }

    /// Roles a voicing must contain (FB-5.4): root and third; root and the sus note for
    /// suspended chords; root and fifth for a power chord.
    #[must_use]
    pub fn required_tones(self) -> &'static [ToneRole] {
        match self {
            ChordQuality::Sus2 | ChordQuality::Sus4 | ChordQuality::Dominant7Sus4 => {
                &[ToneRole::Root, ToneRole::Extension]
            }
            ChordQuality::Power => &[ToneRole::Root, ToneRole::Fifth],
            _ => &[ToneRole::Root, ToneRole::Third],
        }
    }

    /// True if the chord contains a seventh.
    #[must_use]
    pub fn has_seventh(self) -> bool {
        self.intervals().iter().any(|i| i.number() == 7)
    }
}

/// A set of pitch classes (a 12-bit bitset).
///
/// ```
/// use mackey_theory::{PitchClass, PitchClassSet};
/// let mut set = PitchClassSet::default();
/// set.insert(PitchClass::new(4));
/// assert!(set.contains(PitchClass::new(16)));
/// assert_eq!(set.len(), 1);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct PitchClassSet(u16);

impl PitchClassSet {
    /// Adds a pitch class.
    pub fn insert(&mut self, pc: PitchClass) {
        self.0 |= 1 << pc.value();
    }

    /// Membership test.
    #[must_use]
    pub fn contains(self, pc: PitchClass) -> bool {
        self.0 & (1 << pc.value()) != 0
    }

    /// Number of pitch classes.
    #[must_use]
    pub fn len(self) -> usize {
        usize::try_from(self.0.count_ones()).unwrap_or(0)
    }

    /// True when empty.
    #[must_use]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The members in ascending pitch-class order.
    pub fn iter(self) -> impl Iterator<Item = PitchClass> {
        (0..12)
            .map(PitchClass::new)
            .filter(move |pc| self.contains(*pc))
    }
}

impl FromIterator<PitchClass> for PitchClassSet {
    fn from_iter<T: IntoIterator<Item = PitchClass>>(iter: T) -> Self {
        let mut set = Self::default();
        for pc in iter {
            set.insert(pc);
        }
        set
    }
}

/// One tone of a chord: its spelling, the interval from the root and its role.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct ChordTone {
    /// The spelled note.
    pub note: Note,
    /// The interval above the chord root.
    pub interval: Interval,
    /// Root, third, fifth, seventh or extension.
    pub role: ToneRole,
}

/// A chord: root, quality and an optional slash bass. The root keeps the spelling the player
/// gave (`A#m` stays `A#m`, G-1.7).
///
/// ```
/// use mackey_theory::{Chord, ChordQuality};
/// let d7 = Chord::new("D".parse().unwrap(), ChordQuality::Dominant7);
/// let names: Vec<String> = d7.tones().iter().map(|t| t.note.name()).collect();
/// assert_eq!(names, ["D", "F#", "A", "C"]);
/// assert_eq!(d7.symbol(), "D7");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Chord {
    /// The root, as spelled.
    pub root: Note,
    /// The quality.
    pub quality: ChordQuality,
    /// The slash bass (`D/F#`), if any.
    pub bass: Option<Note>,
}

impl Chord {
    /// A chord with no slash bass.
    #[must_use]
    pub fn new(root: Note, quality: ChordQuality) -> Self {
        Self {
            root,
            quality,
            bass: None,
        }
    }

    /// The same chord over a bass note.
    #[must_use]
    pub fn with_bass(self, bass: Note) -> Self {
        Self {
            bass: Some(bass),
            ..self
        }
    }

    /// The chord tones spelled by interval from the root (G-1.5): the third of D7 is F#, the
    /// seventh of Bbmaj7 is A. A root with a double accidental that would need a triple one
    /// falls back to a simple spelling of that tone.
    #[must_use]
    pub fn tones(&self) -> Vec<ChordTone> {
        let preference = if self.root.accidental.value() < 0 {
            AccidentalPreference::Flats
        } else {
            AccidentalPreference::Sharps
        };
        self.quality
            .intervals()
            .iter()
            .map(|&interval| ChordTone {
                note: interval.apply(self.root).unwrap_or_else(|_| {
                    Note::from_pitch_class(interval.above(self.root.pitch_class()), preference)
                }),
                interval,
                role: ToneRole::of_interval(interval),
            })
            .collect()
    }

    /// The canonical symbol: "Am7", "D/F#", "Bbmaj7".
    #[must_use]
    pub fn symbol(&self) -> String {
        match self.bass {
            Some(bass) => format!("{}{}/{bass}", self.root, self.quality.symbol()),
            None => format!("{}{}", self.root, self.quality.symbol()),
        }
    }

    /// The lowest note: the slash bass, or the root.
    #[must_use]
    pub fn bass_or_root(&self) -> Note {
        self.bass.unwrap_or(self.root)
    }

    /// The pitch classes of the chord tones and the slash bass.
    #[must_use]
    pub fn pitch_classes(&self) -> PitchClassSet {
        let mut set: PitchClassSet = self
            .quality
            .intervals()
            .iter()
            .map(|i| i.above(self.root.pitch_class()))
            .collect();
        if let Some(bass) = self.bass {
            set.insert(bass.pitch_class());
        }
        set
    }

    /// Moves root and bass by semitones; new spellings follow `preference`.
    #[must_use]
    pub fn transpose(&self, semitones: i32, preference: AccidentalPreference) -> Self {
        Self {
            root: self.root.transpose_semitones(semitones, preference),
            quality: self.quality,
            bass: self
                .bass
                .map(|b| b.transpose_semitones(semitones, preference)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn note(s: &str) -> Note {
        s.parse().unwrap()
    }

    /// (quality, symbol, formula as degree labels, has seventh)
    const CASES: &[(ChordQuality, &str, &str, bool)] = &[
        (ChordQuality::Major, "", "R 3 5", false),
        (ChordQuality::Minor, "m", "R b3 5", false),
        (ChordQuality::Dominant7, "7", "R 3 5 b7", true),
        (ChordQuality::Major7, "maj7", "R 3 5 7", true),
        (ChordQuality::Minor7, "m7", "R b3 5 b7", true),
        (ChordQuality::HalfDiminished7, "m7b5", "R b3 b5 b7", true),
        (ChordQuality::Diminished, "dim", "R b3 b5", false),
        (ChordQuality::Diminished7, "dim7", "R b3 b5 bb7", true),
        (ChordQuality::Augmented, "aug", "R 3 #5", false),
        (ChordQuality::Sus2, "sus2", "R 2 5", false),
        (ChordQuality::Sus4, "sus4", "R 4 5", false),
        (ChordQuality::Dominant7Sus4, "7sus4", "R 4 5 b7", true),
        (ChordQuality::Sixth, "6", "R 3 5 6", false),
        (ChordQuality::MinorSixth, "m6", "R b3 5 6", false),
        (ChordQuality::Dominant9, "9", "R 3 5 b7 9", true),
        (ChordQuality::Major9, "maj9", "R 3 5 7 9", true),
        (ChordQuality::Minor9, "m9", "R b3 5 b7 9", true),
        (ChordQuality::Add9, "add9", "R 3 5 9", false),
        (ChordQuality::Dominant11, "11", "R 3 5 b7 9 11", true),
        (ChordQuality::Dominant13, "13", "R 3 5 b7 9 13", true),
        (ChordQuality::Power, "5", "R 5", false),
    ];

    #[test]
    fn a6_formulas_symbols_and_sevenths() {
        assert_eq!(CASES.len(), ChordQuality::ALL.len());
        for (quality, symbol, formula, seventh) in CASES {
            let labels: Vec<String> = quality
                .intervals()
                .iter()
                .map(|i| i.degree_label())
                .collect();
            assert_eq!(labels.join(" "), *formula, "case {symbol:?} formula");
            assert_eq!(quality.symbol(), *symbol, "case {symbol:?} symbol");
            assert_eq!(quality.has_seventh(), *seventh, "case {symbol:?} seventh");
            assert_eq!(
                quality.roles().len(),
                quality.intervals().len(),
                "case {symbol:?}"
            );
        }
    }

    #[test]
    fn a6_every_quality_starts_on_the_root_and_has_distinct_tones() {
        for quality in ChordQuality::ALL {
            let intervals = quality.intervals();
            assert_eq!(
                intervals.first().copied(),
                Some(Interval::P1),
                "case {quality:?}"
            );
            let mut semis: Vec<u8> = intervals.iter().map(|i| i.semitones() % 12).collect();
            semis.sort_unstable();
            semis.dedup();
            assert_eq!(
                semis.len(),
                intervals.len(),
                "case {quality:?}: duplicate pitch classes"
            );
        }
    }

    #[test]
    fn a6_roles_and_required_tones() {
        use ToneRole::{Extension, Fifth, Root, Seventh, Third};
        assert_eq!(
            ChordQuality::Dominant7.roles(),
            [Root, Third, Fifth, Seventh]
        );
        assert_eq!(ChordQuality::Sus4.roles(), [Root, Extension, Fifth]);
        assert_eq!(
            ChordQuality::Dominant13.roles(),
            [Root, Third, Fifth, Seventh, Extension, Extension]
        );
        assert_eq!(ChordQuality::Major.required_tones(), [Root, Third]);
        assert_eq!(ChordQuality::Dominant9.required_tones(), [Root, Third]);
        assert_eq!(ChordQuality::Sus2.required_tones(), [Root, Extension]);
        assert_eq!(
            ChordQuality::Dominant7Sus4.required_tones(),
            [Root, Extension]
        );
        assert_eq!(ChordQuality::Power.required_tones(), [Root, Fifth]);
    }

    #[test]
    fn a6_tones_carry_note_interval_and_role() {
        let tones = Chord::new(note("D"), ChordQuality::Dominant7).tones();
        let rows: Vec<(String, String, ToneRole)> = tones
            .iter()
            .map(|t| (t.note.name(), t.interval.to_string(), t.role))
            .collect();
        assert_eq!(
            rows,
            [
                ("D".to_owned(), "P1".to_owned(), ToneRole::Root),
                ("F#".to_owned(), "M3".to_owned(), ToneRole::Third),
                ("A".to_owned(), "P5".to_owned(), ToneRole::Fifth),
                ("C".to_owned(), "m7".to_owned(), ToneRole::Seventh),
            ]
        );
    }

    #[test]
    fn a6_slash_bass_and_symbol() {
        let d_over_fs = Chord::new(note("D"), ChordQuality::Major).with_bass(note("F#"));
        assert_eq!(d_over_fs.symbol(), "D/F#");
        assert_eq!(d_over_fs.bass_or_root(), note("F#"));
        assert_eq!(Chord::new(note("A"), ChordQuality::Minor7).symbol(), "Am7");
        assert_eq!(
            Chord::new(note("A"), ChordQuality::Minor7).bass_or_root(),
            note("A")
        );
    }

    #[test]
    fn a6_pitch_classes_include_the_bass() {
        let c = Chord::new(note("C"), ChordQuality::Major);
        let pcs: Vec<u8> = c.pitch_classes().iter().map(PitchClass::value).collect();
        assert_eq!(pcs, [0, 4, 7]);
        let c_over_bb = c.with_bass(note("Bb"));
        let pcs: Vec<u8> = c_over_bb
            .pitch_classes()
            .iter()
            .map(PitchClass::value)
            .collect();
        assert_eq!(pcs, [0, 4, 7, 10]);
    }

    #[test]
    fn a6_transpose_moves_root_and_bass() {
        let c = Chord::new(note("C"), ChordQuality::Minor).with_bass(note("Eb"));
        let up = c.transpose(2, AccidentalPreference::Flats);
        assert_eq!(up.symbol(), "Dm/F");
        let flats = c.transpose(1, AccidentalPreference::Flats);
        assert_eq!(flats.symbol(), "Dbm/E");
        let sharps = c.transpose(1, AccidentalPreference::Sharps);
        assert_eq!(sharps.symbol(), "C#m/E");
    }

    #[test]
    fn a6_a_root_that_needs_a_triple_accidental_falls_back_to_a_simple_spelling() {
        let weird = Chord::new(note("F##"), ChordQuality::Augmented);
        let names: Vec<String> = weird.tones().iter().map(|t| t.note.name()).collect();
        assert_eq!(names.len(), 3);
        assert_eq!(weird.tones()[2].note.pitch_class().value(), 3);
    }

    #[test]
    fn a6_pitch_class_set_basics() {
        let mut set = PitchClassSet::default();
        assert!(set.is_empty());
        set.insert(PitchClass::new(3));
        set.insert(PitchClass::new(15));
        set.insert(PitchClass::new(0));
        assert_eq!(set.len(), 2);
        assert!(set.contains(PitchClass::new(3)));
        assert!(!set.contains(PitchClass::new(4)));
        assert_eq!(
            set.iter().map(PitchClass::value).collect::<Vec<_>>(),
            [0, 3]
        );
    }
}
