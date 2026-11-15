//! Every command name, once. The x-macro feeds the Tauri handler, `COMMAND_NAMES` and (through
//! the manifest test) the generated UI wrappers, so a command cannot be added in one place only.

/// Calls `$m!` with the comma-separated list of command names.
macro_rules! for_each_command {
    ($m:ident) => {
        $m! {
            app_info,
            get_snapshot,
            fretboard_view,
            select_key,
            select_diatonic_chord,
            select_chord,
            parse_chord,
            select_note,
            select_tuning,
            set_capo,
            set_view_type,
            set_labels_mode,
            set_scale_overlay,
            set_option,
            cycle,
            voicings_for,
            fit_selection,
            play_selection,
            play_note,
            stop_playback,
            list_input_devices,
            tuner_targets,
            tuner_start,
            tuner_stop,
            tuner_set_mode,
            tuner_reference_tone,
            open_system_settings,
            get_settings,
            set_settings,
            reset_settings,
            take_settings_notice
        }
    };
}

macro_rules! names_of {
    ($($name:ident),* $(,)?) => { &[$(stringify!($name)),*] };
}

macro_rules! handler_of {
    ($($name:ident),* $(,)?) => { tauri::generate_handler![$($crate::commands::$name),*] };
}

/// Names of every registered command, in registration order.
pub const COMMAND_NAMES: &[&str] = for_each_command!(names_of);

pub(crate) use for_each_command;
pub(crate) use handler_of;
