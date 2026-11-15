//! The explorer: the Rust-owned state behind the Explore screen (selection, instrument,
//! handedness) and the assembly of the snapshot the UI renders.
//!
//! No music logic lives here. Every decision is a call into `mackey-voicing` (`apply`,
//! `fretboard_view`, `instrument_changed`, `playback_plan`) or `mackey-theory` (parsing, keys,
//! diatonic chords); this module holds state, validates arguments and builds panel data from
//! core values.

use crate::error::AppError;
use crate::ports::SoundNote;
use crate::settings::{Settings, TuningChoice};
use mackey_audio::dsp::{TunerTarget, targets_from_pitches};
use mackey_theory::{
    Degree, Key, Mode, Note, ParseOptions, ParseOutcome, Pitch, ReferencePitch, Scale, parse_chord,
    roman_to_chord,
};
use mackey_voicing::{
    Action, ArpeggioDirection, Bpm, Capo, CapoNames, Context, Focus, Fret, FretCount, FretRange,
    FretboardView, Handedness, Instrument, LabelsMode, Selection, StringNumber, Tuning,
    TuningPreset, TuningWarning, ViewRequest, ViewType, VoicingCache, apply_cached,
    fretboard_view_cached, instrument_changed_cached, playback_plan_cached,
};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

/// What the chord field shows under its text (FB-4.4, E-FB-1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "kind")]
#[ts(export)]
pub enum ChordFieldStatus {
    /// Nothing typed or selected.
    Empty,
    /// The text is the selected chord.
    Valid {
        /// The canonical symbol.
        symbol: String,
    },
    /// The text did not parse; the selection is unchanged.
    Error {
        /// "Not a chord symbol I know: Hm7. Did you mean Bm7?"
        message: String,
        /// A full replacement symbol, when one is close.
        suggestion: Option<String>,
    },
}

/// A tonic the key picker offers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TonicChoice {
    /// "F#", "Gb".
    pub label: String,
    /// The note to send back to `select_key`.
    pub note: Note,
    /// True for the tonic of the current key.
    pub selected: bool,
}

/// A mode the key picker offers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ModeChoice {
    /// "Major", "Dorian".
    pub label: String,
    /// The mode to send back.
    pub mode: Mode,
    /// True for the mode of the current key.
    pub selected: bool,
}

/// The key picker's data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct KeyPanel {
    /// "G major".
    pub label: String,
    /// Tonics for the current mode.
    pub tonic_choices: Vec<TonicChoice>,
    /// Modes usable as a key.
    pub modes: Vec<ModeChoice>,
    /// "F# major and Gb major sound the same" for 6-accidental keys.
    pub enharmonic_hint: Option<String>,
}

/// One chip of the diatonic chord strip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct DiatonicChip {
    /// The degree to send back to `select_diatonic_chord`.
    pub degree: Degree,
    /// "I", "ii", "viiø7".
    pub roman: String,
    /// "G", "Am7".
    pub symbol: String,
    /// True when this chord is the selected one.
    pub selected: bool,
}

/// A tuning the picker offers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct TuningItem {
    /// "Standard", "Drop D".
    pub label: String,
    /// "E2 A2 D3 G3 B3 E4".
    pub pitches: String,
    /// The choice to send back to `select_tuning`.
    pub choice: TuningChoice,
    /// True for the tuning in use.
    pub selected: bool,
}

/// Tuning, capo and layout data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct InstrumentPanel {
    /// "Standard".
    pub tuning_name: String,
    /// "E2 A2 D3 G3 B3 E4" for the tuning in use.
    pub tuning_pitches: String,
    /// The presets, plus the custom tuning when one is in use.
    pub tuning_choices: Vec<TuningItem>,
    /// "String 6: Unusual tension, check your strings".
    pub warnings: Vec<String>,
    /// The capo fret, 0 for none.
    pub capo: u8,
    /// Frets on the neck.
    pub frets: u8,
    /// Left-handed and flip.
    pub handedness: Handedness,
}

/// A scale the overlay picker offers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ScaleChoice {
    /// "G minor pentatonic".
    pub label: String,
    /// The scale to send back.
    pub scale: Scale,
    /// True for the scale in use.
    pub selected: bool,
}

