//! Settings: the FR-028 subset, stored as JSON in the app data folder (G-4, 08 §6).
//!
//! Plain data plus load, save and validation. Nothing here knows about music beyond the types
//! it stores; the explorer reads instrument settings from here.

use crate::error::{AppError, AppErrorKind};
use mackey_theory::ReferencePitch;
use mackey_voicing::{FretCount, Handedness, LabelsMode, TuningPreset};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use ts_rs::TS;

/// File name inside the app data folder.
pub const SETTINGS_FILE: &str = "settings.json";

/// The version written to the file.
pub const SETTINGS_VERSION: u32 = 1;

/// The tuning new sessions start in (LH-2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum TuningChoice {
    /// One of the nine presets.
    Preset(TuningPreset),
    /// Six pitch names low to high, for example `["D2", "A2", "D3", "G3", "B3", "E4"]`.
    Custom(Vec<String>),
}

impl Default for TuningChoice {
    fn default() -> Self {
        Self::Preset(TuningPreset::Standard)
    }
}

/// The colour theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum Theme {
    /// Follow the system.
    #[default]
    System,
    /// Always light.
    Light,
    /// Always dark.
    Dark,
}

/// Instrument defaults (LH-1, LH-2, FB-1.1).
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct InstrumentSettings {
    /// Left-handed and string flip.
    pub handedness: Handedness,
    /// Frets on the neck, 19 to 24.
    pub frets: FretCount,
    /// The tuning a new session starts in.
    pub default_tuning: TuningChoice,
    /// Allow a 5-fret span in voicings.
    pub allow_stretches: bool,
}

/// Display options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct DisplaySettings {
    /// Colour theme.
    pub theme: Theme,
    /// The labels mode a new session starts in.
    pub labels_default: LabelsMode,
    /// Show fret numbers under the neck.
    pub show_fret_numbers: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self {
            theme: Theme::System,
            labels_default: LabelsMode::NoteName,
            show_fret_numbers: true,
        }
    }
}

/// Sound options.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct SoundSettings {
    /// Output volume, 0 to 100 percent.
    pub volume: u8,
    /// Silence everything.
    pub mute: bool,
}

impl Default for SoundSettings {
    fn default() -> Self {
        Self {
            volume: 80,
            mute: false,
        }
    }
}

/// Tuner options.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct TunerSettings {
    /// The input device id; `None` is the system default.
    pub input_device: Option<String>,
    /// A4 in Hz (415 to 466).
    pub reference_a4: ReferencePitch,
    /// True once the microphone explainer was shown (E-TU-1).
    pub mic_explained: bool,
}

/// Every setting, grouped as the Settings screen groups them.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[serde(default)]
#[ts(export)]
pub struct Settings {
    /// File format version.
    pub version: u32,
    /// Instrument group.
    pub instrument: InstrumentSettings,
    /// Display group.
    pub display: DisplaySettings,
    /// Sound group.
    pub sound: SoundSettings,
    /// Tuner group.
    pub tuner: TunerSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            version: SETTINGS_VERSION,
            instrument: InstrumentSettings::default(),
            display: DisplaySettings::default(),
            sound: SoundSettings::default(),
            tuner: TunerSettings::default(),
        }
    }
}

/// A settings group, for "Reset" buttons (AC-FR-028).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum SettingsGroup {
    /// Instrument.
    Instrument,
    /// Display.
    Display,
    /// Sound.
    Sound,
    /// Tuner.
    Tuner,
}

/// Replacement values for some groups; absent groups stay as they are.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct SettingsPatch {
    /// New instrument group.
    pub instrument: Option<InstrumentSettings>,
    /// New display group.
    pub display: Option<DisplaySettings>,
    /// New sound group.
    pub sound: Option<SoundSettings>,
    /// New tuner group.
    pub tuner: Option<TunerSettings>,
}

impl Settings {
    /// Applies a patch, clamping values that have a range (volume 0 to 100).
    pub fn apply(&mut self, patch: SettingsPatch) {
        if let Some(v) = patch.instrument {
            self.instrument = v;
        }
        if let Some(v) = patch.display {
            self.display = v;
        }
        if let Some(mut v) = patch.sound {
            v.volume = v.volume.min(100);
            self.sound = v;
        }
        if let Some(v) = patch.tuner {
            self.tuner = v;
        }
    }

