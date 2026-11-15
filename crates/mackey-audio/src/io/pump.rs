//! The device-free halves of the glue: what the output callback does with its command queue
//! and synth, and what the analysis thread does with the input ring. They take plain
//! `rtrb` rings and slices, so tests drive them with fakes.

use std::time::Duration;

use cpal::{FromSample, Sample};
use rtrb::{Consumer, PopError};

use crate::dsp::{Synth, SynthCommand, Tuner, TunerReading};

/// The output callback's state: drains commands, renders mono, fans out to all channels.
///
/// [`OutputCore::fill`] allocates nothing, takes no lock and does no I/O (real-time rules).
pub(crate) struct OutputCore {
    synth: Synth,
    commands: Consumer<SynthCommand>,
    mono: Vec<f32>,
    channels: usize,
}

impl OutputCore {
    /// `scratch_frames` bounds how many frames are rendered at a time; larger callbacks are
    /// processed in pieces.
    pub(crate) fn new(
        synth: Synth,
        commands: Consumer<SynthCommand>,
        channels: usize,
        scratch_frames: usize,
    ) -> Self {
        Self {
            synth,
            commands,
            mono: vec![0.0; scratch_frames.max(1)],
            channels: channels.max(1),
        }
    }

    /// Fills an interleaved device buffer.
    pub(crate) fn fill<T: Sample + FromSample<f32>>(&mut self, data: &mut [T]) {
        // `PopError::Empty` is the only error; the loop ends when the queue is drained.
        while let Ok(command) = self.commands.pop() {
            self.synth.apply(command);
        }
        let channels = self.channels;
        for chunk in data.chunks_mut(self.mono.len() * channels) {
            let frames = chunk.len() / channels;
            self.synth.render(&mut self.mono[..frames]);
            for (frame, &sample) in chunk.chunks_exact_mut(channels).zip(&self.mono) {
                let converted = T::from_sample(sample);
                frame.fill(converted);
            }
            // A trailing partial frame (should not happen) is silence.
            let tail = chunk.len() - frames * channels;
            if tail > 0 {
                let start = chunk.len() - tail;
                chunk[start..].fill(T::EQUILIBRIUM);
            }
        }
    }
}

/// Averages each interleaved frame of `data` to one sample and hands it to `push`.
#[expect(clippy::cast_precision_loss, reason = "channel counts are tiny")]
pub(crate) fn mixdown<T: Sample>(data: &[T], channels: usize, mut push: impl FnMut(f32))
where
    f32: FromSample<T>,
{
    let channels = channels.max(1);
    let scale = 1.0 / channels as f32;
    for frame in data.chunks_exact(channels) {
        let sum: f32 = frame.iter().map(|&s| f32::from_sample(s)).sum();
        push(sum * scale);
    }
}

/// Detects a microphone the OS has silently muted: an input stream that is running but
/// delivers nothing except exact zeros for a whole second (Q-5).
#[derive(Debug)]
pub(crate) struct ZeroWatch {
    limit: usize,
    run: usize,
}

impl ZeroWatch {
    pub(crate) fn new(sample_rate: u32) -> Self {
        Self {
            limit: sample_rate as usize,
            run: 0,
        }
    }

    /// Feeds a block; true while the current run of exact zeros has reached one second.
    pub(crate) fn feed(&mut self, block: &[f32]) -> bool {
        for &s in block {
            if s == 0.0 {
                self.run = self.run.saturating_add(1);
            } else {
                self.run = 0;
            }
        }
        self.run >= self.limit
    }
}

/// Moves what is waiting in `ring` through `tuner` (in pieces of at most `scratch.len()`),
/// passing each reading to `on_reading`. Stops once `max_samples` have been consumed, so one
/// call never runs for more than about that much audio however fast the producer is (FR-002).
/// Returns the number of samples consumed and whether the stream looks silent.
pub(crate) fn pump_input(
    ring: &mut Consumer<f32>,
    scratch: &mut [f32],
    max_samples: usize,
    tuner: &mut Tuner,
    zero: &mut ZeroWatch,
    on_reading: &dyn Fn(TunerReading),
) -> (usize, bool) {
    let mut total = 0;
    let mut silent = false;
    loop {
        let mut n = 0;
        while n < scratch.len() {
            match ring.pop() {
                Ok(s) => {
                    scratch[n] = s;
                    n += 1;
                }
                Err(PopError::Empty) => break,
            }
        }
        if n == 0 {
            break;
        }
        silent = zero.feed(&scratch[..n]);
        for reading in tuner.push_block(&scratch[..n]) {
            on_reading(reading);
        }
        total += n;
        if n < scratch.len() || total >= max_samples {
            break;
        }
    }
    (total, silent)
}

