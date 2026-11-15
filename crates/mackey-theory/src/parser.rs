//! The chord-symbol parser: a pure function from `&str` to a chord plus positioned diagnostics.
//!
//! Grammar (FB-4.3): `symbol := root quality? ("/" bass)?` with `root := [A-G] ("#"|"b"|"♯"|"♭")?`.
//! Symbols are case-sensitive where case matters (`M7` is major seventh, `m7` minor). Leading
//! and trailing whitespace is trimmed; whitespace inside a symbol is an error. The player's root
//! spelling is kept (`A#m` stays `A#m`).

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::chord::{Chord, ChordQuality};
use crate::pitch::{Accidental, Letter, Note};

/// A byte range into the parsed text (`end` exclusive), always on `char` boundaries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Span {
    /// First byte.
    pub start: usize,
    /// One past the last byte.
    pub end: usize,
}

impl Span {
    /// A span from byte offsets.
    #[must_use]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// As a `Range` for slicing.
    #[must_use]
    pub fn range(self) -> std::ops::Range<usize> {
        self.start..self.end
    }
}

/// How serious a diagnostic is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum Severity {
    /// The input cannot be used.
    Error,
    /// The input was accepted but something is worth saying.
    Warning,
}

/// A positioned message the UI can underline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Diagnostic {
    /// Where in the input.
    pub span: Span,
    /// Error or warning.
    pub severity: Severity,
    /// Text for the player (E-FB-1 wording).
    pub message: String,
    /// A full replacement symbol, when one is close enough to offer.
    pub suggestion: Option<String>,
}

/// Parser settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct ParseOptions {
    /// Read the German `H` as `B` (default off).
    pub read_h_as_b: bool,
}

/// The result of [`parse_chord`]: the chord when the whole input parsed, plus every diagnostic.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct ParseOutcome {
    /// `Some` when the whole input parsed (warnings may still accompany it).
    pub chord: Option<Chord>,
    /// All errors and warnings, in input order.
    pub diagnostics: Vec<Diagnostic>,
}

/// Why a symbol was refused. `symbol` is the whole trimmed input and `suggestion` a full
/// replacement symbol.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ChordParseError {
    /// Nothing but whitespace.
    #[error("Type a chord symbol such as Am7.")]
    Empty,
    /// The first character is not a note name.
    #[error("{}", unknown_message(symbol, suggestion.as_deref()))]
    UnknownRoot {
        /// The offending text.
        span: Span,
        /// The whole symbol.
        symbol: String,
        /// The text at `span`.
        found: String,
        /// A corrected symbol (`Hm7` -> `Bm7`).
        suggestion: Option<String>,
    },
    /// The text after the root is not a chord quality.
    #[error("{}", unknown_message(symbol, suggestion.as_deref()))]
    UnknownQuality {
        /// The offending text.
        span: Span,
        /// The whole symbol.
        symbol: String,
        /// The text at `span`.
        found: String,
        /// A corrected symbol (`Cmaj77` -> `Cmaj7`).
        suggestion: Option<String>,
    },
    /// The text after `/` is not a note name.
    #[error("{}", bass_message(symbol, found))]
    InvalidBass {
        /// The offending text.
        span: Span,
        /// The whole symbol.
        symbol: String,
        /// The text at `span`.
        found: String,
    },
    /// A valid chord followed by text that cannot belong to it (spaces, punctuation).
    #[error("Not a chord symbol I know: {symbol}. Unexpected text after the chord.")]
    TrailingInput {
        /// The unexpected text.
        span: Span,
        /// The whole symbol.
        symbol: String,
    },
}

fn unknown_message(symbol: &str, suggestion: Option<&str>) -> String {
    match suggestion {
        Some(s) => format!("Not a chord symbol I know: {symbol}. Did you mean {s}?"),
        None => format!("Not a chord symbol I know: {symbol}."),
    }
}

fn bass_message(symbol: &str, found: &str) -> String {
    if found.is_empty() {
        format!("Not a chord symbol I know: {symbol}. A bass note is missing after the slash.")
    } else {
        format!("Not a chord symbol I know: {symbol}. {found:?} is not a bass note.")
    }
}

