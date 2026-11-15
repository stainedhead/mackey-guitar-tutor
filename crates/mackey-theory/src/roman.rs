//! Roman numerals and diatonic chords.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::chord::{Chord, ChordQuality};
use crate::key::Key;
use crate::parser::Span;
use crate::pitch::{Accidental, AccidentalPreference, Note};
use crate::scale::{Degree, Mode, Scale};

/// Upper case (major, augmented) or lower case (minor, diminished).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum NumeralCase {
    /// `IV`
    Upper,
    /// `ii`
    Lower,
}

/// What follows the numeral.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum RomanSuffix {
    /// Triad.
    None,
    /// `°`: diminished triad (lower case only).
    Dim,
    /// `ø7`: half-diminished seventh (lower case only).
    HalfDim,
    /// `+`: augmented triad (upper case only).
    Aug,
    /// `7`: dominant seventh (upper) or minor seventh (lower).
    Seventh,
    /// `maj7` (upper case only).
    Maj7,
    /// `°7`: diminished seventh (lower case only).
    Dim7,
}

/// A Roman numeral chord label: `I`, `ii`, `vii°`, `viiø7`, `bVII`, `V7`, `IVmaj7`.
///
/// ```
/// use mackey_theory::RomanNumeral;
/// let n: RomanNumeral = "bVII".parse().unwrap();
/// assert_eq!(n.to_string(), "bVII");
/// assert_eq!("viiø7".parse::<RomanNumeral>().unwrap().to_string(), "viiø7");
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct RomanNumeral {
    /// Scale degree 1..=7.
    pub degree: Degree,
    /// `-1` for `b`, `1` for `#`, `0` for none.
    pub accidental: Accidental,
    /// Upper or lower case.
    pub case: NumeralCase,
    /// The suffix.
    pub suffix: RomanSuffix,
}

const UPPER: [&str; 7] = ["I", "II", "III", "IV", "V", "VI", "VII"];
const LOWER: [&str; 7] = ["i", "ii", "iii", "iv", "v", "vi", "vii"];

impl RomanNumeral {
    /// The numeral for a chord quality on a degree; `None` for qualities without a numeral form
    /// (suspended, sixth, ninth chords and so on).
    #[must_use]
    pub fn from_quality(
        degree: Degree,
        accidental: Accidental,
        quality: ChordQuality,
    ) -> Option<Self> {
        let (case, suffix) = match quality {
            ChordQuality::Major => (NumeralCase::Upper, RomanSuffix::None),
            ChordQuality::Minor => (NumeralCase::Lower, RomanSuffix::None),
            ChordQuality::Diminished => (NumeralCase::Lower, RomanSuffix::Dim),
            ChordQuality::Augmented => (NumeralCase::Upper, RomanSuffix::Aug),
            ChordQuality::Dominant7 => (NumeralCase::Upper, RomanSuffix::Seventh),
            ChordQuality::Major7 => (NumeralCase::Upper, RomanSuffix::Maj7),
            ChordQuality::Minor7 => (NumeralCase::Lower, RomanSuffix::Seventh),
            ChordQuality::HalfDiminished7 => (NumeralCase::Lower, RomanSuffix::HalfDim),
            ChordQuality::Diminished7 => (NumeralCase::Lower, RomanSuffix::Dim7),
            _ => return None,
        };
        Some(Self {
            degree,
            accidental,
            case,
            suffix,
        })
    }

