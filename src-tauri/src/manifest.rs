//! The command manifest: for every command, its UI-facing argument names and TypeScript
//! types. The bindings generator (`make bindings`) turns this table into
//! `ui/src/bindings/commands.ts`; a test checks it lists exactly the registered commands.
//!
//! `ts-rs` exports types but not command signatures, and `tauri-specta` for Tauri 2 is still
//! a release candidate (D-05), so the signatures are written once here.

/// One argument of a command.
#[derive(Debug, Clone, Copy)]
pub struct ArgSpec {
    /// The argument name as the UI sends it (camelCase of the Rust parameter).
    pub name: &'static str,
    /// Its TypeScript type.
    pub ts: &'static str,
    /// True for an `Option<T>` parameter the UI may leave out.
    pub optional: bool,
}

/// One command.
#[derive(Debug, Clone, Copy)]
pub struct CommandSpec {
    /// The Rust command name (`snake_case`), as `invoke` names it.
    pub name: &'static str,
    /// Its arguments.
    pub args: &'static [ArgSpec],
    /// The TypeScript type of the resolved value.
    pub returns: &'static str,
}

const fn req(name: &'static str, ts: &'static str) -> ArgSpec {
    ArgSpec {
        name,
        ts,
        optional: false,
    }
}

const fn opt(name: &'static str, ts: &'static str) -> ArgSpec {
    ArgSpec {
        name,
        ts,
        optional: true,
    }
}

const VISIBLE: ArgSpec = opt("visible", "FretRange");

/// Every command, in registration order.
pub const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        name: "app_info",
        args: &[],
        returns: "AppInfo",
    },
    CommandSpec {
        name: "get_snapshot",
        args: &[VISIBLE],
        returns: "ExplorerSnapshot",
    },
    CommandSpec {
        name: "fretboard_view",
        args: &[VISIBLE],
        returns: "FretboardView",
    },
    CommandSpec {
        name: "select_key",
        args: &[req("tonic", "Note"), req("mode", "Mode"), VISIBLE],
        returns: "ExplorerSnapshot",
    },
    CommandSpec {
        name: "select_diatonic_chord",
        args: &[req("degree", "Degree"), VISIBLE],
        returns: "ExplorerSnapshot",
    },
    CommandSpec {
        name: "select_chord",
        args: &[req("text", "string"), VISIBLE],
        returns: "ExplorerSnapshot",
    },
    CommandSpec {
        name: "parse_chord",
        args: &[req("text", "string")],
        returns: "ParseOutcome",
    },
    CommandSpec {
        name: "select_note",
        args: &[req("string", "number"), req("fret", "number"), VISIBLE],
        returns: "ExplorerSnapshot",
    },
    CommandSpec {
        name: "select_tuning",
        args: &[req("tuning", "TuningChoice"), VISIBLE],
        returns: "ExplorerSnapshot",
    },
    CommandSpec {
        name: "set_capo",
        args: &[req("capo", "number"), VISIBLE],
        returns: "ExplorerSnapshot",
    },
    CommandSpec {
        name: "set_view_type",
        args: &[req("view", "ViewType"), VISIBLE],
        returns: "ExplorerSnapshot",
    },
    CommandSpec {
        name: "set_labels_mode",
        args: &[req("labels", "LabelsMode"), VISIBLE],
        returns: "ExplorerSnapshot",
    },
    CommandSpec {
        name: "set_scale_overlay",
        args: &[req("overlay", "OverlayRequest"), VISIBLE],
        returns: "ExplorerSnapshot",
    },
    CommandSpec {
        name: "set_option",
        args: &[req("option", "ExplorerOption"), VISIBLE],
        returns: "ExplorerSnapshot",
    },
    CommandSpec {
        name: "cycle",
        args: &[req("action", "CycleAction"), VISIBLE],
        returns: "ExplorerSnapshot",
    },
    CommandSpec {
        name: "voicings_for",
        args: &[req("text", "string"), opt("options", "VoicingOptions")],
        returns: "VoicingResult",
    },
    CommandSpec {
        name: "fit_selection",
        args: &[],
        returns: "FretRange",
    },
    CommandSpec {
        name: "play_selection",
        args: &[VISIBLE],
        returns: "void",
    },
    CommandSpec {
        name: "play_note",
        args: &[req("string", "number"), req("fret", "number")],
        returns: "void",
    },
    CommandSpec {
        name: "stop_playback",
        args: &[],
        returns: "void",
    },
    CommandSpec {
        name: "list_input_devices",
        args: &[],
        returns: "InputDevice[]",
    },
    CommandSpec {
        name: "tuner_targets",
        args: &[],
        returns: "TunerTarget[]",
    },
    CommandSpec {
        name: "tuner_start",
        args: &[opt("device", "string")],
        returns: "TunerStartInfo",
    },
    CommandSpec {
        name: "tuner_stop",
        args: &[],
        returns: "void",
    },
    CommandSpec {
        name: "tuner_set_mode",
        args: &[req("mode", "TunerMode")],
        returns: "void",
    },
    CommandSpec {
        name: "tuner_reference_tone",
        args: &[req("string", "number"), req("on", "boolean")],
        returns: "void",
    },
    CommandSpec {
        name: "open_system_settings",
        args: &[req("pane", "SettingsPane")],
        returns: "void",
    },
    CommandSpec {
        name: "get_settings",
        args: &[],
        returns: "Settings",
    },
    CommandSpec {
        name: "set_settings",
        args: &[req("patch", "SettingsPatch")],
        returns: "Settings",
    },
    CommandSpec {
        name: "reset_settings",
        args: &[req("group", "SettingsGroup")],
        returns: "Settings",
    },
    CommandSpec {
        name: "take_settings_notice",
        args: &[],
        returns: "string | null",
    },
];