impl ChordParseError {
    /// Where the problem is (the whole input for [`ChordParseError::Empty`] is `0..0`).
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            ChordParseError::Empty => Span::new(0, 0),
            ChordParseError::UnknownRoot { span, .. }
            | ChordParseError::UnknownQuality { span, .. }
            | ChordParseError::InvalidBass { span, .. }
            | ChordParseError::TrailingInput { span, .. } => *span,
        }
    }

    /// The suggested replacement symbol, if any.
    #[must_use]
    pub fn suggestion(&self) -> Option<&str> {
        match self {
            ChordParseError::UnknownRoot { suggestion, .. }
            | ChordParseError::UnknownQuality { suggestion, .. } => suggestion.as_deref(),
            _ => None,
        }
    }

    fn to_diagnostic(&self) -> Diagnostic {
        Diagnostic {
            span: self.span(),
            severity: Severity::Error,
            message: self.to_string(),
            suggestion: self.suggestion().map(str::to_owned),
        }
    }
}

/// Accepted spellings of each quality. The empty string is plain major.
const ALIASES: &[(&str, ChordQuality)] = &[
    ("", ChordQuality::Major),
    ("maj", ChordQuality::Major),
    ("M", ChordQuality::Major),
    ("Maj", ChordQuality::Major),
    ("m", ChordQuality::Minor),
    ("min", ChordQuality::Minor),
    ("-", ChordQuality::Minor),
    ("mi", ChordQuality::Minor),
    ("7", ChordQuality::Dominant7),
    ("maj7", ChordQuality::Major7),
    ("M7", ChordQuality::Major7),
    ("Maj7", ChordQuality::Major7),
    ("Δ", ChordQuality::Major7),
    ("Δ7", ChordQuality::Major7),
    ("△", ChordQuality::Major7),
    ("△7", ChordQuality::Major7),
    ("m7", ChordQuality::Minor7),
    ("min7", ChordQuality::Minor7),
    ("-7", ChordQuality::Minor7),
    ("m7b5", ChordQuality::HalfDiminished7),
    ("ø", ChordQuality::HalfDiminished7),
    ("ø7", ChordQuality::HalfDiminished7),
    ("min7b5", ChordQuality::HalfDiminished7),
    ("-7b5", ChordQuality::HalfDiminished7),
    ("dim", ChordQuality::Diminished),
    ("°", ChordQuality::Diminished),
    ("o", ChordQuality::Diminished),
    ("dim7", ChordQuality::Diminished7),
    ("°7", ChordQuality::Diminished7),
    ("o7", ChordQuality::Diminished7),
    ("aug", ChordQuality::Augmented),
    ("+", ChordQuality::Augmented),
    ("+5", ChordQuality::Augmented),
    ("sus2", ChordQuality::Sus2),
    ("sus4", ChordQuality::Sus4),
    ("sus", ChordQuality::Sus4),
    ("7sus4", ChordQuality::Dominant7Sus4),
    ("7sus", ChordQuality::Dominant7Sus4),
    ("6", ChordQuality::Sixth),
    ("maj6", ChordQuality::Sixth),
    ("m6", ChordQuality::MinorSixth),
    ("min6", ChordQuality::MinorSixth),
    ("9", ChordQuality::Dominant9),
    ("maj9", ChordQuality::Major9),
    ("M9", ChordQuality::Major9),
    ("Δ9", ChordQuality::Major9),
    ("m9", ChordQuality::Minor9),
    ("min9", ChordQuality::Minor9),
    ("add9", ChordQuality::Add9),
    ("add2", ChordQuality::Add9),
    ("11", ChordQuality::Dominant11),
    ("13", ChordQuality::Dominant13),
    ("5", ChordQuality::Power),
];

/// Unicode accidentals are matched as their ASCII twins.
fn norm(c: char) -> char {
    match c {
        '♭' => 'b',
        '♯' => '#',
        other => other,
    }
}

fn is_accidental_char(c: char) -> bool {
    matches!(c, '#' | '♯' | 'b' | '♭')
}

