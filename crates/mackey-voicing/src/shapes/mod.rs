//! The curated shape library (architecture §4.1, FR-014): open shapes, movable CAGED forms and
//! Drop D entries. Curated shapes apply to Standard tuning, to tunings that move every string
//! by the same amount (Half-step down, Whole-step down) and, for the Drop D entries, to Drop D.
//! Every candidate is verified note by note against the real tuning and capo before it is
//! offered, so a curated shape can never show a wrong note.

mod caged;
mod drop_d;
mod open;

use mackey_theory::{Chord, ChordQuality};

pub use caged::{CAGED_TEMPLATES, CagedTemplate};
pub use drop_d::DROP_D_SHAPES;
pub use open::OPEN_SHAPES;

use crate::fingering::assign_fingering;
use crate::instrument::{Fret, Instrument, StringNumber, TuningPreset};
use crate::rules::{Build, Rules, weights};
use crate::voicing::{Barre, Fingering, Origin, StringState, Voicing, VoicingKind};

/// A hand-written open shape: frets and fingers as six characters, string 6 first.
#[derive(Debug, Clone, Copy)]
pub struct OpenShape {
    /// Pitch class of the chord root (C = 0).
    pub root_pc: u8,
    /// The chord quality.
    pub quality: ChordQuality,
    /// Frets: `x` muted, digits pressed or open.
    pub frets: &'static str,
    /// Fingers: `x` muted, `0` open, `1` to `4`.
    pub fingers: &'static str,
    /// An explicit barre `(fret, from string, to string)`, if the shape has one.
    pub barre: Option<(u8, u8, u8)>,
    /// The name, such as "Open C".
    pub name: &'static str,
    /// Open or barre.
    pub kind: VoicingKind,
}

impl OpenShape {
    /// Parses the fret string: `None` is muted, `Some(0)` open.
    #[must_use]
    pub fn parse_frets(&self) -> Option<[Option<u8>; 6]> {
        parse_six(self.frets)
    }

    /// Parses the finger string: `None` is muted, `Some(0)` open.
    #[must_use]
    pub fn parse_fingers(&self) -> Option<[Option<u8>; 6]> {
        parse_six(self.fingers)
    }
}

fn parse_six(text: &str) -> Option<[Option<u8>; 6]> {
    let mut out = [None; 6];
    let mut count = 0;
    for (slot, c) in out.iter_mut().zip(text.chars()) {
        *slot = match c {
            'x' | 'X' => None,
            d => Some(u8::try_from(d.to_digit(10)?).ok()?),
        };
        count += 1;
    }
    (count == 6 && text.chars().count() == 6).then_some(out)
}

/// What kind of curated library a tuning gets.
enum Family {
    /// All strings shifted by 0, -1 or -2 semitones from standard.
    Shifted(i32),
    /// Drop D.
    DropD,
    /// Search only (LH-4).
    None,
}

fn family_of(instrument: &Instrument) -> Family {
    if instrument.tuning.matches(TuningPreset::DropD) {
        return Family::DropD;
    }
    match instrument.tuning.shift_from_standard() {
        Some(shift) if (-2..=0).contains(&shift) => Family::Shifted(i32::from(shift)),
        _ => Family::None,
    }
}

/// True when the tuning belongs to the curated library (so its search results carry no
/// "Found by search" label, LH-4).
#[must_use]
pub fn is_curated_tuning(instrument: &Instrument) -> bool {
    !matches!(family_of(instrument), Family::None)
}

/// The tuning's uniform shift for shape lookup (0 for Drop D, whose entries are absolute).
fn shape_shift(family: &Family) -> i32 {
    match family {
        Family::Shifted(s) => *s,
        _ => 0,
    }
}

/// The pitch class of the chord the fingers play: the sounding chord minus the capo and the
/// tuning shift (D-04).
#[must_use]
pub(crate) fn shape_root_pc(chord: Chord, instrument: &Instrument) -> u8 {
    let shift = shape_shift(&family_of(instrument));
    let pc =
        i32::from(chord.root.pitch_class().value()) - i32::from(instrument.capo.value()) - shift;
    u8::try_from(pc.rem_euclid(12)).unwrap_or(0)
}

fn to_states(frets: &[Option<u8>; 6], capo: u8) -> [StringState; 6] {
    frets.map(|f| match f {
        None => StringState::Muted,
        Some(0) => StringState::Open,
        Some(n) => {
            Fret::new(n + capo).map_or(StringState::Muted, |fret| StringState::Fretted { fret })
        }
    })
}

