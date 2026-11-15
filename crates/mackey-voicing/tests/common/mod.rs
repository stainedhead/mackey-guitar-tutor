//! Shared test helpers. The FB-5.4 checker here is written independently of the crate's search.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]
#![allow(dead_code, clippy::trivially_copy_pass_by_ref)] // each test binary uses a different subset; signatures mirror the borrowed library API

use mackey_theory::{Chord, ToneRole};
use mackey_voicing::{Instrument, StringNumber, StringState, Voicing};

/// MIDI of each sounding string (low to high index), `None` when muted.
pub(crate) fn midis(ins: &Instrument, strings: &[StringState; 6]) -> Vec<Option<i32>> {
    strings
        .iter()
        .zip(StringNumber::LOW_TO_HIGH)
        .map(|(s, string)| match s {
            StringState::Muted => None,
            StringState::Open => Some(ins.open_sounding_pitch(string).midi()),
            StringState::Fretted { fret } => Some(ins.pitch_at(string, *fret).unwrap().midi()),
        })
        .collect()
}

fn pc(midi: i32) -> u8 {
    u8::try_from(midi.rem_euclid(12)).unwrap()
}

/// Checks FB-5.4 for one voicing and returns the reason it fails.
pub(crate) fn check_fb_5_4(
    chord: &Chord,
    ins: &Instrument,
    v: &Voicing,
    stretch: bool,
) -> Result<(), String> {
    let capo = ins.capo.value();
    let pressed: Vec<u8> = v.strings.iter().filter_map(|s| s.fretted()).collect();
    if let Some(bad) = pressed
        .iter()
        .find(|f| **f <= capo || **f > ins.frets.value())
    {
        return Err(format!(
            "fret {bad} outside capo {capo}..={}",
            ins.frets.value()
        ));
    }
    if let (Some(lo), Some(hi)) = (pressed.iter().min(), pressed.iter().max()) {
        let limit = if stretch { 5 } else { 4 };
        if hi - lo > limit {
            return Err(format!("span {} > {limit}", hi - lo));
        }
    }
    let mut fingers: Vec<u8> = v
        .fingering
        .fingers
        .iter()
        .flatten()
        .copied()
        .filter(|f| *f > 0)
        .collect();
    fingers.sort_unstable();
    fingers.dedup();
    if fingers.len() > 4 {
        return Err(format!("{} fingers", fingers.len()));
    }
    let tones = chord.tones();
    let mut allowed: Vec<u8> = tones.iter().map(|t| t.note.pitch_class().value()).collect();
    if let Some(b) = chord.bass {
        allowed.push(b.pitch_class().value());
    }
    let sounding: Vec<i32> = midis(ins, &v.strings).into_iter().flatten().collect();
    if let Some(m) = sounding.iter().find(|m| !allowed.contains(&pc(**m))) {
        return Err(format!("foreign pitch class {}", pc(*m)));
    }
    for role in chord.quality.required_tones() {
        let needed: Vec<u8> = tones
            .iter()
            .filter(|t| t.role == *role)
            .map(|t| t.note.pitch_class().value())
            .collect();
        if !sounding.iter().any(|m| needed.contains(&pc(*m))) {
            return Err(format!("required {role:?} missing"));
        }
    }
    let lowest = sounding.iter().min().ok_or("no sounding string")?;
    let bass = chord.bass_or_root().pitch_class().value();
    if pc(*lowest) != bass {
        return Err(format!(
            "lowest pitch class {} is not the bass {bass}",
            pc(*lowest)
        ));
    }
    Ok(())
}

/// The roles present in a voicing (for assertions).
pub(crate) fn roles(chord: &Chord, ins: &Instrument, v: &Voicing) -> Vec<ToneRole> {
    let tones = chord.tones();
    midis(ins, &v.strings)
        .into_iter()
        .flatten()
        .filter_map(|m| {
            tones
                .iter()
                .find(|t| t.note.pitch_class().value() == pc(m))
                .map(|t| t.role)
        })
        .collect()
}