    /// Restores one group to its defaults and leaves the others alone (AC-FR-028).
    pub fn reset(&mut self, group: SettingsGroup) {
        let defaults = Settings::default();
        match group {
            SettingsGroup::Instrument => self.instrument = defaults.instrument,
            SettingsGroup::Display => self.display = defaults.display,
            SettingsGroup::Sound => self.sound = defaults.sound,
            SettingsGroup::Tuner => self.tuner = defaults.tuner,
        }
    }
}

/// The result of reading the settings file.
#[derive(Debug, Clone, PartialEq)]
pub struct Loaded {
    /// The settings to use.
    pub settings: Settings,
    /// A plain message when the file was unreadable and defaults were used (shown once).
    pub notice: Option<String>,
}

fn settings_error(what: &str, e: &dyn std::fmt::Display) -> AppError {
    AppError::of(
        AppErrorKind::Settings,
        format!("Could not {what} your settings"),
    )
    .with_details(e.to_string())
}

/// Reads `dir/settings.json`. A missing file gives the defaults; an unparsable one is renamed
/// to `settings.json.bak-<timestamp>` and the defaults are used with a notice. Unknown fields
/// are ignored.
#[must_use]
pub fn load(dir: &Path, timestamp: u64) -> Loaded {
    let path = dir.join(SETTINGS_FILE);
    let text = match std::fs::read_to_string(&path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Loaded {
                settings: Settings::default(),
                notice: None,
            };
        }
        Err(e) => {
            tracing::warn!(error = %e, "settings file unreadable");
            return Loaded {
                settings: Settings::default(),
                notice: Some("Your settings could not be read, so defaults are in use".into()),
            };
        }
    };
    match serde_json::from_str::<Settings>(&text) {
        Ok(mut settings) => {
            settings.version = SETTINGS_VERSION;
            settings.sound.volume = settings.sound.volume.min(100);
            Loaded {
                settings,
                notice: None,
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "settings file corrupt; keeping a backup");
            let backup: PathBuf = dir.join(format!("{SETTINGS_FILE}.bak-{timestamp}"));
            if let Err(rename_error) = std::fs::rename(&path, &backup) {
                tracing::warn!(error = %rename_error, "could not back up the settings file");
            }
            Loaded {
                settings: Settings::default(),
                notice: Some(
                    "Your settings file was damaged, so defaults are in use. A copy was kept."
                        .into(),
                ),
            }
        }
    }
}

