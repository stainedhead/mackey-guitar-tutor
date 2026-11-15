//! The tuner state machine: a pure function of sample blocks.
//!
//! Time is counted in samples (`ms = samples * 1000 / sample_rate`), never read from a clock,
//! so every rule is deterministic and tested with synthetic signals. The first reading is made
//! as soon as one analysis window (2048 samples, about 43 ms at 48 kHz) exists and then one
//! per hop (512 samples, about 94 per second); the app throttles what it shows to about 30 Hz.

use mackey_theory::{Note, Pitch, ReferencePitch};
use serde::{Deserialize, Serialize};

use super::cents::{cents_between, nearest_pitch};
use super::level::{Level, LevelMeter};
use super::pitch::PitchDetector;

/// In tune: within this many cents of the target (TU-1.1).
const IN_TUNE_CENTS: f32 = 3.0;
/// The offset must hold this long before "In tune" shows (TU-1.1).
const IN_TUNE_HOLD_MS: u64 = 300;
/// With no pitch for this long, "Play a single string" shows (TU-1.4).
const NO_PITCH_MS: u64 = 500;
/// Level below this for [`QUIET_MS`] shows "too quiet" (E-TU-3): -50 dBFS RMS.
const QUIET_RMS: f32 = 0.003_162_277_7;
/// See [`QUIET_RMS`].
const QUIET_MS: u64 = 3000;
/// "Too loud" stays up this long after the last clipped sample so it does not flicker.
const CLIP_HOLD_MS: u64 = 1000;
/// A candidate this clear but below the detector threshold means two strings are ringing.
const LOW_CONFIDENCE_FLOOR: f32 = 0.5;
/// More than this many cents from every target string is "far from any string" (TU-2.1).
const FAR_CENTS: f64 = 300.0;

/// One string to tune to, built by the app from the instrument (tuning plus capo).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct TunerTarget {
    /// String number, 1 (thinnest) to 6 (thickest).
    pub string: u8,
    /// The sounding pitch of the open string (capo applied).
    pub pitch: Pitch,
    /// Its frequency in Hz for the configured A4.
    pub frequency_hz: f64,
}

/// Targets for six strings given low to high (index 0 is string 6), as the app has them from
/// the instrument.
///
/// ```
/// use mackey_audio::dsp::targets_from_pitches;
/// use mackey_theory::{AccidentalPreference::Sharps, Pitch, ReferencePitch};
/// let p = |m| Pitch::from_midi(m, Sharps).unwrap();
/// let std = [p(40), p(45), p(50), p(55), p(59), p(64)];
/// let t = targets_from_pitches(&std, ReferencePitch::default());
/// assert_eq!((t[0].string, t[5].string), (6, 1));
/// assert!((t[1].frequency_hz - 110.0).abs() < 1e-9);
/// ```
#[must_use]
pub fn targets_from_pitches(pitches: &[Pitch; 6], a4: ReferencePitch) -> Vec<TunerTarget> {
    pitches
        .iter()
        .enumerate()
        .map(|(i, &pitch)| TunerTarget {
            string: 6 - i as u8,
            pitch,
            frequency_hz: pitch.frequency(a4),
        })
        .collect()
}

/// Chromatic (any note) or string mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum TunerMode {
    /// Show the nearest note.
    Chromatic,
    /// Tune to the target strings; `locked` pins one string instead of auto-selecting.
    Strings {
        /// The tapped string, if any.
        locked: Option<u8>,
    },
}

/// Tuner settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct TunerConfig {
    /// Reference pitch for A4 (415 to 466 Hz).
    pub a4: ReferencePitch,
    /// Chromatic or string mode.
    pub mode: TunerMode,
    /// Target strings for string mode; may be empty in chromatic mode.
    pub targets: Vec<TunerTarget>,
}

impl TunerConfig {
    /// Chromatic mode with no targets.
    #[must_use]
    pub fn chromatic(a4: ReferencePitch) -> Self {
        Self {
            a4,
            mode: TunerMode::Chromatic,
            targets: Vec::new(),
        }
    }
}

