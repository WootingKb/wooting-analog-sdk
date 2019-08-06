#[macro_use]
extern crate enum_primitive;
#[macro_use]
extern crate log;
extern crate ffi_support;

use enum_primitive::FromPrimitive;
use ffi_support::FfiStr;
use std::any::Any;
use std::collections::HashMap;
use std::ffi::CString;
use std::hash::Hasher;
use std::ops::Deref;
use std::os::raw::{c_char, c_float, c_int, c_ushort};

/// Version number of the plugin ABI which is exported in plugins so the SDK can determine how to handle the plugin based on which ABI version it's on
#[no_mangle]
pub static ANALOG_SDK_PLUGIN_ABI_VERSION: u32 = 1;

/// The core Plugin trait which needs to be implemented for an Analog Plugin to function
pub trait Plugin: Any + Send + Sync {
    /// Get a name describing the `Plugin`.
    fn name(&mut self) -> SDKResult<&'static str>;

    /// A callback fired immediately after the plugin is loaded. Usually used
    /// for initialization.
    fn initialise(&mut self) -> AnalogSDKResult;

    /// A function fired to check if the plugin is currently initialised
    fn is_initialised(&mut self) -> bool;

    /// Set a callback which should be fired when a device handled by the `Plugin` is connected or disconnected
    fn set_device_event_cb(
        &mut self,
        cb: extern "C" fn(DeviceEventType, DeviceInfoPointer),
    ) -> AnalogSDKResult;

    /// Clear the device event callback
    fn clear_device_event_cb(&mut self) -> AnalogSDKResult;

    /// This function is fired by the SDK to collect up all Device Info structs. The memory for the struct should be retained and only dropped
    /// when the device is disconnected or the plugin is unloaded. This ensures that the Device Info is not garbled when it's being accessed by the client.
    ///
    /// # Notes
    ///
    /// Although, the client should be copying any data they want to use for a prolonged time as there is no lifetime guarantee on the data.
    fn device_info(&mut self, buffer: &mut [DeviceInfoPointer]) -> SDKResult<c_int>;

    /// A callback fired immediately before the plugin is unloaded. Use this if
    /// you need to do any cleanup.
    fn unload(&mut self) {}

    /// Function called to get the analog value for a particular HID key `code` from the device with ID `device`.
    /// If `device` is 0 then no specific device is specified and the value should be read from all devices and combined
    fn read_analog(&mut self, code: u16, device: DeviceID) -> SDKResult<f32>;

    /// Function called to get the full analog read buffer for a particular device with ID `device`. `max_length` is the maximum amount
    /// of keys that can be accepted, any more beyond this will be ignored by the SDK.
    fn read_full_buffer(
        &mut self,
        max_length: usize,
        device: DeviceID,
    ) -> SDKResult<HashMap<c_ushort, c_float>>;
}

/// Declare a plugin type and its constructor.
///
/// # Notes
///
/// This works by automatically generating an `extern "C"` function with a
/// pre-defined signature and symbol name. Therefore you will only be able to
/// declare one plugin per library.
#[macro_export]
macro_rules! declare_plugin {
    ($plugin_type:ty, $constructor:path) => {
        #[no_mangle]
        pub extern "C" fn _plugin_create() -> *mut $crate::Plugin {
            // make sure the constructor is the correct type.
            let constructor: fn() -> $plugin_type = $constructor;

            let object = constructor();
            let boxed: Box<$crate::Plugin> = Box::new(object);
            Box::into_raw(boxed)
        }
    };
}

/// The core `DeviceInfo` struct which contains all the interesting information
/// for a particular device
#[repr(C)]
pub struct DeviceInfo {
    /// Device Vendor ID `vid`
    pub vendor_id: u16,
    /// Device Product ID `pid`
    pub product_id: u16,
    //TODO: Consider switching these to FFiStr
    /// Device Manufacturer name
    manufacturer_name: *const c_char,
    /// Device name
    device_name: *const c_char,
    /// Unique device ID, which should be generated using `generate_device_id`
    pub device_id: DeviceID,
}

impl DeviceInfo {
    pub fn new(
        vendor_id: u16,
        product_id: u16,
        manufacturer_name: &str,
        device_name: &str,
        serial_number: &str,
    ) -> Self {
        DeviceInfo {
            vendor_id,
            product_id,
            manufacturer_name: CString::new(manufacturer_name).unwrap().into_raw(),
            device_name: CString::new(device_name).unwrap().into_raw(),
            device_id: generate_device_id(serial_number, vendor_id, product_id),
        }
    }

    pub fn new_with_id(
        vendor_id: u16,
        product_id: u16,
        manufacturer_name: &str,
        device_name: &str,
        device_id: DeviceID,
    ) -> Self {
        DeviceInfo {
            vendor_id,
            product_id,
            manufacturer_name: CString::new(manufacturer_name).unwrap().into_raw(),
            device_name: CString::new(device_name).unwrap().into_raw(),
            device_id,
        }
    }

    pub fn to_ptr(self) -> DeviceInfoPointer {
        Box::into_raw(Box::new(self)).into()
    }
}

#[derive(Clone)]
pub struct DeviceInfoPointer(pub *mut DeviceInfo);

impl Default for DeviceInfoPointer {
    fn default() -> Self {
        DeviceInfoPointer(std::ptr::null_mut())
    }
}

