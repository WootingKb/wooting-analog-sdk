#[macro_use]
pub extern crate enum_primitive;
#[macro_use]
extern crate log;
extern crate ffi_support;

pub use enum_primitive::FromPrimitive;
use ffi_support::FfiStr;
use std::ffi::{CStr, CString};
use std::ops::Deref;
use std::os::raw::{c_char, c_int};

#[cfg(target_os = "macos")]
pub const DEFAULT_PLUGIN_DIR: &str = "/usr/local/share/WootingAnalogPlugins";
#[cfg(target_os = "linux")]
pub const DEFAULT_PLUGIN_DIR: &str = "/usr/local/share/WootingAnalogPlugins";
#[cfg(target_os = "windows")]
pub const DEFAULT_PLUGIN_DIR: &str = "C:\\Program Files\\WootingAnalogPlugins";

/// The core `DeviceInfo` struct which contains all the interesting information
/// for a particular device. This is for use internally and should be ignored if you're
/// trying to use it when trying to interact with the SDK using the wrapper
#[derive(Clone)]
pub struct DeviceInfo {
    /// Device Vendor ID `vid`
    pub vendor_id: u16,
    /// Device Product ID `pid`
    pub product_id: u16,
    /// Device Manufacturer name
    pub manufacturer_name: String,
    /// Device name
    pub device_name: String,
    /// Unique device ID, which should be generated using `generate_device_id`
    pub device_id: DeviceID,
    /// Hardware type of the Device
    pub device_type: DeviceType,
}

/// The core `DeviceInfo` struct which contains all the interesting information
/// for a particular device. This is the version which the consumer of the SDK will receive
/// through the wrapper. This is not for use in the Internal workings of the SDK, that is what
/// DeviceInfo is for
#[repr(C)]
pub struct DeviceInfo_FFI {
    /// Device Vendor ID `vid`
    pub vendor_id: u16,
    /// Device Product ID `pid`
    pub product_id: u16,
    /// Device Manufacturer name
    pub manufacturer_name: *mut c_char,
    /// Device name
    pub device_name: *mut c_char,
    /// Unique device ID, which should be generated using `generate_device_id`
    pub device_id: DeviceID,
    /// Hardware type of the Device
    pub device_type: DeviceType,
}

impl From<DeviceInfo> for DeviceInfo_FFI {
    fn from(device: DeviceInfo) -> Self {
        DeviceInfo_FFI {
            vendor_id: device.vendor_id,
            product_id: device.product_id,
            manufacturer_name: CString::new(device.manufacturer_name).unwrap().into_raw(),
            device_name: CString::new(device.device_name).unwrap().into_raw(),
            device_id: device.device_id,
            device_type: device.device_type,
        }
    }
}

impl Drop for DeviceInfo_FFI {
    fn drop(&mut self) {
        //Ensure we properly drop the memory for the char pointers
        unsafe {
            CString::from_raw(self.manufacturer_name);
            CString::from_raw(self.device_name);
        }
    }
}

impl DeviceInfo {
    //    pub fn new(
    //        vendor_id: u16,
    //        product_id: u16,
    //        manufacturer_name: &str,
    //        device_name: &str,
    //        serial_number: &str,
    //        device_type: DeviceType,
    //    ) -> Self {
    //        DeviceInfo {
    //            vendor_id,
    //            product_id,
    //            manufacturer_name,
    //            device_name,
    //            device_id: generate_device_id(serial_number, vendor_id, product_id),
    //            device_type
    //        }
    //    }

    pub fn new_with_id(
        vendor_id: u16,
        product_id: u16,
        manufacturer_name: String,
        device_name: String,
        device_id: DeviceID,
        device_type: DeviceType,
    ) -> Self {
        DeviceInfo {
            vendor_id,
            product_id,
            manufacturer_name,
            device_name,
            device_id,
            device_type,
        }
    }
}

/// Create a new device info struct. This is only for use in Plugins that are written in C
/// Rust plugins should use the native constructor
/// The memory for the struct has been allocated in Rust. So `drop_device_info` must be called
/// for the memory to be properly released
#[no_mangle]
pub extern "C" fn new_device_info(
    vendor_id: u16,
    product_id: u16,
    manufacturer_name: *mut c_char,
    device_name: *mut c_char,
    device_id: DeviceID,
    device_type: DeviceType,
) -> *mut DeviceInfo {
    Box::into_raw(Box::new(DeviceInfo::new_with_id(
        vendor_id,
        product_id,
        unsafe {
            CStr::from_ptr(manufacturer_name)
                .to_string_lossy()
                .into_owned()
        },
        unsafe { CStr::from_ptr(device_name).to_string_lossy().into_owned() },
        device_id,
        device_type,
    )))
}

