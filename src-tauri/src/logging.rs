//! Logging to a local file: `app_data_dir/logs/mackey.log`, rotated daily, three files kept.
//! Nothing leaves the machine.

use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_appender::rolling::{RollingFileAppender, Rotation};

/// Starts the file logger. Keep the returned guard alive for the life of the app: dropping it
/// flushes and stops the writer thread.
///
/// # Errors
/// A message when the log folder or file cannot be created; the app then runs without a log.
pub fn init(log_dir: &Path) -> Result<WorkerGuard, String> {
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::DAILY)
        .filename_prefix("mackey")
        .filename_suffix("log")
        .max_log_files(3)
        .build(log_dir)
        .map_err(|e| {
            format!(
                "could not create the log file in {}: {e}",
                log_dir.display()
            )
        })?;
    let (writer, guard) = tracing_appender::non_blocking(appender);
    let subscriber = tracing_subscriber::fmt()
        .with_writer(writer)
        .with_ansi(false)
        .with_max_level(tracing::Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .map_err(|e| format!("a logger is already installed: {e}"))?;
    Ok(guard)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The file appears in the log folder and receives what is logged (D2).
    #[test]
    fn writes_a_log_file() {
        let dir = tempfile::tempdir().unwrap();
        let guard = init(dir.path()).unwrap();
        tracing::info!("hello from the logging test");
        drop(guard);
        let mut found = false;
        for entry in std::fs::read_dir(dir.path()).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            assert!(
                name.starts_with("mackey") && name.ends_with("log"),
                "{name}"
            );
            found |= std::fs::read_to_string(&path)
                .unwrap()
                .contains("hello from the logging test");
        }
        assert!(found, "the message reached the file");
        // A second logger cannot be installed in the same process.
        assert!(init(dir.path()).is_err());
    }
}
