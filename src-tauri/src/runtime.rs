//! Tauri runtime glue: builds the app and runs it. Excluded from coverage (D-16): it needs a
//! window system, and all logic it calls is tested elsewhere.

use crate::commands_list::{for_each_command, handler_of};
use crate::ports::RealAudio;
use crate::state::AppState;
use tauri::Manager;

/// Starts the app. Desktop `main` and the iOS entry point both call this.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let result = tauri::Builder::default()
        .setup(|app| {
            let data_dir = app.path().app_data_dir()?;
            // The guard must live as long as the app so the log writer keeps running.
            let log_guard = crate::logging::init(&data_dir.join("logs"));
            match log_guard {
                Ok(guard) => {
                    app.manage(LogGuard(guard));
                }
                Err(e) => tracing::warn!(error = %e, "logging to a file is off"),
            }
            app.manage(AppState::new(data_dir, Box::<RealAudio>::default()));
            Ok(())
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { .. } = event
                && let Some(state) = window.try_state::<AppState>()
            {
                let _ = state.tuner_stop();
            }
        })
        .invoke_handler(for_each_command!(handler_of))
        .run(tauri::generate_context!());
    if let Err(e) = result {
        tracing::error!(error = %e, "the app failed to start");
        std::process::exit(1);
    }
}

/// Keeps the logging worker alive in Tauri's managed state.
struct LogGuard(#[allow(dead_code)] tracing_appender::non_blocking::WorkerGuard);
