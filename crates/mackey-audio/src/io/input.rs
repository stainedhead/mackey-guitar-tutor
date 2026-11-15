//! The cpal input stream and the tuner analysis thread.
//!
//! One worker thread (`mackey-tuner`) builds and owns the stream (cpal streams are not `Send`
//! on every platform), reads the ring the input callback fills, runs [`Tuner::push_block`] and
//! hands every reading to the app's callback. The input callback itself only mixes down to
//! mono and pushes into an `rtrb` ring. Audio is analysed in memory and never written
//! anywhere (ADR 0006, NF-7.1).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample, Stream, StreamConfig};
use rtrb::{Consumer, Producer, RingBuffer};

use super::devices::{input_config, open_input_device};
use super::pump::{ZeroWatch, mixdown, pump_input};
use super::select::{InputDevice, MicPermission, map_cpal_error};
use crate::dsp::{DetectorConfig, PitchDetector, Tuner, TunerConfig, TunerNotice, TunerReading};
use crate::error::AudioError;

/// How long the worker sleeps when the ring is empty.
const IDLE_SLEEP: Duration = Duration::from_millis(2);
/// Samples handled per pump iteration.
const SCRATCH: usize = 2048;

type OnReading = Box<dyn Fn(TunerReading) + Send + 'static>;

#[derive(Debug, Default)]
struct Shared {
    stop: AtomicBool,
    stream_failed: AtomicBool,
    permission_denied: AtomicBool,
    overruns: AtomicU32,
}

enum Command {
    Config(Box<TunerConfig>),
    Lock(Option<u8>),
    ResetTicks,
}

/// What [`InputHandle::start`] learned from the device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputInfo {
    /// The device in use (the default when the requested one was missing).
    pub device: InputDevice,
    /// Its sample rate in Hz.
    pub sample_rate: u32,
}

/// A running tuner input: stream, analysis thread and the channel to steer it.
#[derive(Debug)]
pub struct InputHandle {
    shared: Arc<Shared>,
    commands: Sender<Command>,
    thread: Option<JoinHandle<()>>,
    info: InputInfo,
}

impl InputHandle {
    /// Opens the input device with id `device` (or the default; a missing device falls back
    /// to the default with [`TunerNotice::DeviceChanged`]) and starts analysing. Every
    /// reading goes to `on_reading`, on the analysis thread; throttling to about 30 Hz is the
    /// caller's job.
    ///
    /// # Errors
    /// [`AudioError::NoInputDevice`], [`AudioError::PermissionDenied`] or another
    /// [`AudioError`] naming what failed.
    pub fn start(
        device: Option<&str>,
        config: TunerConfig,
        on_reading: impl Fn(TunerReading) + Send + 'static,
    ) -> Result<Self, AudioError> {
        let shared = Arc::new(Shared::default());
        let (ready_tx, ready_rx) = mpsc::channel();
        let (command_tx, command_rx) = mpsc::channel();
        let worker = Worker {
            wanted: device.map(str::to_owned),
            config,
            on_reading: Box::new(on_reading),
            shared: Arc::clone(&shared),
            commands: command_rx,
        };
        let thread = thread::Builder::new()
            .name("mackey-tuner".into())
            .spawn(move || worker.run(&ready_tx))
            .map_err(|e| AudioError::StreamBuild(format!("starting the tuner thread: {e}")))?;
        match ready_rx.recv() {
            Ok(Ok(info)) => Ok(Self {
                shared,
                commands: command_tx,
                thread: Some(thread),
                info,
            }),
            Ok(Err(e)) => {
                let _ = thread.join();
                Err(e)
            }
            Err(_) => {
                let _ = thread.join();
                Err(AudioError::StreamBuild(
                    "the tuner thread ended early".into(),
                ))
            }
        }
    }

    /// The device and sample rate in use.
    #[must_use]
    pub fn info(&self) -> &InputInfo {
        &self.info
    }

    /// Replaces the tuner configuration (mode, targets, A4).
    pub fn set_config(&self, config: TunerConfig) {
        let _ = self.commands.send(Command::Config(Box::new(config)));
    }

    /// Locks the target string (tap) or unlocks with `None`.
    pub fn lock(&self, string: Option<u8>) {
        let _ = self.commands.send(Command::Lock(string));
    }

    /// Forgets which strings have been in tune.
    pub fn reset_ticks(&self) {
        let _ = self.commands.send(Command::ResetTicks);
    }

    /// `Denied` once a full second of exact zeros arrived while the stream was running (the
    /// OS muting a microphone it refused); `Granted` while audio flows.
    #[must_use]
    pub fn permission(&self) -> MicPermission {
        if self.shared.permission_denied.load(Ordering::Relaxed) {
            MicPermission::Denied
        } else {
            MicPermission::Granted
        }
    }

    /// Samples dropped because the analysis thread fell behind.
    #[must_use]
    pub fn overruns(&self) -> u32 {
        self.shared.overruns.load(Ordering::Relaxed)
    }

