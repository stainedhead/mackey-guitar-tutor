//! `mackey-app`: the thin Tauri adapter. It maps commands to core calls and forwards events;
//! no music logic lives here (see `docs/architecture.md`).

pub mod commands;
mod commands_list;
pub mod error;
pub mod explorer;
pub mod guard;
pub mod logging;
pub mod manifest;
pub mod ports;
mod runtime;
pub mod settings;
pub mod state;
pub mod tuner;

#[cfg(test)]
mod bindings;

pub use commands::{AppInfo, app_info};
pub use commands_list::COMMAND_NAMES;
pub use error::{AppError, AppErrorKind};
pub use explorer::{ChordFieldStatus, Explorer, ExplorerSnapshot};
pub use guard::guard;
pub use ports::{AudioPort, FakeAudio, RealAudio};
pub use runtime::run;
pub use settings::Settings;
pub use state::AppState;
