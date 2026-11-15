//! App state and the body of every command, free of the Tauri runtime so tests call them
//! directly. Three mutexes, locked in the order explorer, settings, audio; none is ever
//! locked from the audio or analysis thread (those only call the emitter closure).

use crate::error::AppError;
use crate::explorer::{CycleAction, Explorer, ExplorerOption, ExplorerSnapshot, OverlayRequest};
use crate::guard::guard;
use crate::ports::{AudioPort, TunerStartInfo};
use crate::settings::{self, Settings, SettingsGroup, SettingsPatch};
use crate::tuner::Throttle;
use mackey_audio::dsp::{TunerConfig, TunerMode, TunerReading};
use mackey_audio::io::InputDevice;
use mackey_theory::{Degree, Mode, Note};
use mackey_voicing::{
    FretRange, FretboardView, LabelsMode, ViewType, VoicingOptions, VoicingResult, voicings_for,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use ts_rs::TS;

use crate::settings::TuningChoice;

/// Where the tuner's readings go (the Tauri event in the app, a recorder in tests).
pub type Emitter = Arc<dyn Fn(&TunerReading) + Send + Sync>;

/// Locks a mutex, recovering the data when another thread panicked while holding it.
pub fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(|poisoned| {
        tracing::error!("a mutex was poisoned by a panic; recovering its data");
        PoisonError::into_inner(poisoned)
    })
}

/// Which system settings pane to open.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SettingsPane {
    /// Privacy, Microphone.
    Microphone,
}

/// Everything the commands share.
pub struct AppState {
    explorer: Mutex<Explorer>,
    settings: Mutex<Settings>,
    audio: Mutex<Box<dyn AudioPort>>,
    settings_dir: PathBuf,
    notice: Mutex<Option<String>>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState").finish_non_exhaustive()
    }
}

fn now_seconds() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs())
}

impl AppState {
    /// Loads settings from `settings_dir` and builds the explorer from them.
    #[must_use]
    pub fn new(settings_dir: PathBuf, mut audio: Box<dyn AudioPort>) -> Self {
        let loaded = settings::load(&settings_dir, now_seconds());
        audio.set_volume(volume_of(&loaded.settings), loaded.settings.sound.mute);
        Self {
            explorer: Mutex::new(Explorer::new(&loaded.settings)),
            settings: Mutex::new(loaded.settings),
            audio: Mutex::new(audio),
            settings_dir,
            notice: Mutex::new(loaded.notice),
        }
    }

    // ---- explorer ----

    /// `get_snapshot`.
    ///
    /// # Errors
    /// Never fails today.
    pub fn get_snapshot(&self, visible: Option<FretRange>) -> Result<ExplorerSnapshot, AppError> {
        guard(|| Ok(lock(&self.explorer).snapshot(visible)))
    }

    /// `fretboard_view`.
    ///
    /// # Errors
    /// Never fails today.
    pub fn fretboard_view(&self, visible: Option<FretRange>) -> Result<FretboardView, AppError> {
        guard(|| Ok(lock(&self.explorer).view(visible)))
    }

    fn with_explorer(
        &self,
        visible: Option<FretRange>,
        change: impl FnOnce(&mut Explorer) -> Result<(), AppError>,
    ) -> Result<ExplorerSnapshot, AppError> {
        guard(|| {
            let mut explorer = lock(&self.explorer);
            change(&mut explorer)?;
            Ok(explorer.snapshot(visible))
        })
    }

    /// `select_key`.
    ///
    /// # Errors
    /// `Invalid` when no such key exists.
    pub fn select_key(
        &self,
        tonic: Note,
        mode: Mode,
        visible: Option<FretRange>,
    ) -> Result<ExplorerSnapshot, AppError> {
        self.with_explorer(visible, |e| e.select_key(tonic, mode))
    }

    /// `select_diatonic_chord`.
    ///
    /// # Errors
    /// Never fails today.
    pub fn select_diatonic_chord(
        &self,
        degree: Degree,
        visible: Option<FretRange>,
    ) -> Result<ExplorerSnapshot, AppError> {
        self.with_explorer(visible, |e| {
            e.select_diatonic(degree);
            Ok(())
        })
    }

