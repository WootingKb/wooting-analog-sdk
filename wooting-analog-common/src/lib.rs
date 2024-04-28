#[macro_use]
extern crate enum_primitive_derive;
extern crate ffi_support;
extern crate num_traits;

use std::convert::TryFrom;
pub use num_traits::{FromPrimitive, ToPrimitive};
#[cfg(feature = "serdes")]
use serde::{Deserialize, Serialize};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_float, c_int};
use thiserror::Error;

#[cfg(target_os = "macos")]
pub const DEFAULT_PLUGIN_DIR: &str = "/usr/local/share/WootingAnalogPlugins";
#[cfg(target_os = "linux")]
pub const DEFAULT_PLUGIN_DIR: &str = "/usr/local/share/WootingAnalogPlugins";
#[cfg(target_os = "windows")]
pub const DEFAULT_PLUGIN_DIR: &str = "C:\\Program Files\\WootingAnalogPlugins";

/// The core `DeviceInfo` struct which contains all the interesting information
/// for a particular device. This is for use internally and should be ignored if you're
/// trying to use it when trying to interact with the SDK using the wrapper
#[cfg_attr(feature = "serdes", derive(Serialize, Deserialize))]
#[derive(Clone, Debug)]
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
            let _c_string = CString::from_raw(self.manufacturer_name);
            let _c_string = CString::from_raw(self.device_name);
        }
    }
}

