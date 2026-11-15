//! Pure music theory for Mackey Guitar Tutor: notes, intervals, scales, keys, chords, the
//! chord-symbol parser, Roman numerals, chord-function analysis and display spelling.
//!
//! The crate does no I/O, spawns no threads and keeps no global state; every function is
//! deterministic. Spelling follows the rules in `docs/prd/README.md` (G-1): a note's name comes
//! from its key, chord or scale context, never from a fixed table, so F major has Bb and the
//! third of D7 is F#.
//!
//! ```
//! use mackey_theory::{analyze, parse_chord_symbol, Key, Mode};
//!
//! let c_major = Key::new("C".parse().unwrap(), Mode::Major).unwrap();
//! let d7 = parse_chord_symbol("D7").unwrap();
//! let notes: Vec<String> = d7.tones().iter().map(|t| t.note.name()).collect();
//! assert_eq!(notes, ["D", "F#", "A", "C"]);
//! assert_eq!(analyze(&c_major, &d7).badge().as_deref(), Some("Not in key (V/V)"));
//! ```

mod analysis;
mod chord;
mod error;
mod interval;
mod key;
mod parser;
mod pitch;
mod roman;
mod scale;
mod spelling;

pub use analysis::{ChordAnalysis, ChordFunction, analyze};
pub use chord::{Chord, ChordQuality, ChordTone, PitchClassSet, ToneRole};
pub use error::TheoryError;
pub use interval::{Interval, IntervalQuality};
pub use key::{Key, KeyError, KeyNotice, KeySignature, OFFERED_MAJOR_TONICS, OFFERED_MINOR_TONICS};
pub use parser::{
    ChordParseError, Diagnostic, ParseOptions, ParseOutcome, Severity, Span, parse_chord,
    parse_chord_symbol,
};
pub use pitch::{
    Accidental, AccidentalPreference, Letter, Note, Pitch, PitchClass, ReferencePitch,
};
pub use roman::{
    DiatonicChord, NumeralCase, RomanNumeral, RomanParseError, RomanSuffix, roman_to_chord,
};
pub use scale::{Degree, Mode, Scale};
pub use spelling::{DisplayNote, Speller};
