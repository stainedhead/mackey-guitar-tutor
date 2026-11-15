//! Playback plans: when each note of a strum, arpeggio or scale starts, built from plain MIDI
//! note numbers. The app turns a chord voicing or scale pattern into the list; this module
//! owns the timing, so strum spacing and tempo maths are tested here.

use mackey_theory::{AccidentalPreference, Pitch, ReferencePitch};

use super::synth::SynthCommand;
use crate::error::AudioError;

/// Gap between the strings of a strum, low to high.
pub const STRUM_GAP_MS: f64 = 30.0;
/// How long the last note is left to ring in [`PlaybackPlan::duration_ms`].
pub const RING_MS: f64 = 1500.0;
/// Accepted tempo range for arpeggios and scales.
const BPM_RANGE: std::ops::RangeInclusive<f64> = 20.0..=300.0;

/// One note of a plan.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TimedNote {
    /// MIDI note number, `0..=127`.
    pub midi: u8,
    /// Start relative to the plan's start, in milliseconds.
    pub start_ms: f64,
    /// Pluck strength `0.0..=1.0`.
    pub velocity: f32,
}

/// A timed list of notes.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaybackPlan {
    /// The notes in start order.
    pub notes: Vec<TimedNote>,
    /// Start of the last note plus the ring time; 0 for an empty plan.
    pub duration_ms: f64,
}

fn check(midis: &[u8]) -> Result<(), AudioError> {
    match midis.iter().find(|&&m| m > 127) {
        Some(m) => Err(AudioError::InvalidConfig(format!(
            "MIDI note {m} is above 127"
        ))),
        None => Ok(()),
    }
}

fn eighth_ms(bpm: f64) -> Result<f64, AudioError> {
    if BPM_RANGE.contains(&bpm) {
        Ok(30_000.0 / bpm)
    } else {
        Err(AudioError::InvalidConfig(format!(
            "tempo {bpm} is outside 20 to 300 bpm"
        )))
    }
}

impl PlaybackPlan {
    fn from_steps(midis: impl Iterator<Item = u8>, step_ms: f64, velocity: f32) -> Self {
        let notes: Vec<TimedNote> = midis
            .enumerate()
            .map(|(i, midi)| TimedNote {
                midi,
                start_ms: i as f64 * step_ms,
                velocity,
            })
            .collect();
        let duration_ms = notes.last().map_or(0.0, |n| n.start_ms + RING_MS);
        Self { notes, duration_ms }
    }

    /// A strum: the notes in the order given (strings low to high), 30 ms apart.
    ///
    /// # Errors
    /// [`AudioError::InvalidConfig`] for a MIDI number above 127.
    ///
    /// ```
    /// use mackey_audio::dsp::PlaybackPlan;
    /// let plan = PlaybackPlan::strum(&[40, 47, 52]).unwrap();
    /// let starts: Vec<f64> = plan.notes.iter().map(|n| n.start_ms).collect();
    /// assert_eq!(starts, [0.0, 30.0, 60.0]);
    /// ```
    pub fn strum(midis: &[u8]) -> Result<Self, AudioError> {
        check(midis)?;
        Ok(Self::from_steps(midis.iter().copied(), STRUM_GAP_MS, 0.8))
    }

    /// An arpeggio: the notes in the order given, one eighth note apart at `bpm`.
    ///
    /// # Errors
    /// [`AudioError::InvalidConfig`] for a MIDI number above 127 or a tempo outside 20 to 300.
    ///
    /// ```
    /// use mackey_audio::dsp::PlaybackPlan;
    /// let plan = PlaybackPlan::arpeggio(&[48, 52, 55], 120.0).unwrap();
    /// assert_eq!(plan.notes[1].start_ms, 250.0);
    /// ```
    pub fn arpeggio(midis: &[u8], bpm: f64) -> Result<Self, AudioError> {
        check(midis)?;
        Ok(Self::from_steps(
            midis.iter().copied(),
            eighth_ms(bpm)?,
            0.7,
        ))
    }

    /// A scale played ascending (the notes are sorted low to high), one eighth note apart.
    ///
    /// # Errors
    /// As [`PlaybackPlan::arpeggio`].
    ///
    /// ```
    /// use mackey_audio::dsp::PlaybackPlan;
    /// let plan = PlaybackPlan::scale_ascending(&[55, 48, 52], 60.0).unwrap();
    /// let midis: Vec<u8> = plan.notes.iter().map(|n| n.midi).collect();
    /// assert_eq!(midis, [48, 52, 55]);
    /// ```
    pub fn scale_ascending(midis: &[u8], bpm: f64) -> Result<Self, AudioError> {
        check(midis)?;
        let step = eighth_ms(bpm)?;
        let mut sorted = midis.to_vec();
        sorted.sort_unstable();
        Ok(Self::from_steps(sorted.into_iter(), step, 0.7))
    }

