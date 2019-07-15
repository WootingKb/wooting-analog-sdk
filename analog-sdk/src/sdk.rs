use std::any::Any;
use libloading::{Library, Symbol};
use std::fs;
use std::path::{Path, PathBuf};
use crate::errors::*;
//use libc::c_char;
use std::ffi::{CString, OsStr};
use std::os::raw::{c_uint, c_int, c_float, c_ushort};
use ffi_support::{FfiStr};
use std::collections::HashMap;
use crate::keycode::*;
//use std::ops::Deref;

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
                        /*let func = self.funcs.get(stringify!($fn_names));
                        let func :  Option<Symbol<FnPtr>> = match func {
                            Some(x) => {
                                x
                            },
                            None => {
                                self.funcs.insert(stringify!($fn_names), self.lib.get(stringify!($fn_names).as_bytes()).map_err(|e| {
                                    println!("{}", e);
                                }).ok());
                                self.funcs.get(stringify!($fn_names)).unwrap()
                            }
                        };*/

                          
                        let func :  Option<Symbol<FnPtr>>  = self.lib.get(stringify!($fn_names).as_bytes()).map_err(|e| {
                                    println!("{}", e);
                                }).ok();
                        /*lazy_static! {
                            static ref FUNC: Option<Symbol<'static, FnPtr>> = {
                                //Get func, print and discard error as we don't need it again
                                self.lib.get(stringify!($fn_names).as_bytes()).map_err(|e| {
                                    println!("{}", e);
                                }).ok()
                            };
                        }*/
                        //func.map(|f| f($($fn_arg_names),*))
                        match func {
                            Some(f) => f($($fn_arg_names),*).into(),
                            _ => Default::default()
                            
                        }

                        //func($($fn_arg_names),*)
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
                fn $fn_names(&mut self, $($fn_arg_names: $fn_arg_tys),*) $(-> Option<$fn_ret_tys>)* {
                    unsafe {
                        type FnPtr = unsafe fn($($fn_arg_tys),*) $(-> $fn_ret_tys)*;
                        /*let func = self.funcs.get(stringify!($fn_names));
                        let func :  Option<Symbol<FnPtr>> = match func {
                            Some(x) => {
                                x
                            },
                            None => {
                                self.funcs.insert(stringify!($fn_names), self.lib.get(stringify!($fn_names).as_bytes()).map_err(|e| {
                                    println!("{}", e);
                                }).ok());
                                self.funcs.get(stringify!($fn_names)).unwrap()
                            }
                        };*/

                          
                        let func :  Option<Symbol<FnPtr>>  = self.lib.get(stringify!($fn_names).as_bytes()).map_err(|e| {
                                    println!("{}", e);
                                }).ok();
                        /*lazy_static! {
                            static ref FUNC: Option<Symbol<'static, FnPtr>> = {
                                //Get func, print and discard error as we don't need it again
                                self.lib.get(stringify!($fn_names).as_bytes()).map_err(|e| {
                                    println!("{}", e);
                                }).ok()
                            };
                        }*/
                        //func.map(|f| f($($fn_arg_names),*))
                        match func {
                            Some(f) => f($($fn_arg_names),*).into(),
                            _ => Default::default()
                            
                        }

                        //func($($fn_arg_names),*)
                    }
                }
            //}
        )*
    };
}

pub trait Plugin: Any + Send + Sync {
    /// Get a name describing the `Plugin`.
    fn name(&mut self) -> Option<&'static str>;
    /// A callback fired immediately after the plugin is loaded. Usually used 
    /// for initialization.
    fn initialise(&mut self) -> bool;
    
    fn is_initialised(&mut self) -> bool;

    fn device_info(&mut self, buffer: &mut [DeviceInfoPointer]) -> c_int;
    /// A callback fired immediately before the plugin is unloaded. Use this if
    /// you need to do any cleanup.
    fn unload(&mut self) {}

    fn add(&mut self, x: u32, y: u32) -> Option<u32>;
    fn read_analog(&mut self, code: u16) -> Option<f32>;
    fn read_full_buffer(&mut self, max_length: usize, device: DeviceID) -> Option<Vec<(c_ushort, c_float)>>;
    

    //fn neg(&mut self, x: u32, y: u32) -> Option<u32>;
}

struct CPlugin {
    lib: Library,
    //funcs: HashMap<&'static str, Option<Symbol>>
}

impl CPlugin {
    fn new(lib: Library) -> CPlugin {
        CPlugin {
            lib: lib,
            //funcs: HashMap::new()
        }
    }