/// Everything the panels show that is not the neck.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExplorerPanel {
    /// Key picker data.
    pub key: KeyPanel,
    /// The chord strip.
    pub diatonic: Vec<DiatonicChip>,
    /// What the chord field contains.
    pub chord_text: String,
    /// Validation state of the chord field.
    pub chord_status: ChordFieldStatus,
    /// Tuning and capo.
    pub instrument: InstrumentPanel,
    /// The selection, echoed so controls can show their state.
    pub selection: Selection,
    /// Scales for the overlay picker.
    pub scale_choices: Vec<ScaleChoice>,
}

/// The one value every explorer command returns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct ExplorerSnapshot {
    /// Increments on every state change; the UI drops a snapshot older than the one it has
    /// (out-of-order responses, SAC-18).
    pub revision: u32,
    /// The neck.
    pub view: FretboardView,
    /// The panels.
    pub panel: ExplorerPanel,
    /// Brief notices caused by the action that produced this snapshot.
    pub notices: Vec<String>,
    /// What a screen reader says for that action (AC-FB-9.1c).
    pub announcement: String,
}

/// The FB-9 cycling and navigation actions (no payload).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum CycleAction {
    /// Next chord in the key.
    ChordNext,
    /// Previous chord in the key.
    ChordPrev,
    /// Next voicing.
    VoicingNext,
    /// Previous voicing.
    VoicingPrev,
    /// Next inversion.
    InversionNext,
    /// Previous inversion.
    InversionPrev,
    /// Next string set.
    StringSetNext,
    /// Previous string set.
    StringSetPrev,
    /// Next position.
    PositionNext,
    /// Previous position.
    PositionPrev,
    /// Cycle the view type.
    CycleView,
    /// Cycle the labels mode.
    CycleLabels,
    /// Toggle the scale overlay.
    ToggleOverlay,
    /// Next key on the circle of fifths.
    KeyNext,
    /// Previous key on the circle of fifths.
    KeyPrev,
    /// Relative major or minor.
    RelativeToggle,
    /// Back to the key's notes.
    ClearFocus,
}

impl From<CycleAction> for Action {
    fn from(a: CycleAction) -> Self {
        match a {
            CycleAction::ChordNext => Action::ChordNext,
            CycleAction::ChordPrev => Action::ChordPrev,
            CycleAction::VoicingNext => Action::VoicingNext,
            CycleAction::VoicingPrev => Action::VoicingPrev,
            CycleAction::InversionNext => Action::InversionNext,
            CycleAction::InversionPrev => Action::InversionPrev,
            CycleAction::StringSetNext => Action::StringSetNext,
            CycleAction::StringSetPrev => Action::StringSetPrev,
            CycleAction::PositionNext => Action::PositionNext,
            CycleAction::PositionPrev => Action::PositionPrev,
            CycleAction::CycleView => Action::CycleView,
            CycleAction::CycleLabels => Action::CycleLabels,
            CycleAction::ToggleOverlay => Action::ToggleOverlay,
            CycleAction::KeyNext => Action::KeyNext,
            CycleAction::KeyPrev => Action::KeyPrev,
            CycleAction::RelativeToggle => Action::RelativeToggle,
            CycleAction::ClearFocus => Action::ClearFocus,
        }
    }
}

/// A single explorer setting with a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub enum ExplorerOption {
    /// Seventh chords on the strip.
    ShowSevenths(bool),
    /// Show all inversions on the string set.
    ShowAllOnSet(bool),
    /// Show play order numbers.
    ShowOrder(bool),
    /// Arpeggio direction.
    ArpeggioDirection(ArpeggioDirection),
    /// Allow a 5-fret span.
    AllowStretch(bool),
    /// Allow an omitted fifth.
    AllowOmittedFifth(bool),
    /// Which capo name leads.
    CapoNames(CapoNames),
    /// Follow the chords with the overlay scale.
    FollowChords(bool),
}

/// Overlay arguments of `set_scale_overlay`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct OverlayRequest {
    /// Show the overlay.
    pub on: bool,
    /// The scale; `None` is the key's own.
    pub scale: Option<Scale>,
    /// Follow the chords.
    pub follow: bool,
}

/// Parses `"F#3"` or `"D2"` into a pitch.
///
/// # Errors
/// [`AppErrorKind::Invalid`] naming the text.
pub fn parse_pitch(text: &str) -> Result<Pitch, AppError> {
    let bad = || AppError::new(format!("\"{text}\" is not a note like E2 or F#3"));
    let t = text.trim();
    let split = t
        .find(|c: char| c.is_ascii_digit() || c == '-')
        .ok_or_else(bad)?;
    let note: Note = t[..split].parse().map_err(|_| bad())?;
    let octave: i8 = t[split..].parse().map_err(|_| bad())?;
    Pitch::new(note, octave).map_err(|_| bad())
}

