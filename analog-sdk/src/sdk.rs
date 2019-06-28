use std::any::Any;
use libloading::{Library, Symbol};
use std::fs;
use std::path::{Path, PathBuf};
use crate::errors::*;
//use libc::c_char;
use std::ffi::{CString, OsStr};
use std::os::raw::c_char;
use ffi_support::{FfiStr};
//use std::collections::HashMap;
//use std::ops::Deref;
use scancode::Scancode;

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
    /// A callback fired immediately before the plugin is unloaded. Use this if
    /// you need to do any cleanup.
    fn unload(&mut self) {}

    fn add(&mut self, x: u32, y: u32) -> Option<u32>;
    fn read_analog_hid(&mut self, code: u8) -> Option<f32>;
    

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
    lib_wrap!{
        fn initialise() -> bool;
        fn is_initialised() -> bool;
        fn unload();
    }
    lib_wrap_option! {
        
        fn add(x: u32, y: u32) -> u32;
        fn read_analog_hid(code: u8) -> f32;
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

pub struct AnalogSDK {
    pub initialised: bool,
    pub disconnected_callback: Option<extern fn(*const c_char)>,

    plugins: Vec<Box<Plugin>>,
    loaded_libraries: Vec<Library>,
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
            disconnected_callback: None
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
            cb(CString::new("Yeet").unwrap().as_ptr());
        }


        results
    }

    pub fn read_analog_hid(&mut self, code: u8) -> f32 {
        if self.plugins.len() <= 0 {
            return -1.0;
        }

        let mut value: f32 = 0.0;
        for p in self.plugins.iter_mut() {
            if let Some(x) = p.read_analog_hid(code) {
                value = value.max(x);
            }
        }
        value
    }

    pub fn read_analog_vk(&mut self, code: u8, translate: bool) -> f32 {
        #[cfg(windows)]
        unsafe {

            use winapi::um::winuser::{GetForegroundWindow, GetWindowThreadProcessId, GetKeyboardLayout, MapVirtualKeyExA, MapVirtualKeyA};
            let scancode: u32;
            if translate {
                let window_handle = GetForegroundWindow();
                let thread = GetWindowThreadProcessId(window_handle, 0 as *mut u32);
                let layout = GetKeyboardLayout(thread);
                scancode = MapVirtualKeyExA(code.into(), 0, layout);
                //println!("Window handle: {:?}, thread: {:?}, layout: {:?}, code: {} scancode: {}", window_handle, thread, layout, code, scancode);
            }
            else{
                scancode = MapVirtualKeyA(code.into(), 0);
            }

            self.read_analog_sc(scancode as u8)
        }

        #[cfg(not(windows))]
        -1.0
    }

    pub fn read_analog_sc(&mut self, code: u8) -> f32 {
        match Scancode::new(code) {
            Some(hid) => {
                self.read_analog_hid(hid as u8)
            },
            None => {
                -1.0
            }
        }

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