//! The error type every command returns to the UI.

use mackey_audio::AudioError;
use mackey_theory::{ChordParseError, RomanParseError, TheoryError};
use mackey_voicing::VoicingError;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What went wrong, so the UI can branch (show "No audio output" once, offer the microphone
/// settings button) without parsing text. The UI never displays the kind itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum AppErrorKind {
    /// An argument was out of range or text did not parse; state is unchanged.
    Invalid,
    /// No audio output device (E-FB-8).
    NoAudioOutput,
    /// No audio input device.
    NoInputDevice,
    /// The OS refused microphone access (E-TU-2).
    MicrophoneDenied,
    /// The settings file could not be read or written.
    Settings,
    /// A bug: the command panicked or state was corrupt (NF-24).
    Internal,
}

/// A serialisable error the UI can render: a plain message for the user and optional
/// technical details for the log or a "show details" disclosure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, thiserror::Error)]
#[error("{message}")]
#[ts(export)]
pub struct AppError {
    /// What kind of failure this is.
    pub kind: AppErrorKind,
    /// Plain-language message safe to show to the user (no codes, NF-22).
    pub message: String,
    /// Technical details (never shown by default).
    pub details: Option<String>,
}

impl AppError {
    /// An [`AppErrorKind::Invalid`] error with a message and no details.
    ///
    /// # Examples
    ///
    /// ```
    /// let e = mackey_app::AppError::new("No audio output");
    /// assert_eq!(e.message, "No audio output");
    /// assert!(e.details.is_none());
    /// ```
    pub fn new(message: impl Into<String>) -> Self {
        Self::of(AppErrorKind::Invalid, message)
    }

    /// An error of a given kind.
    pub fn of(kind: AppErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            details: None,
        }
    }

    /// The same error with technical details attached.
    #[must_use]
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    /// The generic error shown when a command panics.
    pub fn internal(details: impl Into<String>) -> Self {
        Self::of(AppErrorKind::Internal, "Something went wrong").with_details(details)
    }

    /// "No audio output", shown once per session (E-FB-8).
    #[must_use]
    pub fn no_audio_output() -> Self {
        Self::of(AppErrorKind::NoAudioOutput, "No audio output")
    }
}

impl From<AudioError> for AppError {
    fn from(e: AudioError) -> Self {
        tracing::warn!(error = %e, "audio error");
        match e {
            AudioError::NoOutputDevice => Self::no_audio_output(),
            AudioError::NoInputDevice => {
                Self::of(AppErrorKind::NoInputDevice, "No microphone found")
            }
            AudioError::PermissionDenied => {
                Self::of(AppErrorKind::MicrophoneDenied, "Microphone access is off")
            }
            other => Self::of(AppErrorKind::Internal, "Audio is not available right now")
                .with_details(other.to_string()),
        }
    }
}

impl From<ChordParseError> for AppError {
    fn from(e: ChordParseError) -> Self {
        Self::new(e.to_string())
    }
}

impl From<RomanParseError> for AppError {
    fn from(e: RomanParseError) -> Self {
        Self::new(e.to_string())
    }
}

impl From<VoicingError> for AppError {
    fn from(e: VoicingError) -> Self {
        Self::new(e.to_string())
    }
}

impl From<TheoryError> for AppError {
    fn from(e: TheoryError) -> Self {
        Self::new(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialises_with_kind_message_and_details() {
        let e = AppError::new("Nope").with_details("why");
        let json = serde_json::to_value(&e).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"kind": "Invalid", "message": "Nope", "details": "why"})
        );
    }

    #[test]
    fn internal_hides_details_in_message() {
        let e = AppError::internal("index out of bounds");
        assert_eq!(e.kind, AppErrorKind::Internal);
        assert_eq!(e.message, "Something went wrong");
        assert_eq!(e.details.as_deref(), Some("index out of bounds"));
        assert_eq!(e.to_string(), "Something went wrong");
    }

    /// Core error to kind and plain message (D2).
    #[test]
    fn core_errors_map_to_plain_messages() {
        let cases: [(&str, AppError, AppErrorKind, &str); 5] = [
            (
                "no output",
                AudioError::NoOutputDevice.into(),
                AppErrorKind::NoAudioOutput,
                "No audio output",
            ),
            (
                "no input",
                AudioError::NoInputDevice.into(),
                AppErrorKind::NoInputDevice,
                "No microphone found",
            ),
            (
                "denied",
                AudioError::PermissionDenied.into(),
                AppErrorKind::MicrophoneDenied,
                "Microphone access is off",
            ),
            (
                "stream",
                AudioError::StreamBuild("x".into()).into(),
                AppErrorKind::Internal,
                "Audio is not available right now",
            ),
            (
                "voicing",
                VoicingError::StringOutOfRange(7).into(),
                AppErrorKind::Invalid,
                "",
            ),
        ];
        for (label, err, kind, message) in cases {
            assert_eq!(err.kind, kind, "{label}");
            if !message.is_empty() {
                assert_eq!(err.message, message, "{label}");
            }
        }
        let parse: AppError = "Hm7".parse::<mackey_theory::Chord>().unwrap_err().into();
        assert_eq!(
            parse.message,
            "Not a chord symbol I know: Hm7. Did you mean Bm7?"
        );
        let roman: AppError =
            mackey_theory::roman_to_chord("ii?", &mackey_voicing::Selection::default_g_major().key)
                .unwrap_err()
                .into();
        assert_eq!(roman.kind, AppErrorKind::Invalid);
        let theory: AppError = TheoryError::MidiOutOfRange(999).into();
        assert_eq!(theory.kind, AppErrorKind::Invalid);
    }
}
