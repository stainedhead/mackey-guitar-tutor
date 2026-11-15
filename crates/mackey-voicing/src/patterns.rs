//! Scale patterns: the full neck, pentatonic boxes, 3-notes-per-string patterns and the
//! follow-the-chords scale choice (SC-3, SC-4, FR-018).
//!
//! Algorithm (D-21): box (or pattern) *k* starts at scale degree *k* on string 6, at the lowest
//! fret at or above the capo whose note is that degree. Notes are then assigned in ascending
//! scale order, two per string (pentatonic) or three per string (seven-note scales), the frets
//! coming from pitch arithmetic, so every tuning works. Harmonic minor and the blues scale have
//! the full neck only (D-52).

use mackey_theory::{ChordQuality, Degree, Key, Mode, Scale, analyze};
use serde::{Deserialize, Serialize};

use crate::instrument::{Fret, Instrument, StringNumber};

/// Which pattern a [`ScalePattern`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum PatternKind {
    /// Every scale note on the neck.
    FullNeck,
    /// One of the five pentatonic boxes (`1..=5`).
    PentatonicBox {
        /// The box number.
        number: u8,
    },
    /// One of the seven 3-notes-per-string patterns (`1..=7`).
    ThreeNps {
        /// The pattern number.
        number: u8,
    },
}

/// One note of a pattern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct PatternNote {
    /// The string.
    pub string: StringNumber,
    /// The absolute fret (the capo fret is the open string).
    pub fret: Fret,
    /// The 1-based degree of the pattern's scale (SC-5).
    pub degree: Degree,
}

/// A fretboard pattern of a scale.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct ScalePattern {
    /// The scale.
    pub scale: Scale,
    /// Which pattern.
    pub kind: PatternKind,
    /// The notes, string 6 first, ascending in pitch.
    pub notes: Vec<PatternNote>,
    /// The lowest fret used.
    pub lowest_fret: Fret,
    /// The highest fret used.
    pub highest_fret: Fret,
}

fn scale_mask(scale: Scale) -> u16 {
    scale.mode.intervals().iter().fold(0u16, |m, i| {
        m | (1u16 << i.above(scale.tonic.pitch_class()).value())
    })
}

fn degree_note(
    scale: Scale,
    instrument: &Instrument,
    index: usize,
    fret: u8,
) -> Option<PatternNote> {
    let pc = instrument.pc_at_index(index, fret);
    Some(PatternNote {
        string: StringNumber::from_index(index)?,
        fret: Fret::new(fret).ok()?,
        degree: scale.degree_of(pc)?,
    })
}

fn pattern_from(scale: Scale, kind: PatternKind, notes: Vec<PatternNote>) -> Option<ScalePattern> {
    let lowest = notes.iter().map(|n| n.fret).min()?;
    let highest = notes.iter().map(|n| n.fret).max()?;
    Some(ScalePattern {
        scale,
        kind,
        notes,
        lowest_fret: lowest,
        highest_fret: highest,
    })
}

/// Every scale note on the neck from the capo up (SC-3 "Show all positions").
///
/// ```
/// use mackey_theory::{Mode, Scale};
/// use mackey_voicing::{full_neck, Instrument, PatternKind};
/// let a_minor = Scale { tonic: "A".parse().unwrap(), mode: Mode::NaturalMinor };
/// let all = full_neck(&a_minor, &Instrument::default());
/// assert_eq!(all.kind, PatternKind::FullNeck);
/// ```
#[must_use]
pub fn full_neck(scale: &Scale, instrument: &Instrument) -> ScalePattern {
    let mask = scale_mask(*scale);
    let mut notes = Vec::new();
    for index in 0..6 {
        for fret in instrument.capo.value()..=instrument.frets.value() {
            if mask & (1u16 << instrument.midi_at_index(index, fret).rem_euclid(12)) != 0
                && let Some(n) = degree_note(*scale, instrument, index, fret)
            {
                notes.push(n);
            }
        }
    }
    pattern_from(*scale, PatternKind::FullNeck, notes).unwrap_or(ScalePattern {
        scale: *scale,
        kind: PatternKind::FullNeck,
        notes: Vec::new(),
        lowest_fret: instrument.lowest_fret(),
        highest_fret: instrument.lowest_fret(),
    })
}

fn build_positional(
    scale: Scale,
    instrument: &Instrument,
    k: usize,
    per_string: usize,
) -> Option<Vec<PatternNote>> {
    let mask = scale_mask(scale);
    let intervals = scale.mode.intervals();
    let target = intervals.get(k)?.above(scale.tonic.pitch_class());
    let capo = instrument.capo.value();
    let frets = instrument.frets.value();
    let start_fret = (capo..=frets).find(|&f| instrument.pc_at_index(0, f) == target)?;
    let mut notes = Vec::new();
    let mut previous_top: Option<i32> = None;
    for index in 0..6 {
        let open = instrument.midi_at_index(index, 0);
        let floor = match previous_top {
            None => open + i32::from(start_fret),
            Some(top) => (top + 1).max(open + i32::from(capo)),
        };
        let mut taken = 0;
        let mut midi = floor;
        while taken < per_string && midi <= open + i32::from(frets) {
            if mask & (1u16 << midi.rem_euclid(12)) != 0 {
                let fret = u8::try_from(midi - open).ok()?;
                notes.push(degree_note(scale, instrument, index, fret)?);
                previous_top = Some(midi);
                taken += 1;
            }
            midi += 1;
        }
        if taken < per_string {
            break;
        }
    }
    Some(notes)
}