    /// The chord quality the numeral stands for; `None` for an inconsistent combination such as
    /// an upper-case `°` (possible only by building the struct by hand).
    #[must_use]
    pub fn quality(&self) -> Option<ChordQuality> {
        use NumeralCase::{Lower, Upper};
        use RomanSuffix as S;
        Some(match (self.case, self.suffix) {
            (Upper, S::None) => ChordQuality::Major,
            (Lower, S::None) => ChordQuality::Minor,
            (Lower, S::Dim) => ChordQuality::Diminished,
            (Upper, S::Aug) => ChordQuality::Augmented,
            (Upper, S::Seventh) => ChordQuality::Dominant7,
            (Lower, S::Seventh) => ChordQuality::Minor7,
            (Upper, S::Maj7) => ChordQuality::Major7,
            (Lower, S::HalfDim) => ChordQuality::HalfDiminished7,
            (Lower, S::Dim7) => ChordQuality::Diminished7,
            _ => return None,
        })
    }

    /// This numeral without its suffix, upper case: `V`, `bVII`. Used for chord functions.
    #[must_use]
    pub fn plain(&self) -> Self {
        Self {
            suffix: RomanSuffix::None,
            ..*self
        }
    }
}

impl fmt::Display for RomanNumeral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let accidental = match self.accidental.value() {
            -1 => "b",
            1 => "#",
            _ => "",
        };
        let table = if self.case == NumeralCase::Upper {
            &UPPER
        } else {
            &LOWER
        };
        let numeral = table.get(self.degree.index()).copied().unwrap_or("?");
        let suffix = match self.suffix {
            RomanSuffix::None => "",
            RomanSuffix::Dim => "°",
            RomanSuffix::HalfDim => "ø7",
            RomanSuffix::Aug => "+",
            RomanSuffix::Seventh => "7",
            RomanSuffix::Maj7 => "maj7",
            RomanSuffix::Dim7 => "°7",
        };
        write!(f, "{accidental}{numeral}{suffix}")
    }
}

/// Why a Roman numeral could not be read.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RomanParseError {
    /// Nothing to parse.
    #[error("Type a Roman numeral such as vi or V7.")]
    Empty,
    /// Not made of `I`, `V`, `i`, `v` in one case.
    #[error("Not a Roman numeral: {found}")]
    NotANumeral {
        /// Where.
        span: Span,
        /// The text.
        found: String,
    },
    /// A numeral above VII (`VIII`).
    #[error("Roman numerals go from I to VII, not {found}")]
    DegreeOutOfRange {
        /// Where.
        span: Span,
        /// The text.
        found: String,
    },
    /// A suffix this app does not know, or one that does not suit the case (`V°`).
    #[error("Not a chord type I know after the numeral: {found}")]
    UnsupportedSuffix {
        /// Where.
        span: Span,
        /// The text.
        found: String,
    },
}

