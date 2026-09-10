use ffi_support::FfiStr;
use libloading::{Library, Symbol};
use log::{debug, error, info};
use std::{
    collections::HashMap,
    os::raw::{c_float, c_int, c_uint, c_ushort, c_void},
};

use crate::{
    AnalogValue, KeyCode, KeyPosition, PhysicalKey, Plugin,
    device::{DeviceEventType, DeviceID, DeviceInfo, DeviceInfo_FFI},
    err::{DeviceError, PluginError, ReadError, WootingAnalogResult},
    plugin::wooting::ANALOG_MAX_SIZE,
};

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
                #[unsafe(no_mangle)]
                fn $fn_names(&mut self, $($fn_arg_names: $fn_arg_tys),*) $(-> $fn_ret_tys)* {
                    unsafe {
                        type FnPtr = unsafe fn($($fn_arg_tys),*) $(-> $fn_ret_tys)*;

                        //TODO: Retain the obtained function pointer between calls
                        self.lib.get(stringify!($fn_names)
                            .as_bytes())
                            .inspect_err(|e| {
                                error!("{}", e);
                            })
                            .map(|f: Symbol<FnPtr>| f($($fn_arg_names),*))
                            .unwrap_or_default()
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
                #[unsafe(no_mangle)]
                fn $fn_names(&mut self, $($fn_arg_names: $fn_arg_tys),*) $(-> Result<$fn_ret_tys, WootingAnalogResult>)* {
                    unsafe {
                        type FnPtr = unsafe fn($($fn_arg_tys),*) $(-> $fn_ret_tys)*;

                        self.lib.get(stringify!($fn_names)
                            .as_bytes())
                            .inspect_err(|e| {
                                error!("{}", e);
                            })
                            .map(|f: Symbol<FnPtr>| f($($fn_arg_names),*))
                            .map_err(|_| WootingAnalogResult::FunctionNotFound)
                    }
                }
            //}
        )*
    };
}

type DeviceCallback = dyn Fn(DeviceEventType, &DeviceInfo) + Send + Sync + 'static;

struct CallbackState {
    callback: std::sync::Arc<DeviceCallback>,
}

type DataPtr = Option<*mut Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send + Sync>>;

const CPLUGIN_ABI_VERSION: u32 = 2;

#[derive(Debug)]
pub struct DynamicPlugin {
    lib: Library,
    cb_data_ptr: DataPtr,
    //funcs: HashMap<&'static str, Option<Symbol>>
    code_buffer: Vec<u16>,
    value_buffer: Vec<f32>,
    analog_data: HashMap<u16, f32>,
    device_ids: Vec<DeviceID>,
}

// SAFETY: The raw pointer `cb_data_ptr` is only used for FFI with C plugins.
// It points to a leaked Box that is:
// - Created in `initialise` via `Box::into_raw`
// - Accessed in the C callback (`call_closure`) which reconstructs and re-leaks it
// - Cleaned up in `unload` via `Box::from_raw`
// The pointer is not shared or accessed concurrently - the C plugin serializes callback invocations.
unsafe impl Send for DynamicPlugin {}
unsafe impl Sync for DynamicPlugin {}

impl DynamicPlugin {
    pub(crate) fn new(lib: Library) -> Result<DynamicPlugin, PluginError> {
        unsafe {
            if let Ok(ver) = lib.get::<*mut u32>(b"ANALOG_SDK_PLUGIN_ABI_VERSION") {
                let v = **ver;
                info!("Got cplugin abi: {:?}", v);
                if v != CPLUGIN_ABI_VERSION {
                    error!(
                        "CPlugin ABI version does not match! Given: {}, Expected: {}",
                        v, CPLUGIN_ABI_VERSION
                    );
                    return Err(PluginError::VersionMismatch {
                        version: v,
                        expected: CPLUGIN_ABI_VERSION,
                    });
                }
            }
        }

        Ok(DynamicPlugin {
            lib,
            cb_data_ptr: None, //funcs: HashMap::new()
            code_buffer: vec![0; ANALOG_MAX_SIZE],
            value_buffer: vec![0.0; ANALOG_MAX_SIZE],
            analog_data: HashMap::new(),
            device_ids: Vec::new(),
        })
    }

    lib_wrap_option! {
        //c_name has to be over here due to it not being part of the Plugin trait
        fn initialise(data: *const c_void, callback: extern "C" fn(*mut c_void, DeviceEventType, *const DeviceInfo_FFI)) -> i32;
        fn name() -> FfiStr<'static>;

        fn read_analog(code: u16, device: DeviceID) -> f32;
        fn read_full_buffer(code_buffer: *mut c_ushort, analog_buffer: *mut c_float, len: c_uint, device: DeviceID) -> c_int;
        fn device_info(buffer: *mut *const DeviceInfo_FFI, len: c_uint) -> c_int;
    }

    lib_wrap! {
        fn is_initialised() -> bool;
        fn unload();
    }

    /// Check if this plugin owns the given device_id.
    /// Returns true if device_id is 0 (all devices) or if we own it.
    fn has_device(&self, device_id: DeviceID) -> bool {
        device_id == 0 || self.device_ids.contains(&device_id)
    }

    /// Fetch device info from the C plugin and cache the device IDs
    fn refresh_device_ids(&mut self) {
        let mut device_infos: Vec<*const DeviceInfo_FFI> = vec![std::ptr::null(); 10];

        if let Ok(num) = self
            .device_info(device_infos.as_mut_ptr(), device_infos.len() as c_uint)
            .map(|no| no as usize)
        {
            let num = num.min(device_infos.len());
            device_infos.truncate(num);
            self.device_ids = unsafe {
                device_infos
                    .iter()
                    .filter_map(|dev| dev.as_ref())
                    .map(|dev| dev.device_id)
                    .collect()
            };
            debug!("DynamicPlugin owns device IDs: {:?}", self.device_ids);
        }
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

        if device_raw.is_null() {
            error!("We got a null device_raw pointer in call_closure!");
            return;
        }

        // Use to_device_info() to borrow and copy the data without freeing the C-owned memory
        let device_info = device_raw.as_ref().unwrap().to_device_info();

        let state = &*(data as *const CallbackState);
        let callback = std::sync::Arc::clone(&state.callback);

        callback(event, &device_info);
    }
}