/// The positional patterns of a scale (SC-3), ordered by lowest fret from the first at or above
/// the capo: five boxes for the two pentatonics, seven 3-notes-per-string patterns for the
/// seven-note modes (Major, Natural minor and the five church modes). Harmonic minor and
/// blues have none: use [`full_neck`].
///
/// ```
/// use mackey_theory::{Mode, Scale};
/// use mackey_voicing::{scale_patterns, Instrument};
/// let am_pent = Scale { tonic: "A".parse().unwrap(), mode: Mode::MinorPentatonic };
/// assert_eq!(scale_patterns(&am_pent, &Instrument::default()).len(), 5);
/// ```
#[must_use]
pub fn scale_patterns(scale: &Scale, instrument: &Instrument) -> Vec<ScalePattern> {
    let (count, per_string, boxed) = match scale.mode {
        Mode::MajorPentatonic | Mode::MinorPentatonic => (5, 2, true),
        Mode::Blues | Mode::HarmonicMinor => return Vec::new(),
        _ => (7, 3, false),
    };
    let mut patterns: Vec<ScalePattern> = (0..count)
        .filter_map(|k| {
            let notes = build_positional(*scale, instrument, k, per_string)?;
            let number = u8::try_from(k + 1).ok()?;
            let kind = if boxed {
                PatternKind::PentatonicBox { number }
            } else {
                PatternKind::ThreeNps { number }
            };
            pattern_from(*scale, kind, notes)
        })
        .collect();
    patterns.sort_by_key(|p| (p.lowest_fret, p.highest_fret));
    patterns
}

/// Why `scale_for_chord` chose a scale (SC-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum FollowReason {
    /// The chord is in the key: the key's own scale.
    Diatonic,
    /// A dominant chord: the Mixolydian mode on its root.
    DominantMixolydian,
    /// A major chord out of key: the major scale (Ionian) on its root.
    MajorIonian,
    /// A minor chord out of key: Dorian on its root.
    MinorDorian,
    /// A diminished chord out of key: Locrian on its root.
    DiminishedLocrian,
    /// A chord SC-4 does not cover (sus, power, augmented): the key's own scale.
    Fallback,
}

/// The scale "Follow the chords" shows for a chord (SC-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct FollowChoice {
    /// The scale.
    pub scale: Scale,
    /// Why.
    pub reason: FollowReason,
}

/// The scale that fits a chord in a key (SC-4): the key's scale when the chord is diatonic;
/// otherwise the chord root's mode that contains it: Mixolydian for dominant chords, Ionian for
/// major chords, Dorian for minor chords, Locrian for diminished and half-diminished chords.
///
/// ```
/// use mackey_theory::{parse_chord_symbol, Key, Mode};
/// use mackey_voicing::{scale_for_chord, FollowReason};
/// let c = Key::new("C".parse().unwrap(), Mode::Major).unwrap();
/// let choice = scale_for_chord(&c, &parse_chord_symbol("A7").unwrap());
/// assert_eq!(choice.scale.name(), "A Mixolydian");
/// assert_eq!(choice.reason, FollowReason::DominantMixolydian);
/// ```
#[must_use]
pub fn scale_for_chord(key: &Key, chord: &mackey_theory::Chord) -> FollowChoice {
    use ChordQuality as Q;
    let key_scale = key.scale();
    if analyze(key, chord).in_key {
        return FollowChoice {
            scale: key_scale,
            reason: FollowReason::Diatonic,
        };
    }
    let (mode, reason) = match chord.quality {
        Q::Dominant7 | Q::Dominant9 | Q::Dominant11 | Q::Dominant13 | Q::Dominant7Sus4 => {
            (Mode::Mixolydian, FollowReason::DominantMixolydian)
        }
        Q::Major | Q::Major7 | Q::Major9 | Q::Add9 | Q::Sixth => {
            (Mode::Major, FollowReason::MajorIonian)
        }
        Q::Minor | Q::Minor7 | Q::Minor9 | Q::MinorSixth => {
            (Mode::Dorian, FollowReason::MinorDorian)
        }
        Q::Diminished | Q::Diminished7 | Q::HalfDiminished7 => {
            (Mode::Locrian, FollowReason::DiminishedLocrian)
        }
        Q::Sus2 | Q::Sus4 | Q::Power | Q::Augmented => {
            return FollowChoice {
                scale: key_scale,
                reason: FollowReason::Fallback,
            };
        }
    };
    FollowChoice {
        scale: Scale {
            tonic: chord.root,
            mode,
        },
        reason,
    }
}
