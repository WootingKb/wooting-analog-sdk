use crate::cplugin::*;
use crate::keycode::*;
use anyhow::{Context, Error, Result};
use libloading::{Library, Symbol};
use log::{error, info, warn};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::{fs, thread};
use wooting_analog_common::*;
use wooting_analog_plugin_dev::*;

//This is so that we can ensure that the separate tests which use the test plugin can ensure that they aren't running at the same time
#[cfg(test)]
lazy_static! {
    pub static ref TEST_PLUGIN_LOCK: Mutex<()> = Mutex::new(());
}

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
        if !plugin_dir.is_dir() {
            error!("The plugin directory '{:?}' does not exist! Make sure you have it created and have plugins in there", plugin_dir);
            return Err(WootingAnalogResult::NoPlugins).into();
        }
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
            let plugin_data = p
                .read_full_buffer(max_length - analog_data.len(), device_id)
                .into();
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

#[cfg(test)]
mod tests {
    use super::*;
    use shared_memory::*;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    struct SharedState {
        pub vendor_id: u16,
        /// Device Product ID `pid`
        pub product_id: u16,
        //TODO: Consider switching these to FFiStr
        /// Device Manufacturer name
        pub manufacturer_name: [u8; 20],
        /// Device name
        pub device_name: [u8; 20],

        pub device_type: DeviceType,

        pub device_connected: bool,
        pub dirty_device_info: bool,

        pub analog_values: [u8; 0xFF],
    }

    unsafe impl SharedMemCast for SharedState {}

    fn shared_init() {
        env_logger::try_init_from_env(env_logger::Env::from("debug"))
            .map_err(|e| println!("ERROR: Could not initialise env_logger. '{:?}'", e));
    }

    #[test]
    fn initialise_no_plugins() {
        shared_init();

        let dir = "./test_np";
        ::std::fs::create_dir(dir);

        //Don't need to use the Singleton instance of the SDK here as we're not actually gonna initialise it
        let mut sdk = AnalogSDK::new();
        assert_eq!(
            sdk.initialise_with_plugin_path(dir, true).0,
            Err(WootingAnalogResult::NoPlugins)
        );
        assert!(!sdk.initialised);
        ::std::fs::remove_dir(dir);
    }

    #[test]
    fn initialise_no_dir() {
        shared_init();

        let dir = "./test_n";
        ::std::fs::remove_dir(dir);

        //Don't need to use the Singleton instance of the SDK here as we're not actually gonna initialise it
        let mut sdk = AnalogSDK::new();
        assert_eq!(
            sdk.initialise_with_plugin_path(dir, true).0,
            Err(WootingAnalogResult::NoPlugins)
        );
        assert!(!sdk.initialised)
    }

    fn get_wlock(shmem: &mut SharedMem) -> WriteLockGuard<SharedState> {
        match shmem.wlock::<SharedState>(0) {
            Ok(v) => v,
            Err(_) => panic!("Failed to acquire write lock !"),
        }
    }

    //    lazy_static! { static ref  }
    //    fn connect_cb

    fn wait_for_connected(got_connected: &Arc<Mutex<bool>>, attempts: u32, connected: bool) {
        let mut n = 0;
        while *got_connected.lock().unwrap() != connected {
            if n > attempts {
                panic!(
                    "Waiting for device to be connected status: {:?} timed out!",
                    connected
                );
            }
            ::std::thread::sleep(Duration::from_millis(500));
            n += 1;
        }
        info!("Got {:?} after {} attempts", connected, n);
    }