fn parse_numeral_at(text: &str, offset: usize) -> Result<RomanNumeral, RomanParseError> {
    if text.trim().is_empty() {
        return Err(RomanParseError::Empty);
    }
    let sp = |a: usize, b: usize| Span::new(offset + a, offset + b);
    let mut pos = 0;
    let mut accidental = Accidental::NATURAL;
    if let Some(c) = text
        .chars()
        .next()
        .filter(|c| matches!(c, 'b' | '♭' | '#' | '♯'))
    {
        accidental = if matches!(c, '#' | '♯') {
            Accidental::SHARP
        } else {
            Accidental::FLAT
        };
        pos += c.len_utf8();
    }
    let rest = text.get(pos..).unwrap_or_default();
    let run_len: usize = rest
        .chars()
        .take_while(|c| matches!(c, 'I' | 'V' | 'i' | 'v'))
        .map(char::len_utf8)
        .sum();
    let run = rest.get(..run_len).unwrap_or_default();
    let upper = !run.is_empty() && run.chars().all(|c| c.is_ascii_uppercase());
    let lower = !run.is_empty() && run.chars().all(|c| c.is_ascii_lowercase());
    if !upper && !lower {
        let end = if run_len == 0 {
            pos + rest.chars().next().map_or(0, char::len_utf8)
        } else {
            pos + run_len
        };
        return Err(RomanParseError::NotANumeral {
            span: sp(pos, end),
            found: text.get(pos..end).unwrap_or_default().to_owned(),
        });
    }
    let (case, table) = if upper {
        (NumeralCase::Upper, &UPPER)
    } else {
        (NumeralCase::Lower, &LOWER)
    };
    let degree = table
        .iter()
        .position(|s| *s == run)
        .and_then(|i| u8::try_from(i + 1).ok())
        .and_then(|n| Degree::new(n).ok())
        .ok_or_else(|| RomanParseError::DegreeOutOfRange {
            span: sp(pos, pos + run_len),
            found: run.to_owned(),
        })?;
    pos += run_len;
    let suffix_text = text.get(pos..).unwrap_or_default();
    let suffix = match suffix_text {
        "" => RomanSuffix::None,
        "°" | "o" | "dim" => RomanSuffix::Dim,
        "ø" | "ø7" | "m7b5" => RomanSuffix::HalfDim,
        "+" | "aug" => RomanSuffix::Aug,
        "7" => RomanSuffix::Seventh,
        "maj7" | "M7" | "Δ" | "Δ7" => RomanSuffix::Maj7,
        "°7" | "o7" | "dim7" => RomanSuffix::Dim7,
        _ => {
            return Err(RomanParseError::UnsupportedSuffix {
                span: sp(pos, text.len()),
                found: suffix_text.to_owned(),
            });
        }
    };
    let numeral = RomanNumeral {
        degree,
        accidental,
        case,
        suffix,
    };
    if numeral.quality().is_none() {
        return Err(RomanParseError::UnsupportedSuffix {
            span: sp(pos, text.len()),
            found: suffix_text.to_owned(),
        });
    }
    Ok(numeral)
}

impl FromStr for RomanNumeral {
    type Err = RomanParseError;

    /// Reads `I`..`VII` or `i`..`vii` with an optional `b`/`#` prefix and a suffix (`°`, `ø7`,
    /// `+`, `7`, `maj7`, `°7`). The case must suit the suffix.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let trimmed = s.trim();
        let offset = s.len() - s.trim_start().len();
        parse_numeral_at(trimmed, offset)
    }
}

/// One chip of the diatonic chord strip.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct DiatonicChord {
    /// The scale degree.
    pub degree: Degree,
    /// Its numeral in the key (`ii`, `V7`, `viiø7`).
    pub roman: RomanNumeral,
    /// The chord.
    pub chord: Chord,
}

/// The quality of the chord built on `notes[index]` by stacking thirds within the scale.
pub(crate) fn stacked_quality(notes: &[Note], index: usize, seventh: bool) -> Option<ChordQuality> {
    let at = |step: usize| notes.get((index + step) % notes.len().max(1)).copied();
    let root = at(0)?.pitch_class();
    let up = |step: usize| at(step).map(|n| root.semitones_to(n.pitch_class()));
    let (third, fifth) = (up(2)?, up(4)?);
    if !seventh {
        return match (third, fifth) {
            (4, 7) => Some(ChordQuality::Major),
            (3, 7) => Some(ChordQuality::Minor),
            (3, 6) => Some(ChordQuality::Diminished),
            (4, 8) => Some(ChordQuality::Augmented),
            _ => None,
        };
    }
    match (third, fifth, up(6)?) {
        (4, 7, 10) => Some(ChordQuality::Dominant7),
        (4, 7, 11) => Some(ChordQuality::Major7),
        (3, 7, 10) => Some(ChordQuality::Minor7),
        (3, 6, 10) => Some(ChordQuality::HalfDiminished7),
        (3, 6, 9) => Some(ChordQuality::Diminished7),
        _ => None,
    }
}