    /// Stops the stream and joins the analysis thread.
    pub fn stop(mut self) {
        self.close();
    }

    fn close(&mut self) {
        self.shared.stop.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for InputHandle {
    fn drop(&mut self) {
        self.close();
    }
}

/// Holds at most one [`InputHandle`]: the app's tuner on/off switch. Starting while running
/// stops the previous input first; stopping when nothing runs does nothing.
#[derive(Debug, Default)]
pub struct TunerInput {
    current: Mutex<Option<InputHandle>>,
}

impl TunerInput {
    /// Nothing running.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Stops any running input, then starts a new one (see [`InputHandle::start`]).
    ///
    /// # Errors
    /// As [`InputHandle::start`]; nothing is running after an error.
    pub fn start(
        &self,
        device: Option<&str>,
        config: TunerConfig,
        on_reading: impl Fn(TunerReading) + Send + 'static,
    ) -> Result<InputInfo, AudioError> {
        let mut slot = self.current.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some(previous) = slot.take() {
            previous.stop();
        }
        let handle = InputHandle::start(device, config, on_reading)?;
        let info = handle.info().clone();
        *slot = Some(handle);
        Ok(info)
    }

    /// Stops the running input; a no-op when none runs.
    pub fn stop(&self) {
        let taken = self
            .current
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(handle) = taken {
            handle.stop();
        }
    }

    /// True while an input is open.
    #[must_use]
    pub fn is_running(&self) -> bool {
        self.current
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .is_some()
    }

    /// Runs `f` on the open input, if any.
    pub fn with<R>(&self, f: impl FnOnce(&InputHandle) -> R) -> Option<R> {
        self.current
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .as_ref()
            .map(f)
    }
}

struct Session {
    // Dropping the stream closes it; the field is only held.
    _stream: Stream,
    ring: Consumer<f32>,
    info: InputInfo,
    fell_back: bool,
}

struct Worker {
    wanted: Option<String>,
    config: TunerConfig,
    on_reading: OnReading,
    shared: Arc<Shared>,
    commands: Receiver<Command>,
}

impl Worker {
    fn run(self, ready: &Sender<Result<InputInfo, AudioError>>) {
        let outcome = catch_unwind(AssertUnwindSafe(|| self.work(ready)));
        if outcome.is_err() {
            // The panic message went to the panic hook; tell the app the tuner stopped.
            tracing::error!("the tuner analysis thread panicked");
            let notice = TunerReading::notice_only(TunerNotice::AnalysisStopped, 0.0);
            // Called on a fresh unwind guard: a panicking callback must not take the thread
            // down twice.
            let _ = catch_unwind(AssertUnwindSafe(|| (self.on_reading)(notice)));
        }
    }

    fn work(&self, ready: &Sender<Result<InputInfo, AudioError>>) {
        let session = match open_session(self.wanted.as_deref(), &self.shared) {
            Ok(s) => s,
            Err(e) => {
                let _ = ready.send(Err(e));
                return;
            }
        };
        let mut tuner = match make_tuner(self.config.clone(), session.info.sample_rate) {
            Ok(t) => t,
            Err(e) => {
                let _ = ready.send(Err(e));
                return;
            }
        };
        if session.fell_back {
            tuner.queue_notice(TunerNotice::DeviceChanged);
        }
        let _ = ready.send(Ok(session.info.clone()));
        let mut zero = ZeroWatch::new(session.info.sample_rate);
        let mut scratch = vec![0.0_f32; SCRATCH];
        let mut session_slot = Some(session);

        while !self.shared.stop.load(Ordering::Acquire) {
            while let Ok(command) = self.commands.try_recv() {
                match command {
                    Command::Config(c) => tuner.set_config(*c),
                    Command::Lock(s) => tuner.lock(s),
                    Command::ResetTicks => tuner.reset_ticks(),
                }
            }
            if self.shared.stream_failed.swap(false, Ordering::AcqRel) {
                // E-TU-5: the device went away; carry on with the default input.
                tracing::warn!("input device lost; switching to the default input");
                drop(session_slot.take());
                match open_session(None, &self.shared).and_then(|s| {
                    make_tuner(tuner.config().clone(), s.info.sample_rate).map(|t| (s, t))
                }) {
                    Ok((s, mut t)) => {
                        t.queue_notice(TunerNotice::DeviceDisconnectedUsingDefault);
                        zero = ZeroWatch::new(s.info.sample_rate);
                        tuner = t;
                        session_slot = Some(s);
                    }
                    Err(e) => {
                        tracing::error!(error = %e, "no input device to fall back to");
                        (self.on_reading)(TunerReading::notice_only(
                            TunerNotice::AnalysisStopped,
                            0.0,
                        ));
                        return;
                    }
                }
            }
            let Some(live) = session_slot.as_mut() else {
                return;
            };
            let (n, silent) = pump_input(
                &mut live.ring,
                &mut scratch,
                &mut tuner,
                &mut zero,
                &*self.on_reading,
            );
            if silent {
                self.shared.permission_denied.store(true, Ordering::Relaxed);
            } else if n > 0 {
                self.shared
                    .permission_denied
                    .store(false, Ordering::Relaxed);
            }
            if n == 0 {
                thread::sleep(IDLE_SLEEP);
            }
        }
        tracing::info!("tuner input closed");
    }
}

fn make_tuner(config: TunerConfig, sample_rate: u32) -> Result<Tuner, AudioError> {
    let detector = PitchDetector::new(DetectorConfig::new(sample_rate)?)?;
    Ok(Tuner::new(config, detector))
}

fn open_session(wanted: Option<&str>, shared: &Arc<Shared>) -> Result<Session, AudioError> {
    let (device, device_info, fell_back) = open_input_device(wanted)?;
    let supported = input_config(&device)?;
    let config = supported.config();
    // One second of mono samples: plenty for a thread that wakes every few milliseconds.
    let (producer, ring) = RingBuffer::new(config.sample_rate as usize);
    let stream = match supported.sample_format() {
        SampleFormat::F32 => build::<f32>(&device, config, producer, shared)?,
        SampleFormat::I16 => build::<i16>(&device, config, producer, shared)?,
        SampleFormat::I32 => build::<i32>(&device, config, producer, shared)?,
        SampleFormat::U16 => build::<u16>(&device, config, producer, shared)?,
        other => {
            return Err(AudioError::UnsupportedConfig(format!(
                "sample format {other}"
            )));
        }
    };
    stream
        .play()
        .map_err(|e| match map_cpal_error("starting the input stream", &e) {
            AudioError::StreamBuild(m) => AudioError::StreamPlay(m),
            other => other,
        })?;
    tracing::info!(device = %device_info.name, rate = config.sample_rate, "input stream open");
    Ok(Session {
        _stream: stream,
        ring,
        info: InputInfo {
            device: device_info,
            sample_rate: config.sample_rate,
        },
        fell_back,
    })
}

fn build<T>(
    device: &cpal::Device,
    config: StreamConfig,
    mut producer: Producer<f32>,
    shared: &Arc<Shared>,
) -> Result<Stream, AudioError>
where
    T: SizedSample + Send + 'static,
    f32: FromSample<T>,
{
    let channels = usize::from(config.channels);
    let callback_shared = Arc::clone(shared);
    let error_shared = Arc::clone(shared);
    device
        .build_input_stream::<T, _, _>(
            config,
            // The callback: mix down and push; no allocation, locks, logging or I/O.
            move |data: &[T], _| {
                mixdown(data, channels, |s| {
                    if producer.push(s).is_err() {
                        callback_shared.overruns.fetch_add(1, Ordering::Relaxed);
                    }
                });
            },
            move |_| error_shared.stream_failed.store(true, Ordering::Relaxed),
            None,
        )
        .map_err(|e| map_cpal_error("building the input stream", &e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use mackey_theory::ReferencePitch;

    #[test]
    fn stop_without_start_is_a_no_op() {
        let input = TunerInput::new();
        assert!(!input.is_running());
        input.stop();
        input.stop();
        assert!(input.with(InputHandle::overruns).is_none());
    }

    #[test]
    fn start_succeeds_or_reports_an_error_never_panics() {
        // CI machines have no microphone: both outcomes are fine.
        let input = TunerInput::new();
        let cfg = TunerConfig::chromatic(ReferencePitch::default());
        match input.start(None, cfg.clone(), |_| {}) {
            Ok(info) => {
                assert!(info.sample_rate >= 8_000);
                // Starting again stops the first one.
                assert!(
                    input
                        .start(Some("no-such-device"), cfg.clone(), |_| {})
                        .is_ok()
                );
                input.with(|h| {
                    h.lock(Some(2));
                    h.set_config(cfg);
                    h.reset_ticks();
                    let _ = (h.permission(), InputHandle::overruns);
                });
                assert!(input.is_running());
                input.stop();
                assert!(!input.is_running());
            }
            Err(e) => {
                assert!(!input.is_running());
                assert!(
                    matches!(
                        e,
                        AudioError::NoInputDevice
                            | AudioError::PermissionDenied
                            | AudioError::DeviceUnavailable(_)
                            | AudioError::UnsupportedConfig(_)
                            | AudioError::StreamBuild(_)
                            | AudioError::StreamPlay(_)
                    ),
                    "{e}"
                );
            }
        }
    }

    #[test]
    fn make_tuner_validates_the_sample_rate() {
        let cfg = TunerConfig::chromatic(ReferencePitch::default());
        assert!(make_tuner(cfg.clone(), 48_000).is_ok());
        assert!(matches!(
            make_tuner(cfg, 8_000),
            Err(AudioError::InvalidConfig(_))
        ));
    }
}
