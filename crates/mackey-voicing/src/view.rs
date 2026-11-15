//! The `FretboardView`: the one value the UI draws the neck from (G-3.2, FR-019).
//!
//! Everything a test should check is computed here: which positions light up, their role, labels
//! for every labels mode, accessibility text, notices, the capo label, off-range counts and the
//! "Fit selection" range. The UI never re-derives theory (ADR 0007).

use mackey_theory::{
    Chord, ChordTone, DisplayNote, Note, PitchClass, ReferencePitch, Scale, Speller, ToneRole,
    analyze,
};
use serde::{Deserialize, Serialize};

use crate::arpeggio::{arpeggio, arpeggio_positions, default_arpeggio_position};
use crate::error::VoicingError;
use crate::instrument::{Fret, Handedness, Instrument, StringNumber};
use crate::patterns::{PatternNote, full_neck, pattern_label, scale_patterns};
use crate::selection::{
    CapoNames, Context, Focus, LabelsMode, Notice, Selection, ViewType, VoicingCache, announce,
    spoken_chord,
};
use crate::shapes::is_curated_tuning;
use crate::triads::{triad_ladder, triad_note, triads};
use crate::voicing::{Barre, Origin, StringState, Voicing, VoicingGroup, VoicingResult};

/// An inclusive range of frets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct FretRange {
    /// First fret.
    pub from: Fret,
    /// Last fret.
    pub to: Fret,
}

impl FretRange {
    /// Validates `from <= to`.
    ///
    /// # Errors
    /// [`VoicingError::InvalidRange`] when the range is reversed.
    pub fn new(from: Fret, to: Fret) -> Result<Self, VoicingError> {
        if from <= to {
            Ok(Self { from, to })
        } else {
            Err(VoicingError::InvalidRange)
        }
    }
}

/// What the UI asks for with each view.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct ViewRequest {
    /// The visible fret range, or `None` for the whole neck (no off-range counts).
    pub visible: Option<FretRange>,
    /// The reference pitch (carried for playback; the view's text does not use it).
    pub a4: ReferencePitch,
}

/// The role of a drawn note (G-2); the UI picks shape and colour.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum NoteRole {
    /// Root: filled rounded square.
    Root,
    /// Third: filled circle.
    Third,
    /// Fifth: filled circle.
    Fifth,
    /// Seventh: filled circle.
    Seventh,
    /// Other chord tones (sus note, 6th, 9th, 11th, 13th): filled circle.
    Extension,
    /// Scale tone that is not a chord tone: hollow ring.
    ScaleTone,
    /// The single selected note: filled diamond.
    SingleNote,
}

impl NoteRole {
    /// The word the tooltip shows ("Root", "3rd", ...).
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            NoteRole::Root => "Root",
            NoteRole::Third => "3rd",
            NoteRole::Fifth => "5th",
            NoteRole::Seventh => "7th",
            NoteRole::Extension => "Extension",
            NoteRole::ScaleTone => "Scale tone",
            NoteRole::SingleNote => "Selected note",
        }
    }

    fn of_tone(role: ToneRole) -> Self {
        match role {
            ToneRole::Root => NoteRole::Root,
            ToneRole::Third => NoteRole::Third,
            ToneRole::Fifth => NoteRole::Fifth,
            ToneRole::Seventh => NoteRole::Seventh,
            ToneRole::Extension => NoteRole::Extension,
        }
    }
}

/// One precomputed label per labels mode (G-2.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct PositionLabels {
    /// "G", "Bb"; never a double accidental (G-1.6).
    pub note_name: String,
    /// "1", "b3", "5", relative to the overlay scale's tonic, else the key's (SC-5).
    pub scale_degree: String,
    /// "R", "3", "b7" relative to the chord root; `None` without a chord.
    pub interval: Option<String>,
    /// "0" to "4"; `None` unless the position is part of a voicing, triad or arpeggio.
    pub finger: Option<String>,
}

/// The hover or tap text (FB-1.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Tooltip {
    /// "G3".
    pub note_octave: String,
    /// Scale degree in the key: "1".
    pub degree: String,
    /// Interval to the chord root: "R"; `None` without a chord.
    pub interval: Option<String>,
    /// "Root".
    pub role: String,
    /// The double-accidental spelling when the label was simplified (G-1.6).
    pub theory_spelling: Option<String>,
}