/// Builds a tuning from a choice.
///
/// # Errors
/// [`AppErrorKind::Invalid`] for a custom tuning that is not six valid pitches in range.
pub fn resolve_tuning(choice: &TuningChoice) -> Result<(Tuning, Vec<TuningWarning>), AppError> {
    match choice {
        TuningChoice::Preset(p) => Ok((Tuning::preset(*p), Vec::new())),
        TuningChoice::Custom(names) => {
            let pitches: Vec<Pitch> = names
                .iter()
                .map(|n| parse_pitch(n))
                .collect::<Result<_, _>>()?;
            let strings: [Pitch; 6] = pitches
                .try_into()
                .map_err(|_| AppError::new("A tuning needs six notes, one per string"))?;
            Ok(Tuning::custom(strings)?)
        }
    }
}

fn pitch_names(tuning: &Tuning) -> String {
    tuning
        .strings
        .iter()
        .map(|p| p.name())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The explorer state. One per window, behind the app's mutex.
#[derive(Debug)]
pub struct Explorer {
    selection: Selection,
    instrument: Instrument,
    handedness: Handedness,
    tuning_choice: TuningChoice,
    warnings: Vec<TuningWarning>,
    a4: ReferencePitch,
    cache: VoicingCache,
    revision: u32,
    chord_error: Option<(String, ChordFieldStatus)>,
    last_notices: Vec<String>,
    last_announcement: String,
}

impl Explorer {
    /// A new explorer with the G-4 defaults and the instrument settings applied.
    #[must_use]
    pub fn new(settings: &Settings) -> Self {
        let mut selection = Selection::default_g_major();
        selection.labels = settings.display.labels_default;
        selection.options.allow_stretch = settings.instrument.allow_stretches;
        let (tuning, warnings) = resolve_tuning(&settings.instrument.default_tuning)
            .unwrap_or_else(|_| (Tuning::preset(TuningPreset::Standard), Vec::new()));
        let instrument = Instrument {
            tuning,
            frets: settings.instrument.frets,
            capo: Capo::default(),
        };
        Self {
            selection,
            instrument,
            handedness: settings.instrument.handedness,
            tuning_choice: settings.instrument.default_tuning.clone(),
            warnings,
            a4: settings.tuner.reference_a4,
            cache: VoicingCache::new(),
            revision: 0,
            chord_error: None,
            last_notices: Vec::new(),
            last_announcement: String::new(),
        }
    }

    fn ctx_of<'a>(instrument: &'a Instrument, handedness: &'a Handedness) -> Context<'a> {
        Context {
            instrument,
            handedness,
        }
    }

    fn mutated(&mut self, notices: Vec<String>, announcement: String) {
        self.revision = self.revision.wrapping_add(1);
        self.chord_error = None;
        self.last_notices = notices;
        self.last_announcement = announcement;
    }

    /// Runs a core action and records its notices and announcement.
    pub fn act(&mut self, action: Action) {
        let ctx = Self::ctx_of(&self.instrument, &self.handedness);
        let applied = apply_cached(&self.selection, &ctx, action, &mut self.cache);
        self.selection = applied.selection;
        let notices = applied
            .notices
            .iter()
            .map(|n| n.text().to_owned())
            .collect();
        self.mutated(notices, applied.announcement);
    }

    /// Runs a cycling action.
    pub fn cycle(&mut self, action: CycleAction) {
        self.act(action.into());
    }

    /// Picks a key; a tonic that is not offered is normalised (G-1.4) with a notice.
    ///
    /// # Errors
    /// [`AppErrorKind::Invalid`] when no key exists for the tonic and mode.
    pub fn select_key(&mut self, tonic: Note, mode: Mode) -> Result<(), AppError> {
        let (key, notice) =
            Key::new_normalised(tonic, mode).map_err(|e| AppError::new(e.to_string()))?;
        self.act(Action::SelectKey(key));
        if let Some(n) = notice {
            self.last_notices.insert(0, n.message());
        }
        Ok(())
    }

    /// Picks the diatonic chord on a degree.
    pub fn select_diatonic(&mut self, degree: Degree) {
        self.act(Action::SelectDiatonic(degree));
    }

    /// Selects a chord typed as a symbol or a Roman numeral. Text that is neither leaves the
    /// selection unchanged and puts the error in the chord field (FB-4.4, E-FB-1).
    pub fn select_chord_text(&mut self, text: &str) {
        let trimmed = text.trim();
        let outcome = parse_chord(trimmed, ParseOptions::default());
        let chord = outcome
            .chord
            .or_else(|| roman_to_chord(trimmed, &self.selection.key).ok());
        if let Some(chord) = chord {
            self.act(Action::SelectChord(chord));
            return;
        }
        let first = outcome.diagnostics.into_iter().next();
        let status = ChordFieldStatus::Error {
            message: first.as_ref().map_or_else(
                || format!("Not a chord symbol I know: {trimmed}"),
                |d| d.message.clone(),
            ),
            suggestion: first.and_then(|d| d.suggestion),
        };
        self.revision = self.revision.wrapping_add(1);
        self.last_notices = Vec::new();
        self.last_announcement = String::new();
        self.chord_error = Some((text.to_owned(), status));
    }

    /// Stateless chord validation for the live field (FB-4.4).
    #[must_use]
    pub fn parse(&self, text: &str) -> ParseOutcome {
        let outcome = parse_chord(text.trim(), ParseOptions::default());
        if outcome.chord.is_some() {
            return outcome;
        }
        match roman_to_chord(text.trim(), &self.selection.key) {
            Ok(chord) => ParseOutcome {
                chord: Some(chord),
                diagnostics: Vec::new(),
            },
            Err(_) => outcome,
        }
    }

    /// Clicks a position.
    ///
    /// # Errors
    /// [`AppErrorKind::Invalid`] for string 0 or 7, or a fret above the fret count.
    pub fn select_note(&mut self, string: u8, fret: u8) -> Result<(), AppError> {
        let string = StringNumber::new(string)?;
        let fret = Fret::new(fret)?;
        if fret.value() > self.instrument.frets.value() {
            return Err(AppError::new(format!(
                "Fret {} is past the end of the neck",
                fret.value()
            )));
        }
        self.act(Action::SelectNote { string, fret });
        Ok(())
    }

    fn change_instrument(&mut self, new: Instrument) {
        let old_ctx = Self::ctx_of(&self.instrument, &self.handedness);
        let new_ctx = Self::ctx_of(&new, &self.handedness);
        let applied =
            instrument_changed_cached(&self.selection, &old_ctx, &new_ctx, &mut self.cache);
        self.selection = applied.selection;
        self.instrument = new;
        let notices = applied
            .notices
            .iter()
            .map(|n| n.text().to_owned())
            .collect();
        self.mutated(notices, applied.announcement);
    }

    /// Changes the tuning (G-3.4).
    ///
    /// # Errors
    /// [`AppErrorKind::Invalid`] for a custom tuning that does not parse or is out of range.
    pub fn select_tuning(&mut self, choice: TuningChoice) -> Result<(), AppError> {
        let (tuning, warnings) = resolve_tuning(&choice)?;
        let mut new = self.instrument.clone();
        new.tuning = tuning;
        self.tuning_choice = choice;
        self.warnings = warnings;
        self.change_instrument(new);
        Ok(())
    }

    /// Sets the capo fret (E-FB-4).
    ///
    /// # Errors
    /// [`AppErrorKind::Invalid`] above fret 12.
    pub fn set_capo(&mut self, capo: u8) -> Result<(), AppError> {
        let mut new = self.instrument.clone();
        new.capo = Capo::new(capo)?;
        self.change_instrument(new);
        Ok(())
    }

    /// Changes the view type.
    pub fn set_view_type(&mut self, view: ViewType) {
        self.act(Action::SetView(view));
    }

    /// Changes the labels mode.
    pub fn set_labels(&mut self, labels: LabelsMode) {
        self.act(Action::SetLabels(labels));
    }

    /// Switches the scale overlay on or off and chooses its scale.
    pub fn set_overlay(&mut self, request: OverlayRequest) {
        if request.on {
            self.act(Action::SetOverlayScale(request.scale));
            self.act(Action::SetFollowChords(request.follow));
        } else if self.selection.overlay.on {
            self.act(Action::ToggleOverlay);
        }
    }

    /// Sets one explorer option.
    pub fn set_option(&mut self, option: ExplorerOption) {
        let action = match option {
            ExplorerOption::ShowSevenths(v) => Action::SetShowSevenths(v),
            ExplorerOption::ShowAllOnSet(v) => Action::SetShowAllOnSet(v),
            ExplorerOption::ShowOrder(v) => Action::SetShowOrder(v),
            ExplorerOption::ArpeggioDirection(d) => Action::SetArpeggioDirection(d),
            ExplorerOption::AllowStretch(v) => Action::SetAllowStretch(v),
            ExplorerOption::AllowOmittedFifth(v) => Action::SetAllowOmittedFifth(v),
            ExplorerOption::CapoNames(n) => Action::SetCapoNames(n),
            ExplorerOption::FollowChords(v) => Action::SetFollowChords(v),
        };
        self.act(action);
    }

    /// Feeds changed settings into the explorer: handedness, fret count, default tuning,
    /// default labels, stretches and A4.
    ///
    /// # Errors
    /// [`AppErrorKind::Invalid`] when the new default tuning cannot be built.
    pub fn apply_settings(&mut self, old: &Settings, new: &Settings) -> Result<(), AppError> {
        self.a4 = new.tuner.reference_a4;
        self.handedness = new.instrument.handedness;
        let mut changed = old.instrument.handedness != new.instrument.handedness;
        if old.instrument.allow_stretches != new.instrument.allow_stretches {
            self.selection.options.allow_stretch = new.instrument.allow_stretches;
            self.selection.voicing_index = 0;
            changed = true;
        }
        if old.display.labels_default != new.display.labels_default {
            self.act(Action::SetLabels(new.display.labels_default));
            changed = false;
        }
        if old.instrument.default_tuning != new.instrument.default_tuning {
            self.select_tuning(new.instrument.default_tuning.clone())?;
            changed = false;
        }
        if old.instrument.frets != new.instrument.frets {
            self.set_frets(new.instrument.frets);
            changed = false;
        }
        if changed {
            self.mutated(Vec::new(), String::new());
        }
        Ok(())
    }

    fn set_frets(&mut self, frets: FretCount) {
        let mut new = self.instrument.clone();
        new.frets = frets;
        self.change_instrument(new);
    }

    /// The tuner's target strings: open strings as they sound with the capo (AC-TU-5.1).
    #[must_use]
    pub fn tuner_targets(&self) -> Vec<TunerTarget> {
        let pitches: Vec<Pitch> = StringNumber::LOW_TO_HIGH
            .iter()
            .map(|s| self.instrument.open_sounding_pitch(*s))
            .collect();
        match <[Pitch; 6]>::try_from(pitches) {
            Ok(six) => targets_from_pitches(&six, self.a4),
            Err(_) => Vec::new(),
        }
    }

    /// The notes Play sounds, with delays (FB-8.1).
    pub fn play_notes(&mut self, visible: Option<FretRange>) -> Vec<SoundNote> {
        let plan = playback_plan_cached(
            &self.selection,
            &self.instrument,
            visible,
            Bpm::default(),
            &mut self.cache,
        );
        plan.notes
            .iter()
            .map(|n| SoundNote {
                #[expect(clippy::cast_possible_truncation, reason = "audio frequencies fit f32")]
                frequency_hz: n.pitch.frequency(self.a4) as f32,
                velocity: 0.8,
                delay_ms: n.delay_ms,
            })
            .collect()
    }

    /// The frequency of one open or fretted note (FB-1.7).
    ///
    /// # Errors
    /// [`AppErrorKind::Invalid`] for a bad string, a fret outside the neck, or a fret below the
    /// capo.
    pub fn note_frequency(&self, string: u8, fret: u8) -> Result<f32, AppError> {
        let string = StringNumber::new(string)?;
        let fret = Fret::new(fret)?;
        if fret.value() > self.instrument.frets.value() {
            return Err(AppError::new("That fret is past the end of the neck"));
        }
        let pitch = self.instrument.pitch_at(string, fret)?;
        #[expect(clippy::cast_possible_truncation, reason = "audio frequencies fit f32")]
        Ok(pitch.frequency(self.a4) as f32)
    }

    /// The A4 reference in use.
    #[must_use]
    pub fn a4(&self) -> ReferencePitch {
        self.a4
    }

    /// The revision of the last change.
    #[must_use]
    pub fn revision(&self) -> u32 {
        self.revision
    }

    /// The current selection (read-only).
    #[must_use]
    pub fn selection(&self) -> &Selection {
        &self.selection
    }

    /// The instrument in use (read-only).
    #[must_use]
    pub fn instrument(&self) -> &Instrument {
        &self.instrument
    }

    /// The neck for the current state.
    pub fn view(&mut self, visible: Option<FretRange>) -> FretboardView {
        fretboard_view_cached(
            &self.selection,
            &self.instrument,
            &self.handedness,
            &ViewRequest {
                visible,
                a4: self.a4,
            },
            &mut self.cache,
        )
    }

    /// The snapshot of the current state.
    pub fn snapshot(&mut self, visible: Option<FretRange>) -> ExplorerSnapshot {
        let view = self.view(visible);
        let panel = self.panel();
        ExplorerSnapshot {
            revision: self.revision,
            view,
            panel,
            notices: self.last_notices.clone(),
            announcement: self.last_announcement.clone(),
        }
    }

    fn panel(&self) -> ExplorerPanel {
        let sel = &self.selection;
        let key = sel.key;
        let tonics: Vec<Note> = match key.mode() {
            Mode::Major => mackey_theory::OFFERED_MAJOR_TONICS.to_vec(),
            Mode::NaturalMinor => mackey_theory::OFFERED_MINOR_TONICS.to_vec(),
            mode => Key::all_offered(mode).iter().map(Key::tonic).collect(),
        };
        let key_panel = KeyPanel {
            label: key.name(),
            tonic_choices: tonics
                .into_iter()
                .map(|note| TonicChoice {
                    label: note.to_string(),
                    note,
                    selected: note == key.tonic(),
                })
                .collect(),
            modes: Mode::DIATONIC
                .iter()
                .map(|mode| ModeChoice {
                    label: mode.display_name().to_owned(),
                    mode: *mode,
                    selected: *mode == key.mode(),
                })
                .collect(),
            enharmonic_hint: key
                .enharmonic_alternative()
                .map(|alt| format!("{} sounds the same as {}", key.name(), alt.name())),
        };
        let selected_chord = if sel.focus == Focus::Chord {
            sel.chord
        } else {
            None
        };
        let diatonic = key
            .diatonic_chords(sel.show_sevenths)
            .into_iter()
            .map(|d| DiatonicChip {
                degree: d.degree,
                roman: d.roman.to_string(),
                symbol: d.chord.symbol(),
                selected: selected_chord.is_some_and(|c| {
                    c.root.pitch_class() == d.chord.root.pitch_class()
                        && c.quality == d.chord.quality
                        && c.bass == d.chord.bass
                }),
            })
            .collect();
        let (chord_text, chord_status) = match (&self.chord_error, selected_chord) {
            (Some((text, status)), _) => (text.clone(), status.clone()),
            (None, Some(chord)) => (
                chord.symbol(),
                ChordFieldStatus::Valid {
                    symbol: chord.symbol(),
                },
            ),
            (None, None) => (String::new(), ChordFieldStatus::Empty),
        };
        let in_use = sel.overlay_scale();
        let scale_choices = Mode::ALL
            .iter()
            .map(|mode| {
                let scale = Scale {
                    tonic: key.tonic(),
                    mode: *mode,
                };
                ScaleChoice {
                    label: scale.name(),
                    scale,
                    selected: in_use == Some(scale),
                }
            })
            .collect();
        ExplorerPanel {
            key: key_panel,
            diatonic,
            chord_text,
            chord_status,
            instrument: self.instrument_panel(),
            selection: sel.clone(),
            scale_choices,
        }
    }

    fn instrument_panel(&self) -> InstrumentPanel {
        let mut choices: Vec<TuningItem> = TuningPreset::ALL
            .iter()
            .map(|p| {
                let t = Tuning::preset(*p);
                TuningItem {
                    label: t.name.clone(),
                    pitches: pitch_names(&t),
                    choice: TuningChoice::Preset(*p),
                    selected: self.tuning_choice == TuningChoice::Preset(*p),
                }
            })
            .collect();
        if matches!(self.tuning_choice, TuningChoice::Custom(_)) {
            choices.push(TuningItem {
                label: "Custom".to_owned(),
                pitches: pitch_names(&self.instrument.tuning),
                choice: self.tuning_choice.clone(),
                selected: true,
            });
        }
        InstrumentPanel {
            tuning_name: self.instrument.tuning.name.clone(),
            tuning_pitches: pitch_names(&self.instrument.tuning),
            tuning_choices: choices,
            warnings: self
                .warnings
                .iter()
                .map(|w| match w {
                    TuningWarning::UnusualTension { string, .. } => {
                        format!("String {}: {}", string.number(), w.message())
                    }
                })
                .collect(),
            capo: self.instrument.capo.value(),
            frets: self.instrument.frets.value(),
            handedness: self.handedness,
        }
    }
}
