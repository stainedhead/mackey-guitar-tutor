//! Chord function analysis: in key, secondary dominant, borrowed, or unknown (FB-4.5).

use serde::{Deserialize, Serialize};

use crate::chord::{Chord, ChordQuality};
use crate::key::Key;
use crate::pitch::{Accidental, Note};
use crate::roman::{RomanNumeral, stacked_quality};
use crate::scale::{Degree, Mode, Scale};

/// What a chord outside the key does, where that is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum ChordFunction {
    /// A dominant (seventh or major triad) on the fifth above a diatonic degree: `D7` in C is
    /// `V/V`.
    SecondaryDominant {
        /// The degree it points to, as a numeral in the key (`V`, `ii`).
        target: RomanNumeral,
    },
    /// A chord diatonic to a parallel mode: `Bb` in C is `bVII`, borrowed from C minor.
    Borrowed {
        /// The mode it comes from.
        from: Mode,
        /// Its numeral in the home key.
        roman: RomanNumeral,
    },
    /// No function this app names.
    Unknown,
}

/// The result of [`analyze`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct ChordAnalysis {
    /// True when every tone of the chord (and its bass) is in the key's scale.
    pub in_key: bool,
    /// The numeral, when the chord has one.
    pub roman: Option<RomanNumeral>,
    /// The function of a chord that is not in the key.
    pub function: Option<ChordFunction>,
    /// The key the chord was analysed in, for the badge text.
    pub key: Key,
}

impl ChordAnalysis {
    /// The badge text: `None` in key; "Not in key (V/V)", "Not in key (bVII, borrowed from G
    /// minor)", or "Not in key" when the function is unknown.
    ///
    /// ```
    /// use mackey_theory::{analyze, Key, Mode, parse_chord_symbol};
    /// let c = Key::new("C".parse().unwrap(), Mode::Major).unwrap();
    /// let d7 = parse_chord_symbol("D7").unwrap();
    /// assert_eq!(analyze(&c, &d7).badge().as_deref(), Some("Not in key (V/V)"));
    /// ```
    #[must_use]
    pub fn badge(&self) -> Option<String> {
        if self.in_key {
            return None;
        }
        Some(match self.function {
            Some(ChordFunction::SecondaryDominant { target }) => {
                format!("Not in key (V/{target})")
            }
            Some(ChordFunction::Borrowed { from, roman }) => {
                let scale = Scale {
                    tonic: self.key.tonic(),
                    mode: from,
                };
                format!("Not in key ({roman}, borrowed from {})", scale.name())
            }
            _ => "Not in key".to_owned(),
        })
    }
}

fn is_dominant_family(quality: ChordQuality) -> bool {
    matches!(
        quality,
        ChordQuality::Major
            | ChordQuality::Dominant7
            | ChordQuality::Dominant9
            | ChordQuality::Dominant11
            | ChordQuality::Dominant13
    )
}

/// The numeral of a root note relative to a key's scale: the degree by letter, the accidental
/// by pitch class (only `b`, natural or `#` are representable).
fn numeral_for(key: Key, root: Note, quality: ChordQuality) -> Option<RomanNumeral> {
    let scale = key.scale();
    let notes = scale.degrees();
    let index = notes.iter().position(|n| n.letter == root.letter)?;
    let base = notes.get(index)?;
    let diff = (i32::from(root.pitch_class().value()) - i32::from(base.pitch_class().value()) + 6)
        .rem_euclid(12)
        - 6;
    let accidental = Accidental::try_from(i8::try_from(diff).ok()?).ok()?;
    if accidental.value().abs() > 1 {
        return None;
    }
    let degree = Degree::new(u8::try_from(index + 1).ok()?).ok()?;
    RomanNumeral::from_quality(degree, accidental, quality)
}