/// One drawn note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct ViewPosition {
    /// The string.
    pub string: StringNumber,
    /// The absolute fret.
    pub fret: Fret,
    /// True when the string rings at the capo fret (drawn as a ring above the nut).
    pub is_open: bool,
    /// The sounding note, spelled for display.
    pub note: Note,
    /// The octave (scientific pitch, C4 = middle C).
    pub octave: i8,
    /// Shape and colour.
    pub role: NoteRole,
    /// Labels for every labels mode.
    pub labels: PositionLabels,
    /// The finger, 0 for open, when this note belongs to a voicing, triad or arpeggio.
    pub finger: Option<u8>,
    /// The play-order number with "Show order"; in "all on string set" the inversion number.
    pub order: Option<u8>,
    /// The tooltip.
    pub tooltip: Tooltip,
    /// "String 3, fret 2, A, third of F".
    pub a11y_label: String,
    /// Always true for emitted positions (frets below the capo are never emitted).
    pub playable: bool,
}

/// What a string marker is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum MarkerKind {
    /// A muted string: "x" above the nut.
    Muted,
}

/// A marker above the nut.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct StringMarker {
    /// The string.
    pub string: StringNumber,
    /// The marker.
    pub kind: MarkerKind,
}

/// The open-string label at the nut side (FB-1.3), as it sounds with the capo on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct StringLabel {
    /// The string.
    pub string: StringNumber,
    /// The note.
    pub note: Note,
    /// The octave.
    pub octave: i8,
}

/// A fretboard inlay (FB-1.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Inlay {
    /// The fret.
    pub fret: Fret,
    /// True for the double dots at 12 and 24.
    pub double: bool,
}

/// Highlighted positions outside the visible range (E-FB-5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct OffRange {
    /// How many are below the first visible fret (left of the range).
    pub below: u16,
    /// How many are above the last visible fret (right of the range).
    pub above: u16,
}

/// "3 of 7" (FB-9.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct ListPosition {
    /// "3 of 7".
    pub label: String,
    /// 1-based index.
    pub index: usize,
    /// List length.
    pub count: usize,
}

/// "G shape (sounds A)" (LH-5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct CapoLabel {
    /// The shape the fingers play: "G".
    pub shape: String,
    /// What sounds: "A".
    pub sounding: String,
    /// "G shape (sounds A)", or "A (G shape)" with sounding-first names.
    pub text: String,
}

/// One mini chord diagram of the voicing list (FB-5.2).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct VoicingListEntry {
    /// The list group.
    pub group: VoicingGroup,
    /// "Open C", "E-shape barre, fret 3".
    pub name: String,
    /// The diagram's first fret.
    pub lowest_fret: Fret,
    /// Strings, low to high.
    pub strings: [StringState; 6],
    /// Fingers, low to high (`None` muted, `Some(0)` open).
    pub fingers: [Option<u8>; 6],
    /// Barres.
    pub barres: Vec<Barre>,
    /// True when the voicing came from the search in a tuning outside the curated library (LH-4).
    pub found_by_search: bool,
    /// True for the voicing on the neck.
    pub selected: bool,
    /// The capo label, when a capo is on.
    pub capo_label: Option<CapoLabel>,
}

