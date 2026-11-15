//! NF-2 fretboard-view budget: at most 2 ms to build a `FretboardView`.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use mackey_theory::parse_chord_symbol;
use mackey_voicing::{
    Focus, Handedness, Instrument, Selection, ViewRequest, ViewType, VoicingCache, fretboard_view,
    fretboard_view_cached,
};

fn views(c: &mut Criterion) {
    let instrument = Instrument::default();
    let hand = Handedness::default();
    let req = ViewRequest::default();
    let base = Selection::default_g_major();
    c.bench_function("fretboard_view/key only", |b| {
        b.iter(|| black_box(fretboard_view(black_box(&base), &instrument, &hand, &req)));
    });
    let chord = parse_chord_symbol("Am7").ok();
    for (name, view) in [
        ("all tones", ViewType::AllTones),
        ("voicing", ViewType::Voicing),
        ("triad", ViewType::Triad),
        ("arpeggio", ViewType::Arpeggio),
    ] {
        let mut sel = base.clone();
        sel.focus = Focus::Chord;
        sel.chord = chord;
        sel.view = view;
        sel.overlay.on = true;
        let mut cache = VoicingCache::new();
        c.bench_function(&format!("fretboard_view/Am7 {name} (warm cache)"), |b| {
            b.iter(|| {
                black_box(fretboard_view_cached(
                    black_box(&sel),
                    &instrument,
                    &hand,
                    &req,
                    &mut cache,
                ))
            });
        });
        c.bench_function(&format!("fretboard_view/Am7 {name} (cold)"), |b| {
            b.iter(|| black_box(fretboard_view(black_box(&sel), &instrument, &hand, &req)));
        });
    }
}

criterion_group!(benches, views);
criterion_main!(benches);