fn accidental_of(c: char) -> Accidental {
    if matches!(c, '#' | '♯') {
        Accidental::SHARP
    } else {
        Accidental::FLAT
    }
}

/// Characters that can be part of a quality (as opposed to stray punctuation or spaces).
fn is_quality_char(c: char) -> bool {
    c.is_alphanumeric() || "#♯♭+-°øΔ△()".contains(c)
}

/// Longest text the edit-distance rows hold; callers keep both sides well under this.
const ROW: usize = 24;

/// Edit distance between two short texts (under `ROW` chars each, which the only caller
/// guarantees), on stack rows: no allocation, since this runs for every unknown quality inside
/// the parser's time budget.
fn levenshtein(a: &[char], b: &[char]) -> usize {
    let mut prev = [0usize; ROW];
    let mut cur = [0usize; ROW];
    for (j, slot) in prev.iter_mut().enumerate().take(b.len() + 1) {
        *slot = j;
    }
    for (i, ca) in a.iter().enumerate() {
        if let Some(first) = cur.first_mut() {
            *first = i + 1;
        }
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            let diagonal = prev.get(j).copied().unwrap_or(0) + cost;
            let above = prev.get(j + 1).copied().unwrap_or(0) + 1;
            let left = cur.get(j).copied().unwrap_or(0) + 1;
            if let Some(slot) = cur.get_mut(j + 1) {
                *slot = diagonal.min(above).min(left);
            }
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev.get(b.len()).copied().unwrap_or(usize::MAX)
}

/// The closest known quality spelling to an unknown token, if one is near enough (distance
/// at most 2 and at most half the token). Tokens that look like extensions or alterations
/// (`maj13#11`, `7b9`) get no suggestion: they are valid music this grammar does not cover.
fn suggest_quality(token: &[char]) -> Option<&'static str> {
    let looks_altered = token.iter().enumerate().any(|(i, c)| {
        matches!(c, '#' | '♯' | '♭')
            || (*c == 'b' && i > 0 && token.get(i - 1).is_some_and(char::is_ascii_digit))
    });
    if looks_altered || token.is_empty() || token.len() > 16 {
        return None;
    }
    let max_distance = (token.len() / 2).min(2);
    let normalised: Vec<char> = token.iter().copied().map(norm).collect();
    ALIASES
        .iter()
        .enumerate()
        .filter(|(_, (alias, _))| !alias.is_empty())
        .filter_map(|(index, (alias, _))| {
            let candidate_len = alias.chars().count();
            if candidate_len.abs_diff(normalised.len()) > max_distance {
                return None;
            }
            let candidate: Vec<char> = alias.chars().collect();
            let distance = levenshtein(&normalised, &candidate);
            if distance > max_distance {
                return None;
            }
            let common_prefix = normalised
                .iter()
                .zip(&candidate)
                .take_while(|(a, b)| a == b)
                .count();
            // Lower is better: distance, then more shared prefix, then the longer spelling.
            Some((
                (
                    distance,
                    usize::MAX - common_prefix,
                    usize::MAX - candidate.len(),
                    index,
                ),
                *alias,
            ))
        })
        .min_by_key(|(rank, _)| *rank)
        .map(|(_, alias)| alias)
}

struct Collected {
    chord: Option<Chord>,
    errors: Vec<ChordParseError>,
    warnings: Vec<Diagnostic>,
}

/// Parses a root or bass note at the start of `text`. Returns the note (when valid), the byte
/// length consumed and whether an `H` was read as `B`.
fn parse_note(text: &str, opts: ParseOptions) -> (Option<Note>, usize, bool) {
    let mut chars = text.chars();
    let Some(first) = chars.next() else {
        return (None, 0, false);
    };
    let mut used = first.len_utf8();
    let (letter, h_as_b) = match first {
        'A' => (Some(Letter::A), false),
        'B' => (Some(Letter::B), false),
        'C' => (Some(Letter::C), false),
        'D' => (Some(Letter::D), false),
        'E' => (Some(Letter::E), false),
        'F' => (Some(Letter::F), false),
        'G' => (Some(Letter::G), false),
        'H' if opts.read_h_as_b => (Some(Letter::B), true),
        _ => (None, false),
    };
    if letter.is_none() && first != 'H' {
        return (None, used, false);
    }
    let mut accidental = Accidental::NATURAL;
    if let Some(c) = chars.next().filter(|c| is_accidental_char(*c)) {
        accidental = accidental_of(c);
        used += c.len_utf8();
    }
    (
        letter.map(|letter| Note { letter, accidental }),
        used,
        h_as_b,
    )
}

