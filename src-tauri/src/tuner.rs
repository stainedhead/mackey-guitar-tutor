//! Tuner glue that is not audio: the emission throttle (TU: about 30 Hz to the UI).

use mackey_audio::dsp::TunerReading;

/// Minimum spacing of emitted readings, in stream milliseconds (about 30 Hz).
pub const EMIT_INTERVAL_MS: f64 = 30.0;

/// Decides which readings reach the UI. The analysis thread produces about 94 per second;
/// the UI gets the latest one every 30 ms, measured on the stream clock **and** the wall clock
/// (so a device that delivers faster than real time cannot flood the UI, FR-002). Readings that
/// carry a notice and readings whose status changed pass at once, so a state change is never
/// delayed; the analysis loop's pace guard bounds how often those can happen.
#[derive(Debug, Default)]
pub struct Throttle {
    stream_ms: Option<f64>,
    wall_ms_seen: Option<f64>,
    status_seen: Option<mackey_audio::dsp::TunerStatus>,
}

impl Throttle {
    /// True when `reading` should be emitted. `wall_ms` is the wall clock in milliseconds
    /// (any epoch), injected so tests are deterministic.
    pub fn accept(&mut self, reading: &TunerReading, wall_ms: f64) -> bool {
        let stream_due = self
            .stream_ms
            .is_none_or(|last| reading.t_ms - last >= EMIT_INTERVAL_MS || reading.t_ms < last);
        let wall_due = self
            .wall_ms_seen
            .is_none_or(|last| wall_ms - last >= EMIT_INTERVAL_MS || wall_ms < last);
        let changed = self.status_seen != Some(reading.status);
        if (stream_due && wall_due) || changed || reading.notice.is_some() {
            self.stream_ms = Some(reading.t_ms);
            self.wall_ms_seen = Some(wall_ms);
            self.status_seen = Some(reading.status);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mackey_audio::dsp::{TunerNotice, TunerStatus};

    fn reading(t_ms: f64, status: TunerStatus) -> TunerReading {
        let mut r = TunerReading::notice_only(TunerNotice::DeviceChanged, t_ms);
        r.notice = None;
        r.status = status;
        r
    }

    /// 94 Hz input (one reading every 10.6 ms) comes out at about 30 Hz.
    #[test]
    fn rate_is_about_thirty_per_second() {
        let mut t = Throttle::default();
        let accepted = (0..940)
            .filter(|i| {
                let ms = f64::from(*i) * 1000.0 / 94.0;
                t.accept(&reading(ms, TunerStatus::Pitch), ms)
            })
            .count();
        assert!(
            (290..=330).contains(&accepted),
            "{accepted} per 10 s of input"
        );
    }

    #[test]
    fn status_changes_and_notices_always_pass() {
        let mut t = Throttle::default();
        assert!(t.accept(&reading(0.0, TunerStatus::Listening), 0.0));
        assert!(!t.accept(&reading(5.0, TunerStatus::Listening), 5.0));
        assert!(t.accept(&reading(6.0, TunerStatus::Pitch), 6.0));
        let notice = TunerReading::notice_only(TunerNotice::AnalysisStopped, 7.0);
        assert!(t.accept(&notice, 7.0));
    }

    #[test]
    fn a_restarted_clock_passes() {
        let mut t = Throttle::default();
        assert!(t.accept(&reading(5000.0, TunerStatus::Pitch), 0.0));
        assert!(t.accept(&reading(10.0, TunerStatus::Pitch), 40.0));
    }

    /// FR-002: a source at 50x real time (readings 0.21 ms apart on the wall clock) is held
    /// to about 30 per wall second.
    #[test]
    fn fr002_a_fast_source_is_limited_in_wall_clock_time() {
        let mut t = Throttle::default();
        let accepted = (0..4700)
            .filter(|i| {
                let stream_ms = f64::from(*i) * 1000.0 / 94.0;
                let wall_ms = stream_ms / 50.0;
                t.accept(&reading(stream_ms, TunerStatus::Pitch), wall_ms)
            })
            .count();
        assert!(accepted <= 35, "{accepted} emitted in one wall second");
    }
}
