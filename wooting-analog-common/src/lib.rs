#[macro_use]
pub extern crate enum_primitive;
#[macro_use]
extern crate log;
extern crate ffi_support;

pub use enum_primitive::FromPrimitive;
use ffi_support::FfiStr;
use std::ffi::CString;
use std::ops::Deref;
use std::os::raw::{c_char, c_int};

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
    pub manufacturer_name: *const c_char,
    /// Device name
    pub device_name: *const c_char,
    /// Unique device ID, which should be generated using `generate_device_id`
    pub device_id: DeviceID,
}

impl DeviceInfo {
    /*pub fn new(
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
    }*/

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

unsafe impl Send for DeviceInfoPointer {}

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
    #[derive(Debug, PartialEq)]
    #[repr(C)]
    pub enum DeviceEventType  {
        /// Device has been connected
        Connected = 1,
        /// Device has been disconnected
        Disconnected
    }
}

enum_from_primitive! {
    #[derive(Debug, PartialEq)]
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
        NotAvailable

    }
}

impl WootingAnalogResult {
    pub fn is_ok(&self) -> bool {
        *self == WootingAnalogResult::Ok
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

impl<T> From<WootingAnalogResult> for SDKResult<T> {
    fn from(res: WootingAnalogResult) -> Self {
        Err(res).into()
    }
}