    /// `select_chord`: unparsable text still returns `Ok` with the error in the panel.
    ///
    /// # Errors
    /// Never fails for bad text (FB-4.4, NF-23).
    pub fn select_chord(
        &self,
        text: &str,
        visible: Option<FretRange>,
    ) -> Result<ExplorerSnapshot, AppError> {
        self.with_explorer(visible, |e| {
            e.select_chord_text(text);
            Ok(())
        })
    }

    /// `parse_chord`.
    ///
    /// # Errors
    /// Never fails today.
    pub fn parse_chord(&self, text: &str) -> Result<mackey_theory::ParseOutcome, AppError> {
        guard(|| Ok(lock(&self.explorer).parse(text)))
    }

    /// `select_note`.
    ///
    /// # Errors
    /// `Invalid` for string 0 or 7 or a fret past the end.
    pub fn select_note(
        &self,
        string: u8,
        fret: u8,
        visible: Option<FretRange>,
    ) -> Result<ExplorerSnapshot, AppError> {
        self.with_explorer(visible, |e| e.select_note(string, fret))
    }

    /// `select_tuning`.
    ///
    /// # Errors
    /// `Invalid` for a custom tuning that does not parse.
    pub fn select_tuning(
        &self,
        choice: TuningChoice,
        visible: Option<FretRange>,
    ) -> Result<ExplorerSnapshot, AppError> {
        self.with_explorer(visible, |e| e.select_tuning(choice))
    }

    /// `set_capo`.
    ///
    /// # Errors
    /// `Invalid` above fret 12.
    pub fn set_capo(
        &self,
        capo: u8,
        visible: Option<FretRange>,
    ) -> Result<ExplorerSnapshot, AppError> {
        self.with_explorer(visible, |e| e.set_capo(capo))
    }

    /// `set_view_type`.
    ///
    /// # Errors
    /// Never fails today.
    pub fn set_view_type(
        &self,
        view: ViewType,
        visible: Option<FretRange>,
    ) -> Result<ExplorerSnapshot, AppError> {
        self.with_explorer(visible, |e| {
            e.set_view_type(view);
            Ok(())
        })
    }

    /// `set_labels_mode`.
    ///
    /// # Errors
    /// Never fails today.
    pub fn set_labels_mode(
        &self,
        labels: LabelsMode,
        visible: Option<FretRange>,
    ) -> Result<ExplorerSnapshot, AppError> {
        self.with_explorer(visible, |e| {
            e.set_labels(labels);
            Ok(())
        })
    }

    /// `set_scale_overlay`.
    ///
    /// # Errors
    /// Never fails today.
    pub fn set_scale_overlay(
        &self,
        request: OverlayRequest,
        visible: Option<FretRange>,
    ) -> Result<ExplorerSnapshot, AppError> {
        self.with_explorer(visible, |e| {
            e.set_overlay(request);
            Ok(())
        })
    }

    /// `set_option`.
    ///
    /// # Errors
    /// Never fails today.
    pub fn set_option(
        &self,
        option: ExplorerOption,
        visible: Option<FretRange>,
    ) -> Result<ExplorerSnapshot, AppError> {
        self.with_explorer(visible, |e| {
            e.set_option(option);
            Ok(())
        })
    }

    /// `cycle`.
    ///
    /// # Errors
    /// Never fails today.
    pub fn cycle(
        &self,
        action: CycleAction,
        visible: Option<FretRange>,
    ) -> Result<ExplorerSnapshot, AppError> {
        self.with_explorer(visible, |e| {
            e.cycle(action);
            Ok(())
        })
    }

    /// `voicings_for`: the voicing list for any chord text, without selecting it.
    ///
    /// # Errors
    /// `Invalid` when the text is not a chord.
    pub fn voicings_for(
        &self,
        text: &str,
        options: Option<VoicingOptions>,
    ) -> Result<VoicingResult, AppError> {
        guard(|| {
            let explorer = lock(&self.explorer);
            let chord = explorer.parse(text).chord.ok_or_else(|| {
                AppError::new(format!("Not a chord symbol I know: {}", text.trim()))
            })?;
            Ok(voicings_for(
                &chord,
                explorer.instrument(),
                &options.unwrap_or(explorer.selection().options),
            ))
        })
    }

