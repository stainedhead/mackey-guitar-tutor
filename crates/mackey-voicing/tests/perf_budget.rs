//! NF-2 / AC-FB-9.1a: voicing search p95 at most 5 ms and `fretboard_view` p95 at most 2 ms.
//! The assertions only run in release builds (`make props` uses `--release`); in a debug build
//! the timings say nothing about the budget, so the test just exercises the code.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]

use std::time::{Duration, Instant};

use mackey_theory::{ChordQuality, parse_chord_symbol};
use mackey_voicing::{
    Focus, Handedness, Instrument, Selection, ViewRequest, ViewType, VoicingOptions,
    fretboard_view, voicings_for,
};

const RUNS: usize = 200;

fn p95(mut samples: Vec<Duration>) -> Duration {
    samples.sort();
    samples[samples.len() * 95 / 100]
}

#[test]
fn nf_2_voicing_search_p95_is_within_5_ms() {
    let ins = Instrument::default();
    let options = VoicingOptions::default();
    let roots = [
        "C", "Db", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
    ];
    let mut samples = Vec::new();
    for run in 0..RUNS {
        let quality = ChordQuality::ALL[run % ChordQuality::ALL.len()];
        let chord = mackey_theory::Chord::new(roots[run % roots.len()].parse().unwrap(), quality);
        let start = Instant::now();
        let result = voicings_for(&chord, &ins, &options);
        samples.push(start.elapsed());
        std::hint::black_box(result);
    }
    let p = p95(samples);
    if cfg!(not(debug_assertions)) {
        assert!(
            p <= Duration::from_millis(5),
            "search p95 {p:?} exceeds 5 ms"
        );
    }
}

#[test]
fn nf_2_fretboard_view_p95_is_within_2_ms() {
    let ins = Instrument::default();
    let hand = Handedness::default();
    let req = ViewRequest::default();
    let chord = parse_chord_symbol("F#m7b5").unwrap();
    let mut samples = Vec::new();
    for run in 0..RUNS {
        let mut sel = Selection::default_g_major();
        sel.focus = Focus::Chord;
        sel.chord = Some(chord);
        sel.view = [
            ViewType::AllTones,
            ViewType::Voicing,
            ViewType::Triad,
            ViewType::Arpeggio,
        ][run % 4];
        sel.overlay.on = run % 2 == 0;
        let start = Instant::now();
        let view = fretboard_view(&sel, &ins, &hand, &req);
        samples.push(start.elapsed());
        std::hint::black_box(view);
    }
    let p = p95(samples);
    if cfg!(not(debug_assertions)) {
        assert!(p <= Duration::from_millis(2), "view p95 {p:?} exceeds 2 ms");
    }
}
