//! The selection model and all FB-9 cycling (G-3, FR-020).
//!
//! `apply` is a pure function `(Selection, Context, Action) -> Applied`. Lists (diatonic chords,
//! voicings, string sets, inversions, positions, patterns, labels) are computed from the context
//! inside `apply`, so wrap-around uses the real list length for the current instrument.

use mackey_theory::{Chord, Degree, Key, Mode, Note, Scale, roman_to_chord};
use serde::{Deserialize, Serialize};

use crate::arpeggio::{ArpeggioDirection, arpeggio_positions, default_arpeggio_position};
use crate::instrument::{Capo, Fret, Handedness, Instrument, Inversion, StringNumber, StringSet};
use crate::patterns::{scale_for_chord, scale_patterns};
use crate::search::voicings_for;
use crate::triads::triads;
use crate::voicing::{VoicingOptions, VoicingResult};

/// What is in focus (G-3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum Focus {
    /// The key's notes (FB-3).
    None,
    /// A chord.
    Chord,
    /// A scale overlay on its own.
    Scale,
    /// One note.
    Note,
}

/// The chord views (FB-4.6, FB-5, FB-6, FB-7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum ViewType {
    /// Every occurrence of each chord tone.
    AllTones,
    /// One voicing from the list.
    Voicing,
    /// A triad on a string set.
    Triad,
    /// An arpeggio in a position.
    Arpeggio,
}

impl ViewType {
    /// The next view in FB-9 order, wrapping.
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            ViewType::AllTones => ViewType::Voicing,
            ViewType::Voicing => ViewType::Triad,
            ViewType::Triad => ViewType::Arpeggio,
            ViewType::Arpeggio => ViewType::AllTones,
        }
    }

    /// The label "All tones", "Voicing", "Triad", "Arpeggio".
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            ViewType::AllTones => "All tones",
            ViewType::Voicing => "Voicing",
            ViewType::Triad => "Triad",
            ViewType::Arpeggio => "Arpeggio",
        }
    }
}

/// What a position's label shows (G-2.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum LabelsMode {
    /// "G", "Bb".
    NoteName,
    /// "1", "b3", "5".
    ScaleDegree,
    /// "R", "3", "b7" (needs a chord).
    Interval,
    /// "1" to "4" (needs a voicing, triad or arpeggio).
    Finger,
    /// No label.
    None,
}

/// Which name leads under a capo (LH-5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum CapoNames {
    /// "G shape (sounds A)".
    ShapeFirst,
    /// "A (G shape)".
    SoundingFirst,
}

/// The scale overlay (SC-1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct ScaleOverlay {
    /// Whether the overlay is drawn.
    pub on: bool,
    /// The chosen scale; `None` means the key's own scale (or the follow-the-chords choice).
    pub scale: Option<Scale>,
    /// Change the scale with the chord (SC-4).
    pub follow_chords: bool,
    /// The pattern shown; `None` is the full neck (SC-3).
    pub pattern_index: Option<usize>,
}

/// The one current selection of a window (G-3.1).
///
/// `position` has one meaning per view: the placement index in the triad view, the start fret in
/// the arpeggio view, and nothing elsewhere. `None` uses the default rule (FB-7.4, lowest
/// placement).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Selection {
    /// The key.
    pub key: Key,
    /// What is in focus.
    pub focus: Focus,
    /// The chord (set exactly when `focus` is `Chord`).
    pub chord: Option<Chord>,
    /// The chord view.
    pub view: ViewType,
    /// Index into the voicing list.
    pub voicing_index: usize,
    /// The triad inversion.
    pub inversion: Inversion,
    /// The triad string set.
    pub string_set: StringSet,
    /// Show all three inversions on the string set (FB-6.2).
    pub show_all_on_set: bool,
    /// Triad placement index or arpeggio start fret.
    pub position: Option<u8>,
    /// The scale overlay.
    pub overlay: ScaleOverlay,
    /// The labels mode.
    pub labels: LabelsMode,
    /// Capo name emphasis.
    pub capo_names: CapoNames,
    /// Seventh chords on the diatonic strip (FB-4.2).
    pub show_sevenths: bool,
    /// Show the play-order numbers (FB-7.3).
    pub show_order: bool,
    /// Arpeggio direction.
    pub arpeggio_direction: ArpeggioDirection,
    /// Voicing search options.
    pub options: VoicingOptions,
    /// The selected note (set exactly when `focus` is `Note`).
    pub note: Option<(StringNumber, Fret)>,
}

