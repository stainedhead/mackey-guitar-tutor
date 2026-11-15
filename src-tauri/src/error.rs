//! The error type every command returns to the UI.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// A serialisable error the UI can render: a plain message for the user and optional
/// technical details for the log or a "show details" disclosure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS, thiserror::Error)]
#[error("{message}")]
#[ts(export)]
pub struct AppError {
    /// Plain-language message safe to show to the user.
    pub message: String,
    /// Technical details (never shown by default).
    pub details: Option<String>,
}

impl AppError {
    /// An error with a message and no details.
    ///
    /// # Examples
    ///
    /// ```
    /// let e = mackey_app::AppError::new("No audio output");
    /// assert_eq!(e.message, "No audio output");
    /// assert!(e.details.is_none());
    /// ```
    pub fn new(message: impl Into<String>) -> Self {
        Self {
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
        Self::new("Something went wrong").with_details(details)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialises_with_plain_message_and_details() {
        let e = AppError::new("Nope").with_details("why");
        let json = serde_json::to_value(&e).unwrap();
        assert_eq!(
            json,
            serde_json::json!({"message": "Nope", "details": "why"})
        );
    }

    #[test]
    fn internal_hides_details_in_message() {
        let e = AppError::internal("index out of bounds");
        assert_eq!(e.message, "Something went wrong");
        assert_eq!(e.details.as_deref(), Some("index out of bounds"));
        assert_eq!(e.to_string(), "Something went wrong");
    }
}
