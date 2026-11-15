//! What "Play" sounds for the current selection (FB-8.1, FB-1.7, FR-024).
//!
//! A plan is pure data: pitches with a start delay and a duration. The app hands it to the audio
//! engine; nothing here touches a device. An empty plan means there is nothing to play.

use mackey_theory::{Note, Pitch, Scale};
use serde::Serialize;

use crate::arpeggio::{arpeggio, default_arpeggio_position};
use crate::instrument::{Bpm, Instrument, StringNumber};
use crate::selection::{Focus, Selection, ViewType, VoicingCache};
use crate::triads::triads;
use crate::view::FretRange;
use crate::voicing::{StringState, Voicing};

/// Milliseconds between the strings of a strummed chord, low to high (FB-8.1).
pub const STRUM_SPACING_MS: u32 = 30;

/// How long each strummed string rings.
pub const STRUM_RING_MS: u32 = 1500;

/// What kind of thing is played.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum PlanKind {
    /// A voicing or triad strummed from the lowest string to the highest.
    Strum,
    /// An arpeggio as eighth notes in view order.
    Arpeggio,
    /// The key's notes as an ascending scale.
    Scale,
    /// One clicked note.
    SingleNote,
}

/// One note of a plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct PlannedNote {
    /// The pitch, spelled for the string it is played on.
    pub pitch: Pitch,
    /// Milliseconds after the plan starts.
    pub delay_ms: u32,
    /// How long it sounds.
    pub duration_ms: u32,
}

/// The notes to sound, in start order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct PlaybackPlan {
    /// The kind of plan.
    pub kind: PlanKind,
    /// The notes; empty when there is nothing to play.
    pub notes: Vec<PlannedNote>,
}

impl PlaybackPlan {
    fn empty(kind: PlanKind) -> Self {
        Self {
            kind,
            notes: Vec::new(),
        }
    }
}

/// An eighth note in milliseconds, truncated: `30000 / bpm` (428 at 70 bpm).
fn eighth_ms(tempo: Bpm) -> u32 {
    30_000 / u32::from(tempo.value())
}

/// The start of the `index`-th eighth note, from exact arithmetic truncated once.
fn eighth_start_ms(index: u32, tempo: Bpm) -> u32 {
    index * 30_000 / u32::from(tempo.value())
}

fn strum(voicing: &Voicing, instrument: &Instrument) -> PlaybackPlan {
    let mut notes = Vec::new();
    for (state, string) in voicing.strings.iter().zip(StringNumber::LOW_TO_HIGH) {
        let fret = match state {
            StringState::Muted => continue,
            StringState::Open => instrument.lowest_fret(),
            StringState::Fretted { fret } => *fret,
        };
        if let Ok(pitch) = instrument.pitch_at(string, fret) {
            let index = u32::try_from(notes.len()).unwrap_or(0);
            notes.push(PlannedNote {
                pitch,
                delay_ms: index * STRUM_SPACING_MS,
                duration_ms: STRUM_RING_MS,
            });
        }
    }
    PlaybackPlan {
        kind: PlanKind::Strum,
        notes,
    }
}

/// The lowest MIDI note, within the visible frets, that is the scale's tonic.
fn lowest_tonic(scale: Scale, instrument: &Instrument, visible: Option<FretRange>) -> Option<i32> {
    let from = visible
        .map_or(0, |r| r.from.value())
        .max(instrument.capo.value());
    let to = visible
        .map_or(instrument.frets.value(), |r| r.to.value())
        .min(instrument.frets.value());
    let tonic = scale.tonic.pitch_class();
    (0..6)
        .flat_map(|index| (from..=to).map(move |fret| instrument.midi_at_index(index, fret)))
        .filter(|midi| mackey_theory::PitchClass::new(*midi) == tonic)
        .min()
}

fn octave_of(midi: i32, note: Note) -> i8 {
    let base = i32::from(note.letter.natural_pc().value()) + i32::from(note.accidental.value());
    i8::try_from((midi - base).div_euclid(12) - 1).unwrap_or(0)
}

fn scale_plan(
    sel: &Selection,
    instrument: &Instrument,
    visible: Option<FretRange>,
    tempo: Bpm,
) -> PlaybackPlan {
    let scale = sel.overlay_scale().unwrap_or_else(|| sel.key.scale());
    let Some(start) = lowest_tonic(scale, instrument, visible) else {
        return PlaybackPlan::empty(PlanKind::Scale);
    };
    let degrees = scale.degrees();
    let tonic_pc = scale.tonic.pitch_class();
    let mut wanted: Vec<Note> = degrees.clone();
    wanted.push(scale.tonic);
    let notes = wanted
        .iter()
        .enumerate()
        .filter_map(|(i, note)| {
            let above = i32::from(tonic_pc.semitones_to(note.pitch_class()));
            // The closing tonic is an octave up (semitones_to wraps it to 0).
            let midi = start + if i == degrees.len() { 12 } else { above };
            let pitch = Pitch::new(*note, octave_of(midi, *note)).ok()?;
            let index = u32::try_from(i).ok()?;
            Some(PlannedNote {
                pitch,
                delay_ms: eighth_start_ms(index, tempo),
                duration_ms: eighth_ms(tempo),
            })
        })
        .collect();
    PlaybackPlan {
        kind: PlanKind::Scale,
        notes,
    }
}

