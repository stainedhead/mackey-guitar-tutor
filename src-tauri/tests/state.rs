//! Command-path tests: `AppState` driven the way the Tauri commands drive it, with a fake
//! audio port and a temp settings folder. No window system and no sound card.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    reason = "test helpers may panic on bad fixtures; clippy.toml only relaxes the lints inside #[test] functions"
)]

use mackey_app::explorer::{ChordFieldStatus, CycleAction, ExplorerOption, OverlayRequest};
use mackey_app::ports::{Call, CallLog, FakeAudio, SharedSink};
use mackey_app::settings::{
    DisplaySettings, InstrumentSettings, SettingsGroup, SettingsPatch, Theme, TuningChoice,
};
use mackey_app::state::{AppState, Emitter};
use mackey_app::{AppErrorKind, ExplorerSnapshot};
use mackey_audio::dsp::{TunerMode, TunerNotice, TunerReading, TunerStatus};
use mackey_theory::{Degree, Mode, Note};
use mackey_voicing::{Focus, LabelsMode, TuningPreset, ViewType, VoicingResult};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

struct Rig {
    state: AppState,
    calls: CallLog,
    sink: SharedSink,
    dir: tempfile::TempDir,
}

fn rig_with(fake: FakeAudio) -> Rig {
    let dir = tempfile::tempdir().unwrap();
    let calls = Arc::clone(&fake.calls);
    let sink = Arc::clone(&fake.sink);
    Rig {
        state: AppState::new(dir.path().to_path_buf(), Box::new(fake)),
        calls,
        sink,
        dir,
    }
}

fn rig() -> Rig {
    rig_with(FakeAudio::default())
}

fn note(text: &str) -> Note {
    text.parse().unwrap()
}

fn degree(n: u8) -> Degree {
    Degree::new(n).unwrap()
}

fn chord_of(s: &ExplorerSnapshot) -> Option<String> {
    s.panel.selection.chord.map(|c| c.symbol())
}

#[test]
fn start_up_snapshot_is_g_major_with_six_strings_and_22_frets() {
    // AC-P0-5 through the command path.
    let r = rig();
    let s = r.state.get_snapshot(None).unwrap();
    assert_eq!(s.view.key_label, "G major");
    assert_eq!(s.view.strings.len(), 6);
    assert_eq!(s.view.fret_count, 22);
    assert_eq!(s.panel.key.label, "G major");
    assert_eq!(s.panel.diatonic.len(), 7);
    assert_eq!(s.panel.chord_status, ChordFieldStatus::Empty);
    assert_eq!(r.state.fretboard_view(None).unwrap().fret_count, 22);
}

/// AC-FB-2.2: G major, IV is C; changing the key to D keeps the degree and selects G.
#[test]
fn ac_fb_2_2_key_change_keeps_the_degree() {
    let r = rig();
    let s = r.state.select_diatonic_chord(degree(4), None).unwrap();
    assert_eq!(chord_of(&s).as_deref(), Some("C"));
    assert!(s.panel.diatonic[3].selected);
    let s = r.state.select_key(note("D"), Mode::Major, None).unwrap();
    assert_eq!(chord_of(&s).as_deref(), Some("G"));
    assert_eq!(s.panel.key.label, "D major");
}

#[test]
fn roman_numeral_vi_in_g_selects_em() {
    let r = rig();
    let s = r.state.select_chord("vi", None).unwrap();
    assert_eq!(chord_of(&s).as_deref(), Some("Em"));
    assert_eq!(
        s.panel.chord_status,
        ChordFieldStatus::Valid {
            symbol: "Em".into()
        }
    );
}

/// FB-4.4, E-FB-1: bad text is not an error; the selection stays and the field explains.
#[test]
fn hm7_returns_the_message_and_keeps_the_selection() {
    let r = rig();
    let before = r.state.select_chord("Am", None).unwrap();
    let after = r.state.select_chord("Hm7", None).unwrap();
    assert_eq!(after.panel.selection, before.panel.selection);
    assert_eq!(after.panel.chord_text, "Hm7");
    match &after.panel.chord_status {
        ChordFieldStatus::Error {
            message,
            suggestion,
        } => {
            assert_eq!(message, "Not a chord symbol I know: Hm7. Did you mean Bm7?");
            assert_eq!(suggestion.as_deref(), Some("Bm7"));
        }
        other => panic!("expected an error status, got {other:?}"),
    }
    assert!(after.revision > before.revision);
    // The next good action clears the error.
    let ok = r.state.select_chord("Bm7", None).unwrap();
    assert_eq!(ok.panel.chord_text, "Bm7");
}

