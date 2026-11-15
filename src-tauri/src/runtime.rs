//! Tauri runtime glue: builds the app and runs it. Excluded from coverage (D-16): it needs a
//! window system, and all logic it calls is tested elsewhere.

use crate::commands;

/// Starts the app. Desktop `main` and the iOS entry point both call this.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let result = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![commands::app_info])
        .run(tauri::generate_context!());
    if let Err(e) = result {
        tracing::error!(error = %e, "the app failed to start");
        std::process::exit(1);
    }
}