fn hand_fingering(shape: &OpenShape, frets: &[Option<u8>; 6], capo: u8) -> Option<Fingering> {
    let fingers = shape.parse_fingers()?;
    let barres = match shape.barre {
        Some((fret, from, to)) => vec![Barre {
            fret: Fret::new(fret + capo).ok()?,
            from: StringNumber::new(from).ok()?,
            to: StringNumber::new(to).ok()?,
        }],
        None => Vec::new(),
    };
    // The fingers must agree with the frets: muted where muted, open where open.
    for (finger, fret) in fingers.iter().zip(frets.iter()) {
        let ok = match fret {
            None => finger.is_none(),
            Some(0) => *finger == Some(0),
            Some(_) => matches!(finger, Some(1..=4)),
        };
        if !ok {
            return None;
        }
    }
    Some(Fingering { fingers, barres })
}

/// The curated voicings for a chord on this instrument: open shapes (with capo and shift
/// translation), movable CAGED forms and the Drop D entries. Empty for slash chords and for
/// tunings outside the curated library.
pub(crate) fn curated_voicings(
    chord: Chord,
    instrument: &Instrument,
    rules: &Rules,
) -> Vec<Voicing> {
    let family = family_of(instrument);
    if matches!(family, Family::None) || chord.bass.is_some() {
        return Vec::new();
    }
    let capo = instrument.capo.value();
    let shift = shape_shift(&family);
    let shape_pc = shape_root_pc(chord, instrument);
    let renamed = capo > 0 || shift != 0;
    let mut out = Vec::new();
    let mut push = |shape: &OpenShape| {
        if shape.root_pc != shape_pc || shape.quality != chord.quality {
            return;
        }
        let Some(frets) = shape.parse_frets() else {
            return;
        };
        let Some(fingering) = hand_fingering(shape, &frets, capo) else {
            return;
        };
        let strings = to_states(&frets, capo);
        let Some(evaluated) = rules.evaluate(&strings) else {
            return;
        };
        let name = if renamed && shape.name.starts_with("Open ") {
            format!("{} shape", shape.name)
        } else {
            shape.name.to_owned()
        };
        out.push(
            Build {
                chord,
                capo: instrument.capo,
                strings,
                evaluated: crate::rules::Evaluated {
                    fingering,
                    ..evaluated
                },
                kind: shape.kind,
                origin: Origin::Curated,
                name,
                bonus: weights::CURATED,
            }
            .finish(),
        );
    };
    for shape in OPEN_SHAPES {
        push(shape);
    }
    if matches!(family, Family::DropD) {
        for shape in DROP_D_SHAPES {
            push(shape);
        }
    }
    for template in CAGED_TEMPLATES
        .iter()
        .filter(|t| t.quality == chord.quality)
    {
        out.extend(caged_voicings(template, chord, instrument, rules));
    }
    out
}

fn caged_voicings(
    template: &CagedTemplate,
    chord: Chord,
    instrument: &Instrument,
    rules: &Rules,
) -> Vec<Voicing> {
    let capo = instrument.capo.value();
    let tuning = instrument.tuning.midis();
    let open = tuning.get(template.root_index).copied().unwrap_or(0) + i32::from(capo);
    let root = i32::from(chord.root.pitch_class().value());
    let Ok(rel) = u8::try_from((root - open).rem_euclid(12)) else {
        return Vec::new();
    };
    // A form that needs a higher root fret than the root's lowest position is played an octave up.
    let base = if rel < template.min_root_fret {
        rel + 12
    } else {
        rel
    };
    let mut out = Vec::new();
    {
        let mut strings = [StringState::Muted; 6];
        let mut ok = true;
        for (slot, offset) in strings.iter_mut().zip(template.frets.iter()) {
            if let Some(offset) = offset {
                let fret = i32::from(base) + i32::from(*offset) + i32::from(capo);
                match u8::try_from(fret).ok().and_then(|f| Fret::new(f).ok()) {
                    Some(fret)
                        if fret.value() > capo && fret.value() <= instrument.frets.value() =>
                    {
                        *slot = StringState::Fretted { fret };
                    }
                    _ => ok = false,
                }
            }
        }
        if !ok {
            return out;
        }
        let Some(evaluated) = rules.evaluate(&strings) else {
            return out;
        };
        let lowest = evaluated.lowest_pressed.unwrap_or(0);
        let barre = !evaluated.fingering.barres.is_empty();
        let name = if barre {
            format!("{}-shape barre, fret {}", template.shape.letter(), lowest)
        } else {
            format!("{}-shape, fret {}", template.shape.letter(), lowest)
        };
        out.push(
            Build {
                chord,
                capo: instrument.capo,
                strings,
                evaluated,
                kind: VoicingKind::Caged {
                    shape: template.shape,
                },
                origin: Origin::Curated,
                name,
                bonus: weights::CURATED,
            }
            .finish(),
        );
    }
    out
}

/// Re-derives a fingering for a hand-written shape, for tests.
#[must_use]
pub fn derived_fingering(shape: &OpenShape, open_midis: [i32; 6]) -> Option<Fingering> {
    let frets = shape.parse_frets()?;
    assign_fingering(&to_states(&frets, 0), open_midis)
}