#[test]
fn parse_chord_validates_without_selecting() {
    let r = rig();
    let bad = r.state.parse_chord("Hm7").unwrap();
    assert!(bad.chord.is_none());
    assert_eq!(bad.diagnostics[0].suggestion.as_deref(), Some("Bm7"));
    assert_eq!(
        r.state.parse_chord("vi").unwrap().chord.unwrap().symbol(),
        "Em"
    );
    assert_eq!(
        r.state.get_snapshot(None).unwrap().panel.selection.focus,
        Focus::None
    );
}

#[test]
fn invalid_arguments_return_invalid_and_change_nothing() {
    let r = rig();
    let start = r.state.get_snapshot(None).unwrap();
    let cases: Vec<(&str, mackey_app::AppError)> = vec![
        ("string 0", r.state.select_note(0, 1, None).unwrap_err()),
        ("string 7", r.state.select_note(7, 1, None).unwrap_err()),
        ("fret 23", r.state.select_note(3, 23, None).unwrap_err()),
        ("capo 13", r.state.set_capo(13, None).unwrap_err()),
        (
            "bad custom tuning",
            r.state
                .select_tuning(TuningChoice::Custom(vec!["E2".into()]), None)
                .unwrap_err(),
        ),
        (
            "bad pitch",
            r.state
                .select_tuning(TuningChoice::Custom(vec!["x".into(); 6]), None)
                .unwrap_err(),
        ),
        ("play_note", r.state.play_note(7, 0).unwrap_err()),
        (
            "unknown chord",
            r.state.voicings_for("Hm7", None).unwrap_err(),
        ),
    ];
    for (label, err) in cases {
        assert_eq!(err.kind, AppErrorKind::Invalid, "{label}");
    }
    assert_eq!(r.state.get_snapshot(None).unwrap(), start);
}

#[test]
fn every_change_increments_the_revision() {
    let r = rig();
    let mut last = r.state.get_snapshot(None).unwrap().revision;
    for action in [
        CycleAction::ChordNext,
        CycleAction::VoicingNext,
        CycleAction::CycleView,
        CycleAction::KeyNext,
    ] {
        let s = r.state.cycle(action, None).unwrap();
        assert!(s.revision > last, "{action:?}");
        last = s.revision;
    }
}

