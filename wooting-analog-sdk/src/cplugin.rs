use ffi_support::FfiStr;
use libloading::{Library, Symbol};
use log::*;
use std::collections::HashMap;
use std::os::raw::{c_float, c_int, c_uint, c_ushort};
use wooting_analog_common::*;
use wooting_analog_plugin_dev::*;

macro_rules! lib_wrap {
    //(@as_item $i:item) => {$i};

    (
        $(
            fn $fn_names:ident($($fn_arg_names:ident: $fn_arg_tys:ty),*) $(-> $fn_ret_tys:ty)*;
        )*
    ) => {
        $(
            //lib_wrap! {
            //    @as_item
                #[no_mangle]
                fn $fn_names(&mut self, $($fn_arg_names: $fn_arg_tys),*) $(-> $fn_ret_tys)* {
                    unsafe {
                        type FnPtr = unsafe fn($($fn_arg_tys),*) $(-> $fn_ret_tys)*;
                        //TODO: Retain the obtained function pointer between calls
                        let func :  Option<Symbol<FnPtr>>  = self.lib.get(stringify!($fn_names).as_bytes()).map_err(|e| {
                                    error!("{}", e);
                                }).ok();
                        match func {
                            Some(f) => f($($fn_arg_names),*).into(),
                            _ => Default::default()

                        }
                    }
                }
            //}
        )*
    };
}

macro_rules! lib_wrap_option {
    //(@as_item $i:item) => {$i};

    (
        $(
            fn $fn_names:ident($($fn_arg_names:ident: $fn_arg_tys:ty),*) $(-> $fn_ret_tys:ty)*;
        )*
    ) => {
        $(
            //lib_wrap! {
            //    @as_item
                #[no_mangle]
                fn $fn_names(&mut self, $($fn_arg_names: $fn_arg_tys),*) $(-> SDKResult<$fn_ret_tys>)* {
                    unsafe {
                        type FnPtr = unsafe fn($($fn_arg_tys),*) $(-> $fn_ret_tys)*;
                        let func :Option<Symbol<FnPtr>>  = self.lib.get(stringify!($fn_names).as_bytes()).map_err(|e| {
                                    error!("{}", e);
                                }).ok();
                        match func {
                            Some(f) => f($($fn_arg_names),*).into(),
                            _ => Err(WootingAnalogResult::FunctionNotFound).into()

                        }
                    }
                }
            //}
        )*
    };
}

const CPLUGIN_ABI_VERSION: u32 = 0;

pub struct CPlugin {
    lib: Library,
    //funcs: HashMap<&'static str, Option<Symbol>>
}

impl CPlugin {
    pub fn new(lib: Library) -> CPlugin {
        CPlugin {
            lib,
            //funcs: HashMap::new()
        }
    }

    lib_wrap_option! {
        //c_name has to be over here due to it not being part of the Plugin trait
        fn _initialise(callback: extern "C" fn(DeviceEventType, DeviceInfoPointer)) -> i32;
        fn _name() -> FfiStr<'static>;

        fn _read_full_buffer(code_buffer: *const c_ushort, analog_buffer: *const c_float, len: c_uint, device: DeviceID) -> c_int;
        fn _device_info(buffer: *mut DeviceInfoPointer, len: c_uint) -> c_int;
    }
}

impl Plugin for CPlugin {
    fn name(&mut self) -> SDKResult<&'static str> {
        self._name().0.map(|s| s.as_str()).into()
    }

    fn initialise(&mut self, callback: extern "C" fn(DeviceEventType, DeviceInfoPointer)) -> SDKResult<u32>{
        self._initialise(callback).0.map(|res| res as u32).into()
    }

    fn read_full_buffer(
        &mut self,
        max_length: usize,
        device: DeviceID,
    ) -> SDKResult<HashMap<c_ushort, c_float>> {
        let mut code_buffer: Vec<c_ushort> = Vec::with_capacity(max_length);
        let mut analog_buffer: Vec<c_float> = Vec::with_capacity(max_length);
        code_buffer.resize(max_length, 0);
        analog_buffer.resize(max_length, 0.0);
        let count: usize = {
            let ret = self
                ._read_full_buffer(
                    code_buffer.as_ptr(),
                    analog_buffer.as_ptr(),
                    max_length as c_uint,
                    device,
                )
                .0;
            if let Err(e) = ret {
                //debug!("Error got: {:?}",e);
                return Err(e).into();
            }
            let ret = ret.unwrap();
            max_length.min(ret as usize)
        };

        let mut analog_data: HashMap<c_ushort, c_float> = HashMap::with_capacity(count);
        //println!("Count was {}", count);
        for i in 0..count {
            analog_data.insert(code_buffer[i], analog_buffer[i]);
        }

        Ok(analog_data).into()
    }

    fn device_info(&mut self) -> SDKResult<Vec<DeviceInfoPointer>> {
        let mut device_infos: Vec<DeviceInfoPointer> = vec![Default::default(); 10];

        match self._device_info(device_infos.as_mut_ptr(), device_infos.len() as c_uint).0.map(|no| no as u32) {
            Ok(num) => {
                device_infos.truncate(num as usize);
                Ok(device_infos).into()
            },
            Err(e) =>{
                Err(e).into()
            }
        }
    }

    lib_wrap! {
        fn is_initialised() -> bool;
        fn unload();
    }
    lib_wrap_option! {
        fn read_analog(code: u16, device: DeviceID) -> f32;
        //fn neg(x: u32, y: u32) -> u32;
    }
}