    #[test]
    fn initialise_test_plugin() {
        shared_init();

        //Claim the mutex lock
        let _lock = TEST_PLUGIN_LOCK.lock().unwrap();

        let got_connected: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
        let got_connected_borrow = got_connected.clone();
        let mut _sdk = Arc::new(Mutex::new(AnalogSDK::new()));
        let sdk_borrow = _sdk.clone();
        let sdk = || _sdk.lock().unwrap();
        let dir = format!(
            "../target/{}/test_plugin",
            std::env::var("TEST_TARGET").unwrap_or("debug".to_owned())
        );
        info!("Loading plugins from: {:?}", dir);
        assert!(!sdk().initialised);
        assert_eq!(
            sdk()
                .initialise_with_plugin_path(dir.as_str(), !dir.ends_with("debug"))
                .0,
            Ok(0)
        );
        assert!(sdk().initialised);

        //Wait a slight bit to ensure that the test-plugin worker thread has initialised the shared mem
        ::std::thread::sleep(Duration::from_millis(500));

        let mut shmem = match SharedMem::open_linked(
            std::env::temp_dir()
                .join("wooting-test-plugin.link")
                .as_os_str(),
        ) {
            Ok(v) => v,
            Err(e) => {
                println!("Error : {}", e);
                println!("Failed to open SharedMem...");
                assert!(false);
                return;
            }
        };

        sdk().set_device_event_cb(move |event: DeviceEventType, _device: DeviceInfo| {
            debug!("Got cb {:?}", event);

            *got_connected_borrow.lock().unwrap() = event == DeviceEventType::Connected;
            if event == DeviceEventType::Connected {
                debug!("Started reading");
                assert_eq!(sdk_borrow.lock().unwrap().read_analog(1, 0).0, Ok(0.0));
                assert_eq!(
                    sdk_borrow
                        .lock()
                        .unwrap()
                        .get_device_info()
                        .0
                        .map(|dev| dev.len()),
                    Ok(1)
                );
                debug!("Finished reading");
            }
        });

        //Check the connected cb is called
        {
            {
                let mut shared_state = get_wlock(&mut shmem);
                shared_state.device_connected = true;
            }
            wait_for_connected(&got_connected, 5, true);
        }

        //Check that we now have one device
        {
            assert_eq!(sdk().get_device_info().0.map(|dev| dev.len()), Ok(1));
        }

        //Check the cb is called with disconnected
        {
            {
                let mut shared_state = get_wlock(&mut shmem);
                shared_state.device_connected = false;
            }
            wait_for_connected(&got_connected, 5, false);
        }

        //Check that we now have no devices
        {
            assert_eq!(sdk().get_device_info().0.map(|dev| dev.len()), Ok(0));
        }

        let analog_val = 0xF4;
        let f_analog_val = f32::from(analog_val) / 255_f32;
        let analog_key = 5;
        //Connect the device again, set a keycode to a val
        let device_id = {
            let mut shared_state = get_wlock(&mut shmem);
            shared_state.analog_values[analog_key] = analog_val;
            shared_state.device_connected = true;
            1
        };

        wait_for_connected(&got_connected, 5, true);

        //Check we get the val with no id specified
        assert_eq!(sdk().read_analog(analog_key as u16, 0).0, Ok(f_analog_val));
        //Check we get the val with the device_id we use
        assert_eq!(
            sdk().read_analog(analog_key as u16, device_id).0,
            Ok(f_analog_val)
        );
        //Check we don't get a val with invalid device id
        assert_eq!(
            sdk().read_analog(analog_key as u16, device_id + 1).0,
            Err(WootingAnalogResult::NoDevices)
        );
        //Check if the next value is 0
        assert_eq!(
            sdk().read_analog((analog_key + 1) as u16, device_id).0,
            Ok(0.0)
        );

        //Check that it does code mapping
        sdk().keycode_mode = KeycodeType::ScanCode1;
        assert_eq!(
            sdk()
                .read_analog(
                    hid_to_code(analog_key as u16, &KeycodeType::ScanCode1).unwrap(),
                    device_id
                )
                .0,
            Ok(f_analog_val)
        );
        sdk().keycode_mode = KeycodeType::HID;

        let buffer_len = 5;
        let analog_data = sdk().read_full_buffer(buffer_len, 0).0.unwrap();
        //Check it reads buffer properly with no device id
        assert_eq!(analog_data.len(), 1);
        assert_eq!(
            analog_data.iter().next(),
            Some((&(analog_key as u16), &f_analog_val))
        );

        let analog_data = sdk().read_full_buffer(buffer_len, device_id).0.unwrap();
        //Check it reads buffer properly with proper device_id
        assert_eq!(analog_data.len(), 1);
        assert_eq!(
            analog_data.iter().next(),
            Some((&(analog_key as u16), &f_analog_val))
        );

        //Check it errors on read buffer with invalid device_id
        assert_eq!(
            sdk().read_full_buffer(buffer_len, device_id + 1).0,
            Err(WootingAnalogResult::NoDevices)
        );

        //Check that it does code mapping
        sdk().keycode_mode = KeycodeType::ScanCode1;
        let analog_data = sdk().read_full_buffer(buffer_len, device_id).0.unwrap();
        assert_eq!(analog_data.len(), 1);
        assert_eq!(
            analog_data.iter().next(),
            Some((
                &hid_to_code(analog_key as u16, &KeycodeType::ScanCode1).unwrap(),
                &f_analog_val
            ))
        );
        sdk().keycode_mode = KeycodeType::HID;

        {
            let mut shared_state = get_wlock(&mut shmem);
            shared_state.analog_values[analog_key] = 0;
        }
        ::std::thread::sleep(Duration::from_secs(1));
        let analog_data = sdk().read_full_buffer(buffer_len, device_id).0.unwrap();
        //Check that it is returning the released key in the next call
        assert_eq!(analog_data.len(), 1);
        assert_eq!(analog_data[&(analog_key as u16)], 0.0);

        assert_eq!(sdk().read_analog(analog_key as u16, 0).0, Ok(0.0));

        let analog_data = sdk().read_full_buffer(buffer_len, device_id).0;
        assert_eq!(analog_data.unwrap().len(), 0);

        sdk().clear_device_event_cb();
        {
            let mut shared_state = get_wlock(&mut shmem);
            shared_state.device_connected = false;
        }
        ::std::thread::sleep(Duration::from_secs(1));
        //This shouldn't have updated if the cb is not there
        assert!(*Arc::clone(&got_connected).lock().unwrap());

        sdk().unload();
    }

