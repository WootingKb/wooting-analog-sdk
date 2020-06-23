#[macro_use]
extern crate lazy_static;
extern crate wooting_analog_common;

pub use wooting_analog_common::*;
pub mod ffi;
use ffi::*;
use std::collections::HashMap;
use std::os::raw::c_uint;
use std::ptr;

pub fn version() -> SDKResult<u32> {
    return unsafe { wooting_analog_version().into() };
}

// fn wooting_analog_version() -> c_int;

// fn wooting_analog_initialise() -> c_int;

pub fn initialise() -> SDKResult<u32> {
    return unsafe { wooting_analog_initialise().into() };
}

// fn wooting_analog_is_initialised() -> bool;
pub fn is_initialised() -> bool {
    return unsafe { wooting_analog_is_initialised() };
}

// fn wooting_analog_uninitialise() -> WootingAnalogResult;

pub fn uninitialise() -> SDKResult<()> {
    return unsafe { wooting_analog_uninitialise().into() };
}

// fn wooting_analog_set_keycode_mode(mode: KeycodeType) -> WootingAnalogResult;
pub fn set_keycode_mode(mode: KeycodeType) -> SDKResult<()> {
    return unsafe { wooting_analog_set_keycode_mode(mode).into() };
}

// fn wooting_analog_read_analog(code: c_ushort) -> f32;
pub fn read_analog(code: u16) -> SDKResult<f32> {
    return unsafe { wooting_analog_read_analog(code).into() };
}
// fn wooting_analog_read_analog_device(code: c_ushort, device_id: DeviceID) -> f32;
pub fn read_analog_device(code: u16, device_id: DeviceID) -> SDKResult<f32> {
    return unsafe { wooting_analog_read_analog_device(code, device_id).into() };
}
// fn wooting_analog_set_device_event_cb(
//     cb: extern "C" fn(DeviceEventType, *mut DeviceInfo_FFI),
// ) -> WootingAnalogResult;
pub fn set_device_event_cb(
    cb: extern "C" fn(DeviceEventType, *mut DeviceInfo_FFI),
) -> SDKResult<()> {
    return unsafe { wooting_analog_set_device_event_cb(cb).into() };
}
// fn wooting_analog_clear_device_event_cb() -> WootingAnalogResult;
pub fn clear_device_event_cb() -> SDKResult<()> {
    return unsafe { wooting_analog_clear_device_event_cb().into() };
}
// fn wooting_analog_get_connected_devices_info(
//     buffer: *mut *mut DeviceInfo_FFI,
//     len: c_uint,
// ) -> c_int;
const DEVICE_BUFFER_LEN: usize = 20;

pub fn get_connected_devices_info() -> SDKResult<Vec<DeviceInfo>> {
    unsafe {
        let mut buffer: Vec<*mut DeviceInfo_FFI> = vec![ptr::null_mut(); DEVICE_BUFFER_LEN];

        let ret: SDKResult<u32> = wooting_analog_get_connected_devices_info(
            buffer.as_mut_ptr(),
            DEVICE_BUFFER_LEN as c_uint,
        )
        .into();

        return ret
            .0
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

// fn wooting_analog_read_full_buffer(
//     code_buffer: *mut c_ushort,
//     analog_buffer: *mut c_float,
//     len: c_uint,
// ) -> c_int;
const ANALOG_BUFFER_LEN: usize = 30;
pub fn read_full_buffer_device(device_id: DeviceID) -> SDKResult<HashMap<u16, f32>> {
    unsafe {
        let mut code_buffer: Vec<u16> = vec![0; ANALOG_BUFFER_LEN];
        let mut analog_buffer: Vec<f32> = vec![0.0; ANALOG_BUFFER_LEN];

        let ret: SDKResult<u32> = wooting_analog_read_full_buffer_device(
            code_buffer.as_mut_ptr(),
            analog_buffer.as_mut_ptr(),
            ANALOG_BUFFER_LEN as u32,
            device_id,
        )
        .into();

        return ret
            .0
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

pub fn read_full_buffer() -> SDKResult<HashMap<u16, f32>> {
    return read_full_buffer_device(0);
}

// fn wooting_analog_read_full_buffer_device(
//     code_buffer: *mut c_ushort,
//     analog_buffer: *mut c_float,
//     len: c_uint,
//     device_id: DeviceID,
// ) -> c_int;
