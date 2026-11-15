//! Karplus-Strong plucked-string synth.
//!
//! Everything is allocated in [`Synth::new`]; [`Synth::render`] and [`Synth::apply`] never
//! allocate, lock, block or log, so the cpal output callback may call them (real-time rules in
//! `AGENTS.md`). The random excitation uses a seeded generator owned by the synth, so the
//! output is a deterministic function of the command stream.

use std::f32::consts::PI;

/// Quietest note the delay lines are sized for.
const MIN_HZ: f32 = 30.0;
/// Reference tones are plucked again this often (TU-4).
const REFERENCE_REPLUCK_SECONDS: f32 = 1.5;
/// Time for a plucked string to fall 60 dB.
const DECAY_SECONDS: f32 = 3.0;
/// Loop gain used to fade a released voice quickly.
const RELEASE_GAIN: f32 = 0.85;
/// A voice whose envelope falls under this (about -100 dBFS) is finished.
const SILENCE: f32 = 1.0e-5;
/// Smoothing coefficient for volume changes (about 10 ms), so they do not click.
const VOLUME_SMOOTHING: f32 = 0.002;

/// A message for the synth. Fixed size and `Copy`, so it travels through an `rtrb` queue with
/// no heap data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SynthCommand {
    /// Pluck a string. `delay_samples` counts from the start of the block in which the
    /// command is consumed, so a strum is one `NoteOn` per string with growing delays.
    NoteOn {
        /// Handle for [`SynthCommand::NoteOff`].
        id: u32,
        /// Pitch in Hz (clamped to 30 Hz up to a quarter of the sample rate).
        frequency_hz: f32,
        /// Pluck strength `0.0..=1.0`.
        velocity: f32,
        /// Samples to wait before the note starts.
        delay_samples: u32,
    },
    /// Damp every voice started with `id`.
    NoteOff {
        /// The id given to `NoteOn`.
        id: u32,
    },
    /// Silence everything now, including notes still waiting for their delay.
    StopAll,
    /// Master volume `0.0..=1.0`, linear.
    SetVolume(f32),
    /// Mute without losing the volume setting.
    SetMute(bool),
    /// Start a reference tone: plucked now and again every 1.5 s until switched off (TU-4).
    ReferenceToneOn {
        /// Handle for the tone's voices.
        id: u32,
        /// Pitch in Hz.
        frequency_hz: f32,
    },
    /// Stop re-plucking the reference tone and damp it.
    ReferenceToneOff,
}

#[derive(Debug)]
struct Voice {
    buf: Vec<f32>,
    len: usize,
    idx: usize,
    delay: u32,
    active: bool,
    id: u32,
    prev: f32,
    ap_in: f32,
    ap_out: f32,
    eta: f32,
    gain: f32,
    env: f32,
    age: u32,
    started: u64,
}

impl Voice {
    fn new(capacity: usize) -> Self {
        Self {
            buf: vec![0.0; capacity],
            len: 2,
            idx: 0,
            delay: 0,
            active: false,
            id: 0,
            prev: 0.0,
            ap_in: 0.0,
            ap_out: 0.0,
            eta: 0.0,
            gain: 0.0,
            env: 0.0,
            age: 0,
            started: 0,
        }
    }

