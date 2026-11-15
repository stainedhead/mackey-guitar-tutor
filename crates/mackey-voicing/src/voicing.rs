//! Voicing types (FB-5): what is played on each string, how it is fingered and how it is
//! grouped and named.

use mackey_theory::Chord;
use serde::{Deserialize, Serialize};

use crate::instrument::{Capo, Fret, Inversion, StringNumber, StringSet};

/// What one string does in a voicing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StringState {
    /// Not played.
    Muted,
    /// Rings open (at the capo fret when a capo is on).
    Open,
    /// Pressed at an absolute fret above the capo.
    Fretted {
        /// The absolute fret.
        fret: Fret,
    },
}

impl StringState {
    /// True when the string sounds.
    #[must_use]
    pub fn is_sounding(self) -> bool {
        !matches!(self, StringState::Muted)
    }

    /// The absolute fret of a pressed string.
    #[must_use]
    pub fn fretted(self) -> Option<u8> {
        match self {
            StringState::Fretted { fret } => Some(fret.value()),
            _ => None,
        }
    }
}

/// A barre: one finger across several strings at one fret (FB-5.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Barre {
    /// The absolute fret.
    pub fret: Fret,
    /// The lower-pitched end (higher string number).
    pub from: StringNumber,
    /// The higher-pitched end (lower string number).
    pub to: StringNumber,
}

/// Finger numbers and barres for a voicing.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Fingering {
    /// Per string, low to high: `Some(1..=4)` pressed, `Some(0)` open, `None` muted.
    pub fingers: [Option<u8>; 6],
    /// Barres, drawn as a bar across the strings they cover.
    pub barres: Vec<Barre>,
}

/// The five CAGED forms.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum CagedShape {
    /// C form.
    C,
    /// A form.
    A,
    /// G form.
    G,
    /// E form.
    E,
    /// D form.
    D,
}

impl CagedShape {
    /// The letter.
    #[must_use]
    pub fn letter(self) -> &'static str {
        match self {
            CagedShape::C => "C",
            CagedShape::A => "A",
            CagedShape::G => "G",
            CagedShape::E => "E",
            CagedShape::D => "D",
        }
    }
}

/// How a voicing came to be and what it looks like.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "type")]
pub enum VoicingKind {
    /// An open-position shape.
    Open,
    /// A movable CAGED form.
    Caged {
        /// The form.
        shape: CagedShape,
    },
    /// A movable barre that matches no CAGED name.
    Barre,
    /// A triad on three adjacent strings.
    Triad {
        /// The string set.
        set: StringSet,
        /// The inversion.
        inversion: Inversion,
    },
    /// Everything else.
    Other,
}

/// The list groups of FB-5.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum VoicingGroup {
    /// Uses at least one open string within frets 0 to 4 (relative to the capo).
    Open,
    /// Movable barre and CAGED forms.
    BarreAndCaged,
    /// Everything else.
    Other,
}

impl VoicingGroup {
    /// The heading the list shows.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            VoicingGroup::Open => "Open",
            VoicingGroup::BarreAndCaged => "Barre and CAGED",
            VoicingGroup::Other => "Other",
        }
    }
}

/// Where a voicing came from (LH-4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum Origin {
    /// From the curated shape library.
    Curated,
    /// Found by the search.
    Search,
}

/// One way to play a chord (or triad) on the instrument.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Voicing {
    /// The sounding chord.
    pub chord: Chord,
    /// One state per string, low to high.
    pub strings: [StringState; 6],
    /// Finger numbers and barres.
    pub fingering: Fingering,
    /// What kind of voicing it is.
    pub kind: VoicingKind,
    /// The list group.
    pub group: VoicingGroup,
    /// Curated or searched.
    pub origin: Origin,
    /// "Open C", "E-shape barre, fret 3", "Found by search".
    pub name: String,
    /// The lowest fret of the diagram: the capo for open voicings, else the lowest pressed fret.
    pub lowest_fret: Fret,
    /// Highest minus lowest pressed fret.
    pub span: u8,
    /// Higher is better (internal; serialised for debugging).
    pub score: i32,
}

