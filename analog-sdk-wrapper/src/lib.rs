#[macro_use]
extern crate lazy_static;
extern crate analog_sdk_common;
use libloading as libl;
//use std::ffi::c_void;
use std::ops::Deref;
use std::os::raw::{c_float, c_int, c_uint, c_ushort};
//use std::ptr;
pub use analog_sdk_common::{AnalogSDKResult, DeviceEventType, DeviceID, DeviceInfo, KeycodeType};

/*pub struct Void(*mut c_void);

impl Default for Void {
    fn default() -> Self {
        Void(ptr::null_mut())
    }
}
*/

macro_rules! dynamic_extern {
    (@as_item $i:item) => {$i};

    (
        #[link=$lib:tt]
        extern $cconv:tt {
            $(
                fn $fn_names:ident($($fn_arg_names:ident: $fn_arg_tys:ty),*) $(-> $fn_ret_tys:ty)*;
            )*
        }
    ) => {
        lazy_static! {
            static ref LIB : Option<libl::Library> = {
                #[cfg(unix)]
                let lib_path = concat!($lib, ".so");
                #[cfg(windows)]
                let lib_path = $lib;

                //Attempt to load the library, if it fails print the error and discard the error
                libl::Library::new(lib_path).map_err(|e| {
                    println!("Unable to load library: {}\nErr: {}", lib_path, e);
                }).ok()
            };
        }
        $(
            dynamic_extern! {
                @as_item
                #[no_mangle]
                pub unsafe extern fn $fn_names($($fn_arg_names: $fn_arg_tys),*) $(-> $fn_ret_tys)* {
                    type FnPtr = unsafe extern $cconv fn($($fn_arg_tys),*) $(-> $fn_ret_tys)*;

                    lazy_static! {
                        static ref FUNC: Option<libl::Symbol<'static, FnPtr>> = {
                            LIB.as_ref().and_then(|lib| unsafe {
                                //Get func, print and discard error as we don't need it again
                                lib.get(stringify!($fn_names).as_bytes()).map_err(|e| {
                                    println!("{}", e);
                                }).ok()
                            })
                        };
                    }
                    match FUNC.deref() {
                        Some(f) => f($($fn_arg_names),*),
                        _ => Default::default()
                    }
                }
            }
        )*
    };
}

dynamic_extern! {
    #[link="libanalog_sdk"]
    extern "C" {
        fn sdk_initialise() -> AnalogSDKResult;
        fn sdk_is_initialised() -> bool;
        fn sdk_uninitialise() -> AnalogSDKResult;
        fn sdk_set_mode(mode: KeycodeType) -> AnalogSDKResult;
        fn sdk_read_analog(code: c_ushort) -> f32;
        fn sdk_read_analog_device(code: c_ushort, device_id: DeviceID) -> f32;
        fn sdk_set_device_event_cb(cb: extern fn(DeviceEventType, *mut DeviceInfo)) -> AnalogSDKResult;
        fn sdk_clear_device_event_cb() -> AnalogSDKResult;
        fn sdk_device_info(buffer: *mut *mut DeviceInfo, len: c_uint) -> c_int;
        fn sdk_read_full_buffer(code_buffer: *mut c_ushort, analog_buffer: *mut c_float, len: c_uint) -> c_int;
        fn sdk_read_full_buffer_device(code_buffer: *mut c_ushort, analog_buffer: *mut c_float, len: c_uint, device_id: DeviceID) -> c_int;
    }
}

/*fn main() {
    unsafe { println!("We got {}", test_function(16)); };
}*/
