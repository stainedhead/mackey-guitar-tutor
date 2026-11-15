//! Input level: RMS, peak and clipping.

use serde::{Deserialize, Serialize};

/// Level of one block of samples, linear `0.0..=1.0` (full scale = 1.0).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Level {
    /// Root-mean-square amplitude.
    pub rms: f32,
    /// Largest absolute sample.
    pub peak: f32,
    /// True when the peak stayed at or above 0.99 for at least two consecutive samples.
    pub clipping: bool,
}

impl Level {
    /// Silence.
    pub const SILENT: Self = Self {
        rms: 0.0,
        peak: 0.0,
        clipping: false,
    };
}

/// Measures [`Level`]s. Stateless apart from the clipping run length, which carries across
/// block boundaries.
#[derive(Debug, Default, Clone)]
pub struct LevelMeter {
    run: u32,
}

/// Peak at or above this counts as a clipped sample.
const CLIP_THRESHOLD: f32 = 0.99;

impl LevelMeter {
    /// A fresh meter.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Measures `block`. An empty block is silence; non-finite samples count as zero.
    ///
    /// ```
    /// use mackey_audio::dsp::LevelMeter;
    /// let mut m = LevelMeter::new();
    /// let l = m.process(&[0.5, -0.5, 0.5, -0.5]);
    /// assert!((l.rms - 0.5).abs() < 1e-6 && !l.clipping);
    /// ```
    pub fn process(&mut self, block: &[f32]) -> Level {
        if block.is_empty() {
            return Level::SILENT;
        }
        let mut sum = 0.0_f64;
        let mut peak = 0.0_f32;
        let mut clipping = false;
        for &raw in block {
            let s = if raw.is_finite() { raw } else { 0.0 };
            let a = s.abs();
            sum += f64::from(s) * f64::from(s);
            peak = peak.max(a);
            if a >= CLIP_THRESHOLD {
                self.run += 1;
                clipping |= self.run >= 2;
            } else {
                self.run = 0;
            }
        }
        Level {
            rms: (sum / block.len() as f64).sqrt() as f32,
            peak: peak.min(1.0),
            clipping,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_silent() {
        assert_eq!(LevelMeter::new().process(&[]), Level::SILENT);
    }

    #[test]
    fn rms_and_peak() {
        let l = LevelMeter::new().process(&[0.0, 1.0, 0.0, -0.5]);
        assert!((l.peak - 1.0).abs() < 1e-6);
        assert!((l.rms - (1.25f32 / 4.0).sqrt()).abs() < 1e-6);
    }

    #[test]
    fn clipping_needs_two_in_a_row() {
        let mut m = LevelMeter::new();
        assert!(!m.process(&[1.0, 0.0, 1.0, 0.0]).clipping);
        assert!(m.process(&[0.0, 1.0, -1.0]).clipping);
    }

    #[test]
    fn clipping_run_spans_blocks() {
        let mut m = LevelMeter::new();
        assert!(!m.process(&[0.1, 1.0]).clipping);
        assert!(m.process(&[1.0, 0.1]).clipping);
        assert!(!m.process(&[0.1, 0.1]).clipping);
    }

    #[test]
    fn non_finite_is_zero() {
        let l = LevelMeter::new().process(&[f32::NAN, f32::INFINITY, 0.5]);
        assert!(l.rms.is_finite() && (l.peak - 0.5).abs() < 1e-6);
    }
}