impl Selection {
    /// The G-4 defaults: G major, nothing in focus, note-name labels.
    ///
    /// ```
    /// use mackey_voicing::{Focus, Selection};
    /// let s = Selection::default_g_major();
    /// assert_eq!(s.key.name(), "G major");
    /// assert_eq!(s.focus, Focus::None);
    /// ```
    #[must_use]
    pub fn default_g_major() -> Self {
        let key = Key::new(Note::natural(mackey_theory::Letter::G), Mode::Major)
            .unwrap_or_else(|_| unreachable!("G major is an offered key"));
        Self {
            key,
            focus: Focus::None,
            chord: None,
            view: ViewType::AllTones,
            voicing_index: 0,
            inversion: Inversion::Root,
            string_set: StringSet::S123,
            show_all_on_set: false,
            position: None,
            overlay: ScaleOverlay::default(),
            labels: LabelsMode::NoteName,
            capo_names: CapoNames::ShapeFirst,
            show_sevenths: false,
            show_order: false,
            arpeggio_direction: ArpeggioDirection::Up,
            options: VoicingOptions::default(),
            note: None,
        }
    }

    /// The scale the overlay draws, if it is on (SC-1, SC-4): the chosen scale; else, when
    /// following the chords, the minor pentatonic of a minor chord's root (SC-1) or the SC-4
    /// choice; else the key's own scale.
    #[must_use]
    pub fn overlay_scale(&self) -> Option<Scale> {
        if !self.overlay.on {
            return None;
        }
        if let Some(scale) = self.overlay.scale {
            return Some(scale);
        }
        if self.overlay.follow_chords
            && let Some(chord) = &self.chord
        {
            use mackey_theory::ChordQuality as Q;
            if matches!(chord.quality, Q::Minor | Q::Minor7) {
                return Some(Scale {
                    tonic: chord.root,
                    mode: Mode::MinorPentatonic,
                });
            }
            return Some(scale_for_chord(&self.key, chord).scale);
        }
        Some(self.key.scale())
    }

    /// True when every index is in range for the instrument and the focus fields agree.
    #[must_use]
    pub fn is_valid(&self, ctx: &Context<'_>) -> bool {
        let focus_ok = match self.focus {
            Focus::Chord => self.chord.is_some() && self.note.is_none(),
            Focus::Note => self.note.is_some() && self.chord.is_none(),
            Focus::None | Focus::Scale => self.chord.is_none() && self.note.is_none(),
        };
        if !focus_ok {
            return false;
        }
        let mut cache = VoicingCache::default();
        if let Some(chord) = &self.chord {
            let len = cache
                .get(chord, ctx.instrument, &self.options)
                .voicings()
                .len();
            if self.voicing_index > len.saturating_sub(1) {
                return false;
            }
            match (self.view, self.position) {
                (ViewType::Triad, Some(p)) => {
                    let n = triads(chord, ctx.instrument, self.string_set, self.inversion)
                        .placements()
                        .len();
                    if usize::from(p) >= n.max(1) {
                        return false;
                    }
                }
                (ViewType::Arpeggio, Some(p)) if p < ctx.instrument.capo.value() || p > 20 => {
                    return false;
                }
                _ => {}
            }
        }
        if let Some(i) = self.overlay.pattern_index {
            match self.overlay_scale() {
                Some(scale) => {
                    if i >= scale_patterns(&scale, ctx.instrument).len() {
                        return false;
                    }
                }
                None => return false,
            }
        }
        if let Some((s, f)) = self.note
            && (f.value() < ctx.instrument.capo.value()
                || f.value() > ctx.instrument.frets.value()
                || s.number() == 0)
        {
            return false;
        }
        true
    }
}

/// What the instrument looks like to cycling (list lengths depend on it).
#[derive(Debug, Clone, Copy)]
pub struct Context<'a> {
    /// Tuning, fret count, capo.
    pub instrument: &'a Instrument,
    /// Left-handed and flip options.
    pub handedness: &'a Handedness,
}

/// A brief notice shown with a change (G-3.4, E-FB-4, FB-6.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum Notice {
    /// The selected voicing no longer exists for the new tuning or capo.
    VoicingChangedForTuning,
    /// The position was below the capo (E-FB-4).
    MovedAboveCapo,
    /// No voicing meets the constraints (E-FB-3).
    NoPlayableVoicing,
    /// The triad cannot be played on that string set (FB-6.4).
    TriadUnavailable,
    /// The note is below the capo (LH-3).
    NoteBelowCapo,
    /// The chord is not in the new key, so it was kept.
    ChordKeptKeyChanged,
}

