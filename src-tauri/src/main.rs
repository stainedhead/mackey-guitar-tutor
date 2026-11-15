//! Desktop entry point. All logic is in the library so iOS can share it.

// Hide the console window on Windows release builds (Windows is deferred but harmless).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    mackey_app::run();
}