fn secondary_dominant(key: Key, chord: Chord) -> Option<ChordFunction> {
    if !is_dominant_family(chord.quality) {
        return None;
    }
    let target_pc = chord.root.pitch_class().transpose(-7);
    let degree = key.degree_of(target_pc)?;
    if degree.value() == 1 {
        return None;
    }
    let target = key.diatonic_chord(degree, false);
    if !matches!(
        target.chord.quality,
        ChordQuality::Major | ChordQuality::Minor
    ) {
        return None;
    }
    Some(ChordFunction::SecondaryDominant {
        target: target.roman,
    })
}

fn parallel_modes(key: Key) -> &'static [Mode] {
    match key.mode() {
        Mode::Major => &[Mode::NaturalMinor],
        Mode::NaturalMinor => &[Mode::HarmonicMinor, Mode::Major],
        _ => &[Mode::Major, Mode::NaturalMinor],
    }
}

fn borrowed(key: Key, chord: Chord) -> Option<ChordFunction> {
    let wanted = chord.root.pitch_class();
    for &mode in parallel_modes(key) {
        let notes = Scale {
            tonic: key.tonic(),
            mode,
        }
        .degrees();
        for (index, note) in notes.iter().enumerate() {
            if note.pitch_class() != wanted {
                continue;
            }
            for seventh in [false, true] {
                if stacked_quality(&notes, index, seventh) != Some(chord.quality) {
                    continue;
                }
                // Harmonic minor is the minor key with one raised note, so its chords are
                // numbered against itself (`V`, `vii°`), not as altered natural-minor degrees.
                let roman = if mode == Mode::HarmonicMinor {
                    u8::try_from(index + 1)
                        .ok()
                        .and_then(|n| Degree::new(n).ok())
                        .and_then(|d| {
                            RomanNumeral::from_quality(d, Accidental::NATURAL, chord.quality)
                        })
                } else {
                    numeral_for(key, *note, chord.quality)
                };
                if let Some(roman) = roman {
                    return Some(ChordFunction::Borrowed { from: mode, roman });
                }
            }
        }
    }
    None
}

/// Analyses a chord in a key. In-key chords get a numeral and no function; chords outside the
/// key get a function where one is known (AC-FB-4.1: `D7` in C is `V/V`).
///
/// A dominant seventh family chord is tried as a secondary dominant first, then as borrowed;
/// any other chord is tried as borrowed first. In a minor key the major `V`/`V7` and `vii°`
/// of the harmonic minor count as borrowed from it.
///
/// ```
/// use mackey_theory::{analyze, parse_chord_symbol, Key, Mode};
/// let g = Key::new("G".parse().unwrap(), Mode::Major).unwrap();
/// let f = parse_chord_symbol("F").unwrap();
/// assert_eq!(
///     analyze(&g, &f).badge().as_deref(),
///     Some("Not in key (bVII, borrowed from G minor)")
/// );
/// ```
#[must_use]
pub fn analyze(key: &Key, chord: &Chord) -> ChordAnalysis {
    let scale = key.scale();
    let in_key = chord.pitch_classes().iter().all(|pc| scale.contains(pc));
    if in_key {
        let roman = key
            .degree_of(chord.root.pitch_class())
            .and_then(|d| RomanNumeral::from_quality(d, Accidental::NATURAL, chord.quality));
        return ChordAnalysis {
            in_key,
            roman,
            function: None,
            key: *key,
        };
    }
    let dominant_first = chord.quality != ChordQuality::Major && is_dominant_family(chord.quality);
    let function = if dominant_first {
        secondary_dominant(*key, *chord).or_else(|| borrowed(*key, *chord))
    } else {
        borrowed(*key, *chord).or_else(|| secondary_dominant(*key, *chord))
    };
    let roman = match function {
        Some(ChordFunction::Borrowed { roman, .. }) => Some(roman),
        Some(ChordFunction::SecondaryDominant { .. }) => crate::scale::Degree::new(5)
            .ok()
            .and_then(|d| RomanNumeral::from_quality(d, Accidental::NATURAL, chord.quality)),
        _ => numeral_for(*key, chord.root, chord.quality),
    };
    ChordAnalysis {
        in_key,
        roman,
        function: Some(function.unwrap_or(ChordFunction::Unknown)),
        key: *key,
    }
}