/// How long the worker sleeps when the ring is empty.
pub(crate) const IDLE_WAIT: Duration = Duration::from_millis(2);
/// How long it yields after a full, bounded step, so a producer that never stops cannot make
/// the thread spin.
pub(crate) const YIELD_WAIT: Duration = Duration::from_millis(1);
/// How long it sleeps once the input is judged not to be paced by a real clock.
pub(crate) const UNPACED_WAIT: Duration = Duration::from_millis(100);

/// Detects an input that delivers audio much faster than real time (an unpaced virtual or null
/// device). Wall-clock time comes from the caller, so tests use a fake clock.
#[derive(Debug)]
pub(crate) struct PaceGuard {
    rate: f64,
    window_start: Option<Duration>,
    samples: usize,
}

impl PaceGuard {
    /// Judged over windows of one second.
    const WINDOW: Duration = Duration::from_secs(1);
    /// More than this multiple of the nominal rate is "not paced". A device that bursts out
    /// a stalled second of backlog stays under it.
    const LIMIT: f64 = 2.5;

    pub(crate) fn new(sample_rate: u32) -> Self {
        Self {
            rate: f64::from(sample_rate),
            window_start: None,
            samples: 0,
        }
    }

    /// Counts `n` samples seen at wall time `now`; true when the finished window ran too fast.
    #[expect(
        clippy::cast_precision_loss,
        reason = "sample counts stay far below 2^52"
    )]
    pub(crate) fn feed(&mut self, n: usize, now: Duration) -> bool {
        let start = *self.window_start.get_or_insert(now);
        self.samples = self.samples.saturating_add(n);
        let elapsed = now.saturating_sub(start);
        if elapsed < Self::WINDOW {
            return false;
        }
        let too_fast = self.samples as f64 > Self::LIMIT * self.rate * elapsed.as_secs_f64();
        self.window_start = Some(now);
        self.samples = 0;
        too_fast
    }
}

/// What one analysis step did and how long the worker should wait before the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Step {
    /// Samples consumed.
    pub(crate) consumed: usize,
    /// The stream has been exact zeros for a second.
    pub(crate) silent: bool,
    /// The input was just judged not paced; the worker tells the user once.
    pub(crate) newly_unpaced: bool,
    /// Sleep before the next step. Never zero after a step that hit the bound.
    pub(crate) wait: Duration,
}

/// The analysis loop's decisions, apart from the device: bounded work per step, the yield
/// policy and the unpaced-input guard (FR-002, FR-005).
#[derive(Debug)]
pub(crate) struct Analyzer {
    scratch: Vec<f32>,
    max_per_step: usize,
    zero: ZeroWatch,
    pace: PaceGuard,
    unpaced: bool,
}

impl Analyzer {
    /// `scratch_len` samples per piece; a step handles about 100 ms of audio at most.
    pub(crate) fn new(sample_rate: u32, scratch_len: usize) -> Self {
        Self {
            scratch: vec![0.0; scratch_len.max(1)],
            max_per_step: (sample_rate as usize / 10).max(1),
            zero: ZeroWatch::new(sample_rate),
            pace: PaceGuard::new(sample_rate),
            unpaced: false,
        }
    }

    /// True once the input was judged not paced; analysis is then off.
    #[cfg(test)]
    pub(crate) fn is_unpaced(&self) -> bool {
        self.unpaced
    }

    /// One bounded pump at wall time `now`.
    pub(crate) fn step(
        &mut self,
        ring: &mut Consumer<f32>,
        tuner: &mut Tuner,
        now: Duration,
        on_reading: &dyn Fn(TunerReading),
    ) -> Step {
        if self.unpaced {
            return Step {
                consumed: 0,
                silent: false,
                newly_unpaced: false,
                wait: UNPACED_WAIT,
            };
        }
        let (consumed, silent) = pump_input(
            ring,
            &mut self.scratch,
            self.max_per_step,
            tuner,
            &mut self.zero,
            on_reading,
        );
        let newly_unpaced = self.pace.feed(consumed, now);
        self.unpaced |= newly_unpaced;
        let wait = if consumed == 0 {
            IDLE_WAIT
        } else if consumed >= self.max_per_step {
            YIELD_WAIT
        } else {
            Duration::ZERO
        };
        Step {
            consumed,
            silent,
            newly_unpaced,
            wait,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::cast_possible_truncation, reason = "test signal maths")]

    use super::*;
    use crate::dsp::{DetectorConfig, PitchDetector, TunerConfig, TunerStatus};
    use mackey_theory::ReferencePitch;
    use rtrb::RingBuffer;
    use std::cell::RefCell;

