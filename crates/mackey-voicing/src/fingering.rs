//! Finger assignment and barre detection (FB-5.4, FB-5.5, B4).
//!
//! The rule: the lowest pressed fret, when repeated on two or more strings with no open string
//! between the outermost two, is a barre (one finger, finger 1). Every other pressed note takes
//! the next finger in order of fret, then string from the lowest. Notes at the same fret on
//! strings tuned to the same pitch share a finger (E-FB-6). More than four fingers means the
//! shape is rejected.

use crate::instrument::{Fret, StringNumber};
use crate::voicing::{Barre, Fingering, StringState};

/// Assigns fingers to a set of string states, or `None` when more than four fingers are
/// needed. `open_midis` are the open-string pitches low to high; they decide which strings are
/// identical.
///
/// ```
/// use mackey_voicing::{assign_fingering, Fret, StringState};
/// let f = |n| StringState::Fretted { fret: Fret::new(n).unwrap() };
/// // C: x32010
/// let strings = [StringState::Muted, f(3), f(2), StringState::Open, f(1), StringState::Open];
/// let fingering = assign_fingering(&strings, [40, 45, 50, 55, 59, 64]).unwrap();
/// assert_eq!(fingering.fingers, [None, Some(3), Some(2), Some(0), Some(1), Some(0)]);
/// ```
#[must_use]
pub fn assign_fingering(strings: &[StringState; 6], open_midis: [i32; 6]) -> Option<Fingering> {
    let mut fretted: Vec<(usize, u8)> = strings
        .iter()
        .enumerate()
        .filter_map(|(i, s)| s.fretted().map(|f| (i, f)))
        .collect();
    fretted.sort_by_key(|&(i, f)| (f, i));

    let mut fingers: [Option<u8>; 6] = strings.map(|s| match s {
        StringState::Muted => None,
        _ => Some(0),
    });
    let mut barres = Vec::new();

    let Some(&(_, f_min)) = fretted.first() else {
        return Some(Fingering { fingers, barres });
    };

    // Distinct notes at the lowest fret (strings with identical tuning count once).
    let at_min: Vec<usize> = fretted
        .iter()
        .filter(|&&(_, f)| f == f_min)
        .map(|&(i, _)| i)
        .collect();
    let distinct_at_min = distinct_count(&at_min, &open_midis);
    let mut next_finger: u8 = 1;
    let mut barre_range: Option<(usize, usize)> = None;
    if distinct_at_min >= 2
        && let (Some(&lo), Some(&hi)) = (at_min.iter().min(), at_min.iter().max())
    {
        let blocked = strings
            .iter()
            .enumerate()
            .any(|(i, s)| i > lo && i < hi && matches!(s, StringState::Open));
        if !blocked {
            barre_range = Some((lo, hi));
            next_finger = 2;
            if let (Some(from), Some(to), Ok(fret)) = (
                StringNumber::from_index(lo),
                StringNumber::from_index(hi),
                Fret::new(f_min),
            ) {
                barres.push(Barre { fret, from, to });
            }
            for &i in &at_min {
                if let Some(slot) = fingers.get_mut(i) {
                    *slot = Some(1);
                }
            }
        }
    }
    if barre_range.is_none() {
        // No barre: the lowest-fret notes get fingers like any other note, below.
        next_finger = 1;
    }

    let mut assigned: Vec<(usize, u8, u8)> = Vec::new(); // (index, fret, finger)
    if barre_range.is_some() {
        for &i in &at_min {
            assigned.push((i, f_min, 1));
        }
    }
    for &(i, f) in &fretted {
        if barre_range.is_some() && f == f_min {
            continue;
        }
        let open = open_midis.get(i).copied().unwrap_or(0);
        let shared = assigned
            .iter()
            .find(|&&(j, fj, _)| fj == f && open_midis.get(j).copied().unwrap_or(-1) == open);
        let finger = if let Some(&(_, _, finger)) = shared {
            finger
        } else {
            let finger = next_finger;
            next_finger += 1;
            finger
        };
        assigned.push((i, f, finger));
        if let Some(slot) = fingers.get_mut(i) {
            *slot = Some(finger);
        }
    }
    if next_finger.saturating_sub(1) > 4 {
        return None;
    }
    Some(Fingering { fingers, barres })
}

fn distinct_count(indices: &[usize], open_midis: &[i32; 6]) -> usize {
    let mut seen: Vec<i32> = Vec::new();
    for &i in indices {
        let open = open_midis.get(i).copied().unwrap_or(-1);
        if !seen.contains(&open) {
            seen.push(open);
        }
    }
    seen.len()
}

/// How many fingers a fingering uses, counting a barre once.
#[must_use]
pub(crate) fn finger_count(fingering: &Fingering) -> usize {
    let mut used: Vec<u8> = fingering
        .fingers
        .iter()
        .flatten()
        .copied()
        .filter(|f| *f > 0)
        .collect();
    used.sort_unstable();
    used.dedup();
    used.len()
}
