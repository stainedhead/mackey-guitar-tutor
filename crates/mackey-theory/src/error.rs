//! Errors for the theory crate.

use thiserror::Error;

use crate::interval::IntervalQuality;
use crate::key::KeyError;

/// Everything that can go wrong when building or combining theory values.
///
/// Constructors validate their input, so a value that exists is valid; these errors name the
/// operation that was refused and carry the offending input.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum TheoryError {
    /// A pitch class outside `0..=11` was given to a checked constructor.
    #[error("pitch class {0} is outside 0..=11")]
    PitchClassOutOfRange(i32),
    /// An accidental outside double flat to double sharp (`-2..=2`).
    #[error("accidental {0} is outside -2..=2 (double flat to double sharp)")]
    AccidentalOutOfRange(i32),
    /// Text that is not a note name such as `F#` or `Bb`.
    #[error("not a note name: {0:?}")]
    InvalidNote(String),
    /// A quality and number that do not form an interval ("perfect third", "major fifth").
    #[error("{quality:?} {number} is not a valid interval")]
    InvalidInterval {
        /// The requested quality.
        quality: IntervalQuality,
        /// The requested interval number (1 = unison).
        number: u8,
    },
    /// An interval would need more than a double sharp or double flat to spell.
    #[error("spelling the result needs an accidental beyond a double sharp or double flat")]
    SpellingOutOfRange,
    /// A MIDI note outside `0..=127`.
    #[error("MIDI note {0} is outside 0..=127")]
    MidiOutOfRange(i32),
    /// A reference pitch (A4) outside 415 to 466 Hz.
    #[error("reference pitch {0} Hz is outside 415..=466 Hz")]
    ReferencePitchOutOfRange(f64),
    /// A scale degree outside `1..=7`.
    #[error("scale degree {0} is outside 1..=7")]
    InvalidDegree(u8),
    /// A key could not be built.
    #[error("invalid key: {0}")]
    Key(#[from] KeyError),
}
