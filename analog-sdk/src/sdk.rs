use crate::errors::*;
use crate::keycode::*;
use analog_sdk_common::*;
use ffi_support::FfiStr;
use libloading::{Library, Symbol};
use log::{error, info, warn};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::os::raw::{c_float, c_int, c_uint, c_ushort};
use std::path::{Path, PathBuf};

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
                            _ => Err(AnalogSDKResult::FunctionNotFound).into()

                        }
                    }
                }
            //}
        )*
    };
}

struct CPlugin {
    lib: Library,
    //funcs: HashMap<&'static str, Option<Symbol>>
}

impl CPlugin {
    fn new(lib: Library) -> CPlugin {
        CPlugin {
            lib,
            //funcs: HashMap::new()
        }
    }

    lib_wrap_option! {
        //c_name has to be over here due to it not being part of the Plugin trait
        fn _name() -> FfiStr<'static>;

        fn _read_full_buffer(code_buffer: *const c_ushort, analog_buffer: *const c_float, len: c_uint, device: DeviceID) -> c_int;
        fn _device_info(buffer: *mut DeviceInfoPointer, len: c_uint) -> c_int;
    }
}

impl Plugin for CPlugin {
    fn name(&mut self) -> SDKResult<&'static str> {
        /*let s = self.c_name();
        let c_str = unsafe {
            assert!(!s.is_null());

            CStr::from_ptr(s)
        };

        c_str.to_str().unwrap()*/
        self._name().0.map(|s| s.as_str()).into()
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
                ._read_full_buffer(
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

    fn device_info(&mut self, buffer: &mut [DeviceInfoPointer]) -> SDKResult<c_int> {
        self._device_info(buffer.as_mut_ptr(), buffer.len() as c_uint)
    }

    lib_wrap! {
        fn initialise() -> AnalogSDKResult;
        fn is_initialised() -> bool;
        fn unload();
        fn set_device_event_cb(cb: extern fn(DeviceEventType, DeviceInfoPointer)) -> AnalogSDKResult;
        fn clear_device_event_cb() -> AnalogSDKResult;
    }
    lib_wrap_option! {
        fn read_analog(code: u16, device: DeviceID) -> f32;
        //fn neg(x: u32, y: u32) -> u32;
    }
}

unsafe impl Send for AnalogSDK {}

pub struct AnalogSDK {
    pub initialised: bool,
    //pub disconnected_callback: Option<extern fn(FfiStr)>,
    pub keycode_mode: KeycodeType,

    plugins: Vec<Box<Plugin>>,
    loaded_libraries: Vec<Library>,
    //pub device_info: *mut DeviceInfo
}

#[cfg(target_os = "macos")]
static LIB_EXT: &str = "dylib";
#[cfg(target_os = "linux")]
static LIB_EXT: &str = "so";
#[cfg(target_os = "windows")]
static LIB_EXT: &str = "dll";

const ENV_PLUGIN_DIR_KEY: &str = "ANALOG_SDK_PLUGINS_PATH";
const DEFAULT_PLUGIN_DIR: &str = "~/.analog_plugins";

impl AnalogSDK {
    pub fn new() -> AnalogSDK {
        AnalogSDK {
            plugins: Vec::new(),
            loaded_libraries: Vec::new(),
            initialised: false,
            //disconnected_callback: None,
            keycode_mode: KeycodeType::HID, //device_info: Box::into_raw(Box::new(DeviceInfo { name: b"Device Yeet\0" as *const u8, val:20 }))
        }
    }