    #[test]
    fn unitialised_sdk_functions_new() {
        shared_init();

        let mut sdk = AnalogSDK::new();
        uninitialised_sdk_functions(&mut sdk);
    }

    const TEST_PLUGIN_DIR: &str = "test_c_plugin";

    /// Basic test to ensure the plugin.h is up to date and to ensure the CPlugin interface is working correctly
    #[test]
    fn test_c_plugin_interface() {
        shared_init();
        let mut sdk = AnalogSDK::new();

        let dir = format!("./{}/build/", TEST_PLUGIN_DIR);
        info!("Loading plugins from: {:?}", dir);
        assert!(!sdk.initialised);
        assert_eq!(sdk.initialise_with_plugin_path(dir.as_str(), true).0, Ok(1));
        assert!(sdk.initialised);
        let got_cb = Arc::new(AtomicBool::new(false));
        let got_cb_inner = got_cb.clone();

        sdk.set_device_event_cb(move |event, device| {
            println!("We got that callbackkkk {:?} {:?}", event, device);
            got_cb_inner.store(true, Ordering::Relaxed);

            //A couple of basic checks to ensure the callback gets valid data
            assert_eq!(event, DeviceEventType::Connected);
            assert_eq!(device.device_id, 7);
        });

        assert_eq!(sdk.read_analog(30, 0).0, Ok(0.56));
        //We told it to execute the callback when read_analog is called so let's just call it a second time to ensure it can be called multiple times without dying
        assert_eq!(sdk.read_analog(30, 0).0, Ok(0.56));
        assert_eq!(sdk.read_full_buffer(30, 0).0.unwrap().get(&5), Some(&0.4));
        let device = sdk.get_device_info().0.unwrap().first().unwrap().clone();
        println!("Got device: {:?}", device);
        assert_eq!(device.device_id, 7);

        //Wait a wee bit to ensure the callback has been executed
        ::std::thread::sleep(Duration::from_millis(500));

        assert!(got_cb.load(Ordering::Relaxed));
    }

    /*#[test]
    fn unitialised_sdk_functions_failed_init() {
        shared_init();

        let mut sdk = AnalogSDK::new();
        let dir = "./test_n";
        ::std::fs::remove_dir(dir);
        ::std::env::set_var(ENV_PLUGIN_DIR_KEY, dir);

        assert_eq!(sdk.initialise(), WootingAnalogResult::NoPlugins);
        assert!(!sdk.initialised);

        uninitialised_sdk_functions(&mut sdk);
    }*/

    fn cb(_event: DeviceEventType, _device: DeviceInfo) {}

    fn uninitialised_sdk_functions(sdk: &mut AnalogSDK) {
        assert_eq!(sdk.initialised, false);
        assert_eq!(
            sdk.set_device_event_cb(cb).0,
            Err(WootingAnalogResult::UnInitialized)
        );
        assert_eq!(
            sdk.clear_device_event_cb().0,
            Err(WootingAnalogResult::UnInitialized)
        );
        assert_eq!(
            sdk.read_analog(0, 0).0,
            Err(WootingAnalogResult::UnInitialized)
        );
        assert_eq!(
            sdk.get_device_info().0.err(),
            Some(WootingAnalogResult::UnInitialized)
        );

        assert_eq!(
            sdk.read_full_buffer(0, 0).0,
            Err(WootingAnalogResult::UnInitialized)
        );
    }
}
