//! Audio for Mackey Guitar Tutor: pitch detection and tuner logic, a plucked-string synth and
//! the cpal device glue that runs them.
//!
//! * [`dsp`] is pure: it needs no device, thread or clock and is fully unit tested.
//! * [`io`] is the thin glue that opens cpal streams and moves samples through `rtrb` rings.
//!
//! The real-time rules from `AGENTS.md` apply: the output callback only drains a command queue
//! and calls [`dsp::Synth::render`] (no allocation, locks, logging or I/O); tuner analysis runs
//! on its own thread, never in a callback; audio never leaves the device.

pub mod dsp;
mod error;
pub mod io;

pub use error::AudioError;