impl Key {
    /// The diatonic chord on a degree: stacked thirds from the key's own scale, so
    /// `Imaj7 ii7 iii7 IVmaj7 V7 vi7 viiø7` in major and the mode's own qualities in the other
    /// modes. Roots are spelled by the scale.
    ///
    /// ```
    /// use mackey_theory::{Degree, Key, Mode};
    /// let g = Key::new("G".parse().unwrap(), Mode::Major).unwrap();
    /// let vi = g.diatonic_chord(Degree::new(6).unwrap(), false);
    /// assert_eq!(vi.chord.symbol(), "Em");
    /// assert_eq!(vi.roman.to_string(), "vi");
    /// ```
    #[must_use]
    pub fn diatonic_chord(&self, degree: Degree, seventh: bool) -> DiatonicChord {
        let notes = self.scale().degrees();
        let root = notes.get(degree.index()).copied().unwrap_or(self.tonic());
        // Every one of the seven offered modes stacks into a supported quality (tested for all
        // offered keys), so the fallback is never taken.
        let quality =
            stacked_quality(&notes, degree.index(), seventh).unwrap_or(ChordQuality::Major);
        let roman = RomanNumeral::from_quality(degree, Accidental::NATURAL, quality).unwrap_or(
            RomanNumeral {
                degree,
                accidental: Accidental::NATURAL,
                case: NumeralCase::Upper,
                suffix: RomanSuffix::None,
            },
        );
        DiatonicChord {
            degree,
            roman,
            chord: Chord::new(root, quality),
        }
    }

    /// The seven diatonic chords, triads or sevenths.
    #[must_use]
    pub fn diatonic_chords(&self, seventh: bool) -> Vec<DiatonicChord> {
        Degree::all()
            .into_iter()
            .map(|d| self.diatonic_chord(d, seventh))
            .collect()
    }
}

/// The note a numeral's degree and accidental name in a seven-note scale.
fn numeral_root(scale: Scale, numeral: RomanNumeral) -> Option<Note> {
    let base = scale.note_at(numeral.degree)?;
    let pc = base
        .pitch_class()
        .transpose(i32::from(numeral.accidental.value()));
    Some(
        Note::from_letter_and_pitch_class(base.letter, pc)
            .unwrap_or_else(|| Note::from_pitch_class(pc, AccidentalPreference::Sharps)),
    )
}

/// Turns a Roman numeral into a chord in a key: `vi` in G is Em, `bVII` in C is Bb. A numeral
/// may be followed by `/` and a target numeral (`V7/V` in C is D7): it is then read in the
/// major key built on the target's root.
///
/// ```
/// use mackey_theory::{roman_to_chord, Key, Mode};
/// let c = Key::new("C".parse().unwrap(), Mode::Major).unwrap();
/// assert_eq!(roman_to_chord("bVII", &c).unwrap().symbol(), "Bb");
/// assert_eq!(roman_to_chord("V7/V", &c).unwrap().symbol(), "D7");
/// ```
///
/// # Errors
/// [`RomanParseError`] naming the part that failed.
pub fn roman_to_chord(s: &str, key: &Key) -> Result<Chord, RomanParseError> {
    let trimmed = s.trim();
    let offset = s.len() - s.trim_start().len();
    let (head, target) = match trimmed.find('/') {
        Some(i) => (
            trimmed.get(..i).unwrap_or(trimmed),
            trimmed.get(i + 1..).map(|t| (t, i + 1)),
        ),
        None => (trimmed, None),
    };
    let numeral = parse_numeral_at(head, offset)?;
    let scale = match target {
        Some((text, at)) => {
            let target_numeral = parse_numeral_at(text, offset + at)?;
            let root = numeral_root(key.scale(), target_numeral).unwrap_or(key.tonic());
            Scale {
                tonic: root,
                mode: Mode::Major,
            }
        }
        None => key.scale(),
    };
    let root = numeral_root(scale, numeral).unwrap_or(key.tonic());
    let quality = numeral
        .quality()
        .ok_or(RomanParseError::UnsupportedSuffix {
            span: Span::new(offset, offset + head.len()),
            found: head.to_owned(),
        })?;
    Ok(Chord::new(root, quality))
}