    /// `fit_selection`: the range that shows every highlighted note (FB-1.5).
    ///
    /// # Errors
    /// Never fails today.
    pub fn fit_selection(&self) -> Result<FretRange, AppError> {
        guard(|| Ok(lock(&self.explorer).view(None).fit_range))
    }

    // ---- sound ----

    /// `play_selection` (FB-8.1).
    ///
    /// # Errors
    /// `NoAudioOutput` when the machine has no output device; the explorer keeps working.
    pub fn play_selection(&self, visible: Option<FretRange>) -> Result<(), AppError> {
        guard(|| {
            let notes = lock(&self.explorer).play_notes(visible);
            lock(&self.audio).play(&notes)
        })
    }

    /// `play_note` (FB-1.7).
    ///
    /// # Errors
    /// `Invalid` for a bad position, `NoAudioOutput` with no device.
    pub fn play_note(&self, string: u8, fret: u8) -> Result<(), AppError> {
        guard(|| {
            let hz = lock(&self.explorer).note_frequency(string, fret)?;
            lock(&self.audio).play(&[crate::ports::SoundNote {
                frequency_hz: hz,
                velocity: 0.8,
                delay_ms: 0,
            }])
        })
    }

    /// `stop_playback`.
    ///
    /// # Errors
    /// Never fails today.
    pub fn stop_playback(&self) -> Result<(), AppError> {
        guard(|| {
            lock(&self.audio).stop();
            Ok(())
        })
    }

    // ---- tuner ----

    /// `list_input_devices`.
    ///
    /// # Errors
    /// An audio error when the host cannot list devices.
    pub fn list_input_devices(&self) -> Result<Vec<InputDevice>, AppError> {
        guard(|| lock(&self.audio).input_devices())
    }

    /// `tuner_targets`: the strings to tune to for the current instrument (capo and A4 applied).
    ///
    /// # Errors
    /// Never fails today.
    pub fn tuner_targets(&self) -> Result<Vec<mackey_audio::dsp::TunerTarget>, AppError> {
        guard(|| Ok(lock(&self.explorer).tuner_targets()))
    }

    /// `tuner_start`: stops selection playback (TU-7), opens the microphone and starts the
    /// analysis; readings reach `emit` at about 30 Hz.
    ///
    /// # Errors
    /// `NoInputDevice` or `MicrophoneDenied`.
    pub fn tuner_start(
        &self,
        device: Option<String>,
        emit: Emitter,
    ) -> Result<TunerStartInfo, AppError> {
        guard(|| {
            let (targets, a4) = {
                let explorer = lock(&self.explorer);
                (explorer.tuner_targets(), explorer.a4())
            };
            let wanted = device.or_else(|| lock(&self.settings).tuner.input_device.clone());
            let config = TunerConfig {
                a4,
                mode: TunerMode::Chromatic,
                targets: targets.clone(),
            };
            // Only the analysis thread touches this mutex; it is not app state.
            let throttle = Mutex::new(Throttle::default());
            let sink: crate::ports::ReadingSink = Box::new(move |reading| {
                if lock(&throttle).accept(&reading) {
                    emit(&reading);
                }
            });
            let mut audio = lock(&self.audio);
            audio.stop();
            let (device, permission) = audio.tuner_start(wanted.as_deref(), config, sink)?;
            Ok(TunerStartInfo {
                permission,
                device,
                targets,
            })
        })
    }

    /// `tuner_stop`; a no-op when the tuner is not running.
    ///
    /// # Errors
    /// Never fails today.
    pub fn tuner_stop(&self) -> Result<(), AppError> {
        guard(|| {
            lock(&self.audio).tuner_stop();
            Ok(())
        })
    }

