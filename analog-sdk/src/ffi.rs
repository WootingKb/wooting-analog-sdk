use crate::sdk::*;
use enum_primitive::FromPrimitive;
use std::sync::Mutex;
//use ffi_support::FfiStr;
use analog_sdk_common::*;
use std::os::raw::{c_float, c_int, c_uint, c_ushort};
use std::slice;

lazy_static! {
    static ref ANALOG_SDK: Mutex<AnalogSDK> = Mutex::new(AnalogSDK::new());
}

#[no_mangle]
pub extern "C" fn sdk_initialise() -> AnalogSDKResult {
    env_logger::init();
    ANALOG_SDK.lock().unwrap().initialise()
}

#[no_mangle]
pub extern "C" fn sdk_is_initialised() -> bool {
    ANALOG_SDK.lock().unwrap().initialised
}

#[no_mangle]
pub extern "C" fn sdk_uninitialise() -> AnalogSDKResult {
    ANALOG_SDK.lock().unwrap().unload();
    AnalogSDKResult::Ok
}

#[no_mangle]
pub extern "C" fn sdk_set_mode(mode: u32) -> AnalogSDKResult {
    if let Some(key_mode) = KeycodeType::from_u32(mode) {
        ANALOG_SDK.lock().unwrap().keycode_mode = key_mode;
        AnalogSDKResult::Ok
    } else {
        AnalogSDKResult::InvalidArgument
    }
}

#[no_mangle]
pub extern "C" fn sdk_read_analog(code: c_ushort) -> f32 {
    sdk_read_analog_device(code, 0)
}

#[no_mangle]
pub extern "C" fn sdk_read_analog_device(code: c_ushort, device_id: DeviceID) -> f32 {
    ANALOG_SDK
        .lock()
        .unwrap()
        .read_analog(code, device_id)
        .into()
}

#[no_mangle]
pub extern "C" fn sdk_set_device_event_cb(
    cb: extern "C" fn(DeviceEventType, DeviceInfoPointer),
) -> AnalogSDKResult {
    ANALOG_SDK.lock().unwrap().set_device_event_cb(cb)
}

#[no_mangle]
pub extern "C" fn sdk_clear_device_event_cb() -> AnalogSDKResult {
    ANALOG_SDK.lock().unwrap().clear_device_event_cb()
}

#[no_mangle]
pub unsafe extern "C" fn sdk_device_info(buffer: *mut DeviceInfoPointer, len: c_uint) -> c_int {
    let buff = {
        assert!(!buffer.is_null());

        slice::from_raw_parts_mut(buffer, len as usize)
    };

    ANALOG_SDK.lock().unwrap().get_device_info(buff).into()
}

#[no_mangle]
pub unsafe extern "C" fn sdk_read_full_buffer(
    code_buffer: *mut c_ushort,
    analog_buffer: *mut c_float,
    len: c_uint,
) -> c_int {
    sdk_read_full_buffer_device(code_buffer, analog_buffer, len, 0)
}

#[no_mangle]
pub unsafe extern "C" fn sdk_read_full_buffer_device(
    code_buffer: *mut c_ushort,
    analog_buffer: *mut c_float,
    len: c_uint,
    device_id: DeviceID,
) -> c_int {
    let codes = {
        assert!(!code_buffer.is_null());

        slice::from_raw_parts_mut(code_buffer, len as usize)
    };

    let analog = {
        assert!(!analog_buffer.is_null());

        slice::from_raw_parts_mut(analog_buffer, len as usize)
    };

    ANALOG_SDK
        .lock()
        .unwrap()
        .read_full_buffer(codes, analog, device_id)
        .into()
}
