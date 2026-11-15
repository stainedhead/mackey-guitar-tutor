//! Synthetic signal generators and a block feeder shared by the integration tests.
//!
//! Each test file uses a different subset, so unused items are expected.
#![allow(
    dead_code,
    reason = "each integration test file uses a different subset"
)]

use std::f64::consts::PI;

use mackey_audio::dsp::{DetectorConfig, PitchDetector, Tuner, TunerConfig, TunerReading};

/// Seeded white-noise source (xorshift), uniform in `-1.0..1.0`.
pub(crate) struct Noise(u32);

impl Noise {
    pub(crate) fn new(seed: u32) -> Self {
        Self(seed.max(1))
    }

    pub(crate) fn next(&mut self) -> f32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        (self.0 >> 8) as f32 / 8_388_608.0 - 1.0
    }
}

/// A pure sine of `amp` at `hz`.
pub(crate) fn sine(hz: f64, sr: u32, n: usize, amp: f32) -> Vec<f32> {
    (0..n)
        .map(|i| amp * (2.0 * PI * hz * i as f64 / f64::from(sr)).sin() as f32)
        .collect()
}

/// A guitar-like tone: fundamental plus six harmonics at 1/n amplitude with a little
/// inharmonicity (`f_n = n f0 sqrt(1 + B n^2)`, B = 1e-4), exponential decay (time constant
/// 1.5 s) and white noise at -50 dBFS RMS.
pub(crate) fn guitar_tone(f0: f64, sr: u32, n: usize, amp: f32, seed: u32) -> Vec<f32> {
    let mut noise = Noise::new(seed);
    // Uniform noise of amplitude A has RMS A / sqrt(3); -50 dBFS RMS is 0.003162.
    let noise_amp = 0.003_162_f32 * 3.0_f32.sqrt();
    (0..n)
        .map(|i| {
            let t = i as f64 / f64::from(sr);
            let mut v = 0.0_f64;
            for h in 1..=6_u32 {
                let hn = f64::from(h);
                let f = hn * f0 * (1.0 + 1.0e-4 * hn * hn).sqrt();
                v += (2.0 * PI * f * t).sin() / hn;
            }
            let env = (-t / 1.5).exp();
            amp * (v * env / 2.0) as f32 + noise_amp * noise.next()
        })
        .collect()
}

/// `hz` moved by `cents`.
pub(crate) fn detuned(hz: f64, cents: f64) -> f64 {
    hz * 2f64.powf(cents / 1200.0)
}

/// A tuner for `sr` with the default detector.
pub(crate) fn tuner(cfg: TunerConfig, sr: u32) -> Tuner {
    Tuner::new(
        cfg,
        PitchDetector::new(DetectorConfig::new(sr).unwrap()).unwrap(),
    )
}

/// A reading with the time at which the block that produced it ended.
pub(crate) struct Fed {
    pub(crate) reading: TunerReading,
    pub(crate) delivered_ms: f64,
}

/// Feeds `signal` in blocks of `block` samples and records when each reading was delivered
/// (the end of its block), as a device with that buffer size would.
pub(crate) fn feed(tuner: &mut Tuner, signal: &[f32], block: usize, sr: u32) -> Vec<Fed> {
    let mut out = Vec::new();
    let mut fed = 0usize;
    for chunk in signal.chunks(block) {
        fed += chunk.len();
        let delivered_ms = fed as f64 * 1000.0 / f64::from(sr);
        out.extend(tuner.push_block(chunk).into_iter().map(|reading| Fed {
            reading,
            delivered_ms,
        }));
    }
    out
}
