//! The panic guard every command body runs inside (AC-NF-24, D-17).

use crate::error::AppError;
use std::panic::{AssertUnwindSafe, catch_unwind};

/// Runs a command body, turning a panic into `AppError { "Something went wrong" }`.
///
/// The panic text goes to `details` and the log, never into `message`. The release profile
/// keeps `panic = "unwind"` so this works; `panic = "abort"` would defeat it.
///
/// # Errors
///
/// Returns the body's own error, or an internal [`AppError`] if the body panicked.
///
/// # Examples
///
/// ```
/// let r: Result<u8, mackey_app::AppError> = mackey_app::guard(|| Ok(7));
/// assert_eq!(r.unwrap(), 7);
/// let r: Result<u8, mackey_app::AppError> = mackey_app::guard(|| panic!("boom"));
/// assert_eq!(r.unwrap_err().message, "Something went wrong");
/// ```
pub fn guard<T>(body: impl FnOnce() -> Result<T, AppError>) -> Result<T, AppError> {
    match catch_unwind(AssertUnwindSafe(body)) {
        Ok(result) => result,
        Err(payload) => {
            let text = panic_text(payload.as_ref());
            tracing::error!(panic = %text, "command panicked");
            Err(AppError::internal(text))
        }
    }
}

fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_owned()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passes_through_ok_and_err() {
        assert_eq!(guard(|| Ok(1)), Ok(1));
        assert_eq!(
            guard::<u8>(|| Err(AppError::new("bad"))),
            Err(AppError::new("bad"))
        );
    }

    type Body = fn() -> Result<u8, AppError>;

    /// Each panic payload type, with the text it should report (AC-NF-24).
    #[test]
    fn panics_become_something_went_wrong() {
        let cases: [(&str, Body, &str); 3] = [
            ("str payload", || panic!("static boom"), "static boom"),
            (
                "string payload",
                || panic!("formatted {}", 42),
                "formatted 42",
            ),
            (
                "other payload",
                || std::panic::panic_any(5_u8),
                "unknown panic",
            ),
        ];
        for (label, body, want_details) in cases {
            let err = guard(body).unwrap_err();
            assert_eq!(err.message, "Something went wrong", "{label}");
            assert_eq!(err.details.as_deref(), Some(want_details), "{label}");
            assert!(
                !err.message.contains(want_details),
                "{label}: payload leaked"
            );
        }
    }
}
