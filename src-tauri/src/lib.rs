//! `mackey-app`: the thin Tauri adapter. It maps commands to core calls and forwards events;
//! no music logic lives here (see `docs/architecture.md`).

mod commands;
mod error;
mod guard;
mod runtime;

pub use commands::{AppInfo, app_info};
pub use error::AppError;
pub use guard::guard;
pub use runtime::run;