/// What Play sounds (FB-8.1): the selected voicing or triad as a strum (low to high, 30 ms
/// apart), the arpeggio as eighth notes at `tempo` in view order, a clicked note alone, and with
/// no chord the key's (or overlay's) notes as an ascending scale from the lowest tonic in the
/// visible range, one octave plus the tonic.
///
/// ```
/// use mackey_theory::parse_chord_symbol;
/// use mackey_voicing::{playback_plan, Action, Bpm, Context, Handedness, Instrument, PlanKind, Selection, apply};
/// let ins = Instrument::default();
/// let h = Handedness::default();
/// let ctx = Context { instrument: &ins, handedness: &h };
/// let c = parse_chord_symbol("C").unwrap();
/// let sel = apply(&Selection::default_g_major(), &ctx, Action::SelectChord(c)).selection;
/// let plan = playback_plan(&sel, &ins, None, Bpm::default());
/// assert_eq!(plan.kind, PlanKind::Strum);
/// let delays: Vec<u32> = plan.notes.iter().map(|n| n.delay_ms).collect();
/// assert_eq!(delays, [0, 30, 60, 90, 120]);
/// ```
#[must_use]
pub fn playback_plan(
    sel: &Selection,
    instrument: &Instrument,
    visible: Option<FretRange>,
    tempo: Bpm,
) -> PlaybackPlan {
    playback_plan_cached(sel, instrument, visible, tempo, &mut VoicingCache::new())
}

/// [`playback_plan`] with a caller-owned voicing cache.
#[must_use]
pub fn playback_plan_cached(
    sel: &Selection,
    instrument: &Instrument,
    visible: Option<FretRange>,
    tempo: Bpm,
    cache: &mut VoicingCache,
) -> PlaybackPlan {
    match (sel.focus, sel.chord, sel.note) {
        (Focus::Note, _, Some((string, fret))) => {
            let notes = instrument
                .pitch_at(string, fret)
                .map(|pitch| PlannedNote {
                    pitch,
                    delay_ms: 0,
                    duration_ms: eighth_ms(tempo) * 2,
                })
                .into_iter()
                .collect();
            PlaybackPlan {
                kind: PlanKind::SingleNote,
                notes,
            }
        }
        (Focus::Chord, Some(chord), _) => match sel.view {
            ViewType::AllTones | ViewType::Voicing => cache
                .get(&chord, instrument, &sel.options)
                .voicings()
                .get(sel.voicing_index)
                .map_or_else(
                    || PlaybackPlan::empty(PlanKind::Strum),
                    |v| strum(v, instrument),
                ),
            ViewType::Triad => {
                let placements = triads(&chord, instrument, sel.string_set, sel.inversion);
                placements
                    .placements()
                    .get(usize::from(sel.position.unwrap_or(0)))
                    .map_or_else(
                        || PlaybackPlan::empty(PlanKind::Strum),
                        |v| strum(v, instrument),
                    )
            }
            ViewType::Arpeggio => {
                let voicing = cache
                    .get(&chord, instrument, &sel.options)
                    .voicings()
                    .get(sel.voicing_index)
                    .cloned();
                let Some(start) = sel
                    .position
                    .or_else(|| default_arpeggio_position(&chord, instrument, voicing.as_ref()))
                else {
                    return PlaybackPlan::empty(PlanKind::Arpeggio);
                };
                let shape = arpeggio(&chord, instrument, start, sel.arpeggio_direction);
                let notes = shape
                    .sequence
                    .iter()
                    .enumerate()
                    .filter_map(|(step, note_index)| {
                        let note = shape.notes.get(*note_index)?;
                        let pitch = instrument.pitch_at(note.string, note.fret).ok()?;
                        Some(PlannedNote {
                            pitch,
                            delay_ms: eighth_start_ms(u32::try_from(step).ok()?, tempo),
                            duration_ms: eighth_ms(tempo),
                        })
                    })
                    .collect();
                PlaybackPlan {
                    kind: PlanKind::Arpeggio,
                    notes,
                }
            }
        },
        _ => scale_plan(sel, instrument, visible, tempo),
    }
}
