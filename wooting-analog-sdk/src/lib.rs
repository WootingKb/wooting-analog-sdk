pub mod analog_value;
pub mod ctx;
pub mod device;
pub mod err;
#[cfg(feature = "ffi")]
mod ffi;
mod key;
pub mod keycode;
mod plugin;
#[cfg(feature = "virtual-input")]
mod virtual_input;

#[doc(inline)]
pub use crate::{analog_value::AnalogValue, keycode::KeyCode};
pub use crate::{
    key::{KeyPosition, PhysicalKey},
    plugin::Plugin,
};

use crate::{
    analog_value::ValueMetadata,
    ctx::{Context, KeyCodeFilter, PositionFilter},
    device::DeviceID,
    device::{DeviceEventType, DeviceInfo},
    err::{PluginError, ReadError},
    keycode::KeycodeType,
    plugin::{DEFAULT_PLUGIN_DIR, dynamic::DynamicPlugin, wooting::WootingPlugin},
};
use libloading::Library;
use log::{debug, error, info, trace};
use std::{
    collections::HashMap,
    env::consts::DLL_EXTENSION,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
    thread,
};

pub struct Initialised {
    device_count: u32,
    plugins: Mutex<Vec<Box<dyn Plugin>>>,
}

impl Initialised {
    pub(crate) fn lock_plugins(&self) -> MutexGuard<'_, Vec<Box<dyn Plugin + 'static>>> {
        self.plugins.lock().expect("mutex should not be poisoned")
    }
}

impl Drop for Initialised {
    fn drop(&mut self) {
        debug!("Unloading plugins");
        for mut plugin in self.lock_plugins().drain(..) {
            let name = plugin.name();
            trace!("Firing on_plugin_unload for {:?}", name);
            plugin.unload();
            debug!("Unload successful for {:?}", name);
        }

        debug!("Finished Analog SDK Uninit");
    }
}

pub struct Uninitialised {
    nested: bool,
    plugin_dir: Option<PathBuf>,
    include_wooting_plugin: bool,
    device_events: Option<Arc<dyn Fn(DeviceEventType, DeviceInfo) + Send + Sync>>,
}

impl Default for Uninitialised {
    fn default() -> Self {
        Self {
            nested: true,
            plugin_dir: None,
            include_wooting_plugin: true,
            device_events: None,
        }
    }
}

impl Uninitialised {
    fn load_plugins_from_dir(&self) -> Result<Vec<Box<dyn Plugin>>, PluginError> {
        let plugin_dir = self.plugin_dir.clone().unwrap_or_else(|| {
            PathBuf::from(
                option_env!("WOOTING_ANALOG_SDK_PLUGINS_PATH").unwrap_or(DEFAULT_PLUGIN_DIR),
            )
        });

        if !plugin_dir.is_dir() {
            return Err(PluginError::InvalidDirectory(plugin_dir.to_path_buf()));
        }

        let mut plugins = Vec::new();

        let mut on_load_plugins = |dir: &Path| match load_plugins(dir) {
            Ok(loaded_plugins) => {
                if loaded_plugins.is_empty() {
                    info!("No plugins found in {:?}", dir);
                    return;
                } else {
                    debug!("Loaded {} plugins from {:?}", loaded_plugins.len(), dir);
                }

                plugins.extend(
                    loaded_plugins
                        .into_iter()
                        .map(|p| -> Box<dyn Plugin> { Box::new(p) }),
                );
            }
            Err(e) => {
                error!("Error: {:?}", e);
            }
        };

        on_load_plugins(&plugin_dir);

        if self.nested {
            for dir in plugin_dir.read_dir()? {
                match dir {
                    Ok(dir) => {
                        if dir.path().is_file() {
                            continue;
                        }

                        on_load_plugins(&dir.path());
                    }
                    Err(e) => {
                        error!("Error reading directory: {}", e);
                    }
                }
            }
        }

        if plugins.is_empty() {
            return Err(PluginError::ZeroPlugins);
        }

        Ok(plugins)
    }
}

#[derive(Default)]
pub struct AnalogSdk<S = Uninitialised> {
    pub keycode_mode: KeycodeType,
    device_events: Option<Arc<dyn Fn(DeviceEventType, DeviceInfo) + Send + Sync>>,
    keycodes: Mutex<HashMap<KeyCode, AnalogValue>>,
    positions: Mutex<HashMap<KeyPosition, PhysicalKey>>,
    state: S,
}

impl AnalogSdk<Uninitialised> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_plugin_directory<P: AsRef<Path>>(self, path: P, nested: bool) -> Self {
        let path = path.as_ref();

