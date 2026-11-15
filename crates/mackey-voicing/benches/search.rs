//! NF-2 voicing-search budget: at most 5 ms per search on a mid-range iPad.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use mackey_theory::parse_chord_symbol;
use mackey_voicing::{Instrument, VoicingOptions, voicings_for};

const SYMBOLS: [&str; 12] = [
    "C", "G", "Am", "F", "Bb7", "F#m7b5", "Dmaj7", "Esus4", "A7", "Cadd9", "B", "Ebm",
];

fn search(c: &mut Criterion) {
    let instrument = Instrument::default();
    let options = VoicingOptions::default();
    let chords: Vec<_> = SYMBOLS
        .iter()
        .filter_map(|s| parse_chord_symbol(s).ok())
        .collect();
    c.bench_function("voicings_for/12 chords, standard tuning", |b| {
        b.iter(|| {
            for chord in &chords {
                black_box(voicings_for(black_box(chord), &instrument, &options));
            }
        });
    });
    let worst = parse_chord_symbol("F#m7b5").unwrap_or_else(|_| unreachable!("valid symbol"));
    c.bench_function("voicings_for/F#m7b5 (one chord)", |b| {
        b.iter(|| black_box(voicings_for(black_box(&worst), &instrument, &options)));
    });
}

criterion_group!(benches, search);
criterion_main!(benches);
