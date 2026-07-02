use crate::{
    AnalogValue, KeyCode, KeyPosition, PhysicalKey,
    device::{DeviceEventType, DeviceID, DeviceInfo_FFI},
    err::{DelegateError, WootingAnalogResult},
};
use libloading::Symbol;
use std::os::raw::{c_char, c_float, c_int, c_uint, c_ushort};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

static SDK_VERSION: LazyLock<i32> = LazyLock::new(|| {
    env!("CARGO_PKG_VERSION")
        .split('.')
        .collect::<Vec<&str>>()
        .first()
        .and_then(|v| v.parse().ok())
        .expect("crate must have correct package semver format")
});

pub static USE_SYS_DLL: LazyLock<bool> = LazyLock::new(|| {
    try_system_dll()
        .inspect_err(|e| eprintln!("failed to delagate to system dll: {e:?}"))
        .is_ok()
});

static LIB: LazyLock<Option<libloading::Library>> = LazyLock::new(|| {
    let base_name = if cfg!(target_arch = "x86") {
        "wooting_analog_sdk32"
    } else {
        "wooting_analog_sdk"
    };

    let filename = libloading::library_filename(base_name);

    let sdk_root = if cfg!(target_os = "windows") {
        Path::new("C:/Program Files/wooting-analog-sdk/")
    } else if cfg!(target_os = "macos") {
        Path::new("/usr/local/lib/")
    } else {
        Path::new("/usr/lib/")
    };

    let lib_path = Some(sdk_root.join(&filename))
        .filter(|p| p.exists())
        .or_else(|| find_dll_in_path(filename.to_str()?))?;

    unsafe {
        //Attempt to load the library, if it fails print the error and discard the error
        libloading::Library::new(&lib_path)
            .inspect_err(|e| {
                eprintln!("unable to load library: {:?}\nErr: {}", lib_path, e);
            })
            .ok()
    }
});

fn find_dll_in_path(filename: &str) -> Option<PathBuf> {
    let search_env_var = |key: &str| -> Option<PathBuf> {
        let env_val = std::env::var_os(key)?;
        std::env::split_paths(&env_val).find(|dir| dir.join(filename).is_file())
    };

    #[cfg(target_os = "linux")]
    if let Some(path) = search_env_var("LD_LIBRARY_PATH") {
        return Some(path);
    }

    #[cfg(target_os = "macos")]
    {
        if let Some(path) = search_env_var("DYLD_LIBRARY_PATH") {
            return Some(path);
        }
        if let Some(path) = search_env_var("LD_LIBRARY_PATH") {
            return Some(path);
        }
    }

    search_env_var("PATH")
}

fn try_system_dll() -> Result<(), DelegateError> {
    if LIB.is_none() {
        return Err(DelegateError::DllNotFound);
    }

    if wooting_analog_version() == *SDK_VERSION {
        Ok(())
    } else {
        Err(DelegateError::IncompatibleSystemDll)
    }
}

fn load_symbol<T>(name: &str) -> Option<Symbol<'_, T>> {
    LIB.as_ref().and_then(|lib| unsafe {
        lib.get::<T>(name.as_bytes())
            .inspect_err(|e| {
                eprintln!("failed to load symbol '{}': {}", name, e);
            })
            .ok()
    })
}

macro_rules! delegate_sys {
    ($($fn_name:ident($($args:ident: $fn_args:ty),*) -> *const c_char;)*) => {
        $(
            #[must_use]
            pub(in crate::ffi) fn $fn_name($($args: $fn_args),*) -> *const c_char {
                static FN: LazyLock<Option<Symbol<fn($($fn_args),*) -> *const c_char>>> =
                    LazyLock::new(|| load_symbol(stringify!($fn_name)));

                return match FN.as_deref() {
                    Some(f) => f($($args),*),
                    None => std::ptr::null(),
                };
            }
        )*
    };

    ($($fn_name:ident($($args:ident: $fn_args:ty),*) $(-> $fn_ret:ty)*;)*) => {
        $(
            #[must_use]
            pub(in crate::ffi) fn $fn_name($($args: $fn_args),*) $(-> $fn_ret)? {
                static FN: LazyLock<Option<Symbol<fn($($fn_args),*) $(-> $fn_ret)*>>> =
                    LazyLock::new(|| load_symbol(stringify!($fn_name)));

                return match FN.as_deref() {
                    Some(f) => f($($args),*),
                    None => WootingAnalogResult::FunctionNotFound.into(),
                };
            }
        )*
    };
}

delegate_sys! {
    wooting_analog_initialise() -> c_int;
    wooting_analog_version() -> c_int;
    wooting_analog_is_initialised() -> bool;
    wooting_analog_uninitialise() -> WootingAnalogResult;
    wooting_analog_set_keycode_mode(mode: c_uint) -> WootingAnalogResult;
    wooting_analog_read_analog_device(code: c_ushort, device_id: DeviceID) -> c_float;
    wooting_analog_read_keycode_device(
        keycode: c_ushort,
        value: *mut AnalogValue,
        device_id: DeviceID
    ) -> WootingAnalogResult;
    wooting_analog_read_position_device(
        position: *const KeyPosition,
        physical_key: *mut PhysicalKey,
        device_id: DeviceID
    ) -> WootingAnalogResult;
    wooting_analog_set_device_event_cb(
        cb: extern "C" fn(DeviceEventType, *mut DeviceInfo_FFI)
    ) -> WootingAnalogResult;
    wooting_analog_clear_device_event_cb() -> WootingAnalogResult;
    wooting_analog_get_connected_devices_info(
        buffer: *mut *mut DeviceInfo_FFI,
        len: c_uint
    ) -> c_int;
    wooting_analog_read_full_buffer_device(
        code_buffer: *mut c_ushort,
        analog_buffer: *mut c_float,
        len: c_uint,
        device_id: DeviceID
    ) -> c_int;
    wooting_analog_read_full_buffer_v2_device(
        code_buffer: *mut KeyCode,
        analog_buffer: *mut AnalogValue,
        len: c_uint,
        device_id: DeviceID
    ) -> c_int;
    wooting_analog_read_positions_device(
        physical_keys: *mut PhysicalKey,
        len: c_uint,
        device_id: DeviceID
    ) -> c_int;
}

delegate_sys! {
    wooting_analog_version_semver() -> *const c_char;
}
