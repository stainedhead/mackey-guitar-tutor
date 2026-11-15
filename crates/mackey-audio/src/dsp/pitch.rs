//! `McLeod` Pitch Method (MPM) detector.
//!
//! The normalised square difference function (NSDF) of a window is computed directly (about
//! 1.6 million multiply-adds for a 2048-sample window at 48 kHz, well under the 2 ms budget when
//! optimised), its key maxima are picked, the first one above `k` times the highest wins and
//! the peak is refined with a parabola for sub-sample accuracy. The NSDF is amplitude
//! normalised, so a decaying string does not bias the result.

use serde::{Deserialize, Serialize};

use crate::error::AudioError;

/// Detector settings. [`DetectorConfig::new`] gives the defaults for a sample rate.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct DetectorConfig {
    /// Samples per second.
    pub sample_rate: u32,
    /// Analysis window in samples.
    pub window: usize,
    /// New samples between two analyses.
    pub hop: usize,
    /// Lowest frequency reported (Hz).
    pub min_hz: f32,
    /// Highest frequency reported (Hz).
    pub max_hz: f32,
    /// Clarity (NSDF peak height) a detection needs to count as a pitch.
    pub clarity_threshold: f32,
    /// MPM peak-pick constant: the first key maximum above `k * highest` is the period.
    pub k: f32,
}

impl DetectorConfig {
    /// Defaults for `sample_rate`: 2048-sample window (4096 above 51.2 kHz so the window still
    /// holds about two periods of 60 Hz), hop a quarter of the window (about 94 analyses per
    /// second at 48 kHz), 60 to 1,400 Hz, clarity 0.80, k 0.93.
    ///
    /// # Errors
    /// [`AudioError::InvalidConfig`] when the rate is outside 22,050 to 192,000 Hz.
    ///
    /// ```
    /// use mackey_audio::dsp::DetectorConfig;
    /// let cfg = DetectorConfig::new(48_000).unwrap();
    /// assert_eq!((cfg.window, cfg.hop), (2048, 512));
    /// assert!(DetectorConfig::new(8_000).is_err());
    /// ```
    pub fn new(sample_rate: u32) -> Result<Self, AudioError> {
        let window = if sample_rate > 51_200 { 4096 } else { 2048 };
        let cfg = Self {
            sample_rate,
            window,
            hop: window / 4,
            min_hz: 60.0,
            max_hz: 1400.0,
            clarity_threshold: 0.80,
            k: 0.93,
        };
        cfg.validate()?;
        Ok(cfg)
    }

    fn validate(&self) -> Result<(), AudioError> {
        let bad = |what: &str| Err(AudioError::InvalidConfig(what.to_owned()));
        if !(22_050..=192_000).contains(&self.sample_rate) {
            return bad("sample rate must be 22,050 to 192,000 Hz");
        }
        if self.window < 512 || self.hop == 0 || self.hop > self.window {
            return bad("window must be at least 512 and hop within 1..=window");
        }
        if !(self.min_hz > 0.0 && self.min_hz < self.max_hz) {
            return bad("frequency range must satisfy 0 < min < max");
        }
        let unit = |v: f32| v > 0.0 && v <= 1.0;
        if !unit(self.clarity_threshold) || !unit(self.k) {
            return bad("clarity threshold and k must be in (0, 1]");
        }
        if (self.window as f32) < 1.5 * self.sample_rate as f32 / self.min_hz {
            return bad("window must hold at least 1.5 periods of the lowest frequency");
        }
        Ok(())
    }
}

/// One detection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PitchEstimate {
    /// Fundamental frequency in Hz.
    pub frequency_hz: f64,
    /// NSDF peak height, `0.0..=1.0`; near 1.0 for a clean periodic tone.
    pub clarity: f32,
}

/// The detector. Owns its scratch buffers, allocated once in [`PitchDetector::new`].
#[derive(Debug, Clone)]
pub struct PitchDetector {
    cfg: DetectorConfig,
    scratch: Vec<f32>,
    nsdf: Vec<f32>,
    peaks: Vec<(usize, f32)>,
    min_tau: usize,
    max_tau: usize,
}

impl PitchDetector {
    /// Builds a detector.
    ///
    /// # Errors
    /// [`AudioError::InvalidConfig`] when `cfg` is out of range.
    pub fn new(cfg: DetectorConfig) -> Result<Self, AudioError> {
        cfg.validate()?;
        let sr = cfg.sample_rate as f32;
        let min_tau = ((sr / cfg.max_hz).floor() as usize)
            .saturating_sub(1)
            .max(2);
        let max_tau = (((sr / cfg.min_hz).ceil() as usize) + 2).min(cfg.window - 3);
        Ok(Self {
            cfg,
            scratch: vec![0.0; cfg.window],
            nsdf: vec![0.0; max_tau + 2],
            peaks: Vec::with_capacity(max_tau),
            min_tau,
            max_tau,
        })
    }

    /// The configuration in use.
    #[must_use]
    pub fn config(&self) -> &DetectorConfig {
        &self.cfg
    }