    pub fn initialise(&mut self) -> AnalogSDKResult {
        if self.initialised {
            self.unload();
        }
        let plugin_dir: std::result::Result<Vec<PathBuf>, std::env::VarError> =
            std::env::var(ENV_PLUGIN_DIR_KEY).map(|var| {
                var.split(':')
                    .filter_map(|path| {
                        if path.is_empty() {
                            None
                        } else {
                            Some(PathBuf::from(path))
                        }
                    })
                    .collect()
            });
        let mut plugin_dir = match plugin_dir {
            Ok(v) => {
                info!(
                    "Found ${}, loading plugins from {:?}",
                    ENV_PLUGIN_DIR_KEY, v
                );
                v
            }
            Err(e) => {
                warn!(
                    "{} is not set, defaulting to {}.\nError: {}",
                    ENV_PLUGIN_DIR_KEY, DEFAULT_PLUGIN_DIR, e
                );
                vec![PathBuf::from(String::from(DEFAULT_PLUGIN_DIR))]
            }
        };
        for dir in plugin_dir.drain(..) {
            match self.load_plugins(&dir) {
                Ok(0) => {
                    warn!("Failed to load any plugins from {:?}!", dir);
                    self.initialised = false;
                    //AnalogSDKResult::NoPlugins
                }
                Ok(i) => {
                    info!("Loaded {} plugins from {:?}", i, dir);
                    //AnalogSDKResult::Ok
                }
                Err(e) => {
                    error!("Error: {:?}", e);
                    self.initialised = false;
                }
            }
        }

        let mut plugins_initialised = 0;
        for p in self.plugins.iter_mut() {
            if p.initialise().is_ok() {
                plugins_initialised += 1;
            }
        }
        info!("{} plugins successfully initialised", plugins_initialised);

        self.initialised = plugins_initialised > 0;
        if !self.initialised {
            AnalogSDKResult::NoPlugins
        } else {
            AnalogSDKResult::Ok
        }
    }

    fn load_plugins(&mut self, dir: &Path) -> Result<u32> {
        if dir.is_dir() {
            let mut i: u32 = 0;
            for entry in fs::read_dir(dir)
                .chain_err(|| format!("Unable to load dir \"{}\"", dir.display()))?
            {
                let path = entry.chain_err(|| "Err with entry")?.path();

                if let Some(ext) = path.extension().and_then(OsStr::to_str) {
                    if ext == LIB_EXT {
                        info!("Attempting to load plugin: \"{}\"", path.display());
                        unsafe {
                            self.load_plugin(&path)?; //.map_err(|e| error!("{:?}", e));
                        }
                        i += 1;
                    }
                }
            }
            return Ok(i);
        }

        bail!("Path: {:?} is not a dir!", dir)
    }

    unsafe fn load_plugin(&mut self, filename: &Path) -> Result<()> {
        if filename.is_dir() {
            return Err("Path is directory!".into());
        }

        type PluginCreate = unsafe extern "C" fn() -> *mut Plugin;

        let lib = Library::new(filename.as_os_str()).chain_err(|| "Unable to load the plugin")?;

        // We need to keep the library around otherwise our plugin's vtable will
        // point to garbage. We do this little dance to make sure the library
        // doesn't end up getting moved.
        self.loaded_libraries.push(lib);

        let lib = self.loaded_libraries.last().unwrap();

        let version: Option<Symbol<*mut u32>> = lib.get(b"ANALOG_SDK_PLUGIN_ABI_VERSION").ok();
        if let Some(ver) = version {
            info!("We got version: {}", **ver);
        }

        let constructor: Option<Symbol<PluginCreate>> = lib
            .get(b"_plugin_create")
            .map_err(|e| {
                error!("Find constructor error: {}", e);
            })
            .ok();
        //    .chain_err(|| "The `_plugin_create` symbol wasn't found.");

        let mut plugin = match constructor {
            Some(f) => {
                debug!("We got it and we're trying");
                Box::from_raw(f())
            }
            None => {
                warn!("Didn't find _plugin_create, assuming it's a c plugin");
                let lib = self.loaded_libraries.pop().unwrap();
                Box::new(CPlugin::new(lib))
            }
        };

        info!("Loaded plugin: {:?}", plugin.name());
        //plugin.on_plugin_load();
        self.plugins.push(plugin);

        Ok(())
    }

