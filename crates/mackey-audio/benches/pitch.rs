//! Cost of one pitch-detector window (2048 samples), the budget behind the tuner's 94
//! analyses per second: at most 2 ms per window so the analysis thread keeps up (task C8).

#![allow(
    clippy::unwrap_used,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    reason = "benchmark setup may unwrap; signal maths casts freely"
)]

use std::f64::consts::PI;
use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use mackey_audio::dsp::{DetectorConfig, PitchDetector, Tuner, TunerConfig};
use mackey_theory::ReferencePitch;

fn tone(hz: f64, sr: u32, n: usize) -> Vec<f32> {
    (0..n)
        .map(|i| {
            let t = i as f64 / f64::from(sr);
            let v: f64 = (1..=6)
                .map(|h| (2.0 * PI * hz * f64::from(h) * t).sin() / f64::from(h))
                .sum();
            (0.3 * v) as f32
        })
        .collect()
}

fn bench_detector(c: &mut Criterion) {
    for sr in [44_100_u32, 48_000] {
        let mut detector = PitchDetector::new(DetectorConfig::new(sr).unwrap()).unwrap();
        // E2 has the longest period to search; E4 the shortest in normal use.
        for (name, hz) in [("E2", 82.41), ("E4", 329.63)] {
            let window = tone(hz, sr, 2048);
            c.bench_function(&format!("detect {name} 2048 @ {sr}"), |b| {
                b.iter(|| black_box(detector.detect(black_box(&window))));
            });
        }
    }
}

fn bench_tuner(c: &mut Criterion) {
    let sr = 48_000;
    let signal = tone(110.0, sr, 2048 + 512);
    c.bench_function("tuner push 2560 samples (2 readings)", |b| {
        b.iter(|| {
            let detector = PitchDetector::new(DetectorConfig::new(sr).unwrap()).unwrap();
            let mut tuner = Tuner::new(TunerConfig::chromatic(ReferencePitch::default()), detector);
            black_box(tuner.push_block(black_box(&signal)))
        });
    });
}

criterion_group!(benches, bench_detector, bench_tuner);
criterion_main!(benches);
