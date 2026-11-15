//! The crate's error type. cpal's own errors never appear in a public signature: they are
//! wrapped into these variants with the backend message as text.

use thiserror::Error;

/// Everything that can go wrong when opening devices or building audio values.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AudioError {
    /// No audio output device exists ("No audio output", E-FB-8).
    #[error("no audio output device is available")]
    NoOutputDevice,
    /// No audio input device exists.
    #[error("no audio input device is available")]
    NoInputDevice,
    /// A named device could not be found or opened.
    #[error("audio device unavailable: {0}")]
    DeviceUnavailable(String),
    /// The device offers no usable sample format or rate.
    #[error("unsupported audio configuration: {0}")]
    UnsupportedConfig(String),
    /// Building the stream failed.
    #[error("could not build the audio stream: {0}")]
    StreamBuild(String),
    /// Starting the stream failed.
    #[error("could not start the audio stream: {0}")]
    StreamPlay(String),
    /// A value handed to a constructor is out of range.
    #[error("invalid audio configuration: {0}")]
    InvalidConfig(String),
    /// The operating system refused microphone access (E-TU-2).
    #[error("microphone access was denied")]
    PermissionDenied,
}
