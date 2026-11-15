//! Device glue: cpal streams, `rtrb` rings and the analysis thread.
//!
//! Split so that everything with a decision in it is testable without a sound card:
//!
//! * `select` picks devices and stream configurations from plain lists;
//! * `pump` is what the output callback and the analysis thread do with their rings;
//! * `devices`, `output` and `input` are the cpal calls and threads that remain (excluded from
//!   the coverage gate because they need hardware, see the `Makefile`).
//!
//! The app starts the tuner with [`TunerInput::start`], plays sound through
//! [`OutputHandle::send`], and lists devices with [`list_input_devices`].

mod devices;
mod input;
mod output;
mod pump;
mod select;

pub use devices::list_input_devices;
pub use input::{InputHandle, InputInfo, TunerInput};
pub use output::OutputHandle;
pub use select::{DeviceChoice, InputDevice, MicPermission, choose_config, choose_input_device};
