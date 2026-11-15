//! Arpeggios inside a position (FB-7, FR-017).
//!
//! A position is a start fret `s`: the window holds every chord tone on frets `s-1..=s+4`
//! (four frets plus one either side, D-24). Fingering rule (FB-7.2): index finger on the start
//! fret, one finger per fret, finger 1 stretching back to `s-1` and finger 4 reaching to `s+4`.

use mackey_theory::{Chord, PitchClassSet, ToneRole};
use serde::{Deserialize, Serialize};

use crate::instrument::{Fret, Instrument, StringNumber};
use crate::rules::Rules;
use crate::voicing::{StringState, Voicing, VoicingOptions};

/// The play direction of an arpeggio (FB-7.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum ArpeggioDirection {
    /// Lowest note to highest.
    #[default]
    Up,
    /// Highest to lowest.
    Down,
    /// Up, then back down without repeating the top note.
    UpDown,
}

impl ArpeggioDirection {
    /// The label "up", "down", "up-down".
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            ArpeggioDirection::Up => "up",
            ArpeggioDirection::Down => "down",
            ArpeggioDirection::UpDown => "up and down",
        }
    }
}

/// One chord tone inside the window.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct ArpeggioNote {
    /// The string.
    pub string: StringNumber,
    /// The absolute fret.
    pub fret: Fret,
    /// True when the string rings open at the capo fret.
    pub is_open: bool,
    /// The chord tone's role.
    pub role: ToneRole,
    /// Suggested finger: 0 for open, 1 to 4 otherwise.
    pub finger: u8,
}

/// The notes of one arpeggio position, lowest pitch first, and the play order.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Arpeggio {
    /// The start fret of the position.
    pub start: Fret,
    /// Every chord tone in the window, ascending in pitch.
    pub notes: Vec<ArpeggioNote>,
    /// Indexes into `notes` in play order (`UpDown` visits inner notes twice).
    pub sequence: Vec<usize>,
}

impl Arpeggio {
    /// The 1-based play-order number of each note (its first visit), for "Show order".
    #[must_use]
    pub fn order_numbers(&self) -> Vec<u8> {
        (0..self.notes.len())
            .map(|i| {
                let pos = self.sequence.iter().position(|&s| s == i).unwrap_or(0);
                u8::try_from(pos + 1).unwrap_or(u8::MAX)
            })
            .collect()
    }
}

fn chord_pc_set(chord: Chord) -> PitchClassSet {
    chord
        .quality
        .intervals()
        .iter()
        .map(|i| i.above(chord.root.pitch_class()))
        .collect()
}

/// Every chord tone in the window of position `start` (FB-7.1): frets `start-1..=start+4`,
/// clamped to the capo and the fret count. The order follows `direction`.
///
/// ```
/// use mackey_theory::{Chord, ChordQuality};
/// use mackey_voicing::{arpeggio, ArpeggioDirection, Instrument};
/// let am = Chord::new("A".parse().unwrap(), ChordQuality::Minor);
/// let a = arpeggio(&am, &Instrument::default(), 5, ArpeggioDirection::Up);
/// assert!(a.notes.iter().all(|n| (4..=9).contains(&n.fret.value())));
/// assert!(a.notes.iter().any(|n| n.fret.value() == 5 && n.finger == 1));
/// ```
#[must_use]
pub fn arpeggio(
    chord: &Chord,
    instrument: &Instrument,
    start: u8,
    direction: ArpeggioDirection,
) -> Arpeggio {
    let tones = chord.tones();
    let pcs = chord_pc_set(*chord);
    let capo = instrument.capo.value();
    let lo = start.saturating_sub(1).max(capo);
    let hi = start.saturating_add(4).min(instrument.frets.value());
    let mut found: Vec<(i32, usize, ArpeggioNote)> = Vec::new();
    for index in 0..6 {
        let Some(string) = StringNumber::from_index(index) else {
            continue;
        };
        for fret in lo..=hi {
            let midi = instrument.midi_at_index(index, fret);
            let pc = mackey_theory::PitchClass::new(midi);
            if !pcs.contains(pc) {
                continue;
            }
            let role = tones
                .iter()
                .find(|t| t.note.pitch_class() == pc)
                .map_or(ToneRole::Extension, |t| t.role);
            let is_open = fret == capo;
            let finger = if is_open {
                0
            } else {
                let raw = i16::from(fret) - i16::from(start) + 1;
                u8::try_from(raw.clamp(1, 4)).unwrap_or(1)
            };
            let Ok(fret) = Fret::new(fret) else { continue };
            found.push((
                midi,
                index,
                ArpeggioNote {
                    string,
                    fret,
                    is_open,
                    role,
                    finger,
                },
            ));
        }
    }
    found.sort_by_key(|(midi, index, _)| (*midi, *index));
    let notes: Vec<ArpeggioNote> = found.into_iter().map(|(_, _, n)| n).collect();
    let n = notes.len();
    let sequence: Vec<usize> = match direction {
        ArpeggioDirection::Up => (0..n).collect(),
        ArpeggioDirection::Down => (0..n).rev().collect(),
        ArpeggioDirection::UpDown => (0..n).chain((0..n.saturating_sub(1)).rev()).collect(),
    };
    Arpeggio {
        start: Fret::new(start).unwrap_or(Fret::OPEN),
        notes,
        sequence,
    }
}

