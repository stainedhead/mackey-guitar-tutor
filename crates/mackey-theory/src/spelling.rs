//! Spelling a pitch class for display (G-1.5, G-1.6).

use serde::{Deserialize, Serialize};

use crate::chord::Chord;
use crate::key::Key;
use crate::pitch::{Note, PitchClass};

/// A note as shown on a label, with the theory spelling kept for the tooltip (G-1.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct DisplayNote {
    /// What the label shows: never a double sharp or double flat.
    pub label: Note,
    /// The spelling theory gives (`F##`); equal to `label` unless it was simplified.
    pub theory: Note,
}

impl DisplayNote {
    /// Simplifies a spelled note for display.
    ///
    /// ```
    /// use mackey_theory::{DisplayNote, Note};
    /// let d = DisplayNote::of("F##".parse::<Note>().unwrap());
    /// assert_eq!(d.label.name(), "G");
    /// assert_eq!(d.theory.name(), "F##");
    /// assert!(d.is_simplified());
    /// ```
    #[must_use]
    pub fn of(theory: Note) -> Self {
        Self {
            label: theory.simplified(),
            theory,
        }
    }

    /// True when the label differs from the theory spelling.
    #[must_use]
    pub fn is_simplified(&self) -> bool {
        self.label != self.theory
    }
}

/// Resolves the spelling of a pitch class: first as a tone of the chord in play, then as a
/// tone of the key's scale, then by the key's sharps-or-flats preference (G-1.5).
///
/// ```
/// use mackey_theory::{Chord, ChordQuality, Key, Mode, PitchClass, Speller};
/// let c_major = Key::new("C".parse().unwrap(), Mode::Major).unwrap();
/// let d7 = Chord::new("D".parse().unwrap(), ChordQuality::Dominant7);
/// let speller = Speller::new(c_major).with_chord(d7);
/// assert_eq!(speller.spell(PitchClass::new(6)).name(), "F#");
/// assert_eq!(Speller::new(c_major).spell(PitchClass::new(10)).name(), "Bb");
/// ```
#[derive(Debug, Clone, Copy)]
pub struct Speller {
    key: Key,
    chord: Option<Chord>,
}

impl Speller {
    /// A speller with no chord context.
    #[must_use]
    pub fn new(key: Key) -> Self {
        Self { key, chord: None }
    }

    /// Adds the chord in play: its tones are spelled by function.
    #[must_use]
    pub fn with_chord(self, chord: Chord) -> Self {
        Self {
            chord: Some(chord),
            ..self
        }
    }

    /// The theory spelling (may be a double accidental, for example the third of `F#aug`... or
    /// the raised seventh of G# minor as `F##`).
    #[must_use]
    pub fn spell(&self, pc: PitchClass) -> Note {
        if let Some(chord) = &self.chord {
            if let Some(tone) = chord
                .tones()
                .into_iter()
                .find(|t| t.note.pitch_class() == pc)
            {
                return tone.note;
            }
            if let Some(bass) = chord.bass.filter(|b| b.pitch_class() == pc) {
                return bass;
            }
        }
        if let Some(note) = self
            .key
            .scale()
            .degrees()
            .into_iter()
            .find(|n| n.pitch_class() == pc)
        {
            return note;
        }
        Note::from_pitch_class(pc, self.key.preference())
    }

    /// The label to draw and the theory spelling for the tooltip.
    ///
    /// ```
    /// use mackey_theory::{Chord, ChordQuality, Key, Mode, PitchClass, Speller};
    /// let g_sharp_minor = Key::new("G#".parse().unwrap(), Mode::NaturalMinor).unwrap();
    /// let d_sharp_7 = Chord::new("D#".parse().unwrap(), ChordQuality::Dominant7);
    /// let shown = Speller::new(g_sharp_minor).with_chord(d_sharp_7).display(PitchClass::new(7));
    /// assert_eq!(shown.label.name(), "G");
    /// assert_eq!(shown.theory.name(), "F##");
    /// ```
    #[must_use]
    pub fn display(&self, pc: PitchClass) -> DisplayNote {
        DisplayNote::of(self.spell(pc))
    }
}
