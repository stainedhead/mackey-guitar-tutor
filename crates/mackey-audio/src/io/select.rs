//! Device and stream-configuration choices, as pure functions over plain lists so they are
//! tested without a sound card.

use cpal::{
    ErrorKind, SAMPLE_RATE_48K, SAMPLE_RATE_CD, SampleFormat, SupportedStreamConfig,
    SupportedStreamConfigRange,
};
use serde::{Deserialize, Serialize};

use crate::error::AudioError;

/// An audio input device as the settings screen lists it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct InputDevice {
    /// Stable identifier (cpal's device id string, host and device); save this in settings.
    pub id: String,
    /// Name to show.
    pub name: String,
    /// True for the system default input.
    pub is_default: bool,
}

/// Whether the app may use the microphone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum MicPermission {
    /// Not asked yet or not known.
    Unknown,
    /// Audio is flowing.
    Granted,
    /// The OS refused (or returns only silence).
    Denied,
}

/// Which listed device to open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceChoice {
    /// The device that was asked for, at this index.
    Requested(usize),
    /// Nothing was asked for: the system default, at this index.
    Default(usize),
    /// The device asked for is gone: the system default (or the first device) instead
    /// (E-TU-5).
    Fallback(usize),
    /// There is no input device at all.
    None,
}

/// Picks a device from `devices`: the one with id `wanted`, else the default, else the first.
///
/// ```
/// use mackey_audio::io::{choose_input_device, DeviceChoice, InputDevice};
/// let d = |id: &str, def| InputDevice { id: id.into(), name: id.into(), is_default: def };
/// let list = [d("a", false), d("b", true)];
/// assert_eq!(choose_input_device(&list, Some("a")), DeviceChoice::Requested(0));
/// assert_eq!(choose_input_device(&list, Some("gone")), DeviceChoice::Fallback(1));
/// assert_eq!(choose_input_device(&[], None), DeviceChoice::None);
/// ```
#[must_use]
pub fn choose_input_device(devices: &[InputDevice], wanted: Option<&str>) -> DeviceChoice {
    if devices.is_empty() {
        return DeviceChoice::None;
    }
    let fallback = devices.iter().position(|d| d.is_default).unwrap_or(0);
    match wanted {
        None => DeviceChoice::Default(fallback),
        Some(id) => devices
            .iter()
            .position(|d| d.id == id)
            .map_or(DeviceChoice::Fallback(fallback), DeviceChoice::Requested),
    }
}

fn format_rank(f: SampleFormat) -> Option<u8> {
    match f {
        SampleFormat::F32 => Some(0),
        SampleFormat::I16 => Some(1),
        SampleFormat::I32 => Some(2),
        SampleFormat::U16 => Some(3),
        _ => None,
    }
}

/// Chooses a stream configuration: 48 kHz, then 44.1 kHz, in the best sample format the glue
/// converts (f32, i16, i32, u16), then the fewest channels; else the device default when its
/// format is usable.
#[must_use]
pub fn choose_config(
    ranges: &[SupportedStreamConfigRange],
    default: Option<SupportedStreamConfig>,
) -> Option<SupportedStreamConfig> {
    let best = ranges
        .iter()
        .filter_map(|r| {
            let rank = format_rank(r.sample_format())?;
            let (order, rate) = [SAMPLE_RATE_48K, SAMPLE_RATE_CD]
                .into_iter()
                .enumerate()
                .find(|&(_, rate)| r.contains_rate(rate))?;
            Some(((order, rank, r.channels()), r.try_with_sample_rate(rate)?))
        })
        .min_by_key(|(key, _)| *key)
        .map(|(_, config)| config);
    best.or_else(|| default.filter(|d| format_rank(d.sample_format()).is_some()))
}

