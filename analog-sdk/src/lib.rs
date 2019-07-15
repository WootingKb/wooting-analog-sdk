pub mod sdk;
pub mod keycode;
#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;
extern crate ffi_support;
extern crate scancode;
#[macro_use] extern crate enum_primitive;
mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{}
}
#[macro_use] extern crate lazy_static;
#[cfg(windows)] extern crate winapi;
use enum_primitive::FromPrimitive;
use sdk::*;
use std::sync::Mutex;
use ffi_support::FfiStr;
use std::os::raw::{c_uint, c_float, c_int, c_ushort};
use std::slice;
use crate::keycode::KeycodeType;

lazy_static! {
    static ref ANALOG_SDK: Mutex<AnalogSDK> = Mutex::new(AnalogSDK::new());
}

#[no_mangle]
pub extern "C" fn sdk_initialise() -> bool {
    ANALOG_SDK.lock().unwrap().initialise()
}

#[no_mangle]
pub extern "C" fn sdk_is_initialised() -> bool {
    ANALOG_SDK.lock().unwrap().initialised    
}

#[no_mangle]
pub extern "C" fn sdk_uninitialise() -> bool {
    ANALOG_SDK.lock().unwrap().unload();
    true
}

#[no_mangle]
pub extern "C" fn sdk_add(x: u32, y: u32) -> u32 {
    if !ANALOG_SDK.lock().unwrap().initialised {
        return Default::default();
    }

    ANALOG_SDK.lock().unwrap().add(x, y).pop().unwrap_or(0)
}

#[no_mangle]
pub extern "C" fn sdk_set_mode(mode: u32) -> c_int {
    if !ANALOG_SDK.lock().unwrap().initialised {
        return -1;
    }
    if let Some(key_mode) = KeycodeType::from_u32(mode) {
        ANALOG_SDK.lock().unwrap().keycode_mode = key_mode;
        1
    }
    else {
        -1
    }
}

#[no_mangle]
pub extern "C" fn sdk_read_analog(code: c_ushort) -> f32 {
    if !ANALOG_SDK.lock().unwrap().initialised {
        return -1.0;
    }

    ANALOG_SDK.lock().unwrap().read_analog(code)
}

#[no_mangle]
pub extern "C" fn sdk_set_disconnected_cb(cb: extern fn(FfiStr)) {
    ANALOG_SDK.lock().unwrap().disconnected_callback = Some(cb);
}

#[no_mangle]
pub extern "C" fn sdk_clear_disconnected_cb() {
    ANALOG_SDK.lock().unwrap().disconnected_callback = None;
}

#[no_mangle]
pub extern "C" fn sdk_device_info(buffer: *mut DeviceInfoPointer, len: c_uint ) -> c_int {
    let buff = unsafe {
        assert!(!buffer.is_null());

        slice::from_raw_parts_mut(buffer, len as usize)
    };

    ANALOG_SDK.lock().unwrap().get_device_info(buff)
}

#[no_mangle]
pub extern "C" fn sdk_read_full_buffer(code_buffer: *mut c_ushort, analog_buffer: *mut c_float, len: c_uint) -> c_int {
    sdk_read_full_buffer_device(code_buffer, analog_buffer, len, 0)
}

#[no_mangle]
pub extern "C" fn sdk_read_full_buffer_device(code_buffer: *mut c_ushort, analog_buffer: *mut c_float, len: c_uint, device: DeviceID) -> c_int {
    if !ANALOG_SDK.lock().unwrap().initialised {
        return -1;
    }

    let codes = unsafe {
        assert!(!code_buffer.is_null());

        slice::from_raw_parts_mut(code_buffer, len as usize)
    };

    let analog = unsafe {
        assert!(!analog_buffer.is_null());

        slice::from_raw_parts_mut(analog_buffer, len as usize)
    };

    ANALOG_SDK.lock().unwrap().read_full_buffer(codes, analog, device)
}


/*#[no_mangle]
pub extern "C" fn test_function(x: u32) -> u32 {
    x * 2
}*/

/*#[no_mangle]
pub extern "C" fn add(x: u32, y: u32) -> u32 {
    x + y
}*/