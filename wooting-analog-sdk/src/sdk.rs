use crate::DeviceEventType;
use crate::DeviceID;
use crate::DeviceInfo;
use crate::KeycodeType;
use crate::Plugin;
use crate::SDKResult;
use crate::WootingAnalogResult;
use crate::keycode::*;
use crate::plugin::ANALOG_SDK_PLUGIN_VERSION;
use crate::plugin::DEFAULT_PLUGIN_DIR;
use crate::plugin::WootingPlugin;
use crate::plugin::c::CPlugin;
use anyhow::bail;
use anyhow::{Context, Error, Result};
use libloading::{Library, Symbol};
use log::debug;
use log::trace;
use log::{error, info, warn};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::{fs, thread};

unsafe impl Send for AnalogSDK {}

pub struct AnalogSDK {
    pub initialised: bool,
    pub keycode_mode: KeycodeType,

    plugins: Vec<Box<dyn Plugin>>,
    loaded_libraries: Vec<Library>,
    device_event_callback: Arc<Mutex<Option<Box<dyn Fn(DeviceEventType, DeviceInfo) + Send>>>>,
}

pub fn print_error(err: Error) -> Error {
    error!("{:#}", err);
    err
}

pub fn print_warn(err: Error) -> Error {
    warn!("{:#}", err);
    err
}

#[cfg(target_os = "macos")]
static LIB_EXT: &str = "dylib";
#[cfg(target_os = "linux")]
static LIB_EXT: &str = "so";
#[cfg(target_os = "windows")]
static LIB_EXT: &str = "dll";

impl AnalogSDK {
    pub fn new() -> AnalogSDK {
        AnalogSDK {
            plugins: Vec::new(),
            loaded_libraries: Vec::new(),
            initialised: false,
            keycode_mode: KeycodeType::HID,
            device_event_callback: Arc::new(Mutex::new(None)),
        }
    }

    pub fn initialise(&mut self) -> SDKResult<u32> {
        let dir = option_env!("WOOTING_ANALOG_SDK_PLUGINS_PATH").unwrap_or(DEFAULT_PLUGIN_DIR);
        self.initialise_with_plugin_path(dir, true)
    }

    pub fn initialise_with_plugin_path(
        &mut self,
        plugin_dir: &str,
        nested: bool,
    ) -> SDKResult<u32> {
        if self.initialised {
            self.unload();
        }

        let plugin_dir = PathBuf::from(plugin_dir);
        if plugin_dir.is_dir() {
            /*let mut plugin_dir = match plugin_dir {
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
            };*/
            let mut load_plugins = |dir: &Path| {
                match self.load_plugins(dir) {
                    Ok(0) => {
                        info!("No plugins found in {:?}", dir);
                        //self.initialised = false;
                        //WootingAnalogResult::NoPlugins
                    }
                    Ok(i) => {
                        debug!("Loaded {} plugins from {:?}", i, dir);
                        //WootingAnalogResult::Ok
                    }
                    Err(e) => {
                        error!("Error: {:?}", e);
                        //self.initialised = false;
                    }
                }
            };

            load_plugins(plugin_dir.as_path());

            if nested {
                for dir in plugin_dir.read_dir().expect("Could not read dir") {
                    match dir {
                        Ok(dir) => {
                            if dir.path().is_file() {
                                continue;
                            }

                            load_plugins(&dir.path());
                        }
                        Err(e) => {
                            error!("Error reading directory: {}", e);
                        }
                    }
                }
            }
        }

        self.plugins.push(Box::new(WootingPlugin::new()));

        let mut plugins_initialised = 0;
        let mut device_no: u32 = 0;
        for p in self.plugins.iter_mut() {
            let arc_cb = self.device_event_callback.clone();
            let ret = p.initialise(Box::new(
                move |event: DeviceEventType, device_ref: &DeviceInfo| {
                    let opt_cb = arc_cb.clone();
                    let device = device_ref.clone();
                    thread::spawn(move || {
                        debug!("device event cb thread running");

                        opt_cb.lock().unwrap().as_ref().and_then(|cb| {
                            debug!("calling og callback");
                            cb(event, device);
                            Some(0)
                        });
                    });
                },
            ));
            debug!("{:?}", ret);
            if let Ok(num) = ret.0 {
                plugins_initialised += 1;
                device_no += num;
            }
        }

        info!("{} plugins successfully initialised", plugins_initialised);

        self.initialised = plugins_initialised > 0;
        if !self.initialised {
            Err(WootingAnalogResult::NoPlugins).into()
        } else {
            Ok(device_no).into()
        }
    }

