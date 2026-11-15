//! The device-free halves of the glue: what the output callback does with its command queue
//! and synth, and what the analysis thread does with the input ring. They take plain
//! `rtrb` rings and slices, so tests drive them with fakes.

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

/// Moves everything waiting in `ring` through `tuner` (in pieces of at most `scratch.len()`),
/// passing each reading to `on_reading`. Returns the number of samples consumed.
pub(crate) fn pump_input(
    ring: &mut Consumer<f32>,
    scratch: &mut [f32],
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
        if n < scratch.len() {
            break;
        }
    }
    (total, silent)
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
        let (n, silent) = pump_input(&mut rx, &mut scratch, &mut tuner, &mut zero, &on_reading);
        assert_eq!(n, 4096);
        assert!(!silent);
        let got = got.borrow();
        assert_eq!(got.len(), 1 + (4096 - 2048) / 512);
        assert_eq!(got.last().unwrap().status, TunerStatus::Pitch);
        drop(got);
        // An empty ring does nothing; an exact multiple of the scratch size terminates.
        assert_eq!(
            pump_input(&mut rx, &mut scratch, &mut tuner, &mut zero, &on_reading).0,
            0
        );
        for _ in 0..2000 {
            tx.push(0.0).unwrap();
        }
        let (n, _) = pump_input(&mut rx, &mut scratch, &mut tuner, &mut zero, &on_reading);
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
        let (_, silent) = pump_input(&mut rx, &mut scratch, &mut tuner, &mut zero, &|_| {});
        assert!(silent);
    }
}
