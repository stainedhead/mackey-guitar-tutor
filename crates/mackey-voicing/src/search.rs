//! Voicing search, scoring, grouping and the "no playable voicing" result (FB-5, FR-015).
//!
//! Curated shapes come first, the search fills the gaps (architecture §4.1). Every voicing
//! returned obeys FB-5.4: root and third (or the sus note, or root and fifth for power chords),
//! a span of at most 4 frets (5 with stretches), at most 4 fingers, the right bass note.

use std::collections::HashSet;

use mackey_theory::Chord;

use crate::instrument::{Fret, Instrument};
use crate::rules::{Build, Evaluated, Rules};
use crate::shapes::{curated_voicings, is_curated_tuning};
use crate::voicing::{
    Origin, RelaxOption, StringState, Voicing, VoicingGroup, VoicingKind, VoicingOptions,
    VoicingResult,
};

/// One option for one string: a state and the pitch-class bit it sounds (0 for muted).
#[derive(Clone, Copy)]
struct Choice {
    state: StringState,
    fret: u8,
}

struct Search<'a> {
    rules: &'a Rules,
    /// Per string index: the playable choices (open and pressed frets that carry allowed notes).
    choices: [Vec<Choice>; 6],
    /// Smallest pitch any string from this index on can still sound.
    min_below: [i32; 7],
    out: Vec<([StringState; 6], Evaluated)>,
}

impl Search<'_> {
    fn dfs(
        &mut self,
        index: usize,
        current: &mut [StringState; 6],
        range: Option<(u8, u8)>,
        lowest: Option<i32>,
    ) {
        if index == 6 {
            if let Some(evaluated) = self.rules.evaluate(current) {
                self.out.push((*current, evaluated));
            }
            return;
        }
        // Prune when the lowest pitch so far can no longer be undercut and is not the bass.
        if let Some(low) = lowest {
            let remaining = self.min_below.get(index).copied().unwrap_or(i32::MAX);
            if low <= remaining && u8::try_from(low.rem_euclid(12)).ok() != Some(self.rules.bass) {
                return;
            }
        }
        // Muted.
        if let Some(slot) = current.get_mut(index) {
            *slot = StringState::Muted;
        }
        self.dfs(index + 1, current, range, lowest);
        let options = self.choices.get(index).cloned().unwrap_or_default();
        for choice in options {
            let next_range = match choice.state {
                StringState::Fretted { .. } => match range {
                    None => Some((choice.fret, choice.fret)),
                    Some((lo, hi)) => {
                        let (lo, hi) = (lo.min(choice.fret), hi.max(choice.fret));
                        if hi - lo > self.rules.span_limit {
                            continue;
                        }
                        Some((lo, hi))
                    }
                },
                _ => range,
            };
            let midi = match choice.state {
                StringState::Open => self.rules.sounding_open(index),
                _ => self.rules.tuning.get(index).copied().unwrap_or(0) + i32::from(choice.fret),
            };
            if let Some(slot) = current.get_mut(index) {
                *slot = choice.state;
            }
            self.dfs(
                index + 1,
                current,
                next_range,
                Some(lowest.map_or(midi, |l| l.min(midi))),
            );
        }
        if let Some(slot) = current.get_mut(index) {
            *slot = StringState::Muted;
        }
    }
}

fn build_search(rules: &Rules) -> Search<'_> {
    let mut choices: [Vec<Choice>; 6] = Default::default();
    for (index, list) in choices.iter_mut().enumerate() {
        let open = rules.sounding_open(index);
        if rules.allowed & (1u16 << open.rem_euclid(12)) != 0 {
            list.push(Choice {
                state: StringState::Open,
                fret: rules.capo,
            });
        }
        let base = rules.tuning.get(index).copied().unwrap_or(0);
        for fret in (rules.capo + 1)..=rules.frets {
            let midi = base + i32::from(fret);
            if rules.allowed & (1u16 << midi.rem_euclid(12)) != 0
                && let Ok(f) = Fret::new(fret)
            {
                list.push(Choice {
                    state: StringState::Fretted { fret: f },
                    fret,
                });
            }
        }
    }
    let mut min_below = [i32::MAX; 7];
    for index in (0..6).rev() {
        let here = rules.sounding_open(index);
        let next = min_below.get(index + 1).copied().unwrap_or(i32::MAX);
        if let Some(slot) = min_below.get_mut(index) {
            *slot = here.min(next);
        }
    }
    Search {
        rules,
        choices,
        min_below,
        out: Vec::new(),
    }
}

