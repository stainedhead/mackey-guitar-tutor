//! The constraints every shown voicing must meet (FB-5.4) and its score (D-11).

use mackey_theory::{Chord, ChordQuality};

use crate::fingering::{assign_fingering, finger_count};
use crate::instrument::{Capo, Fret, Instrument};
use crate::voicing::{
    Fingering, Origin, StringState, Voicing, VoicingGroup, VoicingKind, VoicingOptions,
};

/// Search scoring weights in one table (D-11, D-50). Higher scores are better.
pub(crate) mod weights {
    /// Per fret of span.
    pub(crate) const SPAN: i32 = -3;
    /// Per finger used (a barre counts once).
    pub(crate) const FINGER: i32 = -2;
    /// Per open string when the hand stays within frets 0 to 4 of the capo.
    pub(crate) const OPEN_STRING: i32 = 2;
    /// Root in the bass.
    pub(crate) const ROOT_BASS: i32 = 4;
    /// Per muted string between two sounding strings.
    pub(crate) const SKIPPED: i32 = -12;
    /// Per muted string beyond two.
    pub(crate) const EXTRA_MUTE: i32 = -2;
    /// Per fret the lowest pressed fret is above 7.
    pub(crate) const HIGH_POSITION: i32 = -1;
    /// Per sounding string (fuller voicings first).
    pub(crate) const SOUNDING: i32 = 3;
    /// Bonus when every chord tone is present.
    pub(crate) const COMPLETE: i32 = 2;
    /// Curated shapes outrank searched ones inside a group.
    pub(crate) const CURATED: i32 = 50;
}

fn bit(midi: i32) -> u16 {
    1u16 << midi.rem_euclid(12)
}

/// Constraints derived from a chord, an instrument and the options.
#[derive(Debug, Clone)]
pub(crate) struct Rules {
    /// Pitch classes a sounding string may carry (chord tones and the slash bass).
    pub(crate) allowed: u16,
    /// Pitch classes a voicing must contain.
    pub(crate) must: u16,
    /// All chord tones (not the slash bass).
    pub(crate) all_tones: u16,
    /// The pitch class that must be lowest.
    pub(crate) bass: u8,
    pub(crate) root: u8,
    pub(crate) span_limit: u8,
    pub(crate) min_sounding: usize,
    pub(crate) capo: u8,
    pub(crate) frets: u8,
    pub(crate) tuning: [i32; 6],
}

impl Rules {
    pub(crate) fn new(chord: Chord, instrument: &Instrument, options: &VoicingOptions) -> Self {
        let quality = chord.quality;
        let root_pc = chord.root.pitch_class();
        let intervals = quality.intervals();
        let n = intervals.len();
        let fifth_defines = matches!(
            quality,
            ChordQuality::HalfDiminished7
                | ChordQuality::Diminished
                | ChordQuality::Diminished7
                | ChordQuality::Augmented
        );
        let mut all_tones = 0u16;
        let mut must = 0u16;
        for interval in intervals {
            let pc = interval.above(root_pc);
            let b = 1u16 << pc.value();
            all_tones |= b;
            let number = interval.number();
            let optional = quality != ChordQuality::Power
                && ((number == 5 && ((n >= 4 && !fifth_defines) || options.allow_omitted_fifth))
                    || (number == 9
                        && matches!(quality, ChordQuality::Dominant11 | ChordQuality::Dominant13)));
            if !optional {
                must |= b;
            }
        }
        let bass_pc = chord.bass_or_root().pitch_class();
        let allowed = all_tones | (1u16 << bass_pc.value());
        // The bass is both allowed and, when it is a chord tone already in `must`, kept.
        let span_limit = if options.allow_stretch { 5 } else { 4 };
        Self {
            allowed,
            must,
            all_tones,
            bass: bass_pc.value(),
            root: root_pc.value(),
            span_limit,
            min_sounding: if quality == ChordQuality::Power { 2 } else { 4 },
            capo: instrument.capo.value(),
            frets: instrument.frets.value(),
            tuning: instrument.tuning.midis(),
        }
    }

    pub(crate) fn sounding_open(&self, index: usize) -> i32 {
        self.tuning.get(index).copied().unwrap_or(0) + i32::from(self.capo)
    }

