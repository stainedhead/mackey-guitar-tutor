//! Pure audio maths: pitch detection, level metering, the tuner state machine, the
//! plucked-string synth and the playback-plan builder. Nothing here opens a device, spawns a
//! thread or reads a clock: every function is a function of the samples it is given, so every
//! rule is unit tested without hardware.
//!
//! Casts between sample counts, indices and floats are everywhere in DSP code; counts stay far
//! below 2^24 (a 2048-sample window; sample counters are `u64`/`f64`), so the lossy casts are
//! intended and allowed for the whole module.
#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

mod cents;
mod level;
mod pitch;
mod plan;
mod synth;
mod tuner;

pub use cents::{cents_between, cents_from, nearest_pitch};
pub use level::{Level, LevelMeter};
pub use pitch::{DetectorConfig, PitchDetector, PitchEstimate};
pub use plan::{PlaybackPlan, RING_MS, STRUM_GAP_MS, TimedNote};
pub use synth::{Synth, SynthCommand};
pub use tuner::{
    TargetReading, TuneDirection, Tuner, TunerConfig, TunerMode, TunerNotice, TunerReading,
    TunerStatus, TunerTarget, targets_from_pitches,
};