    /// Detects a pitch in the last `window` samples of `samples`: `None` for fewer samples than
    /// that, non-finite input, silence, or a clarity below the threshold.
    pub fn detect(&mut self, samples: &[f32]) -> Option<PitchEstimate> {
        let threshold = self.cfg.clarity_threshold;
        self.analyse(samples).filter(|e| e.clarity >= threshold)
    }

    /// Like [`PitchDetector::detect`] but returns the best candidate whatever its clarity (still
    /// `None` when there is no key maximum in range). The tuner uses the clarity to tell a
    /// clean string from two strings ringing together.
    pub fn analyse(&mut self, samples: &[f32]) -> Option<PitchEstimate> {
        let len = self.cfg.window;
        if samples.len() < len {
            return None;
        }
        let recent = &samples[samples.len() - len..];
        if recent.iter().any(|s| !s.is_finite()) {
            return None;
        }
        let mean = recent.iter().map(|&s| f64::from(s)).sum::<f64>() / len as f64;
        let mean = mean as f32;
        for (dst, &src) in self.scratch.iter_mut().zip(recent) {
            *dst = src - mean;
        }
        let energy: f64 = self
            .scratch
            .iter()
            .map(|&s| f64::from(s) * f64::from(s))
            .sum();
        // Below -100 dBFS RMS there is nothing to analyse; a non-finite energy means the
        // samples were absurdly large (the mean or the sum overflowed).
        if !energy.is_finite() || energy / (len as f64) < 1e-10 {
            return None;
        }
        // The NSDF is scale free: normalise to a peak of 1 so f32 products cannot overflow.
        let peak = self.scratch.iter().fold(0.0_f32, |acc, s| acc.max(s.abs()));
        let inv = 1.0 / peak;
        for s in &mut self.scratch {
            *s *= inv;
        }
        let energy: f64 = self
            .scratch
            .iter()
            .map(|&s| f64::from(s) * f64::from(s))
            .sum();
        self.compute_nsdf(energy);
        let (tau, clarity) = self.pick_peak()?;
        // Parabola through the peak and its neighbours: sub-sample period and refined height.
        let (before, at, after) = (self.nsdf[tau - 1], self.nsdf[tau], self.nsdf[tau + 1]);
        let curvature = before - 2.0 * at + after;
        let (delta, height) = if curvature < 0.0 {
            let offset = 0.5 * (before - after) / curvature;
            (offset, at - 0.25 * (before - after) * offset)
        } else {
            (0.0, clarity)
        };
        let period = tau as f32 + delta;
        let frequency_hz = f64::from(self.cfg.sample_rate) / f64::from(period);
        if !(f64::from(self.cfg.min_hz)..=f64::from(self.cfg.max_hz)).contains(&frequency_hz) {
            return None;
        }
        Some(PitchEstimate {
            frequency_hz,
            clarity: height.clamp(0.0, 1.0),
        })
    }

    fn compute_nsdf(&mut self, energy: f64) {
        let x = &self.scratch;
        let len = x.len();
        let mut norm = 2.0 * energy;
        self.nsdf[0] = 1.0;
        for tau in 1..self.nsdf.len() {
            norm -= f64::from(x[tau - 1]).powi(2) + f64::from(x[len - tau]).powi(2);
            let corr = dot(&x[..len - tau], &x[tau..]);
            self.nsdf[tau] = if norm > 1e-12 {
                (2.0 * corr / norm) as f32
            } else {
                0.0
            };
        }
    }

    /// First key maximum within `min_tau..=max_tau` that reaches `k * highest`.
    fn pick_peak(&mut self) -> Option<(usize, f32)> {
        self.peaks.clear();
        let last = self.max_tau;
        let nsdf = &self.nsdf;
        // Written as a closure so a NaN counts as "not positive" and can never stall a scan.
        let positive = |v: f32| v > 0.0;
        let mut pos = 1;
        while pos <= last && positive(nsdf[pos]) {
            pos += 1;
        }
        while pos <= last {
            while pos <= last && !positive(nsdf[pos]) {
                pos += 1;
            }
            let mut best = pos;
            while pos <= last && positive(nsdf[pos]) {
                if nsdf[pos] > nsdf[best] {
                    best = pos;
                }
                pos += 1;
            }
            if best >= self.min_tau && best <= last && positive(nsdf[best]) {
                self.peaks.push((best, nsdf[best]));
            }
        }
        let highest = self.peaks.iter().map(|p| p.1).fold(f32::MIN, f32::max);
        let limit = self.cfg.k * highest;
        self.peaks.iter().copied().find(|p| p.1 >= limit)
    }
}