/// Where the pieces of one symbol are, so errors can carry absolute spans.
struct Ctx {
    symbol: String,
    offset: usize,
    options: ParseOptions,
}

impl Ctx {
    fn span(&self, start: usize, end: usize) -> Span {
        Span::new(self.offset + start, self.offset + end)
    }

    fn h_warning(&self, at: usize) -> Diagnostic {
        Diagnostic {
            span: self.span(at, at + 1),
            severity: Severity::Warning,
            message: "H is read as B".to_owned(),
            suggestion: None,
        }
    }
}

/// The root. On failure the error is recorded and parsing goes on, so later problems are
/// reported too.
fn parse_root(ctx: &Ctx, text: &str, main: &str, out: &mut Collected) -> (Option<Note>, usize) {
    let (root, root_len, h_as_b) = parse_note(main, ctx.options);
    if root.is_some() {
        if h_as_b {
            out.warnings.push(ctx.h_warning(0));
        }
        return (root, root_len);
    }
    let found = main.get(..root_len).unwrap_or_default().to_owned();
    let after = found
        .chars()
        .next()
        .map_or("", |c| found.get(c.len_utf8()..).unwrap_or_default());
    let fixed_letter = match found.chars().next() {
        Some('H') => Some('B'),
        Some(c) if c.is_ascii_lowercase() && c <= 'g' => Some(c.to_ascii_uppercase()),
        _ => None,
    };
    let rest = main.get(root_len..).unwrap_or_default();
    let tail = text.get(main.len()..).unwrap_or_default();
    let suggestion = fixed_letter.map(|letter| format!("{letter}{after}{rest}{tail}"));
    out.errors.push(ChordParseError::UnknownRoot {
        span: ctx.span(0, root_len),
        symbol: ctx.symbol.clone(),
        found,
        suggestion,
    });
    (None, root_len)
}

/// The quality token between the root and the slash.
fn parse_quality(
    ctx: &Ctx,
    text: &str,
    main: &str,
    root_len: usize,
    root_ok: bool,
    out: &mut Collected,
) -> Option<ChordQuality> {
    let token = main.get(root_len..).unwrap_or_default();
    let normalised: String = token.chars().map(norm).collect();
    if let Some((_, quality)) = ALIASES.iter().find(|(alias, _)| *alias == normalised) {
        return Some(*quality);
    }
    let prefix = ALIASES
        .iter()
        .filter(|(alias, _)| !alias.is_empty() && normalised.starts_with(alias))
        .map(|(alias, _)| alias.chars().count())
        .max()
        .unwrap_or(0);
    let rest_at = token
        .char_indices()
        .nth(prefix)
        .map_or(token.len(), |(i, _)| i);
    let rest = token.get(rest_at..).unwrap_or_default();
    if rest.chars().next().is_some_and(is_quality_char) {
        let chars: Vec<char> = token.chars().collect();
        // A corrected quality is no use while the root itself is wrong.
        let suggestion = suggest_quality(&chars).filter(|_| root_ok).map(|alias| {
            let head = text.get(..root_len).unwrap_or_default();
            let tail = text.get(main.len()..).unwrap_or_default();
            format!("{head}{alias}{tail}")
        });
        out.errors.push(ChordParseError::UnknownQuality {
            span: ctx.span(root_len, main.len()),
            symbol: ctx.symbol.clone(),
            found: token.to_owned(),
            suggestion,
        });
    } else {
        out.errors.push(ChordParseError::TrailingInput {
            span: ctx.span(root_len + rest_at, main.len()),
            symbol: ctx.symbol.clone(),
        });
    }
    None
}