    /// `tuner_set_mode`: chromatic, or string mode with an optional tap-to-lock.
    ///
    /// # Errors
    /// Never fails today.
    pub fn tuner_set_mode(&self, mode: TunerMode) -> Result<(), AppError> {
        guard(|| {
            let (targets, a4) = {
                let explorer = lock(&self.explorer);
                (explorer.tuner_targets(), explorer.a4())
            };
            let mut audio = lock(&self.audio);
            audio.tuner_config(TunerConfig { a4, mode, targets });
            audio.tuner_lock(match mode {
                TunerMode::Strings { locked } => locked,
                TunerMode::Chromatic => None,
            });
            Ok(())
        })
    }

    /// `tuner_reference_tone`: plays the open string's pitch (capo applied) from the synth.
    ///
    /// # Errors
    /// `Invalid` for a string outside 1 to 6, `NoAudioOutput` with no device.
    pub fn tuner_reference_tone(&self, string: u8, on: bool) -> Result<(), AppError> {
        guard(|| {
            if !on {
                return lock(&self.audio).reference_tone(None);
            }
            let hz = lock(&self.explorer)
                .tuner_targets()
                .iter()
                .find(|t| t.string == string)
                .map(|t| t.frequency_hz)
                .ok_or_else(|| AppError::new("There is no such string"))?;
            #[expect(clippy::cast_possible_truncation, reason = "audio frequencies fit f32")]
            lock(&self.audio).reference_tone(Some(hz as f32))
        })
    }

    /// `open_system_settings`.
    ///
    /// # Errors
    /// `Invalid` with instructions where the app cannot open the pane itself.
    pub fn open_system_settings(&self, pane: SettingsPane) -> Result<(), AppError> {
        guard(|| open_pane(pane))
    }

    // ---- settings ----

    /// `get_settings`.
    ///
    /// # Errors
    /// Never fails today.
    pub fn get_settings(&self) -> Result<Settings, AppError> {
        guard(|| Ok(lock(&self.settings).clone()))
    }

    /// The one-time notice from loading a damaged settings file, if any.
    ///
    /// # Errors
    /// Never fails today.
    pub fn take_settings_notice(&self) -> Result<Option<String>, AppError> {
        guard(|| Ok(lock(&self.notice).take()))
    }

    fn update_settings(&self, change: impl FnOnce(&mut Settings)) -> Result<Settings, AppError> {
        guard(|| {
            let mut explorer = lock(&self.explorer);
            let mut current = lock(&self.settings);
            let old = current.clone();
            change(&mut current);
            let new = current.clone();
            explorer.apply_settings(&old, &new)?;
            settings::save(&self.settings_dir, &new)?;
            lock(&self.audio).set_volume(volume_of(&new), new.sound.mute);
            Ok(new)
        })
    }

    /// `set_settings`: applies at once and saves.
    ///
    /// # Errors
    /// `Invalid` for a default tuning that cannot be built; `Settings` when saving fails.
    pub fn set_settings(&self, patch: SettingsPatch) -> Result<Settings, AppError> {
        self.update_settings(|s| s.apply(patch))
    }

    /// `reset_settings`: one group back to its defaults (AC-FR-028).
    ///
    /// # Errors
    /// As [`AppState::set_settings`].
    pub fn reset_settings(&self, group: SettingsGroup) -> Result<Settings, AppError> {
        self.update_settings(|s| s.reset(group))
    }
}

fn volume_of(settings: &Settings) -> f32 {
    f32::from(settings.sound.volume) / 100.0
}

#[cfg(target_os = "macos")]
fn open_pane(pane: SettingsPane) -> Result<(), AppError> {
    let SettingsPane::Microphone = pane;
    std::process::Command::new("open")
        .arg("x-apple.systempreferences:com.apple.preference.security?Privacy_Microphone")
        .status()
        .map_err(|e| {
            AppError::of(
                crate::error::AppErrorKind::Internal,
                "Could not open System Settings",
            )
            .with_details(e.to_string())
        })?;
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn open_pane(pane: SettingsPane) -> Result<(), AppError> {
    let SettingsPane::Microphone = pane;
    Err(AppError::new(
        "Open Settings, then Privacy, then Microphone, and turn Mackey on",
    ))
}
