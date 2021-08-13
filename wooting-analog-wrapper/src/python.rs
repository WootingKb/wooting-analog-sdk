use crate::{DeviceID, DeviceInfo, FromPrimitive, KeycodeType, WootingAnalogResult};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use std::collections::HashMap;

fn map_sdk_err(err: WootingAnalogResult) -> PyErr {
    PyValueError::new_err(err.to_string())
}

/// Initialises the Analog SDK, this needs to be successfully called before any other functions
/// of the SDK can be called
///
/// # Expected Returns
/// * `>=0`: Meaning the SDK initialised successfully and the number indicates the number of devices that were found on plugin initialisation
///
/// # Possible Errors
/// * `NoPlugins`: Meaning that either no plugins were found or some were found but none were successfully initialised
/// * `FunctionNotFound`: The SDK is either not installed or could not be found
/// * `IncompatibleVersion`: The installed SDK is incompatible with this wrapper as they are on different Major versions
#[pyfunction]
#[pyo3(text_signature = "(/)")]
fn initialise() -> PyResult<u32> {
    crate::initialise().0.map_err(map_sdk_err)
}

/// Returns a bool indicating if the Analog SDK has been initialised
#[pyfunction]
#[pyo3(text_signature = "(/)")]
fn is_initialised() -> bool {
    crate::is_initialised()
}

/// Uninitialises the SDK, returning it to an empty state, similar to how it would be before first initialisation
/// # Expected Returns
/// * `None`: Indicates that the SDK was successfully uninitialised
#[pyfunction]
#[pyo3(text_signature = "(/)")]
fn uninitialise() -> PyResult<()> {
    crate::uninitialise().0.map_err(map_sdk_err)
}

/// Sets the type of Keycodes the Analog SDK will receive (in `read_analog`) and output (in `read_full_buffer`).
///
/// By default, the mode is set to HID
///
/// # Notes
/// * `VirtualKey` and `VirtualKeyTranslate` are only available on Windows
/// * With all modes except `VirtualKeyTranslate`, the key identifier will point to the physical key on the standard layout. i.e. if you ask for the Q key, it will be the key right to tab regardless of the layout you have selected
/// * With `VirtualKeyTranslate`, if you request Q, it will be the key that inputs Q on the current layout, not the key that is Q on the standard layout.
///
/// # Expected Returns
/// * `None`: The Keycode mode was changed successfully
///
/// # Possible Errors
/// * `InvalidArgument`: The given `KeycodeType` is not one supported by the SDK
/// * `NotAvailable`: The given `KeycodeType` is present, but not supported on the current platform
/// * `UnInitialized`: The SDK is not initialised
#[pyfunction]
#[pyo3(text_signature = "(mode, /)")]
fn set_keycode_mode(mode: u8) -> PyResult<()> {
    if let Some(r_mode) = KeycodeType::from_u8(mode) {
        crate::set_keycode_mode(r_mode).0.map_err(map_sdk_err)
    } else {
        Err(PyValueError::new_err("Given keycode mode doesn't exist"))
    }
}

/// Reads the Analog value of the key with identifier `code` from any connected device. The set of key identifiers that is used
/// depends on the Keycode mode set using `set_mode`.
///
/// # Examples
/// ```ignore
/// set_mode(KeycodeType.ScanCode1);
/// read_analog(0x10); //This will get you the value for the key which is Q in the standard US layout (The key just right to tab)
///
/// set_mode(KeycodeType.VirtualKey); //This will only work on Windows
/// read_analog(0x51); //This will get you the value for the key that is Q on the standard layout
///
/// set_mode(KeycodeType.VirtualKeyTranslate);
/// read_analog(0x51); //This will get you the value for the key that inputs Q on the current layout
/// ```
///
/// # Expected Returns
/// * `0.0f - 1.0f`: The Analog value of the key with the given id `code`
///
/// # Possible Errors
/// * `NoMapping`: No keycode mapping was found from the selected mode (set by wooting_analog_set_mode) and HID.
/// * `UnInitialized`: The SDK is not initialised
/// * `NoDevices`: There are no connected devices
#[pyfunction]
#[pyo3(text_signature = "(code, /)")]
fn read_analog(code: u16) -> PyResult<f32> {
    crate::read_analog(code).0.map_err(map_sdk_err)
}

