//! The audio port: what the commands need from the sound card, as a trait so tests run
//! without one (AC-E-FB-8). `RealAudio` wraps `mackey-audio`; `FakeAudio` records or refuses.

use crate::error::AppError;
use mackey_audio::dsp::{SynthCommand, TunerConfig, TunerReading};
use mackey_audio::io::{InputDevice, MicPermission, OutputHandle, TunerInput, list_input_devices};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};
use ts_rs::TS;

/// One note to pluck: frequency, strength and delay from the start of the plan.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SoundNote {
    /// Pitch in Hz.
    pub frequency_hz: f32,
    /// Pluck strength, 0 to 1.
    pub velocity: f32,
    /// Milliseconds after the start.
    pub delay_ms: u32,
}

/// What `tuner_start` tells the UI.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TunerStartInfo {
    /// Whether the microphone is usable.
    pub permission: MicPermission,
    /// The device in use.
    pub device: InputDevice,
    /// The strings to tune to (empty in chromatic mode).
    pub targets: Vec<mackey_audio::dsp::TunerTarget>,
}

/// Callback for tuner readings; runs on the analysis thread and must not touch app state.
pub type ReadingSink = Box<dyn Fn(TunerReading) + Send + 'static>;

/// Everything the commands ask of the audio system. Devices open lazily, so an app on a
/// machine with no device still runs.
pub trait AudioPort: Send {
    /// Plucks the notes (a strum, arpeggio or single note).
    ///
    /// # Errors
    /// `NoAudioOutput` when no output device exists.
    fn play(&mut self, notes: &[SoundNote]) -> Result<(), AppError>;
    /// Silences everything that is playing.
    fn stop(&mut self);
    /// Sets the master volume (0 to 1) and mute.
    fn set_volume(&mut self, volume: f32, mute: bool);
    /// Starts (`Some(hz)`) or stops (`None`) a reference tone.
    ///
    /// # Errors
    /// `NoAudioOutput` when no output device exists.
    fn reference_tone(&mut self, hz: Option<f32>) -> Result<(), AppError>;
    /// Lists microphones and interfaces.
    ///
    /// # Errors
    /// An audio error when the host cannot list devices.
    fn input_devices(&mut self) -> Result<Vec<InputDevice>, AppError>;
    /// Starts the tuner, replacing a running one.
    ///
    /// # Errors
    /// `NoInputDevice` or `MicrophoneDenied`.
    fn tuner_start(
        &mut self,
        device: Option<&str>,
        config: TunerConfig,
        sink: ReadingSink,
    ) -> Result<(InputDevice, MicPermission), AppError>;
    /// Stops the tuner; a no-op when it is not running.
    fn tuner_stop(&mut self);
    /// Changes the running tuner's mode or targets.
    fn tuner_config(&mut self, config: TunerConfig);
    /// Pins a string (tap to lock) or releases it.
    fn tuner_lock(&mut self, string: Option<u8>);
}

/// The real thing: an output stream opened on first use and the tuner input.
#[derive(Debug)]
pub struct RealAudio {
    output: Option<OutputHandle>,
    input: TunerInput,
    volume: f32,
    mute: bool,
    next_id: u32,
}

impl Default for RealAudio {
    fn default() -> Self {
        Self {
            output: None,
            input: TunerInput::new(),
            volume: 0.8,
            mute: false,
            next_id: 1,
        }
    }
}

impl RealAudio {
    fn output(&mut self) -> Result<&OutputHandle, AppError> {
        if self
            .output
            .as_ref()
            .is_some_and(OutputHandle::stream_failed)
        {
            self.output = None;
        }
        if self.output.is_none() {
            let handle = OutputHandle::start()?;
            let _ = handle.send(SynthCommand::SetVolume(self.volume));
            let _ = handle.send(SynthCommand::SetMute(self.mute));
            self.output = Some(handle);
        }
        self.output.as_ref().ok_or_else(AppError::no_audio_output)
    }
}

