//! Device enumeration and opening through cpal. Thin glue: every decision it makes is in
//! `select` (tested); what is left needs a sound card.

use cpal::traits::{DeviceTrait, HostTrait};
use cpal::{Device, Host, SupportedStreamConfig};

use super::select::{
    DeviceChoice, InputDevice, choose_config, choose_input_device, map_cpal_error,
};
use crate::error::AudioError;

fn host() -> Host {
    cpal::default_host()
}

fn describe(device: &Device, is_default: bool) -> Option<InputDevice> {
    let id = device.id().ok()?.to_string();
    let name = device
        .description()
        .map_or_else(|_| id.clone(), |d| d.name().to_owned());
    Some(InputDevice {
        id,
        name,
        is_default,
    })
}

/// Lists every input device, the system default marked. Empty when there is none.
///
/// # Errors
/// [`AudioError::DeviceUnavailable`] when the audio host cannot be queried.
pub fn list_input_devices() -> Result<Vec<InputDevice>, AudioError> {
    let host = host();
    let default_id = host.default_input_device().and_then(|d| d.id().ok());
    let devices = host
        .input_devices()
        .map_err(|e| map_cpal_error("listing input devices", &e))?;
    Ok(devices
        .filter_map(|d| {
            let is_default = default_id
                .as_ref()
                .is_some_and(|id| d.id().ok().as_ref() == Some(id));
            describe(&d, is_default)
        })
        .collect())
}

/// Opens the input device `wanted` (a saved id), or the default. Returns the device, its
/// description and whether the request had to fall back to the default (E-TU-5).
pub(crate) fn open_input_device(
    wanted: Option<&str>,
) -> Result<(Device, InputDevice, bool), AudioError> {
    let host = host();
    let default_id = host.default_input_device().and_then(|d| d.id().ok());
    let devices: Vec<Device> = host
        .input_devices()
        .map_err(|e| map_cpal_error("listing input devices", &e))?
        .collect();
    let described: Vec<(usize, InputDevice)> = devices
        .iter()
        .enumerate()
        .filter_map(|(i, d)| {
            let is_default = default_id
                .as_ref()
                .is_some_and(|id| d.id().ok().as_ref() == Some(id));
            describe(d, is_default).map(|info| (i, info))
        })
        .collect();
    let infos: Vec<InputDevice> = described.iter().map(|(_, d)| d.clone()).collect();
    let (pick, fell_back) = match choose_input_device(&infos, wanted) {
        DeviceChoice::Requested(i) | DeviceChoice::Default(i) => (i, false),
        DeviceChoice::Fallback(i) => (i, true),
        DeviceChoice::None => return Err(AudioError::NoInputDevice),
    };
    let (index, info) = described
        .into_iter()
        .nth(pick)
        .ok_or(AudioError::NoInputDevice)?;
    let device = devices
        .into_iter()
        .nth(index)
        .ok_or(AudioError::NoInputDevice)?;
    Ok((device, info, fell_back))
}

/// The default output device.
pub(crate) fn open_output_device() -> Result<Device, AudioError> {
    host()
        .default_output_device()
        .ok_or(AudioError::NoOutputDevice)
}

/// The stream configuration to use for `device` as an input.
pub(crate) fn input_config(device: &Device) -> Result<SupportedStreamConfig, AudioError> {
    let ranges: Vec<_> = device
        .supported_input_configs()
        .map_err(|e| map_cpal_error("querying input formats", &e))?
        .collect();
    choose_config(&ranges, device.default_input_config().ok())
        .ok_or_else(|| AudioError::UnsupportedConfig("no usable input format".into()))
}

/// The stream configuration to use for `device` as an output.
pub(crate) fn output_config(device: &Device) -> Result<SupportedStreamConfig, AudioError> {
    let ranges: Vec<_> = device
        .supported_output_configs()
        .map_err(|e| map_cpal_error("querying output formats", &e))?
        .collect();
    choose_config(&ranges, device.default_output_config().ok())
        .ok_or_else(|| AudioError::UnsupportedConfig("no usable output format".into()))
}
