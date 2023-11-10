use ffi_support::FfiStr;
use libloading::{Library, Symbol};
use log::*;
use std::collections::HashMap;
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
                            Some(f) => f($($fn_arg_names),*).into(),
                            _ => Err(WootingAnalogResult::FunctionNotFound).into()

                        }
                    }
                }
            //}
        )*
    };
}

const CPLUGIN_ABI_VERSION: u32 = 1;

pub struct CPlugin {
    lib: Library,
    cb_data_ptr: Option<*mut Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send>>,
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
                    return Err(WootingAnalogResult::IncompatibleVersion).into();
                }
            }
        }

        Ok(CPlugin {
            lib,
            cb_data_ptr: None, //funcs: HashMap::new()
        })
        .into()
    }

    lib_wrap_option! {
        //c_name has to be over here due to it not being part of the Plugin trait
        fn initialise(data: *const c_void, callback: extern "C" fn(*mut c_void, DeviceEventType, *const DeviceInfo_FFI)) -> i32;
        fn name() -> FfiStr<'static>;

        fn read_analog(code: u16, device: DeviceID) -> f32;
        fn read_full_buffer(code_buffer: *const c_ushort, analog_buffer: *const c_float, len: c_uint, device: DeviceID) -> c_int;
        fn device_info(buffer: *mut *const DeviceInfo_FFI, len: c_uint) -> c_int;
    }

    lib_wrap! {
        fn is_initialised() -> bool;
        fn unload();
    }
}

extern "C" fn call_closure(
    data: *mut c_void,
    event: DeviceEventType,
    device_raw: *const DeviceInfo_FFI,
) {
    debug!("Got into the callclosure");
    unsafe {
        if data.is_null() {
            error!("We got a null data pointer in call_closure!");
            return;
        }

        let device_info = device_raw.as_ref().unwrap().into_device_info();

        let callback_ptr =
            Box::from_raw(data as *mut Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send>);

        (*callback_ptr)(event, &device_info);

        //Throw it back into raw to prevent it being dropped so the callback can be called multiple times
        Box::into_raw(callback_ptr);
    }
}

impl Plugin for CPlugin {
    fn name(&mut self) -> SDKResult<&'static str> {
        self.name().0.map(|s| s.as_str()).into()
    }

    fn initialise(
        &mut self,
        callback: Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send>,
    ) -> SDKResult<u32> {
        let data = Box::into_raw(Box::new(callback));
        self.cb_data_ptr = Some(data);
        self.initialise(data as *const _, call_closure)
            .0
            .map(|res| res as u32)
            .into()
    }

    fn read_analog(&mut self, code: u16, device: DeviceID) -> SDKResult<f32> {
        self.read_analog(code, device).0.into()
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
                .read_full_buffer(
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

    fn device_info(&mut self) -> SDKResult<Vec<DeviceInfo>> {
        let mut device_infos: Vec<*const DeviceInfo_FFI> = vec![std::ptr::null_mut(); 10];

        match self
            .device_info(device_infos.as_mut_ptr(), device_infos.len() as c_uint)
            .0
            .map(|no| no as u32)
        {
            Ok(num) => unsafe {
                device_infos.truncate(num as usize);
                let devices = device_infos
                    .drain(..)
                    .map(|dev| dev.as_ref().unwrap().into_device_info())
                    .collect();
                Ok(devices).into()
            },
            Err(e) => Err(e).into(),
        }
    }

    fn is_initialised(&mut self) -> bool {
        self.is_initialised()
    }

    fn unload(&mut self) {
        self.unload();
        // Drop cb_data_ptr
        if let Some(ptr) = self.cb_data_ptr {
            unsafe {
                drop(Box::from_raw(
                    ptr as *mut Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send>,
                ));
            }
        }
    }
}
