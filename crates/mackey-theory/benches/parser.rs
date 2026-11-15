//! NF-2 parser budget: at most 1 ms per 1,000 characters of chord symbols.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use mackey_theory::{ParseOptions, parse_chord, parse_chord_symbol};

const ROOTS: [&str; 12] = [
    "C", "Db", "D", "Eb", "E", "F", "F#", "G", "Ab", "A", "Bb", "B",
];
const SUFFIXES: [&str; 21] = [
    "", "m", "7", "maj7", "m7", "m7b5", "dim", "dim7", "aug", "sus2", "sus4", "7sus4", "6", "m6",
    "9", "maj9", "m9", "add9", "11", "13", "5",
];

/// 1,000 valid symbols (root x quality, some with a slash bass), about 4,500 characters.
fn symbols() -> Vec<String> {
    (0..1_000)
        .map(|i| {
            let root = ROOTS[i % ROOTS.len()];
            let suffix = SUFFIXES[(i / 3) % SUFFIXES.len()];
            if i % 7 == 0 {
                format!("{root}{suffix}/{}", ROOTS[(i + 4) % ROOTS.len()])
            } else {
                format!("{root}{suffix}")
            }
        })
        .collect()
}

fn bench_parser(c: &mut Criterion) {
    let valid = symbols();
    let invalid: Vec<String> = (0..1_000).map(|i| format!("H{}maj77x{i}", i % 5)).collect();
    c.bench_function("parse_chord_symbol x1000 valid", |b| {
        b.iter(|| {
            for s in &valid {
                black_box(parse_chord_symbol(black_box(s)).ok());
            }
        });
    });
    c.bench_function("parse_chord x1000 with diagnostics (invalid)", |b| {
        b.iter(|| {
            for s in &invalid {
                black_box(parse_chord(black_box(s), ParseOptions::default()));
            }
        });
    });
}

criterion_group!(benches, bench_parser);
criterion_main!(benches);
