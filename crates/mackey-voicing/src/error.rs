//! Errors for the voicing crate.

use mackey_theory::TheoryError;
use thiserror::Error;

use crate::instrument::{Capo, Fret};

/// Everything that can go wrong when building instrument values. Search and view functions
/// that cannot fail return plain values (an empty list or `VoicingResult::NoneFound`), never
/// an error (FB-5.6).
#[derive(Debug, Clone, PartialEq, Error)]
pub enum VoicingError {
    /// A string number outside `1..=6`.
    #[error("string number {0} is outside 1..=6")]
    StringOutOfRange(u8),
    /// A fret outside `0..=24`.
    #[error("fret {0} is outside 0..=24")]
    FretOutOfRange(u8),
    /// A fret below the capo is not playable.
    #[error("fret {fret:?} is below the capo at {capo:?}")]
    BelowCapo {
        /// The requested fret.
        fret: Fret,
        /// The capo position.
        capo: Capo,
    },
    /// A fret count outside `19..=24`.
    #[error("fret count {0} is outside 19..=24")]
    FretCountOutOfRange(u8),
    /// A capo outside `0..=12`.
    #[error("capo {0} is outside 0..=12")]
    CapoOutOfRange(u8),
    /// A string tuned outside C1..=C6 (MIDI 24..=84).
    #[error("string {string} is tuned to MIDI {midi}, outside 24..=84 (C1 to C6)")]
    TuningOutOfRange {
        /// The string number `1..=6`.
        string: u8,
        /// The MIDI note requested.
        midi: i32,
    },
    /// A tempo outside 30..=250 bpm.
    #[error("tempo {0} is outside 30..=250 bpm")]
    TempoOutOfRange(u16),
    /// A fret range whose start is after its end.
    #[error("fret range is empty or reversed")]
    InvalidRange,
    /// The selected note does not fit on the neck.
    #[error("the note does not fit on this instrument")]
    NoteDoesNotFit,
    /// A theory error.
    #[error(transparent)]
    Theory(#[from] TheoryError),
}