    pub fn set_device_event_cb(
        &mut self,
        cb: extern "C" fn(DeviceEventType, DeviceInfoPointer),
    ) -> AnalogSDKResult {
        if !self.initialised {
            return AnalogSDKResult::UnInitialized;
        }

        let mut result = AnalogSDKResult::Ok;
        for p in self.plugins.iter_mut() {
            let ret = p.set_device_event_cb(cb);
            if ret != AnalogSDKResult::Ok {
                result = ret;
            }
        }
        result
    }

    pub fn clear_device_event_cb(&mut self) -> AnalogSDKResult {
        if !self.initialised {
            return AnalogSDKResult::UnInitialized;
        }

        let mut result = AnalogSDKResult::Ok;
        for p in self.plugins.iter_mut() {
            let ret = p.clear_device_event_cb();
            if ret != AnalogSDKResult::Ok {
                result = ret;
            }
        }
        result
    }

    pub fn get_device_info(&mut self, buffer: &mut [DeviceInfoPointer]) -> SDKResult<c_int> {
        if !self.initialised {
            return AnalogSDKResult::UnInitialized.into();
        }
        let mut count: usize = 0;
        for p in self.plugins.iter_mut() {
            let num = p.device_info(&mut buffer[count..]).0.unwrap_or(0) as usize;
            if num > 0 {
                count += num
            }
        }
        (count as c_int).into()
    }

    pub fn read_analog(&mut self, code: u16, device_id: DeviceID) -> SDKResult<f32> {
        if !self.initialised {
            return AnalogSDKResult::UnInitialized.into();
        }

        let hid_code = code_to_hid(code, &self.keycode_mode);
        if let Some(hid_code) = hid_code {
            let mut value: f32 = -1.0;
            let mut err = AnalogSDKResult::Ok;
            for p in self.plugins.iter_mut() {
                match p.read_analog(hid_code, device_id).into() {
                    Ok(x) => {
                        value = value.max(x);
                        //If we were looking to read from a specific device, we've found that read, so no need to continue
                        if device_id != 0 {
                            break;
                        }
                    }
                    Err(e) => {
                        //TODO: Improve collating of multiple errors
                        err = e
                    }
                }
            }

            if value < 0.0 {
                return err.into();
            }

            return value.into();
        } else {
            return AnalogSDKResult::NoMapping.into();
        }
    }

    pub fn read_full_buffer(
        &mut self,
        code_buffer: &mut [c_ushort],
        analog_buffer: &mut [c_float],
        device_id: DeviceID,
    ) -> SDKResult<c_int> {
        if !self.initialised {
            return AnalogSDKResult::UnInitialized.into();
        }

        let mut analog_data: HashMap<c_ushort, c_float> = HashMap::with_capacity(code_buffer.len());

        let mut err = AnalogSDKResult::Ok;
        let mut any_success = false;
        //Read from all and add up
        for p in self.plugins.iter_mut() {
            let plugin_data = p.read_full_buffer(code_buffer.len(), 0).into();
            match plugin_data {
                Ok(mut data) => {
                    for (hid_code, analog) in data.drain() {
                        if analog == 0.0 {
                            continue;
                        }
                        let code = hid_to_code(hid_code, &self.keycode_mode);
                        if let Some(code) = code {
                            let mut total_analog = analog;

                            //No point in checking if the value is already present if we are only looking for data from one device
                            if device_id == 0 {
                                if let Some(val) = analog_data.get(&code) {
                                    total_analog = total_analog.max(*val);
                                }
                            }
                            analog_data.insert(code, total_analog);
                        } else {
                            //warn!("Couldn't map HID:{} to {:?}", hid_code, self.keycode_mode);
                        }
                    }

                    any_success = true;
                }
                Err(e) => {
                    //TODO: Improve collating of multiple errors
                    err = e
                }
            }
            //If we are looking for a specific device, just break out when we find one that returns good
            if device_id != 0 {
                break;
            }
        }
        if !any_success {
            return err.into();
        }

        //Fill up given slices
        let mut count: usize = 0;
        for (code, analog) in analog_data.drain() {
            if count >= code_buffer.len() {
                break;
            }

            code_buffer[count] = code;
            analog_buffer[count] = analog;
            count += 1;
        }
        (count as c_int).into()
    }

