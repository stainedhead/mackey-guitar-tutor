# Data Dictionary: Fretboard Foundation

**Feature:** fretboard-foundation
**Created:** 2026-11-15
**Spec:** [spec.md](spec.md) · **Architecture:** [architecture.md](architecture.md)

Concrete Rust types for this run. Signatures are the contract; field names and variants may be refined during implementation if the change is recorded in [implementation-notes.md](implementation-notes.md) and the TypeScript bindings are regenerated. Rules for all types (AGENTS.md): strong types over primitives, invalid states unrepresentable (validate in `TryFrom`/`FromStr`/`new`), public items documented, no panics, one `thiserror` error enum per crate or module.

**Legend (Derives column).** `S` = `serde::Serialize + Deserialize`. `TS` = `ts_rs::TS` behind the crate feature `ts` (`#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]`), generated into `ui/src/bindings/`. Every type also derives `Clone, Debug, PartialEq`; `Eq, Hash` where fields allow (no `f32`/`f64`). "Internal" types do not cross the IPC boundary.

## Conventions

- **String number** `1..=6`, 1 = highest-pitched. **Index** into arrays is low to high: `index = 6 - number`.
- **Fret** is absolute from the nut, `0..=24`. With a capo at `c`, a string that is not fretted rings from the capo (`StringState::Open`), and frets below `c` are unavailable.
- **Pitch classes** are `0..=11`, C = 0. **MIDI** C-1 = 0, A4 = 69, C4 = 60.
- Frequencies are `f64` in Hz; cents are `f32`.
- Spans (`start..end`) are byte offsets into the input `&str`, `end` exclusive, always on `char` boundaries.

## 1. `mackey-theory`

### 1.1 Notes and pitches

