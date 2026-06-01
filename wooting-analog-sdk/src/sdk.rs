use crate::AnalogData;
use crate::AnalogValue;
use crate::DeviceEventType;
use crate::DeviceID;
use crate::DeviceInfo;
use crate::KeyCode;
use crate::KeyPosition;
use crate::KeycodeType;
use crate::PhysicalKey;
use crate::Plugin;
use crate::err::PluginError;
use crate::err::ReadError;
use crate::keycode::*;
use crate::plugin::DEFAULT_PLUGIN_DIR;
use crate::plugin::WootingPlugin;
use crate::plugin::c::CPlugin;
use libloading::Library;
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

    pub fn initialise(&mut self) -> Result<u32, ReadError> {
        let dir = option_env!("WOOTING_ANALOG_SDK_PLUGINS_PATH").unwrap_or(DEFAULT_PLUGIN_DIR);
        self.initialise_with_plugin_path(dir, true)
    }

    pub fn initialise_with_plugin_path(
        &mut self,
        plugin_dir: &str,
        nested: bool,
    ) -> Result<u32, ReadError> {
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
            if let Ok(num) = ret {
                plugins_initialised += 1;
                device_no += num;
            }
        }

        info!("{} plugins successfully initialised", plugins_initialised);

        self.initialised = plugins_initialised > 0;
        if !self.initialised {
            Err(ReadError::Plugin(PluginError::ZeroPlugins))
        } else {
            Ok(device_no)
        }
    }

    fn load_plugins(&mut self, dir: &Path) -> Result<u32, PluginError> {
        if dir.is_dir() {
            let mut i: u32 = 0;
            for entry in fs::read_dir(dir)? {
                let path = entry?.path();

                if let Some(ext) = path.extension().and_then(OsStr::to_str) {
                    if ext == LIB_EXT {
                        info!("Loading plugin: \"{}\"", path.display());
                        unsafe {
                            if self
                                .load_plugin(&path)
                                .inspect_err(|e| error!("failed to load plugin: {e}"))
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

        Err(PluginError::InvalidDirectory(dir.to_path_buf()))
    }

    unsafe fn load_plugin(&mut self, path: &Path) -> Result<CPlugin, PluginError> {
        if path.is_dir() {
            return Err(PluginError::InvalidPlugin(path.to_path_buf()));
        }

        let mut plugin = CPlugin::new(
            unsafe { Library::new(path) }
                .map_err(|e| PluginError::DynamicLibraryError { source: e })?,
        )?;

        println!("calling name of plugin");
        plugin
            .name()
            .inspect(|name| println!("Loaded plugin: {:?}", name))?;

        Ok(plugin)
    }

    pub fn set_device_event_cb(
        &mut self,
        cb: impl Fn(DeviceEventType, DeviceInfo) + 'static + Send,
    ) -> Result<(), ReadError> {
        if !self.initialised {
            return Err(ReadError::Uninitialized);
        }
        self.device_event_callback
            .lock()
            .unwrap()
            .replace(Box::new(cb));

        Ok(())
    }

    pub fn clear_device_event_cb(&mut self) -> Result<(), ReadError> {
        if !self.initialised {
            return Err(ReadError::Uninitialized);
        }
        self.device_event_callback.lock().unwrap().take();

        Ok(())
    }

    pub fn get_device_info(&mut self) -> Result<Vec<DeviceInfo>, ReadError> {
        if !self.initialised {
            return Err(ReadError::Uninitialized);
        }
        let mut devices: Vec<DeviceInfo> = vec![];
        let mut error = None;
        for p in self.plugins.iter_mut() {
            if !p.is_initialised() {
                continue;
            }

            //Give a reference to the buffer at the point where there is free space
            match p.device_info() {
                Ok(mut p_devices) => {
                    devices.append(&mut p_devices);
                }
                Err(e) => {
                    error!(
                        "Plugin {:?} failed to fetch devices with error {:?}",
                        p.name(),
                        e
                    );
                    error = Some(e);
                }
            }
        }

        if let Some(err) = error
            && devices.is_empty()
        {
            return Err(err);
        }

        Ok(devices)
    }

    pub fn read_analog(&mut self, code: u16, device_id: DeviceID) -> Result<f32, ReadError> {
        if !self.initialised {
            return Err(ReadError::Uninitialized);
        }

        //Try and map the given keycode to HID
        let hid_code = code_to_hid(code, &self.keycode_mode);
        if let Some(hid_code) = hid_code {
            let mut value: f32 = -1.0;
            let mut err = None;

            for p in self.plugins.iter_mut() {
                match p.read_analog(hid_code, device_id) {
                    Ok(x) => {
                        value = value.max(x);
                        //If we were looking to read from a specific device, we've found that read, so no need to continue
                        if device_id != 0 {
                            break;
                        }
                    }
                    Err(e) => {
                        //TODO: Improve collating of multiple errors
                        err = Some(e)
                    }
                }
            }

            if let Some(err) = err
                && value < 0.0
            {
                return Err(err);
            }

            Ok(value)
        } else {
            Err(ReadError::NoMapping {
                keycode: code,
                mode: self.keycode_mode.clone(),
            })
        }
    }

    pub fn read_full_buffer(
        &mut self,
        max_length: usize,
        device_id: DeviceID,
    ) -> Result<HashMap<u16, f32>, ReadError> {
        if !self.initialised {
            return Err(ReadError::Uninitialized);
        }

        let mut analog_data: HashMap<u16, f32> = HashMap::with_capacity(max_length);

        let mut err = None;
        let mut any_success = false;
        //Read from all and add up
        for p in self.plugins.iter_mut() {
            // Check if we've already collected enough data
            if analog_data.len() >= max_length {
                break;
            }

            let remaining = max_length.saturating_sub(analog_data.len());
            let plugin_data = p.read_full_buffer(remaining, device_id);
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
                    err = Some(e)
                }
            }
            //If we are looking for a specific device, just break out when we find one that returns good
            if device_id != 0 {
                break;
            }
        }

        if let Some(err) = err
            && !any_success
        {
            return Err(err);
        }

        Ok(analog_data)
    }

    pub(crate) fn read_keycode(&mut self, code: u16, device_id: DeviceID) -> SDKResult<AnalogValue> {
        if !self.initialised {
            return Err(ReadError::Uninitialized);
        }

        let Some(hid_code) = crate::keycode::code_to_hid(code, &self.keycode_mode) else {
            return Err(WootingAnalogResult::NoMapping).into();
        };

        let mut value = AnalogValue::from(-1.0);
        let mut err = WootingAnalogResult::Ok;

        for p in self.plugins.iter_mut() {
            match p.read_keycode(KeyCode::from(hid_code), device_id).into() {
                Ok(x) => {
                    value = value.max(x);
                    if device_id != 0 {
                        break;
                    }
                }
                Err(e) => err = e,
            }
        }

        if value < 0.0 {
            return Err(err).into();
        }

        SDKResult(Ok(value))
    }

    pub(crate) fn read_position(
        &mut self,
        position: KeyPosition,
        device_id: DeviceID,
    ) -> SDKResult<PhysicalKey> {
        if !self.initialised {
            return Err(WootingAnalogResult::UnInitialized).into();
        }

        let mut result = PhysicalKey::new(position);
        let mut err = WootingAnalogResult::Ok;

        for p in self.plugins.iter_mut() {
            match p.read_position(position, device_id).into() {
                Ok(pk) => {
                    for i in 0..pk.state_count {
                        result.push_state(pk.states[i as usize]);
                    }
                    if device_id != 0 {
                        break;
                    }
                }
                Err(e) => err = e,
            }
        }

        if result.state_count == 0 {
            return Err(err).into();
        }

        SDKResult(Ok(result))
    }

    pub(crate) fn inputs(&mut self, device_id: DeviceID) -> SDKResult<AnalogData> {
        if !self.initialised {
            return Err(WootingAnalogResult::UnInitialized).into();
        }

        let mut combined = AnalogData::new();
        let mut err = WootingAnalogResult::Ok;
        let mut any_success = false;

        for p in self.plugins.iter_mut() {
            match p.read_full_buffer_with_ctx(device_id).into() {
                Ok(plugin_data) => {
                    combined.merge(plugin_data);
                    any_success = true;
                }
                Err(e) => {
                    err = e;
                }
            }

            // If looking for a specific device, break after first successful read
            if device_id != 0 && any_success {
                break;
            }
        }

        if !any_success {
            return Err(err);
        }

        Ok(combined).into()
    }

    // TODO: hide hashmap impl detail behind opaque struct
    // will probably be -> InputsPosition { .. }
    // could even try Inputs<Position> ?
    pub(crate) fn read_positions(
        &mut self,
        device_id: DeviceID,
    ) -> SDKResult<HashMap<KeyPosition, PhysicalKey>> {
        self.inputs(device_id)
            .0
            .map(|data| data.position_based)
            .into()
    }

    // TODO: hide hashmap impl detail behind opaque struct
    // will probably be -> InputsKeyCode { .. }
    // could even try Inputs<KeyCode> ?
    pub(crate) fn read_keycodes(
        &mut self,
        device_id: DeviceID,
    ) -> SDKResult<HashMap<KeyCode, AnalogValue>> {
        self.inputs(device_id)
            .0
            .map(|data| data.keycode_based)
            .into()
    }

    /// Unload all plugins and loaded plugin libraries, making sure to fire
    /// their `on_plugin_unload()` methods so they can do any necessary cleanup.
    pub fn unload(&mut self) {
        debug!("Unloading plugins");
        for mut plugin in self.plugins.drain(..) {
            let name = plugin.name();
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