/// The one value the UI draws from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct FretboardView {
    /// The number of frets.
    pub fret_count: u8,
    /// The capo.
    pub capo: u8,
    /// Left-handed: the nut is drawn on the right.
    pub mirrored: bool,
    /// String 6 is drawn at the top.
    pub flipped: bool,
    /// Open-string labels, string 6 first.
    pub strings: Vec<StringLabel>,
    /// Inlays up to the fret count.
    pub inlays: Vec<Inlay>,
    /// Fret numbers to show (FB-1.2).
    pub fret_numbers: Vec<u8>,
    /// Frets below this are dimmed and not selectable (LH-3).
    pub dimmed_below: u8,
    /// The notes to draw.
    pub positions: Vec<ViewPosition>,
    /// Muted-string markers.
    pub markers: Vec<StringMarker>,
    /// Barres to draw.
    pub barres: Vec<Barre>,
    /// Ghost notes (run mode, later phases); empty in this run.
    pub ghost: Vec<ViewPosition>,
    /// "3 of 7".
    pub list_position: Option<ListPosition>,
    /// Notices for the current state.
    pub notices: Vec<Notice>,
    /// "Not in key (V/V)".
    pub badge: Option<String>,
    /// A short description such as "C major, Open C".
    pub headline: String,
    /// The capo label of the selected chord.
    pub chord_label: Option<CapoLabel>,
    /// Highlighted notes outside the visible range.
    pub off_range: OffRange,
    /// The range "Fit selection" zooms to: every highlighted note plus one fret each side.
    pub fit_range: FretRange,
    /// The voicing list (voicing view) or the inversion ladder (triads, all on string set).
    pub voicing_list: Vec<VoicingListEntry>,
    /// Set when no voicing was found, with the relax options.
    pub no_voicing: Option<VoicingResult>,
    /// "Triad of Cmaj7: C E G".
    pub triad_note: Option<String>,
    /// The `VoiceOver` text for this state.
    pub announcement: String,
    /// The labels mode in effect (Finger falls back to note names when nothing is fingered).
    pub labels_mode: LabelsMode,
    /// "G major".
    pub key_label: String,
}

const CHROMATIC_INTERVAL: [&str; 12] = [
    "R", "b2", "2", "b3", "3", "4", "b5", "5", "b6", "6", "b7", "7",
];

/// Semitones of the major scale degrees, used to name notes outside the scale.
const MAJOR_OFFSETS: [i16; 7] = [0, 2, 4, 5, 7, 9, 11];

/// The degree label of a note relative to the scale's tonic. Scale notes use the scale's own
/// intervals; a note outside the scale is labelled from its spelling, so F# in C is "#4" and
/// Bb is "b7" (G-2.2).
fn scale_degree_label(scale: Scale, note: Note) -> String {
    let pc = note.pitch_class();
    if let Some(degree) = scale.degree_of(pc)
        && let Some(interval) = scale.mode.intervals().get(degree.index())
    {
        let label = interval.degree_label();
        return if label == "R" { "1".to_owned() } else { label };
    }
    let steps =
        (i16::from(note.letter.index()) - i16::from(scale.tonic.letter.index())).rem_euclid(7);
    let natural = MAJOR_OFFSETS
        .get(usize::try_from(steps).unwrap_or(0))
        .copied()
        .unwrap_or(0);
    let mut diff = i16::from(scale.tonic.pitch_class().semitones_to(pc)) - natural;
    if diff > 6 {
        diff -= 12;
    } else if diff < -6 {
        diff += 12;
    }
    let prefix = match diff {
        -2 => "bb",
        -1 => "b",
        1 => "#",
        2 => "##",
        _ => "",
    };
    format!("{prefix}{}", steps + 1)
}

fn scale_role(scale: Scale, pc: PitchClass) -> NoteRole {
    scale
        .degree_of(pc)
        .and_then(|d| scale.mode.intervals().get(d.index()))
        .map_or(NoteRole::ScaleTone, |i| match ToneRole::of_interval(*i) {
            ToneRole::Extension => NoteRole::ScaleTone,
            other => NoteRole::of_tone(other),
        })
}

fn octave_of(midi: i32, note: Note) -> i8 {
    let base = i32::from(note.letter.natural_pc().value()) + i32::from(note.accidental.value());
    i8::try_from((midi - base).div_euclid(12) - 1).unwrap_or(0)
}

/// Spelling and roles for one view.
struct Names {
    spelled: [Note; 12],
    chord: Option<(Chord, Vec<ChordTone>)>,
    /// The scale the degree labels count from: the overlay scale, else the key's.
    degree_scale: Scale,
    /// The scale whose roles apply when there is no chord.
    role_scale: Scale,
}

