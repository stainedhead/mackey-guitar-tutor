//! Triads on three adjacent strings (FB-6, FR-016).

use mackey_theory::{Chord, ChordQuality, ChordTone};

use crate::fingering::assign_fingering;
use crate::instrument::{Fret, Instrument, Inversion, StringSet};
use crate::rules::{Build, Evaluated};
use crate::voicing::{Origin, StringState, TriadResult, Voicing, VoicingKind};

/// The note shown for a seventh chord in triad view: "Triad of Cmaj7: C E G" (FB-6.3).
///
/// ```
/// use mackey_theory::{Chord, ChordQuality};
/// use mackey_voicing::triad_note;
/// let c = Chord::new("C".parse().unwrap(), ChordQuality::Major7);
/// assert_eq!(triad_note(&c).as_deref(), Some("Triad of Cmaj7: C E G"));
/// let plain = Chord::new("C".parse().unwrap(), ChordQuality::Major);
/// assert_eq!(triad_note(&plain), None);
/// ```
#[must_use]
pub fn triad_note(chord: &Chord) -> Option<String> {
    if !chord.quality.has_seventh() {
        return None;
    }
    let names: Vec<String> = chord
        .tones()
        .iter()
        .take(3)
        .map(|t| t.note.name())
        .collect();
    Some(format!("Triad of {}: {}", chord.symbol(), names.join(" ")))
}

/// The three tones of the chord's triad (root, then the next two formula tones). Seventh
/// chords use their root triad (D-12); a power chord has none.
fn triad_tones(chord: Chord) -> Option<[ChordTone; 3]> {
    let tones = chord.tones();
    match tones.as_slice() {
        [a, b, c, ..] if chord.quality != ChordQuality::Power => Some([*a, *b, *c]),
        _ => None,
    }
}

/// Every placement of one inversion of the chord's triad on a string set, lowest fret first
/// (FB-6.1, FB-6.4). One note per string on the three adjacent strings, spanning at most four
/// pressed frets. `Unavailable` says why when there is none.
///
/// ```
/// use mackey_theory::{Chord, ChordQuality};
/// use mackey_voicing::{triads, Instrument, Inversion, StringSet, TriadResult, Capo};
/// let c = Chord::new("C".parse().unwrap(), ChordQuality::Major);
/// let TriadResult::Placements(list) = triads(&c, &Instrument::default(), StringSet::S234, Inversion::First)
///     else { panic!("placements exist") };
/// // E on string 4 fret 2, G on string 3 open, C on string 2 fret 1.
/// assert_eq!(list[0].fret_string(Capo::default()), "xx201x");
/// ```
#[must_use]
pub fn triads(
    chord: &Chord,
    instrument: &Instrument,
    set: StringSet,
    inversion: Inversion,
) -> TriadResult {
    let Some(tones) = triad_tones(*chord) else {
        return TriadResult::Unavailable {
            reason: "A power chord has no third, so it has no triad".to_owned(),
        };
    };
    if inversion == Inversion::Third {
        return TriadResult::Unavailable {
            reason: "A triad has three inversions: root, 1st and 2nd".to_owned(),
        };
    }
    let shift = inversion.index();
    let [top, mid, low] = set.strings();
    // Lowest-pitched string carries the inversion's bass tone, then the tones rise in order.
    let assignment = [
        (low, tones.get(shift).copied()),
        (mid, tones.get((shift + 1) % 3).copied()),
        (top, tones.get((shift + 2) % 3).copied()),
    ];
    let capo = instrument.capo.value();
    let frets = instrument.frets.value();
    let open_midis = instrument.tuning.midis();
    let pitch = |string: crate::StringNumber, fret: u8| -> i32 {
        open_midis.get(string.index()).copied().unwrap_or(0) + i32::from(fret)
    };
    let per_string: Vec<Vec<u8>> = assignment
        .iter()
        .map(|(string, tone)| {
            let Some(tone) = tone else { return Vec::new() };
            (capo..=frets)
                .filter(|&f| instrument.pc_at_index(string.index(), f) == tone.note.pitch_class())
                .collect()
        })
        .collect();
    let (Some(low_frets), Some(mid_frets), Some(top_frets)) =
        (per_string.first(), per_string.get(1), per_string.get(2))
    else {
        return unavailable();
    };
    let mut found: Vec<[u8; 3]> = Vec::new();
    for &fret_low in low_frets {
        for &fret_mid in mid_frets {
            for &fret_top in top_frets {
                let combo = [fret_low, fret_mid, fret_top];
                // The pitches must rise from the low string to the top one, or the lowest
                // sounding note is not the inversion's bass (FR-003 of the auto review).
                let [(s_low, _), (s_mid, _), (s_top, _)] = assignment;
                let (p_low, p_mid, p_top) = (
                    pitch(s_low, fret_low),
                    pitch(s_mid, fret_mid),
                    pitch(s_top, fret_top),
                );
                if !(p_low < p_mid && p_mid < p_top) {
                    continue;
                }
                let pressed: Vec<u8> = combo.iter().copied().filter(|f| *f > capo).collect();
                let span = match (pressed.iter().min(), pressed.iter().max()) {
                    (Some(lo), Some(hi)) => hi - lo,
                    _ => 0,
                };
                if span <= 4 {
                    found.push(combo);
                }
            }
        }
    }
    found.sort_by_key(|combo| {
        (
            combo.iter().copied().min().unwrap_or(0),
            combo.iter().copied().max().unwrap_or(0),
            *combo,
        )
    });
    let list: Vec<Voicing> = found
        .into_iter()
        .filter_map(|combo| triad_voicing(*chord, instrument, set, inversion, &assignment, combo))
        .collect();
    if list.is_empty() {
        unavailable()
    } else {
        TriadResult::Placements(list)
    }
}