impl DeviceInfo_FFI {
    pub fn into_device_info(&self) -> DeviceInfo {
        DeviceInfo {
            vendor_id: self.vendor_id.clone(),
            product_id: self.product_id.clone(),
            // In this case we use CStr rather than CString as we don't want the memory to be dropped here which may cause a double free
            // We leave it up to ffi interface to drop the memory
            manufacturer_name: unsafe {
                CStr::from_ptr(self.manufacturer_name)
                    .to_str()
                    .unwrap()
                    .to_owned()
            },
            device_name: unsafe {
                CStr::from_ptr(self.device_name)
                    .to_str()
                    .unwrap()
                    .to_owned()
            },
            device_id: self.device_id.clone(),
            device_type: self.device_type.clone(),
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
    drop(Box::from_raw(device));
}

#[cfg_attr(feature = "serdes", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Clone, Primitive)]
#[repr(C)]
pub enum KeycodeType {
    /// USB HID Keycodes https://www.usb.org/document-library/hid-usage-tables-112 pg53
    HID = 0,
    /// Scan code set 1
    ScanCode1 = 1,
    /// Windows Virtual Keys
    VirtualKey = 2,
    /// Windows Virtual Keys which are translated to the current keyboard locale
    VirtualKeyTranslate = 3,
}

pub type DeviceID = u64;

#[cfg_attr(feature = "serdes", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Clone, Primitive)]
#[repr(C)]
pub enum DeviceType {
    /// Device is of type Keyboard
    Keyboard = 1,
    /// Device is of type Keypad
    Keypad = 2,
    /// Device
    Other = 3,
}

#[cfg_attr(feature = "serdes", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Clone, Primitive)]
#[repr(C)]
pub enum DeviceEventType {
    /// Device has been connected
    Connected = 1,
    /// Device has been disconnected
    Disconnected = 2,
}

#[cfg_attr(feature = "serdes", derive(Serialize, Deserialize))]
#[derive(Debug, Default, PartialEq, Clone, Primitive, Error)]
#[repr(C)]
pub enum WootingAnalogResult {
    #[error("All OK")]
    Ok = 1,
    /// Item hasn't been initialized
    #[error("SDK has not been initialized")]
    UnInitialized = -2000isize,
    /// No Devices are connected
    #[error("No Devices are connected")]
    NoDevices = -1999isize,
    /// Device has been disconnected
    #[error("Device has been disconnected")]
    DeviceDisconnected = -1998isize,
    /// Generic Failure
    #[error("Generic Failure")]
    Failure = -1997isize,
    /// A given parameter was invalid
    #[error("A given parameter was invalid")]
    InvalidArgument = -1996isize,
    /// No Plugins were found
    #[error("No Plugins were found")]
    NoPlugins = -1995isize,
    /// The specified function was not found in the library
    #[default]
    #[error("The specified function was not found in the library")]
    FunctionNotFound = -1994isize,
    /// No Keycode mapping to HID was found for the given Keycode
    #[error("No Keycode mapping to HID was found for the given Keycode")]
    NoMapping = -1993isize,
    /// Indicates that it isn't available on this platform
    #[error("Unavailable on this platform")]
    NotAvailable = -1992isize,
    /// Indicates that the operation that is trying to be used is for an older version
    #[error("Incompatible SDK Version")]
    IncompatibleVersion = -1991isize,
    /// Indicates that the Analog SDK could not be found on the system
    #[error("The Wooting Analog SDK could not be found on the system")]
    DLLNotFound = -1990isize,
}

impl WootingAnalogResult {
    pub fn is_ok(&self) -> bool {
        *self == WootingAnalogResult::Ok
    }

    pub fn is_ok_or_no_device(&self) -> bool {
        *self == WootingAnalogResult::Ok || *self == WootingAnalogResult::NoDevices
    }

    pub fn into_sdk_result(self) -> SDKResult<()> {
        if self.is_ok() { Ok(()) } else { Err(self) }
    }
}

pub type SDKResult<T> = Result<T, WootingAnalogResult>;

pub trait IntoSDKResultExt<T> {
    fn into_sdk_result(self) -> SDKResult<T>;
}

impl IntoSDKResultExt<u32> for c_int {
    fn into_sdk_result(self) -> SDKResult<u32> {
        u32::try_from(self).map_err(|_|
            WootingAnalogResult::try_from(self)
                .unwrap_or(WootingAnalogResult::Failure)
        )
    }
}

impl IntoSDKResultExt<f32> for f32 {
    fn into_sdk_result(self) -> SDKResult<f32> {
        match self {
            //the RFC was made with matching exact values in mind,
            //to prevent problems with structural vs semantic equality (e.g. -0 vs +0).
            //there was never any consensus about how ranges should be treated.
            //none of this matters though, as the RFC got rejected. the lint just hasn't been removed yet
            #[allow(illegal_floating_point_literal_pattern)]
            0_f32..=1_f32 => Ok(self),
            _ => (self as c_int).into_sdk_result().map(|v| v as f32) //wtf
        }
    }
}

pub trait IntoWootingAnalogResultExt {
    fn into_wooting_analog_result(self) -> WootingAnalogResult;
}

impl IntoWootingAnalogResultExt for SDKResult<()> {
    fn into_wooting_analog_result(self) -> WootingAnalogResult {
        self.map(|_| WootingAnalogResult::Ok).unwrap_or_else(|e| e) //for some reason rust doesnt have .unwrap_err_or()
    }
}

pub trait IntoCResultExt<T> {
    fn into_c_result(self) -> T;
}

impl IntoCResultExt<c_int> for SDKResult<u32> {
    fn into_c_result(self) -> c_int {
        self.map(|value| value as _)
            .unwrap_or_else(|error| error as _)
    }
}

impl IntoCResultExt<c_float> for SDKResult<f32> {
    fn into_c_result(self) -> c_float {
        self.map(|value| value as _)
            .unwrap_or_else(|error| error as isize as _)
    }
}

impl Into<c_int> for WootingAnalogResult {
    fn into(self) -> c_int {
        self as c_int
    }
}

impl Into<f32> for WootingAnalogResult {
    fn into(self) -> f32 {
        (self as i32) as f32
    }
}

impl Into<bool> for WootingAnalogResult {
    fn into(self) -> bool {
        self == WootingAnalogResult::Ok
    }
}

#[cfg_attr(feature = "serdes", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Clone, Hash, Eq, Primitive)]
#[repr(C)]
pub enum HIDCodes {
    A = 0x04,
    B = 0x05, //US_B
    C = 0x06, //US_C
    D = 0x07, //US_D

    E = 0x08, //US_E
    F = 0x09, //US_F
    G = 0x0a, //US_G
    H = 0x0b, //US_H
    I = 0x0c, //US_I
    J = 0x0d, //US_J
    K = 0x0e, //US_K
    L = 0x0f, //US_L

    M = 0x10, //US_M
    N = 0x11, //US_N
    O = 0x12, //US_O
    P = 0x13, //US_P
    Q = 0x14, //US_Q
    R = 0x15, //US_R
    S = 0x16, //US_S
    T = 0x17, //US_T

    U = 0x18,  //US_U
    V = 0x19,  //US_V
    W = 0x1a,  //US_W
    X = 0x1b,  //US_X
    Y = 0x1c,  //US_Y
    Z = 0x1d,  //US_Z
    N1 = 0x1e, //DIGIT1
    N2 = 0x1f, //DIGIT2

    N3 = 0x20, //DIGIT3
    N4 = 0x21, //DIGIT4
    N5 = 0x22, //DIGIT5
    N6 = 0x23, //DIGIT6
    N7 = 0x24, //DIGIT7
    N8 = 0x25, //DIGIT8
    N9 = 0x26, //DIGIT9
    N0 = 0x27, //DIGIT0

    Enter = 0x28,       //ENTER
    Escape = 0x29,      //ESCAPE
    Backspace = 0x2a,   //BACKSPACE
    Tab = 0x2b,         //TAB
    Space = 0x2c,       //SPACE
    Minus = 0x2d,       //MINUS
    Equal = 0x2e,       //EQUAL
    BracketLeft = 0x2f, //BRACKET_LEFT

    BracketRight = 0x30, //BRACKET_RIGHT
    Backslash = 0x31,    //BACKSLASH

    // = 0x32, //INTL_HASH
    Semicolon = 0x33, //SEMICOLON
    Quote = 0x34,     //QUOTE
    Backquote = 0x35, //BACKQUOTE
    Comma = 0x36,     //COMMA
    Period = 0x37,    //PERIOD

    Slash = 0x38,    //SLASH
    CapsLock = 0x39, //CAPS_LOCK
    F1 = 0x3a,       //F1
    F2 = 0x3b,       //F2
    F3 = 0x3c,       //F3
    F4 = 0x3d,       //F4
    F5 = 0x3e,       //F5
    F6 = 0x3f,       //F6

    F7 = 0x40,          //F7
    F8 = 0x41,          //F8
    F9 = 0x42,          //F9
    F10 = 0x43,         //F10
    F11 = 0x44,         //F11
    F12 = 0x45,         //F12
    PrintScreen = 0x46, //PRINT_SCREEN
    ScrollLock = 0x47,  //SCROLL_LOCK

    PauseBreak = 0x48, //PAUSE
    Insert = 0x49,     //INSERT
    Home = 0x4a,       //HOME
    PageUp = 0x4b,     //PAGE_UP
    Delete = 0x4c,     //DEL
    End = 0x4d,        //END
    PageDown = 0x4e,   //PAGE_DOWN
    ArrowRight = 0x4f, //ARROW_RIGHT

    ArrowLeft = 0x50,      //ARROW_LEFT
    ArrowDown = 0x51,      //ARROW_DOWN
    ArrowUp = 0x52,        //ARROW_UP
    NumLock = 0x53,        //NUM_LOCK
    NumpadDivide = 0x54,   //NUMPAD_DIVIDE
    NumpadMultiply = 0x55, //NUMPAD_MULTIPLY
    NumpadSubtract = 0x56, //NUMPAD_SUBTRACT
    NumpadAdd = 0x57,      //NUMPAD_ADD

    NumpadEnter = 0x58, //NUMPAD_ENTER
    Numpad1 = 0x59,     //NUMPAD1
    Numpad2 = 0x5a,     //NUMPAD2
    Numpad3 = 0x5b,     //NUMPAD3
    Numpad4 = 0x5c,     //NUMPAD4
    Numpad5 = 0x5d,     //NUMPAD5
    Numpad6 = 0x5e,     //NUMPAD6
    Numpad7 = 0x5f,     //NUMPAD7

    Numpad8 = 0x60,                //NUMPAD8
    Numpad9 = 0x61,                //NUMPAD9
    Numpad0 = 0x62,                //NUMPAD0
    NumpadDecimal = 0x63,          //NUMPAD_DECIMAL
    InternationalBackslash = 0x64, //INTL_BACKSLASH
    ContextMenu = 0x65,            //CONTEXT_MENU
    Power = 0x66,                  //POWER
    NumpadEqual = 0x67,            //NUMPAD_EQUAL

    F13 = 0x68, //F13
    F14 = 0x69, //F14
    F15 = 0x6a, //F15
    F16 = 0x6b, //F16
    F17 = 0x6c, //F17
    F18 = 0x6d, //F18
    F19 = 0x6e, //F19
    F20 = 0x6f, //F20

    F21 = 0x70, //F21
    F22 = 0x71, //F22
    F23 = 0x72, //F23

    F24 = 0x73,  //F24
    Open = 0x74, //OPEN

    Help = 0x75, //HELP

    // = 0x77, //SELECT
    Again = 0x79,      //AGAIN
    Undo = 0x7a,       //UNDO
    Cut = 0x7b,        //CUT
    Copy = 0x7c,       //COPY
    Paste = 0x7d,      //PASTE
    Find = 0x7e,       //FIND
    VolumeMute = 0x7f, //VOLUME_MUTE

    VolumeUp = 0x80,    //VOLUME_UP
    VolumeDown = 0x81,  //VOLUME_DOWN
    NumpadComma = 0x85, //NUMPAD_COMMA

    InternationalRO = 0x87,  //INTL_RO
    KanaMode = 0x88,         //KANA_MODE
    InternationalYen = 0x89, //INTL_YEN
    Convert = 0x8a,          //CONVERT
    NonConvert = 0x8b,       //NON_CONVERT
    Lang1 = 0x90,            //LANG1
    Lang2 = 0x91,            //LANG2
    Lang3 = 0x92,            //LANG3
    Lang4 = 0x93,            //LANG4

    LeftCtrl = 0xe0,   //CONTROL_LEFT
    LeftShift = 0xe1,  //SHIFT_LEFT
    LeftAlt = 0xe2,    //ALT_LEFT
    LeftMeta = 0xe3,   //META_LEFT
    RightCtrl = 0xe4,  //CONTROL_RIGHT
    RightShift = 0xe5, //SHIFT_RIGHT
    RightAlt = 0xe6,   //ALT_RIGHT
    RightMeta = 0xe7,  //META_RIGHT
}