/// Dot product with eight independent accumulators so the compiler vectorises it.
fn dot(lhs: &[f32], rhs: &[f32]) -> f64 {
    let mut lanes = [0.0_f32; 8];
    let (lhs_chunks, lhs_rest) = lhs.as_chunks::<8>();
    let (rhs_chunks, rhs_rest) = rhs.as_chunks::<8>();
    let rest: f32 = lhs_rest.iter().zip(rhs_rest).map(|(x, y)| x * y).sum();
    for (xs, ys) in lhs_chunks.iter().zip(rhs_chunks) {
        for ((lane, x), y) in lanes.iter_mut().zip(xs).zip(ys) {
            *lane += x * y;
        }
    }
    f64::from(lanes.iter().sum::<f32>() + rest)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine(f: f64, sr: u32, n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| {
                (0.5 * (2.0 * std::f64::consts::PI * f * i as f64 / f64::from(sr)).sin()) as f32
            })
            .collect()
    }

    fn det(sr: u32) -> PitchDetector {
        PitchDetector::new(DetectorConfig::new(sr).unwrap()).unwrap()
    }

    #[test]
    fn config_defaults_and_validation() {
        let c = DetectorConfig::new(44_100).unwrap();
        assert_eq!((c.window, c.hop), (2048, 512));
        assert_eq!(DetectorConfig::new(96_000).unwrap().window, 4096);
        assert!(DetectorConfig::new(10_000).is_err());
        let ok = DetectorConfig::new(48_000).unwrap();
        let cases = [
            ("hop zero", DetectorConfig { hop: 0, ..ok }),
            ("hop big", DetectorConfig { hop: 4096, ..ok }),
            (
                "window small",
                DetectorConfig {
                    window: 256,
                    hop: 64,
                    ..ok
                },
            ),
            (
                "range",
                DetectorConfig {
                    min_hz: 2000.0,
                    ..ok
                },
            ),
            (
                "clarity",
                DetectorConfig {
                    clarity_threshold: 0.0,
                    ..ok
                },
            ),
            ("k", DetectorConfig { k: 1.5, ..ok }),
            (
                "window vs 60 Hz",
                DetectorConfig {
                    window: 1024,
                    hop: 256,
                    ..ok
                },
            ),
        ];
        for (label, cfg) in cases {
            assert!(PitchDetector::new(cfg).is_err(), "{label}");
        }
    }

    #[test]
    fn detects_a440_at_both_rates() {
        for sr in [44_100, 48_000] {
            let mut d = det(sr);
            let e = d.detect(&sine(440.0, sr, 2048)).unwrap();
            assert!(
                (e.frequency_hz - 440.0).abs() < 0.2,
                "{sr}: {}",
                e.frequency_hz
            );
            assert!(e.clarity > 0.95);
            assert_eq!(d.config().sample_rate, sr);
        }
    }

    #[test]
    fn uses_the_last_window_of_a_longer_slice() {
        let mut d = det(48_000);
        let mut s = sine(300.0, 48_000, 1000);
        s.extend(sine(440.0, 48_000, 2048));
        let e = d.detect(&s).unwrap();
        assert!((e.frequency_hz - 440.0).abs() < 0.3);
    }

    #[test]
    fn dc_offset_is_ignored() {
        let mut d = det(48_000);
        let s: Vec<f32> = sine(220.0, 48_000, 2048).iter().map(|v| v + 0.3).collect();
        assert!((d.detect(&s).unwrap().frequency_hz - 220.0).abs() < 0.2);
    }

    #[test]
    fn rejects_bad_input() {
        let mut d = det(48_000);
        assert!(d.detect(&[]).is_none());
        assert!(d.detect(&[0.1; 100]).is_none(), "too short");
        assert!(d.detect(&[0.0; 2048]).is_none(), "silence");
        let mut nan = sine(440.0, 48_000, 2048);
        nan[7] = f32::NAN;
        assert!(d.detect(&nan).is_none(), "nan");
        nan[7] = f32::INFINITY;
        assert!(d.analyse(&nan).is_none(), "inf");
    }

    #[test]
    fn huge_samples_neither_hang_nor_overflow() {
        // Regression: f32 products of values near f32::MAX overflowed to NaN, which stalled
        // the peak scan forever.
        let mut d = det(48_000);
        let huge: Vec<f32> = (0..2048)
            .map(|i| if i % 2 == 0 { 3.0e38 } else { -3.0e38 })
            .collect();
        let _ = d.detect(&huge);
        // A large but finite sine is normalised and still read correctly.
        let big: Vec<f32> = sine(440.0, 48_000, 2048)
            .iter()
            .map(|v| v * 1.0e30)
            .collect();
        assert!((d.detect(&big).unwrap().frequency_hz - 440.0).abs() < 0.3);
    }

    #[test]
    fn out_of_range_tones_are_not_reported() {
        let mut d = det(48_000);
        // 40 Hz has a period beyond the search range; no in-range peak reaches the threshold.
        assert!(d.detect(&sine(40.0, 48_000, 2048)).is_none());
    }

    #[test]
    fn noise_has_low_clarity() {
        let mut d = det(48_000);
        let mut state = 12345_u32;
        let noise: Vec<f32> = (0..2048)
            .map(|_| {
                state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                (state >> 8) as f32 / 8_388_608.0 - 1.0
            })
            .collect();
        assert!(d.detect(&noise).is_none());
    }

    #[test]
    fn range_edges() {
        let mut d = det(48_000);
        for f in [65.0, 1300.0] {
            let e = d.detect(&sine(f, 48_000, 2048)).unwrap();
            assert!(
                (e.frequency_hz - f).abs() / f < 0.002,
                "{f}: {}",
                e.frequency_hz
            );
        }
    }
}