| Type | Definition | Validation and notes | Derives |
|------|------------|----------------------|---------|
| `PitchClass` | `struct PitchClass(u8)` | `0..=11`. `PitchClass::new(i32)` reduces modulo 12 (infallible); `transpose(self, semitones: i32) -> PitchClass`; `semitones_to(self, other) -> u8` (0..=11 up). | S, TS (as `number`) |
| `Letter` | `enum Letter { C, D, E, F, G, A, B }` | `natural_pc() -> PitchClass`; `step(self, n: i8) -> Letter` (wraps); `index() -> u8` (C = 0). | S, TS, Copy, Eq, Hash, Ord |
| `Accidental` | `struct Accidental(i8)` | `-2..=2` (double flat to double sharp); `Accidental::try_from(i8)`. `symbol(self) -> &'static str` returns `"bb" "b" "" "#" "x"`-style display (`x` never shown as a label, see G-1.6). | S, TS (`number`), Copy, Eq, Hash |
| `Note` | `struct Note { pub letter: Letter, pub accidental: Accidental }` | Spelled: `F#` and `Gb` are different values. `pitch_class() -> PitchClass`; `name() -> String` ("F#", "Bb"); `FromStr` ("F#", "Bb", "C##" accepted, "H" rejected); `transpose_semitones(self, n, prefer: AccidentalPreference) -> Note`; `simplified(self, pref: AccidentalPreference) -> Note` (G-1.6: removes double accidentals, keeps pitch class); `is_double(self) -> bool`. | S, TS, Copy, Eq, Hash, Ord |
| `AccidentalPreference` | `enum AccidentalPreference { Sharps, Flats }` | Used when no key or chord context spells a pitch (G-1.5). | S, TS, Copy |
| `Pitch` | `struct Pitch { pub note: Note, pub octave: i8 }` | Scientific pitch; C4 = middle C. `from_midi(i32, AccidentalPreference) -> Result<Pitch, TheoryError>` (0..=127); `midi() -> i32` (`12 * (octave + 1) + pc`, careful with Cb and B#); `frequency(self, a4: ReferencePitch) -> f64`; `add_semitones(self, n: i32, pref) -> Result<Pitch, TheoryError>`; `name() -> String` ("G3"). | S, TS, Copy, Eq, Hash, Ord (by MIDI then note) |
| `ReferencePitch` | `struct ReferencePitch(f64)` | `415.0..=466.0` Hz, step 1 Hz in settings; `ReferencePitch::new(f64) -> Result<_, TheoryError>`; `Default` = 440.0. | S, TS, Copy |

### 1.2 Intervals

| Type | Definition | Notes | Derives |
|------|------------|-------|---------|
| `IntervalQuality` | `enum IntervalQuality { Perfect, Major, Minor, Augmented, Diminished }` | Validity depends on the number: 1, 4, 5, 8 (and 11, 12, 15) are perfect-class. | S, TS, Copy, Eq, Hash |
| `Interval` | `struct Interval { pub quality: IntervalQuality, pub number: u8 }` | `number` `1..=15` (compound intervals for 9, 11, 13). `Interval::new(q, n) -> Result<Interval, TheoryError>` rejects "major fifth", "perfect third", number 0 or above 15. `semitones() -> u8`; `short_name() -> String` ("M3", "P5", "b7" style via `degree_label()`: "R", "b3", "5", "b7"); `apply(self, Note) -> Note` (letter steps by number - 1, accidental fixed by semitone count); `between(Note, Note) -> Interval` (upward, ascending within an octave; `None`-free: result always valid for spelled notes up to double accidentals); `invert()`. Consts: `Interval::P1, M2, m3, M3, P4, P5, m6, M6, m7, M7, M9, ...`. | S, TS, Copy, Eq, Hash |

### 1.3 Scales, modes, keys

| Type | Definition | Notes | Derives |
|------|------------|-------|---------|
| `Mode` | `enum Mode { Major, NaturalMinor, HarmonicMinor, Dorian, Phrygian, Lydian, Mixolydian, Locrian, MajorPentatonic, MinorPentatonic, Blues }` | `intervals() -> &'static [Interval]` from the tonic (Major: P1 M2 M3 P4 P5 M6 M7; Blues: P1 m3 P4 d5 P5 m7); `display_name() -> &'static str` ("major", "natural minor", "E Dorian" built by `Scale::name`); `is_diatonic_mode()` (the 7 heptatonic modes incl. harmonic minor excluded); `parent_major_offset() -> Option<u8>` (semitones from the mode's tonic down to its parent major tonic: Dorian 2, Phrygian 4, Lydian 5, Mixolydian 7, Aeolian 9, Locrian 11, Ionian 0); `relative(self) -> Option<Mode>` (Major ↔ NaturalMinor). | S, TS, Copy, Eq, Hash |
| `Scale` | `struct Scale { pub tonic: Note, pub mode: Mode }` | `degrees() -> Vec<Note>` spelled by letter stepping; `contains(PitchClass) -> bool`; `degree_of(PitchClass) -> Option<u8>` (1-based); `name() -> String` ("G major"). | S, TS, Eq, Hash |
| `KeyChoice` | `struct KeyChoice { pub tonic: Note, pub mode: Mode }` | Alias semantics for the picker; same shape as `Key`. | internal |
| `Key` | `struct Key { tonic: Note, mode: Mode }` (private fields) | Constructed with `Key::new(tonic, mode) -> Result<Key, KeyError>`: the tonic must be one of the offered tonics for the mode (G-1.2, G-1.3) after enharmonic substitution (G-1.4): `Key::new_normalised(...)` applies the substitution (D# Dorian → Eb Dorian) and returns the key plus a `KeyNotice`. `OFFERED_MAJOR_TONICS`, `OFFERED_MINOR_TONICS` consts. `scale() -> Scale`; `signature() -> KeySignature { accidentals: i8 }` (positive = sharps) from the parent major key; `preference() -> AccidentalPreference` (G-1.5 with no chord context: flats for flat keys and C major, sharps for sharp keys); `diatonic_chord(degree: Degree, seventh: bool) -> Chord`; `diatonic_chords(seventh: bool) -> Vec<DiatonicChord>`; `next_fifth() / prev_fifth()` (clockwise / counter-clockwise tonic on the circle for the same mode, honouring the F#/Gb choice already in the key); `relative() -> Option<Key>` (`R` toggles Major ↔ Natural minor only: A minor ↔ C major; `None` for every other mode, D-26); `enharmonic_alternative() -> Option<Key>` (for keys with 6 accidentals: F# ↔ Gb major, D#/Eb minor); `degree_of(Note or PitchClass) -> Option<Degree>`; `name() -> String` ("G major", "E Dorian"). | S (as `{tonic, mode}`), TS |
| `KeySignature` | `struct KeySignature { pub accidentals: i8 }` | `-7..=7`; `count()`. | S, TS |
| `Degree` | `struct Degree(u8)` | `1..=7`; `Degree::new(u8) -> Result<_, TheoryError>`; `index()`. | S, TS, Copy |
| `DiatonicChord` | `struct DiatonicChord { pub degree: Degree, pub roman: RomanNumeral, pub chord: Chord }` | One chip of the chord strip. | S, TS |

### 1.4 Chords

| Type | Definition | Notes | Derives |
|------|------------|-------|---------|
| `ChordQuality` | `enum ChordQuality { Major, Minor, Dominant7, Major7, Minor7, HalfDiminished7, Diminished, Diminished7, Augmented, Sus2, Sus4, Dominant7Sus4, Sixth, MinorSixth, Dominant9, Major9, Minor9, Add9, Dominant11, Dominant13, Power }` | The grammar of FB-4.3. `intervals() -> &'static [Interval]` (the chord formula from the root: `Dominant13` = P1 M3 P5 m7 M9 M13, 11th omitted by convention; `Dominant11` = P1 P5 m7 M9 P11 with the third kept: P1 M3 P5 m7 M9 P11, the third is required by FB-5.4); `symbol() -> &'static str` (canonical display: "", "m", "7", "maj7", "m7", "m7b5", "dim", "dim7", "aug", "sus2", "sus4", "7sus4", "6", "m6", "9", "maj9", "m9", "add9", "11", "13", "5"); `required_tones() -> &'static [ToneRole]` for FB-5.4 (root + third; root + sus note; root + fifth for `Power`); `has_seventh() -> bool`; `roles() -> &'static [ToneRole]` aligned with `intervals()`. | S, TS, Copy, Eq, Hash |
| `ToneRole` | `enum ToneRole { Root, Third, Fifth, Seventh, Extension }` | The sus note, 6th, 9th, 11th and 13th are `Extension` (G-2). Shared with voicing's `NoteRole` mapping. | S, TS, Copy, Eq, Hash |
| `Chord` | `struct Chord { pub root: Note, pub quality: ChordQuality, pub bass: Option<Note> }` | Root keeps the player's spelling (`A#m` stays `A#m`, G-1.7). `tones() -> Vec<ChordTone>` spelled by interval from the root (D7 → D F# A C, G-1.5); `symbol() -> String` ("Am7", "D/F#"); `bass_or_root()`; `pitch_classes() -> PitchClassSet`; `transpose(semitones, pref) -> Chord`. | S, TS, Eq, Hash |
| `ChordTone` | `struct ChordTone { pub note: Note, pub interval: Interval, pub role: ToneRole }` | One entry of `Chord::tones`. | S, TS |
| `PitchClassSet` | `struct PitchClassSet(u16)` | Bitset of 12 pitch classes; `contains`, `insert`, `len`, `iter`. Internal helper for the search hot path. | Copy, Eq, Hash (internal) |
| `ParseOptions` | `struct ParseOptions { pub read_h_as_b: bool }` | Default `false` (Songs setting "Read H as B" lands later; the chord field uses the default). | S, TS, Default |
| `ParseOutcome` | `struct ParseOutcome { pub chord: Option<Chord>, pub diagnostics: Vec<Diagnostic> }` | The parser reports diagnostics, not just the first error (AGENTS.md). `chord` is `Some` when the whole input parsed; warnings may accompany it. | S, TS |
| `Diagnostic` | `struct Diagnostic { pub span: Range<usize>, pub severity: Severity, pub message: String, pub suggestion: Option<String> }` | `span` is serialised as `{ start, end }`. | S, TS |
| `Severity` | `enum Severity { Error, Warning }` | | S, TS, Copy |
| `ChordParseError` | `enum ChordParseError { Empty, UnknownRoot { span, found: String, suggestion: Option<String> }, UnknownQuality { span, found: String, suggestion: Option<String> }, InvalidBass { span, found: String }, TrailingInput { span } }` (`thiserror`) | `parse_chord_symbol(&str) -> Result<Chord, ChordParseError>` is the strict form; `parse_chord(&str, ParseOptions) -> ParseOutcome` the diagnostic form. `Display` text follows E-FB-1: "Not a chord symbol I know: Hm7. Did you mean Bm7?". `suggestion` is a full replacement symbol. | Debug, PartialEq, Eq; `S + TS` on a mirror `ChordParseErrorView` or via `ParseOutcome` only |

Parser grammar (informal, for task A7): `symbol := root quality? ("/" bassnote)?`; `root := [A-G] ("#" | "b" | "♯" | "♭")?`; quality aliases (case-sensitive where the case matters): major: `""`, `maj`, `M`, `Maj`; minor: `m`, `min`, `-`, `mi`; `7`; `maj7`, `M7`, `Maj7`, `Δ`, `Δ7`, `△7`; `m7`, `min7`, `-7`; `m7b5`, `ø`, `ø7`, `min7b5`, `-7b5`; `dim`, `°`, `o`; `dim7`, `°7`, `o7`; `aug`, `+`, `+5`; `sus2`; `sus4`, `sus`; `7sus4`, `7sus`; `6`, `maj6`; `m6`, `min6`; `9`; `maj9`, `M9`, `Δ9`; `m9`, `min9`; `add9`, `add2`; `11`; `13`; `5`. Whitespace around the symbol is trimmed; internal whitespace is an error. Unicode `♯ ♭` accepted for accidentals.

### 1.5 Roman numerals and analysis

| Type | Definition | Notes | Derives |
|------|------------|-------|---------|
| `RomanNumeral` | `struct RomanNumeral { pub degree: Degree, pub accidental: Accidental, pub case: NumeralCase, pub suffix: RomanSuffix }` | `accidental` ∈ {-1, 0, 1} (`bVII`, `#iv`). `Display`: "I", "ii", "vii°", "viiø7", "bVII", "V7", "IVmaj7". `FromStr` returns `RomanParseError`. | S, TS, Eq, Hash |
| `NumeralCase` | `enum NumeralCase { Upper, Lower }` | Upper = major/augmented, lower = minor/diminished. | S, TS, Copy |
| `RomanSuffix` | `enum RomanSuffix { None, Dim, HalfDim, Aug, Seventh, Maj7, Dim7 }` | | S, TS, Copy |
| `RomanParseError` | `enum RomanParseError { Empty, NotANumeral { span }, DegreeOutOfRange { span }, UnsupportedSuffix { span, found } }` (`thiserror`) | `roman_to_chord(s: &str, key: &Key) -> Result<Chord, RomanParseError>`; `vi` in G → Em; `V7/V` is accepted as secondary dominant. | Debug, PartialEq |
| `ChordAnalysis` | `struct ChordAnalysis { pub in_key: bool, pub roman: Option<RomanNumeral>, pub function: Option<ChordFunction> }` | `analyze(key: &Key, chord: &Chord) -> ChordAnalysis`. `badge() -> Option<String>`: `None` when in key; "Not in key (V/V)"; "Not in key (bVII, borrowed from G minor)"; "Not in key" when the function is unknown. | S, TS |
| `ChordFunction` | `enum ChordFunction { SecondaryDominant { target: RomanNumeral }, Borrowed { from: Mode, roman: RomanNumeral }, Unknown }` | Secondary dominants: dominant seventh (or major triad) on the fifth above a diatonic degree. Borrowed: diatonic to the parallel minor or major (`bVII`, `bVI`, `iv`, `bIII`). | S, TS |

### 1.6 Errors

`TheoryError` (`thiserror`): `PitchClassOutOfRange(i32)`, `InvalidInterval { quality, number }`, `MidiOutOfRange(i32)`, `ReferencePitchOutOfRange(f64)`, `InvalidDegree(u8)`, `Key(KeyError)` (`#[from]`). `KeyError`: `TonicNotOffered { tonic: Note, mode: Mode }`, `UnsupportedMode`.

## 2. `mackey-voicing`

### 2.1 Instrument

| Type | Definition | Notes | Derives |
|------|------------|-------|---------|
| `StringNumber` | `struct StringNumber(u8)` | `1..=6`; `index() -> usize` (6 - n); `from_index`. | S, TS (`number`), Copy, Eq, Hash, Ord |
| `Fret` | `struct Fret(u8)` | `0..=24`; arithmetic helpers return `Option<Fret>`. | S, TS (`number`), Copy, Eq, Hash, Ord |
| `Tuning` | `struct Tuning { pub name: String, pub strings: [Pitch; 6] }` | Low to high. `strings` must be ascending or equal in MIDI order? Not required (open tunings are not always ascending): only each pitch must be within MIDI 24..=84 (C1..C6). `Tuning::preset(TuningPreset) -> Tuning`; `Tuning::custom(strings: [Pitch; 6]) -> (Tuning, Vec<TuningWarning>)`; `shift_from_standard() -> Option<i8>` (uniform offset such as -1 for Half-step down, used for curated-shape reuse); `open_pitch(StringNumber) -> Pitch`; `is_standard()`. | S, TS, Eq, Hash |
| `TuningPreset` | `enum TuningPreset { Standard, DropD, DoubleDropD, Dadgad, OpenG, OpenD, OpenE, HalfStepDown, WholeStepDown }` | The nine of LH-2. `display_name()`, `strings()` (e.g. Open D: D2 A2 D3 F#3 A3 D4). | S, TS, Copy, Eq, Hash |
| `TuningWarning` | `enum TuningWarning { UnusualTension { string: StringNumber, standard: Pitch, chosen: Pitch } }` | Outside "a minor third below to a whole tone above" the standard pitch for that string (LH-2). Message "Unusual tension, check your strings". | S, TS |
| `Capo` | `struct Capo(u8)` | `0..=12`. | S, TS (`number`), Copy, Eq, Hash |
| `FretCount` | `struct FretCount(u8)` | `19..=24`, default 22. | S, TS (`number`), Copy, Eq, Hash |
| `Instrument` | `struct Instrument { pub tuning: Tuning, pub frets: FretCount, pub capo: Capo }` | `pitch_at(StringNumber, Fret) -> Pitch` (absolute fret; error if below the capo); `open_sounding_pitch(StringNumber) -> Pitch` (open string at the capo); `lowest_fret() -> Fret` (= capo). | S, TS, Eq, Hash |
| `Handedness` | `struct Handedness { pub left_handed: bool, pub flip_strings: bool }` | LH-1; independent flags. | S, TS, Default (both false) |
| `StringSet` | `enum StringSet { S123, S234, S345, S456 }` | `strings() -> [StringNumber; 3]` highest first (123 = strings 1, 2, 3); `next/prev` wrap; label "123". | S, TS, Copy, Eq, Hash |
| `Inversion` | `enum Inversion { Root, First, Second, Third }` | `Third` only for chords with a seventh. `next(has_seventh)/prev` wrap. Label "root", "1st", "2nd", "3rd". | S, TS, Copy, Eq, Hash |

### 2.2 Voicings

| Type | Definition | Notes | Derives |
|------|------------|-------|---------|
| `StringState` | `enum StringState { Muted, Open, Fretted(Fret) }` | `Open` rings at the capo fret. Serialised as `{ "kind": "muted" }` etc. | S, TS, Copy, Eq, Hash |
| `Barre` | `struct Barre { pub fret: Fret, pub from: StringNumber, pub to: StringNumber }` | `from` is the higher-numbered (lower-pitched) string; covers every string between, inclusive. | S, TS, Copy, Eq, Hash |
| `Fingering` | `struct Fingering { pub fingers: [Option<u8>; 6], pub barres: Vec<Barre> }` | `fingers[index]`: `Some(1..=4)` fretted, `Some(0)` open, `None` muted. Strings that are identical open strings (E-FB-6) are never counted twice. | S, TS, Eq, Hash |
| `CagedShape` | `enum CagedShape { C, A, G, E, D }` | | S, TS, Copy, Eq, Hash |
| `VoicingKind` | `enum VoicingKind { Open, Caged { shape: CagedShape }, Barre, Triad { set: StringSet, inversion: Inversion }, Arpeggio { start_fret: Fret }, Other }` | `Barre` is a movable barre not matching a CAGED name. | S, TS, Eq, Hash |
| `VoicingGroup` | `enum VoicingGroup { Open, BarreAndCaged, Other }` | FB-5.2. Open = uses at least one open string within frets 0..=4 (relative to the capo). | S, TS, Copy, Eq, Hash |
| `Origin` | `enum Origin { Curated, Search }` | `Search` shows "Found by search" in non-curated tunings (LH-4). | S, TS, Copy, Eq, Hash |
| `Voicing` | `struct Voicing { pub chord: Chord, pub strings: [StringState; 6], pub fingering: Fingering, pub kind: VoicingKind, pub group: VoicingGroup, pub origin: Origin, pub name: String, pub lowest_fret: Fret, pub span: u8, pub score: i32 }` | `strings` low to high. `name` e.g. "Open C", "E-shape barre, fret 3", "Found by search". `lowest_fret` is the lowest fretted fret (open voicings 0). `span` is highest minus lowest fretted fret. `score`: higher is better (internal, serialised for debugging). `shape_name`/`sounding_name` computed in the view for capo labels. `fret_string()` → "x32010" for tests (open strings print as 0, relative to the capo). | S, TS, Eq, Hash |
| `VoicingOptions` | `struct VoicingOptions { pub allow_stretch: bool, pub allow_omitted_fifth: bool, pub max_results: usize }` | `allow_stretch` = span 5. Defaults: false, false, 24. | S, TS |
| `VoicingResult` | `enum VoicingResult { Found(Vec<Voicing>), NoneFound { options: Vec<RelaxOption> } }` | AC-FB-5.6. | S, TS |
| `RelaxOption` | `enum RelaxOption { AllowStretches, AllowOmittedFifth }` | UI labels: "Allow stretches", "Allow omitted fifth". | S, TS, Copy |
| `TriadResult` | `enum TriadResult { Placements(Vec<Voicing>), Unavailable { reason: String } }` | FB-6.4 ("not playable within a 4-fret span on that string set"). | S, TS |

### 2.3 Scale patterns

| Type | Definition | Notes | Derives |
|------|------------|-------|---------|
| `PatternKind` | `enum PatternKind { FullNeck, PentatonicBox { number: u8 }, ThreeNps { number: u8 } }` | `PentatonicBox` 1..=5; `ThreeNps` 1..=7 (7-note modes only). | S, TS, Eq, Hash |
| `PatternNote` | `struct PatternNote { pub string: StringNumber, pub fret: Fret, pub degree: Degree }` | `degree` is the 1-based degree of the pattern's scale (SC-5). | S, TS, Eq, Hash |
| `ScalePattern` | `struct ScalePattern { pub scale: Scale, pub kind: PatternKind, pub notes: Vec<PatternNote>, pub lowest_fret: Fret, pub highest_fret: Fret }` | `scale_patterns(scale, instrument) -> Vec<ScalePattern>` returns all patterns of the applicable kinds in ascending lowest-fret order, starting at the first one at or above the capo (cycling order). | S, TS, Eq |
| `FollowChoice` | `struct FollowChoice { pub scale: Scale, pub reason: FollowReason }` | `scale_for_chord(key, chord) -> FollowChoice` (SC-4, AC-SC-4.1). | S, TS |
| `FollowReason` | `enum FollowReason { Diatonic, DominantMixolydian, MajorIonian, MinorDorian, DiminishedLocrian, Fallback }` | `Fallback` for qualities SC-4 does not cover (sus, power, augmented): the key's own scale. | S, TS, Copy |

### 2.4 Selection

| Type | Definition | Notes | Derives |
|------|------------|-------|---------|
| `Focus` | `enum Focus { None, Chord, Scale, Note }` | G-3.1. `Note` focus holds a single selected pitch class and string/fret in `Selection::note`. | S, TS, Copy, Eq |
| `ViewType` | `enum ViewType { AllTones, Voicing, Triad, Arpeggio }` | `next()` wraps in FB-9 order. | S, TS, Copy, Eq |
| `LabelsMode` | `enum LabelsMode { NoteName, ScaleDegree, Interval, Finger, None }` | G-2.2; `Finger` only when a voicing is shown (otherwise skipped when cycling). | S, TS, Copy, Eq |
| `CapoNames` | `enum CapoNames { ShapeFirst, SoundingFirst }` | LH-5 toggle. | S, TS, Copy, Eq |
| `ArpeggioDirection` | `enum ArpeggioDirection { Up, Down, UpDown }` | FB-7.3. | S, TS, Copy, Eq |
| `ScaleOverlay` | `struct ScaleOverlay { pub on: bool, pub scale: Option<Scale>, pub follow_chords: bool, pub pattern_index: Option<usize> }` | `scale: None` means "the key's own scale" (or the chord root's minor pentatonic for minor chords in follow mode, SC-1). `pattern_index: None` = full neck. | S, TS, Eq |
| `Selection` | `struct Selection { pub key: Key, pub focus: Focus, pub chord: Option<Chord>, pub view: ViewType, pub voicing_index: usize, pub inversion: Inversion, pub string_set: StringSet, pub show_all_on_set: bool, pub position: Option<u8>, pub overlay: ScaleOverlay, pub labels: LabelsMode, pub capo_names: CapoNames, pub show_sevenths: bool, pub show_order: bool, pub arpeggio_direction: ArpeggioDirection, pub options: VoicingOptions, pub note: Option<(StringNumber, Fret)> }` | The one current selection (G-3.1). `position` is the start fret for the arpeggio view, the placement index for triads, or `None` to use the default rule (FB-7.4). `Selection::default_g_major()` = G-4 defaults (G major, focus None, labels NoteName). | S, TS, Eq |
| `Action` | `enum Action { SelectKey(Key), SelectDiatonic(Degree), SelectChord(Chord), SelectRoman(String), SelectNote { string, fret }, ClearFocus, ChordNext, ChordPrev, VoicingNext, VoicingPrev, InversionNext, InversionPrev, StringSetNext, StringSetPrev, PositionNext, PositionPrev, CycleView, CycleLabels, ToggleOverlay, SetOverlayScale(Option<Scale>), SetFollowChords(bool), KeyNext, KeyPrev, RelativeToggle, SetView(ViewType), SetLabels(LabelsMode), SetShowSevenths(bool), SetShowAllOnSet(bool), SetShowOrder(bool), SetArpeggioDirection(ArpeggioDirection), SetAllowStretch(bool), SetAllowOmittedFifth(bool), SetCapoNames(CapoNames) }` | `apply(sel: &Selection, ctx: &Context<'_>, action: Action) -> Applied`. `CycleAction` (the subset keyed by shortcuts: the `*Next/*Prev`, `CycleView`, `CycleLabels`, `ToggleOverlay`, `KeyNext/KeyPrev`, `RelativeToggle`) is the type the UI sends for `cycle`. | S, TS |
| `Context` | `struct Context<'a> { pub instrument: &'a Instrument, pub handedness: &'a Handedness }` | Cycling needs list lengths, which depend on the instrument. Internal. | Internal |
| `Applied` | `struct Applied { pub selection: Selection, pub notices: Vec<Notice>, pub announcement: String }` | `announcement` is the VoiceOver text ("E minor, first inversion, strings 2 3 4"). | S, TS |
| `Notice` | `enum Notice { VoicingChangedForTuning, MovedAboveCapo, NoPlayableVoicing, TriadUnavailable, NoteBelowCapo, ChordKeptKeyChanged }` | `text() -> &'static str`: "Voicing changed for new tuning", "Moved above the capo", ... | S, TS, Copy |

### 2.5 `FretboardView`

| Type | Definition | Notes | Derives |
|------|------------|-------|---------|
| `ViewRequest` | `struct ViewRequest { pub visible: Option<FretRange>, pub a4: ReferencePitch }` | `fretboard_view(sel: &Selection, instrument: &Instrument, handedness: &Handedness, req: &ViewRequest) -> FretboardView`. `visible: None` = whole neck (no off-range counts). | S, TS |
| `FretRange` | `struct FretRange { pub from: Fret, pub to: Fret }` | `from <= to`. `FretRange::new` validates. | S, TS, Copy, Eq |
| `NoteRole` | `enum NoteRole { Root, Third, Fifth, Seventh, Extension, ScaleTone, SingleNote }` | G-2 mapping; UI decides shape and colour token. | S, TS, Copy, Eq, Hash |
| `PositionLabels` | `struct PositionLabels { pub note_name: String, pub scale_degree: String, pub interval: Option<String>, pub finger: Option<String> }` | One precomputed string per labels mode (G-2.2). `interval` is `None` without a chord; `finger` is `None` without a voicing. Notes are display-spelled (no double accidentals, G-1.6). | S, TS |
| `Tooltip` | `struct Tooltip { pub note_octave: String, pub degree: String, pub interval: Option<String>, pub role: String, pub theory_spelling: Option<String> }` | FB-1.6. AC-FB-1.6: `"G3"`, `"1"`, `"R"`, `"Root"`. `theory_spelling` carries the double-accidental spelling when the label was simplified. | S, TS |
| `ViewPosition` | `struct ViewPosition { pub string: StringNumber, pub fret: Fret, pub is_open: bool, pub note: Note, pub octave: i8, pub role: NoteRole, pub labels: PositionLabels, pub finger: Option<u8>, pub order: Option<u8>, pub tooltip: Tooltip, pub a11y_label: String, pub playable: bool }` | `is_open` = the string rings at the capo fret and is drawn as a ring above the nut (G-2). `order` is the play-order number when "Show order" is on. `a11y_label` e.g. "String 3, fret 2, A, third of F" or "String 3, open, G, root of G". `playable` false for positions below the capo (never emitted as highlighted; see `dimmed_frets`). | S, TS |
| `StringMarker` | `struct StringMarker { pub string: StringNumber, pub kind: MarkerKind }` | Muted "×" above the nut. | S, TS |
| `MarkerKind` | `enum MarkerKind { Muted }` | Open strings are positions with `is_open`. | S, TS |
| `StringLabel` | `struct StringLabel { pub string: StringNumber, pub note: Note, pub octave: i8 }` | Open-string note for the current tuning and capo (FB-1.3). | S, TS |
| `Inlay` | `struct Inlay { pub fret: Fret, pub double: bool }` | Frets 3, 5, 7, 9, 15, 17, 19, 21 single; 12, 24 double (FB-1.1); only up to the fret count. | S, TS |
| `OffRange` | `struct OffRange { pub below: u16, pub above: u16 }` | Highlighted positions left and right of the visible range (E-FB-5). | S, TS |
| `ListPosition` | `struct ListPosition { pub label: String, pub index: usize, pub count: usize }` | "3 of 7" (FB-9.2); `index` is 1-based. `None` when no list applies. | S, TS |
| `VoicingListEntry` | `struct VoicingListEntry { pub group: VoicingGroup, pub name: String, pub lowest_fret: Fret, pub strings: [StringState; 6], pub fingers: [Option<u8>; 6], pub barres: Vec<Barre>, pub found_by_search: bool, pub selected: bool, pub capo_label: Option<CapoLabel> }` | Mini diagram data for the voicing list (FB-5.2). | S, TS |
| `CapoLabel` | `struct CapoLabel { pub shape: String, pub sounding: String, pub text: String }` | `text` = "G shape (sounds A)" (shape first) or "A (G shape)" when `CapoNames::SoundingFirst`. AC-LH-5.1. | S, TS |
| `FretboardView` | `struct FretboardView { pub fret_count: u8, pub capo: u8, pub mirrored: bool, pub flipped: bool, pub strings: Vec<StringLabel>, pub inlays: Vec<Inlay>, pub fret_numbers: Vec<u8>, pub dimmed_below: u8, pub positions: Vec<ViewPosition>, pub markers: Vec<StringMarker>, pub barres: Vec<Barre>, pub ghost: Vec<ViewPosition>, pub list_position: Option<ListPosition>, pub notices: Vec<Notice>, pub badge: Option<String>, pub headline: String, pub chord_label: Option<CapoLabel>, pub off_range: OffRange, pub fit_range: FretRange, pub voicing_list: Vec<VoicingListEntry>, pub no_voicing: Option<VoicingResult>, pub triad_note: Option<String>, pub announcement: String, pub labels_mode: LabelsMode, pub key_label: String }` | The one value the UI draws from (G-3.2). `headline` is a short text such as "C major, open C" for the panel and live region. `ghost` is empty in this run (run mode is P2) but the field exists so later phases do not break bindings. `fret_numbers` per FB-1.2 (3, 5, 7, 9, 12, 15, 17, 19, 21, 24 and the first visible fret, the last only when `visible` was given). `mirrored`/`flipped` echo handedness so the SVG can transform (AC-LH-1.1 checks the SVG, not the data). | S, TS |

### 2.6 Playback plan

| Type | Definition | Notes | Derives |
|------|------------|-------|---------|
| `PlanKind` | `enum PlanKind { Strum, Arpeggio, Scale, SingleNote }` | FB-8.1, FB-1.7. | S, TS, Copy |
| `PlannedNote` | `struct PlannedNote { pub pitch: Pitch, pub delay_ms: u32, pub duration_ms: u32 }` | Strum: delays 0, 30, 60, ... low to high. Arpeggio: eighth notes at the tempo (70 bpm → 428.57 ms, rounded). Scale: ascending from the lowest tonic in the visible range, eighths at 70 bpm. | S, TS |
| `PlaybackPlan` | `struct PlaybackPlan { pub kind: PlanKind, pub notes: Vec<PlannedNote> }` | `playback_plan(sel, instrument, visible: Option<FretRange>, tempo: Bpm) -> PlaybackPlan`; empty when there is nothing to play (never an error). | S, TS |
| `Bpm` | `struct Bpm(u16)` | `30..=250`; default 70. | S, TS, Copy |

### 2.7 Errors

`VoicingError` (`thiserror`): `StringOutOfRange(u8)`, `FretOutOfRange(u8)`, `BelowCapo { fret: Fret, capo: Capo }`, `FretCountOutOfRange(u8)`, `CapoOutOfRange(u8)`, `Theory(TheoryError)` (`#[from]`), `InvalidRange`, `NoteDoesNotFit`. Search and view functions that cannot fail return plain values (an empty list or `VoicingResult::NoneFound`), not errors (FB-5.6).

## 3. `mackey-audio`

### 3.1 Tuner (pure `dsp`)

| Type | Definition | Notes | Derives |
|------|------------|-------|---------|
| `DetectorConfig` | `struct DetectorConfig { pub sample_rate: u32, pub window: usize, pub hop: usize, pub min_hz: f32, pub max_hz: f32, pub clarity_threshold: f32, pub k: f32 }` | Defaults: 48,000 (or the device rate), 2048, 512, 60.0, 1400.0, 0.80, 0.93 (MPM's peak-pick constant). `DetectorConfig::new(sample_rate)` validates: sample rate 22,050 Hz or more (so the 2,048-sample window holds at least two periods of 60 Hz) and `hop <= window`. | S, TS, Copy |
| `PitchEstimate` | `struct PitchEstimate { pub frequency_hz: f64, pub clarity: f32 }` | `PitchDetector::detect(&mut self, window: &[f32]) -> Option<PitchEstimate>`: `None` below the clarity threshold or outside the range. The detector owns its scratch buffers (allocated in `new`). | Copy |
| `PitchDetector` | `struct PitchDetector { .. }` | `new(cfg) -> Result<_, AudioError>`; `config()`. Not `Send`-restricted. Internal. | Internal |
| `LevelMeter` | `struct LevelMeter { .. }` | `process(&mut self, block: &[f32]) -> Level`. | Internal |
| `Level` | `struct Level { pub rms: f32, pub peak: f32, pub clipping: bool }` | `0.0..=1.0` linear; `clipping` when `peak >= 0.99` for at least 2 consecutive samples. | S, TS, Copy |
| `CentsOffset` | `struct CentsOffset(f32)` | `cents_from(freq: f64, target: Pitch, a4: ReferencePitch) -> CentsOffset`; `nearest_pitch(freq, a4) -> (Pitch, CentsOffset)` (−50..=+50, nearest equal-tempered pitch, spelled with `AccidentalPreference::Sharps`). | S, TS, Copy |
| `TunerTarget` | `struct TunerTarget { pub string: u8, pub pitch: Pitch, pub frequency_hz: f64 }` | Built by the app from the instrument (`open_sounding_pitch` per string) so `mackey-audio` needs only theory (D-08). | S, TS |
| `TunerMode` | `enum TunerMode { Chromatic, Strings { locked: Option<u8> } }` | | S, TS, Copy |
| `TunerConfig` | `struct TunerConfig { pub a4: ReferencePitch, pub mode: TunerMode, pub targets: Vec<TunerTarget> }` | `Tuner::set_config`. | S, TS |
| `TunerStatus` | `enum TunerStatus { Listening, Pitch, NoStablePitch, TooQuiet, TooLoud, LowConfidence }` | `NoStablePitch` after 500 ms without a pitch ("Play a single string", TU-1.4); `TooQuiet` after 3 s of low level (E-TU-3); `TooLoud` when clipping (E-TU-4); `LowConfidence` with two strings ringing (E-TU-6, shows the TU-1.4 message). | S, TS, Copy, Eq |
| `TuneDirection` | `enum TuneDirection { InTune, Tighten, Loosen }` | Tighten = pitch is flat. | S, TS, Copy, Eq |
| `TunerReading` | `struct TunerReading { pub status: TunerStatus, pub note: Option<Note>, pub octave: Option<i8>, pub cents: Option<f32>, pub frequency_hz: Option<f64>, pub confidence: f32, pub level: Level, pub in_tune: bool, pub direction: Option<TuneDirection>, pub target: Option<TargetReading>, pub far_from_any: bool, pub ticks: Vec<u8>, pub notice: Option<TunerNotice>, pub t_ms: f64 }` | Emitted as the Tauri event `tuner_reading` at about 30 Hz. `in_tune` = within ±3 cents held for 300 ms (TU-1.1). `ticks` = strings that have been in tune this session (TU-2.2). `t_ms` = stream time from the sample counter. | S, TS |
| `TargetReading` | `struct TargetReading { pub string: u8, pub cents: f32 }` | The auto-selected (or locked) string and the offset from it. `far_from_any` when the pitch is more than 3 semitones from every target (TU-2.1). | S, TS |
| `TunerNotice` | `enum TunerNotice { DeviceDisconnectedUsingDefault, DeviceChanged, AnalysisStopped }` | E-TU-5. `AnalysisStopped` is emitted once if the analysis thread panics (caught with `catch_unwind`); the tuner screen shows "The tuner stopped unexpectedly. Close and reopen it." | S, TS, Copy |
| `Tuner` | `struct Tuner { .. }` | Pure state machine: `new(cfg: TunerConfig, detector: PitchDetector)`, `push_block(&mut self, block: &[f32]) -> Vec<TunerReading>` (hop-sized windows; sample counters, never wall-clock time, so tests are deterministic, D-13), `lock(Option<u8>)`, `reset_ticks()`. Median-of-3 smoothing on cents. | Internal |

### 3.2 Synth and playback (pure `dsp` plus glue)

| Type | Definition | Notes | Derives |
|------|------------|-------|---------|
| `SynthCommand` | `enum SynthCommand { NoteOn { id: u32, frequency_hz: f32, velocity: f32, delay_samples: u32 }, NoteOff { id: u32 }, StopAll, SetVolume(f32), SetMute(bool), ReferenceToneOn { id: u32, frequency_hz: f32 }, ReferenceToneOff }` | `Copy`, fixed size, sent over `rtrb` (never contains heap data). `delay_samples` is relative to the callback block start at which the command is consumed; strums are sent as one `NoteOn` per string with increasing delays. `ReferenceToneOn` loops (re-plucks every 1.5 s) for 3 s or until `ReferenceToneOff` (TU-4). | Copy, Debug, PartialEq (internal) |
| `PluckedVoice` | `struct PluckedVoice { .. }` | Karplus-Strong string: preallocated delay line, damping and a one-pole lowpass loop filter, pick-position comb, decay by `velocity`; `fn start(&mut self, freq, velocity, sr)`; `fn next(&mut self) -> f32`; `fn is_active()`. Internal. | Internal |
| `Synth` | `struct Synth { .. }` | `Synth::new(sample_rate: u32, voices: usize)` allocates everything up front (default 16 voices, delay lines sized for 55 Hz); `apply(&mut self, cmd: SynthCommand)`; `render(&mut self, out: &mut [f32])` is allocation-free and lock-free (`#![deny]` via `assert_no_alloc` test). Mono; the output glue duplicates to all channels. Steals the oldest voice when full. | Internal |
| `AudioError` | `enum AudioError { NoOutputDevice, NoInputDevice, DeviceUnavailable(String), UnsupportedConfig(String), StreamBuild(String), StreamPlay(String), InvalidConfig(String), PermissionDenied }` (`thiserror`) | `NoOutputDevice` is the "No audio output" case (E-FB-8). Strings carry the backend message (cpal errors are wrapped, never exposed, per AGENTS.md). | Debug |
| `InputDevice` | `struct InputDevice { pub id: String, pub name: String, pub is_default: bool }` | `id` is the cpal device name (stable enough on macOS; documented in D-27). | S, TS |
| `MicPermission` | `enum MicPermission { Unknown, Granted, Denied }` | See Q-5. | S, TS, Copy |
| `OutputHandle` | `struct OutputHandle { .. }` | Glue (`io` module): owns the cpal output stream and the `rtrb::Producer<SynthCommand>`; `send(SynthCommand) -> Result<(), AudioError>` (queue full → dropped with a counter, never blocks); `sample_rate()`. | Internal |
| `InputHandle` | `struct InputHandle { .. }` | Glue: owns the cpal input stream, the ring buffer producer inside the callback, and the analysis thread (`std::thread`); callback closure `Fn(TunerReading) + Send + 'static` supplied by the app; `stop(self)` joins the thread. | Internal |

## 4. `mackey-app` (`src-tauri`)

### 4.1 Errors

| Type | Definition | Notes | Derives |
|------|------------|-------|---------|
| `AppError` | `struct AppError { pub kind: AppErrorKind, pub message: String, pub details: Option<String> }` | `message` is plain words, no codes (NF-22). `details` is the technical chain for the "Details" disclosure. Implements `std::error::Error`; `From<ChordParseError>`, `From<RomanParseError>`, `From<AudioError>`, `From<VoicingError>`, `From<SettingsError>` each pick a kind and a plain message, and the full chain goes to `tracing`. | S, TS |
| `AppErrorKind` | `enum AppErrorKind { Invalid, NoAudioOutput, NoInputDevice, MicrophoneDenied, Settings, Internal }` | The UI branches on `kind` (for example show "No audio output" once, MIC settings button) but never displays it. `Internal` carries the text "Something went wrong" (NF-24). | S, TS, Copy, Eq |
| `SettingsError` | `enum SettingsError { Io(String), Parse(String), Invalid { field: String } }` (`thiserror`) | | Debug |

### 4.2 Settings

All structs `#[serde(default)]` and `#[serde(deny_unknown_fields)]` is **not** set (unknown fields are ignored so older and newer builds can share a file); each group has `Default` equal to the G-4 / 08 §6 defaults.

| Type | Fields (default) | Derives |
|------|------------------|---------|
| `Settings` | `version: u32 (1)`, `instrument: InstrumentSettings`, `display: DisplaySettings`, `sound: SoundSettings`, `tuner: TunerSettings` | S, TS |
| `InstrumentSettings` | `handedness: Handedness (right, no flip)`, `frets: FretCount (22)`, `default_tuning: TuningChoice (Preset(Standard))`, `allow_stretches: bool (false)` | S, TS |
| `TuningChoice` | `enum TuningChoice { Preset(TuningPreset), Custom([Pitch; 6]) }` | S, TS |
| `DisplaySettings` | `theme: Theme (System)`, `labels_default: LabelsMode (NoteName)`, `show_fret_numbers: bool (true)` | S, TS |
| `Theme` | `enum Theme { System, Light, Dark }` | S, TS, Copy |
| `SoundSettings` | `volume: Percent (80)`, `mute: bool (false)` | S, TS |
| `Percent` | `struct Percent(u8)` `0..=100` | S, TS |
| `TunerSettings` | `input_device: Option<String> (None = system default)`, `reference_a4: ReferencePitch (440)`, `mic_explained: bool (false)` | S, TS |
| `SettingsGroup` | `enum SettingsGroup { Instrument, Display, Sound, Tuner }` | S, TS, Copy |
| `SettingsPatch` | `struct SettingsPatch { pub instrument: Option<InstrumentSettings>, pub display: Option<DisplaySettings>, pub sound: Option<SoundSettings>, pub tuner: Option<TunerSettings> }` | S, TS |

File: `settings.json` in the app data directory (Tauri `app_data_dir`), written atomically (temp file in the same directory, then rename). A missing file means defaults; a corrupt file is renamed `settings.json.bak-<unix-seconds>`, defaults are used and a notice is returned once. Values out of range are clamped on load.

### 4.3 Commands, events and snapshot types

| Type | Definition | Notes | Derives |
|------|------------|-------|---------|
| `ExplorerSnapshot` | `struct ExplorerSnapshot { pub revision: u32, pub view: FretboardView, pub panel: ExplorerPanel }` | Returned by every explorer command. `revision` increments on every state change; the UI ignores a snapshot whose revision is lower than the one it holds (out-of-order responses, SAC-18, D-31). | S, TS |
| `ExplorerPanel` | `struct ExplorerPanel { pub key: KeyPanel, pub diatonic: Vec<DiatonicChip>, pub chord_text: String, pub chord_status: ChordFieldStatus, pub instrument: InstrumentPanel, pub selection: Selection, pub scale_choices: Vec<ScaleChoice> }` | Everything the toolbar and panels show that is not the neck. `selection` echoes the Rust selection for controls (selected view type, indexes). | S, TS |
| `KeyPanel` | `struct KeyPanel { pub label: String, pub tonic_choices: Vec<TonicChoice>, pub modes: Vec<ModeChoice>, pub enharmonic_hint: Option<String> }` | "G major"; for F#/Gb the picker asks which spelling (G-1.2). | S, TS |
| `DiatonicChip` | `struct DiatonicChip { pub roman: String, pub symbol: String, pub selected: bool }` | FB-4.2. | S, TS |
| `ChordFieldStatus` | `enum ChordFieldStatus { Empty, Ok, Error { message: String, suggestion: Option<String>, span: Option<(usize, usize)> } }` | E-FB-1; the selection is unchanged on error (FB-4.4). | S, TS |
| `InstrumentPanel` | `struct InstrumentPanel { pub tuning_name: String, pub tuning_choices: Vec<String>, pub warnings: Vec<TuningWarning>, pub capo: u8, pub frets: u8, pub handedness: Handedness }` | | S, TS |
| `ScaleChoice` | `struct ScaleChoice { pub mode: Mode, pub label: String }` | | S, TS |
| `TonicChoice` | `struct TonicChoice { pub label: String, pub note: Note, pub alt: Option<Note> }` | `alt` is the second spelling for F#/Gb and D#/Eb. | S, TS |
| `ModeChoice` | `struct ModeChoice { pub mode: Mode, pub label: String }` | | S, TS |
| `CommandSpec` | `struct CommandSpec { pub name: &'static str, pub args: &'static [(&'static str, TsTypeFn)], pub ret: TsTypeFn }` | Internal manifest entry used by the bindings generator (D-05). | Internal |

Tauri commands (full list in [architecture.md](architecture.md#tauri-commands)): every command returns `Result<T, AppError>`. Events: `tuner_reading` (payload `TunerReading`).

## 5. UI types (`ui/`)

TypeScript only declares view state; everything else is imported from `ui/src/bindings/`. Never hand-edit `ui/src/bindings/`.

| Type (file) | Shape | Notes |
|-------------|-------|-------|
| `UiState` (`ui/src/state/store.ts`, Zustand) | `{ snapshot: ExplorerSnapshot \| null; visible: FretRange; area: 'explore' \| 'tuner' \| 'settings'; sidebarOpen: boolean; tooltip: { string, fret } \| null; notice: string \| null; liveMessage: string; settings: Settings \| null; tuner: TunerReading \| null; audioNoticeShown: boolean }` | View state only (ADR 0007). |
| `Layout` (`ui/src/layout/useLayout.ts`) | `'mac' \| 'ipad-landscape' \| 'ipad-portrait' \| 'narrow'` | From viewport width and pointer type; sets the default visible range (G-4: 0 to 15, 0 to 12, 0 to 7). |
| `Shortcut` (`ui/src/shortcuts.ts`) | `{ area: 'global' \| 'explore'; key: string; shift?: boolean; meta?: boolean; action: CycleAction \| UiCommand; label: string }` | The table AC-UX-7 tests. |
| `Commands` (`ui/src/commands/index.ts`) | the generated `commands` object (real) or `mockCommands` (Playwright, Vitest) | Selected at start-up by `window.__TAURI_INTERNALS__` presence and `import.meta.env.MODE === 'mock'`. |
| `Scenario` (`ui/src/mocks/scenarios.ts`) | `{ name: string; steps: Array<{ call: string; args: unknown; result: ExplorerSnapshot }> }` | Recorded from the real Rust core by the bindings export test, so mocks never contain theory (D-14). |

## 6. Constants shared across crates

| Constant | Value | Where | Source |
|----------|-------|-------|--------|
| Default key | G major | `Selection::default_g_major` | G-4 |
| Default tuning | Standard | `Instrument::default` | G-4 |
| Default frets, capo | 22, 0 | `Instrument::default` | G-4 |
| Default labels | `NoteName` | `Selection` | G-4 |
| Default tempo for playback | 70 bpm | `Bpm::default` | G-4 |
| Reference A4 | 440 Hz, 415..=466 | `ReferencePitch` | G-5, TU-3 |
| Strum spacing | 30 ms | `playback_plan` | FB-8.1 |
| In-tune window / hold | ±3 cents / 300 ms | `Tuner` | TU-1.1 |
| No-pitch timeout | 500 ms | `Tuner` | TU-1.4 |
| Too-quiet timeout | 3 s | `Tuner` | E-TU-3 |
| Detection range | 60 to 1,400 Hz | `DetectorConfig` | TU-1.2 |
| Tuner event rate | about 30 Hz | app emitter | architecture §5 |
| Min fret width | 24 px (Mac), 28 pt (iPad) | `ui/src/fretboard/geometry.ts` | FB-1.4 |
