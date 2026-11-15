//! String-mode acceptance tests (AC-TU-2.1, AC-TU-5.1, SAC-15 string rules).

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    reason = "test helpers may panic on bad fixtures and signal maths casts freely; clippy.toml only relaxes the lints inside #[test] functions"
)]

mod common;

use common::{feed, guitar_tone, tuner};
use mackey_audio::dsp::{
    TuneDirection, TunerConfig, TunerMode, TunerStatus, TunerTarget, targets_from_pitches,
};
use mackey_theory::{AccidentalPreference::Sharps, Pitch, ReferencePitch};

fn pitch(midi: i32) -> Pitch {
    Pitch::from_midi(midi, Sharps).unwrap()
}

/// Six open strings low to high from MIDI numbers.
fn targets(midis: [i32; 6], a4: ReferencePitch) -> Vec<TunerTarget> {
    targets_from_pitches(&midis.map(pitch), a4)
}

fn strings_cfg(targets: Vec<TunerTarget>) -> TunerConfig {
    TunerConfig {
        a4: ReferencePitch::default(),
        mode: TunerMode::Strings { locked: None },
        targets,
    }
}

const STANDARD: [i32; 6] = [40, 45, 50, 55, 59, 64];
const DADGAD: [i32; 6] = [38, 45, 50, 55, 57, 62];

#[test]
fn ac_tu_2_1_dadgad_146_83_hz_selects_string_4_in_tune() {
    let mut t = tuner(
        strings_cfg(targets(DADGAD, ReferencePitch::default())),
        48_000,
    );
    let sig = guitar_tone(146.83, 48_000, 48_000, 0.5, 5);
    let r = feed(&mut t, &sig, 512, 48_000).pop().unwrap().reading;
    assert_eq!(r.status, TunerStatus::Pitch);
    let target = r.target.expect("target");
    assert_eq!(target.string, 4);
    assert!(target.cents.abs() <= 3.0, "{}", target.cents);
    assert!(r.in_tune);
    assert_eq!(r.direction, Some(TuneDirection::InTune));
    assert_eq!(r.ticks, [4]);
    assert!(!r.far_from_any);
}

#[test]
fn ac_tu_5_1_capo_2_targets() {
    // The app transposes the sounding open pitches by the capo; the helper numbers the
    // strings from 6 (lowest) to 1.
    let capo2 = STANDARD.map(|m| m + 2);
    let t = targets(capo2, ReferencePitch::default());
    let names: Vec<String> = t.iter().map(|x| x.pitch.name()).collect();
    assert_eq!(names, ["F#2", "B2", "E3", "A3", "C#4", "F#4"]);
    let strings: Vec<u8> = t.iter().map(|x| x.string).collect();
    assert_eq!(strings, [6, 5, 4, 3, 2, 1]);
    let freqs: Vec<f64> = t.iter().map(|x| x.frequency_hz).collect();
    for (f, want) in freqs
        .iter()
        .zip([92.4986, 123.4708, 164.8138, 220.0, 277.1826, 369.9944])
    {
        assert!((f - want).abs() < 0.001, "{f} vs {want}");
    }
}

#[test]
fn ac_tu_5_1_capo_2_strings_are_tuned_against_the_capoed_pitches() {
    let capo2 = STANDARD.map(|m| m + 2);
    let t = targets(capo2, ReferencePitch::default());
    for target in &t {
        let mut tn = tuner(strings_cfg(t.clone()), 48_000);
        let sig = guitar_tone(target.frequency_hz, 48_000, 48_000, 0.5, 9);
        let r = feed(&mut tn, &sig, 512, 48_000).pop().unwrap().reading;
        let got = r.target.expect("target");
        assert_eq!(got.string, target.string, "{}", target.pitch.name());
        assert!(r.in_tune, "{}", target.pitch.name());
    }
}

#[test]
fn a4_reference_moves_the_targets() {
    let t = targets(STANDARD, ReferencePitch::new(442.0).unwrap());
    // A2 is 110 * 442 / 440.
    assert!((t[1].frequency_hz - 110.5).abs() < 1e-9);
}

#[test]
fn every_standard_string_is_auto_selected_with_direction() {
    let t = targets(STANDARD, ReferencePitch::default());
    for target in &t {
        for (cents, want) in [
            (-15.0, TuneDirection::Tighten),
            (15.0, TuneDirection::Loosen),
        ] {
            let mut tn = tuner(strings_cfg(t.clone()), 44_100);
            let f = common::detuned(target.frequency_hz, cents);
            let sig = guitar_tone(f, 44_100, 22_050, 0.5, 2);
            let r = feed(&mut tn, &sig, 512, 44_100).pop().unwrap().reading;
            let label = format!("string {} {cents:+}", target.string);
            assert_eq!(r.target.expect("target").string, target.string, "{label}");
            assert_eq!(r.direction, Some(want), "{label}");
            assert!(!r.in_tune, "{label}");
            assert!(
                (f64::from(r.target.unwrap().cents) - cents).abs() <= 3.0,
                "{label}"
            );
        }
    }
}

#[test]
fn locking_a_string_overrides_auto_select() {
    let t = targets(STANDARD, ReferencePitch::default());
    let mut tn = tuner(strings_cfg(t), 48_000);
    tn.lock(Some(3));
    // A2 is sounding but string 3 (G3) is locked: 5 semitones flat of it.
    let sig = guitar_tone(110.0, 48_000, 24_000, 0.5, 4);
    let r = feed(&mut tn, &sig, 512, 48_000).pop().unwrap().reading;
    let target = r.target.unwrap();
    assert_eq!(target.string, 3);
    assert!(
        (f64::from(target.cents) + 1000.0).abs() < 10.0,
        "{}",
        target.cents
    );
    assert_eq!(r.direction, Some(TuneDirection::Tighten));
}
