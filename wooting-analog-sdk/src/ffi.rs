#[cfg(feature = "dist")]
mod delegate_sys;

use crate::{
    AnalogValue, DeviceEventType, DeviceID, DeviceInfo, DeviceInfo_FFI, KeyCode, KeySource,
    KeycodeType, Position, SDKResult, ValueMetadata, WootingAnalogResult, sdk::*,
};
#[cfg(feature = "dist")]
use delegate_sys::USE_SYS_DLL;
use log::{error, trace};
use num_traits::FromPrimitive;
use std::cell::RefCell;
use std::os::raw::{c_char, c_float, c_int, c_uint, c_ushort};
use std::sync::{LazyLock, Mutex};
use std::{env, panic, slice};

static ANALOG_SDK: LazyLock<Mutex<AnalogSDK>> = LazyLock::new(|| {
    // Initialising logger with default "off".
    // If the library user wants logging, they can set the RUST_LOG environment variable, e.g. to "info".
    // TODO: Consider using file logging or allowing the user to set a custom log callback.
    if let Err(e) =
        env_logger::try_init_from_env(env_logger::Env::default().default_filter_or("off"))
    {
        println!("ERROR: Could not initialise logging. '{:?}'", e);
    }

    Mutex::new(AnalogSDK::new())
});

/// Initialises the Analog SDK, this needs to be successfully called before any other functions
/// of the SDK can be called
///
/// # Expected Returns
/// * `ret>=0`: Meaning the SDK initialised successfully and the number indicates the number of devices that were found on plugin initialisation
/// * `NoPlugins`: Meaning that either no plugins were found or some were found but none were successfully initialised
#[unsafe(no_mangle)]
pub extern "C" fn wooting_analog_initialise() -> c_int {
    #[cfg(feature = "dist")]
    if *USE_SYS_DLL {
        return delegate_sys::wooting_analog_initialise();
    }

    let result = panic::catch_unwind(|| {
        trace!("wooting_analog_initialise called");
        ANALOG_SDK.lock().unwrap().initialise().into()
    });
    trace!("catch unwind result: {:?}", result);
    match result {
        Ok(c) => c,
        Err(e) => {
            error!("An error occurred in wooting_analog_initialise: {:?}", e);
            WootingAnalogResult::Failure.into()
        }
    }
}

/// Provides the major version of the SDK, a difference in this value to what is expected indicates that
/// there may be some breaking changes that have been made so the SDK should not be attempted to be used
#[unsafe(no_mangle)]
pub extern "C" fn wooting_analog_version() -> c_int {
    #[cfg(feature = "dist")]
    if *USE_SYS_DLL {
        return delegate_sys::wooting_analog_version();
    }

    env!("CARGO_PKG_VERSION")
        .split('.')
        .collect::<Vec<&str>>()
        .first()
        .and_then(|v| v.parse().ok())
        .expect("crate must have correct package semver format")
}

/// SDK version as a static null-terminated string in SemVer format.
#[unsafe(no_mangle)]
pub extern "C" fn wooting_analog_version_semver() -> *const c_char {
    #[cfg(feature = "dist")]
    if *USE_SYS_DLL {
        return delegate_sys::wooting_analog_version_semver();
    }

    static VERSION: &str = concat!(env!("CARGO_PKG_VERSION"), "\0");
    VERSION.as_ptr() as *const c_char
}

/// Returns a bool indicating if the Analog SDK has been initialised
#[unsafe(no_mangle)]
pub extern "C" fn wooting_analog_is_initialised() -> bool {
    #[cfg(feature = "dist")]
    if *USE_SYS_DLL {
        return delegate_sys::wooting_analog_is_initialised();
    }

    ANALOG_SDK.lock().unwrap().initialised
}