impl Notice {
    /// The text shown.
    #[must_use]
    pub fn text(self) -> &'static str {
        match self {
            Notice::VoicingChangedForTuning => "Voicing changed for new tuning",
            Notice::MovedAboveCapo => "Moved above the capo",
            Notice::NoPlayableVoicing => "No playable voicing in this tuning",
            Notice::TriadUnavailable => "Not playable on these strings within 4 frets",
            Notice::NoteBelowCapo => "That note is below the capo",
            Notice::ChordKeptKeyChanged => "Chord is not in the new key; kept as it was",
        }
    }
}

/// Everything the explorer can be told to do (FB-9, G-3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum Action {
    /// Pick a key (G-3.3 moves the chord to the same degree).
    SelectKey(Key),
    /// Pick the chord on a degree of the key.
    SelectDiatonic(Degree),
    /// Pick any chord.
    SelectChord(Chord),
    /// Type a Roman numeral; an invalid one changes nothing (FB-4.4).
    SelectRoman(String),
    /// Click a fret position.
    SelectNote {
        /// The string.
        string: StringNumber,
        /// The fret.
        fret: Fret,
    },
    /// Back to the key's notes.
    ClearFocus,
    /// Next chord in the key (`↓`).
    ChordNext,
    /// Previous chord in the key (`↑`).
    ChordPrev,
    /// Next voicing (`→`).
    VoicingNext,
    /// Previous voicing (`←`).
    VoicingPrev,
    /// Next inversion (`I`).
    InversionNext,
    /// Previous inversion (`Shift-I`).
    InversionPrev,
    /// Next string set (`S`).
    StringSetNext,
    /// Previous string set (`Shift-S`).
    StringSetPrev,
    /// Next position (`P`).
    PositionNext,
    /// Previous position (`Shift-P`).
    PositionPrev,
    /// Cycle the view type (`V`).
    CycleView,
    /// Cycle the labels (`L`).
    CycleLabels,
    /// Toggle the scale overlay (`O`).
    ToggleOverlay,
    /// Choose the overlay scale; `None` is the key's own.
    SetOverlayScale(Option<Scale>),
    /// Follow the chords (SC-4).
    SetFollowChords(bool),
    /// Next tonic on the circle of fifths (`]`).
    KeyNext,
    /// Previous tonic on the circle of fifths (`[`).
    KeyPrev,
    /// Relative major or minor (`R`).
    RelativeToggle,
    /// Set the view type.
    SetView(ViewType),
    /// Set the labels mode.
    SetLabels(LabelsMode),
    /// Sevenths on the chord strip.
    SetShowSevenths(bool),
    /// Show all inversions on the string set.
    SetShowAllOnSet(bool),
    /// Show play order.
    SetShowOrder(bool),
    /// Set the arpeggio direction.
    SetArpeggioDirection(ArpeggioDirection),
    /// Allow a 5-fret span.
    SetAllowStretch(bool),
    /// Allow an omitted fifth.
    SetAllowOmittedFifth(bool),
    /// Choose which name leads under a capo.
    SetCapoNames(CapoNames),
}

/// The result of an action: the new selection, notices and the `VoiceOver` text (FB-9.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Applied {
    /// The new selection.
    pub selection: Selection,
    /// Notices to show briefly.
    pub notices: Vec<Notice>,
    /// What `VoiceOver` should say.
    pub announcement: String,
}

/// A one-entry memo of the last voicing search, owned by the caller (the app's explorer state),
/// so cycling does not repeat the search. Not a global: pass it explicitly.
#[derive(Debug, Clone, Default)]
pub struct VoicingCache {
    entry: Option<(Chord, Instrument, VoicingOptions, VoicingResult)>,
}

impl VoicingCache {
    /// A new, empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The voicings for a chord, searching only when the inputs changed.
    pub fn get(
        &mut self,
        chord: &Chord,
        instrument: &Instrument,
        options: &VoicingOptions,
    ) -> &VoicingResult {
        let hit = matches!(&self.entry, Some((c, i, o, _)) if c == chord && i == instrument && o == options);
        if !hit {
            let result = voicings_for(chord, instrument, options);
            self.entry = Some((*chord, instrument.clone(), *options, result));
        }
        match &self.entry {
            Some((_, _, _, result)) => result,
            None => unreachable!("the entry was set above"),
        }
    }
}

/// "F sharp", "B flat", "G".
#[must_use]
pub fn spoken_note(note: Note) -> String {
    let word = match note.accidental.value() {
        -2 => " double flat",
        -1 => " flat",
        1 => " sharp",
        2 => " double sharp",
        _ => "",
    };
    format!("{}{word}", note.letter.as_str())
}

