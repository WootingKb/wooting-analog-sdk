use ffi_support::FfiStr;
use libloading::{Library, Symbol};
use log::*;
use std::collections::HashMap;
use std::ffi::CStr;
use std::os::raw::{c_float, c_int, c_uint, c_ushort, c_void};
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
                            Some(f) => Ok(f($($fn_arg_names),*)),
                            _ => Err(WootingAnalogResult::FunctionNotFound)

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
    pub fn new(lib: Library) -> SDKResult<CPlugin> {
        unsafe {
            if let Some(ver) = lib.get::<*mut u32>(b"ANALOG_SDK_PLUGIN_ABI_VERSION").ok() {
                let v = **ver;
                info!("Got cplugin abi: {:?}", v);
                if v != CPLUGIN_ABI_VERSION {
                    error!(
                        "CPlugin ABI version does not match! Given: {}, Expected: {}",
                        v, CPLUGIN_ABI_VERSION
                    );
                    return Err(WootingAnalogResult::IncompatibleVersion);
                }
            }
        }

        Ok(CPlugin {
            lib,
            //funcs: HashMap::new()
        })
    }

    lib_wrap_option! {
        //c_name has to be over here due to it not being part of the Plugin trait
        fn _initialise(data: *mut c_void, callback: extern "C" fn(*mut c_void, DeviceEventType, *mut DeviceInfo)) -> i32;
        fn _name() -> FfiStr<'static>;

        fn _read_full_buffer(code_buffer: *const c_ushort, analog_buffer: *const c_float, len: c_uint, device: DeviceID) -> c_int;
        fn _device_info(buffer: *mut *mut DeviceInfo_FFI, len: c_uint) -> c_int;
    }
}

extern "C" fn call_closure(data: *mut c_void, event: DeviceEventType, device_raw: *mut DeviceInfo) {
    debug!("Got into the callclosure");
    unsafe {
        if data.is_null() {
            error!("We got a null data pointer in call_closure!");
            return;
        }

        let device = Box::from_raw(device_raw);

        let callback_ptr =
            Box::from_raw(data as *mut Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send>);

        (*callback_ptr)(event, &device);
        //Throw it back into raw to prevent it being dropped so the callback can be called multiple times
        Box::into_raw(callback_ptr);
        //We also want to convert this back to a pointer as we want the C Plugin to be in control and aware of
        //when this memory is being dropped
        Box::into_raw(device);
    }
}

impl Plugin for CPlugin {
    fn name(&mut self) -> SDKResult<&'static str> {
        self._name().map(|s| s.as_str())
    }

    fn initialise(
        &mut self,
        callback: Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send>,
    ) -> SDKResult<u32> {
        let data = Box::into_raw(Box::new(callback));
        self._initialise(data as *mut _, call_closure)
            .map(|res| res as u32)
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
                );
            if let Err(e) = ret {
                //debug!("Error got: {:?}",e);
                return Err(e);
            }
            let ret = ret.unwrap();
            max_length.min(ret as usize)
        };

        let mut analog_data: HashMap<c_ushort, c_float> = HashMap::with_capacity(count);
        //println!("Count was {}", count);
        for i in 0..count {
            analog_data.insert(code_buffer[i], analog_buffer[i]);
        }

        Ok(analog_data)
    }

    fn device_info(&mut self) -> SDKResult<Vec<DeviceInfo>> {
        let mut device_infos: Vec<*mut DeviceInfo_FFI> = vec![std::ptr::null_mut(); 10];

        match self
            ._device_info(device_infos.as_mut_ptr(), device_infos.len() as c_uint)
            .map(|no| no as u32)
        {
            Ok(num) => unsafe {
                device_infos.truncate(num as usize);
                let devices = device_infos
                    .drain(..)
                    .map(|dev| {
                        DeviceInfo {
                            vendor_id: (*dev).vendor_id,
                            product_id: (*dev).product_id,
                            manufacturer_name: CStr::from_ptr((*dev).manufacturer_name).to_str().unwrap().to_owned(),
                            device_name: CStr::from_ptr((*dev).device_name).to_str().unwrap().to_owned(),
                            device_id: (*dev).device_id,
                            device_type: (*dev).device_type.clone(),
                        }
                    })
                    .collect();
                Ok(devices)
            },
            Err(e) => Err(e),
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
