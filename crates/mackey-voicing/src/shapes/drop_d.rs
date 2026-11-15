//! Curated entries for Drop D (D A D G B E) that use the open low D string (AC-LH-4.1).

use mackey_theory::ChordQuality as Q;

use super::OpenShape;
use crate::voicing::VoicingKind;

const fn shape(
    root_pc: u8,
    quality: Q,
    frets: &'static str,
    fingers: &'static str,
    name: &'static str,
) -> OpenShape {
    OpenShape {
        root_pc,
        quality,
        frets,
        fingers,
        barre: None,
        name,
        kind: VoicingKind::Open,
    }
}

/// Six-string D shapes for Drop D. Frets read from string 6 (the dropped D) to string 1.
pub const DROP_D_SHAPES: &[OpenShape] = &[
    shape(2, Q::Major, "000232", "000132", "Open D (all six strings)"),
    shape(2, Q::Minor, "000231", "000231", "Open Dm (all six strings)"),
    shape(
        2,
        Q::Dominant7,
        "000212",
        "000213",
        "Open D7 (all six strings)",
    ),
    shape(
        2,
        Q::Major7,
        "000222",
        "000123",
        "Open Dmaj7 (all six strings)",
    ),
    shape(
        2,
        Q::Sus2,
        "000230",
        "000120",
        "Open Dsus2 (all six strings)",
    ),
    shape(
        2,
        Q::Sus4,
        "000233",
        "000123",
        "Open Dsus4 (all six strings)",
    ),
    shape(2, Q::Power, "000xxx", "000xxx", "Open D5"),
];
