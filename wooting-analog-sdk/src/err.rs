//! Errors management.

use enum_primitive_derive::Primitive;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{ffi::c_int, path::PathBuf};
use thiserror::Error;

use crate::{device::DeviceID, keycode::KeycodeType};

/// Delegating from the distributable dll to the system dll.
#[derive(Error, Debug)]
pub enum DelegateError {
    #[error("dll not found")]
    DllNotFound,

    #[error("incompatible system version dll")]
    IncompatibleSystemDll,
}

#[derive(Error, Debug)]
pub enum PluginError {
    #[error("plugin failed to call \"{0}\": function not found")]
    FunctionUnavailable(&'static str),

    #[error(
        "plugin with version {version} is incompatible with the current SDK version ({expected})"
    )]
    VersionMismatch { version: u32, expected: u32 },

    #[error("dynamic plugin read failed for device: {0}")]
    InvalidRead(DeviceID),

    #[error("file \"{0:?}\" is not a valid plugin")]
    InvalidPlugin(PathBuf),

    #[error("path \"{0:?}\" is not a valid plugin directory")]
    InvalidDirectory(PathBuf),

    #[error("failed to read plugin directory: {0}")]
    IoError(#[from] std::io::Error),

    #[error("failed to load dll")]
    DynamicLibraryError {
        #[source]
        source: libloading::Error,
    },

    #[error("zero plugins available")]
    ZeroPlugins,
}

/// A more general purpose error for anything related to reading data from devices.
#[derive(Error, Debug)]
pub enum ReadError {
    #[error("the SDK or plugin was not initialized")]
    Uninitialized,

    #[error("keycode {keycode} does not map to any type using {mode:?} mode")]
    NoMapping { keycode: u16, mode: KeycodeType },

    #[error(transparent)]
    Device(#[from] DeviceError),

    #[error(transparent)]
    Plugin(#[from] PluginError),
}

impl ReadError {
    pub fn function_unavailable(name: &'static str) -> Self {
        Self::Plugin(PluginError::FunctionUnavailable(name))
    }
}

#[derive(Error, Debug)]
pub enum DeviceErrorKind {
    #[error("device disconnected")]
    Disconnected,

    #[error("unable to fetch devices")]
    ZeroDevices,

    #[error("unknown device: {0}")]
    Unknown(DeviceID),

    #[error("hid error")]
    HidError {
        #[source]
        source: hidapi::HidError,
    },
}

/// An opaque device error that could include which device the error originated from.
#[derive(Error, Debug)]
pub struct DeviceError {
    pub(crate) kind: DeviceErrorKind,
    pub(crate) device_id: Option<DeviceID>,
}

impl DeviceError {
    pub(crate) fn disconnected(id: Option<DeviceID>) -> Self {
        Self {
            kind: DeviceErrorKind::Disconnected,
            device_id: id,
        }
    }

    pub fn is_disconnected(&self) -> bool {
        matches!(self.kind, DeviceErrorKind::Disconnected)
    }

    pub(crate) fn zero_devices() -> Self {
        Self {
            kind: DeviceErrorKind::ZeroDevices,
            device_id: None,
        }
    }

    pub(crate) fn hid_err(err: hidapi::HidError) -> Self {
        Self {
            kind: DeviceErrorKind::HidError { source: err },
            device_id: None,
        }
    }

    pub(crate) fn unknown_device(device_id: DeviceID) -> Self {
        Self {
            kind: DeviceErrorKind::Unknown(device_id),
            device_id: Some(device_id),
        }
    }

    pub fn kind(&self) -> &DeviceErrorKind {
        &self.kind
    }

    pub fn device_id(&self) -> Option<DeviceID> {
        self.device_id
    }
}

impl std::fmt::Display for DeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.device_id {
            Some(id) => write!(f, "{} (id: {id})", self.kind),
            None => write!(f, "{}", self.kind),
        }
    }
}

/// FFI-safe error conversions.
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

impl From<ReadError> for WootingAnalogResult {
    fn from(err: ReadError) -> Self {
        match err {
            ReadError::Uninitialized => Self::UnInitialized,
            ReadError::NoMapping { .. } => Self::NoMapping,
            ReadError::Device(device_error) => Self::from(device_error),
            ReadError::Plugin(plugin_error) => Self::from(plugin_error),
        }
    }
}

impl From<PluginError> for WootingAnalogResult {
    fn from(err: PluginError) -> Self {
        match err {
            PluginError::FunctionUnavailable(_) => Self::FunctionNotFound,
            PluginError::VersionMismatch { .. } => Self::IncompatibleVersion,
            PluginError::InvalidRead(_) => Self::Failure,
            PluginError::InvalidPlugin(_)
            | PluginError::InvalidDirectory(_)
            | PluginError::IoError(_)
            | PluginError::ZeroPlugins => Self::NoPlugins,
            PluginError::DynamicLibraryError { .. } => Self::NotAvailable,
        }
    }
}

impl From<DeviceError> for WootingAnalogResult {
    fn from(err: DeviceError) -> Self {
        match err.kind {
            DeviceErrorKind::Disconnected => Self::DeviceDisconnected,
            DeviceErrorKind::Unknown(_) => Self::Failure,
            DeviceErrorKind::HidError { .. } => Self::Failure,
            DeviceErrorKind::ZeroDevices => Self::NoDevices,
        }
    }
}
