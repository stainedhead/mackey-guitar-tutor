//! Detector and chromatic-tuner acceptance tests on synthetic signals (AC-TU-1.1, 1.2, 1.3,
//! 3.1, AC-NF-9, E-TU-6).

mod common;

use common::{detuned, feed, guitar_tone, sine, tuner};
use mackey_audio::dsp::{
    DetectorConfig, PitchDetector, TunerConfig, TunerMode, TunerStatus, nearest_pitch,
};
use mackey_theory::ReferencePitch;
use proptest::prelude::*;

const A4: f64 = 440.0;

fn chromatic() -> TunerConfig {
    TunerConfig::chromatic(ReferencePitch::default())
}

#[test]
fn ac_tu_1_1_a440_reads_a4_in_tune_within_400_ms() {
    // (sample rate, block size): device buffer sizes the app may see.
    const CASES: &[(u32, usize)] = &[
        (44_100, 256),
        (44_100, 512),
        (44_100, 1024),
        (48_000, 256),
        (48_000, 512),
        (48_000, 1024),
    ];
    for &(sr, block) in CASES {
        let mut t = tuner(chromatic(), sr);
        let readings = feed(&mut t, &sine(A4, sr, sr as usize, 0.5), block, sr);
        let first = readings
            .iter()
            .find(|f| f.reading.in_tune)
            .expect("in tune");
        let label = format!("{sr} Hz, block {block}");
        assert!(
            first.reading.t_ms <= 400.0,
            "{label}: reading at {} ms",
            first.reading.t_ms
        );
        assert!(
            first.delivered_ms <= 400.0,
            "{label}: delivered at {} ms",
            first.delivered_ms
        );
        let r = &first.reading;
        assert_eq!(r.note.unwrap().name(), "A", "{label}");
        assert_eq!(r.octave, Some(4), "{label}");
        assert!(r.cents.unwrap().abs() <= 1.0, "{label}: {:?}", r.cents);
    }
}

#[test]
fn ac_tu_1_2_open_strings_detuned_are_within_3_cents() {
    // Standard tuning E2 A2 D3 G3 B3 E4 (equal temperament, A4 = 440).
    const STRINGS: &[(&str, f64)] = &[
        ("E2", 82.4069),
        ("A2", 110.0),
        ("D3", 146.8324),
        ("G3", 195.9977),
        ("B3", 246.9417),
        ("E4", 329.6276),
    ];
    const DETUNE: &[f64] = &[-20.0, -5.0, 5.0, 20.0];
    for sr in [44_100_u32, 48_000] {
        for &(name, hz) in STRINGS {
            for &cents in DETUNE {
                let f = detuned(hz, cents);
                let sig = guitar_tone(f, sr, sr as usize / 2, 0.5, 7);
                let mut t = tuner(chromatic(), sr);
                let readings = feed(&mut t, &sig, 512, sr);
                let pitched: Vec<_> = readings
                    .iter()
                    .filter(|f| f.reading.status == TunerStatus::Pitch)
                    .collect();
                assert!(!pitched.is_empty(), "{name} {cents:+} at {sr}: no pitch");
                for p in pitched {
                    let got = f64::from(p.reading.cents.unwrap());
                    assert!(
                        (got - cents).abs() <= 3.0,
                        "{name} {cents:+} at {sr} Hz, t={:.0} ms: read {got:.2}",
                        p.reading.t_ms
                    );
                }
            }
        }
    }
}

#[test]
fn ac_tu_1_3_73_42_hz_is_d2() {
    for sr in [44_100_u32, 48_000] {
        let mut t = tuner(chromatic(), sr);
        let readings = feed(
            &mut t,
            &guitar_tone(73.42, sr, sr as usize / 2, 0.5, 3),
            512,
            sr,
        );
        let r = &readings.last().unwrap().reading;
        assert_eq!(r.status, TunerStatus::Pitch, "{sr}");
        let pitch = format!("{}{}", r.note.unwrap().name(), r.octave.unwrap());
        assert_eq!(pitch, "D2", "{sr}");
    }
}

#[test]
fn ac_tu_3_1_reference_442_reads_about_minus_8_cents() {
    let mut cfg = chromatic();
    cfg.a4 = ReferencePitch::new(442.0).unwrap();
    let mut t = tuner(cfg, 48_000);
    let r = feed(&mut t, &sine(440.0, 48_000, 9600, 0.5), 512, 48_000)
        .pop()
        .unwrap()
        .reading;
    assert_eq!(r.note.unwrap().name(), "A");
    assert!(
        (f64::from(r.cents.unwrap()) + 7.85).abs() < 1.0,
        "{:?}",
        r.cents
    );
}