/// Reads the Analog value of the key with identifier `code` from the device with id `device_id`. The set of key identifiers that is used
/// depends on the Keycode mode set using `set_mode`.
///
/// The `device_id` can be found through calling `device_info` and getting the DeviceID from one of the DeviceInfo structs
///
/// # Expected Returns
/// * `0.0f - 1.0f`: The Analog value of the key with the given id `code` from device with id `device_id`
///
/// # Possible Errors
/// * `NoMapping`: No keycode mapping was found from the selected mode (set by wooting_analog_set_mode) and HID.
/// * `UnInitialized`: The SDK is not initialised
/// * `NoDevices`: There are no connected devices with id `device_id`
#[pyfunction]
#[pyo3(text_signature = "(code, device_id, /)")]
fn read_analog_device(code: u16, device_id: DeviceID) -> PyResult<f32> {
    crate::read_analog_device(code, device_id)
        .0
        .map_err(map_sdk_err)
}

/// Returns all connected devices with a max Vector return length of `max_devices` (as many that can fit in the buffer)
///
/// # Expected Returns
/// Similar to read_analog, the errors and returns are encoded into one type. Values >=0 indicate the number of items filled into the buffer, with `<0` being of type WootingAnalogResult
/// * `>=0`: The number of connected devices that have been filled into the buffer
///
/// # Possible Errors
/// * `UnInitialized`: Indicates that the Analog SDK hasn't been initialised
#[pyfunction]
#[pyo3(text_signature = "(max_devices, /)")]
fn get_connected_devices_info(max_devices: usize) -> PyResult<Vec<DeviceInfo>> {
    crate::get_connected_devices_info(max_devices)
        .0
        .map_err(map_sdk_err)
}

/// Reads all the analog values for pressed keys for all devices and combines their values, returning a HashMap of keycode -> analog value.
///
/// # Notes
/// * `max_items` is the maximum length of items that can be returned in the HashMap
/// * The keycodes returned are of the KeycodeType set with `set_mode`
/// * If two devices have the same key pressed, the greater value will be given
/// * When a key is released it will be returned with an analog value of 0.0f in the first read_full_buffer call after the key has been released
///
/// # Expected Returns
/// * `Dict[int, float]` containing pairs of keycode -> analog value
///
/// # Possible Errors
/// * `UnInitialized`: Indicates that the Analog SDK hasn't been initialised
/// * `NoDevices`: Indicates no devices are connected
#[pyfunction]
#[pyo3(text_signature = "(max_items, /)")]
fn read_full_buffer(max_items: usize) -> PyResult<HashMap<u16, f32>> {
    crate::read_full_buffer(max_items).0.map_err(map_sdk_err)
}

/// Reads all the analog values for pressed keys for the device with id `device_id`,returning a HashMap of keycode -> analog value.
///
/// # Notes
/// * `max_items` is the maximum length of items that can be returned in the HashMap
/// * The keycodes returned are of the KeycodeType set with `set_mode`
/// * When a key is released it will be returned with an analog value of 0.0f in the first read_full_buffer call after the key has been released
///
/// # Expected Returns
/// * `Dict[int, float]` containing pairs of keycode -> analog value
///
/// # Possible Errors
/// * `UnInitialized`: Indicates that the Analog SDK hasn't been initialised
/// * `NoDevices`: Indicates no devices are connected
#[pyfunction]
#[pyo3(text_signature = "(max_devices, device_id, /)")]
fn read_full_buffer_device(max_items: usize, device_id: DeviceID) -> PyResult<HashMap<u16, f32>> {
    crate::read_full_buffer_device(max_items, device_id)
        .0
        .map_err(map_sdk_err)
}

#[pymodule]
fn wooting_analog_wrapper(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(initialise, m)?)?;
    m.add_function(wrap_pyfunction!(is_initialised, m)?)?;
    m.add_function(wrap_pyfunction!(uninitialise, m)?)?;
    m.add_function(wrap_pyfunction!(set_keycode_mode, m)?)?;
    m.add_function(wrap_pyfunction!(read_analog, m)?)?;
    m.add_function(wrap_pyfunction!(read_analog_device, m)?)?;
    m.add_function(wrap_pyfunction!(get_connected_devices_info, m)?)?;
    m.add_function(wrap_pyfunction!(read_full_buffer, m)?)?;
    m.add_function(wrap_pyfunction!(read_full_buffer_device, m)?)?;

    Ok(())
}