/// What the tuner screen should show.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum TunerStatus {
    /// Just started; nothing heard yet.
    Listening,
    /// A pitch is shown.
    Pitch,
    /// No stable pitch for 500 ms: "Play a single string".
    NoStablePitch,
    /// Level under -50 dBFS for 3 s: "Too quiet".
    TooQuiet,
    /// Clipping: "Too loud".
    TooLoud,
    /// Something periodic but unclear, usually two strings ringing: "Play a single string".
    LowConfidence,
}

/// Which way to turn the peg.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum TuneDirection {
    /// Within 3 cents.
    InTune,
    /// The pitch is flat: tighten the string.
    Tighten,
    /// The pitch is sharp: loosen the string.
    Loosen,
}

/// The auto-selected (or locked) target string and the offset from it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct TargetReading {
    /// String number, 1 to 6.
    pub string: u8,
    /// Smoothed offset from that string's pitch in cents (may exceed 50 when far off).
    pub cents: f32,
}

/// Something the glue wants the user to know.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum TunerNotice {
    /// The input device went away; the default input is in use (E-TU-5).
    DeviceDisconnectedUsingDefault,
    /// The input device changed.
    DeviceChanged,
    /// The analysis thread panicked and stopped.
    AnalysisStopped,
    /// The input delivers audio far faster than real time (an unpaced virtual device), so
    /// analysis was stopped instead of spinning.
    InputNotPaced,
}

/// One tuner update, emitted per hop.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct TunerReading {
    /// What to show.
    pub status: TunerStatus,
    /// Nearest equal-tempered note (spelled with sharps) when a pitch is shown.
    pub note: Option<Note>,
    /// Its octave.
    pub octave: Option<i8>,
    /// Smoothed offset from that note in cents, `-50..=50`.
    pub cents: Option<f32>,
    /// Detected frequency in Hz (raw, not smoothed).
    pub frequency_hz: Option<f64>,
    /// Clarity of the latest detection, `0..=1`.
    pub confidence: f32,
    /// Input level of the latest hop.
    pub level: Level,
    /// Within 3 cents of the target for 300 ms.
    pub in_tune: bool,
    /// Tighten, loosen or in tune (instant, within 3 cents; `in_tune` is the held version).
    pub direction: Option<TuneDirection>,
    /// String mode: the target string and the offset from it.
    pub target: Option<TargetReading>,
    /// String mode: more than 3 semitones from every target.
    pub far_from_any: bool,
    /// Strings that have been in tune this session, ascending.
    pub ticks: Vec<u8>,
    /// A one-off notice from the device glue.
    pub notice: Option<TunerNotice>,
    /// Stream time in milliseconds, from the sample counter.
    pub t_ms: f64,
}

impl TunerReading {
    /// A reading that carries only a notice (used when the analysis thread has to stop).
    #[must_use]
    pub fn notice_only(notice: TunerNotice, t_ms: f64) -> Self {
        Self {
            status: TunerStatus::Listening,
            note: None,
            octave: None,
            cents: None,
            frequency_hz: None,
            confidence: 0.0,
            level: Level::SILENT,
            in_tune: false,
            direction: None,
            target: None,
            far_from_any: false,
            ticks: Vec::new(),
            notice: Some(notice),
            t_ms,
        }
    }
}

/// Median of the last three values for one key (a MIDI note or a string); a new key restarts
/// it. With fewer than three values it uses what it has (two: their mean) so the first reading
/// is not delayed.
#[derive(Debug, Clone, Default)]
struct Median3 {
    vals: [f32; 3],
    len: usize,
    key: Option<i64>,
}

impl Median3 {
    fn reset(&mut self) {
        self.len = 0;
        self.key = None;
    }