    lib_wrap_option!{
        //c_name has to be over here due to it not being part of the Plugin trait
        fn c_name() -> FfiStr<'static>;

    }
    lib_wrap!{
        fn c_read_full_buffer(code_buffer: *const c_ushort, analog_buffer: *const c_float, len: c_uint, device: DeviceID) -> c_int;
        fn c_device_info(buffer: *mut DeviceInfoPointer, len: c_uint) -> c_int;
    }
}

impl Plugin for CPlugin {
    fn name(&mut self) -> Option<&'static str> {
        /*let s = self.c_name();
        let c_str = unsafe {
            assert!(!s.is_null());

            CStr::from_ptr(s)
        };

        c_str.to_str().unwrap()*/
        self.c_name().map(|s| s.as_str())
    }

    fn read_full_buffer(&mut self, max_length: usize, device: DeviceID) -> Option<Vec<(c_ushort, c_float)>> {
        let code_buffer: Vec<c_ushort> = Vec::with_capacity(max_length.into());
        let analog_buffer: Vec<c_float> = Vec::with_capacity(max_length.into());

        let count: usize = {
            let mut ret = self.c_read_full_buffer(code_buffer.as_ptr(), analog_buffer.as_ptr(), max_length as c_uint, device);
            if ret < 0{
                return None;
            }
            else if ret > max_length as i32{
                println!("What you playing at, this return too big");
                ret = max_length as i32;
            }
            ret as usize
        };

        let mut analog_data : Vec<(c_ushort, c_float)> = Vec::with_capacity(count);
        for i in 0..count {
            analog_data.push( (code_buffer[i], analog_buffer[i]) );
        }

        Some(analog_data)
    }

    fn device_info(&mut self, buffer: &mut [DeviceInfoPointer]) -> c_int {
        self.c_device_info(buffer.as_mut_ptr(), buffer.len() as c_uint)
    }

    lib_wrap!{
        fn initialise() -> bool;
        fn is_initialised() -> bool;
        fn unload();
    }
    lib_wrap_option! {
        fn add(x: u32, y: u32) -> u32;
        fn read_analog(code: u16) -> f32;
        //fn neg(x: u32, y: u32) -> u32;
    }
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
        pub extern "C" fn _plugin_create() -> *mut $crate::sdk::Plugin {
            // make sure the constructor is the correct type.
            let constructor: fn() -> $plugin_type = $constructor;

            let object = constructor();
            let boxed: Box<$crate::sdk::Plugin> = Box::new(object);
            Box::into_raw(boxed)
        }
    };
}

#[repr(C)]
pub struct DeviceInfo {
    pub name: *const u8,
    pub val: u32
}

pub type DeviceInfoPointer = *mut DeviceInfo;
pub type DeviceID = u32;


unsafe impl Send for AnalogSDK{

}

pub struct AnalogSDK {
    pub initialised: bool,
    pub disconnected_callback: Option<extern fn(FfiStr)>,
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

const ENV_PLUGIN_DIR_KEY: &str = "ANALOG_SDK_PATH";
const DEFAULT_PLUGIN_DIR: &str = "~/.analog_plugins";



impl AnalogSDK {
    pub fn new() -> AnalogSDK {
        AnalogSDK {
            plugins: Vec::new(),
            loaded_libraries: Vec::new(),
            initialised: false,
            disconnected_callback: None,
            keycode_mode: KeycodeType::HID
            //device_info: Box::into_raw(Box::new(DeviceInfo { name: b"Device Yeet\0" as *const u8, val:20 }))
        }
    }

    pub fn initialise(&mut self) -> bool {
        if self.initialised {
            self.unload();
        }

        let plugin_dir = std::env::var(ENV_PLUGIN_DIR_KEY).map(|path| PathBuf::from(path));
        let plugin_dir = match plugin_dir {
            Ok(v) => {
                println!("Found ${}, loading plugins from {:?}", ENV_PLUGIN_DIR_KEY, v);
                v
            },
            Err(e) => {
                println!("{} is not set, defaulting to {}.\nError: {}", ENV_PLUGIN_DIR_KEY, DEFAULT_PLUGIN_DIR, e);
                PathBuf::from(String::from(DEFAULT_PLUGIN_DIR))
            } 
        };
        match self.load_plugins(&plugin_dir) {
            Ok(0) => { 
                println!("Failed to load any plugins!");
                self.initialised = false;
            },
            Ok(i) => {
                println!("Loaded {} plugins", i);

                let mut x = 0;
                for p in self.plugins.iter_mut() {
                    if p.initialise() {
                        x = x + 1;
                    }
                }
                println!("{} plugins successfully initialised", x);

                self.initialised = x > 0;
            },
            Err(e) => {
                println!("Error: {}", e);
                self.initialised = false;
            }
        }



        return self.initialised;
    }