/// The start frets that make a valid position: at or above the capo, at most 20, and whose
/// window holds every required chord tone (FB-7.1). Ascending.
///
/// ```
/// use mackey_theory::{Chord, ChordQuality};
/// use mackey_voicing::{arpeggio_positions, Instrument};
/// let am = Chord::new("A".parse().unwrap(), ChordQuality::Minor);
/// assert!(arpeggio_positions(&am, &Instrument::default()).contains(&5));
/// ```
#[must_use]
pub fn arpeggio_positions(chord: &Chord, instrument: &Instrument) -> Vec<u8> {
    let rules = Rules::new(*chord, instrument, &VoicingOptions::default());
    let first = instrument.capo.value();
    let last = 20.min(instrument.frets.value());
    let mut positions: Vec<u8> = (first..=last)
        .filter(|&s| window_has_required(*chord, instrument, s, rules.must))
        .collect();
    positions.dedup();
    positions
}

fn window_has_required(chord: Chord, instrument: &Instrument, start: u8, must: u16) -> bool {
    let capo = instrument.capo.value();
    let lo = start.saturating_sub(1).max(capo);
    let hi = start.saturating_add(4).min(instrument.frets.value());
    let mut seen = 0u16;
    let pcs = chord_pc_set(chord);
    for index in 0..6 {
        for fret in lo..=hi {
            let midi = instrument.midi_at_index(index, fret);
            if pcs.contains(mackey_theory::PitchClass::new(midi)) {
                seen |= 1u16 << midi.rem_euclid(12);
            }
        }
    }
    seen & must == must
}

/// The default position (FB-7.4): the valid position that holds the selected voicing's pressed
/// notes (closest to its lowest fret); otherwise the lowest valid position that holds the root
/// on string 6 or 5. `None` only when the chord has no valid position at all.
#[must_use]
pub fn default_arpeggio_position(
    chord: &Chord,
    instrument: &Instrument,
    voicing: Option<&Voicing>,
) -> Option<u8> {
    let positions = arpeggio_positions(chord, instrument);
    if let Some(v) = voicing {
        let pressed: Vec<u8> = v.strings.iter().filter_map(|s| s.fretted()).collect();
        if let (Some(&lo), Some(&hi)) = (pressed.iter().min(), pressed.iter().max()) {
            let mut candidates: Vec<u8> = positions
                .iter()
                .copied()
                .filter(|&s| s <= lo + 1 && s + 4 >= hi)
                .collect();
            candidates.sort_by_key(|&s| (u8::abs_diff(s, lo), s));
            if let Some(best) = candidates.first() {
                return Some(*best);
            }
        } else if v.strings.iter().any(|s| matches!(s, StringState::Open))
            && let Some(first) = positions.first()
        {
            return Some(*first);
        }
    }
    let root = chord.root.pitch_class();
    positions.into_iter().find(|&s| {
        let capo = instrument.capo.value();
        let lo = s.saturating_sub(1).max(capo);
        let hi = s.saturating_add(4).min(instrument.frets.value());
        [0usize, 1]
            .iter()
            .any(|&index| (lo..=hi).any(|f| instrument.pc_at_index(index, f) == root))
    })
}
