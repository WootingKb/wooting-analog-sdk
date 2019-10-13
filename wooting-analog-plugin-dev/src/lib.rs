extern crate ffi_support;
pub extern crate wooting_analog_common;

use ffi_support::FfiStr;
use std::collections::HashMap;
use std::hash::Hasher;
use std::os::raw::{c_float, c_ushort};
use wooting_analog_common::*;

/// Version number of the plugin ABI which is exported in plugins so the SDK can determine how to handle the plugin based on which ABI version it's on
#[no_mangle]
pub static ANALOG_SDK_PLUGIN_ABI_VERSION: u32 = 1;

/// The core Plugin trait which needs to be implemented for an Analog Plugin to function
pub trait Plugin {
    /// Get a name describing the `Plugin`.
    fn name(&mut self) -> SDKResult<&'static str>;

    /// Initialise the plugin with the given function for device events. Returns an int indicating the number of connected devices
    fn initialise(&mut self, callback: extern "C" fn(DeviceEventType, DeviceInfoPointer)) -> SDKResult<i32>;

    /// A function fired to check if the plugin is currently initialised
    fn is_initialised(&mut self) -> bool;

    /// This function is fired by the SDK to collect up all Device Info structs. The memory for the struct should be retained and only dropped
    /// when the device is disconnected or the plugin is unloaded. This ensures that the Device Info is not garbled when it's being accessed by the client.
    ///
    /// # Notes
    ///
    /// Although, the client should be copying any data they want to use for a prolonged time as there is no lifetime guarantee on the data.
    fn device_info(&mut self, buffer: &mut [DeviceInfoPointer]) -> SDKResult<i32>;

    /// A callback fired immediately before the plugin is unloaded. Use this if
    /// you need to do any cleanup.
    fn unload(&mut self) {}

    /// Function called to get the analog value for a particular HID key `code` from the device with ID `device`.
    /// If `device` is 0 then no specific device is specified and the value should be read from all devices and combined
    fn read_analog(&mut self, code: u16, device: DeviceID) -> SDKResult<f32>;

    /// Function called to get the full analog read buffer for a particular device with ID `device`. `max_length` is the maximum amount
    /// of keys that can be accepted, any more beyond this will be ignored by the SDK.
    /// If `device` is 0 then no specific device is specified and the data should be read from all devices and combined
    fn read_full_buffer(
        &mut self,
        max_length: usize,
        device: DeviceID,
    ) -> SDKResult<HashMap<c_ushort, c_float>>;
}

/// Declare a plugin type and its constructor.
///
/// # Notes
///
/// This works by automatically generating an `extern "C"` function with a
/// pre-defined signature and symbol name. Therefore you will only be able to
/// declare one plugin per library.
#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $constructor:path) => {
        #[no_mangle]
        pub extern "C" fn _plugin_create() -> *mut $crate::Plugin {
            // make sure the constructor is the correct type.
            let constructor: fn() -> $plugin_type = $constructor;

            let object = constructor();
            let boxed: Box<$crate::Plugin> = Box::new(object);
            Box::into_raw(boxed)
        }
    };
}

pub fn generate_device_id(serial_number: &str, vendor_id: u16, product_id: u16) -> DeviceID {
    use std::collections::hash_map::DefaultHasher;
    let mut s = DefaultHasher::new();
    s.write_u16(vendor_id);
    s.write_u16(product_id);
    s.write(serial_number.as_bytes());
    s.finish()
}

mod ffi {
    use super::*;

    #[no_mangle]
    pub extern "C" fn generate_device_id(
        serial_number: FfiStr,
        vendor_id: u16,
        product_id: u16,
    ) -> DeviceID {
        let serial = {
            if let Some(str) = serial_number.into_opt_string() {
                str
            } else {
                return 0;
            }
        };
        super::generate_device_id(&serial, vendor_id, product_id)
    }
}