    fn load_plugins(&mut self, dir: &Path) -> Result<u32> {
        if dir.is_dir() {
            let mut i: u32 = 0;
            for entry in fs::read_dir(dir).chain_err(|| format!("Unable to load dir \"{}\"", dir.display()))? {
                let path = entry.chain_err(|| "Err with entry")?.path();
                
                if let Some(ext) = path.extension().and_then(OsStr::to_str) {
                    if ext == LIB_EXT {
                        println!("Attempting to load plugin: \"{}\"", path.display());
                        unsafe {
                            self.load_plugin(&path)?;
                        }
                        i = i+1;
                    }
                }
            }
            return Ok(i);
        }
        Err("Path is not a dir!".into())
        //Err(io::Error::new(io::ErrorKind::Other, format!("Path \"{}\" is not a directory", dir.display())))
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

        let constructor: Option<Symbol<PluginCreate>> = lib.get(b"_plugin_create").map_err(|e| {
            println!("{}", e);
        }).ok();
        //    .chain_err(|| "The `_plugin_create` symbol wasn't found.");

        let mut plugin = match constructor {
            Some(f) => {
                println!("We got it and we're trying");
                Box::from_raw(f())
            },
            None => {
                println!("Didn't find _plugin_create, assuming it's a c plugin");
                let lib = self.loaded_libraries.pop().unwrap();
                Box::new(CPlugin::new(lib))
            }
        };

        println!("Loaded plugin: {:?}", plugin.name());
        //plugin.on_plugin_load();
        self.plugins.push(plugin);


        Ok(())
    }

    pub fn add(&mut self, x: u32, y: u32) -> Vec<u32> {
        if self.plugins.len() <= 0 {
            return Vec::new();
        }
        let mut results: Vec<u32> = Vec::new();
        for p in self.plugins.iter_mut() {
            if let Some(x) = p.add(x, y) {
                results.push(x);
            }
        }
        //testing disconnected cb
        if let Some(cb) = self.disconnected_callback {
            cb(FfiStr::from_cstr(CString::new("Yeet").unwrap().as_c_str()));
        }


        results
    }

    pub fn read_analog(&mut self, code: u16) -> f32 {
        if self.plugins.len() <= 0 {
            return -1.0;
        }
        let hid_code = code_to_hid(code, &self.keycode_mode);
        if let Some(hid_code) = hid_code {
            let mut value: f32 = 0.0;
            for p in self.plugins.iter_mut() {
                if let Some(x) = p.read_analog(hid_code) {
                    value = value.max(x);
                }
            }
            return value;
        }
        else{
            return -1.0;
        }
    }

    pub fn get_device_info(&mut self, buffer: &mut [DeviceInfoPointer]) -> c_int {
        if self.plugins.len() <= 0 {
            return -1;
        }
        let mut count: usize = 0;
        for p in self.plugins.iter_mut() {
            let num = p.device_info(&mut buffer[count..]) as usize;
            if num > 0 {
                count = count + num
            }
        }
        count as c_int
    }

    pub fn read_full_buffer(&mut self, code_buffer: &mut [c_ushort], analog_buffer: &mut [c_float], device: DeviceID) -> c_int {
        if self.plugins.len() <= 0 {
            return -1;
        }

        let mut analog_data: HashMap<c_ushort, c_float> = HashMap::with_capacity(code_buffer.len());

        //Read from all and add up
        for p in self.plugins.iter_mut() {
            let plugin_data = p.read_full_buffer(code_buffer.len(), 0);
            if let Some(mut data) = plugin_data {

                for (hid_code, analog) in data.drain(0..){
                    if analog == 0.0 {
                        continue;
                    }
                    let code = hid_to_code(hid_code, &self.keycode_mode);
                    if let Some(code) = code {
                        let mut total_analog = analog;

                        //No point in checking if the value is already present if we are only looking for data from one device
                        if device == 0 {
                            if let Some(val) = analog_data.get(&code) {
                                total_analog = total_analog.max(*val);
                            }
                        }
                        analog_data.insert(code, total_analog);
                    }
                    else{
                        println!("Couldn't map HID:{} to {:?}", hid_code, self.keycode_mode);
                    }
                }

                //If we are looking for a specific device, just break out when we find one that returns good
                if device != 0 {
                    break;
                }
            }
        }

        //Fill up given slices
        let mut count: usize = 0;
        for (code, analog) in analog_data.drain() {
            if count >= code_buffer.len() {
                break;
            }

            code_buffer[count] = code;
            analog_buffer[count] = analog;
            count = count + 1;
        }
        count as c_int
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