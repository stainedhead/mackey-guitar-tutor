//! Synth behaviour measured through the pitch detector and the playback-plan builder
//! (FR-024, AC-FB-9.1 timings of the audio side).

use mackey_audio::dsp::{
    DetectorConfig, PitchDetector, PlaybackPlan, STRUM_GAP_MS, Synth, SynthCommand,
};
use mackey_theory::ReferencePitch;

fn render(s: &mut Synth, n: usize) -> Vec<f32> {
    let mut out = vec![0.0; n];
    s.render(&mut out);
    out
}

#[test]
fn synth_pitch_is_within_3_cents_across_the_range() {
    // Open strings of standard tuning plus a high fretted note and an octave of A.
    const NOTES: &[f64] = &[
        82.41, 110.0, 146.83, 196.0, 246.94, 329.63, 440.0, 659.26, 880.0,
    ];
    for sr in [44_100_u32, 48_000] {
        let mut det = PitchDetector::new(DetectorConfig::new(sr).unwrap()).unwrap();
        for &hz in NOTES {
            let mut s = Synth::new(sr, 16);
            s.apply(SynthCommand::NoteOn {
                id: 1,
                frequency_hz: hz as f32,
                velocity: 0.8,
                delay_samples: 0,
            });
            let out = render(&mut s, sr as usize / 2);
            // Skip the noisy attack, analyse a window from 100 ms in.
            let start = sr as usize / 10;
            let est = det
                .detect(&out[start..start + 2048])
                .unwrap_or_else(|| panic!("{hz} Hz at {sr}: no pitch"));
            let cents = 1200.0 * (est.frequency_hz / hz).log2();
            assert!(cents.abs() <= 3.0, "{hz} Hz at {sr}: {cents:.2} cents");
        }
    }
}

#[test]
fn render_is_deterministic_for_the_same_commands() {
    let run = || {
        let mut s = Synth::new(48_000, 16);
        let plan = PlaybackPlan::strum(&[40, 47, 52, 56, 59, 64]).unwrap();
        for c in plan.commands(48_000, ReferencePitch::default(), 0) {
            s.apply(c);
        }
        render(&mut s, 48_000)
    };
    assert_eq!(run(), run());
}

#[test]
fn strum_notes_start_30_ms_apart_sample_exact() {
    const SR: u32 = 48_000;
    let plan = PlaybackPlan::strum(&[40, 45, 50, 55, 59, 64]).unwrap();
    let cmds = plan.commands(SR, ReferencePitch::default(), 100);
    assert_eq!(cmds.len(), 6);
    let gap = (STRUM_GAP_MS * f64::from(SR) / 1000.0) as i64;
    for (k, cmd) in cmds.iter().enumerate() {
        // Each note alone in a fresh synth: its first non-zero sample is its start.
        let mut s = Synth::new(SR, 4);
        s.apply(*cmd);
        let out = render(&mut s, 12_000);
        let first = out.iter().position(|v| *v != 0.0).unwrap() as i64;
        let want = gap * k as i64;
        assert!(
            (first - want).abs() <= 1,
            "note {k}: starts at {first}, want {want}"
        );
    }
}

#[test]
fn strum_plays_all_strings_in_one_render() {
    let mut s = Synth::new(48_000, 16);
    let plan = PlaybackPlan::strum(&[40, 45, 50, 55, 59, 64]).unwrap();
    for c in plan.commands(48_000, ReferencePitch::default(), 0) {
        s.apply(c);
    }
    let out = render(&mut s, 48_000);
    assert!(out.iter().all(|v| v.is_finite() && v.abs() <= 1.0));
    // Silent for the first sample, sounding by the end of the strum.
    assert!(out[300..].iter().any(|v| v.abs() > 0.1));
}