/// One placement: the three tones on the set's strings at `combo` frets (low string first).
fn triad_voicing(
    chord: Chord,
    instrument: &Instrument,
    set: StringSet,
    inversion: Inversion,
    assignment: &[(crate::StringNumber, Option<ChordTone>); 3],
    combo: [u8; 3],
) -> Option<Voicing> {
    let capo = instrument.capo.value();
    let open_midis = instrument.tuning.midis();
    let mut strings = [StringState::Muted; 6];
    for ((string, _), fret) in assignment.iter().zip(combo) {
        let state = if fret == capo {
            StringState::Open
        } else {
            StringState::Fretted {
                fret: Fret::new(fret).ok()?,
            }
        };
        *strings.get_mut(string.index())? = state;
    }
    let fingering = assign_fingering(&strings, open_midis)?;
    let pressed: Vec<u8> = strings.iter().filter_map(|s| s.fretted()).collect();
    let (lo, hi) = (pressed.iter().min().copied(), pressed.iter().max().copied());
    let span = match (lo, hi) {
        (Some(l), Some(h)) => h - l,
        _ => 0,
    };
    Some(
        Build {
            chord,
            capo: instrument.capo,
            strings,
            evaluated: Evaluated {
                fingering,
                span,
                lowest_pressed: lo,
                open_position: false,
                score: 0,
            },
            kind: VoicingKind::Triad { set, inversion },
            origin: Origin::Search,
            name: format!("Strings {}, {} inversion", set.label(), inversion.label()),
            bonus: 0,
        }
        .finish(),
    )
}

fn unavailable() -> TriadResult {
    TriadResult::Unavailable {
        reason: "Not playable on these strings within 4 frets".to_owned(),
    }
}

/// "Show all on string set" (FB-6.2): the lowest placement of each inversion, in fret order up
/// the neck, each labelled by its inversion. Inversions with no placement are left out.
///
/// ```
/// use mackey_theory::{Chord, ChordQuality};
/// use mackey_voicing::{triad_ladder, Instrument, Inversion, StringSet};
/// let g = Chord::new("G".parse().unwrap(), ChordQuality::Major);
/// let ladder = triad_ladder(&g, &Instrument::default(), StringSet::S123);
/// let order: Vec<Inversion> = ladder.iter().map(|(i, _)| *i).collect();
/// assert_eq!(order, [Inversion::Root, Inversion::First, Inversion::Second]);
/// ```
#[must_use]
pub fn triad_ladder(
    chord: &Chord,
    instrument: &Instrument,
    set: StringSet,
) -> Vec<(Inversion, Voicing)> {
    let mut ladder: Vec<(Inversion, Voicing)> =
        [Inversion::Root, Inversion::First, Inversion::Second]
            .into_iter()
            .filter_map(|inversion| {
                triads(chord, instrument, set, inversion)
                    .placements()
                    .first()
                    .cloned()
                    .map(|v| (inversion, v))
            })
            .collect();
    ladder.sort_by_key(|(_, v)| lowest_incl_open(v, instrument));
    ladder
}

/// The fret ordering key: the lowest fret including open strings (open = the capo).
pub(crate) fn lowest_incl_open(v: &Voicing, instrument: &Instrument) -> u8 {
    v.strings
        .iter()
        .filter_map(|s| match s {
            StringState::Open => Some(instrument.capo.value()),
            StringState::Fretted { fret } => Some(fret.value()),
            StringState::Muted => None,
        })
        .min()
        .unwrap_or(0)
}
