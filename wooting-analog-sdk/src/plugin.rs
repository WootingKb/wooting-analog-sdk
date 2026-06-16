pub(crate) mod dynamic;
pub(crate) mod wooting;

use std::collections::HashMap;
use std::os::raw::{c_float, c_ushort};

use crate::device::{DeviceEventType, DeviceInfo};
use crate::err::{PluginError, ReadError};
use crate::{AnalogValue, device::DeviceID, KeyCode, KeyPosition, PhysicalKey};

#[cfg(target_os = "macos")]
pub const DEFAULT_PLUGIN_DIR: &str = "/usr/local/share/WootingAnalogPlugins";
#[cfg(target_os = "linux")]
pub const DEFAULT_PLUGIN_DIR: &str = "/usr/local/share/WootingAnalogPlugins";
#[cfg(target_os = "windows")]
pub const DEFAULT_PLUGIN_DIR: &str = "C:\\Program Files\\WootingAnalogPlugins";

pub static ANALOG_SDK_PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The core Plugin trait which needs to be implemented for an Analog Plugin to function
pub trait Plugin: Send + Sync {
    /// Get a name describing the `Plugin`.
    fn name(&mut self) -> Result<&'static str, PluginError>;

    /// Initialise the plugin with the given function for device events. Returns an int indicating the number of connected devices
    fn initialise(
        &mut self,
        callback: Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send + Sync>,
    ) -> Result<u32, ReadError>;

    /// A function fired to check if the plugin is currently initialised
    fn is_initialised(&mut self) -> bool;

    /// This function is fired by the SDK to collect up all Device Info structs. The memory for the struct should be retained and only dropped
    /// when the device is disconnected or the plugin is unloaded. This ensures that the Device Info is not garbled when it's being accessed by the client.
    ///
    /// # Notes
    ///
    /// Although, the client should be copying any data they want to use for a prolonged time as there is no lifetime guarantee on the data.
    fn device_info(&mut self) -> Result<Vec<DeviceInfo>, ReadError>;

    /// A callback fired immediately before the plugin is unloaded. Use this if
    /// you need to do any cleanup.
    fn unload(&mut self) {}

    /// Function called to get the analog value for a particular HID key `code` from the device with ID `device`.
    /// If `device` is 0 then no specific device is specified and the value should be read from all devices and combined
    fn read_analog(&mut self, code: u16, device_id: DeviceID) -> Result<f32, ReadError>;

    fn read_keycode(
        &mut self,
        code: KeyCode,
        device_id: DeviceID,
    ) -> Result<AnalogValue, ReadError>;

    fn read_position(
        &mut self,
        position: KeyPosition,
        device_id: DeviceID,
    ) -> Result<PhysicalKey, ReadError>;

    /// Function called to get the full analog read buffer for a particular device with ID `device`. `max_length` is the maximum amount
    /// of keys that can be accepted, any more beyond this will be ignored by the SDK.
    /// If `device` is 0 then no specific device is specified and the data should be read from all devices and combined
    fn read_full_buffer(
        &mut self,
        device: DeviceID,
    ) -> Result<HashMap<c_ushort, c_float>, ReadError>;

    fn read_keycodes(
        &mut self,
        device_id: DeviceID,
    ) -> Result<HashMap<KeyCode, AnalogValue>, ReadError>;

    fn read_positions(
        &mut self,
        device_id: DeviceID,
    ) -> Result<HashMap<KeyPosition, PhysicalKey>, ReadError>;
}