/// "E minor", "C major 7", "D major over F sharp" (`VoiceOver` text).
#[must_use]
pub fn spoken_chord(chord: &Chord) -> String {
    use mackey_theory::ChordQuality as Q;
    let quality = match chord.quality {
        Q::Major => "major",
        Q::Minor => "minor",
        Q::Dominant7 => "dominant 7",
        Q::Major7 => "major 7",
        Q::Minor7 => "minor 7",
        Q::HalfDiminished7 => "half diminished 7",
        Q::Diminished => "diminished",
        Q::Diminished7 => "diminished 7",
        Q::Augmented => "augmented",
        Q::Sus2 => "suspended 2",
        Q::Sus4 => "suspended 4",
        Q::Dominant7Sus4 => "dominant 7 suspended 4",
        Q::Sixth => "6",
        Q::MinorSixth => "minor 6",
        Q::Dominant9 => "dominant 9",
        Q::Major9 => "major 9",
        Q::Minor9 => "minor 9",
        Q::Add9 => "add 9",
        Q::Dominant11 => "dominant 11",
        Q::Dominant13 => "dominant 13",
        Q::Power => "power chord",
    };
    let mut text = format!("{} {quality}", spoken_note(chord.root));
    if let Some(bass) = chord.bass {
        text.push_str(" over ");
        text.push_str(&spoken_note(bass));
    }
    text
}

fn same_chord(a: Chord, b: Chord) -> bool {
    a.root.pitch_class() == b.root.pitch_class() && a.quality == b.quality && a.bass == b.bass
}

fn degree_of_chord(key: Key, chord: Chord) -> Option<Degree> {
    key.diatonic_chords(chord.quality.has_seventh())
        .into_iter()
        .find(|d| same_chord(d.chord, chord))
        .map(|d| d.degree)
}

fn set_chord(s: &mut Selection, chord: Chord) {
    if s.focus != Focus::Chord {
        s.view = ViewType::AllTones;
    }
    s.focus = Focus::Chord;
    s.chord = Some(chord);
    s.note = None;
    s.voicing_index = 0;
    s.position = None;
    s.overlay.pattern_index = None;
}

fn change_key(s: &mut Selection, new_key: Key, notices: &mut Vec<Notice>) {
    let old = s.key;
    if let (Focus::Chord, Some(chord)) = (s.focus, s.chord) {
        match degree_of_chord(old, chord) {
            Some(degree) => {
                let moved = new_key
                    .diatonic_chord(degree, chord.quality.has_seventh())
                    .chord;
                s.chord = Some(moved);
                s.voicing_index = 0;
                s.position = None;
            }
            None => notices.push(Notice::ChordKeptKeyChanged),
        }
    }
    if let Some(scale) = s.overlay.scale
        && scale.tonic.pitch_class() == old.tonic().pitch_class()
    {
        s.overlay.scale = Some(Scale {
            tonic: new_key.tonic(),
            mode: scale.mode,
        });
    }
    s.overlay.pattern_index = None;
    s.key = new_key;
}

fn available_labels(s: &Selection) -> Vec<LabelsMode> {
    let mut list = vec![LabelsMode::NoteName, LabelsMode::ScaleDegree];
    if s.chord.is_some() {
        list.push(LabelsMode::Interval);
        if s.view != ViewType::AllTones {
            list.push(LabelsMode::Finger);
        }
    }
    list.push(LabelsMode::None);
    list
}

fn pattern_count(s: &Selection, ctx: &Context<'_>) -> usize {
    s.overlay_scale()
        .map_or(0, |scale| scale_patterns(&scale, ctx.instrument).len())
}

fn step_pattern(s: &mut Selection, ctx: &Context<'_>, forward: bool) -> bool {
    let n = pattern_count(s, ctx);
    if n == 0 {
        return false;
    }
    s.overlay.pattern_index = match (s.overlay.pattern_index, forward) {
        (None, true) => Some(0),
        (Some(i), true) if i + 1 < n => Some(i + 1),
        (Some(_), true) | (Some(0), false) => None,
        (None, false) => Some(n - 1),
        (Some(i), false) => Some(i - 1),
    };
    true
}

