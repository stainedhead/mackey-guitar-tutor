# Data Dictionary: Fretboard Foundation Auto Review Fixes

**Created:** 2026-11-15

Types that change (all in the original spec's [data dictionary](../data-dictionary.md)):

- Pattern list label: `"Box {number} of {count}"` for pentatonic boxes, `"Pattern {number} of {count}"` for seven-note patterns; numbering from `PatternKind`, never from the sort index.
- Stepper state: `pattern_index` selects within the list ordered by pattern number.
- Tuner notice text for an unpaced input: "This input is not delivering audio at a normal rate".
- Triad placement: unchanged shape; invariant added that pitches ascend low string to top string and the lowest is the inversion's bass.