    /// Checks a candidate against FB-5.4 and scores it.
    pub(crate) fn evaluate(&self, strings: &[StringState; 6]) -> Option<Evaluated> {
        let mut sounded = 0u16;
        let mut first: Option<usize> = None;
        let mut last: Option<usize> = None;
        let mut min_fret: Option<u8> = None;
        let mut max_fret: Option<u8> = None;
        let mut opens = 0usize;
        let mut mutes = 0usize;
        let mut lowest: Option<i32> = None;
        for (i, state) in strings.iter().enumerate() {
            let midi = match state {
                StringState::Muted => {
                    mutes += 1;
                    continue;
                }
                StringState::Open => {
                    opens += 1;
                    self.sounding_open(i)
                }
                StringState::Fretted { fret } => {
                    let f = fret.value();
                    if f <= self.capo || f > self.frets {
                        return None;
                    }
                    min_fret = Some(min_fret.map_or(f, |m| m.min(f)));
                    max_fret = Some(max_fret.map_or(f, |m| m.max(f)));
                    self.tuning.get(i).copied().unwrap_or(0) + i32::from(f)
                }
            };
            let b = bit(midi);
            if b & self.allowed == 0 {
                return None;
            }
            if first.is_none() {
                first = Some(i);
            }
            lowest = Some(lowest.map_or(midi, |l: i32| l.min(midi)));
            last = Some(i);
            sounded |= b;
        }
        let (first, last) = (first?, last?);
        // The lowest sounding pitch must carry the bass (the root, or the slash bass).
        if u8::try_from(lowest?.rem_euclid(12)).ok()? != self.bass {
            return None;
        }
        let sounding = last - first + 1 - (mutes_between(strings, first, last));
        if sounding < self.min_sounding || sounded & self.must != self.must {
            return None;
        }
        let span = match (min_fret, max_fret) {
            (Some(lo), Some(hi)) => hi - lo,
            _ => 0,
        };
        if span > self.span_limit {
            return None;
        }
        let fingering = assign_fingering(strings, self.tuning)?;
        let skips = mutes_between(strings, first, last);
        let rel_max = max_fret.map_or(0, |m| m - self.capo);
        let rel_min = min_fret.map_or(0, |m| m - self.capo);
        let open_position = opens > 0 && rel_max <= 4;
        let mut score = 0i32;
        score += weights::SPAN * i32::from(span);
        score += weights::FINGER * i32::try_from(finger_count(&fingering)).unwrap_or(4);
        if open_position {
            score += weights::OPEN_STRING * i32::try_from(opens).unwrap_or(0);
        }
        if self.bass == self.root {
            score += weights::ROOT_BASS;
        }
        score += weights::SKIPPED * i32::try_from(skips).unwrap_or(0);
        score += weights::EXTRA_MUTE * i32::try_from(mutes.saturating_sub(2)).unwrap_or(0);
        score += weights::HIGH_POSITION * i32::from(rel_min.saturating_sub(7));
        score += weights::SOUNDING * i32::try_from(sounding).unwrap_or(0);
        if sounded & self.all_tones == self.all_tones {
            score += weights::COMPLETE;
        }
        Some(Evaluated {
            fingering,
            span,
            lowest_pressed: min_fret,
            open_position,
            score,
        })
    }
}

fn mutes_between(strings: &[StringState; 6], first: usize, last: usize) -> usize {
    strings
        .iter()
        .enumerate()
        .filter(|(i, s)| *i > first && *i < last && !s.is_sounding())
        .count()
}

/// A candidate that passed [`Rules::evaluate`].
#[derive(Debug, Clone)]
pub(crate) struct Evaluated {
    pub(crate) fingering: Fingering,
    pub(crate) span: u8,
    pub(crate) lowest_pressed: Option<u8>,
    pub(crate) open_position: bool,
    pub(crate) score: i32,
}

/// Picks the list group (FB-5.2): movable shapes first, then open position, then the rest.
pub(crate) fn group_of(kind: VoicingKind, open_position: bool) -> VoicingGroup {
    match kind {
        VoicingKind::Caged { .. } | VoicingKind::Barre => VoicingGroup::BarreAndCaged,
        _ if open_position => VoicingGroup::Open,
        VoicingKind::Open => VoicingGroup::Open,
        _ => VoicingGroup::Other,
    }
}

/// Assembles a finished [`Voicing`].
pub(crate) struct Build {
    pub(crate) chord: Chord,
    pub(crate) capo: Capo,
    pub(crate) strings: [StringState; 6],
    pub(crate) evaluated: Evaluated,
    pub(crate) kind: VoicingKind,
    pub(crate) origin: Origin,
    pub(crate) name: String,
    pub(crate) bonus: i32,
}

impl Build {
    pub(crate) fn finish(self) -> Voicing {
        let group = group_of(self.kind, self.evaluated.open_position);
        let lowest = if group == VoicingGroup::Open && self.evaluated.open_position {
            self.capo.value()
        } else {
            self.evaluated.lowest_pressed.unwrap_or(self.capo.value())
        };
        Voicing {
            chord: self.chord,
            strings: self.strings,
            fingering: self.evaluated.fingering,
            kind: self.kind,
            group,
            origin: self.origin,
            name: self.name,
            lowest_fret: Fret::new(lowest).unwrap_or(Fret::OPEN),
            span: self.evaluated.span,
            score: self.evaluated.score + self.bonus,
        }
    }
}