impl Names {
    fn new(sel: &Selection, chord: Option<Chord>, overlay: Option<Scale>) -> Self {
        let speller = Speller::new(sel.key);
        let mut table: [Note; 12] =
            std::array::from_fn(|i| speller.spell(PitchClass::new(i32::try_from(i).unwrap_or(0))));
        if let Some(scale) = overlay {
            for note in scale.degrees() {
                if let Some(slot) = table.get_mut(usize::from(note.pitch_class().value())) {
                    *slot = note;
                }
            }
        }
        let tones = chord.map(|c| (c, c.tones()));
        if let Some((c, tones)) = &tones {
            for tone in tones {
                if let Some(slot) = table.get_mut(usize::from(tone.note.pitch_class().value())) {
                    *slot = tone.note;
                }
            }
            if let Some(bass) = c.bass
                && let Some(slot) = table.get_mut(usize::from(bass.pitch_class().value()))
            {
                *slot = bass;
            }
        }
        let key_scale = sel.key.scale();
        Self {
            spelled: table,
            chord: tones,
            degree_scale: overlay.unwrap_or(key_scale),
            role_scale: overlay.unwrap_or(key_scale),
        }
    }

    fn theory(&self, pc: PitchClass) -> Note {
        self.spelled
            .get(usize::from(pc.value()))
            .copied()
            .unwrap_or(Note::natural(mackey_theory::Letter::C))
    }

    fn chord_tone(&self, pc: PitchClass) -> Option<&ChordTone> {
        self.chord
            .as_ref()
            .and_then(|(_, tones)| tones.iter().find(|t| t.note.pitch_class() == pc))
    }

    fn role(&self, pc: PitchClass) -> NoteRole {
        match &self.chord {
            Some((chord, _)) => match self.chord_tone(pc) {
                Some(tone) => NoteRole::of_tone(tone.role),
                None if chord.bass.is_some_and(|b| b.pitch_class() == pc) => NoteRole::Extension,
                None => NoteRole::ScaleTone,
            },
            None => scale_role(self.role_scale, pc),
        }
    }

    fn interval_label(&self, pc: PitchClass) -> Option<String> {
        let (chord, _) = self.chord.as_ref()?;
        if let Some(tone) = self.chord_tone(pc) {
            return Some(tone.interval.degree_label());
        }
        let offset = chord.root.pitch_class().semitones_to(pc);
        Some(
            CHROMATIC_INTERVAL
                .get(usize::from(offset))
                .copied()
                .unwrap_or("?")
                .to_owned(),
        )
    }
}

/// A position before names and labels are attached.
#[derive(Clone, Copy)]
struct Base {
    index: usize,
    fret: u8,
    role: Option<NoteRole>,
    finger: Option<u8>,
    order: Option<u8>,
}

fn role_phrase(role: NoteRole, subject: &str) -> String {
    match role {
        NoteRole::Root => format!("root of {subject}"),
        NoteRole::Third => format!("third of {subject}"),
        NoteRole::Fifth => format!("fifth of {subject}"),
        NoteRole::Seventh => format!("seventh of {subject}"),
        NoteRole::Extension => format!("extension of {subject}"),
        NoteRole::ScaleTone => format!("scale tone of {subject}"),
        NoteRole::SingleNote => "selected note".to_owned(),
    }
}

fn make_position(
    base: &Base,
    instrument: &Instrument,
    names: &Names,
    subject: &str,
) -> Option<ViewPosition> {
    let string = StringNumber::from_index(base.index)?;
    let fret = Fret::new(base.fret).ok()?;
    let midi = instrument.midi_at_index(base.index, base.fret);
    let pc = PitchClass::new(midi);
    let display = DisplayNote::of(names.theory(pc));
    let role = base.role.unwrap_or_else(|| names.role(pc));
    let is_open = base.fret == instrument.capo.value();
    let octave = octave_of(midi, display.label);
    let degree = scale_degree_label(names.degree_scale, names.theory(pc));
    let interval = names.interval_label(pc);
    let note_name = display.label.name();
    let finger_text = base.finger.map(|f| f.to_string());
    let where_ = if is_open {
        format!("String {}, open", string.number())
    } else {
        format!("String {}, fret {}", string.number(), base.fret)
    };
    Some(ViewPosition {
        string,
        fret,
        is_open,
        note: display.label,
        octave,
        role,
        labels: PositionLabels {
            note_name: note_name.clone(),
            scale_degree: degree.clone(),
            interval: interval.clone(),
            finger: finger_text,
        },
        finger: base.finger,
        order: base.order,
        tooltip: Tooltip {
            note_octave: format!("{note_name}{octave}"),
            degree,
            interval,
            role: role.name().to_owned(),
            theory_spelling: display.is_simplified().then(|| display.theory.name()),
        },
        a11y_label: format!("{where_}, {note_name}, {}", role_phrase(role, subject)),
        playable: true,
    })
}