/// All voicings for a chord on an instrument (FB-5.1), ordered Open, Barre and CAGED, Other;
/// inside a group best score first; at most `options.max_results`. When nothing meets FB-5.4
/// the result is `NoneFound` with the relaxations not yet switched on (FB-5.6).
///
/// ```
/// use mackey_theory::{Chord, ChordQuality};
/// use mackey_voicing::{voicings_for, Instrument, VoicingOptions, VoicingResult};
/// let f = Chord::new("F".parse().unwrap(), ChordQuality::Major);
/// let result = voicings_for(&f, &Instrument::default(), &VoicingOptions::default());
/// let VoicingResult::Found(list) = result else { panic!("F has voicings") };
/// let capo = Instrument::default().capo;
/// let shapes: Vec<String> = list.iter().map(|v| v.fret_string(capo)).collect();
/// assert!(shapes.contains(&"xx3211".to_owned()));
/// assert!(shapes.contains(&"133211".to_owned()));
/// ```
#[must_use]
pub fn voicings_for(
    chord: &Chord,
    instrument: &Instrument,
    options: &VoicingOptions,
) -> VoicingResult {
    let rules = Rules::new(*chord, instrument, options);
    let mut all = curated_voicings(*chord, instrument, &rules);
    let mut seen: HashSet<[StringState; 6]> = all.iter().map(|v| v.strings).collect();

    let mut search = build_search(&rules);
    let mut current = [StringState::Muted; 6];
    search.dfs(0, &mut current, None, None);
    let curated_tuning = is_curated_tuning(instrument);
    for (strings, evaluated) in search.out {
        if !seen.insert(strings) {
            continue;
        }
        let open_position = evaluated.open_position;
        let barre = !evaluated.fingering.barres.is_empty();
        let kind = if open_position {
            VoicingKind::Open
        } else if barre {
            VoicingKind::Barre
        } else {
            VoicingKind::Other
        };
        let lowest = evaluated.lowest_pressed.unwrap_or(rules.capo);
        let name = if curated_tuning {
            match kind {
                VoicingKind::Open => "Open-position voicing".to_owned(),
                VoicingKind::Barre => format!("Barre voicing, fret {lowest}"),
                _ => format!("Voicing, fret {lowest}"),
            }
        } else {
            "Found by search".to_owned()
        };
        all.push(
            Build {
                chord: *chord,
                capo: instrument.capo,
                strings,
                evaluated,
                kind,
                origin: Origin::Search,
                name,
                bonus: 0,
            }
            .finish(),
        );
    }

    if all.is_empty() {
        let mut relax = Vec::new();
        if !options.allow_stretch {
            relax.push(RelaxOption::AllowStretches);
        }
        if !options.allow_omitted_fifth {
            relax.push(RelaxOption::AllowOmittedFifth);
        }
        return VoicingResult::NoneFound { options: relax };
    }
    VoicingResult::Found(rank(all, options.max_results))
}

fn strings_code(strings: &[StringState; 6]) -> [u8; 6] {
    strings.map(|s| match s {
        StringState::Muted => 0,
        StringState::Open => 1,
        StringState::Fretted { fret } => fret.value() + 2,
    })
}

fn group_rank(group: VoicingGroup) -> u8 {
    match group {
        VoicingGroup::Open => 0,
        VoicingGroup::BarreAndCaged => 1,
        VoicingGroup::Other => 2,
    }
}

/// Orders by group, then score (descending), then lowest fret, then the strings; then caps.
/// The cap is shared out so a crowded group cannot push another group off the list.
fn rank(mut all: Vec<Voicing>, max: usize) -> Vec<Voicing> {
    all.sort_by(|a, b| {
        group_rank(a.group)
            .cmp(&group_rank(b.group))
            .then(b.score.cmp(&a.score))
            .then(a.lowest_fret.cmp(&b.lowest_fret))
            .then_with(|| strings_code(&a.strings).cmp(&strings_code(&b.strings)))
    });
    let caps = [max.div_ceil(3), max.div_ceil(2), max];
    let mut taken = [0usize; 3];
    let mut out = Vec::new();
    for v in all {
        let g = usize::from(group_rank(v.group));
        let cap = caps.get(g).copied().unwrap_or(max);
        if out.len() >= max {
            break;
        }
        if let Some(count) = taken.get_mut(g)
            && *count < cap
        {
            *count += 1;
            out.push(v);
        }
    }
    out
}