        Self {
            state: Uninitialised {
                nested,
                plugin_dir: Some(path.to_path_buf()),
                ..self.state
            },
            ..self
        }
    }

    pub fn without_wooting_plugin(self) -> Self {
        Self {
            state: Uninitialised {
                include_wooting_plugin: false,
                ..self.state
            },
            ..self
        }
    }

    pub fn with_keycode_mode(mut self, keycode_type: KeycodeType) -> Self {
        self.keycode_mode = keycode_type;
        self
    }

    pub fn with_device_events<F>(self, f: F) -> Self
    where
        F: Fn(DeviceEventType, DeviceInfo) + 'static + Send + Sync,
    {
        Self {
            state: Uninitialised {
                device_events: Some(Arc::new(f)),
                ..self.state
            },
            ..self
        }
    }

    pub fn initialise(self) -> Result<AnalogSdk<Initialised>, PluginError> {
        let mut plugins = self.state.load_plugins_from_dir()?;

        if self.state.include_wooting_plugin {
            plugins.push(Box::new(WootingPlugin::new()));
        }

        let mut plugins_initialised = 0;
        let mut device_count: u32 = 0;
        for p in plugins.iter_mut() {
            let arc_cb = self.state.device_events.clone();
            let ret = p.initialise(Box::new(
                move |event: DeviceEventType, device_ref: &DeviceInfo| {
                    let opt_cb = arc_cb.clone();
                    let device = device_ref.clone();
                    thread::spawn(move || {
                        debug!("device event cb thread running");

                        if let Some(cb) = opt_cb {
                            cb(event, device)
                        }
                    });
                },
            ));
            debug!("{:?}", ret);
            if let Ok(num) = ret {
                plugins_initialised += 1;
                device_count += num;
            }
        }

        info!("{} plugins successfully initialised", plugins_initialised);

        Ok(AnalogSdk {
            keycode_mode: self.keycode_mode,
            device_events: self.device_events,
            keycodes: self.keycodes,
            positions: self.positions,
            state: Initialised {
                device_count,
                plugins: Mutex::new(plugins),
            },
        })
    }
}

impl AnalogSdk<Initialised> {
    pub fn read_keycode<T>(&self, code: T) -> Result<AnalogValue, ReadError>
    where
        T: Into<u16>,
    {
        self.read_keycode_for(0, code)
    }

    pub fn read_keycode_for<T>(
        &self,
        device_id: DeviceID,
        code: T,
    ) -> Result<AnalogValue, ReadError>
    where
        T: Into<u16>,
    {
        let code = code.into();

        fn inner_read_keycode(
            sdk: &AnalogSdk<Initialised>,
            device_id: DeviceID,
            code: u16,
        ) -> Result<AnalogValue, ReadError> {
            let Some(hid_code) = crate::keycode::code_to_hid(code, &sdk.keycode_mode) else {
                return Err(ReadError::NoMapping {
                    keycode: code,
                    mode: sdk.keycode_mode.clone(),
                });
            };

            let mut value = AnalogValue::from(-1.0);
            let mut error = None;

            for p in sdk.state.lock_plugins().iter_mut() {
                match p.read_keycode(KeyCode::from(hid_code), device_id) {
                    Ok(x) => {
                        value = value.max(x);
                        if device_id != 0 {
                            break;
                        }
                    }
                    Err(e) => error = Some(e),
                }
            }

            if let Some(err) = error
                && value < 0.0
            {
                return Err(err);
            }

            Ok(value)
        }

        inner_read_keycode(self, device_id, code)
    }

    pub fn read_position(&self, position: KeyPosition) -> Result<PhysicalKey, ReadError> {
        self.read_position_for(0, position)
    }

    pub fn read_position_for(
        &self,
        device_id: DeviceID,
        position: KeyPosition,
    ) -> Result<PhysicalKey, ReadError> {
        let mut physical_key = PhysicalKey::new(position);
        let mut error = None;
        let mut any_success = false;

        for p in self.state.lock_plugins().iter_mut() {
            match p.read_position(position, device_id) {
                Ok(pk) => {
                    for i in 0..pk.active_key_count {
                        physical_key.push_state(pk.states[i as usize]);
                    }

                    any_success = true;

                    if device_id != 0 {
                        break;
                    }
                }
                Err(e) => error = Some(e),
            }
        }

        if let Some(err) = error
            && !any_success
        {
            return Err(err);
        }

        Ok(physical_key)
    }

    pub fn read_keycodes<F>(&self, f: F) -> Result<(), ReadError>
    where
        F: FnOnce(Context<KeyCodeFilter>),
    {
        self.read_keycodes_for(0, f)
    }