fn pattern_bases(notes: &[PatternNote], instrument: &Instrument) -> Vec<Base> {
    notes
        .iter()
        .filter(|n| n.fret.value() >= instrument.capo.value())
        .map(|n| Base {
            index: n.string.index(),
            fret: n.fret.value(),
            role: None,
            finger: None,
            order: None,
        })
        .collect()
}

fn voicing_bases(v: &Voicing, instrument: &Instrument) -> Vec<Base> {
    v.strings
        .iter()
        .enumerate()
        .filter_map(|(index, state)| {
            let fret = match state {
                StringState::Muted => return None,
                StringState::Open => instrument.capo.value(),
                StringState::Fretted { fret } => fret.value(),
            };
            Some(Base {
                index,
                fret,
                role: None,
                finger: v.fingering.fingers.get(index).copied().flatten(),
                order: None,
            })
        })
        .collect()
}

/// "G shape (sounds A)" for a chord under a capo (LH-5, AC-LH-5.1).
fn capo_label(chord: Chord, instrument: &Instrument, names: CapoNames) -> Option<CapoLabel> {
    let capo = instrument.capo.value();
    if capo == 0 {
        return None;
    }
    let preference = if chord.root.accidental.value() < 0 {
        mackey_theory::AccidentalPreference::Flats
    } else {
        mackey_theory::AccidentalPreference::Sharps
    };
    let shape = chord.transpose(-i32::from(capo), preference).symbol();
    let sounding = chord.symbol();
    let text = match names {
        CapoNames::ShapeFirst => format!("{shape} shape (sounds {sounding})"),
        CapoNames::SoundingFirst => format!("{sounding} ({shape} shape)"),
    };
    Some(CapoLabel {
        shape,
        sounding,
        text,
    })
}

fn entry_of(
    v: &Voicing,
    selected: bool,
    curated_tuning: bool,
    label: Option<CapoLabel>,
) -> VoicingListEntry {
    VoicingListEntry {
        group: v.group,
        name: v.name.clone(),
        lowest_fret: v.lowest_fret,
        strings: v.strings,
        fingers: v.fingering.fingers,
        barres: v.fingering.barres.clone(),
        found_by_search: v.origin == Origin::Search && !curated_tuning,
        selected,
        capo_label: label,
    }
}

fn inlays(frets: u8) -> Vec<Inlay> {
    [3u8, 5, 7, 9, 12, 15, 17, 19, 21, 24]
        .into_iter()
        .filter(|f| *f <= frets)
        .filter_map(|f| {
            Fret::new(f).ok().map(|fret| Inlay {
                fret,
                double: f == 12 || f == 24,
            })
        })
        .collect()
}

fn fret_numbers(frets: u8, visible: Option<FretRange>) -> Vec<u8> {
    let mut numbers: Vec<u8> = [3u8, 5, 7, 9, 12, 15, 17, 19, 21, 24]
        .into_iter()
        .filter(|f| *f <= frets)
        .collect();
    if let Some(range) = visible {
        numbers.push(range.from.value());
    }
    numbers.sort_unstable();
    numbers.dedup();
    numbers
}