/// The text after the slash: exactly one note name.
fn parse_bass(ctx: &Ctx, bass_text: &str, slash: usize, out: &mut Collected) -> Option<Note> {
    let (note, used, h_as_b) = parse_note(bass_text, ctx.options);
    if let Some(note) = note.filter(|_| used == bass_text.len()) {
        if h_as_b {
            out.warnings.push(ctx.h_warning(slash + 1));
        }
        return Some(note);
    }
    let (start, end) = if bass_text.is_empty() {
        (slash, slash + 1)
    } else {
        (slash + 1, slash + 1 + bass_text.len())
    };
    out.errors.push(ChordParseError::InvalidBass {
        span: ctx.span(start, end),
        symbol: ctx.symbol.clone(),
        found: bass_text.to_owned(),
    });
    None
}

fn parse_inner(input: &str, options: ParseOptions) -> Collected {
    let mut out = Collected {
        chord: None,
        errors: Vec::new(),
        warnings: Vec::new(),
    };
    let text = input.trim();
    if text.is_empty() {
        out.errors.push(ChordParseError::Empty);
        return out;
    }
    let ctx = Ctx {
        symbol: text.to_owned(),
        offset: input.len() - input.trim_start().len(),
        options,
    };
    // Split at the first slash: the quality never contains one.
    let slash = text.find('/');
    let main = slash.and_then(|i| text.get(..i)).unwrap_or(text);

    let (root, root_len) = parse_root(&ctx, text, main, &mut out);
    let quality = parse_quality(&ctx, text, main, root_len, root.is_some(), &mut out);
    let mut bass = None;
    let mut bass_ok = true;
    if let Some(slash) = slash {
        bass = parse_bass(
            &ctx,
            text.get(slash + 1..).unwrap_or_default(),
            slash,
            &mut out,
        );
        bass_ok = bass.is_some();
    }
    if let (Some(root), Some(quality), true) = (root, quality, bass_ok) {
        out.chord = Some(Chord {
            root,
            quality,
            bass,
        });
    }
    out
}

/// Parses a chord symbol and reports every problem with its position.
///
/// The chord is `Some` only when the whole input parsed. Errors carry E-FB-1 wording and, when
/// a close match exists, a full replacement symbol.
///
/// ```
/// use mackey_theory::{parse_chord, ParseOptions};
/// let ok = parse_chord("Am7", ParseOptions::default());
/// assert_eq!(ok.chord.unwrap().symbol(), "Am7");
/// let bad = parse_chord("Hm7", ParseOptions::default());
/// assert!(bad.chord.is_none());
/// assert_eq!(bad.diagnostics[0].suggestion.as_deref(), Some("Bm7"));
/// assert_eq!(
///     bad.diagnostics[0].message,
///     "Not a chord symbol I know: Hm7. Did you mean Bm7?"
/// );
/// ```
#[must_use]
pub fn parse_chord(input: &str, options: ParseOptions) -> ParseOutcome {
    let collected = parse_inner(input, options);
    let mut diagnostics: Vec<Diagnostic> = collected
        .errors
        .iter()
        .map(ChordParseError::to_diagnostic)
        .collect();
    diagnostics.extend(collected.warnings);
    diagnostics.sort_by_key(|d| (d.span.start, d.span.end));
    ParseOutcome {
        chord: collected.chord,
        diagnostics,
    }
}

/// The strict form: the chord, or the first error.
///
/// ```
/// use mackey_theory::{parse_chord_symbol, ChordParseError};
/// assert_eq!(parse_chord_symbol("D/F#").unwrap().symbol(), "D/F#");
/// assert_eq!(parse_chord_symbol("  "), Err(ChordParseError::Empty));
/// ```
///
/// # Errors
/// The first [`ChordParseError`] found, in input order.
pub fn parse_chord_symbol(input: &str) -> Result<Chord, ChordParseError> {
    let collected = parse_inner(input, ParseOptions::default());
    match collected.chord {
        Some(chord) => Ok(chord),
        None => Err(collected
            .errors
            .into_iter()
            .next()
            .unwrap_or(ChordParseError::Empty)),
    }
}

impl std::str::FromStr for Chord {
    type Err = ChordParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_chord_symbol(s)
    }
}
