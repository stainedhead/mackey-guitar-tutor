//! Tuner glue that is not audio: the emission throttle (TU: about 30 Hz to the UI).

use mackey_audio::dsp::TunerReading;

/// Minimum spacing of emitted readings, in stream milliseconds (about 30 Hz).
pub const EMIT_INTERVAL_MS: f64 = 30.0;

/// Decides which readings reach the UI. The analysis thread produces about 94 per second;
/// the UI gets the latest one every 30 ms of stream time (about every third reading). Readings that carry a notice and
/// readings whose status changed always pass, so a state change is never delayed.
#[derive(Debug, Default)]
pub struct Throttle {
    last_t_ms: Option<f64>,
    last_status: Option<mackey_audio::dsp::TunerStatus>,
}

impl Throttle {
    /// True when `reading` should be emitted.
    pub fn accept(&mut self, reading: &TunerReading) -> bool {
        let due = self
            .last_t_ms
            .is_none_or(|last| reading.t_ms - last >= EMIT_INTERVAL_MS || reading.t_ms < last);
        let changed = self.last_status != Some(reading.status);
        if due || changed || reading.notice.is_some() {
            self.last_t_ms = Some(reading.t_ms);
            self.last_status = Some(reading.status);
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
            .filter(|i| t.accept(&reading(f64::from(*i) * 1000.0 / 94.0, TunerStatus::Pitch)))
            .count();
        assert!(
            (290..=330).contains(&accepted),
            "{accepted} per 10 s of input"
        );
    }

    #[test]
    fn status_changes_and_notices_always_pass() {
        let mut t = Throttle::default();
        assert!(t.accept(&reading(0.0, TunerStatus::Listening)));
        assert!(!t.accept(&reading(5.0, TunerStatus::Listening)));
        assert!(t.accept(&reading(6.0, TunerStatus::Pitch)));
        let notice = TunerReading::notice_only(TunerNotice::AnalysisStopped, 7.0);
        assert!(t.accept(&notice));
    }

    #[test]
    fn a_restarted_clock_passes() {
        let mut t = Throttle::default();
        assert!(t.accept(&reading(5000.0, TunerStatus::Pitch)));
        assert!(t.accept(&reading(10.0, TunerStatus::Pitch)));
    }
}
