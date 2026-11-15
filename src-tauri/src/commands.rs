//! Tauri commands. Each is a thin wrapper: call the core, map the error, return data.

use crate::error::AppError;
use crate::guard::guard;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// Static facts about the running app, shown in the UI shell.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct AppInfo {
    /// Product name.
    pub name: String,
    /// Crate version of `mackey-app`.
    pub version: String,
    /// Operating system the app was built for (`macos`, `ios`, `linux`, ...).
    pub os: String,
}

impl AppInfo {
    /// The info for this build.
    #[must_use]
    pub fn current() -> Self {
        Self {
            name: "Mackey Guitar Tutor".to_owned(),
            version: env!("CARGO_PKG_VERSION").to_owned(),
            os: std::env::consts::OS.to_owned(),
        }
    }
}

/// Command `app_info`: name, version and platform of the running app.
///
/// # Errors
///
/// Never fails today; the `Result` is the shape every command has (see [`guard`]).
#[tauri::command]
pub fn app_info() -> Result<AppInfo, AppError> {
    guard(|| Ok(AppInfo::current()))
}

/// Names of every registered command, in registration order. The capability file and the
/// UI's command layer must stay in step with this list.
#[cfg(test)]
pub(crate) const COMMAND_NAMES: &[&str] = &["app_info"];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn app_info_describes_this_build() {
        let info = app_info().unwrap();
        assert_eq!(info.name, "Mackey Guitar Tutor");
        assert_eq!(info.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(info.os, std::env::consts::OS);
    }

    #[test]
    fn command_names_are_unique() {
        let unique: BTreeSet<_> = COMMAND_NAMES.iter().collect();
        assert_eq!(unique.len(), COMMAND_NAMES.len());
    }

    /// The UI's command layer (`ui/src/ipc/tauri.ts`) must name every command it invokes.
    #[test]
    fn ui_command_layer_names_every_command() {
        let source = include_str!("../../ui/src/ipc/tauri.ts");
        for name in COMMAND_NAMES {
            assert!(
                source.contains(&format!("'{name}'")),
                "{name} missing from tauri.ts"
            );
        }
    }
}