#[test]
fn ac_nf_9_first_reading_within_150_ms_of_onset_and_20_per_second() {
    // The first reading is made when the first 2048 samples exist (42.7 ms at 48 kHz); a
    // note that starts in the middle of that window is recognised once about three periods
    // of it are inside (NSDF clarity needs that overlap), so low strings take a few hops more.
    const SR: u32 = 48_000;
    const ONSET_MS: f64 = 20.0;
    const CASES: &[(&str, f64)] = &[("E2", 82.41), ("A2", 110.0), ("E4", 329.63)];
    for &(name, hz) in CASES {
        let mut signal = vec![0.0_f32; (ONSET_MS * f64::from(SR) / 1000.0) as usize];
        signal.extend(guitar_tone(hz, SR, SR as usize, 0.5, 11));
        let mut t = tuner(chromatic(), SR);
        let readings = feed(&mut t, &signal, 512, SR);
        let first = readings
            .iter()
            .find(|f| f.reading.status == TunerStatus::Pitch)
            .unwrap_or_else(|| panic!("{name}: no pitch"));
        let latency = first.reading.t_ms - ONSET_MS;
        assert!(
            latency <= 150.0,
            "{name}: first pitch {latency:.1} ms after onset"
        );
        // At least 20 readings per second of tone (the hop gives about 94).
        let in_second = readings
            .iter()
            .filter(|f| f.reading.t_ms <= 1000.0 + ONSET_MS)
            .count();
        assert!(in_second >= 20, "{name}: {in_second} readings in 1 s");
    }
}

#[test]
fn e_tu_6_two_strings_never_give_a_wrong_in_tune_verdict() {
    // 110 Hz (A2) with E3 (164.81 Hz) at several balances.
    for (a, b) in [(0.3_f32, 0.3_f32), (0.4, 0.2), (0.2, 0.4), (0.3, 0.05)] {
        let lo = sine(110.0, 48_000, 48_000, a);
        let hi = sine(164.81, 48_000, 48_000, b);
        let mix: Vec<f32> = lo.iter().zip(&hi).map(|(x, y)| x + y).collect();
        let mut t = tuner(chromatic(), 48_000);
        for f in feed(&mut t, &mix, 512, 48_000) {
            let r = f.reading;
            if r.status == TunerStatus::Pitch {
                // Strongest pitch or a nearby result; never an unrelated note.
                let hz = r.frequency_hz.unwrap();
                assert!(
                    (hz - 110.0).abs() < 1.5 || (hz - 164.81).abs() < 1.5,
                    "balance {a}/{b}: {hz:.2} Hz"
                );
            } else {
                assert!(!r.in_tune, "balance {a}/{b}: in_tune without a pitch");
                assert!(matches!(
                    r.status,
                    TunerStatus::Listening
                        | TunerStatus::NoStablePitch
                        | TunerStatus::LowConfidence
                ));
            }
        }
    }
}

#[test]
fn chromatic_mode_has_no_string_data() {
    let mut t = tuner(chromatic(), 48_000);
    assert_eq!(t.config().mode, TunerMode::Chromatic);
    let r = feed(&mut t, &sine(A4, 48_000, 24_000, 0.5), 512, 48_000)
        .pop()
        .unwrap()
        .reading;
    assert!(r.target.is_none() && r.ticks.is_empty());
}

#[test]
fn nearest_pitch_agrees_with_the_tuner() {
    let (p, c) = nearest_pitch(73.42, ReferencePitch::default()).unwrap();
    assert_eq!(p.name(), "D2");
    assert!(c.abs() < 1.0);
}

proptest! {
    #[test]
    fn pure_sine_is_within_1_cent(f in 70.0_f64..1300.0, rate in prop::sample::select(vec![44_100_u32, 48_000])) {
        let mut d = PitchDetector::new(DetectorConfig::new(rate).unwrap()).unwrap();
        let est = d.detect(&sine(f, rate, 2048, 0.5)).expect("pitch");
        let cents = 1200.0 * (est.frequency_hz / f).log2();
        prop_assert!(cents.abs() <= 1.0, "{f} Hz at {rate}: {cents:.3} cents");
    }

    #[test]
    fn detector_never_panics(samples in prop::collection::vec(prop::num::f32::ANY, 1..3000)) {
        let mut d = PitchDetector::new(DetectorConfig::new(48_000).unwrap()).unwrap();
        let _ = d.detect(&samples);
        let _ = d.analyse(&samples);
    }

    #[test]
    fn tuner_never_panics(samples in prop::collection::vec(prop::num::f32::ANY, 1..6000), block in 1_usize..2000) {
        let mut t = tuner(TunerConfig::chromatic(ReferencePitch::default()), 44_100);
        for chunk in samples.chunks(block) {
            for r in t.push_block(chunk) {
                prop_assert!(r.t_ms.is_finite());
            }
        }
    }
}