    fn note(id: u32, hz: f32) -> SynthCommand {
        SynthCommand::NoteOn {
            id,
            frequency_hz: hz,
            velocity: 0.8,
            delay_samples: 0,
        }
    }

    #[test]
    fn output_core_drains_commands_and_fans_out_channels() {
        let (mut tx, rx) = RingBuffer::new(8);
        let mut core = OutputCore::new(Synth::new(48_000, 4), rx, 2, 64);
        let mut buf = vec![0.0_f32; 200 * 2];
        core.fill(&mut buf);
        assert!(buf.iter().all(|v| *v == 0.0), "silent without commands");
        tx.push(note(1, 220.0)).unwrap();
        core.fill(&mut buf);
        assert!(buf.iter().any(|v| *v != 0.0));
        for frame in buf.as_chunks::<2>().0 {
            assert_eq!(frame[0], frame[1], "mono duplicated across channels");
        }
    }

    #[test]
    fn output_core_converts_formats_and_handles_odd_sizes() {
        let (mut tx, rx) = RingBuffer::new(8);
        let mut core = OutputCore::new(Synth::new(48_000, 4), rx, 3, 16);
        tx.push(note(1, 110.0)).unwrap();
        let mut buf = vec![0_i16; 100 * 3 + 2]; // 2 stray samples after the last frame
        core.fill(&mut buf);
        assert!(buf[..300].iter().any(|v| *v != 0));
        assert_eq!(&buf[300..], &[0, 0]);
        // Unsigned formats have their equilibrium at the midpoint.
        let mut u = vec![0_u16; 2 * 8 + 1];
        let (mut tx2, rx2) = RingBuffer::new(2);
        tx2.push(SynthCommand::SetMute(true)).unwrap();
        OutputCore::new(Synth::new(48_000, 1), rx2, 2, 8).fill(&mut u);
        assert_eq!(u[16], 32_768);
    }

    #[test]
    fn mixdown_averages_channels_and_converts() {
        let mut out = Vec::new();
        mixdown(&[0.5_f32, -0.5, 1.0, 0.0], 2, |s| out.push(s));
        assert_eq!(out, [0.0, 0.5]);
        let mut out = Vec::new();
        mixdown(&[16_384_i16, 16_384, 99], 1, |s| out.push(s));
        assert_eq!(out.len(), 3);
        assert!((out[0] - 0.5).abs() < 1e-3);
        // Zero channels is treated as one; a partial frame is dropped.
        let mut count = 0;
        mixdown(&[0.0_f32; 5], 0, |_| count += 1);
        assert_eq!(count, 5);
        let mut count = 0;
        mixdown(&[0.0_f32; 5], 2, |_| count += 1);
        assert_eq!(count, 2);
    }

    #[test]
    fn zero_watch_needs_a_full_second_of_exact_zeros() {
        let mut w = ZeroWatch::new(1000);
        assert!(!w.feed(&[0.0; 999]));
        assert!(w.feed(&[0.0; 1]));
        assert!(!w.feed(&[0.001]), "any signal clears it");
        assert!(!w.feed(&[0.0; 500]));
    }

    #[test]
    fn pump_assembles_blocks_into_readings() {
        let sr = 48_000;
        let (mut tx, mut rx) = RingBuffer::new(20_000);
        let mut tuner = Tuner::new(
            TunerConfig::chromatic(ReferencePitch::default()),
            PitchDetector::new(DetectorConfig::new(sr).unwrap()).unwrap(),
        );
        let mut zero = ZeroWatch::new(sr);
        let got = RefCell::new(Vec::new());
        let on_reading = |r: TunerReading| got.borrow_mut().push(r);
        let mut scratch = vec![0.0; 1000];
        for i in 0..4096_u32 {
            let s = (0.5
                * (2.0 * std::f64::consts::PI * 440.0 * f64::from(i) / f64::from(sr)).sin())
                as f32;
            tx.push(s).unwrap();
        }
        let (n, silent) = pump_input(
            &mut rx,
            &mut scratch,
            usize::MAX,
            &mut tuner,
            &mut zero,
            &on_reading,
        );
        assert_eq!(n, 4096);
        assert!(!silent);
        let got = got.borrow();
        assert_eq!(got.len(), 1 + (4096 - 2048) / 512);
        assert_eq!(got.last().unwrap().status, TunerStatus::Pitch);
        drop(got);
        // An empty ring does nothing; an exact multiple of the scratch size terminates.
        assert_eq!(
            pump_input(
                &mut rx,
                &mut scratch,
                usize::MAX,
                &mut tuner,
                &mut zero,
                &on_reading
            )
            .0,
            0
        );
        for _ in 0..2000 {
            tx.push(0.0).unwrap();
        }
        let (n, _) = pump_input(
            &mut rx,
            &mut scratch,
            usize::MAX,
            &mut tuner,
            &mut zero,
            &on_reading,
        );
        assert_eq!(n, 2000);
    }