    /// Phase delay in samples of the first-order allpass `(eta + z^-1) / (1 + eta z^-1)` at
    /// angular frequency `w`.
    fn allpass_delay(eta: f32, w: f32) -> f32 {
        let phase = -(w.sin()).atan2(eta + w.cos()) + (eta * w.sin()).atan2(1.0 + eta * w.cos());
        -phase / w
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "one voice start needs all of these values"
    )]
    fn start(
        &mut self,
        sr: f32,
        id: u32,
        freq: f32,
        velocity: f32,
        delay: u32,
        order: u64,
        seed: &mut u32,
    ) {
        let freq = freq.clamp(MIN_HZ, sr * 0.25);
        let period = sr / freq;
        // Loop delay = line length + 0.5 (two-point average) + allpass delay; the allpass takes
        // up the fraction, corrected for its frequency-dependent phase delay.
        let len = ((period - 1.0).floor() as usize).clamp(2, self.buf.len());
        let w = 2.0 * PI * freq / sr;
        let want = period - 0.5 - len as f32;
        let mut frac = want.clamp(0.1, 1.9);
        for _ in 0..3 {
            let eta = (1.0 - frac) / (1.0 + frac);
            frac += want - Self::allpass_delay(eta, w);
            frac = frac.clamp(0.1, 1.9);
        }
        self.eta = (1.0 - frac) / (1.0 + frac);
        self.len = len;
        self.idx = 0;
        self.delay = delay;
        self.active = true;
        self.id = id;
        self.prev = 0.0;
        self.ap_in = 0.0;
        self.ap_out = 0.0;
        self.env = 1.0;
        self.age = 0;
        self.started = order;
        self.gain = 10.0_f32.powf(-3.0 / (DECAY_SECONDS * freq));

        // Excitation: noise through a one-pole lowpass, DC removed, peak set by velocity.
        let mut lp = 0.0_f32;
        let mut sum = 0.0_f32;
        for slot in &mut self.buf[..len] {
            *seed ^= *seed << 13;
            *seed ^= *seed >> 17;
            *seed ^= *seed << 5;
            let white = (*seed >> 8) as f32 / 8_388_608.0 - 1.0;
            lp += 0.6 * (white - lp);
            *slot = lp;
            sum += lp;
        }
        let mean = sum / len as f32;
        let mut peak = 1.0e-9_f32;
        for slot in &mut self.buf[..len] {
            *slot -= mean;
            peak = peak.max(slot.abs());
        }
        let scale = 0.5 * velocity / peak;
        for slot in &mut self.buf[..len] {
            *slot *= scale;
        }
    }

    fn add_into(&mut self, out: &mut [f32]) {
        for o in out {
            if self.delay > 0 {
                self.delay -= 1;
                continue;
            }
            let x = self.buf[self.idx];
            let avg = 0.5 * (x + self.prev) * self.gain;
            self.prev = x;
            let y = self.eta * avg + self.ap_in - self.eta * self.ap_out;
            self.ap_in = avg;
            self.ap_out = y;
            self.buf[self.idx] = y;
            self.idx += 1;
            if self.idx == self.len {
                self.idx = 0;
            }
            *o += x;
            self.env = (self.env * 0.9995).max(x.abs());
            self.age = self.age.saturating_add(1);
            if self.env < SILENCE && self.age as usize > 2 * self.len {
                self.active = false;
                return;
            }
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ReferenceTone {
    id: u32,
    frequency_hz: f32,
    countdown: u32,
}

/// The synth: mono, 16 voices by default, oldest voice stolen when all are busy.
#[derive(Debug)]
pub struct Synth {
    sr: f32,
    voices: Vec<Voice>,
    setting: f32,
    volume: f32,
    volume_target: f32,
    muted: bool,
    seed: u32,
    order: u64,
    reference: Option<ReferenceTone>,
    repluck_samples: u32,
}

impl Synth {
    /// A synth for `sample_rate` with `voices` voices (at least one), everything allocated now.
    #[must_use]
    pub fn new(sample_rate: u32, voices: usize) -> Self {
        let sr = sample_rate.max(8_000) as f32;
        let capacity = (sr / MIN_HZ).ceil() as usize + 4;
        Self {
            sr,
            voices: (0..voices.max(1)).map(|_| Voice::new(capacity)).collect(),
            setting: 1.0,
            volume: 1.0,
            volume_target: 1.0,
            muted: false,
            seed: 0x9E37_79B9,
            order: 0,
            reference: None,
            repluck_samples: (REFERENCE_REPLUCK_SECONDS * sr) as u32,
        }
    }

    /// The sample rate in Hz.
    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.sr as u32
    }

    /// True while any voice is sounding or waiting to start.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.voices.iter().any(|v| v.active)
    }

    /// Applies one command. Invalid values (non-finite or non-positive frequency, NaN
    /// velocity or volume) are ignored.
    pub fn apply(&mut self, cmd: SynthCommand) {
        match cmd {
            SynthCommand::NoteOn {
                id,
                frequency_hz,
                velocity,
                delay_samples,
            } => {
                self.pluck(id, frequency_hz, velocity, delay_samples);
            }
            SynthCommand::NoteOff { id } => self.release(id),
            SynthCommand::StopAll => {
                self.reference = None;
                for v in &mut self.voices {
                    v.active = false;
                }
            }
            SynthCommand::SetVolume(v) => {
                if v.is_finite() {
                    self.setting = v.clamp(0.0, 1.0);
                    self.retarget();
                }
            }
            SynthCommand::SetMute(m) => {
                self.muted = m;
                self.retarget();
            }
            SynthCommand::ReferenceToneOn { id, frequency_hz } => {
                if let Some(old) = self.reference.take() {
                    self.release(old.id);
                }
                if frequency_hz.is_finite() && frequency_hz > 0.0 {
                    self.pluck(id, frequency_hz, 0.8, 0);
                    self.reference = Some(ReferenceTone {
                        id,
                        frequency_hz,
                        countdown: self.repluck_samples,
                    });
                }
            }
            SynthCommand::ReferenceToneOff => {
                if let Some(old) = self.reference.take() {
                    self.release(old.id);
                }
            }
        }
    }

    fn retarget(&mut self) {
        self.volume_target = if self.muted { 0.0 } else { self.setting };
    }

    fn release(&mut self, id: u32) {
        for v in self.voices.iter_mut().filter(|v| v.active && v.id == id) {
            v.gain = RELEASE_GAIN;
        }
    }

    fn pluck(&mut self, id: u32, freq: f32, velocity: f32, delay: u32) {
        if !(freq.is_finite() && freq > 0.0 && velocity.is_finite()) {
            return;
        }
        let slot = self.voices.iter().position(|v| !v.active).or_else(|| {
            self.voices
                .iter()
                .enumerate()
                .min_by_key(|(_, v)| v.started)
                .map(|(i, _)| i)
        });
        // `voices` always holds at least one voice, so a slot exists.
        let Some(i) = slot else { return };
        self.order += 1;
        let order = self.order;
        let sr = self.sr;
        let mut seed = self.seed;
        self.voices[i].start(
            sr,
            id,
            freq,
            velocity.clamp(0.0, 1.0),
            delay,
            order,
            &mut seed,
        );
        self.seed = seed;
    }

    /// Renders `out.len()` mono samples, overwriting `out`. Allocation-free.
    pub fn render(&mut self, out: &mut [f32]) {
        out.fill(0.0);
        let mut off = 0;
        while off < out.len() {
            let mut n = out.len() - off;
            if let Some(r) = &self.reference {
                n = n.min(r.countdown.max(1) as usize);
            }
            for v in self.voices.iter_mut().filter(|v| v.active) {
                v.add_into(&mut out[off..off + n]);
            }
            if let Some(r) = self.reference {
                let left = r.countdown.saturating_sub(n as u32);
                if left == 0 {
                    self.pluck(r.id, r.frequency_hz, 0.8, 0);
                    self.reference = Some(ReferenceTone {
                        countdown: self.repluck_samples,
                        ..r
                    });
                } else {
                    self.reference = Some(ReferenceTone {
                        countdown: left,
                        ..r
                    });
                }
            }
            off += n;
        }
        for s in out.iter_mut() {
            self.volume += (self.volume_target - self.volume) * VOLUME_SMOOTHING;
            *s = soft_limit(*s) * self.volume;
        }
    }
}