fn step_position(
    s: &mut Selection,
    ctx: &Context<'_>,
    cache: &mut VoicingCache,
    notices: &mut Vec<Notice>,
    forward: bool,
) {
    let Some(chord) = s.chord else {
        step_pattern(s, ctx, forward);
        return;
    };
    match s.view {
        ViewType::Triad => {
            if s.show_all_on_set {
                return;
            }
            let result = triads(&chord, ctx.instrument, s.string_set, s.inversion);
            let n = result.placements().len();
            if n == 0 {
                notices.push(Notice::TriadUnavailable);
                return;
            }
            let current = usize::from(s.position.unwrap_or(0)).min(n - 1);
            let next = if forward {
                (current + 1) % n
            } else {
                (current + n - 1) % n
            };
            s.position = u8::try_from(next).ok();
        }
        ViewType::Arpeggio => {
            let positions = arpeggio_positions(&chord, ctx.instrument);
            if positions.is_empty() {
                return;
            }
            let selected = cache
                .get(&chord, ctx.instrument, &s.options)
                .voicings()
                .get(s.voicing_index)
                .cloned();
            let current = s
                .position
                .or_else(|| default_arpeggio_position(&chord, ctx.instrument, selected.as_ref()))
                .unwrap_or(0);
            let next = if forward {
                positions
                    .iter()
                    .copied()
                    .find(|p| *p > current)
                    .or(positions.first().copied())
            } else {
                positions
                    .iter()
                    .rev()
                    .copied()
                    .find(|p| *p < current)
                    .or(positions.last().copied())
            };
            s.position = next;
        }
        ViewType::Voicing => {
            let list = cache
                .get(&chord, ctx.instrument, &s.options)
                .voicings()
                .to_vec();
            if list.is_empty() {
                notices.push(Notice::NoPlayableVoicing);
                return;
            }
            let current = list
                .get(s.voicing_index)
                .map_or(0, |v| v.lowest_fret.value());
            let pick = |cmp: &dyn Fn(u8) -> bool, best_low: bool| -> Option<usize> {
                let candidates: Vec<(usize, u8)> = list
                    .iter()
                    .enumerate()
                    .map(|(i, v)| (i, v.lowest_fret.value()))
                    .filter(|(_, f)| cmp(*f))
                    .collect();
                let chosen = if best_low {
                    candidates.iter().min_by_key(|(i, f)| (*f, *i))
                } else {
                    candidates
                        .iter()
                        .max_by_key(|(i, f)| (*f, std::cmp::Reverse(*i)))
                };
                chosen.map(|(i, _)| *i)
            };
            let next = if forward {
                pick(&|f| f > current, true).or_else(|| pick(&|_| true, true))
            } else {
                pick(&|f| f < current, false).or_else(|| pick(&|_| true, false))
            };
            if let Some(i) = next {
                s.voicing_index = i;
            }
        }
        ViewType::AllTones => {
            step_pattern(s, ctx, forward);
        }
    }
}

/// Applies an action to a selection (FB-9, G-3). Builds a temporary [`VoicingCache`]; use
/// [`apply_cached`] to reuse searches across calls.
///
/// ```
/// use mackey_theory::{Degree, Key, Mode};
/// use mackey_voicing::{apply, Action, Context, Handedness, Instrument, Selection};
/// let instrument = Instrument::default();
/// let handedness = Handedness::default();
/// let ctx = Context { instrument: &instrument, handedness: &handedness };
/// let s = Selection::default_g_major();
/// let applied = apply(&s, &ctx, Action::SelectDiatonic(Degree::new(4).unwrap()));
/// assert_eq!(applied.selection.chord.unwrap().symbol(), "C");
/// let d = Key::new("D".parse().unwrap(), Mode::Major).unwrap();
/// let applied = apply(&applied.selection, &ctx, Action::SelectKey(d));
/// assert_eq!(applied.selection.chord.unwrap().symbol(), "G"); // IV of D
/// ```
#[must_use]
pub fn apply(sel: &Selection, ctx: &Context<'_>, action: Action) -> Applied {
    apply_cached(sel, ctx, action, &mut VoicingCache::new())
}

