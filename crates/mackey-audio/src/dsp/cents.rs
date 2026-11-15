//! Cents arithmetic against equal-tempered pitches.

use mackey_theory::{AccidentalPreference, Pitch, ReferencePitch};

/// Signed distance in cents from `reference_hz` to `freq_hz` (positive when `freq_hz` is
/// sharp). Returns `None` when either frequency is not a positive finite number.
///
/// ```
/// use mackey_audio::dsp::cents_between;
/// let c = cents_between(880.0, 440.0).unwrap();
/// assert!((c - 1200.0).abs() < 1e-9);
/// assert!(cents_between(0.0, 440.0).is_none());
/// ```
#[must_use]
pub fn cents_between(freq_hz: f64, reference_hz: f64) -> Option<f64> {
    let ok = |f: f64| f.is_finite() && f > 0.0;
    (ok(freq_hz) && ok(reference_hz)).then(|| 1200.0 * (freq_hz / reference_hz).log2())
}

/// Cents from `target` (tuned against `a4`) to `freq_hz`.
///
/// ```
/// use mackey_audio::dsp::cents_from;
/// use mackey_theory::{Pitch, ReferencePitch};
/// let a4 = Pitch::new("A".parse().unwrap(), 4).unwrap();
/// let c = cents_from(440.0, a4, ReferencePitch::new(442.0).unwrap()).unwrap();
/// assert!((c + 7.85).abs() < 0.01);
/// ```
#[must_use]
pub fn cents_from(freq_hz: f64, target: Pitch, a4: ReferencePitch) -> Option<f64> {
    cents_between(freq_hz, target.frequency(a4))
}

/// The nearest equal-tempered pitch (spelled with sharps) and the offset in cents, within
/// `-50.0..=50.0`. `None` for a non-positive frequency or one outside the MIDI range.
///
/// ```
/// use mackey_audio::dsp::nearest_pitch;
/// use mackey_theory::ReferencePitch;
/// let (p, cents) = nearest_pitch(73.42, ReferencePitch::default()).unwrap();
/// assert_eq!(p.name(), "D2");
/// assert!(cents.abs() < 1.0);
/// ```
#[must_use]
pub fn nearest_pitch(freq_hz: f64, a4: ReferencePitch) -> Option<(Pitch, f64)> {
    let semitones = 12.0 * (freq_hz / a4.hz()).log2();
    if !semitones.is_finite() {
        return None;
    }
    let midi_f = 69.0 + semitones;
    let midi = midi_f.round();
    if !(0.0..=127.0).contains(&midi) {
        return None;
    }
    let pitch = Pitch::from_midi(midi as i32, AccidentalPreference::Sharps).ok()?;
    Some((pitch, (midi_f - midi) * 100.0))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(midi: i32) -> Pitch {
        Pitch::from_midi(midi, AccidentalPreference::Sharps).unwrap()
    }

    #[test]
    fn between_rejects_bad_input() {
        for (f, r) in [(0.0, 440.0), (-1.0, 440.0), (440.0, 0.0), (f64::NAN, 440.0)] {
            assert!(cents_between(f, r).is_none(), "{f} vs {r}");
        }
    }

    #[test]
    fn between_octave_and_semitone() {
        assert!((cents_between(220.0, 440.0).unwrap() + 1200.0).abs() < 1e-9);
        let semi = 440.0 * 2f64.powf(1.0 / 12.0);
        assert!((cents_between(semi, 440.0).unwrap() - 100.0).abs() < 1e-9);
    }

    #[test]
    fn ac_tu_3_1_reference_442() {
        let c = cents_from(440.0, p(69), ReferencePitch::new(442.0).unwrap()).unwrap();
        assert!((c - (-7.85)).abs() < 0.01, "{c}");
    }

    #[test]
    fn nearest_rounds_and_bounds() {
        let a4 = ReferencePitch::default();
        const CASES: &[(f64, i32)] = &[(440.0, 69), (73.42, 38), (82.41, 40), (261.63, 60)];
        for &(f, midi) in CASES {
            let (pitch, cents) = nearest_pitch(f, a4).unwrap();
            assert_eq!(pitch.midi(), midi, "{f}");
            assert!(cents.abs() <= 50.0, "{f}");
        }
        let (pitch, cents) = nearest_pitch(440.0 * 2f64.powf(0.51 / 12.0), a4).unwrap();
        assert_eq!(pitch.midi(), 70);
        assert!((cents + 49.0).abs() < 0.01);
    }

    #[test]
    fn nearest_rejects_out_of_range() {
        let a4 = ReferencePitch::default();
        for f in [0.0, -5.0, f64::NAN, f64::INFINITY, 1.0, 1.0e6] {
            assert!(nearest_pitch(f, a4).is_none(), "{f}");
        }
    }
}
