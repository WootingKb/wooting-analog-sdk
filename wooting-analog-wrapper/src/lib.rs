#[macro_use]
extern crate lazy_static;
extern crate wooting_analog_common;

pub use wooting_analog_common::*;
pub mod ffi;
use ffi::*;
use std::collections::HashMap;
use std::os::raw::c_uint;
use std::ptr;

pub(crate) const SDK_ABI_VERSION: u32 = 0;

/// Provides the major version of the SDK, a difference in this value to what is expected (SDK_ABI_VERSION) indicates that
/// there may be some breaking changes that have been made so the SDK should not be attempted to be used
pub fn version() -> SDKResult<u32> {
    return unsafe { wooting_analog_version().into() };
}

/// Initialises the Analog SDK, this needs to be successfully called before any other functions
/// of the SDK can be called
///
/// # Expected Returns
/// * `Ok(>=0)`: Meaning the SDK initialised successfully and the number indicates the number of devices that were found on plugin initialisation
/// * `Err(NoPlugins)`: Meaning that either no plugins were found or some were found but none were successfully initialised
/// * `Err(FunctionNotFound)`: The SDK is either not installed or could not be found
/// * `Err(IncompatibleVersion)`: The installed SDK is incompatible with this wrapper as they are on different Major versions
pub fn initialise() -> SDKResult<u32> {
    return unsafe { wooting_analog_initialise().into() };
}

/// Returns a bool indicating if the Analog SDK has been initialised
pub fn is_initialised() -> bool {
    return unsafe { wooting_analog_is_initialised() };
}