    fn load_plugins(&mut self, dir: &Path) -> Result<u32> {
        if dir.is_dir() {
            let mut i: u32 = 0;
            for entry in fs::read_dir(dir)
                .with_context(|| format!("Unable to load dir \"{}\"", dir.display()))?
            {
                let path = entry.context("Err with entry")?.path();

                if let Some(ext) = path.extension().and_then(OsStr::to_str) {
                    if ext == LIB_EXT {
                        info!("Loading plugin: \"{}\"", path.display());
                        unsafe {
                            if self
                                .load_plugin(&path)
                                .context("Load Plugin failed")
                                .map_err(print_error)
                                .is_ok()
                            {
                                i += 1;
                            }
                        }
                    }
                }
            }
            return Ok(i);
        }

        bail!("Path: {:?} is not a dir!", dir)
    }

    unsafe fn load_plugin(&mut self, filename: &Path) -> Result<()> {
        if filename.is_dir() {
            bail!("Path is directory!");
        }

        type PluginCreate = unsafe extern "C" fn() -> *mut dyn Plugin;
        type PluginVersion = unsafe extern "C" fn() -> &'static str;

        let lib = Library::new(filename.as_os_str()).context("Unable to load the plugin")?;

        // We need to keep the library around otherwise our plugin's vtable will
        // point to garbage. We do this little dance to make sure the library
        // doesn't end up getting moved.
        self.loaded_libraries.push(lib);

        let lib = self.loaded_libraries.last().unwrap();

        let full_version: Option<Symbol<PluginVersion>> = lib.get(b"plugin_version").ok();
        let mut got_ver = false;
        if let Some(f_ver) = full_version {
            got_ver = true;
            let ver = f_ver();
            debug!(
                "Plugin got plugin-dev sem version: {}. SDK: {}",
                ver, ANALOG_SDK_PLUGIN_VERSION
            );

            if let Some(major_ver) = ANALOG_SDK_PLUGIN_VERSION
                .split('.')
                .collect::<Vec<&str>>()
                .first()
            {
                if let Some(plugin_major_ver) = ver.split('.').collect::<Vec<&str>>().first() {
                    if major_ver.eq(plugin_major_ver) {
                        info!("Plugin and SDK are compatible!");
                    } else {
                        bail!(
                            "Plugin has major version {}, which is incompatible with the SDK's: {}",
                            plugin_major_ver,
                            ANALOG_SDK_PLUGIN_VERSION
                        );
                    }
                } else {
                    bail!(
                        "Unable to get the Plugin's major version from SemVer {}",
                        ver
                    );
                }
            } else {
                bail!(
                    "Unable to get the SDK's Plugin major version from SemVer {}",
                    ANALOG_SDK_PLUGIN_VERSION
                );
            }
        } else {
            warn!("Unable to determine the Plugin's SemVer!");
        }

        let constructor: Option<Symbol<PluginCreate>> = lib
            .get(b"_plugin_create")
            .context("Failed to find constructor (_plugin_create symbol)")
            .map_err(print_warn)
            .ok();

        let mut plugin = match constructor {
            Some(f) => {
                if !got_ver {
                    bail!("Unable to determine the Plugin's SemVer!");
                }

                debug!("We got it and we're trying");
                Box::from_raw(f())
            }
            None => {
                info!("Didn't find _plugin_create, assuming it's a C plugin");
                let lib = self.loaded_libraries.pop().unwrap();
                match CPlugin::new(lib).0 {
                    Ok(cplugin) => Box::new(cplugin),
                    Err(WootingAnalogResult::IncompatibleVersion) => {
                        bail!(
                            "Plugin is a C plugin which is incompatible with this version of the SDK"
                        );
                    }
                    Err(_) => {
                        bail!("Plugin isn't a valid C or Rust plugin");
                    }
                }
            }
        };
        let name = plugin.name();
        match name.0 {
            Ok(name) => {
                info!("Loaded plugin: {:?}", name);
                //plugin.on_plugin_load();
                self.plugins.push(plugin);
            }
            Err(WootingAnalogResult::FunctionNotFound) => {
                bail!("Plugin isn't a valid plugin, name function not found");
            }
            Err(e) => {
                bail!("Plugin failed with unhandled error {:?}", e);
            }
        }

        Ok(())
    }