/// Builds the view for a selection (FB-1 to FB-9). Searches voicings itself when the voicing
/// view needs them; use [`fretboard_view_cached`] to reuse a search.
///
/// ```
/// use mackey_voicing::{fretboard_view, Handedness, Instrument, Selection, ViewRequest};
/// let view = fretboard_view(
///     &Selection::default_g_major(),
///     &Instrument::default(),
///     &Handedness::default(),
///     &ViewRequest::default(),
/// );
/// assert_eq!(view.key_label, "G major");
/// assert!(!view.positions.is_empty());
/// ```
#[must_use]
pub fn fretboard_view(
    sel: &Selection,
    instrument: &Instrument,
    handedness: &Handedness,
    req: &ViewRequest,
) -> FretboardView {
    fretboard_view_cached(sel, instrument, handedness, req, &mut VoicingCache::new())
}

/// [`fretboard_view`] with a caller-owned voicing cache (the app keeps one per explorer).
#[must_use]
#[allow(clippy::too_many_lines)] // One linear build of a record with many fields; splitting hides the order.
pub fn fretboard_view_cached(
    sel: &Selection,
    instrument: &Instrument,
    handedness: &Handedness,
    req: &ViewRequest,
    cache: &mut VoicingCache,
) -> FretboardView {
    let ctx = Context {
        instrument,
        handedness,
    };
    let chord = if sel.focus == Focus::Chord {
        sel.chord
    } else {
        None
    };
    let overlay = sel.overlay_scale();
    let names = Names::new(sel, chord, overlay);
    let capo = instrument.capo.value();
    let frets = instrument.frets.value();
    let curated_tuning = is_curated_tuning(instrument);
    let label = chord.and_then(|c| capo_label(c, instrument, sel.capo_names));
    let subject = match chord {
        Some(c) => c.symbol(),
        None => overlay.map_or_else(|| sel.key.name(), |s| s.name()),
    };

    let mut bases: Vec<Base> = Vec::new();
    let mut markers: Vec<StringMarker> = Vec::new();
    let mut barres: Vec<Barre> = Vec::new();
    let mut list_position: Option<ListPosition> = None;
    let mut voicing_list: Vec<VoicingListEntry> = Vec::new();
    let mut no_voicing: Option<VoicingResult> = None;
    let mut notices: Vec<Notice> = Vec::new();
    let mut triad_text: Option<String> = None;
    let mut headline = format!("{}, all notes", sel.key.name());
    let mut has_fingers = false;

    let scale_bases = |scale: &Scale, pattern_index: Option<usize>| -> Vec<Base> {
        match pattern_index.and_then(|i| scale_patterns(scale, instrument).into_iter().nth(i)) {
            Some(p) => pattern_bases(&p.notes, instrument),
            None => pattern_bases(&full_neck(scale, instrument).notes, instrument),
        }
    };

    match (sel.focus, chord) {
        (Focus::Chord, Some(chord)) => {
            let chord_pcs = chord.pitch_classes();
            headline = format!(
                "{}, {}",
                spoken_chord(&chord),
                sel.view.label().to_lowercase()
            );
            match sel.view {
                ViewType::AllTones => {
                    for index in 0..6 {
                        for fret in capo..=frets {
                            if chord_pcs.contains(instrument.pc_at_index(index, fret)) {
                                bases.push(Base {
                                    index,
                                    fret,
                                    role: None,
                                    finger: None,
                                    order: None,
                                });
                            }
                        }
                    }
                }
                ViewType::Voicing => {
                    let result = cache.get(&chord, instrument, &sel.options).clone();
                    match &result {
                        VoicingResult::Found(list) => {
                            let at = sel.voicing_index.min(list.len().saturating_sub(1));
                            if let Some(v) = list.get(at) {
                                bases = voicing_bases(v, instrument);
                                has_fingers = true;
                                barres.clone_from(&v.fingering.barres);
                                for (i, s) in v.strings.iter().enumerate() {
                                    if let (StringState::Muted, Some(string)) =
                                        (s, StringNumber::from_index(i))
                                    {
                                        markers.push(StringMarker {
                                            string,
                                            kind: MarkerKind::Muted,
                                        });
                                    }
                                }
                                headline = format!("{}, {}", spoken_chord(&chord), v.name);
                            }
                            list_position = Some(ListPosition {
                                label: format!("{} of {}", at + 1, list.len()),
                                index: at + 1,
                                count: list.len(),
                            });
                            voicing_list = list
                                .iter()
                                .enumerate()
                                .map(|(i, v)| entry_of(v, i == at, curated_tuning, label.clone()))
                                .collect();
                        }
                        VoicingResult::NoneFound { .. } => {
                            notices.push(Notice::NoPlayableVoicing);
                            no_voicing = Some(result);
                        }
                    }
                }
                ViewType::Triad => {
                    triad_text = triad_note(&chord);
                    let inversion = sel.inversion.clamped(false);
                    if sel.show_all_on_set {
                        let ladder = triad_ladder(&chord, instrument, sel.string_set);
                        if ladder.is_empty() {
                            notices.push(Notice::TriadUnavailable);
                        }
                        for (inv, v) in &ladder {
                            let number = u8::try_from(inv.index() + 1).ok();
                            for mut b in voicing_bases(v, instrument) {
                                b.order = number;
                                bases.push(b);
                            }
                            voicing_list.push(entry_of(
                                v,
                                *inv == inversion,
                                curated_tuning,
                                label.clone(),
                            ));
                            if let Some(entry) = voicing_list.last_mut() {
                                entry.name = format!("{} inversion", inv.label());
                            }
                        }
                        has_fingers = !ladder.is_empty();
                        headline = format!(
                            "{}, all inversions, strings {}",
                            spoken_chord(&chord),
                            sel.string_set.label()
                        );
                    } else {
                        let result = triads(&chord, instrument, sel.string_set, inversion);
                        let list = result.placements();
                        if list.is_empty() {
                            notices.push(Notice::TriadUnavailable);
                        } else {
                            let at = usize::from(sel.position.unwrap_or(0)).min(list.len() - 1);
                            if let Some(v) = list.get(at) {
                                bases = voicing_bases(v, instrument);
                                has_fingers = true;
                            }
                            list_position = Some(ListPosition {
                                label: format!("{} of {}", at + 1, list.len()),
                                index: at + 1,
                                count: list.len(),
                            });
                        }
                        headline = format!(
                            "{}, {}, strings {}",
                            spoken_chord(&chord),
                            inversion.spoken(),
                            sel.string_set.label()
                        );
                    }
                }
                ViewType::Arpeggio => {
                    let selected = cache
                        .get(&chord, instrument, &sel.options)
                        .voicings()
                        .get(sel.voicing_index)
                        .cloned();
                    let positions = arpeggio_positions(&chord, instrument);
                    let start = sel
                        .position
                        .filter(|p| *p >= capo)
                        .or_else(|| {
                            default_arpeggio_position(&chord, instrument, selected.as_ref())
                        })
                        .unwrap_or(capo);
                    let arp = arpeggio(&chord, instrument, start, sel.arpeggio_direction);
                    let order = arp.order_numbers();
                    for (i, n) in arp.notes.iter().enumerate() {
                        bases.push(Base {
                            index: n.string.index(),
                            fret: n.fret.value(),
                            role: None,
                            finger: Some(n.finger),
                            order: sel.show_order.then(|| order.get(i).copied()).flatten(),
                        });
                    }
                    has_fingers = true;
                    if let Some(at) = positions.iter().position(|p| *p == start) {
                        list_position = Some(ListPosition {
                            label: format!("{} of {}", at + 1, positions.len()),
                            index: at + 1,
                            count: positions.len(),
                        });
                    }
                    headline = format!("{}, arpeggio, position {start}", spoken_chord(&chord));
                }
            }
        }
        (Focus::Note, _) => {
            if let Some((string, fret)) = sel.note {
                let index = string.index();
                let pc = instrument.pc_at_index(index, fret.value());
                for i in 0..6 {
                    for f in capo..=frets {
                        if instrument.pc_at_index(i, f) == pc {
                            let selected = i == index && f == fret.value();
                            bases.push(Base {
                                index: i,
                                fret: f,
                                role: Some(if selected {
                                    NoteRole::SingleNote
                                } else {
                                    NoteRole::ScaleTone
                                }),
                                finger: None,
                                order: None,
                            });
                        }
                    }
                }
                headline = format!("Note {}", names.theory(pc).simplified().name());
            }
        }
        _ => {
            // Key notes (FB-3) or a scale on its own.
            let scale = overlay.unwrap_or_else(|| sel.key.scale());
            bases = scale_bases(
                &scale,
                if overlay.is_some() {
                    sel.overlay.pattern_index
                } else {
                    None
                },
            );
            if overlay.is_some() {
                headline = format!("{} scale", scale.name());
            }
        }
    }

    // Scale overlay on top of a chord view or a note (SC-2): scale tones that are not chord
    // tones are rings; positions already drawn keep their own role and finger.
    let primary: Vec<(usize, u8)> = bases.iter().map(|b| (b.index, b.fret)).collect();
    if let Some(scale) = overlay
        && matches!(sel.focus, Focus::Chord | Focus::Note)
    {
        let extra = scale_bases(&scale, sel.overlay.pattern_index);
        for e in extra {
            if !bases.iter().any(|b| b.index == e.index && b.fret == e.fret) {
                bases.push(e);
            }
        }
        if let Some(i) = sel.overlay.pattern_index {
            let n = scale_patterns(&scale, instrument).len();
            if list_position.is_none()
                && let Some(label) = pattern_label(scale, instrument, i)
            {
                list_position = Some(ListPosition {
                    label,
                    index: i + 1,
                    count: n,
                });
            }
        }
    }
    if matches!(sel.focus, Focus::None | Focus::Scale)
        && let (Some(scale), Some(i)) = (overlay, sel.overlay.pattern_index)
    {
        let n = scale_patterns(&scale, instrument).len();
        if let Some(label) = pattern_label(scale, instrument, i) {
            list_position = Some(ListPosition {
                label,
                index: i + 1,
                count: n,
            });
        }
    }

    bases.sort_by_key(|b| (b.index, b.fret));
    bases.dedup_by_key(|b| (b.index, b.fret));
    let positions: Vec<ViewPosition> = bases
        .iter()
        .filter_map(|b| make_position(b, instrument, &names, &subject))
        .collect();

    // The fit range uses the selection's own notes, not the overlay.
    let frets_used: Vec<u8> = primary.iter().map(|(_, f)| *f).collect();
    let fit_range = match (frets_used.iter().min(), frets_used.iter().max()) {
        (Some(&lo), Some(&hi)) => FretRange {
            from: Fret::new(lo.saturating_sub(1)).unwrap_or(Fret::OPEN),
            to: Fret::new((hi + 1).min(frets)).unwrap_or(Fret::OPEN),
        },
        _ => FretRange {
            from: Fret::OPEN,
            to: Fret::new(frets).unwrap_or(Fret::OPEN),
        },
    };
    let mut off_range = OffRange::default();
    if let Some(range) = req.visible {
        for p in &positions {
            if p.fret < range.from {
                off_range.below += 1;
            } else if p.fret > range.to {
                off_range.above += 1;
            }
        }
    }

    let badge = chord.and_then(|c| analyze(&sel.key, &c).badge());
    let strings: Vec<StringLabel> = StringNumber::LOW_TO_HIGH
        .iter()
        .map(|&string| {
            let pitch = instrument.open_sounding_pitch(string);
            StringLabel {
                string,
                note: pitch.note(),
                octave: pitch.octave(),
            }
        })
        .collect();
    let mut labels_mode = sel.labels;
    if labels_mode == LabelsMode::Finger && !has_fingers {
        labels_mode = LabelsMode::NoteName;
    }
    if labels_mode == LabelsMode::Interval && chord.is_none() {
        labels_mode = LabelsMode::NoteName;
    }
    FretboardView {
        fret_count: frets,
        capo,
        mirrored: handedness.left_handed,
        flipped: handedness.flip_strings,
        strings,
        inlays: inlays(frets),
        fret_numbers: fret_numbers(frets, req.visible),
        dimmed_below: capo,
        positions,
        markers,
        barres,
        ghost: Vec::new(),
        list_position,
        notices,
        badge,
        headline,
        chord_label: label,
        off_range,
        fit_range,
        voicing_list,
        no_voicing,
        triad_note: triad_text,
        announcement: announce(sel, &ctx, cache),
        labels_mode,
        key_label: sel.key.name(),
    }
}