/// Uninitialises the SDK, returning it to an empty state, similar to how it would be before first initialisation
/// # Expected Returns
/// * `Ok(())`: Indicates that the SDK was successfully uninitialised
pub fn uninitialise() -> SDKResult<()> {
    return unsafe { wooting_analog_uninitialise().into() };
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
/// * `Ok(())`: The Keycode mode was changed successfully
/// * `Err(InvalidArgument)`: The given `KeycodeType` is not one supported by the SDK
/// * `Err(NotAvailable)`: The given `KeycodeType` is present, but not supported on the current platform
/// * `Err(UnInitialized)`: The SDK is not initialised
pub fn set_keycode_mode(mode: KeycodeType) -> SDKResult<()> {
    return unsafe { wooting_analog_set_keycode_mode(mode).into() };
}

/// Reads the Analog value of the key with identifier `code` from any connected device. The set of key identifiers that is used
/// depends on the Keycode mode set using `wooting_analog_set_mode`.
///
/// # Examples
/// ```ignore
/// set_mode(KeycodeType::ScanCode1);
/// read_analog(0x10); //This will get you the value for the key which is Q in the standard US layout (The key just right to tab)
///
/// set_mode(KeycodeType::VirtualKey); //This will only work on Windows
/// read_analog(0x51); //This will get you the value for the key that is Q on the standard layout
///
/// set_mode(KeycodeType::VirtualKeyTranslate);
/// read_analog(0x51); //This will get you the value for the key that inputs Q on the current layout
/// ```
///
/// # Expected Returns
/// * `Ok(0.0f - 1.0f)`: The Analog value of the key with the given id `code`
/// * `Err(NoMapping)`: No keycode mapping was found from the selected mode (set by wooting_analog_set_mode) and HID.
/// * `Err(UnInitialized)`: The SDK is not initialised
/// * `Err(NoDevices)`: There are no connected devices
pub fn read_analog(code: u16) -> SDKResult<f32> {
    return unsafe { wooting_analog_read_analog(code).into() };
}

/// Reads the Analog value of the key with identifier `code` from the device with id `device_id`. The set of key identifiers that is used
/// depends on the Keycode mode set using `wooting_analog_set_mode`.
///
/// The `device_id` can be found through calling `device_info` and getting the DeviceID from one of the DeviceInfo structs
///
/// # Expected Returns
/// * `Ok(0.0f - 1.0f)`: The Analog value of the key with the given id `code` from device with id `device_id`
/// * `Err(NoMapping)`: No keycode mapping was found from the selected mode (set by wooting_analog_set_mode) and HID.
/// * `Err(UnInitialized)`: The SDK is not initialised
/// * `Err(NoDevices)`: There are no connected devices with id `device_id`
pub fn read_analog_device(code: u16, device_id: DeviceID) -> SDKResult<f32> {
    return unsafe { wooting_analog_read_analog_device(code, device_id).into() };
}

/// Set the callback which is called when there is a DeviceEvent. Currently these events can either be Disconnected or Connected(Currently not properly implemented).
/// The callback gets given the type of event `DeviceEventType` and a pointer to the DeviceInfo struct that the event applies to
///
/// # Notes
/// * You must copy the DeviceInfo struct or its data if you wish to use it after the callback has completed, as the memory will be freed straight after
/// * The execution of the callback is performed in a separate thread so it is fine to put time consuming code and further SDK calls inside your callback
///
/// # Expected Returns
/// * `Ok(())`: The callback was set successfully
/// * `Err(UnInitialized)`: The SDK is not initialised
pub fn set_device_event_cb(
    cb: extern "C" fn(DeviceEventType, *mut DeviceInfo_FFI), //TODO: Make this accept a closure
) -> SDKResult<()> {
    return unsafe { wooting_analog_set_device_event_cb(cb).into() };
}

/// Clears the device event callback that has been set
///
/// # Expected Returns
/// * `Ok(())`: The callback was cleared successfully
/// * `Err(UnInitialized)`: The SDK is not initialised
pub fn clear_device_event_cb() -> SDKResult<()> {
    return unsafe { wooting_analog_clear_device_event_cb().into() };
}

/// Returns all connected devices with a max Vector return length of `max_devices` (as many that can fit in the buffer)
///
/// # Notes
/// * The memory of the returned structs will only be kept until the next call of `get_connected_devices_info`, so if you wish to use any data from them, please copy it or ensure you don't reuse references to old memory after calling `get_connected_devices_info` again.
///
/// # Expected Returns
/// Similar to wooting_analog_read_analog, the errors and returns are encoded into one type. Values >=0 indicate the number of items filled into the buffer, with `<0` being of type WootingAnalogResult
/// * `Ok(>=0)`: The number of connected devices that have been filled into the buffer
/// * `Err(UnInitialized)`: Indicates that the AnalogSDK hasn't been initialised
pub fn get_connected_devices_info(max_devices: usize) -> SDKResult<Vec<DeviceInfo>> {
    unsafe {
        let mut buffer: Vec<*mut DeviceInfo_FFI> = vec![ptr::null_mut(); max_devices];

        let ret: SDKResult<u32> =
            wooting_analog_get_connected_devices_info(buffer.as_mut_ptr(), max_devices as c_uint)
                .into();

        return ret
            .clone()
            .map(|device_num| {
                buffer.truncate(device_num as usize);
                buffer
                    .drain(..)
                    .map(|device_ptr| {
                        let boxed = device_ptr.as_ref().unwrap();
                        // Here we only want to make a copy of the DeviceInfo_FFI into a DeviceInfo
                        // As the SDK side will take care of the memory management, so we need to put it back into raw so it doesn't get dropped here
                        boxed.into_device_info()
                    })
                    .collect()
            })
            .into();
    }
}

/// Reads all the analog values for pressed keys for the device with id `device_id`,returning a HashMap of keycode -> analog value.
///
/// # Notes
/// * `max_items` is the maximum length of items that can be returned in the HashMap
/// * The keycodes returned are of the KeycodeType set with `set_mode`
/// * When a key is released it will be returned with an analog value of 0.0f in the first read_full_buffer call after the key has been released
///
/// # Expected Returns
/// * `Ok(HashMap)`
/// * `Err(UnInitialized)`: Indicates that the AnalogSDK hasn't been initialised
/// * `Err(NoDevices)`: Indicates no devices are connected
pub fn read_full_buffer_device(
    max_items: usize,
    device_id: DeviceID,
) -> SDKResult<HashMap<u16, f32>> {
    unsafe {
        let mut code_buffer: Vec<u16> = vec![0; max_items];
        let mut analog_buffer: Vec<f32> = vec![0.0; max_items];

        let ret: SDKResult<u32> = wooting_analog_read_full_buffer_device(
            code_buffer.as_mut_ptr(),
            analog_buffer.as_mut_ptr(),
            max_items as u32,
            device_id,
        )
        .into();

        return ret
            .clone()
            .map(|read_num| {
                let read_num: usize = read_num as usize;
                code_buffer.truncate(read_num);
                analog_buffer.truncate(read_num);
                let mut data: HashMap<u16, f32> = HashMap::with_capacity(read_num);

                for i in 0..read_num {
                    data.insert(code_buffer[i], analog_buffer[i]);
                }
                data
            })
            .into();
    }
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
/// * `Ok(HashMap)`
/// * `Err(UnInitialized)`: Indicates that the AnalogSDK hasn't been initialised
/// * `Err(NoDevices)`: Indicates no devices are connected
pub fn read_full_buffer(max_items: usize) -> SDKResult<HashMap<u16, f32>> {
    return read_full_buffer_device(max_items, 0);
}