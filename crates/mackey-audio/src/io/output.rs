//! The cpal output stream: a callback that drains a command queue and renders the synth.
//!
//! The stream lives on its own thread (cpal streams are not `Send` on every platform), so
//! [`OutputHandle`] is `Send + Sync` and the app can keep it in shared state.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample, Stream, StreamConfig};
use rtrb::{Consumer, Producer, RingBuffer};

use super::devices::{open_output_device, output_config};
use super::pump::OutputCore;
use super::select::map_cpal_error;
use crate::dsp::{Synth, SynthCommand};
use crate::error::AudioError;

/// Commands that fit in the queue before new ones are dropped.
const QUEUE_CAPACITY: usize = 512;
/// Frames rendered per synth call inside the callback.
const SCRATCH_FRAMES: usize = 4096;
/// Voices the synth is built with.
const VOICES: usize = 16;

#[derive(Debug, Default)]
struct Shared {
    dropped: AtomicU32,
    failed: AtomicBool,
}

/// An open output stream. Dropping it (or [`OutputHandle::stop`]) closes the stream.
#[derive(Debug)]
pub struct OutputHandle {
    commands: Mutex<Producer<SynthCommand>>,
    shared: Arc<Shared>,
    sample_rate: u32,
    stop: Option<Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl OutputHandle {
    /// Opens the default output device (f32, i16, i32 or u16 samples at 48 or 44.1 kHz, mono
    /// duplicated across channels) and starts the stream.
    ///
    /// # Errors
    /// [`AudioError::NoOutputDevice`] when there is no output device (E-FB-8), or another
    /// [`AudioError`] naming what failed.
    pub fn start() -> Result<Self, AudioError> {
        let (producer, consumer) = RingBuffer::new(QUEUE_CAPACITY);
        let shared = Arc::new(Shared::default());
        let (ready_tx, ready_rx) = mpsc::channel();
        let (stop_tx, stop_rx) = mpsc::channel();
        let worker_shared = Arc::clone(&shared);
        let thread = thread::Builder::new()
            .name("mackey-output".into())
            .spawn(move || run(consumer, &worker_shared, &ready_tx, &stop_rx))
            .map_err(|e| AudioError::StreamBuild(format!("starting the output thread: {e}")))?;
        match ready_rx.recv() {
            Ok(Ok(sample_rate)) => Ok(Self {
                commands: Mutex::new(producer),
                shared,
                sample_rate,
                stop: Some(stop_tx),
                thread: Some(thread),
            }),
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(e)
            }
            Err(_) => {
                let _ = thread.join();
                Err(AudioError::StreamBuild(
                    "the output thread ended early".into(),
                ))
            }
        }
    }

    /// Queues a command for the audio callback. Never blocks.
    ///
    /// # Errors
    /// [`AudioError::DeviceUnavailable`] when the queue is full; the command is dropped and
    /// counted in [`OutputHandle::dropped_commands`].
    pub fn send(&self, command: SynthCommand) -> Result<(), AudioError> {
        let mut producer = self.commands.lock().unwrap_or_else(PoisonError::into_inner);
        producer.push(command).map_err(|_| {
            self.shared.dropped.fetch_add(1, Ordering::Relaxed);
            AudioError::DeviceUnavailable("the audio command queue is full".into())
        })
    }

    /// Queues several commands; returns how many were accepted.
    pub fn send_all(&self, commands: impl IntoIterator<Item = SynthCommand>) -> usize {
        commands
            .into_iter()
            .filter(|c| self.send(*c).is_ok())
            .count()
    }

    /// The device sample rate in Hz (needed to build a plan's delays).
    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Commands dropped because the queue was full.
    #[must_use]
    pub fn dropped_commands(&self) -> u32 {
        self.shared.dropped.load(Ordering::Relaxed)
    }

    /// True once the stream reported an error (for example the device was unplugged); the
    /// app should drop this handle and start a new one.
    #[must_use]
    pub fn stream_failed(&self) -> bool {
        self.shared.failed.load(Ordering::Relaxed)
    }

    /// Closes the stream and joins its thread.
    pub fn stop(mut self) {
        self.close();
    }

    fn close(&mut self) {
        self.stop.take();
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for OutputHandle {
    fn drop(&mut self) {
        self.close();
    }
}

fn run(
    consumer: Consumer<SynthCommand>,
    shared: &Arc<Shared>,
    ready: &Sender<Result<u32, AudioError>>,
    stop: &Receiver<()>,
) {
    match open(consumer, shared) {
        Ok((stream, sample_rate)) => {
            tracing::info!(sample_rate, "output stream open");
            let _ = ready.send(Ok(sample_rate));
            // Returns when the handle drops its sender.
            let _ = stop.recv();
            drop(stream);
            tracing::info!("output stream closed");
        }
        Err(e) => {
            let _ = ready.send(Err(e));
        }
    }
}

fn open(
    consumer: Consumer<SynthCommand>,
    shared: &Arc<Shared>,
) -> Result<(Stream, u32), AudioError> {
    let device = open_output_device()?;
    let supported = output_config(&device)?;
    let config = supported.config();
    let core = OutputCore::new(
        Synth::new(config.sample_rate, VOICES),
        consumer,
        usize::from(config.channels),
        SCRATCH_FRAMES,
    );
    let stream = match supported.sample_format() {
        SampleFormat::F32 => build::<f32>(&device, config, core, shared)?,
        SampleFormat::I16 => build::<i16>(&device, config, core, shared)?,
        SampleFormat::I32 => build::<i32>(&device, config, core, shared)?,
        SampleFormat::U16 => build::<u16>(&device, config, core, shared)?,
        other => {
            return Err(AudioError::UnsupportedConfig(format!(
                "sample format {other}"
            )));
        }
    };
    stream
        .play()
        .map_err(|e| match map_cpal_error("starting the output stream", &e) {
            AudioError::StreamBuild(m) => AudioError::StreamPlay(m),
            other => other,
        })?;
    Ok((stream, config.sample_rate))
}

fn build<T>(
    device: &cpal::Device,
    config: StreamConfig,
    mut core: OutputCore,
    shared: &Arc<Shared>,
) -> Result<Stream, AudioError>
where
    T: SizedSample + FromSample<f32> + Send + 'static,
{
    let failed = Arc::clone(shared);
    device
        .build_output_stream::<T, _, _>(
            config,
            // The callback: no allocation, locks, logging or I/O.
            move |data: &mut [T], _| core.fill(data),
            move |_| failed.failed.store(true, Ordering::Relaxed),
            None,
        )
        .map_err(|e| map_cpal_error("building the output stream", &e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_succeeds_or_reports_an_error_never_panics() {
        // CI machines have no sound card: both outcomes are fine.
        match OutputHandle::start() {
            Ok(handle) => {
                assert!(handle.sample_rate() >= 8_000);
                assert!(handle.send(SynthCommand::StopAll).is_ok());
                assert_eq!(handle.send_all([SynthCommand::SetVolume(0.5)]), 1);
                let _ = (handle.dropped_commands(), handle.stream_failed());
                handle.stop();
            }
            Err(e) => assert!(
                matches!(
                    e,
                    AudioError::NoOutputDevice
                        | AudioError::DeviceUnavailable(_)
                        | AudioError::UnsupportedConfig(_)
                        | AudioError::StreamBuild(_)
                        | AudioError::StreamPlay(_)
                ),
                "{e}"
            ),
        }
    }
}