/// Turns a cpal error into ours, keeping the backend message and naming the operation.
pub(crate) fn map_cpal_error(op: &str, error: &cpal::Error) -> AudioError {
    let text = format!("{op}: {error}");
    match error.kind() {
        ErrorKind::PermissionDenied => AudioError::PermissionDenied,
        ErrorKind::DeviceNotAvailable | ErrorKind::DeviceBusy | ErrorKind::HostUnavailable => {
            AudioError::DeviceUnavailable(text)
        }
        ErrorKind::UnsupportedConfig | ErrorKind::InvalidInput => {
            AudioError::UnsupportedConfig(text)
        }
        _ => AudioError::StreamBuild(text),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cpal::SupportedBufferSize::Unknown;

    fn dev(id: &str, is_default: bool) -> InputDevice {
        InputDevice {
            id: id.into(),
            name: id.to_uppercase(),
            is_default,
        }
    }

    fn range(ch: u16, lo: u32, hi: u32, f: SampleFormat) -> SupportedStreamConfigRange {
        SupportedStreamConfigRange::new(ch, lo, hi, Unknown, f)
    }

    #[test]
    fn device_choice_table() {
        let list = [dev("a", false), dev("b", true), dev("c", false)];
        // (label, wanted, expected)
        let cases: &[(&str, Option<&str>, DeviceChoice)] = &[
            ("default", None, DeviceChoice::Default(1)),
            ("requested", Some("c"), DeviceChoice::Requested(2)),
            ("gone", Some("zzz"), DeviceChoice::Fallback(1)),
        ];
        for (label, wanted, want) in cases {
            assert_eq!(choose_input_device(&list, *wanted), *want, "{label}");
        }
        let no_default = [dev("a", false), dev("b", false)];
        assert_eq!(
            choose_input_device(&no_default, None),
            DeviceChoice::Default(0)
        );
        assert_eq!(choose_input_device(&[], Some("a")), DeviceChoice::None);
    }

    #[test]
    fn config_prefers_48k_f32_then_fewer_channels() {
        let ranges = [
            range(2, 8_000, 44_100, SampleFormat::F32),
            range(2, 8_000, 96_000, SampleFormat::I16),
            range(1, 8_000, 96_000, SampleFormat::I16),
            range(2, 8_000, 96_000, SampleFormat::F32),
        ];
        let c = choose_config(&ranges, None).unwrap();
        assert_eq!(
            (c.sample_rate(), c.sample_format(), c.channels()),
            (48_000, SampleFormat::F32, 2)
        );
        // Only 44.1 kHz available.
        let c = choose_config(&ranges[..1], None).unwrap();
        assert_eq!(c.sample_rate(), 44_100);
        // Fewest channels wins among equals.
        let c = choose_config(&ranges[1..3], None).unwrap();
        assert_eq!((c.channels(), c.sample_format()), (1, SampleFormat::I16));
    }

    #[test]
    fn config_falls_back_to_the_default_or_nothing() {
        let odd = [
            range(2, 8_000, 16_000, SampleFormat::F32),
            range(2, 44_100, 48_000, SampleFormat::F64),
        ];
        assert!(choose_config(&odd, None).is_none());
        let default = SupportedStreamConfig::new(2, 32_000, Unknown, SampleFormat::F32);
        assert_eq!(
            choose_config(&odd, Some(default)).unwrap().sample_rate(),
            32_000
        );
        let bad = SupportedStreamConfig::new(2, 48_000, Unknown, SampleFormat::F64);
        assert!(choose_config(&[], Some(bad)).is_none());
    }

    #[test]
    fn cpal_errors_map_to_ours() {
        let cases = [
            (ErrorKind::PermissionDenied, AudioError::PermissionDenied),
            (
                ErrorKind::DeviceNotAvailable,
                AudioError::DeviceUnavailable("open: x".into()),
            ),
            (
                ErrorKind::UnsupportedConfig,
                AudioError::UnsupportedConfig("open: x".into()),
            ),
            (ErrorKind::Xrun, AudioError::StreamBuild("open: x".into())),
        ];
        for (kind, want) in cases {
            let e = cpal::Error::with_message(kind, "x");
            assert_eq!(map_cpal_error("open", &e), want, "{kind:?}");
        }
    }
}
