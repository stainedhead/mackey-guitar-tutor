//! Movable CAGED forms (FB-5.2, FR-014): templates relative to the fret of the root, applied
//! to any root. The shapes are written for standard tuning; every candidate is verified against
//! the actual tuning before it is offered.

use mackey_theory::ChordQuality as Q;

use crate::voicing::CagedShape;

/// One movable form. `frets[i]` is the offset from the root fret `r` on the root string
/// (`None` = muted); the form has the root on string index `root_index` (0 = string 6).
#[derive(Debug, Clone, Copy)]
pub struct CagedTemplate {
    /// The CAGED letter.
    pub shape: CagedShape,
    /// The chord quality the form plays.
    pub quality: Q,
    /// Offsets from the root fret, string 6 first.
    pub frets: [Option<i8>; 6],
    /// Index (low to high) of the string carrying the root.
    pub root_index: usize,
    /// The lowest root fret (relative to the capo) at which every fret stays above the nut.
    pub min_root_fret: u8,
}

const X: Option<i8> = None;
#[expect(
    clippy::unnecessary_wraps,
    reason = "reads as a fret cell next to `X` in the template tables"
)]
const fn f(n: i8) -> Option<i8> {
    Some(n)
}

const fn t(
    shape: CagedShape,
    quality: Q,
    frets: [Option<i8>; 6],
    root_index: usize,
    min_root_fret: u8,
) -> CagedTemplate {
    CagedTemplate {
        shape,
        quality,
        frets,
        root_index,
        min_root_fret,
    }
}

use CagedShape::{A, C, D, E, G};

/// The templates: E and A forms (major, minor, 7, m7, maj7, sus), the C form (major, maj7), the
/// G form (major, 7) and the D form (major, minor, 7, maj7, m7). There is no movable C7 form: its
/// open first string becomes a fifth finger once the shape moves up the neck.
pub const CAGED_TEMPLATES: &[CagedTemplate] = &[
    t(E, Q::Major, [f(0), f(2), f(2), f(1), f(0), f(0)], 0, 1),
    t(E, Q::Minor, [f(0), f(2), f(2), f(0), f(0), f(0)], 0, 1),
    t(E, Q::Dominant7, [f(0), f(2), f(0), f(1), f(0), f(0)], 0, 1),
    t(E, Q::Minor7, [f(0), f(2), f(0), f(0), f(0), f(0)], 0, 1),
    t(E, Q::Major7, [f(0), f(2), f(1), f(1), f(0), f(0)], 0, 1),
    t(E, Q::Sus4, [f(0), f(2), f(2), f(2), f(0), f(0)], 0, 1),
    t(A, Q::Major, [X, f(0), f(2), f(2), f(2), f(0)], 1, 1),
    t(A, Q::Minor, [X, f(0), f(2), f(2), f(1), f(0)], 1, 1),
    t(A, Q::Dominant7, [X, f(0), f(2), f(0), f(2), f(0)], 1, 1),
    t(A, Q::Minor7, [X, f(0), f(2), f(0), f(1), f(0)], 1, 1),
    t(A, Q::Major7, [X, f(0), f(2), f(1), f(2), f(0)], 1, 1),
    t(A, Q::Sus2, [X, f(0), f(2), f(2), f(0), f(0)], 1, 1),
    t(A, Q::Sus4, [X, f(0), f(2), f(2), f(3), f(0)], 1, 1),
    t(C, Q::Major, [X, f(0), f(-1), f(-3), f(-2), f(-3)], 1, 4),
    t(C, Q::Major7, [X, f(0), f(-1), f(-3), f(-3), f(-3)], 1, 4),
    t(G, Q::Major, [f(0), f(-1), f(-3), f(-3), f(-3), f(0)], 0, 4),
    t(
        G,
        Q::Dominant7,
        [f(0), f(-1), f(-3), f(-3), f(-3), f(-2)],
        0,
        4,
    ),
    t(D, Q::Major, [X, X, f(0), f(2), f(3), f(2)], 2, 1),
    t(D, Q::Minor, [X, X, f(0), f(2), f(3), f(1)], 2, 1),
    t(D, Q::Dominant7, [X, X, f(0), f(2), f(1), f(2)], 2, 1),
    t(D, Q::Major7, [X, X, f(0), f(2), f(2), f(2)], 2, 1),
    t(D, Q::Minor7, [X, X, f(0), f(2), f(1), f(1)], 2, 1),
];
