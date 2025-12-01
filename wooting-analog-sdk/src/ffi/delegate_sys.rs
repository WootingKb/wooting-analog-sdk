use crate::{DeviceEventType, DeviceID, DeviceInfo_FFI, WootingAnalogResult};
use libloading::Symbol;
use std::os::raw::{c_float, c_int, c_uint, c_ushort};
use std::path::PathBuf;
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
    #[cfg(target_arch = "x86")]
    let name = "wooting_analog_sdk32";

    #[cfg(not(target_arch = "x86"))]
    let name = "wooting_analog_sdk";

    let lib_path = find_dll_in_path(name).map(|p| p.join(libloading::library_filename(name)))?;

    unsafe {
        //Attempt to load the library, if it fails print the error and discard the error
        libloading::Library::new(&lib_path)
            .inspect_err(|e| {
                eprintln!("unable to load library: {:?}\nErr: {}", lib_path, e);
            })
            .ok()
    }
});

fn find_dll_in_path(name: &str) -> Option<PathBuf> {
    if let Ok(path_var) = std::env::var("PATH") {
        for dir in std::env::split_paths(&path_var) {
            if dir.is_dir() && dir.to_str().is_some_and(|d| d == name) {
                return Some(dir);
            }
        }
    }
    None
}

fn try_system_dll() -> Result<(), WootingAnalogResult> {
    if LIB.is_none() {
        return Err(WootingAnalogResult::DLLNotFound);
    }

    if wooting_analog_version() == *SDK_VERSION {
        Ok(())
    } else {
        Err(WootingAnalogResult::IncompatibleVersion)
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
    ($($fn_name:ident($($args:ident: $fn_args:ty),*) $(-> $fn_ret:ty)*;)*) => {
        $(
            #[must_use]
            pub(in crate::ffi) fn $fn_name($($args: $fn_args),*) $(-> $fn_ret)? {
                static FN: LazyLock<Option<Symbol<fn($($fn_args),*) $(-> $fn_ret)*>>> =
                    LazyLock::new(|| load_symbol(stringify!($fn_name)));

                return match FN.as_deref() {
                    Some(f) => f($($args),*),
                    _ => WootingAnalogResult::FunctionNotFound.into(),
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
    wooting_analog_read_analog(code: c_ushort) -> c_float;
    wooting_analog_read_analog_device(code: c_ushort, device_id: DeviceID) -> c_float;
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
}