impl Plugin for DynamicPlugin {
    fn name(&mut self) -> Result<&'static str, PluginError> {
        self.name()
            .map(|s| s.as_str())
            .map_err(|_| PluginError::FunctionUnavailable("name"))
    }

    fn initialise(
        &mut self,
        callback: Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send + Sync>,
    ) -> Result<u32, ReadError> {
        let state = Box::new(CallbackState {
            callback: std::sync::Arc::from(callback),
        });

        let data = Box::into_raw(state);
        self.cb_data_ptr = Some(data as *mut _);

        let result = self
            .initialise(data as *const _, call_closure)
            .map(|res| res as u32)
            .map_err(|_| ReadError::function_unavailable("initialise"));

        if result.is_err()
            && let Some(ptr) = self.cb_data_ptr.take()
        {
            unsafe {
                drop(Box::from_raw(ptr as *mut CallbackState));
            }
        }

        // Cache the device IDs this plugin owns
        self.refresh_device_ids();

        result
    }

    fn read_analog(&mut self, code: u16, device_id: DeviceID) -> Result<f32, ReadError> {
        if !self.has_device(device_id) {
            return Err(ReadError::Device(DeviceError::unknown_device(device_id)));
        }

        self.read_analog(code, device_id)
            .map_err(|_| ReadError::function_unavailable("read_analog"))
    }

    fn read_keycode(
        &mut self,
        code: KeyCode,
        device_id: DeviceID,
    ) -> Result<AnalogValue, ReadError> {
        if !self.has_device(device_id) {
            return Err(ReadError::Device(DeviceError::unknown_device(device_id)));
        }

        match self.read_analog(u16::from(code), device_id) {
            Ok(v) => {
                if v >= 0.0 {
                    Ok(AnalogValue::from(v))
                } else {
                    Err(ReadError::Plugin(PluginError::InvalidRead(device_id)))
                }
            }
            Err(_) => Err(ReadError::function_unavailable("read_keycode")),
        }
    }

    fn read_position(
        &mut self,
        _position: KeyPosition,
        _device_id: DeviceID,
    ) -> Result<PhysicalKey, ReadError> {
        // TODO: for now let's assume c plugins can not yet supply this data
        // can easily be included via an optional fn in plugin.h
        Err(ReadError::function_unavailable("read_position"))
    }

    fn read_full_buffer(
        &mut self,
        device_id: DeviceID,
    ) -> Result<HashMap<c_ushort, c_float>, ReadError> {
        if !self.has_device(device_id) {
            return Err(ReadError::Device(DeviceError::unknown_device(device_id)));
        }

        let count: usize = {
            let code_ptr = self.code_buffer.as_mut_ptr();
            let value_ptr = self.value_buffer.as_mut_ptr();

            let write_count = self
                .read_full_buffer(code_ptr, value_ptr, ANALOG_MAX_SIZE as c_uint, device_id)
                .map_err(|_| ReadError::function_unavailable("read_full_buffer"))?;

            if write_count < 0 {
                return Err(ReadError::Plugin(PluginError::InvalidRead(device_id)));
            }

            ANALOG_MAX_SIZE.min(write_count as usize)
        };

        for i in 0..count {
            self.analog_data
                .insert(self.code_buffer[i], self.value_buffer[i]);
        }

        self.code_buffer.fill(0);
        self.value_buffer.fill(0.0);

        Ok(std::mem::take(&mut self.analog_data))
    }

    fn read_keycodes(
        &mut self,
        device_id: DeviceID,
    ) -> Result<HashMap<KeyCode, AnalogValue>, ReadError> {
        if !self.has_device(device_id) {
            return Err(ReadError::Device(DeviceError::unknown_device(device_id)));
        }

        Ok(Plugin::read_full_buffer(self, device_id)?
            .iter()
            .map(|(k, v)| (KeyCode::from(*k), AnalogValue::from(*v)))
            .collect())
    }

    fn read_positions(
        &mut self,
        _device_id: DeviceID,
    ) -> Result<HashMap<KeyPosition, PhysicalKey>, ReadError> {
        Err(ReadError::function_unavailable("read_positions"))
    }

    fn device_info(&mut self) -> Result<Vec<DeviceInfo>, ReadError> {
        let mut device_infos: Vec<*const DeviceInfo_FFI> = vec![std::ptr::null(); 10];

        match self
            .device_info(device_infos.as_mut_ptr(), device_infos.len() as c_uint)
            .map(|no| no as usize)
        {
            Ok(num) => unsafe {
                let num = num.min(device_infos.len());
                let devices = device_infos
                    .into_iter()
                    .take(num as usize)
                    .filter_map(|dev| dev.as_ref())
                    .map(|dev| dev.to_device_info())
                    .collect();
                Ok(devices)
            },
            Err(_) => Err(ReadError::function_unavailable("device_info")),
        }
    }

    fn is_initialised(&mut self) -> bool {
        self.is_initialised()
    }

    fn unload(&mut self) {
        self.unload();

        // Take and drop cb_data_ptr
        if let Some(ptr) = self.cb_data_ptr.take() {
            unsafe {
                drop(Box::from_raw(ptr as *mut CallbackState));
            }
        }
    }
}