/// Drops the given `DeviceInfo`
#[no_mangle]
pub unsafe extern "C" fn drop_device_info(device: *mut DeviceInfo) {
    Box::from_raw(device);
}

enum_from_primitive! {
    #[derive(Debug, PartialEq, Clone)]
    #[repr(C)]
    pub enum KeycodeType {
        /// USB HID Keycodes https://www.usb.org/document-library/hid-usage-tables-112 pg53
        HID,
        /// Scan code set 1
        ScanCode1,
        /// Windows Virtual Keys
        VirtualKey,
        /// Windows Virtual Keys which are translated to the current keyboard locale
        VirtualKeyTranslate
    }
}

pub type DeviceID = u64;

enum_from_primitive! {
    #[derive(Debug, PartialEq, Clone)]
    #[repr(C)]
    pub enum DeviceType  {
        /// Device is of type Keyboard
        Keyboard = 1,
        /// Device is of type Keypad
        Keypad,
        /// Device
        Other
    }
}

enum_from_primitive! {
    #[derive(Debug, PartialEq, Clone)]
    #[repr(C)]
    pub enum DeviceEventType  {
        /// Device has been connected
        Connected = 1,
        /// Device has been disconnected
        Disconnected
    }
}

enum_from_primitive! {
    #[derive(Debug, PartialEq, Clone)]
    #[repr(C)]
    pub enum WootingAnalogResult {
        Ok = 1,
        /// Item hasn't been initialized
        UnInitialized = -2000,
        /// No Devices are connected
        NoDevices,
        /// Device has been disconnected
        DeviceDisconnected,
        /// Generic Failure
        Failure,
        /// A given parameter was invalid
        InvalidArgument,
        /// No Plugins were found
        NoPlugins,
        /// The specified function was not found in the library
        FunctionNotFound,
        /// No Keycode mapping to HID was found for the given Keycode
        NoMapping,
        /// Indicates that it isn't available on this platform
        NotAvailable,
        /// Indicates that the operation that is trying to be used is for an older version
        IncompatibleVersion,
        /// Indicates that the Analog SDK could not be found on the system
        DLLNotFound,
    }
}

impl WootingAnalogResult {
    pub fn is_ok(&self) -> bool {
        *self == WootingAnalogResult::Ok
    }

    pub fn is_ok_or_no_device(&self) -> bool {
        *self == WootingAnalogResult::Ok || *self == WootingAnalogResult::NoDevices
    }
}

impl Default for WootingAnalogResult {
    fn default() -> Self {
        WootingAnalogResult::FunctionNotFound
    }
}

#[derive(Debug)]
pub struct SDKResult<T>(pub std::result::Result<T, WootingAnalogResult>);

impl<T> Default for SDKResult<T> {
    fn default() -> Self {
        Err(Default::default()).into()
    }
}

impl<T> Deref for SDKResult<T> {
    type Target = std::result::Result<T, WootingAnalogResult>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> From<std::result::Result<T, WootingAnalogResult>> for SDKResult<T> {
    fn from(ptr: std::result::Result<T, WootingAnalogResult>) -> Self {
        SDKResult(ptr)
    }
}

impl<T> Into<std::result::Result<T, WootingAnalogResult>> for SDKResult<T> {
    fn into(self) -> std::result::Result<T, WootingAnalogResult> {
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
            Err(WootingAnalogResult::from_i32(res).unwrap_or(WootingAnalogResult::Failure)).into()
        }
    }
}

impl Into<c_int> for WootingAnalogResult {
    fn into(self) -> c_int {
        self as c_int
    }
}

impl From<u32> for SDKResult<u32> {
    fn from(res: u32) -> Self {
        Ok(res).into()
    }
}

impl Into<i32> for SDKResult<u32> {
    fn into(self) -> i32 {
        match self.0 {
            Ok(v) => v as i32,
            Err(e) => e.into(),
        }
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
            Err(WootingAnalogResult::from_f32(res).unwrap_or(WootingAnalogResult::Failure)).into()
        }
    }
}

impl Into<f32> for WootingAnalogResult {
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

impl Into<WootingAnalogResult> for SDKResult<()> {
    fn into(self) -> WootingAnalogResult {
        match self.0 {
            Ok(_) => WootingAnalogResult::Ok,
            Err(e) => e,
        }
    }
}

impl<T> From<WootingAnalogResult> for SDKResult<T> {
    fn from(res: WootingAnalogResult) -> Self {
        Err(res).into()
    }
}

impl Into<bool> for WootingAnalogResult {
    fn into(self) -> bool {
        self == WootingAnalogResult::Ok
    }
}