impl From<*mut DeviceInfo> for DeviceInfoPointer {
    fn from(ptr: *mut DeviceInfo) -> Self {
        DeviceInfoPointer(ptr)
    }
}

impl Into<*mut DeviceInfo> for DeviceInfoPointer {
    fn into(self) -> *mut DeviceInfo {
        self.0
    }
}

impl DeviceInfoPointer {
    pub fn drop(self) {
        debug!("Dropping DeviceInfoPointer");

        if self.0.is_null() {
            debug!("DeviceInfoPointer is null, ignoring");
            return;
        }

        unsafe {
            let dev: Box<DeviceInfo> = Box::from_raw(self.into());
            if !dev.device_name.is_null() {
                CString::from_raw(dev.device_name as *mut c_char);
            }
            if !dev.manufacturer_name.is_null() {
                CString::from_raw(dev.manufacturer_name as *mut c_char);
            }
        }
    }
}

enum_from_primitive! {
    #[derive(Debug, PartialEq)]
    #[repr(C)]
    pub enum KeycodeType {
        HID,
        ScanCode1,
        VirtualKey,
        VirtualKeyTranslate
    }
}

pub type DeviceID = u64;

enum_from_primitive! {
    #[derive(Debug, PartialEq)]
    #[repr(C)]
    pub enum DeviceEventType  {
        Connected = 1,
        Disconnected
    }
}

enum_from_primitive! {
    #[derive(Debug, PartialEq)]
    #[repr(C)]
    pub enum AnalogSDKResult {
        Ok = 1,
        UnInitialized = -2000,
        NoDevices,
        DeviceDisconnected,
        //Generic Failure
        Failure,
        InvalidArgument,
        NoPlugins,
        FunctionNotFound,
        //No Keycode mapping to HID was found for the given Keycode
        NoMapping

    }
}

impl AnalogSDKResult {
    pub fn is_ok(&self) -> bool {
        *self == AnalogSDKResult::Ok
    }
}

impl Default for AnalogSDKResult {
    fn default() -> Self {
        AnalogSDKResult::FunctionNotFound
    }
}

#[derive(Debug)]
pub struct SDKResult<T>(pub std::result::Result<T, AnalogSDKResult>);

impl<T> Default for SDKResult<T> {
    fn default() -> Self {
        Err(Default::default()).into()
    }
}

impl<T> Deref for SDKResult<T> {
    type Target = std::result::Result<T, AnalogSDKResult>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<std::result::Result<T, AnalogSDKResult>> for SDKResult<T> {
    fn from(ptr: std::result::Result<T, AnalogSDKResult>) -> Self {
        SDKResult(ptr)
    }
}

impl<T> Into<std::result::Result<T, AnalogSDKResult>> for SDKResult<T> {
    fn into(self) -> std::result::Result<T, AnalogSDKResult> {
        self.0
    }
}

//TODO: Figure out a way to not have to use this for the lib_wrap_option in the sdk
impl<'a> From<FfiStr<'a>> for SDKResult<FfiStr<'a>> {
    fn from(res: FfiStr<'a>) -> Self {
        Ok(res).into()
    }
}

impl From<c_int> for SDKResult<c_int> {
    fn from(res: c_int) -> Self {
        if res >= 0 {
            Ok(res).into()
        } else {
            Err(AnalogSDKResult::from_i32(res).unwrap_or(AnalogSDKResult::Failure)).into()
        }
    }
}

impl Into<c_int> for AnalogSDKResult {
    fn into(self) -> c_int {
        self as c_int
    }
}

impl Into<c_int> for SDKResult<c_int> {
    fn into(self) -> c_int {
        match self.0 {
            Ok(v) => v,
            Err(e) => e.into(),
        }
    }
}

impl From<f32> for SDKResult<f32> {
    fn from(res: f32) -> Self {
        if res >= 0.0 {
            Ok(res).into()
        } else {
            Err(AnalogSDKResult::from_f32(res).unwrap_or(AnalogSDKResult::Failure)).into()
        }
    }
}

impl Into<f32> for AnalogSDKResult {
    fn into(self) -> f32 {
        (self as i32) as f32
    }
}

impl Into<f32> for SDKResult<f32> {
    fn into(self) -> f32 {
        match self.0 {
            Ok(v) => v,
            Err(e) => e.into(),
        }
    }
}

impl<T> From<AnalogSDKResult> for SDKResult<T> {
    fn from(res: AnalogSDKResult) -> Self {
        Err(res).into()
    }
}

pub fn generate_device_id(serial_number: &str, vendor_id: u16, product_id: u16) -> DeviceID {
    use std::collections::hash_map::DefaultHasher;
    let mut s = DefaultHasher::new();
    s.write_u16(vendor_id);
    s.write_u16(product_id);
    s.write(serial_number.as_bytes());
    s.finish()
}

mod ffi {
    use super::*;

    #[no_mangle]
    pub extern "C" fn generate_device_id(
        serial_number: FfiStr,
        vendor_id: u16,
        product_id: u16,
    ) -> DeviceID {
        let serial = {
            if let Some(str) = serial_number.into_opt_string() {
                str
            } else {
                return 0;
            }
        };
        super::generate_device_id(&serial, vendor_id, product_id)
    }
}
