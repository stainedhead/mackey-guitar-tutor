//! The fretboard engine for Mackey Guitar Tutor: tunings and the instrument, the curated shape
//! library, voicing search, triads, arpeggios, scale patterns, the selection model with all
//! FB-9 cycling, and the `FretboardView` the UI draws from.
//!
//! Pure and deterministic: no I/O, no threads, no global state. The UI holds view state only;
//! everything a test should check is computed here and sent as data (ADR 0007).
//!
//! ```
//! use mackey_theory::{Chord, ChordQuality};
//! use mackey_voicing::{voicings_for, Instrument, VoicingOptions, VoicingResult};
//!
//! let c = Chord::new("C".parse().unwrap(), ChordQuality::Major);
//! let instrument = Instrument::default();
//! let VoicingResult::Found(list) = voicings_for(&c, &instrument, &VoicingOptions::default())
//!     else { panic!("C has voicings") };
//! assert_eq!(list[0].fret_string(instrument.capo), "x32010");
//! ```

mod arpeggio;
mod error;
mod fingering;
mod instrument;
mod patterns;
mod playback;
mod rules;
mod search;
mod selection;
pub mod shapes;
mod triads;
mod view;
mod voicing;

pub use arpeggio::{
    Arpeggio, ArpeggioDirection, ArpeggioNote, arpeggio, arpeggio_positions,
    default_arpeggio_position,
};
pub use error::VoicingError;
pub use fingering::assign_fingering;
pub use instrument::{
    Bpm, Capo, Fret, FretCount, Handedness, Instrument, Inversion, StringNumber, StringSet, Tuning,
    TuningPreset, TuningWarning,
};
pub use patterns::{
    FollowChoice, FollowReason, PatternKind, PatternNote, ScalePattern, full_neck, scale_for_chord,
    scale_patterns,
};
pub use playback::{
    PlanKind, PlannedNote, PlaybackPlan, STRUM_RING_MS, STRUM_SPACING_MS, playback_plan,
    playback_plan_cached,
};
pub use search::voicings_for;
pub use selection::{
    Action, Applied, CapoNames, Context, Focus, LabelsMode, Notice, ScaleOverlay, Selection,
    ViewType, VoicingCache, announce, apply, apply_cached, instrument_changed,
    instrument_changed_cached, spoken_chord, spoken_note,
};
pub use triads::{triad_ladder, triad_note, triads};
pub use view::{
    CapoLabel, FretRange, FretboardView, Inlay, ListPosition, MarkerKind, NoteRole, OffRange,
    PositionLabels, StringLabel, StringMarker, Tooltip, ViewPosition, ViewRequest,
    VoicingListEntry, fretboard_view, fretboard_view_cached,
};
pub use voicing::{
    Barre, CagedShape, Fingering, Origin, RelaxOption, StringState, TriadResult, Voicing,
    VoicingGroup, VoicingKind, VoicingOptions, VoicingResult,
};
