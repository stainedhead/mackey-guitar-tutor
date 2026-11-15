//! Tauri commands. Each is a thin wrapper: take the state, call the matching [`AppState`]
//! method (which holds the logic and is tested without a runtime), return the result.

#![expect(
    clippy::needless_pass_by_value,
    reason = "Tauri injects State, String and struct arguments by value; the signature is fixed by #[tauri::command]"
)]

use crate::error::AppError;
use crate::explorer::{CycleAction, ExplorerOption, ExplorerSnapshot, OverlayRequest};
use crate::guard::guard;
use crate::ports::TunerStartInfo;
use crate::settings::{Settings, SettingsGroup, SettingsPatch, TuningChoice};
use crate::state::{AppState, Emitter, SettingsPane};
use mackey_audio::dsp::{TunerMode, TunerReading, TunerTarget};
use mackey_audio::io::InputDevice;
use mackey_theory::{Degree, Mode, Note, ParseOutcome};
use mackey_voicing::{
    FretRange, FretboardView, LabelsMode, ViewType, VoicingOptions, VoicingResult,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{Emitter as _, State};
use ts_rs::TS;

/// The name of the event the tuner readings travel on.
pub const TUNER_READING_EVENT: &str = "tuner_reading";

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

type Snapshot = Result<ExplorerSnapshot, AppError>;

/// Command `app_info`: name, version and platform of the running app.
///
/// # Errors
///
/// Never fails today; the `Result` is the shape every command has (see [`guard`]).
#[tauri::command]
pub fn app_info() -> Result<AppInfo, AppError> {
    guard(|| Ok(AppInfo::current()))
}

/// Command `get_snapshot`: the current explorer state.
///
/// # Errors
/// See [`AppState::get_snapshot`].
#[tauri::command]
pub fn get_snapshot(state: State<'_, AppState>, visible: Option<FretRange>) -> Snapshot {
    state.get_snapshot(visible)
}

/// Command `fretboard_view`: just the neck.
///
/// # Errors
/// See [`AppState::fretboard_view`].
#[tauri::command]
pub fn fretboard_view(
    state: State<'_, AppState>,
    visible: Option<FretRange>,
) -> Result<FretboardView, AppError> {
    state.fretboard_view(visible)
}

/// Command `select_key`.
///
/// # Errors
/// See [`AppState::select_key`].
#[tauri::command]
pub fn select_key(
    state: State<'_, AppState>,
    tonic: Note,
    mode: Mode,
    visible: Option<FretRange>,
) -> Snapshot {
    state.select_key(tonic, mode, visible)
}

/// Command `select_diatonic_chord`.
///
/// # Errors
/// See [`AppState::select_diatonic_chord`].
#[tauri::command]
pub fn select_diatonic_chord(
    state: State<'_, AppState>,
    degree: Degree,
    visible: Option<FretRange>,
) -> Snapshot {
    state.select_diatonic_chord(degree, visible)
}

/// Command `select_chord`.
///
/// # Errors
/// See [`AppState::select_chord`].
#[tauri::command]
pub fn select_chord(
    state: State<'_, AppState>,
    text: String,
    visible: Option<FretRange>,
) -> Snapshot {
    state.select_chord(&text, visible)
}

/// Command `parse_chord`.
///
/// # Errors
/// See [`AppState::parse_chord`].
#[tauri::command]
pub fn parse_chord(state: State<'_, AppState>, text: String) -> Result<ParseOutcome, AppError> {
    state.parse_chord(&text)
}

/// Command `select_note`.
///
/// # Errors
/// See [`AppState::select_note`].
#[tauri::command]
pub fn select_note(
    state: State<'_, AppState>,
    string: u8,
    fret: u8,
    visible: Option<FretRange>,
) -> Snapshot {
    state.select_note(string, fret, visible)
}

/// Command `select_tuning`.
///
/// # Errors
/// See [`AppState::select_tuning`].
#[tauri::command]
pub fn select_tuning(
    state: State<'_, AppState>,
    tuning: TuningChoice,
    visible: Option<FretRange>,
) -> Snapshot {
    state.select_tuning(tuning, visible)
}

/// Command `set_capo`.
///
/// # Errors
/// See [`AppState::set_capo`].
#[tauri::command]
pub fn set_capo(state: State<'_, AppState>, capo: u8, visible: Option<FretRange>) -> Snapshot {
    state.set_capo(capo, visible)
}

/// Command `set_view_type`.
///
/// # Errors
/// See [`AppState::set_view_type`].
#[tauri::command]
pub fn set_view_type(
    state: State<'_, AppState>,
    view: ViewType,
    visible: Option<FretRange>,
) -> Snapshot {
    state.set_view_type(view, visible)
}

/// Command `set_labels_mode`.
///
/// # Errors
/// See [`AppState::set_labels_mode`].
#[tauri::command]
pub fn set_labels_mode(
    state: State<'_, AppState>,
    labels: LabelsMode,
    visible: Option<FretRange>,
) -> Snapshot {
    state.set_labels_mode(labels, visible)
}

/// Command `set_scale_overlay`.
///
/// # Errors
/// See [`AppState::set_scale_overlay`].
#[tauri::command]
pub fn set_scale_overlay(
    state: State<'_, AppState>,
    overlay: OverlayRequest,
    visible: Option<FretRange>,
) -> Snapshot {
    state.set_scale_overlay(overlay, visible)
}

/// Command `set_option`.
///
/// # Errors
/// See [`AppState::set_option`].
#[tauri::command]
pub fn set_option(
    state: State<'_, AppState>,
    option: ExplorerOption,
    visible: Option<FretRange>,
) -> Snapshot {
    state.set_option(option, visible)
}

/// Command `cycle`.
///
/// # Errors
/// See [`AppState::cycle`].
#[tauri::command]
pub fn cycle(
    state: State<'_, AppState>,
    action: CycleAction,
    visible: Option<FretRange>,
) -> Snapshot {
    state.cycle(action, visible)
}

/// Command `voicings_for`.
///
/// # Errors
/// See [`AppState::voicings_for`].
#[tauri::command]
pub fn voicings_for(
    state: State<'_, AppState>,
    text: String,
    options: Option<VoicingOptions>,
) -> Result<VoicingResult, AppError> {
    state.voicings_for(&text, options)
}

/// Command `fit_selection`.
///
/// # Errors
/// See [`AppState::fit_selection`].
#[tauri::command]
pub fn fit_selection(state: State<'_, AppState>) -> Result<FretRange, AppError> {
    state.fit_selection()
}

/// Command `play_selection`.
///
/// # Errors
/// See [`AppState::play_selection`].
#[tauri::command]
pub fn play_selection(
    state: State<'_, AppState>,
    visible: Option<FretRange>,
) -> Result<(), AppError> {
    state.play_selection(visible)
}

/// Command `play_note`.
///
/// # Errors
/// See [`AppState::play_note`].
#[tauri::command]
pub fn play_note(state: State<'_, AppState>, string: u8, fret: u8) -> Result<(), AppError> {
    state.play_note(string, fret)
}

/// Command `stop_playback`.
///
/// # Errors
/// See [`AppState::stop_playback`].
#[tauri::command]
pub fn stop_playback(state: State<'_, AppState>) -> Result<(), AppError> {
    state.stop_playback()
}

/// Command `list_input_devices`.
///
/// # Errors
/// See [`AppState::list_input_devices`].
#[tauri::command]
pub fn list_input_devices(state: State<'_, AppState>) -> Result<Vec<InputDevice>, AppError> {
    state.list_input_devices()
}

/// Command `tuner_targets`.
///
/// # Errors
/// See [`AppState::tuner_targets`].
#[tauri::command]
pub fn tuner_targets(state: State<'_, AppState>) -> Result<Vec<TunerTarget>, AppError> {
    state.tuner_targets()
}

/// Command `tuner_start`: readings arrive as `tuner_reading` events.
///
/// # Errors
/// See [`AppState::tuner_start`].
#[tauri::command]
pub fn tuner_start(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    device: Option<String>,
) -> Result<TunerStartInfo, AppError> {
    let emit: Emitter = Arc::new(move |reading: &TunerReading| {
        if let Err(e) = app.emit(TUNER_READING_EVENT, reading) {
            tracing::warn!(error = %e, "could not emit a tuner reading");
        }
    });
    state.tuner_start(device, emit)
}

/// Command `tuner_stop`.
///
/// # Errors
/// See [`AppState::tuner_stop`].
#[tauri::command]
pub fn tuner_stop(state: State<'_, AppState>) -> Result<(), AppError> {
    state.tuner_stop()
}

/// Command `tuner_set_mode`.
///
/// # Errors
/// See [`AppState::tuner_set_mode`].
#[tauri::command]
pub fn tuner_set_mode(state: State<'_, AppState>, mode: TunerMode) -> Result<(), AppError> {
    state.tuner_set_mode(mode)
}

/// Command `tuner_reference_tone`.
///
/// # Errors
/// See [`AppState::tuner_reference_tone`].
#[tauri::command]
pub fn tuner_reference_tone(
    state: State<'_, AppState>,
    string: u8,
    on: bool,
) -> Result<(), AppError> {
    state.tuner_reference_tone(string, on)
}

/// Command `open_system_settings`.
///
/// # Errors
/// See [`AppState::open_system_settings`].
#[tauri::command]
pub fn open_system_settings(
    state: State<'_, AppState>,
    pane: SettingsPane,
) -> Result<(), AppError> {
    state.open_system_settings(pane)
}

/// Command `get_settings`.
///
/// # Errors
/// See [`AppState::get_settings`].
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, AppError> {
    state.get_settings()
}

/// Command `set_settings`.
///
/// # Errors
/// See [`AppState::set_settings`].
#[tauri::command]
pub fn set_settings(
    state: State<'_, AppState>,
    patch: SettingsPatch,
) -> Result<Settings, AppError> {
    state.set_settings(patch)
}

/// Command `reset_settings`.
///
/// # Errors
/// See [`AppState::reset_settings`].
#[tauri::command]
pub fn reset_settings(
    state: State<'_, AppState>,
    group: SettingsGroup,
) -> Result<Settings, AppError> {
    state.reset_settings(group)
}

/// Command `take_settings_notice`: the one-time message from a damaged settings file.
///
/// # Errors
/// See [`AppState::take_settings_notice`].
#[tauri::command]
pub fn take_settings_notice(state: State<'_, AppState>) -> Result<Option<String>, AppError> {
    state.take_settings_notice()
}

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
        let unique: BTreeSet<_> = crate::commands_list::COMMAND_NAMES.iter().collect();
        assert_eq!(unique.len(), crate::commands_list::COMMAND_NAMES.len());
    }
}
