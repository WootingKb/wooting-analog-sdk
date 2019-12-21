use crate::cplugin::*;
use crate::errors::*;
use crate::keycode::*;
use libloading::{Library, Symbol};
use log::{error, info, warn};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::{fs, thread};
use std::os::raw::{c_float, c_int, c_ushort};
use std::path::{Path, PathBuf};
use wooting_analog_common::*;
use wooting_analog_plugin_dev::*;
use std::sync::{Mutex, MutexGuard, Arc};

unsafe impl Send for AnalogSDK {}

pub struct AnalogSDK {
    pub initialised: bool,
    pub keycode_mode: KeycodeType,

    plugins: Vec<Box<dyn Plugin>>,
    loaded_libraries: Vec<Library>,
    device_event_callback: Arc<Mutex<Option<Box<dyn Fn(DeviceEventType, DeviceInfoPointer) + Send>>>>
}

#[cfg(target_os = "macos")]
static LIB_EXT: &str = "dylib";
#[cfg(target_os = "linux")]
static LIB_EXT: &str = "so";
#[cfg(target_os = "windows")]
static LIB_EXT: &str = "dll";

lazy_static! {
    pub static ref ANALOG_SDK: Mutex<AnalogSDK> = Mutex::new(AnalogSDK::new());
}

pub fn get_sdk() -> MutexGuard<'static, AnalogSDK> {
    ANALOG_SDK.lock().unwrap()
}

extern "C" fn device_event_cb(event: DeviceEventType, device: DeviceInfoPointer) {
    let opt_cb = {
        //debug!("device event cb {:?}", ANALOG_SDK.lock().unwrap().device_event_callback);
        ANALOG_SDK.lock().unwrap().device_event_callback.clone()
    };
    thread::spawn(move || {
        debug!("device event cb thread running");

        opt_cb.lock().unwrap().as_ref().and_then(|cb|  {
            debug!("calling og callback");
            cb(event, device);
            Some(0)
        });
    });
}

impl AnalogSDK {
    fn new() -> AnalogSDK {
        AnalogSDK {
            plugins: Vec::new(),
            loaded_libraries: Vec::new(),
            initialised: false,
            keycode_mode: KeycodeType::HID,
            device_event_callback: Arc::new(Mutex::new(None))
        }
    }

    pub fn initialise(&mut self) -> SDKResult<u32> {
        self.initialise_with_plugin_path(DEFAULT_PLUGIN_DIR, true)
    }