    pub fn read_keycodes_for<F>(&self, device_id: DeviceID, f: F) -> Result<(), ReadError>
    where
        F: FnOnce(Context<KeyCodeFilter>),
    {
        let mut error = None;
        let mut any_success = false;

        for p in self.state.lock_plugins().iter_mut() {
            match p.read_keycodes(device_id) {
                Ok(plugin_data) => {
                    for (k, v) in plugin_data {
                        self.lock_keycodes()
                            .entry(k)
                            .and_modify(|existing| {
                                if &v > existing {
                                    *existing = v;
                                }
                            })
                            .or_insert(v);
                    }

                    any_success = true;
                }
                Err(e) => {
                    error = Some(e);
                }
            }

            // If looking for a specific device, break after first successful read
            if device_id != 0 && any_success {
                break;
            }
        }

        if let Some(err) = error
            && !any_success
        {
            return Err(err);
        }

        f(Context::with_keycodes(std::mem::take(
            &mut self.lock_keycodes(),
        )));

        Ok(())
    }

    pub fn read_positions<F>(&self, f: F) -> Result<(), ReadError>
    where
        F: FnOnce(Context<PositionFilter>),
    {
        self.read_positions_for(0, f)
    }

    pub fn read_positions_for<F>(&self, device_id: DeviceID, f: F) -> Result<(), ReadError>
    where
        F: FnOnce(Context<PositionFilter>),
    {
        let mut error = None;
        let mut any_success = false;

        for p in self.state.lock_plugins().iter_mut() {
            match p.read_positions(device_id) {
                Ok(plugin_data) => {
                    for (_, physical_key) in plugin_data {
                        self.lock_positions()
                            .entry(physical_key.pos)
                            .and_modify(|existing: &mut PhysicalKey| {
                                if physical_key.max_value() > existing.max_value() {
                                    *existing = physical_key;
                                }
                            })
                            .or_insert(physical_key);
                    }

                    any_success = true;
                }
                Err(e) => {
                    error = Some(e);
                }
            }

            // If looking for a specific device, break after first successful read
            if device_id != 0 && any_success {
                break;
            }
        }

        if let Some(err) = error
            && !any_success
        {
            return Err(err);
        }

        f(Context::with_positions(std::mem::take(
            &mut self.lock_positions(),
        )));

        Ok(())
    }

    pub fn get_device_info(&self) -> Result<Vec<DeviceInfo>, ReadError> {
        let mut devices: Vec<DeviceInfo> = vec![];
        let mut error = None;
        for p in self.state.lock_plugins().iter_mut() {
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

    pub fn set_device_events<F>(&mut self, f: F)
    where
        F: Fn(DeviceEventType, DeviceInfo) + 'static + Send + Sync,
    {
        self.device_events.replace(Arc::new(f));
    }

    pub fn clear_device_event_cb(&mut self) {
        self.device_events = None;
    }

    pub fn uninitialise(self) -> AnalogSdk<Uninitialised> {
        AnalogSdk::default()
    }

    pub fn device_count(&self) -> u32 {
        self.state.device_count
    }
}

impl<S> AnalogSdk<S> {
    pub(crate) fn lock_keycodes(&self) -> MutexGuard<'_, HashMap<KeyCode, AnalogValue>> {
        self.keycodes.lock().expect("mutex should not be poisoned")
    }

    pub(crate) fn lock_positions(&self) -> MutexGuard<'_, HashMap<KeyPosition, PhysicalKey>> {
        self.positions.lock().expect("mutex should not be poisoned")
    }
}

fn load_plugins(path: &Path) -> Result<Vec<DynamicPlugin>, PluginError> {
    if !path.is_dir() {
        return Err(PluginError::InvalidDirectory(path.to_path_buf()));
    }

    let mut plugins = Vec::new();

    for entry in fs::read_dir(path)? {
        let path = entry?.path();

        if let Some(ext) = path.extension().and_then(OsStr::to_str)
            && ext == DLL_EXTENSION
        {
            info!("Loading plugin: \"{}\"", path.display());

            match load_plugin(&path) {
                Ok(plugin) => plugins.push(plugin),
                Err(e) => error!("failed to load plugin: {e}"),
            }
        }
    }

    Ok(plugins)
}

fn load_plugin(path: &Path) -> Result<DynamicPlugin, PluginError> {
    if path.is_dir() {
        return Err(PluginError::InvalidPlugin(path.to_path_buf()));
    }

    let mut plugin = DynamicPlugin::new(
        unsafe { Library::new(path) }
            .map_err(|e| PluginError::DynamicLibraryError { source: e })?,
    )?;

    plugin
        .name()
        .inspect(|name| info!("Loaded plugin: {:?}", name))?;

    Ok(plugin)
}

#[test]
fn test_sendsync() {
    // Validate all types remain Send + Sync
    fn assert_types<T: Send + Sync>() {}

    assert_types::<AnalogSdk>();
    assert_types::<Initialised>();
    assert_types::<Uninitialised>();
    assert_types::<AnalogSdk<Uninitialised>>();
    assert_types::<AnalogSdk<Initialised>>();
    assert_types::<fn(DeviceEventType, DeviceInfo)>();
    assert_types::<Box<dyn Fn(DeviceEventType, DeviceInfo) + Send + Sync>>();
}
