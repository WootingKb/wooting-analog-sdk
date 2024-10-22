use crate::sdk::*;
use std::cell::RefCell;
use std::os::raw::{c_float, c_int, c_uint, c_ushort};
use std::sync::Mutex;
use std::{panic, slice};
use wooting_analog_common::FromPrimitive;
use wooting_analog_common::*;

lazy_static! {
    pub static ref ANALOG_SDK: Mutex<AnalogSDK> = {
        // Initialising logger with default "off".
        // If the library user wants logging, they can set the RUST_LOG environment variable, e.g. to "info".
        // TODO: Consider using file logging or allowing the user to set a custom log callback.
        if let Err(e) = env_logger::try_init_from_env(env_logger::Env::default().default_filter_or("off")){
            println!("ERROR: Could not initialise logging. '{:?}'", e);
        }

        Mutex::new(AnalogSDK::new())
    };
}

/// Initialises the Analog SDK, this needs to be successfully called before any other functions
/// of the SDK can be called
///
/// # Expected Returns
/// * `ret>=0`: Meaning the SDK initialised successfully and the number indicates the number of devices that were found on plugin initialisation
/// * `NoPlugins`: Meaning that either no plugins were found or some were found but none were successfully initialised
#[no_mangle]
pub extern "C" fn wooting_analog_initialise() -> c_int {
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
#[no_mangle]
pub extern "C" fn wooting_analog_version() -> c_int {
    env!("CARGO_PKG_VERSION")
        .split('.')
        .collect::<Vec<&str>>()
        .first()
        .and_then(|v| v.parse().ok())
        .unwrap()
}

/// Returns a bool indicating if the Analog SDK has been initialised
#[no_mangle]
pub extern "C" fn wooting_analog_is_initialised() -> bool {
    ANALOG_SDK.lock().unwrap().initialised
}

/// Uninitialises the SDK, returning it to an empty state, similar to how it would be before first initialisation
/// # Expected Returns
/// * `Ok`: Indicates that the SDK was successfully uninitialised
#[no_mangle]
pub extern "C" fn wooting_analog_uninitialise() -> WootingAnalogResult {
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
#[no_mangle]
pub extern "C" fn wooting_analog_set_keycode_mode(mode: c_uint) -> WootingAnalogResult {
    if !ANALOG_SDK.lock().unwrap().initialised {
        return WootingAnalogResult::UnInitialized;
    }

    //TODO: Make it return invalid argument when attempting to use virutal keys on platforms other than win
    if let Some(key_mode) = KeycodeType::from_u32(mode) {
        #[cfg(not(windows))]
        {
            if key_mode == KeycodeType::VirtualKey || key_mode == KeycodeType::VirtualKeyTranslate {
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
#[no_mangle]
pub extern "C" fn wooting_analog_read_analog(code: c_ushort) -> c_float {
    wooting_analog_read_analog_device(code, 0)
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
#[no_mangle]
pub extern "C" fn wooting_analog_read_analog_device(
    code: c_ushort,
    device_id: DeviceID,
) -> c_float {
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
#[no_mangle]
pub extern "C" fn wooting_analog_set_device_event_cb(
    cb: extern "C" fn(DeviceEventType, *mut DeviceInfo_FFI),
) -> WootingAnalogResult {
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
#[no_mangle]
pub extern "C" fn wooting_analog_clear_device_event_cb() -> WootingAnalogResult {
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
#[no_mangle]
pub extern "C" fn wooting_analog_get_connected_devices_info(
    buffer: *mut *mut DeviceInfo_FFI,
    len: c_uint,
) -> c_int {
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
#[no_mangle]
pub extern "C" fn wooting_analog_read_full_buffer(
    code_buffer: *mut c_ushort,
    analog_buffer: *mut c_float,
    len: c_uint,
) -> c_int {
    wooting_analog_read_full_buffer_device(code_buffer, analog_buffer, len, 0)
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
#[no_mangle]
pub extern "C" fn wooting_analog_read_full_buffer_device(
    code_buffer: *mut c_ushort,
    analog_buffer: *mut c_float,
    len: c_uint,
    device_id: DeviceID,
) -> c_int {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::keycode::hid_to_code;
    use shared_memory::{
        ReadLockGuard, ReadLockable, SharedMem, SharedMemCast, WriteLockGuard, WriteLockable,
    };

    use std::sync::{Arc, MutexGuard};
    use std::time::Duration;

    #[derive(Debug, PartialEq)]
    struct SharedState {
        pub vendor_id: u16,
        /// Device Product ID `pid`
        pub product_id: u16,
        //TODO: Consider switching these to FFiStr
        /// Device Manufacturer name
        pub manufacturer_name: [u8; 20],
        /// Device name
        pub device_name: [u8; 20],

        pub device_type: DeviceType,

        pub device_connected: bool,
        pub dirty_device_info: bool,

        pub analog_values: [u8; 0xFF],
    }

    unsafe impl SharedMemCast for SharedState {}

    pub fn get_sdk() -> MutexGuard<'static, AnalogSDK> {
        ANALOG_SDK.lock().unwrap()
    }

    lazy_static! {
        static ref got_connected: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    }
    extern "C" fn connect_cb(event: DeviceEventType, _device: *mut DeviceInfo_FFI) {
        info!("Got cb {:?}", event);

        *Arc::clone(&got_connected).lock().unwrap() = event == DeviceEventType::Connected;
    }

    fn wait_for_connected(attempts: u32, connected: bool) {
        let mut n = 0;
        while *Arc::clone(&got_connected).lock().unwrap() != connected {
            if n > attempts {
                panic!(
                    "Waiting for device to be connected status: {:?} timed out!",
                    connected
                );
            }
            ::std::thread::sleep(Duration::from_millis(500));
            n += 1;
        }
        info!("Got {:?} after {} attempts", connected, n);
    }

    fn get_wlock(shmem: &mut SharedMem) -> WriteLockGuard<SharedState> {
        match shmem.wlock::<SharedState>(0) {
            Ok(v) => v,
            Err(_) => panic!("Failed to acquire write lock !"),
        }
    }

    fn get_rlock(shmem: &mut SharedMem) -> ReadLockGuard<SharedState> {
        match shmem.rlock::<SharedState>(0) {
            Ok(v) => v,
            Err(_) => panic!("Failed to acquire write lock !"),
        }
    }

    fn shared_init() {
        env_logger::try_init_from_env(env_logger::Env::from("trace"))
            .map_err(|e| println!("ERROR: Could not initialise env_logger. '{:?}'", e));
    }

    #[test]
    fn test_ffi_interface() {
        shared_init();

        assert_eq!(wooting_analog_version(), 0);

        //Claim the mutex lock
        let _lock = TEST_PLUGIN_LOCK.lock().unwrap();

        let mut mode;
        let dir = format!(
            "../target/{}/test_plugin",
            std::env::var("TEST_TARGET").unwrap_or("debug".to_owned())
        );
        info!("Loading plugins from: {:?}", dir);
        assert!(!wooting_analog_is_initialised());
        assert_eq!(
            get_sdk()
                .initialise_with_plugin_path(dir.as_str(), !dir.ends_with("debug"))
                .0,
            Ok(0)
        );
        assert!(wooting_analog_is_initialised());

        //Wait a slight bit to ensure that the test-plugin worker thread has initialised the shared mem
        ::std::thread::sleep(Duration::from_millis(500));

        let mut shmem = match SharedMem::open_linked(
            std::env::temp_dir()
                .join("wooting-test-plugin.link")
                .as_os_str(),
        ) {
            Ok(v) => v,
            Err(e) => {
                println!("Error : {}", e);
                println!("Failed to open SharedMem...");
                assert!(false);
                return;
            }
        };

        wooting_analog_set_device_event_cb(connect_cb);

        //Check the connected cb is called
        {
            {
                let mut shared_state = get_wlock(&mut shmem);
                shared_state.device_connected = true;
            }
            wait_for_connected(5, true);
        }

        //Check that we now have one device
        {
            let mut device_infos: Vec<*mut DeviceInfo_FFI> = vec![std::ptr::null_mut(); 2];
            assert_eq!(
                wooting_analog_get_connected_devices_info(
                    device_infos.as_mut_ptr(),
                    device_infos.len() as u32
                ),
                1
            );
            //            unsafe {
            //                debug!("comparing stoof");
            //                let shared_state = get_rlock(&mut shmem);
            //                assert_eq!(device_infos[0].0.read().device_id, shared_state.device_id);
            //                assert!(CString::from_raw(device_infos[0].0.read().device_name as *mut i8).eq(&CString::from_raw(shared_state.device_name.as_ptr() as *mut i8)));
            //                assert!(CString::from_raw(device_infos[0].0.read().manufacturer_name as *mut i8).eq(&CString::from_raw(shared_state.manufacturer_name.as_ptr() as *mut i8)));
            //                assert_eq!(device_infos[0].0.read().product_id, shared_state.product_id);
            //                assert_eq!(device_infos[0].0.read().vendor_id, shared_state.vendor_id);
            //                assert_eq!(device_infos[0].0.read().device_type, shared_state.device_type);
            //                debug!("done comparing stoof");
            //            }
        }

        //Check the cb is called with disconnected
        {
            {
                let mut shared_state = get_wlock(&mut shmem);
                shared_state.device_connected = false;
            }
            wait_for_connected(5, false);
        }

        //Check that we now have no devices
        {
            let mut device_infos: Vec<*mut DeviceInfo_FFI> = vec![std::ptr::null_mut(); 2];
            assert_eq!(
                wooting_analog_get_connected_devices_info(
                    device_infos.as_mut_ptr(),
                    device_infos.len() as u32
                ),
                0
            );
        }

        let analog_val = 0xF4;
        let f_analog_val = f32::from(analog_val) / 255_f32;
        let analog_key = 5;
        //Connect the device again, set a keycode to a val
        let device_id = {
            let mut shared_state = get_wlock(&mut shmem);
            shared_state.analog_values[analog_key] = analog_val;
            shared_state.device_connected = true;
            1
        };

        wait_for_connected(5, true);

        //Check we get the val with no id specified
        assert_eq!(wooting_analog_read_analog(analog_key as u16), f_analog_val);
        //Check we get the val with the device_id we use
        assert_eq!(
            wooting_analog_read_analog_device(analog_key as u16, device_id),
            f_analog_val
        );
        //Check we don't get a val with invalid device id
        assert_eq!(
            wooting_analog_read_analog_device(analog_key as u16, device_id + 1),
            WootingAnalogResult::NoDevices.into()
        );
        //Check if the next value is 0
        assert_eq!(
            wooting_analog_read_analog_device((analog_key + 1) as u16, device_id),
            0.0
        );

        //Check that it does code mapping
        mode = KeycodeType::ScanCode1;
        wooting_analog_set_keycode_mode(mode.clone() as u32);
        assert_eq!(
            wooting_analog_read_analog_device(
                hid_to_code(analog_key as u16, &mode).unwrap(),
                device_id
            ),
            f_analog_val
        );
        mode = KeycodeType::HID;
        wooting_analog_set_keycode_mode(mode.clone() as u32);

        let buffer_len = 5;
        let mut code_buffer: Vec<u16> = vec![0; buffer_len];
        let mut analog_buffer: Vec<f32> = vec![0.0; buffer_len];
        //Check it reads buffer properly with no device id
        assert_eq!(
            wooting_analog_read_full_buffer(
                code_buffer.as_mut_ptr(),
                analog_buffer.as_mut_ptr(),
                buffer_len as u32
            ),
            1
        );
        assert_eq!(code_buffer[0], analog_key as u16);
        assert_eq!(analog_buffer[0], f_analog_val);

        //Check it reads buffer properly with proper device_id
        assert_eq!(
            wooting_analog_read_full_buffer_device(
                code_buffer.as_mut_ptr(),
                analog_buffer.as_mut_ptr(),
                buffer_len as u32,
                device_id
            ),
            1
        );
        assert_eq!(code_buffer[0], analog_key as u16);
        assert_eq!(analog_buffer[0], f_analog_val);

        //Clean the first part of buffer to make sure it isn't written into
        code_buffer[0] = 0;
        analog_buffer[0] = 0.0;
        //Check it errors on read buffer with invalid device_id
        assert_eq!(
            wooting_analog_read_full_buffer_device(
                code_buffer.as_mut_ptr(),
                analog_buffer.as_mut_ptr(),
                buffer_len as u32,
                device_id + 1
            ),
            WootingAnalogResult::NoDevices.into()
        );
        assert_eq!(code_buffer[0], 0);
        assert_eq!(analog_buffer[0], 0.0);

        //Check that it does code mapping
        wooting_analog_set_keycode_mode(mode.clone() as u32);
        assert_eq!(
            wooting_analog_read_full_buffer_device(
                code_buffer.as_mut_ptr(),
                analog_buffer.as_mut_ptr(),
                buffer_len as u32,
                device_id
            ),
            1
        );
        assert_eq!(
            code_buffer[0],
            hid_to_code(analog_key as u16, &mode).unwrap()
        );
        assert_eq!(analog_buffer[0], f_analog_val);
        mode = KeycodeType::HID;
        wooting_analog_set_keycode_mode(mode.clone() as u32);

        {
            let mut shared_state = get_wlock(&mut shmem);
            shared_state.analog_values[analog_key] = 0;
        }
        ::std::thread::sleep(Duration::from_secs(1));

        code_buffer[0] = 0;
        //Check that it returns the now released key with 0 analog in the next call
        assert_eq!(
            wooting_analog_read_full_buffer_device(
                code_buffer.as_mut_ptr(),
                analog_buffer.as_mut_ptr(),
                buffer_len as u32,
                device_id
            ),
            1
        );
        assert_eq!(code_buffer[0], analog_key as u16);
        assert_eq!(analog_buffer[0], 0.0);
        assert_eq!(wooting_analog_read_analog(analog_key as u16), 0.0);

        //Check that the freshly released key is no longer returned
        assert_eq!(
            wooting_analog_read_full_buffer_device(
                code_buffer.as_mut_ptr(),
                analog_buffer.as_mut_ptr(),
                buffer_len as u32,
                device_id
            ),
            0
        );

        assert_eq!(
            wooting_analog_clear_device_event_cb(),
            WootingAnalogResult::Ok
        );
        {
            let mut shared_state = get_wlock(&mut shmem);
            shared_state.device_connected = false;
        }
        ::std::thread::sleep(Duration::from_secs(1));
        //This shouldn't have updated if the cb is not there
        assert!(*Arc::clone(&got_connected).lock().unwrap());

        assert_eq!(wooting_analog_uninitialise(), WootingAnalogResult::Ok);
        assert!(!wooting_analog_is_initialised());

        // Test if re-initialisation works
        wooting_analog_initialise();
        assert_eq!(wooting_analog_uninitialise(), WootingAnalogResult::Ok);
    }
}
