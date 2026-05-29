use enum_primitive_derive::Primitive;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::ffi::{c_float, c_int};
use thiserror::Error;

pub type WootingResult<T> = std::result::Result<T, WootingAnalogResult>;

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Default, PartialEq, Clone, Primitive, Error, Copy)]
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
    #[error("The specified function was not found in the library")]
    #[default]
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
}

impl From<WootingAnalogResult> for c_int {
    fn from(value: WootingAnalogResult) -> Self {
        value as c_int
    }
}

impl From<WootingAnalogResult> for bool {
    fn from(value: WootingAnalogResult) -> Self {
        value == WootingAnalogResult::Ok
    }
}

impl From<WootingAnalogResult> for f32 {
    fn from(value: WootingAnalogResult) -> Self {
        (value as i32) as f32
    }
}

impl<T> From<WootingResult<T>> for WootingAnalogResult {
    fn from(result: WootingResult<T>) -> Self {
        match result {
            Ok(_) => WootingAnalogResult::Ok,
            Err(err) => err,
        }
    }
}

pub(crate) trait IntoFfiCount {
    fn into_ffi_count(self) -> c_int;
}

impl IntoFfiCount for WootingResult<u32> {
    fn into_ffi_count(self) -> c_int {
        match self {
            Ok(count) => count as c_int,
            Err(err) => err as c_int,
        }
    }
}

pub(crate) trait IntoFfiAnalogValue {
    fn into_ffi_analog_value(self) -> c_float;
}

impl IntoFfiAnalogValue for WootingResult<f32> {
    fn into_ffi_analog_value(self) -> c_float {
        match self {
            Ok(value) => value as c_float,
            Err(err) => (err as i32) as c_float,
        }
    }
}