    /// Returns the smoothed value and whether the key changed.
    fn push(&mut self, key: i64, v: f32) -> (f32, bool) {
        let changed = self.key != Some(key);
        if changed {
            self.len = 0;
            self.key = Some(key);
        }
        if self.len == 3 {
            self.vals.rotate_left(1);
            self.vals[2] = v;
        } else {
            self.vals[self.len] = v;
            self.len += 1;
        }
        let m = match self.len {
            1 => self.vals[0],
            2 => f32::midpoint(self.vals[0], self.vals[1]),
            _ => {
                let [a, b, c] = self.vals;
                a.max(b).min(a.min(b).max(c))
            }
        };
        (m, changed)
    }
}

/// What the latest pitch looked like; shown again for up to 500 ms after the pitch is lost.
#[derive(Debug, Clone)]
struct Held {
    note: Note,
    octave: i8,
    cents: f32,
    frequency_hz: f64,
    direction: TuneDirection,
    target: Option<TargetReading>,
    far_from_any: bool,
}

/// The tuner. Feed it audio with [`Tuner::push_block`]; it returns the readings that became
/// due.
#[derive(Debug)]
pub struct Tuner {
    cfg: TunerConfig,
    detector: PitchDetector,
    meter: LevelMeter,
    window: usize,
    hop: u64,
    sample_rate: u64,
    ring: Vec<f32>,
    linear: Vec<f32>,
    write: usize,
    total: u64,
    next_analysis: u64,
    prev_analysis: u64,
    quiet_samples: u64,
    last_clip: Option<u64>,
    last_pitch: Option<u64>,
    held: Option<Held>,
    chroma: Median3,
    target_median: Median3,
    in_tune_since: Option<u64>,
    ticks: Vec<u8>,
    pending_notice: Option<TunerNotice>,
}

impl Tuner {
    /// A tuner around `detector`; the window, hop and sample rate come from the detector's
    /// configuration.
    #[must_use]
    pub fn new(cfg: TunerConfig, detector: PitchDetector) -> Self {
        let dc = *detector.config();
        Self {
            cfg,
            detector,
            meter: LevelMeter::new(),
            window: dc.window,
            hop: dc.hop as u64,
            sample_rate: u64::from(dc.sample_rate),
            ring: vec![0.0; dc.window],
            linear: vec![0.0; dc.window],
            write: 0,
            total: 0,
            next_analysis: dc.window as u64,
            prev_analysis: 0,
            quiet_samples: 0,
            last_clip: None,
            last_pitch: None,
            held: None,
            chroma: Median3::default(),
            target_median: Median3::default(),
            in_tune_since: None,
            ticks: Vec::new(),
            pending_notice: None,
        }
    }

    /// The current configuration.
    #[must_use]
    pub fn config(&self) -> &TunerConfig {
        &self.cfg
    }

    /// Samples between two readings.
    #[must_use]
    pub fn hop(&self) -> usize {
        self.hop as usize
    }