/// [`apply`] with a caller-owned cache of the last voicing search.
#[must_use]
#[expect(
    clippy::too_many_lines,
    reason = "one flat dispatch over every FB-9 action reads better than split fragments"
)]
pub fn apply_cached(
    sel: &Selection,
    ctx: &Context<'_>,
    action: Action,
    cache: &mut VoicingCache,
) -> Applied {
    let mut s = sel.clone();
    let mut notices: Vec<Notice> = Vec::new();
    let mut message: Option<String> = None;
    let capo = ctx.instrument.capo;
    match action {
        Action::SelectKey(key) => change_key(&mut s, key, &mut notices),
        Action::KeyNext => {
            let key = s.key.next_fifth();
            change_key(&mut s, key, &mut notices);
        }
        Action::KeyPrev => {
            let key = s.key.prev_fifth();
            change_key(&mut s, key, &mut notices);
        }
        Action::RelativeToggle => match s.key.relative() {
            Some(key) => change_key(&mut s, key, &mut notices),
            None => message = Some(format!("No relative key for {}", s.key.name())),
        },
        Action::SelectDiatonic(degree) => {
            let chord = s.key.diatonic_chord(degree, s.show_sevenths).chord;
            set_chord(&mut s, chord);
        }
        Action::SelectChord(chord) => set_chord(&mut s, chord),
        Action::SelectRoman(text) => match roman_to_chord(&text, &s.key) {
            Ok(chord) => set_chord(&mut s, chord),
            Err(e) => message = Some(e.to_string()),
        },
        Action::SelectNote { string, fret } => {
            if fret.value() < capo.value() {
                notices.push(Notice::NoteBelowCapo);
            } else {
                s.focus = Focus::Note;
                s.chord = None;
                s.note = Some((string, fret));
            }
        }
        Action::ClearFocus => {
            s.focus = Focus::None;
            s.chord = None;
            s.note = None;
            s.view = ViewType::AllTones;
            s.voicing_index = 0;
            s.position = None;
        }
        Action::ChordNext | Action::ChordPrev => {
            let forward = action == Action::ChordNext;
            let list = s.key.diatonic_chords(s.show_sevenths);
            let current = s
                .chord
                .and_then(|c| list.iter().position(|d| same_chord(d.chord, c)));
            let n = list.len();
            let next = match (current, forward) {
                (None, true) => 0,
                (None, false) => n - 1,
                (Some(i), true) => (i + 1) % n,
                (Some(i), false) => (i + n - 1) % n,
            };
            if let Some(d) = list.get(next) {
                set_chord(&mut s, d.chord);
            }
        }
        Action::VoicingNext | Action::VoicingPrev => {
            let forward = action == Action::VoicingNext;
            if let Some(chord) = s.chord {
                if s.view == ViewType::Voicing {
                    let n = cache
                        .get(&chord, ctx.instrument, &s.options)
                        .voicings()
                        .len();
                    if n == 0 {
                        notices.push(Notice::NoPlayableVoicing);
                    } else {
                        let current = s.voicing_index.min(n - 1);
                        s.voicing_index = if forward {
                            (current + 1) % n
                        } else {
                            (current + n - 1) % n
                        };
                    }
                } else {
                    // The first press from another view switches to the voicing list (D-53).
                    s.view = ViewType::Voicing;
                }
            } else {
                message = Some("Select a chord first".to_owned());
            }
        }
        Action::InversionNext | Action::InversionPrev => {
            if let Some(chord) = s.chord {
                let seventh = chord.quality.has_seventh() && s.view != ViewType::Triad;
                let current = s.inversion.clamped(seventh);
                s.inversion = if action == Action::InversionNext {
                    current.next(seventh)
                } else {
                    current.prev(seventh)
                };
                s.position = None;
            } else {
                message = Some("Select a chord first".to_owned());
            }
        }
        Action::StringSetNext => {
            s.string_set = s.string_set.next();
            s.position = None;
        }
        Action::StringSetPrev => {
            s.string_set = s.string_set.prev();
            s.position = None;
        }
        Action::PositionNext => step_position(&mut s, ctx, cache, &mut notices, true),
        Action::PositionPrev => step_position(&mut s, ctx, cache, &mut notices, false),
        Action::CycleView => {
            if s.chord.is_some() {
                s.view = s.view.next();
                s.position = None;
                if s.view == ViewType::AllTones && s.labels == LabelsMode::Finger {
                    s.labels = LabelsMode::NoteName;
                }
            } else {
                message = Some("Select a chord first".to_owned());
            }
        }
        Action::SetView(view) => {
            if s.chord.is_some() || view == ViewType::AllTones {
                s.view = view;
                s.position = None;
            } else {
                message = Some("Select a chord first".to_owned());
            }
        }
        Action::CycleLabels => {
            let list = available_labels(&s);
            let at = list.iter().position(|l| *l == s.labels);
            s.labels = match at {
                Some(i) => list
                    .get((i + 1) % list.len())
                    .copied()
                    .unwrap_or(LabelsMode::NoteName),
                None => LabelsMode::NoteName,
            };
        }
        Action::SetLabels(labels) => {
            if available_labels(&s).contains(&labels) {
                s.labels = labels;
            } else {
                message = Some("Those labels need a chord or a voicing".to_owned());
            }
        }
        Action::ToggleOverlay => {
            s.overlay.on = !s.overlay.on;
            s.overlay.pattern_index = None;
        }
        Action::SetOverlayScale(scale) => {
            s.overlay.scale = scale;
            s.overlay.on = true;
            s.overlay.pattern_index = None;
            if s.focus == Focus::None && scale.is_some() {
                s.focus = Focus::Scale;
            }
        }
        Action::SetFollowChords(on) => {
            s.overlay.follow_chords = on;
            if on {
                s.overlay.on = true;
            }
            s.overlay.pattern_index = None;
        }
        Action::SetShowSevenths(on) => s.show_sevenths = on,
        Action::SetShowAllOnSet(on) => {
            s.show_all_on_set = on;
            s.position = None;
        }
        Action::SetShowOrder(on) => s.show_order = on,
        Action::SetArpeggioDirection(direction) => s.arpeggio_direction = direction,
        Action::SetAllowStretch(on) => {
            s.options.allow_stretch = on;
            s.voicing_index = 0;
        }
        Action::SetAllowOmittedFifth(on) => {
            s.options.allow_omitted_fifth = on;
            s.voicing_index = 0;
        }
        Action::SetCapoNames(names) => s.capo_names = names,
    }
    // A Scale focus needs an overlay to show anything; fall back to the key's notes.
    if s.focus == Focus::Scale && !s.overlay.on {
        s.focus = Focus::None;
    }
    finish(s, ctx, cache, notices, message)
}