/// Linear up to 0.6, then a tanh knee that stays under 1.0.
fn soft_limit(x: f32) -> f32 {
    let a = x.abs();
    if a <= 0.6 {
        x
    } else {
        x.signum() * (0.6 + 0.4 * ((a - 0.6) / 0.4).tanh())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SR: u32 = 48_000;

    fn note(id: u32, hz: f32, delay: u32) -> SynthCommand {
        SynthCommand::NoteOn {
            id,
            frequency_hz: hz,
            velocity: 0.8,
            delay_samples: delay,
        }
    }

    fn render(s: &mut Synth, n: usize) -> Vec<f32> {
        let mut v = vec![0.0; n];
        s.render(&mut v);
        v
    }

    fn peak(v: &[f32]) -> f32 {
        v.iter().fold(0.0, |m, x| m.max(x.abs()))
    }

    #[test]
    fn silent_until_a_note_is_played() {
        let mut s = Synth::new(SR, 16);
        assert!(!s.is_active());
        assert_eq!(peak(&render(&mut s, 512)), 0.0);
        assert_eq!(s.sample_rate(), SR);
    }

    #[test]
    fn note_on_sounds_and_decays_to_silence() {
        let mut s = Synth::new(SR, 16);
        s.apply(note(1, 110.0, 0));
        assert!(peak(&render(&mut s, 4800)) > 0.1);
        let _ = render(&mut s, SR as usize * 7);
        assert!(peak(&render(&mut s, 4800)) < 1e-4);
        assert!(!s.is_active(), "voice freed after silence");
    }

    #[test]
    fn output_is_deterministic() {
        let run = || {
            let mut s = Synth::new(SR, 16);
            s.apply(note(1, 196.0, 0));
            s.apply(note(2, 82.41, 100));
            render(&mut s, 9_999)
        };
        assert_eq!(run(), run());
    }

    #[test]
    fn delay_is_sample_exact() {
        for delay in [0_u32, 1, 100, 1441, 3000] {
            let mut s = Synth::new(SR, 16);
            s.apply(note(1, 220.0, delay));
            let out = render(&mut s, 5000);
            let first = out.iter().position(|v| v.abs() > 0.0).expect("sounds");
            assert!(
                (first as i64 - i64::from(delay)).abs() <= 1,
                "delay {delay}: {first}"
            );
        }
    }

    #[test]
    fn delay_spans_render_calls() {
        let mut s = Synth::new(SR, 4);
        s.apply(note(1, 220.0, 700));
        assert_eq!(peak(&render(&mut s, 512)), 0.0);
        let out = render(&mut s, 512);
        assert_eq!(out.iter().position(|v| *v != 0.0), Some(188));
    }

    #[test]
    fn note_off_damps_quickly() {
        let mut s = Synth::new(SR, 4);
        s.apply(note(7, 110.0, 0));
        let _ = render(&mut s, 2400);
        s.apply(SynthCommand::NoteOff { id: 7 });
        let _ = render(&mut s, 24_000);
        assert!(peak(&render(&mut s, 2400)) < 1e-3);
        s.apply(SynthCommand::NoteOff { id: 99 });
    }

    #[test]
    fn stop_all_clears_pending_notes_too() {
        let mut s = Synth::new(SR, 4);
        s.apply(note(1, 110.0, 0));
        s.apply(note(2, 220.0, 5000));
        s.apply(SynthCommand::StopAll);
        assert!(!s.is_active());
        assert_eq!(peak(&render(&mut s, 8000)), 0.0);
    }

    #[test]
    fn volume_and_mute() {
        let mut s = Synth::new(SR, 4);
        s.apply(note(1, 110.0, 0));
        s.apply(SynthCommand::SetVolume(0.0));
        let _ = render(&mut s, 12_000);
        assert!(peak(&render(&mut s, 1000)) < 1e-3, "volume 0");
        s.apply(SynthCommand::SetVolume(f32::NAN));
        s.apply(SynthCommand::SetVolume(2.0));
        s.apply(SynthCommand::SetMute(true));
        s.apply(note(2, 110.0, 0));
        let _ = render(&mut s, 12_000);
        assert!(peak(&render(&mut s, 1000)) < 1e-3, "muted");
        s.apply(SynthCommand::SetMute(false));
        let _ = render(&mut s, 12_000);
        assert!(
            peak(&render(&mut s, 1000)) > 0.01,
            "unmuted, volume clamped to 1"
        );
    }

    #[test]
    fn sixteen_voices_stay_under_full_scale() {
        let mut s = Synth::new(SR, 16);
        for i in 0..16 {
            s.apply(SynthCommand::NoteOn {
                id: i,
                frequency_hz: 82.41 * 1.2_f32.powi(i as i32 % 12),
                velocity: 1.0,
                delay_samples: 0,
            });
        }
        for _ in 0..20 {
            let out = render(&mut s, 2048);
            assert!(out.iter().all(|v| v.is_finite() && v.abs() <= 1.0));
        }
    }

    #[test]
    fn the_oldest_voice_is_stolen() {
        let mut s = Synth::new(SR, 2);
        s.apply(note(1, 110.0, 0));
        s.apply(note(2, 220.0, 0));
        s.apply(note(3, 330.0, 0));
        assert!(s.voices.iter().any(|v| v.id == 3) && s.voices.iter().all(|v| v.id != 1));
    }

    #[test]
    fn invalid_commands_are_ignored() {
        let mut s = Synth::new(SR, 2);
        for hz in [0.0, -5.0, f32::NAN, f32::INFINITY] {
            s.apply(note(1, hz, 0));
            s.apply(SynthCommand::ReferenceToneOn {
                id: 1,
                frequency_hz: hz,
            });
        }
        s.apply(SynthCommand::NoteOn {
            id: 1,
            frequency_hz: 100.0,
            velocity: f32::NAN,
            delay_samples: 0,
        });
        assert!(!s.is_active());
        assert_eq!(Synth::new(SR, 0).voices.len(), 1);
    }

    #[test]
    fn extreme_frequencies_are_clamped() {
        let mut s = Synth::new(SR, 2);
        s.apply(note(1, 5.0, 0));
        s.apply(note(2, 40_000.0, 0));
        assert!(render(&mut s, 4096).iter().all(|v| v.is_finite()));
    }

    #[test]
    fn reference_tone_replucks_until_off() {
        let mut s = Synth::new(SR, 16);
        s.apply(SynthCommand::ReferenceToneOn {
            id: 5,
            frequency_hz: 110.0,
        });
        // Short blocks, as a device would call it, over 7 seconds: still sounding at 6.5 s.
        let mut last = Vec::new();
        for _ in 0..(7 * SR as usize / 480) {
            last = render(&mut s, 480);
        }
        assert!(peak(&last) > 0.005, "re-plucked");
        s.apply(SynthCommand::ReferenceToneOn {
            id: 6,
            frequency_hz: 146.83,
        });
        s.apply(SynthCommand::ReferenceToneOff);
        let _ = render(&mut s, SR as usize);
        assert!(peak(&render(&mut s, 4800)) < 1e-3, "off");
    }
}