    #[test]
    fn pump_reports_a_silent_stream() {
        let sr = 1000; // tiny rate so a second of zeros is cheap; the tuner is not asked
        let (mut tx, mut rx) = RingBuffer::new(4096);
        let mut tuner = Tuner::new(
            TunerConfig::chromatic(ReferencePitch::default()),
            PitchDetector::new(DetectorConfig::new(48_000).unwrap()).unwrap(),
        );
        let mut zero = ZeroWatch::new(sr);
        for _ in 0..1100 {
            tx.push(0.0).unwrap();
        }
        let mut scratch = vec![0.0; 256];
        let (_, silent) = pump_input(
            &mut rx,
            &mut scratch,
            usize::MAX,
            &mut tuner,
            &mut zero,
            &|_| {},
        );
        assert!(silent);
    }

    fn fresh_tuner(sr: u32) -> Tuner {
        Tuner::new(
            TunerConfig::chromatic(ReferencePitch::default()),
            PitchDetector::new(DetectorConfig::new(sr).unwrap()).unwrap(),
        )
    }

    #[test]
    fn fr002_a_step_is_bounded_and_yields_however_fast_the_producer_is() {
        let sr = 48_000;
        let (mut tx, mut rx) = RingBuffer::new(sr as usize);
        let mut tuner = fresh_tuner(sr);
        let mut a = Analyzer::new(sr, 1024);
        while tx.push(0.1).is_ok() {}
        let step = a.step(&mut rx, &mut tuner, Duration::ZERO, &|_| {});
        // About 100 ms (4800 samples), rounded up to a scratch piece.
        assert!(
            step.consumed >= 4800 && step.consumed < 4800 + 1024,
            "{}",
            step.consumed
        );
        assert_eq!(step.wait, YIELD_WAIT);
        let empty = {
            while rx.pop().is_ok() {}
            a.step(&mut rx, &mut tuner, Duration::ZERO, &|_| {})
        };
        assert_eq!((empty.consumed, empty.wait), (0, IDLE_WAIT));
    }

    /// A source at 50x real time with a fake clock: the guard trips within about a second of
    /// wall time, announces it once, and every step before that waited.
    #[test]
    fn fr002_unpaced_input_is_detected_announced_once_and_stops_analysing() {
        let sr = 48_000_u32;
        let (mut tx, mut rx) = RingBuffer::new(sr as usize);
        let mut tuner = fresh_tuner(sr);
        let mut a = Analyzer::new(sr, 1024);
        let readings = RefCell::new(0_usize);
        let on_reading = |_r: TunerReading| *readings.borrow_mut() += 1;
        let mut announced = 0;
        let mut waits_zero_after_full_step = 0;
        // Each loop turn is 1 ms of wall time; the fake device adds 50 ms of audio per turn.
        for ms in 0..1500_u64 {
            for _ in 0..(sr as usize * 50 / 1000) {
                let _ = tx.push(0.2);
            }
            let step = a.step(&mut rx, &mut tuner, Duration::from_millis(ms), &on_reading);
            announced += usize::from(step.newly_unpaced);
            if step.consumed >= 4800 && step.wait.is_zero() {
                waits_zero_after_full_step += 1;
            }
            if a.is_unpaced() && !step.newly_unpaced {
                assert_eq!(step.consumed, 0, "no analysis once unpaced");
                assert!(step.wait >= UNPACED_WAIT);
            }
        }
        assert_eq!(announced, 1);
        assert!(a.is_unpaced());
        assert_eq!(waits_zero_after_full_step, 0, "a full step always yields");
    }

    #[test]
    fn fr002_a_paced_input_and_a_one_second_burst_are_not_flagged() {
        let sr = 48_000_u32;
        let (mut tx, mut rx) = RingBuffer::new(sr as usize);
        let mut tuner = fresh_tuner(sr);
        let mut a = Analyzer::new(sr, 1024);
        // A stalled second arrives at once, then audio flows at the nominal rate.
        while tx.push(0.1).is_ok() {}
        for ms in 0..3000_u64 {
            for _ in 0..(sr as usize / 1000) {
                let _ = tx.push(0.1);
            }
            let step = a.step(&mut rx, &mut tuner, Duration::from_millis(ms), &|_| {});
            assert!(!step.newly_unpaced, "flagged at {ms} ms");
        }
    }
}