fn finish(
    s: Selection,
    ctx: &Context<'_>,
    cache: &mut VoicingCache,
    mut notices: Vec<Notice>,
    message: Option<String>,
) -> Applied {
    if let (Focus::Chord, Some(chord)) = (s.focus, s.chord) {
        match s.view {
            ViewType::Voicing => {
                if cache
                    .get(&chord, ctx.instrument, &s.options)
                    .voicings()
                    .is_empty()
                {
                    notices.push(Notice::NoPlayableVoicing);
                }
            }
            ViewType::Triad => {
                let r = triads(
                    &chord,
                    ctx.instrument,
                    s.string_set,
                    s.inversion.clamped(false),
                );
                if r.placements().is_empty() && !s.show_all_on_set {
                    notices.push(Notice::TriadUnavailable);
                }
            }
            _ => {}
        }
    }
    notices.dedup();
    let mut announcement = message.unwrap_or_else(|| announce(&s, ctx, cache));
    for notice in &notices {
        announcement.push_str(". ");
        announcement.push_str(notice.text());
    }
    Applied {
        selection: s,
        notices,
        announcement,
    }
}

/// The `VoiceOver` text for a selection: "E minor, first inversion, strings 2 3 4" (FB-9.1).
#[must_use]
pub fn announce(s: &Selection, ctx: &Context<'_>, cache: &mut VoicingCache) -> String {
    let overlay = s
        .overlay_scale()
        .map_or_else(String::new, |scale| format!(", {} overlay", scale.name()));
    match (s.focus, s.chord) {
        (Focus::Chord, Some(chord)) => {
            let spoken = spoken_chord(&chord);
            match s.view {
                ViewType::AllTones => format!("{spoken}, all tones{overlay}"),
                ViewType::Voicing => {
                    let result = cache.get(&chord, ctx.instrument, &s.options);
                    let list = result.voicings();
                    match list.get(s.voicing_index) {
                        Some(v) => format!(
                            "{spoken}, {}, {} of {}{overlay}",
                            v.name,
                            s.voicing_index + 1,
                            list.len()
                        ),
                        None => format!("{spoken}, no playable voicing"),
                    }
                }
                ViewType::Triad => {
                    let strings: Vec<String> = s
                        .string_set
                        .strings()
                        .iter()
                        .map(|n| n.number().to_string())
                        .collect();
                    let which = if s.show_all_on_set {
                        "all inversions".to_owned()
                    } else {
                        s.inversion.clamped(false).spoken().to_owned()
                    };
                    format!("{spoken}, {which}, strings {}{overlay}", strings.join(" "))
                }
                ViewType::Arpeggio => {
                    let selected = cache
                        .get(&chord, ctx.instrument, &s.options)
                        .voicings()
                        .get(s.voicing_index)
                        .cloned();
                    let position = s.position.or_else(|| {
                        default_arpeggio_position(&chord, ctx.instrument, selected.as_ref())
                    });
                    match position {
                        Some(p) => format!(
                            "{spoken}, arpeggio {}, position {p}{overlay}",
                            s.arpeggio_direction.label()
                        ),
                        None => format!("{spoken}, arpeggio, no position"),
                    }
                }
            }
        }
        (Focus::Note, _) => match s.note {
            Some((string, fret)) => {
                let pitch = ctx.instrument.pitch_at(string, fret);
                let name = pitch.map_or_else(|_| "note".to_owned(), |p| spoken_note(p.note()));
                if fret.value() == ctx.instrument.capo.value() {
                    format!("{name}, string {}, open", string.number())
                } else {
                    format!("{name}, string {}, fret {}", string.number(), fret.value())
                }
            }
            None => "No note".to_owned(),
        },
        (Focus::Scale, _) => match s.overlay_scale() {
            Some(scale) => format!("{} scale", scale.name()),
            None => format!("{}, all notes", s.key.name()),
        },
        _ => format!("{}, all notes{overlay}", s.key.name()),
    }
}

