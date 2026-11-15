//! The curated open shapes of FB-5.3, hand-written from standard teaching charts (D-10, D-50).
//!
//! Frets read from string 6 to string 1 (`x` muted, `0` open). Fingers use the common teaching
//! choice: C 3-2-1, A 1-2-3, G 2-1-3 (the first G fingering of most method books), E 2-3-1,
//! Em 2-3, Am 2-3-1, D 1-3-2, B7 2-1-3-4. Each entry is the sounding chord of its root in
//! standard tuning with no capo.

use mackey_theory::ChordQuality as Q;

use super::OpenShape;
use crate::voicing::VoicingKind;

const fn shape(
    root_pc: u8,
    quality: Q,
    frets: &'static str,
    fingers: &'static str,
    barre: Option<(u8, u8, u8)>,
    name: &'static str,
    kind: VoicingKind,
) -> OpenShape {
    OpenShape {
        root_pc,
        quality,
        frets,
        fingers,
        barre,
        name,
        kind,
    }
}

const O: VoicingKind = VoicingKind::Open;

/// The open shapes of FB-5.3: C, A, G, E, D, Am, Em, Dm, A7, B7, C7, D7, E7, G7, Amaj7,
/// Cmaj7, Dmaj7, Fmaj7, Am7, Dm7, Em7, Asus2, Asus4, Dsus2, Dsus4, Esus4, small F, and an
/// alternative Em7. The full barre F is the movable E shape at fret 1.
pub const OPEN_SHAPES: &[OpenShape] = &[
    shape(0, Q::Major, "x32010", "x32010", None, "Open C", O),
    shape(9, Q::Major, "x02220", "x01230", None, "Open A", O),
    shape(7, Q::Major, "320003", "210003", None, "Open G", O),
    shape(4, Q::Major, "022100", "023100", None, "Open E", O),
    shape(2, Q::Major, "xx0232", "xx0132", None, "Open D", O),
    shape(9, Q::Minor, "x02210", "x02310", None, "Open Am", O),
    shape(4, Q::Minor, "022000", "023000", None, "Open Em", O),
    shape(2, Q::Minor, "xx0231", "xx0231", None, "Open Dm", O),
    shape(9, Q::Dominant7, "x02020", "x01020", None, "Open A7", O),
    shape(11, Q::Dominant7, "x21202", "x21304", None, "Open B7", O),
    shape(0, Q::Dominant7, "x32310", "x32410", None, "Open C7", O),
    shape(2, Q::Dominant7, "xx0212", "xx0213", None, "Open D7", O),
    shape(4, Q::Dominant7, "020100", "020100", None, "Open E7", O),
    shape(7, Q::Dominant7, "320001", "320001", None, "Open G7", O),
    shape(9, Q::Major7, "x02120", "x02130", None, "Open Amaj7", O),
    shape(0, Q::Major7, "x32000", "x32000", None, "Open Cmaj7", O),
    shape(2, Q::Major7, "xx0222", "xx0123", None, "Open Dmaj7", O),
    shape(5, Q::Major7, "xx3210", "xx3210", None, "Open Fmaj7", O),
    shape(9, Q::Minor7, "x02010", "x02010", None, "Open Am7", O),
    shape(
        2,
        Q::Minor7,
        "xx0211",
        "xx0211",
        Some((1, 2, 1)),
        "Open Dm7",
        O,
    ),
    shape(4, Q::Minor7, "020000", "020000", None, "Open Em7", O),
    shape(
        4,
        Q::Minor7,
        "022030",
        "023040",
        None,
        "Open Em7 (with D on string 2)",
        O,
    ),
    shape(9, Q::Sus2, "x02200", "x01200", None, "Open Asus2", O),
    shape(9, Q::Sus4, "x02230", "x01240", None, "Open Asus4", O),
    shape(2, Q::Sus2, "xx0230", "xx0120", None, "Open Dsus2", O),
    shape(2, Q::Sus4, "xx0233", "xx0123", None, "Open Dsus4", O),
    shape(4, Q::Sus4, "022200", "023400", None, "Open Esus4", O),
    shape(
        5,
        Q::Major,
        "xx3211",
        "xx3211",
        Some((1, 2, 1)),
        "Small F",
        VoicingKind::Barre,
    ),
];