#[test]
fn concurrent_commands_serialise() {
    let r = Arc::new(rig());
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let r = Arc::clone(&r);
            std::thread::spawn(move || {
                for _ in 0..25 {
                    r.state.cycle(CycleAction::ChordNext, None).unwrap();
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    assert_eq!(r.state.get_snapshot(None).unwrap().revision, 100);
}

/// AC-E-FB-4: a capo moves what is shown above the capo.
#[test]
fn ac_e_fb_4_capo_keeps_everything_above_the_capo() {
    let r = rig();
    r.state.select_chord("Am", None).unwrap();
    r.state.set_view_type(ViewType::Arpeggio, None).unwrap();
    let s = r.state.set_capo(5, None).unwrap();
    assert_eq!(s.view.capo, 5);
    assert!(s.view.positions.iter().all(|p| p.fret.value() >= 5));
    assert_eq!(s.panel.instrument.capo, 5);
}

#[test]
fn tuning_change_reports_a_notice_and_custom_tunings_warn() {
    let r = rig();
    r.state.select_chord("D", None).unwrap();
    r.state.set_view_type(ViewType::Voicing, None).unwrap();
    let s = r
        .state
        .select_tuning(TuningChoice::Preset(TuningPreset::DropD), None)
        .unwrap();
    assert_eq!(s.panel.instrument.tuning_name, "Drop D");
    assert!(s.panel.instrument.tuning_choices.iter().any(|t| t.selected));
    let custom: Vec<String> = ["F#3", "A2", "D3", "G3", "B3", "E4"]
        .map(String::from)
        .to_vec();
    let s = r
        .state
        .select_tuning(TuningChoice::Custom(custom), None)
        .unwrap();
    assert_eq!(s.panel.instrument.warnings.len(), 1);
    assert!(s.panel.instrument.warnings[0].contains("Unusual tension"));
    assert_eq!(
        s.panel.instrument.tuning_choices.last().unwrap().label,
        "Custom"
    );
}

#[test]
fn options_labels_overlay_and_notes_flow_through() {
    let r = rig();
    let s = r.state.select_note(3, 2, None).unwrap();
    assert_eq!(s.panel.selection.focus, Focus::Note);
    let s = r
        .state
        .set_option(ExplorerOption::ShowSevenths(true), None)
        .unwrap();
    assert!(s.panel.diatonic[0].symbol.contains('7'));
    let s = r
        .state
        .set_labels_mode(LabelsMode::ScaleDegree, None)
        .unwrap();
    assert_eq!(s.view.labels_mode, LabelsMode::ScaleDegree);
    let on = OverlayRequest {
        on: true,
        scale: None,
        follow: false,
    };
    let s = r.state.set_scale_overlay(on, None).unwrap();
    assert!(s.panel.selection.overlay.on);
    let off = OverlayRequest { on: false, ..on };
    let s = r.state.set_scale_overlay(off, None).unwrap();
    assert!(!s.panel.selection.overlay.on);
    assert!(
        s.panel
            .scale_choices
            .iter()
            .any(|c| c.label.contains("pentatonic"))
    );
}

#[test]
fn voicings_for_f_lists_the_small_and_full_barre() {
    let r = rig();
    let VoicingResult::Found(list) = r.state.voicings_for("F", None).unwrap() else {
        panic!("F has voicings");
    };
    let capo = r.state.get_snapshot(None).unwrap().panel.instrument.capo;
    let frets: Vec<String> = list
        .iter()
        .map(|v| v.fret_string(mackey_voicing::Capo::new(capo).unwrap()))
        .collect();
    assert!(frets.contains(&"xx3211".to_owned()), "{frets:?}");
    assert!(frets.contains(&"133211".to_owned()), "{frets:?}");
}

#[test]
fn fit_selection_covers_the_highlighted_notes() {
    let r = rig();
    r.state.select_chord("C", None).unwrap();
    let range = r.state.fit_selection().unwrap();
    assert!(range.from <= range.to);
}

/// AC-E-FB-8: no audio device. Play says so; everything else keeps working.
#[test]
fn ac_e_fb_8_no_output_device() {
    let r = rig_with(FakeAudio::no_device());
    let err = r.state.play_selection(None).unwrap_err();
    assert_eq!(err.kind, AppErrorKind::NoAudioOutput);
    assert_eq!(err.message, "No audio output");
    assert_eq!(
        r.state.play_note(3, 0).unwrap_err().kind,
        AppErrorKind::NoAudioOutput
    );
    let s = r.state.cycle(CycleAction::ChordNext, None).unwrap();
    assert!(s.panel.selection.chord.is_some());
    assert_eq!(r.state.list_input_devices().unwrap().len(), 0);
    assert_eq!(
        r.state.tuner_reference_tone(3, true).unwrap_err().kind,
        AppErrorKind::NoAudioOutput
    );
    r.state.tuner_reference_tone(3, false).unwrap();
}

/// D5: a C voicing strums the five sounding strings 30 ms apart.
#[test]
fn play_selection_strums_the_voicing_low_to_high() {
    let r = rig();
    r.state.select_chord("C", None).unwrap();
    r.state.set_view_type(ViewType::Voicing, None).unwrap();
    r.state.play_selection(None).unwrap();
    let calls = r.calls.lock().unwrap();
    let Some(Call::Play(notes)) = calls.iter().rev().find(|c| matches!(c, Call::Play(_))) else {
        panic!("play was not called");
    };
    assert_eq!(notes.len(), 5, "x32010 sounds five strings");
    let delays: Vec<u32> = notes.iter().map(|n| n.delay_ms).collect();
    assert_eq!(delays, [0, 30, 60, 90, 120]);
    assert!(
        notes
            .windows(2)
            .all(|w| w[0].frequency_hz < w[1].frequency_hz)
    );
}

#[test]
fn play_note_uses_the_instrument_pitch() {
    let r = rig();
    r.state.play_note(3, 0).unwrap();
    let calls = r.calls.lock().unwrap();
    let Some(Call::Play(notes)) = calls.last() else {
        panic!("play was not called");
    };
    assert!((notes[0].frequency_hz - 196.0).abs() < 0.01, "G3 is 196 Hz");
    drop(calls);
    r.state.stop_playback().unwrap();
    assert_eq!(r.calls.lock().unwrap().last(), Some(&Call::Stop));
}

fn recorder() -> (Emitter, Arc<AtomicUsize>) {
    let count = Arc::new(AtomicUsize::new(0));
    let c = Arc::clone(&count);
    (
        Arc::new(move |_: &TunerReading| {
            c.fetch_add(1, Ordering::SeqCst);
        }),
        count,
    )
}

fn pitch_reading(t_ms: f64) -> TunerReading {
    let mut r = TunerReading::notice_only(TunerNotice::DeviceChanged, t_ms);
    r.notice = None;
    r.status = TunerStatus::Pitch;
    r
}

/// AC-TU-5.1 and TU-7: targets follow the capo, and opening the tuner stops playback first.
#[test]
fn tuner_start_builds_targets_with_the_capo_and_stops_playback_first() {
    let r = rig();
    r.state.set_capo(2, None).unwrap();
    let (emit, count) = recorder();
    let info = r.state.tuner_start(None, emit).unwrap();
    let names: Vec<String> = info.targets.iter().map(|t| t.pitch.name()).collect();
    assert_eq!(names, ["F#2", "B2", "E3", "A3", "C#4", "F#4"]);
    assert_eq!(info.permission, mackey_audio::io::MicPermission::Granted);
    {
        let calls = r.calls.lock().unwrap();
        let stop = calls.iter().position(|c| *c == Call::Stop).unwrap();
        let start = calls
            .iter()
            .position(|c| matches!(c, Call::TunerStart(..)))
            .unwrap();
        assert!(stop < start, "playback stops before the tuner starts");
    }
    // 10 s of readings delivered in a burst (an unpaced source): the wall-clock throttle lets
    // through the first and at most a handful more (FR-002); the stream-time rate is tested in
    // `tuner.rs`.
    {
        let sink = r.sink.lock().unwrap();
        let sink = sink.as_ref().unwrap();
        for i in 0..940 {
            sink(pitch_reading(f64::from(i) * 1000.0 / 94.0));
        }
    }
    let emitted = count.load(Ordering::SeqCst);
    assert!(
        (1..=35).contains(&emitted),
        "{emitted} emitted for a burst of 10 s of input"
    );
}

#[test]
fn tuner_mode_lock_and_stop() {
    let r = rig();
    r.state.tuner_stop().unwrap(); // not running: a no-op
    r.state
        .tuner_set_mode(TunerMode::Strings { locked: Some(3) })
        .unwrap();
    {
        let calls = r.calls.lock().unwrap();
        assert!(calls.contains(&Call::TunerLock(Some(3))));
        assert!(
            calls
                .iter()
                .any(|c| matches!(c, Call::TunerConfig(cfg) if cfg.targets.len() == 6))
        );
    }
    r.state.tuner_reference_tone(1, true).unwrap();
    let hz = r
        .calls
        .lock()
        .unwrap()
        .iter()
        .find_map(|c| match c {
            Call::Reference(Some(hz)) => Some(*hz),
            _ => None,
        })
        .unwrap();
    assert!((hz - 329.63).abs() < 0.05, "string 1 is E4");
    assert_eq!(
        r.state.tuner_reference_tone(9, true).unwrap_err().kind,
        AppErrorKind::Invalid
    );
    r.state.tuner_reference_tone(1, false).unwrap();
}

#[test]
fn tuner_errors_are_plain_and_typed() {
    let none = rig_with(FakeAudio::no_device());
    let (emit, _) = recorder();
    let err = none.state.tuner_start(None, emit).unwrap_err();
    assert_eq!(err.kind, AppErrorKind::NoInputDevice);

    let denied = rig_with(FakeAudio {
        permission: Some(mackey_audio::io::MicPermission::Denied),
        ..FakeAudio::default()
    });
    let (emit, _) = recorder();
    let err = denied.state.tuner_start(None, emit).unwrap_err();
    assert_eq!(err.kind, AppErrorKind::MicrophoneDenied);
    assert_eq!(err.message, "Microphone access is off");
    assert_eq!(
        denied
            .state
            .open_system_settings(mackey_app::state::SettingsPane::Microphone)
            .is_err(),
        cfg!(not(target_os = "macos"))
    );
}

/// AC-FR-028 through the commands: apply, persist, reload, reset one group.
#[test]
fn ac_fr_028_settings_persist_and_reset_by_group() {
    let r = rig();
    let mut instrument = InstrumentSettings::default();
    instrument.handedness.left_handed = true;
    let display = DisplaySettings {
        theme: Theme::Dark,
        ..DisplaySettings::default()
    };
    let saved = r
        .state
        .set_settings(SettingsPatch {
            instrument: Some(instrument),
            display: Some(display),
            ..SettingsPatch::default()
        })
        .unwrap();
    assert!(saved.instrument.handedness.left_handed);
    // The explorer picked up handedness at once (LH-1).
    assert!(r.state.get_snapshot(None).unwrap().view.mirrored);

    // A new app on the same folder reads the saved file.
    let again = AppState::new(r.dir.path().to_path_buf(), Box::new(FakeAudio::default()));
    assert_eq!(again.get_settings().unwrap().display.theme, Theme::Dark);
    assert!(again.get_snapshot(None).unwrap().view.mirrored);

    let reset = r.state.reset_settings(SettingsGroup::Display).unwrap();
    assert_eq!(reset.display, DisplaySettings::default());
    assert!(
        reset.instrument.handedness.left_handed,
        "Instrument stays changed"
    );
    assert!(
        r.state
            .get_settings()
            .unwrap()
            .instrument
            .handedness
            .left_handed
    );
}

#[test]
fn settings_feed_the_explorer_and_the_audio_volume() {
    let r = rig();
    let instrument = InstrumentSettings {
        frets: mackey_voicing::FretCount::new(24).unwrap(),
        default_tuning: TuningChoice::Preset(TuningPreset::DropD),
        allow_stretches: true,
        ..InstrumentSettings::default()
    };
    let display = DisplaySettings {
        labels_default: LabelsMode::ScaleDegree,
        ..DisplaySettings::default()
    };
    let sound = mackey_app::settings::SoundSettings {
        volume: 50,
        mute: true,
    };
    r.state
        .set_settings(SettingsPatch {
            instrument: Some(instrument),
            display: Some(display),
            sound: Some(sound),
            tuner: None,
        })
        .unwrap();
    let s = r.state.get_snapshot(None).unwrap();
    assert_eq!(s.view.fret_count, 24);
    assert_eq!(s.panel.instrument.tuning_name, "Drop D");
    assert_eq!(s.view.labels_mode, LabelsMode::ScaleDegree);
    assert!(s.panel.selection.options.allow_stretch);
    assert!(r.calls.lock().unwrap().contains(&Call::Volume(0.5, true)));
}

#[test]
fn a_damaged_settings_file_gives_defaults_and_one_notice() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("settings.json"), "][").unwrap();
    let state = AppState::new(dir.path().to_path_buf(), Box::new(FakeAudio::default()));
    assert!(
        state
            .take_settings_notice()
            .unwrap()
            .unwrap()
            .contains("damaged")
    );
    assert_eq!(state.take_settings_notice().unwrap(), None, "shown once");
    assert_eq!(
        state.get_settings().unwrap(),
        mackey_app::Settings::default()
    );
}

/// AC-NF-24 at the state level: a poisoned mutex is recovered, not fatal.
#[test]
fn a_poisoned_mutex_is_recovered() {
    let m = Arc::new(Mutex::new(5));
    let m2 = Arc::clone(&m);
    let _ = std::thread::spawn(move || {
        let _g = m2.lock().unwrap();
        panic!("poison it");
    })
    .join();
    assert!(m.is_poisoned());
    assert_eq!(*mackey_app::state::lock(&m), 5);
}

#[test]
fn closing_the_window_path_stops_the_tuner() {
    let r = rig();
    let (emit, _) = recorder();
    r.state.tuner_start(None, emit).unwrap();
    assert!(r.sink.lock().unwrap().is_some());
    r.state.tuner_stop().unwrap();
    assert!(r.sink.lock().unwrap().is_none());
}

#[test]
fn tuner_targets_work_without_a_microphone() {
    // E-TU-2: the reference tones need targets even when the microphone is off.
    let r = rig_with(FakeAudio::no_device());
    r.state.set_capo(2, None).unwrap();
    let names: Vec<String> = r
        .state
        .tuner_targets()
        .unwrap()
        .iter()
        .map(|t| t.pitch.name())
        .collect();
    assert_eq!(names, ["F#2", "B2", "E3", "A3", "C#4", "F#4"]);
}