    pub fn initialise_with_plugin_path(&mut self, plugin_dir: &str, nested: bool) -> SDKResult<u32> {
        if self.initialised {
            self.unload();
        }

        let plugin_dir = PathBuf::from(plugin_dir);
        if !plugin_dir.is_dir() {
            error!("The plugin directory '{}' does not exist! Make sure you have it created and have plugins in there", DEFAULT_PLUGIN_DIR);
            return WootingAnalogResult::NoPlugins.into();
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
                    warn!("Failed to load any plugins from {:?}!", dir);
                    //self.initialised = false;
                    //WootingAnalogResult::NoPlugins
                }
                Ok(i) => {
                    info!("Loaded {} plugins from {:?}", i, dir);
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
                    },
                    Err(e) => {
                        error!("Error reading directory: {}", e);
                    }
                }
            }
        }

        let mut plugins_initialised = 0;
        let mut device_no:u32 = 0;
        for p in self.plugins.iter_mut() {
            let ret = p.initialise(device_event_cb);
            info!("{:?}", ret);
            if let Ok(num) = ret.0  {
                plugins_initialised += 1;
                device_no += num;
            }
        }
        info!("{} plugins successfully initialised", plugins_initialised);

        self.initialised = plugins_initialised > 0;
        if !self.initialised {
            WootingAnalogResult::NoPlugins.into()
        } else {
            Ok(device_no).into()
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
                            if self.load_plugin(&path).map_err(|e| error!("{:?}", e)).is_ok() {
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
            return Err("Path is directory!".into());
        }

        type PluginCreate = unsafe extern "C" fn() -> *mut dyn Plugin;
        type PluginVersion = unsafe extern "C" fn() -> &'static str;

        let lib = Library::new(filename.as_os_str()).chain_err(|| "Unable to load the plugin")?;

        // We need to keep the library around otherwise our plugin's vtable will
        // point to garbage. We do this little dance to make sure the library
        // doesn't end up getting moved.
        self.loaded_libraries.push(lib);

        let lib = self.loaded_libraries.last().unwrap();

        let full_version: Option<Symbol<PluginVersion>> = lib.get(b"plugin_version").ok();
        if let Some(ver) = full_version {
            info!("Plugin got plugin-dev sem version: {}", ver());
        }
        else{
            debug!("No symbol");
        }

        let constructor: Option<Symbol<PluginCreate>> = lib
            .get(b"_plugin_create")
            .map_err(|e| {
                warn!("Find constructor error: {}", e);
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
                if lib.get::<unsafe extern "C" fn() -> i32>(b"_initialise").is_ok() {
                    Box::new(CPlugin::new(lib))
                } else {
                    return Err("Plugin isn't a valid C or Rust plugin, couldn't find entry point".into());
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
                return Err("Plugin isn't a valid plugin, name function not found".into());
            },
            Err(e) => {
                return Err(format!("Plugin failed with unhandled error {:?}", e).into());
            }
        }


        Ok(())
    }

    pub fn set_device_event_cb(
        &mut self,
        cb: impl Fn(DeviceEventType, DeviceInfoPointer) + 'static + Send,
    ) -> SDKResult<()> {
        if !self.initialised {
            return WootingAnalogResult::UnInitialized.into();
        }
        self.device_event_callback.lock().unwrap().replace(Box::new(cb));

        Ok(()).into()
    }

    pub fn clear_device_event_cb(&mut self) -> SDKResult<()> {
        if !self.initialised {
            return WootingAnalogResult::UnInitialized.into();
        }
        self.device_event_callback.lock().unwrap().take();

        Ok(()).into()
    }

    pub fn get_device_info(&mut self) -> SDKResult<Vec<DeviceInfoPointer>> {
        if !self.initialised {
            return WootingAnalogResult::UnInitialized.into();
        }
        let mut devices: Vec<DeviceInfoPointer> = vec![];
        let mut error: WootingAnalogResult = WootingAnalogResult::Ok;
        for p in self.plugins.iter_mut() {
            //Give a reference to the buffer at the point where there is free space
            match p.device_info().0 {
                Ok(mut p_devices) => {
                    devices.append(p_devices.as_mut());
                },
                Err(e) => {
                    error!("Plugin {:?} failed to fetch devices with error {:?}", p.name(), e);
                    error = e;
                }
            }
        }
        if devices.is_empty() && !error.is_ok() {
            Err(error).into()
        }else {
            Ok(devices).into()
        }
    }

    pub fn read_analog(&mut self, code: u16, device_id: DeviceID) -> SDKResult<f32> {
        if !self.initialised {
            return WootingAnalogResult::UnInitialized.into();
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
                return err.into();
            }

            value.into()
        } else {
            WootingAnalogResult::NoMapping.into()
        }
    }

    pub fn read_full_buffer(
        &mut self,
        max_length: usize,
        device_id: DeviceID,
    ) -> SDKResult<HashMap<c_ushort, c_float>> {
        if !self.initialised {
            return WootingAnalogResult::UnInitialized.into();
        }

        let mut analog_data: HashMap<c_ushort, c_float> = HashMap::with_capacity(max_length);

        let mut err = WootingAnalogResult::Ok;
        let mut any_success = false;
        //Read from all and add up
        for p in self.plugins.iter_mut() {
            let plugin_data = p.read_full_buffer(max_length-analog_data.len(), device_id).into();
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
            return err.into();
        }

        Ok(analog_data).into()
    }

    /// Unload all plugins and loaded plugin libraries, making sure to fire
    /// their `on_plugin_unload()` methods so they can do any necessary cleanup.
    pub fn unload(&mut self) {
        debug!("Unloading plugins");
        self.initialised = false;
        for mut plugin in self.plugins.drain(..) {
            trace!("Firing on_plugin_unload for {:?}", plugin.name());
            plugin.unload();
        }

        for lib in self.loaded_libraries.drain(..) {
            drop(lib);
        }

        self.device_event_callback.lock().unwrap().take();
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
    use std::time::Duration;
    use std::sync::{Arc,Mutex};

    struct SharedState {
        pub vendor_id: u16,
        /// Device Product ID `pid`
        pub product_id: u16,
        //TODO: Consider switching these to FFiStr
        /// Device Manufacturer name
        pub manufacturer_name: [u8; 20],
        /// Device name
        pub device_name: [u8; 20],
        /// Unique device ID, which should be generated using `generate_device_id`
        pub device_id: u64,

        pub device_type: DeviceType,

        pub device_connected: bool,
        pub dirty_device_info: bool,

        pub analog_values: [u8; 0xFF]
    }

    unsafe impl SharedMemCast for SharedState {}

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

        //Don't need to use the Singleton instance of the SDK here as we're not actually gonna initialise it
        let mut sdk = AnalogSDK::new();
        assert_eq!(sdk.initialise_with_plugin_path(dir, true).0, Err(WootingAnalogResult::NoPlugins));
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
        assert_eq!(sdk.initialise_with_plugin_path(dir, true).0, Err(WootingAnalogResult::NoPlugins));
        assert!(!sdk.initialised)
    }

    fn get_wlock(shmem: &mut SharedMem) -> WriteLockGuard<SharedState> {
        match shmem.wlock::<SharedState>(0) {
            Ok(v) => v,
            Err(_) => panic!("Failed to acquire write lock !"),
        }
    }

    lazy_static! { static ref got_connected: Arc<Mutex<bool>> = Arc::new(Mutex::new(false)); }
    extern "C" fn connect_cb(event: DeviceEventType, device: DeviceInfoPointer) {
        debug!("Got cb {:?}", event);

        *Arc::clone(&got_connected).lock().unwrap() = event == DeviceEventType::Connected;
        if event == DeviceEventType::Connected {
            debug!("Started reading");
            assert_eq!(get_sdk().read_analog(1, 0).0, Ok(0.0));
            assert_eq!(get_sdk().get_device_info().0.map(|dev| dev.len()), Ok(1));
            debug!("Finished reading");
        }
    }

    fn connect_cb_wrap(event: DeviceEventType, device: DeviceInfoPointer) {
        connect_cb(event, device);
    }

    fn wait_for_connected(attempts: u32, connected: bool) {
        let mut n = 0;
        while *Arc::clone(&got_connected).lock().unwrap() != connected {
            if n > attempts {
                panic!("Waiting for device to be connected status: {:?} timed out!", connected);
            }
            ::std::thread::sleep(Duration::from_millis(500));
            n+=1;
        }
        info!("Got {:?} after {} attempts", connected, n);
    }

    #[test]
     fn initialise_test_plugin() {
        shared_init();


        let dir = format!("../wooting-analog-test-plugin/target/{}", std::env::var("TARGET").unwrap_or("/debug".to_owned()));
        info!("Loading plugins from: {:?}", dir);
        assert!(!get_sdk().initialised);
        assert_eq!(get_sdk().initialise_with_plugin_path(dir.as_str(), !dir.ends_with("debug")).0, Ok(0));
        assert!(get_sdk().initialised);

        //Wait a slight bit to ensure that the test-plugin worker thread has initialised the shared mem
        ::std::thread::sleep(Duration::from_millis(500));

        let mut shmem = match SharedMem::open_linked(std::env::temp_dir().join("wooting-test-plugin.link").as_os_str()) {
            Ok(v) => v,
            Err(e) => {
                println!("Error : {}", e);
                println!("Failed to open SharedMem...");
                assert!(false);
                return;
            }
        };

        get_sdk().set_device_event_cb(connect_cb_wrap);

        //Check the connected cb is called
        {
            {
                let mut shared_state = get_wlock(&mut shmem);
                shared_state.device_connected = true;
            }
            wait_for_connected(5,true);
        }

        //Check that we now have one device
        {
            assert_eq!(get_sdk().get_device_info().0.map(|dev| dev.len()), Ok(1));
        }

        //Check the cb is called with disconnected
        {
            {
                let mut shared_state = get_wlock(&mut shmem);
                shared_state.device_connected = false;
            }
            wait_for_connected(5,false);
        }

        //Check that we now have no devices
        {
            assert_eq!(get_sdk().get_device_info().0.map(|dev| dev.len()), Ok(0));
        }


        let analog_val = 0xF4;
        let f_analog_val = f32::from(analog_val) / 255_f32;
        let analog_key = 5;
        //Connect the device again, set a keycode to a val
        let device_id =
        {
            let mut shared_state = get_wlock(&mut shmem);
            shared_state.analog_values[analog_key] = analog_val;
            shared_state.device_connected = true;
            shared_state.device_id
        };

        wait_for_connected(5,true);

        //Check we get the val with no id specified
        assert_eq!(get_sdk().read_analog(analog_key as u16, 0).0, Ok(f_analog_val));
        //Check we get the val with the device_id we use
        assert_eq!(get_sdk().read_analog(analog_key as u16, device_id).0, Ok(f_analog_val));
        //Check we don't get a val with invalid device id
        assert_eq!(get_sdk().read_analog(analog_key as u16, device_id+1).0, Err(WootingAnalogResult::NoDevices));
        //Check if the next value is 0
        assert_eq!(get_sdk().read_analog((analog_key+1) as u16, device_id).0, Ok(0.0));

        //Check that it does code mapping
        get_sdk().keycode_mode = KeycodeType::ScanCode1;
        assert_eq!(get_sdk().read_analog(hid_to_code(analog_key as u16, &KeycodeType::ScanCode1).unwrap(), device_id).0, Ok(f_analog_val));
        get_sdk().keycode_mode = KeycodeType::HID;

        //This bit may want to be reused to make tests for the ffi
        /*let buffer_len = 5;
        let mut code_buffer: Vec<u16> = vec![0;buffer_len];
        let mut analog_buffer: Vec<f32> = vec![0.0;buffer_len];
        //Check it reads buffer properly with no device id
        assert_eq!(sdk.read_full_buffer(code_buffer.as_mut(), analog_buffer.as_mut(), 0).0, Ok(1));
        assert_eq!(code_buffer[0], analog_key as u16);
        assert_eq!(analog_buffer[0], f_analog_val);

        //Check it reads buffer properly with proper device_id
        assert_eq!(sdk.read_full_buffer(code_buffer.as_mut(), analog_buffer.as_mut(), device_id).0, Ok(1));
        assert_eq!(code_buffer[0], analog_key as u16);
        assert_eq!(analog_buffer[0], f_analog_val);

        //Clean the first part of buffer to make sure it isn't written into
        code_buffer[0] = 0;
        analog_buffer[0] = 0.0;
        //Check it errors on read buffer with invalid device_id
        assert_eq!(sdk.read_full_buffer(code_buffer.as_mut(), analog_buffer.as_mut(), device_id + 1).0, Err(WootingAnalogResult::NoDevices));
        assert_eq!(code_buffer[0], 0);
        assert_eq!(analog_buffer[0], 0.0);

        //Check that it does code mapping
        sdk.keycode_mode = KeycodeType::ScanCode1;
        assert_eq!(sdk.read_full_buffer(code_buffer.as_mut(), analog_buffer.as_mut(), device_id).0, Ok(1));
        assert_eq!(code_buffer[0], hid_to_code( analog_key as u16, &sdk.keycode_mode).unwrap());
        assert_eq!(analog_buffer[0], f_analog_val);
        sdk.keycode_mode = KeycodeType::HID;

        {
            let mut shared_state = get_wlock(&mut shmem);
            shared_state.analog_values[analog_key] = 0;
        }
        ::std::thread::sleep(Duration::from_secs(1));

        assert_eq!(sdk.read_full_buffer(code_buffer.as_mut(), analog_buffer.as_mut(), device_id).0, Ok(0));*/
        let buffer_len = 5;
        let analog_data = get_sdk().read_full_buffer(buffer_len, 0).0.unwrap();
        //Check it reads buffer properly with no device id
        assert_eq!(analog_data.len(), 1);
        assert_eq!(analog_data.iter().next(), Some((&(analog_key as u16), &f_analog_val)));

        let analog_data = get_sdk().read_full_buffer(buffer_len, device_id).0.unwrap();
        //Check it reads buffer properly with proper device_id
        assert_eq!(analog_data.len(), 1);
        assert_eq!(analog_data.iter().next(), Some((&(analog_key as u16), &f_analog_val)));


        //Check it errors on read buffer with invalid device_id
        assert_eq!(get_sdk().read_full_buffer(buffer_len, device_id + 1).0, Err(WootingAnalogResult::NoDevices));


        //Check that it does code mapping
        get_sdk().keycode_mode = KeycodeType::ScanCode1;
        let analog_data = get_sdk().read_full_buffer(buffer_len, device_id).0.unwrap();
        assert_eq!(analog_data.len(), 1);
        assert_eq!(analog_data.iter().next(), Some((&hid_to_code( analog_key as u16, &KeycodeType::ScanCode1).unwrap(), &f_analog_val)));
        get_sdk().keycode_mode = KeycodeType::HID;

        {
            let mut shared_state = get_wlock(&mut shmem);
            shared_state.analog_values[analog_key] = 0;
        }
        ::std::thread::sleep(Duration::from_secs(1));
        let analog_data = get_sdk().read_full_buffer(buffer_len, device_id).0.unwrap();
        assert_eq!(analog_data.len(), 1);
        assert_eq!(analog_data[&(analog_key as u16)], 0.0);

        assert_eq!(get_sdk().read_analog(analog_key as u16, 0).0, Ok(0.0));

        let analog_data = get_sdk().read_full_buffer(buffer_len, device_id).0;
        assert_eq!(analog_data.unwrap().len(), 0);


        get_sdk().clear_device_event_cb();
        {
            let mut shared_state = get_wlock(&mut shmem);
            shared_state.device_connected = false;
        }
        ::std::thread::sleep(Duration::from_secs(1));
        //This shouldn't have updated if the cb is not there
        assert!(*Arc::clone(&got_connected).lock().unwrap());

        get_sdk().unload();
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
        assert_eq!(sdk.initialise_with_plugin_path(dir.as_str(), false).0, Ok(1));
        assert!(sdk.initialised);

        assert_eq!(sdk.read_analog(30, 0).0, Ok(0.56));
        assert_eq!(sdk.read_full_buffer(30, 0).0.unwrap().get(&5), Some(&0.4));
        let device = sdk.get_device_info().0.unwrap().first().unwrap().clone();
        unsafe {
            assert_eq!((*device.0).device_id, 7);
        }
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

    fn cb(event: DeviceEventType, device: DeviceInfoPointer) {}

    fn uninitialised_sdk_functions(sdk: &mut AnalogSDK) {
        assert_eq!(
            sdk.initialised,
            false
        );
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
