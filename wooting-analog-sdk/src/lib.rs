//! # Wooting Analog SDK
//! The Wooting Analog SDK is the open driver for Analog keyboards. It's goal is to create native
//! support for Analog keyboards in any game or application.
//!
//! ## Documentation
#![doc = concat!("- [SDK Usage](https://github.com/WootingKb/wooting-analog-sdk/blob/v", env!("CARGO_PKG_VERSION"), "/docs/SDK_USAGE.md) on how to use the SDK")]
#![doc = concat!("- [Installation Guide](https://github.com/WootingKb/wooting-analog-sdk/blob/v", env!("CARGO_PKG_VERSION"), "/docs/INSTALL.md) on how to use and install the distributable and the system SDK.")]
//!
//! ### Developers
#![doc = concat!("- [Contributing Guide](https://github.com/WootingKb/wooting-analog-sdk/blob/v", env!("CARGO_PKG_VERSION"), "/docs/CONTRIBUTING.md) to help you with your first contribution.")]
#![doc = concat!("- [Build Instructions](https://github.com/WootingKb/wooting-analog-sdk/blob/v", env!("CARGO_PKG_VERSION"), "/docs/BUILD.md) on how to build the SDK using Rust.")]
#![doc = concat!("- [Migration Guide](https://github.com/WootingKb/wooting-analog-sdk/blob/v", env!("CARGO_PKG_VERSION"), "/docs/MIGRATION_GUIDE.md) on how to upgrade to the latest version of the SDK.")]
#![doc = concat!("- [Virtual Keyboard](https://github.com/WootingKb/wooting-analog-sdk/blob/v", env!("CARGO_PKG_VERSION"), "/docs/VIRTUAL_KEYBOARD.md) on how to setup and use the virtual keyboard for development without any hardware required.")]
#![doc = concat!("- [Plugin Introduction](https://github.com/WootingKb/wooting-analog-sdk/blob/v", env!("CARGO_PKG_VERSION"), "/docs/PLUGINS.md) for information on creating plugins.")]
//!
//! ## Example
//! ```no_run
//! use wooting_analog_sdk::{AnalogSdk, Initialised};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Configure and initialise the Analog SDK
//! let analog_sdk: AnalogSdk<Initialised> = AnalogSdk::new().initialise()?;
//!
//! loop {
//!     // Poll all available plugins for values by keycode
//!     analog_sdk.read_keycodes(|ctx| {
//!         for (keycode, value) in ctx.iter() {
//!             println!("read keycode: {keycode} with value: {value}");
//!         }
//!     })?;
//!
//!     // Poll all available plugins for values by their matrix position
//!     analog_sdk.read_positions(|ctx| {
//!         for physical_key in ctx.iter() {
//!             println!(
//!                 "read from position: {} with values: {:?}",
//!                 physical_key.position,
//!                 physical_key.state(),
//!             );
//!         }
//!     })?;
//!     # break; // otherwise we will never get out of our doc test run
//! }
//! # Ok(())
//! # }
//! ```

// Add feature badges to show what feature something is gated behind
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

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
    key::{KeyPosition, KeyState, PhysicalKey},
    plugin::Plugin,
};

use crate::{
    analog_value::ValueMetadata,
    ctx::{Ctx, KeyCodeFormat, PositionFormat},
    device::DeviceID,
    device::{DeviceEventType, DeviceInfo},
    err::{PluginError, ReadError},
    keycode::KeycodeType,
    plugin::{DEFAULT_PLUGIN_DIR, dynamic::DynamicPlugin, wooting::WootingPlugin},
};
use libloading::Library;
use log::{debug, error, info, trace, warn};
use std::{
    collections::HashMap,
    env::consts::DLL_EXTENSION,
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard},
    thread,
};

/// A typestate marker for the state of the [`AnalogSdk`].
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

/// A typestate marker for the state of the [`AnalogSdk`].
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

        Ok(plugins)
    }
}

/// The main way to poll data from registered analog devices.
#[derive(Default)]
pub struct AnalogSdk<S = Uninitialised> {
    pub(crate) keycode_mode: KeycodeType,
    device_events: Option<Arc<dyn Fn(DeviceEventType, DeviceInfo) + Send + Sync>>,
    keycodes: Mutex<HashMap<KeyCode, AnalogValue>>,
    positions: Mutex<HashMap<KeyPosition, PhysicalKey>>,
    state: S,
}

impl AnalogSdk<Uninitialised> {
    /// A builder to mark and configure any plugins it needs to load on initialisation.
    ///
    /// ```
    /// use wooting_analog_sdk::{AnalogSdk, keycode::KeycodeType};
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // Configure the SDK before initialising using a builder pattern
    /// let analog_sdk = AnalogSdk::new()
    ///     .with_plugin_directory("/path/to/plugins", true)
    ///     .with_device_events(|event, info| {
    ///         println!("received event: {event:?} for device: {}", info.device_id);
    ///      })
    ///     .with_keycode_mode(KeycodeType::VirtualKey)
    ///     .initialise()?;
    /// # Ok(())
    /// # }
    /// ```
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

    /// Wooting devices are loaded by default. Only use this when you want to operate over
    /// third-party devices and do not want to include Wooting devices in your results.
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
        let mut plugins = match self.state.load_plugins_from_dir() {
            Ok(plugins) => plugins,
            Err(PluginError::InvalidDirectory(path)) => {
                warn!("plugin directory \"{path:?}\" invalid");
                Vec::new()
            }
            Err(e) => return Err(e),
        };

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