impl AudioPort for RealAudio {
    fn play(&mut self, notes: &[SoundNote]) -> Result<(), AppError> {
        let first = self.next_id;
        self.next_id = self.next_id.wrapping_add(16);
        let out = self.output()?;
        let rate = f64::from(out.sample_rate());
        out.send(SynthCommand::StopAll)?;
        for (i, n) in notes.iter().enumerate() {
            let samples = f64::from(n.delay_ms) * rate / 1000.0;
            #[expect(
                clippy::cast_possible_truncation,
                clippy::cast_sign_loss,
                reason = "a delay of a few seconds is far below u32::MAX samples"
            )]
            let delay_samples = samples as u32;
            out.send(SynthCommand::NoteOn {
                id: first.wrapping_add(u32::try_from(i).unwrap_or(0)),
                frequency_hz: n.frequency_hz,
                velocity: n.velocity,
                delay_samples,
            })?;
        }
        Ok(())
    }

    fn stop(&mut self) {
        if let Some(out) = &self.output {
            let _ = out.send(SynthCommand::StopAll);
        }
    }

    fn set_volume(&mut self, volume: f32, mute: bool) {
        self.volume = volume;
        self.mute = mute;
        if let Some(out) = &self.output {
            let _ = out.send(SynthCommand::SetVolume(volume));
            let _ = out.send(SynthCommand::SetMute(mute));
        }
    }

    fn reference_tone(&mut self, hz: Option<f32>) -> Result<(), AppError> {
        match hz {
            Some(frequency_hz) => {
                let out = self.output()?;
                out.send(SynthCommand::ReferenceToneOn {
                    id: 0,
                    frequency_hz,
                })?;
            }
            None => {
                if let Some(out) = &self.output {
                    let _ = out.send(SynthCommand::ReferenceToneOff);
                }
            }
        }
        Ok(())
    }

    fn input_devices(&mut self) -> Result<Vec<InputDevice>, AppError> {
        Ok(list_input_devices()?)
    }

    fn tuner_start(
        &mut self,
        device: Option<&str>,
        config: TunerConfig,
        sink: ReadingSink,
    ) -> Result<(InputDevice, MicPermission), AppError> {
        let info = self.input.start(device, config, sink)?;
        let permission = self
            .input
            .with(mackey_audio::io::InputHandle::permission)
            .unwrap_or(MicPermission::Unknown);
        Ok((info.device, permission))
    }

    fn tuner_stop(&mut self) {
        self.input.stop();
    }

    fn tuner_config(&mut self, config: TunerConfig) {
        self.input.with(|h| h.set_config(config));
    }

    fn tuner_lock(&mut self, string: Option<u8>) {
        self.input.with(|h| h.lock(string));
    }
}

/// A recorded call, for tests.
#[derive(Debug, Clone, PartialEq)]
pub enum Call {
    /// `play`.
    Play(Vec<SoundNote>),
    /// `stop`.
    Stop,
    /// `set_volume`.
    Volume(f32, bool),
    /// `reference_tone`.
    Reference(Option<f32>),
    /// `tuner_start` with the device asked for and the config.
    TunerStart(Option<String>, TunerConfig),
    /// `tuner_stop`.
    TunerStop,
    /// `tuner_config`.
    TunerConfig(TunerConfig),
    /// `tuner_lock`.
    TunerLock(Option<u8>),
}

/// Calls a [`FakeAudio`] received, shared so a test can read them after handing the fake to
/// [`crate::AppState`].
pub type CallLog = Arc<Mutex<Vec<Call>>>;

/// The running tuner's sink, shared so a test can push readings through it.
pub type SharedSink = Arc<Mutex<Option<ReadingSink>>>;

/// A test double: either refuses everything like a machine with no devices, or records calls.
#[derive(Default)]
pub struct FakeAudio {
    /// True to behave like a machine with no audio devices.
    pub no_device: bool,
    /// Calls received, in order.
    pub calls: CallLog,
    /// What `tuner_start` reports (`Denied` makes it fail).
    pub permission: Option<MicPermission>,
    /// The sink of the running tuner.
    pub sink: SharedSink,
}

impl FakeAudio {
    /// A machine with no audio at all.
    #[must_use]
    pub fn no_device() -> Self {
        Self {
            no_device: true,
            ..Self::default()
        }
    }

    fn record(&self, call: Call) {
        self.calls
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(call);
    }
}

impl AudioPort for FakeAudio {
    fn play(&mut self, notes: &[SoundNote]) -> Result<(), AppError> {
        if self.no_device {
            return Err(AppError::no_audio_output());
        }
        self.record(Call::Play(notes.to_vec()));
        Ok(())
    }
    fn stop(&mut self) {
        self.record(Call::Stop);
    }
    fn set_volume(&mut self, volume: f32, mute: bool) {
        self.record(Call::Volume(volume, mute));
    }
    fn reference_tone(&mut self, hz: Option<f32>) -> Result<(), AppError> {
        if self.no_device && hz.is_some() {
            return Err(AppError::no_audio_output());
        }
        self.record(Call::Reference(hz));
        Ok(())
    }
    fn input_devices(&mut self) -> Result<Vec<InputDevice>, AppError> {
        if self.no_device {
            return Ok(Vec::new());
        }
        Ok(vec![InputDevice {
            id: "fake".into(),
            name: "Fake microphone".into(),
            is_default: true,
        }])
    }
    fn tuner_start(
        &mut self,
        device: Option<&str>,
        config: TunerConfig,
        sink: ReadingSink,
    ) -> Result<(InputDevice, MicPermission), AppError> {
        if self.no_device {
            return Err(mackey_audio::AudioError::NoInputDevice.into());
        }
        if self.permission == Some(MicPermission::Denied) {
            return Err(mackey_audio::AudioError::PermissionDenied.into());
        }
        self.record(Call::TunerStart(device.map(str::to_owned), config));
        *self
            .sink
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(sink);
        Ok((
            InputDevice {
                id: device.unwrap_or("fake").into(),
                name: "Fake microphone".into(),
                is_default: true,
            },
            self.permission.unwrap_or(MicPermission::Granted),
        ))
    }
    fn tuner_stop(&mut self) {
        *self
            .sink
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = None;
        self.record(Call::TunerStop);
    }
    fn tuner_config(&mut self, config: TunerConfig) {
        self.record(Call::TunerConfig(config));
    }
    fn tuner_lock(&mut self, string: Option<u8>) {
        self.record(Call::TunerLock(string));
    }
}