    /// Unload all plugins and loaded plugin libraries, making sure to fire
    /// their `on_plugin_unload()` methods so they can do any necessary cleanup.
    pub fn unload(&mut self) {
        debug!("Unloading plugins");

        for mut plugin in self.plugins.drain(..) {
            trace!("Firing on_plugin_unload for {:?}", plugin.name());
            plugin.unload();
        }

        for lib in self.loaded_libraries.drain(..) {
            drop(lib);
        }
    }
}

impl Drop for AnalogSDK {
    fn drop(&mut self) {
        if !self.plugins.is_empty() || !self.loaded_libraries.is_empty() {
            self.unload();
        }
    }
}

impl Default for AnalogSDK {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shared_init() {
        use env_logger::Env;
        let env = Env::new().default_filter_or("trace");
        env_logger::try_init_from_env(env);
    }

    #[test]
    fn initialise_no_plugins() {
        shared_init();

        let dir = "./test_np";
        ::std::fs::create_dir(dir);
        ::std::env::set_var(ENV_PLUGIN_DIR_KEY, dir);

        let mut sdk = AnalogSDK::new();
        assert_eq!(sdk.initialise(), AnalogSDKResult::NoPlugins);
        assert!(!sdk.initialised);
        ::std::fs::remove_dir(dir);
    }

    #[test]
    fn initialise_no_dir() {
        shared_init();

        let dir = "./test_n";
        ::std::fs::remove_dir(dir);
        ::std::env::set_var(ENV_PLUGIN_DIR_KEY, dir);

        let mut sdk = AnalogSDK::new();
        assert_eq!(sdk.initialise(), AnalogSDKResult::NoPlugins);
        assert!(!sdk.initialised)
    }

    #[test]
    fn initialise_multiple_dir() {
        shared_init();

        let dir = "./test:./test1";
        ::std::fs::create_dir("./test_m1");
        ::std::fs::create_dir("./test_m2");
        ::std::env::set_var(ENV_PLUGIN_DIR_KEY, dir);

        let mut sdk = AnalogSDK::new();
        assert_eq!(sdk.initialise(), AnalogSDKResult::NoPlugins);
        assert!(!sdk.initialised);

        ::std::fs::remove_dir("./test_m1");
        ::std::fs::remove_dir("./test_m2");
    }

    #[test]
    fn unitialised_sdk_functions_new() {
        shared_init();

        let mut sdk = AnalogSDK::new();
        uninitialised_sdk_functions(&mut sdk);
    }

    #[test]
    fn unitialised_sdk_functions_failed_init() {
        shared_init();

        let mut sdk = AnalogSDK::new();
        let dir = "./test_n";
        ::std::fs::remove_dir(dir);
        ::std::env::set_var(ENV_PLUGIN_DIR_KEY, dir);

        assert_eq!(sdk.initialise(), AnalogSDKResult::NoPlugins);
        assert!(!sdk.initialised);

        uninitialised_sdk_functions(&mut sdk);
    }

    extern "C" fn cb(event: DeviceEventType, device: DeviceInfoPointer) {}

    fn uninitialised_sdk_functions(sdk: &mut AnalogSDK) {
        assert_eq!(sdk.set_device_event_cb(cb), AnalogSDKResult::UnInitialized);
        assert_eq!(sdk.clear_device_event_cb(), AnalogSDKResult::UnInitialized);
        assert_eq!(sdk.read_analog(0, 0).0, Err(AnalogSDKResult::UnInitialized));
        let mut buf = vec![];
        assert_eq!(
            sdk.get_device_info(buf.as_mut()).0,
            Err(AnalogSDKResult::UnInitialized)
        );

        let mut buf = vec![];
        let mut buf2 = vec![];
        assert_eq!(
            sdk.read_full_buffer(buf.as_mut(), buf2.as_mut(), 0).0,
            Err(AnalogSDKResult::UnInitialized)
        );
    }
}