    pub fn set_device_event_cb(
        &mut self,
        cb: impl Fn(DeviceEventType, DeviceInfo) + 'static + Send,
    ) -> SDKResult<()> {
        if !self.initialised {
            return WootingAnalogResult::UnInitialized.into();
        }
        self.device_event_callback
            .lock()
            .unwrap()
            .replace(Box::new(cb));

        Ok(()).into()
    }

    pub fn clear_device_event_cb(&mut self) -> SDKResult<()> {
        if !self.initialised {
            return Err(WootingAnalogResult::UnInitialized).into();
        }
        self.device_event_callback.lock().unwrap().take();

        Ok(()).into()
    }

    pub fn get_device_info(&mut self) -> SDKResult<Vec<DeviceInfo>> {
        if !self.initialised {
            return Err(WootingAnalogResult::UnInitialized).into();
        }
        let mut devices: Vec<DeviceInfo> = vec![];
        let mut error: WootingAnalogResult = WootingAnalogResult::Ok;
        for p in self.plugins.iter_mut() {
            if !p.is_initialised() {
                continue;
            }

            //Give a reference to the buffer at the point where there is free space
            match p.device_info().0 {
                Ok(mut p_devices) => {
                    devices.append(&mut p_devices);
                }
                Err(e) => {
                    error!(
                        "Plugin {:?} failed to fetch devices with error {:?}",
                        p.name(),
                        e
                    );
                    error = e;
                }
            }
        }
        if devices.is_empty() && !error.is_ok() {
            Err(error).into()
        } else {
            Ok(devices).into()
        }
    }

    pub fn read_analog(&mut self, code: u16, device_id: DeviceID) -> SDKResult<f32> {
        if !self.initialised {
            return Err(WootingAnalogResult::UnInitialized).into();
        }

        //Try and map the given keycode to HID
        let hid_code = code_to_hid(code, &self.keycode_mode);
        if let Some(hid_code) = hid_code {
            let mut value: f32 = -1.0;
            let mut err = WootingAnalogResult::Ok;

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
                return Err(err).into();
            }

            value.into()
        } else {
            Err(WootingAnalogResult::NoMapping).into()
        }
    }

    pub fn read_full_buffer(
        &mut self,
        max_length: usize,
        device_id: DeviceID,
    ) -> SDKResult<HashMap<u16, f32>> {
        if !self.initialised {
            return Err(WootingAnalogResult::UnInitialized).into();
        }

        let mut analog_data: HashMap<u16, f32> = HashMap::with_capacity(max_length);

        let mut err = WootingAnalogResult::Ok;
        let mut any_success = false;
        //Read from all and add up
        for p in self.plugins.iter_mut() {
            // Check if we've already collected enough data
            if analog_data.len() >= max_length {
                break;
            }

            let remaining = max_length.saturating_sub(analog_data.len());
            let plugin_data = p.read_full_buffer(remaining, device_id).into();
            match plugin_data {
                Ok(mut data) => {
                    for (hid_code, analog) in data.drain() {
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
                            warn!("Couldn't map HID:{} to {:?}", hid_code, self.keycode_mode);
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
            return Err(err).into();
        }

        Ok(analog_data).into()
    }

    /// Unload all plugins and loaded plugin libraries, making sure to fire
    /// their `on_plugin_unload()` methods so they can do any necessary cleanup.
    pub fn unload(&mut self) {
        debug!("Unloading plugins");
        for mut plugin in self.plugins.drain(..) {
            let name = plugin.name().0;
            trace!("Firing on_plugin_unload for {:?}", name);
            plugin.unload();
            debug!("Unload successful for {:?}", name);
        }

        debug!("Attempting to drop loaded libraries");
        self.loaded_libraries.drain(..);
        debug!("Succeeded dropping loaded libraries");

        self.device_event_callback.lock().unwrap().take();
        debug!("Finished Analog SDK Uninit");

        self.initialised = false;
    }
}

impl Drop for AnalogSDK {
    fn drop(&mut self) {
        self.unload();
    }
}

impl Default for AnalogSDK {
    fn default() -> Self {
        Self::new()
    }
}