/// Uninitialises the SDK, returning it to an empty state, similar to how it would be before first initialisation
/// # Expected Returns
/// * `Ok`: Indicates that the SDK was successfully uninitialised
#[unsafe(no_mangle)]
pub extern "C" fn wooting_analog_uninitialise() -> WootingAnalogResult {
    #[cfg(feature = "dist")]
    if *USE_SYS_DLL {
        return delegate_sys::wooting_analog_uninitialise();
    }

    trace!("wooting_analog_uninitialise called");
    let result = panic::catch_unwind(|| {
        //Drop the memory that was being kept for the connected devices info call
        CONNECTED_DEVICES.with(|devs| {
            let old = (*devs.borrow_mut()).take();
            if let Some(mut old_devices) = old {
                for dev in old_devices.drain(..) {
                    unsafe {
                        drop(Box::from_raw(dev));
                    }
                }
            }
        });
        ANALOG_SDK.lock().unwrap().unload();
    });

    trace!("catch unwind result {:?}", result);

    WootingAnalogResult::Ok
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
/// * `Ok`: The Keycode mode was changed successfully
/// * `InvalidArgument`: The given `KeycodeType` is not one supported by the SDK
/// * `NotAvailable`: The given `KeycodeType` is present, but not supported on the current platform
/// * `UnInitialized`: The SDK is not initialised
#[unsafe(no_mangle)]
pub extern "C" fn wooting_analog_set_keycode_mode(mode: c_uint) -> WootingAnalogResult {
    #[cfg(feature = "dist")]
    if *USE_SYS_DLL {
        return delegate_sys::wooting_analog_set_keycode_mode(mode);
    }

    if !ANALOG_SDK.lock().unwrap().initialised {
        return WootingAnalogResult::UnInitialized;
    }

    //TODO: Make it return invalid argument when attempting to use VirtualKeyTranslate on platforms other than win
    if let Some(key_mode) = KeycodeType::from_u32(mode) {
        #[cfg(not(windows))]
        {
            if key_mode == KeycodeType::VirtualKeyTranslate {
                return WootingAnalogResult::NotAvailable;
            }
        }
        ANALOG_SDK.lock().unwrap().keycode_mode = key_mode;
        WootingAnalogResult::Ok
    } else {
        WootingAnalogResult::InvalidArgument
    }
}

/// Reads the Analog value of the key with identifier `code` from any connected device. The set of key identifiers that is used
/// depends on the Keycode mode set using `wooting_analog_set_mode`.
///
/// # Examples
/// ```ignore
/// wooting_analog_set_mode(KeycodeType::ScanCode1);
/// wooting_analog_read_analog(0x10); //This will get you the value for the key which is Q in the standard US layout (The key just right to tab)
///
/// wooting_analog_set_mode(KeycodeType::VirtualKey); //This will only work on Windows
/// wooting_analog_read_analog(0x51); //This will get you the value for the key that is Q on the standard layout
///
/// wooting_analog_set_mode(KeycodeType::VirtualKeyTranslate);
/// wooting_analog_read_analog(0x51); //This will get you the value for the key that inputs Q on the current layout
/// ```
///
/// # Expected Returns
/// The float return value can be either a 0->1 analog value, or (if <0) is part of the WootingAnalogResult enum, which is how errors are given back on this call.
/// So if the value is below 0, you should cast it as WootingAnalogResult to see what the error is.
/// * `0.0f - 1.0f`: The Analog value of the key with the given id `code`
/// * `WootingAnalogResult::NoMapping`: No keycode mapping was found from the selected mode (set by wooting_analog_set_mode) and HID.
/// * `WootingAnalogResult::UnInitialized`: The SDK is not initialised
/// * `WootingAnalogResult::NoDevices`: There are no connected devices
#[unsafe(no_mangle)]
pub extern "C" fn wooting_analog_read_analog(code: c_ushort) -> c_float {
    #[cfg(feature = "dist")]
    if *USE_SYS_DLL {
        return delegate_sys::wooting_analog_read_analog(code);
    }

    wooting_analog_read_analog_device(code, 0)
}

// TODO: wooting_analog_read_analog_with_ctx_device ?
#[unsafe(no_mangle)]
pub unsafe extern "C" fn wooting_analog_read_analog_with_ctx(
    key_source: *const KeySource,
    value: *mut AnalogValue,
) -> WootingAnalogResult {
    #[cfg(feature = "dist")]
    if *USE_SYS_DLL {
        return delegate_sys::wooting_analog_read_analog_with_ctx(key_source, value);
    }

    let Some(key_source) = (unsafe { key_source.as_ref() }) else {
        return WootingAnalogResult::InvalidArgument;
    };

    match ANALOG_SDK
        .lock()
        .unwrap()
        .read_analog_with_ctx(*key_source, 0)
        .0
    {
        Ok(v) => {
            if let Some(out) = unsafe { value.as_mut() } {
                *out = v;
                return WootingAnalogResult::Ok;
            }

            WootingAnalogResult::InvalidArgument
        }
        Err(e) => e,
    }
}

/// Reads the Analog value of the key with identifier `code` from the device with id `device_id`. The set of key identifiers that is used
/// depends on the Keycode mode set using `wooting_analog_set_mode`.
///
/// The `device_id` can be found through calling `wooting_analog_device_info` and getting the DeviceID from one of the DeviceInfo structs
///
/// # Expected Returns
/// The float return value can be either a 0->1 analog value, or (if <0) is part of the WootingAnalogResult enum, which is how errors are given back on this call.
/// So if the value is below 0, you should cast it as WootingAnalogResult to see what the error is.
/// * `0.0f - 1.0f`: The Analog value of the key with the given id `code` from device with id `device_id`
/// * `WootingAnalogResult::NoMapping`: No keycode mapping was found from the selected mode (set by wooting_analog_set_mode) and HID.
/// * `WootingAnalogResult::UnInitialized`: The SDK is not initialised
/// * `WootingAnalogResult::NoDevices`: There are no connected devices with id `device_id`
#[unsafe(no_mangle)]
pub extern "C" fn wooting_analog_read_analog_device(
    code: c_ushort,
    device_id: DeviceID,
) -> c_float {
    #[cfg(feature = "dist")]
    if *USE_SYS_DLL {
        return delegate_sys::wooting_analog_read_analog_device(code, device_id);
    }

    ANALOG_SDK
        .lock()
        .unwrap()
        .read_analog(code, device_id)
        .into()
}

/// Set the callback which is called when there is a DeviceEvent. Currently these events can either be Disconnected or Connected(Currently not properly implemented).
/// The callback gets given the type of event `DeviceEventType` and a pointer to the DeviceInfo struct that the event applies to
///
/// # Notes
/// * You must copy the DeviceInfo struct or its data if you wish to use it after the callback has completed, as the memory will be freed straight after
/// * The execution of the callback is performed in a separate thread so it is fine to put time consuming code and further SDK calls inside your callback
///
/// # Expected Returns
/// * `Ok`: The callback was set successfully
/// * `UnInitialized`: The SDK is not initialised
#[unsafe(no_mangle)]
pub extern "C" fn wooting_analog_set_device_event_cb(
    cb: extern "C" fn(DeviceEventType, *mut DeviceInfo_FFI),
) -> WootingAnalogResult {
    #[cfg(feature = "dist")]
    if *USE_SYS_DLL {
        return delegate_sys::wooting_analog_set_device_event_cb(cb);
    }

    ANALOG_SDK
        .lock()
        .unwrap()
        .set_device_event_cb(move |event, device: DeviceInfo| {
            // Create pointer to the C version of Device Info to pass to the callback
            let device_box: Box<DeviceInfo_FFI> = Box::new(device.into());
            let device_raw = Box::into_raw(device_box);
            cb(event, device_raw);
            //We need to box up the pointer again to ensure it is properly dropped
            unsafe {
                drop(Box::from_raw(device_raw));
            }
        })
        .into()
}

/// Clears the device event callback that has been set
///
/// # Expected Returns
/// * `Ok`: The callback was cleared successfully
/// * `UnInitialized`: The SDK is not initialised
#[unsafe(no_mangle)]
pub extern "C" fn wooting_analog_clear_device_event_cb() -> WootingAnalogResult {
    #[cfg(feature = "dist")]
    if *USE_SYS_DLL {
        return delegate_sys::wooting_analog_clear_device_event_cb();
    }

    ANALOG_SDK.lock().unwrap().clear_device_event_cb().into()
}

thread_local!(static CONNECTED_DEVICES: RefCell<Option<Vec<*mut DeviceInfo_FFI>>> = RefCell::new(None));

/// Fills up the given `buffer`(that has length `len`) with pointers to the DeviceInfo structs for all connected devices (as many that can fit in the buffer)
///
/// # Notes
/// * The memory of the returned structs will only be kept until the next call of this function, so if you wish to use any data from them, please copy it or ensure you don't reuse references to old memory after calling this function again.
///
/// # Expected Returns
/// Similar to wooting_analog_read_analog, the errors and returns are encoded into one type. Values >=0 indicate the number of items filled into the buffer, with `<0` being of type WootingAnalogResult
/// * `ret>=0`: The number of connected devices that have been filled into the buffer
/// * `WootingAnalogResult::UnInitialized`: Indicates that the AnalogSDK hasn't been initialised
#[unsafe(no_mangle)]
pub extern "C" fn wooting_analog_get_connected_devices_info(
    buffer: *mut *mut DeviceInfo_FFI,
    len: c_uint,
) -> c_int {
    #[cfg(feature = "dist")]
    if *USE_SYS_DLL {
        return delegate_sys::wooting_analog_get_connected_devices_info(buffer, len);
    }

    let result: SDKResult<Vec<DeviceInfo>> = ANALOG_SDK.lock().unwrap().get_device_info();
    match result.0 {
        Ok(mut devices) => {
            let device_no = (len as usize).min(devices.len());

            let buff = unsafe {
                assert!(!buffer.is_null());

                slice::from_raw_parts_mut(buffer, device_no)
            };

            devices.truncate(device_no);
            // Convert all the DeviceInfo's into DeviceInfo_C pointers
            let c_devices: Vec<*mut DeviceInfo_FFI> = devices
                .drain(..)
                .map(|dev| Box::into_raw(Box::new(dev.into())))
                .collect();

            buff.swap_with_slice(c_devices.clone().as_mut());
            //We want to keep track of the structs that we've allocated and free up the last set that had been
            //given
            CONNECTED_DEVICES.with(|devs| {
                let old = (*devs.borrow_mut()).replace(c_devices);
                if let Some(mut old_devices) = old {
                    for dev in old_devices.drain(..) {
                        unsafe {
                            drop(Box::from_raw(dev));
                        }
                    }
                }
            });
            device_no as i32
        }
        Err(e) => e.into(),
    }
}

/// Reads all the analog values for pressed keys for all devices and combines their values, filling up `code_buffer` with the
/// keycode identifying the pressed key and fills up `analog_buffer` with the corresponding float analog values. i.e. The analog
/// value for they key at index 0 of code_buffer, is at index 0 of analog_buffer.
///
/// # Notes
/// * `len` is the length of code_buffer & analog_buffer, if the buffers are of unequal length, then pass the lower of the two, as it is the max amount of
/// key & analog value pairs that can be filled in.
/// * The codes that are filled into the `code_buffer` are of the KeycodeType set with wooting_analog_set_mode
/// * If two devices have the same key pressed, the greater value will be given
/// * When a key is released it will be returned with an analog value of 0.0f in the first read_full_buffer call after the key has been released
///
/// # Expected Returns
/// Similar to other functions like `wooting_analog_device_info`, the return value encodes both errors and the return value we want.
/// Where >=0 is the actual return, and <0 should be cast as WootingAnalogResult to find the error.
/// * `>=0` means the value indicates how many keys & analog values have been read into the buffers
/// * `WootingAnalogResult::UnInitialized`: Indicates that the AnalogSDK hasn't been initialised
/// * `WootingAnalogResult::NoDevices`: Indicates no devices are connected
#[unsafe(no_mangle)]
pub extern "C" fn wooting_analog_read_full_buffer(
    code_buffer: *mut c_ushort,
    analog_buffer: *mut c_float,
    len: c_uint,
) -> c_int {
    #[cfg(feature = "dist")]
    if *USE_SYS_DLL {
        return delegate_sys::wooting_analog_read_full_buffer_device(
            code_buffer,
            analog_buffer,
            len,
            0,
        );
    }

    wooting_analog_read_full_buffer_device(code_buffer, analog_buffer, len, 0)
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wooting_analog_read_full_buffer_with_ctx(
    code_buffer: *mut KeyCode,
    analog_buffer: *mut AnalogValue,
    len: c_uint,
) -> c_int {
    #[cfg(feature = "dist")]
    if *USE_SYS_DLL {
        return delegate_sys::wooting_analog_read_full_buffer_with_ctx_device(
            code_buffer,
            analog_buffer,
            len,
            0,
        );
    }

    unsafe { wooting_analog_read_full_buffer_with_ctx_device(code_buffer, analog_buffer, len, 0) }
}

/// Reads all the analog values for pressed keys for the device with id `device_id`, filling up `code_buffer` with the
/// keycode identifying the pressed key and fills up `analog_buffer` with the corresponding float analog values. i.e. The analog
/// value for they key at index 0 of code_buffer, is at index 0 of analog_buffer.
///
/// # Notes
/// * `len` is the length of code_buffer & analog_buffer, if the buffers are of unequal length, then pass the lower of the two, as it is the max amount of
/// key & analog value pairs that can be filled in.
/// * The codes that are filled into the `code_buffer` are of the KeycodeType set with wooting_analog_set_mode
/// * When a key is released it will be returned with an analog value of 0.0f in the first read_full_buffer call after the key has been released
///
/// # Expected Returns
/// Similar to other functions like `wooting_analog_device_info`, the return value encodes both errors and the return value we want.
/// Where >=0 is the actual return, and <0 should be cast as WootingAnalogResult to find the error.
/// * `>=0` means the value indicates how many keys & analog values have been read into the buffers
/// * `WootingAnalogResult::UnInitialized`: Indicates that the AnalogSDK hasn't been initialised
/// * `WootingAnalogResult::NoDevices`: Indicates the device with id `device_id` is not connected
#[unsafe(no_mangle)]
pub extern "C" fn wooting_analog_read_full_buffer_device(
    code_buffer: *mut c_ushort,
    analog_buffer: *mut c_float,
    len: c_uint,
    device_id: DeviceID,
) -> c_int {
    #[cfg(feature = "dist")]
    if *USE_SYS_DLL {
        return delegate_sys::wooting_analog_read_full_buffer_device(
            code_buffer,
            analog_buffer,
            len,
            device_id,
        );
    }

    let codes = unsafe {
        assert!(!code_buffer.is_null());

        slice::from_raw_parts_mut(code_buffer, len as usize)
    };

    let analog = unsafe {
        assert!(!analog_buffer.is_null());

        slice::from_raw_parts_mut(analog_buffer, len as usize)
    };

    match ANALOG_SDK
        .lock()
        .unwrap()
        .read_full_buffer(len as usize, device_id)
        .0
    {
        Ok(analog_data) => {
            //Fill up given slices
            let mut count: usize = 0;
            for (code, val) in analog_data.iter() {
                if count >= codes.len() {
                    break;
                }

                codes[count] = *code;
                analog[count] = *val;
                count += 1;
            }
            count as c_int
        }
        Err(e) => e as c_int,
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn wooting_analog_read_full_buffer_with_ctx_device(
    code_buffer: *mut KeyCode,
    analog_buffer: *mut AnalogValue,
    len: c_uint,
    device_id: DeviceID,
) -> c_int {
    #[cfg(feature = "dist")]
    if *USE_SYS_DLL {
        return delegate_sys::wooting_analog_read_full_buffer_with_ctx_device(
            code_buffer,
            analog_buffer,
            len,
            device_id,
        );
    }

    let codes = unsafe {
        assert!(!code_buffer.is_null());

        slice::from_raw_parts_mut(code_buffer, len as usize)
    };

    let analog = unsafe {
        assert!(!analog_buffer.is_null());

        slice::from_raw_parts_mut(analog_buffer, len as usize)
    };

    match ANALOG_SDK
        .lock()
        .unwrap()
        .read_full_buffer_with_ctx(len as usize, device_id)
        .0
    {
        Ok(analog_data) => {
            //Fill up given slices
            let mut count: usize = 0;
            for (code, val) in analog_data.iter() {
                if count >= codes.len() {
                    break;
                }

                codes[count] = *code;
                analog[count] = *val;
                count += 1;
            }
            count as c_int
        }
        Err(e) => e as c_int,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn wooting_analog_using_sys() -> bool {
    #[cfg(feature = "dist")]
    {
        *USE_SYS_DLL
    }

    #[cfg(not(feature = "dist"))]
    {
        true
    }
}