/// Writes the settings atomically (temp file, then rename).
///
/// # Errors
/// [`AppErrorKind::Settings`] when the folder or file cannot be written.
pub fn save(dir: &Path, settings: &Settings) -> Result<(), AppError> {
    std::fs::create_dir_all(dir).map_err(|e| settings_error("save", &e))?;
    let json = serde_json::to_string_pretty(settings).map_err(|e| settings_error("save", &e))?;
    let temp = dir.join(format!("{SETTINGS_FILE}.tmp"));
    std::fs::write(&temp, json).map_err(|e| settings_error("save", &e))?;
    std::fs::rename(&temp, dir.join(SETTINGS_FILE)).map_err(|e| settings_error("save", &e))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn changed() -> Settings {
        let mut s = Settings::default();
        s.instrument.handedness.left_handed = true;
        s.instrument.frets = FretCount::new(24).unwrap();
        s.display.theme = Theme::Dark;
        s.display.show_fret_numbers = false;
        s.sound.volume = 30;
        s.tuner.reference_a4 = ReferencePitch::new(442.0).unwrap();
        s.tuner.mic_explained = true;
        s
    }

    #[test]
    fn defaults_follow_g4() {
        let s = Settings::default();
        assert_eq!(s.instrument.frets.value(), 22);
        assert!(!s.instrument.handedness.left_handed);
        assert_eq!(s.instrument.default_tuning, TuningChoice::default());
        assert_eq!(s.display.labels_default, LabelsMode::NoteName);
        assert!(s.display.show_fret_numbers);
        assert_eq!(s.sound.volume, 80);
        assert!((s.tuner.reference_a4.hz() - 440.0).abs() < f64::EPSILON);
        assert!(!s.tuner.mic_explained);
    }

    /// AC-FR-028: save, reload, then reset one group only.
    #[test]
    fn ac_fr_028_save_reload_and_reset_one_group() {
        let dir = tempfile::tempdir().unwrap();
        save(dir.path(), &changed()).unwrap();
        let loaded = load(dir.path(), 1);
        assert_eq!(loaded.notice, None);
        assert_eq!(loaded.settings, changed());

        let mut s = loaded.settings;
        s.reset(SettingsGroup::Display);
        assert_eq!(s.display, DisplaySettings::default());
        assert!(s.instrument.handedness.left_handed, "Instrument untouched");
        assert_eq!(s.instrument.frets.value(), 24);
        assert_eq!(s.sound.volume, 30);
        for group in [
            SettingsGroup::Instrument,
            SettingsGroup::Sound,
            SettingsGroup::Tuner,
        ] {
            s.reset(group);
        }
        assert_eq!(s, Settings::default());
    }

    #[test]
    fn missing_file_gives_defaults_without_a_notice() {
        let dir = tempfile::tempdir().unwrap();
        let loaded = load(dir.path(), 1);
        assert_eq!(loaded.settings, Settings::default());
        assert_eq!(loaded.notice, None);
    }

    #[test]
    fn corrupt_file_is_kept_as_a_backup() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join(SETTINGS_FILE), "{ not json").unwrap();
        let loaded = load(dir.path(), 42);
        assert_eq!(loaded.settings, Settings::default());
        assert!(loaded.notice.unwrap().contains("damaged"));
        assert!(dir.path().join("settings.json.bak-42").exists());
        assert!(!dir.path().join(SETTINGS_FILE).exists());
    }

    #[test]
    fn out_of_range_values_are_rejected_or_clamped() {
        let dir = tempfile::tempdir().unwrap();
        // Frets 30 fails validation: the whole file counts as corrupt.
        std::fs::write(
            dir.path().join(SETTINGS_FILE),
            r#"{"instrument":{"frets":30}}"#,
        )
        .unwrap();
        assert_eq!(load(dir.path(), 1).settings, Settings::default());
        // Volume above 100 is clamped; unknown fields are ignored.
        std::fs::write(
            dir.path().join(SETTINGS_FILE),
            r#"{"sound":{"volume":250},"future":true}"#,
        )
        .unwrap();
        assert_eq!(load(dir.path(), 1).settings.sound.volume, 100);
    }

    #[test]
    fn apply_replaces_only_patched_groups_and_clamps_volume() {
        let mut s = Settings::default();
        s.apply(SettingsPatch {
            sound: Some(SoundSettings {
                volume: 200,
                mute: true,
            }),
            ..SettingsPatch::default()
        });
        assert_eq!(s.sound.volume, 100);
        assert!(s.sound.mute);
        assert_eq!(s.display, DisplaySettings::default());
        let theme = DisplaySettings {
            theme: Theme::Light,
            ..DisplaySettings::default()
        };
        s.apply(SettingsPatch {
            display: Some(theme.clone()),
            instrument: Some(InstrumentSettings::default()),
            tuner: Some(TunerSettings::default()),
            ..SettingsPatch::default()
        });
        assert_eq!(s.display, theme);
    }

    #[test]
    fn save_reports_an_unwritable_folder() {
        let dir = tempfile::tempdir().unwrap();
        let blocker = dir.path().join("file");
        std::fs::write(&blocker, "x").unwrap();
        let err = save(&blocker.join("sub"), &Settings::default()).unwrap_err();
        assert_eq!(err.kind, AppErrorKind::Settings);
        assert_eq!(err.message, "Could not save your settings");
    }

    #[test]
    fn unreadable_path_gives_defaults_with_a_notice() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::create_dir(dir.path().join(SETTINGS_FILE)).unwrap();
        let loaded = load(dir.path(), 1);
        assert!(loaded.notice.is_some());
    }
}
