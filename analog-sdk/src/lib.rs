pub mod sdk;
#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;
extern crate ffi_support;
extern crate scancode;
mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain!{}
}
#[macro_use] extern crate lazy_static;
#[cfg(windows)] extern crate winapi;

use sdk::*;
use std::sync::Mutex;
use ffi_support::FfiStr;
use std::os::raw::c_char;

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
pub extern "C" fn sdk_read_analog_hid(code: u8) -> f32 {
    if !ANALOG_SDK.lock().unwrap().initialised {
        return -1.0;
    }

    ANALOG_SDK.lock().unwrap().read_analog_hid(code)
}

#[no_mangle]
pub extern "C" fn sdk_read_analog_vk(code: u8, translate: bool) -> f32 {
    if !ANALOG_SDK.lock().unwrap().initialised {
        return -1.0;
    }

    ANALOG_SDK.lock().unwrap().read_analog_vk(code, translate)
}


#[no_mangle]
pub extern "C" fn sdk_read_analog_sc(code: u8) -> f32 {
    if !ANALOG_SDK.lock().unwrap().initialised {
        return -1.0;
    }

    ANALOG_SDK.lock().unwrap().read_analog_sc(code)
}

#[no_mangle]
pub extern "C" fn sdk_set_disconnected_cb(cb: extern fn(*const c_char)) {
    ANALOG_SDK.lock().unwrap().disconnected_callback = Some(cb);
}

#[no_mangle]
pub extern "C" fn sdk_clear_disconnected_cb() {
    ANALOG_SDK.lock().unwrap().disconnected_callback = None;
}

/*#[no_mangle]
pub extern "C" fn test_function(x: u32) -> u32 {
    x * 2
}*/

/*#[no_mangle]
pub extern "C" fn add(x: u32, y: u32) -> u32 {
    x + y
}*/