    /// Sample rate this tuner was built for.
    #[must_use]
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate as u32
    }

    /// Replaces the configuration. Smoothing restarts; ticks are kept unless the targets
    /// changed (a different tuning or capo makes them meaningless).
    pub fn set_config(&mut self, cfg: TunerConfig) {
        if cfg.targets != self.cfg.targets {
            self.ticks.clear();
        }
        self.cfg = cfg;
        self.restart_smoothing();
        self.held = None;
    }

    /// Locks the target to `string` (tap), or back to auto-select with `None`. No effect in
    /// chromatic mode.
    pub fn lock(&mut self, string: Option<u8>) {
        if let TunerMode::Strings { locked } = &mut self.cfg.mode {
            *locked = string;
            self.restart_smoothing();
            self.held = None;
        }
    }

    /// Forgets which strings have been in tune.
    pub fn reset_ticks(&mut self) {
        self.ticks.clear();
    }

    /// The next reading carries `notice`.
    pub fn queue_notice(&mut self, notice: TunerNotice) {
        self.pending_notice = Some(notice);
    }

    fn restart_smoothing(&mut self) {
        self.chroma.reset();
        self.target_median.reset();
        self.in_tune_since = None;
    }

    fn ms_to_samples(&self, ms: u64) -> u64 {
        ms * self.sample_rate / 1000
    }

    fn samples_to_ms(&self, samples: u64) -> f64 {
        samples as f64 * 1000.0 / self.sample_rate as f64
    }

    /// Adds `block` (mono, any length) and returns the readings that fell due inside it, in
    /// order. Non-finite samples are treated as silence.
    pub fn push_block(&mut self, block: &[f32]) -> Vec<TunerReading> {
        let mut out = Vec::new();
        let mut rest = block;
        while !rest.is_empty() {
            let until = (self.next_analysis - self.total) as usize;
            let take = rest.len().min(until);
            for &s in &rest[..take] {
                self.ring[self.write] = if s.is_finite() { s } else { 0.0 };
                self.write += 1;
                if self.write == self.window {
                    self.write = 0;
                }
            }
            self.total += take as u64;
            rest = &rest[take..];
            if self.total == self.next_analysis {
                out.push(self.analyse());
                self.prev_analysis = self.total;
                self.next_analysis += self.hop;
            }
        }
        out
    }

    fn analyse(&mut self) -> TunerReading {
        let (newest, oldest) = self.ring.split_at(self.write);
        self.linear[..oldest.len()].copy_from_slice(oldest);
        self.linear[oldest.len()..].copy_from_slice(newest);

        let t = self.total;
        let advance = t - self.prev_analysis;
        let fresh = (advance as usize).min(self.window);
        let level = self.meter.process(&self.linear[self.window - fresh..]);

        if level.rms < QUIET_RMS {
            self.quiet_samples += advance;
        } else {
            self.quiet_samples = 0;
        }
        if level.clipping {
            self.last_clip = Some(t);
        }
        let clip_hold = self.ms_to_samples(CLIP_HOLD_MS);
        let clipped = self.last_clip.is_some_and(|c| t - c <= clip_hold);

        let estimate = self.detector.analyse(&self.linear);
        let confidence = estimate.map_or(0.0, |e| e.clarity);
        let threshold = self.detector.config().clarity_threshold;

        if clipped {
            self.restart_smoothing();
            return self.reading(TunerStatus::TooLoud, level, confidence, None, t);
        }

        let pitched = estimate
            .filter(|e| e.clarity >= threshold)
            .and_then(|e| self.content(e.frequency_hz, t));
        if let Some(held) = pitched {
            self.last_pitch = Some(t);
            self.held = Some(held.clone());
            return self.reading(TunerStatus::Pitch, level, confidence, Some(held), t);
        }

        let silent_for = self.last_pitch.map_or(t, |p| t - p);
        if silent_for < self.ms_to_samples(NO_PITCH_MS) {
            let shown = self.held.clone();
            let status = if shown.is_some() {
                TunerStatus::Pitch
            } else {
                TunerStatus::Listening
            };
            return self.reading(status, level, confidence, shown, t);
        }
        self.held = None;
        self.restart_smoothing();
        let status = if self.quiet_samples >= self.ms_to_samples(QUIET_MS) {
            TunerStatus::TooQuiet
        } else if confidence >= LOW_CONFIDENCE_FLOOR {
            TunerStatus::LowConfidence
        } else {
            TunerStatus::NoStablePitch
        };
        self.reading(status, level, confidence, None, t)
    }

    /// Smoothing, target selection and the in-tune window for one detected frequency.
    fn content(&mut self, freq: f64, t: u64) -> Option<Held> {
        let (pitch, raw) = nearest_pitch(freq, self.cfg.a4)?;
        let (cents, _) = self.chroma.push(i64::from(pitch.midi()), raw as f32);

        let mut target = None;
        let mut far = false;
        let mut band = cents;
        let mut key_changed = false;
        if let TunerMode::Strings { locked } = self.cfg.mode
            && let Some(chosen) = self.choose_target(freq, locked)
        {
            let raw_t = cents_between(freq, chosen.frequency_hz)?;
            let (smoothed, changed) = self
                .target_median
                .push(i64::from(chosen.string), raw_t as f32);
            key_changed = changed;
            band = smoothed;
            target = Some(TargetReading {
                string: chosen.string,
                cents: smoothed,
            });
            far =
                self.cfg.targets.iter().all(|c| {
                    cents_between(freq, c.frequency_hz).is_none_or(|x| x.abs() > FAR_CENTS)
                });
        }
        if key_changed || band.abs() > IN_TUNE_CENTS {
            self.in_tune_since = None;
        }
        if band.abs() <= IN_TUNE_CENTS {
            self.in_tune_since.get_or_insert(t);
        }
        let direction = if band.abs() <= IN_TUNE_CENTS {
            TuneDirection::InTune
        } else if band < 0.0 {
            TuneDirection::Tighten
        } else {
            TuneDirection::Loosen
        };
        Some(Held {
            note: pitch.note(),
            octave: pitch.octave(),
            cents,
            frequency_hz: freq,
            direction,
            target,
            far_from_any: far,
        })
    }

    fn choose_target(&self, freq: f64, locked: Option<u8>) -> Option<TunerTarget> {
        if let Some(s) = locked
            && let Some(t) = self.cfg.targets.iter().find(|t| t.string == s)
        {
            return Some(*t);
        }
        self.cfg
            .targets
            .iter()
            .filter_map(|t| cents_between(freq, t.frequency_hz).map(|c| (c.abs(), *t)))
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, t)| t)
    }

    fn reading(
        &mut self,
        status: TunerStatus,
        level: Level,
        confidence: f32,
        held: Option<Held>,
        t: u64,
    ) -> TunerReading {
        let hold_samples = self.ms_to_samples(IN_TUNE_HOLD_MS);
        let in_tune = held.is_some()
            && self.in_tune_since.is_some_and(|s| t - s >= hold_samples)
            && status == TunerStatus::Pitch;
        if in_tune
            && let Some(tr) = held.as_ref().and_then(|h| h.target)
            && let Err(pos) = self.ticks.binary_search(&tr.string)
        {
            self.ticks.insert(pos, tr.string);
        }
        let t_ms = self.samples_to_ms(t);
        let notice = self.pending_notice.take();
        match held {
            Some(h) => TunerReading {
                status,
                note: Some(h.note),
                octave: Some(h.octave),
                cents: Some(h.cents),
                frequency_hz: Some(h.frequency_hz),
                confidence,
                level,
                in_tune,
                direction: Some(h.direction),
                target: h.target,
                far_from_any: h.far_from_any,
                ticks: self.ticks.clone(),
                notice,
                t_ms,
            },
            None => TunerReading {
                status,
                note: None,
                octave: None,
                cents: None,
                frequency_hz: None,
                confidence,
                level,
                in_tune: false,
                direction: None,
                target: None,
                far_from_any: false,
                ticks: self.ticks.clone(),
                notice,
                t_ms,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsp::DetectorConfig;
    use mackey_theory::AccidentalPreference::Sharps;

    const SR: u32 = 48_000;

    fn tuner(cfg: TunerConfig) -> Tuner {
        Tuner::new(
            cfg,
            PitchDetector::new(DetectorConfig::new(SR).unwrap()).unwrap(),
        )
    }

    fn sine(f: f64, amp: f32, n: usize) -> Vec<f32> {
        (0..n)
            .map(|i| amp * (2.0 * std::f64::consts::PI * f * i as f64 / f64::from(SR)).sin() as f32)
            .collect()
    }

    fn last(t: &mut Tuner, block: &[f32]) -> TunerReading {
        t.push_block(block).pop().expect("a reading")
    }

    fn std_targets() -> Vec<TunerTarget> {
        let p = |m| Pitch::from_midi(m, Sharps).unwrap();
        targets_from_pitches(
            &[p(40), p(45), p(50), p(55), p(59), p(64)],
            ReferencePitch::default(),
        )
    }

    #[test]
    fn median_of_three() {
        let mut m = Median3::default();
        assert_eq!(m.push(1, 10.0), (10.0, true));
        assert_eq!(m.push(1, 20.0), (15.0, false));
        assert_eq!(m.push(1, 12.0).0, 12.0);
        assert_eq!(m.push(1, 100.0).0, 20.0);
        assert_eq!(m.push(2, 5.0), (5.0, true));
        m.reset();
        assert_eq!(m.push(2, 7.0), (7.0, true));
    }

    #[test]
    fn first_reading_arrives_when_the_window_is_full() {
        let mut t = tuner(TunerConfig::chromatic(ReferencePitch::default()));
        assert_eq!(t.push_block(&sine(440.0, 0.5, 2047)).len(), 0);
        let r = t.push_block(&sine(440.0, 0.5, 1));
        assert_eq!(r.len(), 1);
        assert!((r[0].t_ms - 2048.0 * 1000.0 / 48_000.0).abs() < 1e-9);
        assert_eq!((t.hop(), t.sample_rate()), (512, SR));
    }

    #[test]
    fn large_blocks_yield_several_readings() {
        let mut t = tuner(TunerConfig::chromatic(ReferencePitch::default()));
        let r = t.push_block(&sine(440.0, 0.5, 2048 + 3 * 512));
        assert_eq!(r.len(), 4);
        assert!(r.windows(2).all(|w| w[1].t_ms > w[0].t_ms));
    }

    #[test]
    fn pitch_reading_fields() {
        let mut t = tuner(TunerConfig::chromatic(ReferencePitch::default()));
        let r = last(&mut t, &sine(440.0, 0.5, 2048));
        assert_eq!(r.status, TunerStatus::Pitch);
        assert_eq!(r.note.unwrap().name(), "A");
        assert_eq!(r.octave, Some(4));
        assert!(r.cents.unwrap().abs() < 1.0);
        assert!(r.confidence > 0.9 && r.level.rms > 0.3);
        assert!(r.target.is_none() && !r.far_from_any && r.ticks.is_empty());
    }

    #[test]
    fn direction_follows_the_sign() {
        let flat = 440.0 * 2f64.powf(-20.0 / 1200.0);
        let sharp = 440.0 * 2f64.powf(20.0 / 1200.0);
        for (f, want) in [
            (flat, TuneDirection::Tighten),
            (sharp, TuneDirection::Loosen),
        ] {
            let mut t = tuner(TunerConfig::chromatic(ReferencePitch::default()));
            let r = last(&mut t, &sine(f, 0.5, 2048));
            assert_eq!(r.direction, Some(want), "{f}");
            assert!(!r.in_tune);
        }
    }

    #[test]
    fn in_tune_needs_a_300_ms_hold() {
        let mut t = tuner(TunerConfig::chromatic(ReferencePitch::default()));
        let rs = t.push_block(&sine(440.0, 0.5, 48_000));
        let first = rs.iter().find(|r| r.in_tune).expect("in tune");
        assert!(first.t_ms >= 2048.0 * 1000.0 / 48_000.0 + 300.0 - 1e-6);
        assert!(
            rs.iter()
                .take_while(|r| !r.in_tune)
                .all(|r| r.direction == Some(TuneDirection::InTune))
        );
    }

    #[test]
    fn silence_goes_listening_then_no_stable_pitch_then_too_quiet() {
        // (label, seconds of silence, expected status of the last reading)
        const CASES: &[(&str, f64, TunerStatus)] = &[
            ("start", 0.1, TunerStatus::Listening),
            ("after 500 ms", 0.7, TunerStatus::NoStablePitch),
            ("after 2.9 s", 2.9, TunerStatus::NoStablePitch),
            ("after 3.2 s", 3.3, TunerStatus::TooQuiet),
        ];
        for &(label, secs, want) in CASES {
            let mut t = tuner(TunerConfig::chromatic(ReferencePitch::default()));
            let r = last(&mut t, &vec![0.0; (secs * 48_000.0) as usize]);
            assert_eq!(r.status, want, "{label}");
        }
    }

    #[test]
    fn pitch_is_held_for_500_ms_after_it_stops() {
        let mut t = tuner(TunerConfig::chromatic(ReferencePitch::default()));
        let _ = t.push_block(&sine(440.0, 0.5, 4096));
        // 200 ms of silence: the window is still mostly tone, then mostly silence.
        let r = last(&mut t, &vec![0.0; 9600]);
        assert_eq!(r.status, TunerStatus::Pitch, "held");
        let r = last(&mut t, &vec![0.0; 48_000 / 2]);
        assert_eq!(r.status, TunerStatus::NoStablePitch);
        assert!(r.note.is_none() && !r.in_tune);
    }

    #[test]
    fn noise_is_no_stable_pitch_not_too_quiet() {
        let mut t = tuner(TunerConfig::chromatic(ReferencePitch::default()));
        let mut s = 99_u32;
        let noise: Vec<f32> = (0..48_000)
            .map(|_| {
                s = s.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
                ((s >> 8) as f32 / 8_388_608.0 - 1.0) * 0.2
            })
            .collect();
        let r = last(&mut t, &noise);
        assert!(matches!(
            r.status,
            TunerStatus::NoStablePitch | TunerStatus::LowConfidence
        ));
    }

    #[test]
    fn clipping_is_too_loud_and_holds_for_a_second() {
        let mut t = tuner(TunerConfig::chromatic(ReferencePitch::default()));
        let loud: Vec<f32> = sine(440.0, 3.0, 4096)
            .iter()
            .map(|v| v.clamp(-1.0, 1.0))
            .collect();
        let r = last(&mut t, &loud);
        assert_eq!(r.status, TunerStatus::TooLoud);
        assert!(r.level.clipping);
        let r = last(&mut t, &sine(440.0, 0.5, 24_000));
        assert_eq!(r.status, TunerStatus::TooLoud, "held 1 s");
        let r = last(&mut t, &sine(440.0, 0.5, 48_000));
        assert_eq!(r.status, TunerStatus::Pitch, "released");
    }

    #[test]
    fn two_tones_never_give_a_wrong_pitch() {
        let mut t = tuner(TunerConfig::chromatic(ReferencePitch::default()));
        let mix: Vec<f32> = sine(110.0, 0.3, 48_000)
            .iter()
            .zip(sine(165.0, 0.25, 48_000))
            .map(|(a, b)| a + b)
            .collect();
        for r in t.push_block(&mix) {
            if r.status == TunerStatus::Pitch {
                let f = r.frequency_hz.unwrap();
                assert!((f - 110.0).abs() < 2.0 || (f - 165.0).abs() < 2.0, "{f}");
            }
        }
    }

    #[test]
    fn string_mode_auto_selects_and_ticks() {
        let cfg = TunerConfig {
            a4: ReferencePitch::default(),
            mode: TunerMode::Strings { locked: None },
            targets: std_targets(),
        };
        let mut t = tuner(cfg);
        let rs = t.push_block(&sine(110.0, 0.5, 48_000));
        let r = rs.last().unwrap();
        assert_eq!(r.target.unwrap().string, 5);
        assert!(r.in_tune && r.ticks == [5]);
        assert!(!r.far_from_any);
        // Another string adds a second tick; ticks stay sorted.
        let rs = t.push_block(&sine(329.63, 0.5, 48_000));
        assert_eq!(rs.last().unwrap().ticks, [1, 5]);
        // Off pitch (about +85 cents of string 5), the ticks stay until reset.
        let r = last(&mut t, &sine(110.0 * 1.05, 0.5, 4096));
        assert!(!r.in_tune && r.ticks == [1, 5]);
        t.reset_ticks();
        assert_eq!(last(&mut t, &sine(110.0 * 1.05, 0.5, 512)).ticks.len(), 0);
    }

    #[test]
    fn far_from_any_string_names_the_nearest() {
        let cfg = TunerConfig {
            a4: ReferencePitch::default(),
            mode: TunerMode::Strings { locked: None },
            targets: std_targets(),
        };
        let mut t = tuner(cfg);
        // E-flat-ish: 4+ semitones above the top string E4 (329.63) is G#4/415 Hz.
        let r = last(&mut t, &sine(466.16, 0.5, 4096));
        assert!(r.far_from_any);
        assert_eq!(r.target.unwrap().string, 1);
        assert!(!r.in_tune);
    }

    #[test]
    fn lock_overrides_auto_select() {
        let cfg = TunerConfig {
            a4: ReferencePitch::default(),
            mode: TunerMode::Strings { locked: None },
            targets: std_targets(),
        };
        let mut t = tuner(cfg);
        t.lock(Some(6));
        // 110 Hz is string 5, but string 6 is locked: it is 5 semitones sharp of E2.
        let r = last(&mut t, &sine(110.0, 0.5, 4096));
        let tr = r.target.unwrap();
        assert_eq!(tr.string, 6);
        assert!((tr.cents - 500.0).abs() < 5.0);
        assert_eq!(r.direction, Some(TuneDirection::Loosen));
        t.lock(None);
        assert_eq!(
            last(&mut t, &sine(110.0, 0.5, 4096)).target.unwrap().string,
            5
        );
        // A lock on a string that does not exist falls back to auto-select.
        t.lock(Some(9));
        assert_eq!(
            last(&mut t, &sine(110.0, 0.5, 4096)).target.unwrap().string,
            5
        );
    }

    #[test]
    fn lock_is_ignored_in_chromatic_mode() {
        let mut t = tuner(TunerConfig::chromatic(ReferencePitch::default()));
        t.lock(Some(3));
        assert_eq!(t.config().mode, TunerMode::Chromatic);
    }

    #[test]
    fn set_config_clears_ticks_only_when_targets_change() {
        let mut cfg = TunerConfig {
            a4: ReferencePitch::default(),
            mode: TunerMode::Strings { locked: None },
            targets: std_targets(),
        };
        let mut t = tuner(cfg.clone());
        let _ = t.push_block(&sine(110.0, 0.5, 48_000));
        t.set_config(cfg.clone());
        assert_eq!(last(&mut t, &sine(110.0, 0.5, 4096)).ticks, [5]);
        cfg.targets.truncate(3);
        t.set_config(cfg);
        assert_eq!(last(&mut t, &sine(110.0, 0.5, 4096)).ticks.len(), 0);
    }

    #[test]
    fn notices_ride_on_the_next_reading_once() {
        let mut t = tuner(TunerConfig::chromatic(ReferencePitch::default()));
        t.queue_notice(TunerNotice::DeviceChanged);
        let rs = t.push_block(&sine(440.0, 0.5, 2048 + 512));
        assert_eq!(rs[0].notice, Some(TunerNotice::DeviceChanged));
        assert_eq!(rs[1].notice, None);
        let n = TunerReading::notice_only(TunerNotice::AnalysisStopped, 5.0);
        assert_eq!(
            (n.notice, n.t_ms),
            (Some(TunerNotice::AnalysisStopped), 5.0)
        );
    }

    #[test]
    fn non_finite_samples_do_not_poison_the_window() {
        let mut t = tuner(TunerConfig::chromatic(ReferencePitch::default()));
        let mut s = sine(440.0, 0.5, 4096);
        s[100] = f32::NAN;
        s[200] = f32::INFINITY;
        assert_eq!(last(&mut t, &s).status, TunerStatus::Pitch);
    }

    #[test]
    fn reading_serialises() {
        let mut t = tuner(TunerConfig::chromatic(ReferencePitch::default()));
        let r = last(&mut t, &sine(440.0, 0.5, 2048));
        let json = serde_json::to_string(&r).unwrap();
        let back: TunerReading = serde_json::from_str(&json).unwrap();
        assert_eq!(back, r);
    }

    #[test]
    fn target_helper_orders_strings() {
        let t = std_targets();
        assert_eq!(
            t.iter().map(|x| x.string).collect::<Vec<_>>(),
            [6, 5, 4, 3, 2, 1]
        );
    }
}