impl Voicing {
    /// The fret string used in tests and docs: `x32010`. Open strings print 0 and pressed
    /// strings print the fret relative to the capo; frets of 10 or more print in brackets.
    ///
    /// ```
    /// use mackey_theory::{Chord, ChordQuality};
    /// use mackey_voicing::{voicings_for, Capo, Instrument, VoicingOptions, VoicingResult};
    /// let c = Chord::new("C".parse().unwrap(), ChordQuality::Major);
    /// let VoicingResult::Found(list) = voicings_for(&c, &Instrument::default(), &VoicingOptions::default())
    ///     else { panic!("C has voicings") };
    /// assert_eq!(list[0].fret_string(Capo::default()), "x32010");
    /// ```
    #[must_use]
    pub fn fret_string(&self, capo: Capo) -> String {
        self.strings
            .iter()
            .map(|s| match s {
                StringState::Muted => "x".to_owned(),
                StringState::Open => "0".to_owned(),
                StringState::Fretted { fret } => {
                    let rel = fret.value().saturating_sub(capo.value());
                    if rel >= 10 {
                        format!("({rel})")
                    } else {
                        rel.to_string()
                    }
                }
            })
            .collect()
    }

    /// The number of sounding strings.
    #[must_use]
    pub fn sounding_count(&self) -> usize {
        self.strings.iter().filter(|s| s.is_sounding()).count()
    }
}

/// Options for the voicing search (FB-5.4, FB-5.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct VoicingOptions {
    /// Allow a span of 5 frets instead of 4.
    pub allow_stretch: bool,
    /// Allow dropping the fifth even where the chord's identity needs it.
    pub allow_omitted_fifth: bool,
    /// The most voicings returned.
    pub max_results: usize,
}

impl Default for VoicingOptions {
    fn default() -> Self {
        Self {
            allow_stretch: false,
            allow_omitted_fifth: false,
            max_results: 24,
        }
    }
}

/// A constraint the player may relax when nothing fits (E-FB-3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum RelaxOption {
    /// "Allow stretches".
    AllowStretches,
    /// "Allow omitted fifth".
    AllowOmittedFifth,
}

impl RelaxOption {
    /// The button label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            RelaxOption::AllowStretches => "Allow stretches",
            RelaxOption::AllowOmittedFifth => "Allow omitted fifth",
        }
    }
}

/// The outcome of a voicing search (AC-FB-5.6).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "kind", content = "value")]
pub enum VoicingResult {
    /// Voicings ordered Open, Barre and CAGED, Other; each group best first.
    Found(Vec<Voicing>),
    /// Nothing met FB-5.4; these relaxations may help.
    NoneFound {
        /// The options not yet switched on.
        options: Vec<RelaxOption>,
    },
}

impl VoicingResult {
    /// The voicings, or an empty slice.
    #[must_use]
    pub fn voicings(&self) -> &[Voicing] {
        match self {
            VoicingResult::Found(list) => list,
            VoicingResult::NoneFound { .. } => &[],
        }
    }

    /// "No playable voicing in this tuning" text when nothing was found.
    #[must_use]
    pub fn none_message(&self) -> Option<&'static str> {
        match self {
            VoicingResult::NoneFound { .. } => Some("No playable voicing in this tuning"),
            VoicingResult::Found(_) => None,
        }
    }
}

/// The outcome of a triad request (FB-6.4).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
#[serde(tag = "kind", content = "value")]
pub enum TriadResult {
    /// Placements in fret order, lowest first.
    Placements(Vec<Voicing>),
    /// No placement within a 4-fret span on that string set.
    Unavailable {
        /// Why, in plain words.
        reason: String,
    },
}

impl TriadResult {
    /// The placements, or an empty slice.
    #[must_use]
    pub fn placements(&self) -> &[Voicing] {
        match self {
            TriadResult::Placements(list) => list,
            TriadResult::Unavailable { .. } => &[],
        }
    }
}