/// Brings a selection in line with a changed instrument (G-3.4, E-FB-4): the chord stays, the
/// voicing is kept if it still exists and otherwise the best one is chosen with the notice
/// "Voicing changed for new tuning"; a position below a new capo moves up with "Moved above the
/// capo"; indexes are clamped (a smaller fret count leaves no stale index).
///
/// ```
/// use mackey_theory::{Chord, ChordQuality};
/// use mackey_voicing::{apply, instrument_changed, Action, Capo, Context, Handedness, Instrument, Selection, ViewType};
/// let handedness = Handedness::default();
/// let before = Instrument::default();
/// let ctx = Context { instrument: &before, handedness: &handedness };
/// let g = Chord::new("G".parse().unwrap(), ChordQuality::Major);
/// let s = apply(&Selection::default_g_major(), &ctx, Action::SelectChord(g)).selection;
/// let s = apply(&s, &ctx, Action::SetView(ViewType::Arpeggio)).selection;
/// let mut after = before.clone();
/// after.capo = Capo::new(5).unwrap();
/// let moved = instrument_changed(&s, &ctx, &Context { instrument: &after, handedness: &handedness });
/// assert!(moved.announcement.is_empty() || !moved.announcement.is_empty());
/// ```
#[must_use]
pub fn instrument_changed(sel: &Selection, old: &Context<'_>, new: &Context<'_>) -> Applied {
    instrument_changed_cached(sel, old, new, &mut VoicingCache::new())
}

/// [`instrument_changed`] with a caller-owned cache.
#[must_use]
pub fn instrument_changed_cached(
    sel: &Selection,
    old: &Context<'_>,
    new: &Context<'_>,
    cache: &mut VoicingCache,
) -> Applied {
    let mut s = sel.clone();
    let mut notices = Vec::new();
    let capo: Capo = new.instrument.capo;
    if let (Focus::Chord, Some(chord)) = (s.focus, s.chord) {
        let old_voicing = voicings_for(&chord, old.instrument, &s.options)
            .voicings()
            .get(s.voicing_index)
            .cloned();
        let list = cache
            .get(&chord, new.instrument, &s.options)
            .voicings()
            .to_vec();
        let found = old_voicing
            .as_ref()
            .and_then(|o| list.iter().position(|v| v.strings == o.strings));
        match found {
            Some(i) => s.voicing_index = i,
            None => {
                if s.voicing_index >= list.len() || old_voicing.is_some() {
                    s.voicing_index = 0;
                    if s.view == ViewType::Voicing && !list.is_empty() {
                        notices.push(Notice::VoicingChangedForTuning);
                    }
                }
            }
        }
        if list.is_empty() && s.view == ViewType::Voicing {
            notices.push(Notice::NoPlayableVoicing);
        }
        match (s.view, s.position) {
            (ViewType::Arpeggio, Some(p)) => {
                let positions = arpeggio_positions(&chord, new.instrument);
                if p < capo.value() {
                    s.position = positions.first().copied();
                    notices.push(Notice::MovedAboveCapo);
                } else if let Some(&last) = positions.last() {
                    s.position = Some(p.min(last));
                }
            }
            (ViewType::Triad, Some(p)) => {
                let n = triads(
                    &chord,
                    new.instrument,
                    s.string_set,
                    s.inversion.clamped(false),
                )
                .placements()
                .len();
                s.position = if n == 0 {
                    None
                } else {
                    Some(p.min(u8::try_from(n - 1).unwrap_or(0)))
                };
            }
            _ => {}
        }
    }
    if let Some((_, fret)) = s.note
        && (fret.value() < capo.value() || fret.value() > new.instrument.frets.value())
    {
        s.note = None;
        s.focus = Focus::None;
        notices.push(Notice::NoteBelowCapo);
    }
    if let Some(i) = s.overlay.pattern_index {
        let n = pattern_count(&s, new);
        if i >= n {
            s.overlay.pattern_index = None;
        }
    }
    notices.dedup();
    let mut announcement = announce(&s, new, cache);
    for notice in &notices {
        announcement.push_str(". ");
        announcement.push_str(notice.text());
    }
    Applied {
        selection: s,
        notices,
        announcement,
    }
}