        if plugins.is_empty() {
            return Err(PluginError::ZeroPlugins);
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
    /// Read the single highest analog value across any connected devices by keycode.
    pub fn read_keycode<T>(&self, code: T) -> Result<AnalogValue, ReadError>
    where
        T: Into<u16>,
    {
        self.read_keycode_from(0, code)
    }

    /// Read the single highest analog value from a specific device by keycode.
    pub fn read_keycode_from<T>(
        &self,
        device_id: DeviceID,
        code: T,
    ) -> Result<AnalogValue, ReadError>
    where
        T: Into<u16>,
    {
        let code = code.into();

        // Since the compiler monomorphizes entire functions for every concrete type that fits the
        // bounds of the generic type paremeters, it can be quite costly to have it do so for larger
        // functions. These inner functions reduce compile times and binary size. The outer function
        // will be monomorphized and this inner will be reused for each one.
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

    /// Read all properties of a single physical key across any connected devices by matrix position.
    pub fn read_position(&self, position: KeyPosition) -> Result<PhysicalKey, ReadError> {
        self.read_position_from(0, position)
    }

    /// Read all properties of a single physical key from a specific device by matrix position.
    pub fn read_position_from(
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
                        physical_key.push_state(pk.state[i as usize]);
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

    /// Read all highest analog values across any connected devices, formatted by keycode.
    pub fn read_keycodes<F, R>(&self, ctx: F) -> Result<R, ReadError>
    where
        F: FnOnce(Ctx<KeyCodeFormat<'_>>) -> R,
    {
        self.read_keycodes_from(0, ctx)
    }

    /// Read all highest analog values from a specific device, formatted by keycode.
    pub fn read_keycodes_from<F, R>(&self, device_id: DeviceID, ctx: F) -> Result<R, ReadError>
    where
        F: FnOnce(Ctx<KeyCodeFormat<'_>>) -> R,
    {
        let mut guard = self.lock_keycodes();

        // Since the compiler monomorphizes entire functions for every concrete type that fits the
        // bounds of the generic type paremeters, it can be quite costly to have it do so for larger
        // functions. These inner functions reduce compile times and binary size. The outer function
        // will be monomorphized and this inner will be reused for each one.
        fn inner_read_keycodes_from(
            sdk: &AnalogSdk<Initialised>,
            device_id: DeviceID,
            guard: &mut MutexGuard<'_, HashMap<KeyCode, AnalogValue>>,
        ) -> Result<(), ReadError> {
            let mut error = None;
            let mut any_success = false;

            for p in sdk.state.lock_plugins().iter_mut() {
                match p.read_keycodes(device_id) {
                    Ok(plugin_data) => {
                        for (mut k, v) in plugin_data {
                            if let Some(code) = crate::keycode::hid_to_code(k, &sdk.keycode_mode) {
                                k.inner = code;
                            }

                            guard
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

            Ok(())
        }

        inner_read_keycodes_from(self, device_id, &mut guard)?;

        let result = ctx(Ctx::with_keycodes(&mut guard));
        guard.clear();

        Ok(result)
    }

    /// Read all pressed physical keys across any connected devices, formatted by matrix position.
    pub fn read_positions<F, R>(&self, ctx: F) -> Result<R, ReadError>
    where
        F: FnOnce(Ctx<PositionFormat>) -> R,
    {
        self.read_positions_from(0, ctx)
    }

    /// Read all pressed physical keys for a specific device, formatted by matrix position.
    pub fn read_positions_from<F, R>(&self, device_id: DeviceID, ctx: F) -> Result<R, ReadError>
    where
        F: FnOnce(Ctx<PositionFormat>) -> R,
    {
        let mut guard = self.lock_positions();

        // Since the compiler monomorphizes entire functions for every concrete type that fits the
        // bounds of the generic type paremeters, it can be quite costly to have it do so for larger
        // functions. These inner functions reduce compile times and binary size. The outer function
        // will be monomorphized and this inner will be reused for each one.
        fn inner_read_positions_from(
            sdk: &AnalogSdk<Initialised>,
            device_id: DeviceID,
            guard: &mut MutexGuard<'_, HashMap<KeyPosition, PhysicalKey>>,
        ) -> Result<(), ReadError> {
            let mut error = None;
            let mut any_success = false;

            for p in sdk.state.lock_plugins().iter_mut() {
                match p.read_positions(device_id) {
                    Ok(plugin_data) => {
                        for (_, physical_key) in plugin_data {
                            guard
                                .entry(physical_key.position)
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
            Ok(())
        }

        inner_read_positions_from(self, device_id, &mut guard)?;

        let result = ctx(Ctx::with_positions(&mut guard));
        guard.clear();

        Ok(result)
    }

    pub fn connected_devices(&self) -> Result<Vec<DeviceInfo>, ReadError> {
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

    pub fn insert_plugin<P: Plugin + 'static>(&mut self, plugin: P) {
        self.state.lock_plugins().push(Box::new(plugin));
    }

    /// Uninitialises all plugins and then falls back to the default uninitialised state.
    pub fn uninitialise(self) -> AnalogSdk<Uninitialised> {
        AnalogSdk::default()
    }

    pub fn keycode_mode(&self) -> &KeycodeType {
        &self.keycode_mode
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