    /// The synth commands for this plan: one `NoteOn` per note with ids `first_id`,
    /// `first_id + 1`, ..., delays in samples at `sample_rate`, pitches from `a4`.
    #[must_use]
    pub fn commands(
        &self,
        sample_rate: u32,
        a4: ReferencePitch,
        first_id: u32,
    ) -> Vec<SynthCommand> {
        self.notes
            .iter()
            .enumerate()
            .filter_map(|(i, n)| {
                let pitch =
                    Pitch::from_midi(i32::from(n.midi), AccidentalPreference::Sharps).ok()?;
                Some(SynthCommand::NoteOn {
                    id: first_id.wrapping_add(i as u32),
                    frequency_hz: pitch.frequency(a4) as f32,
                    velocity: n.velocity,
                    delay_samples: (n.start_ms * f64::from(sample_rate) / 1000.0).round() as u32,
                })
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn starts(p: &PlaybackPlan) -> Vec<f64> {
        p.notes.iter().map(|n| n.start_ms).collect()
    }

    #[test]
    fn strum_is_30_ms_apart_in_given_order() {
        let p = PlaybackPlan::strum(&[40, 45, 50, 55, 59, 64]).unwrap();
        assert_eq!(starts(&p), [0.0, 30.0, 60.0, 90.0, 120.0, 150.0]);
        assert_eq!(
            p.notes.iter().map(|n| n.midi).collect::<Vec<_>>(),
            [40, 45, 50, 55, 59, 64]
        );
        assert_eq!(p.duration_ms, 150.0 + RING_MS);
    }

    #[test]
    fn arpeggio_eighths_at_tempo() {
        // (bpm, eighth in ms)
        const CASES: &[(f64, f64)] =
            &[(60.0, 500.0), (120.0, 250.0), (90.0, 333.333_333_333_333_3)];
        for &(bpm, eighth) in CASES {
            let p = PlaybackPlan::arpeggio(&[48, 52, 55, 60], bpm).unwrap();
            for (i, n) in p.notes.iter().enumerate() {
                assert!(
                    (n.start_ms - i as f64 * eighth).abs() < 1e-9,
                    "{bpm} bpm note {i}"
                );
            }
        }
    }

    #[test]
    fn scale_is_sorted_ascending() {
        let p = PlaybackPlan::scale_ascending(&[64, 60, 62, 67], 120.0).unwrap();
        assert_eq!(
            p.notes.iter().map(|n| n.midi).collect::<Vec<_>>(),
            [60, 62, 64, 67]
        );
        assert_eq!(starts(&p), [0.0, 250.0, 500.0, 750.0]);
    }

    #[test]
    fn empty_plan_is_empty() {
        let p = PlaybackPlan::strum(&[]).unwrap();
        assert!(p.notes.is_empty());
        assert_eq!(p.duration_ms, 0.0);
        assert!(p.commands(48_000, ReferencePitch::default(), 0).is_empty());
    }

    #[test]
    fn invalid_input_is_an_error() {
        assert!(PlaybackPlan::strum(&[128]).is_err());
        assert!(PlaybackPlan::arpeggio(&[60], 10.0).is_err());
        assert!(PlaybackPlan::arpeggio(&[60], f64::NAN).is_err());
        assert!(PlaybackPlan::scale_ascending(&[200], 100.0).is_err());
        assert!(PlaybackPlan::scale_ascending(&[60], 400.0).is_err());
    }

    #[test]
    fn commands_carry_ids_delays_and_frequencies() {
        let p = PlaybackPlan::strum(&[45, 57]).unwrap();
        let cmds = p.commands(48_000, ReferencePitch::default(), 10);
        match (cmds[0], cmds[1]) {
            (
                SynthCommand::NoteOn {
                    id: 10,
                    frequency_hz: f0,
                    delay_samples: 0,
                    ..
                },
                SynthCommand::NoteOn {
                    id: 11,
                    frequency_hz: f1,
                    delay_samples: 1440,
                    ..
                },
            ) => {
                assert!((f0 - 110.0).abs() < 1e-3 && (f1 - 220.0).abs() < 1e-3);
            }
            other => panic!("{other:?}"),
        }
        // A4 = 442 raises the pitch.
        let hi = p.commands(48_000, ReferencePitch::new(442.0).unwrap(), 0);
        assert!(matches!(hi[0], SynthCommand::NoteOn { frequency_hz, .. } if frequency_hz > 110.4));
    }
}
