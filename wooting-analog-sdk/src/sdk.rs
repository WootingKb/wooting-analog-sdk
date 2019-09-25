use crate::cplugin::*;
use crate::errors::*;
use crate::keycode::*;
use libloading::{Library, Symbol};
use log::{error, info, warn};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs;
use std::os::raw::{c_float, c_int, c_ushort};
use std::path::{Path, PathBuf};
use wooting_analog_common::*;
use wooting_analog_plugin_dev::*;

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

//const ENV_PLUGIN_DIR_KEY: &str = "WOOTING_ANALOG_SDK_PLUGINS_PATH";



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

    pub fn initialise(&mut self) -> WootingAnalogResult {
        self.initialise_with_plugin_path(DEFAULT_PLUGIN_DIR)
    }

    pub fn initialise_with_plugin_path(&mut self, plugin_dir: &str) -> WootingAnalogResult {
        if self.initialised {
            self.unload();
        }

        let plugin_dir = PathBuf::from(plugin_dir);
        if !plugin_dir.is_dir() {
            error!("The plugin directory '{}' does not exist! Make sure you have it created and have plugins in there", DEFAULT_PLUGIN_DIR);
            return WootingAnalogResult::NoPlugins;
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
        for dir in plugin_dir.read_dir().expect("Could not read dir") {
            match dir {
                Ok(dir) => {
                    if dir.path().is_file() {
                        continue;
                    }

                    match self.load_plugins(&dir.path()) {
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
                },
                Err(e) => {
                    error!("Error reading directory: {}", e);
                }
            }
        }

        let mut plugins_initialised = 0;
        let mut has_devices = false;
        for p in self.plugins.iter_mut() {
            let ret = p.initialise();
            if ret.is_ok_or_no_device()  {
                plugins_initialised += 1;
                if ret.is_ok() {
                    has_devices = true;
                }
            }
        }
        info!("{} plugins successfully initialised", plugins_initialised);

        self.initialised = plugins_initialised > 0;
        if !self.initialised {
            WootingAnalogResult::NoPlugins
        } else if !has_devices {
            WootingAnalogResult::NoDevices
        } else {
            WootingAnalogResult::Ok
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
    ) -> WootingAnalogResult {
        if !self.initialised {
            return WootingAnalogResult::UnInitialized;
        }

        let mut result = WootingAnalogResult::Ok;
        for p in self.plugins.iter_mut() {
            let ret = p.set_device_event_cb(cb);
            if ret != WootingAnalogResult::Ok {
                result = ret;
            }
        }
        result
    }

    pub fn clear_device_event_cb(&mut self) -> WootingAnalogResult {
        if !self.initialised {
            return WootingAnalogResult::UnInitialized;
        }

        let mut result = WootingAnalogResult::Ok;
        for p in self.plugins.iter_mut() {
            let ret = p.clear_device_event_cb();
            if ret != WootingAnalogResult::Ok {
                result = ret;
            }
        }
        result
    }

    pub fn get_device_info(&mut self, buffer: &mut [DeviceInfoPointer]) -> SDKResult<c_int> {
        if !self.initialised {
            return WootingAnalogResult::UnInitialized.into();
        }
        let mut count: usize = 0;
        for p in self.plugins.iter_mut() {
            //Give a reference to the buffer at the point where there is free space
            let num = p.device_info(&mut buffer[count..]).0.unwrap_or(0) as usize;
            if num > 0 {
                count += num
            }
        }
        (count as c_int).into()
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

            return value.into();
        } else {
            return WootingAnalogResult::NoMapping.into();
        }
    }

    pub fn read_full_buffer(
        &mut self,
        code_buffer: &mut [c_ushort],
        analog_buffer: &mut [c_float],
        device_id: DeviceID,
    ) -> SDKResult<c_int> {
        if !self.initialised {
            return WootingAnalogResult::UnInitialized.into();
        }

        let mut analog_data: HashMap<c_ushort, c_float> = HashMap::with_capacity(code_buffer.len());

        let mut err = WootingAnalogResult::Ok;
        let mut any_success = false;
        //Read from all and add up
        for p in self.plugins.iter_mut() {
            let plugin_data = p.read_full_buffer(code_buffer.len(), device_id).into();
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
        self.initialised = false;
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

        let mut sdk = AnalogSDK::new();
        assert_eq!(sdk.initialise_with_plugin_path(dir), WootingAnalogResult::NoPlugins);
        assert!(!sdk.initialised);
        ::std::fs::remove_dir(dir);
    }

    #[test]
    fn initialise_no_dir() {
        shared_init();

        let dir = "./test_n";
        ::std::fs::remove_dir(dir);

        let mut sdk = AnalogSDK::new();
        assert_eq!(sdk.initialise_with_plugin_path(dir), WootingAnalogResult::NoPlugins);
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
        info!("Got cb {:?}", event);

        *Arc::clone(&got_connected).lock().unwrap() = event == DeviceEventType::Connected;
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


        let dir = "../test-plugins";
        let mut sdk = AnalogSDK::new();
        assert_eq!(sdk.initialise_with_plugin_path(dir), WootingAnalogResult::NoDevices);
        assert!(sdk.initialised);


        let mut shmem = match SharedMem::open_linked(std::env::temp_dir().join("wooting-test-plugin.link").as_os_str()) {
            Ok(v) => v,
            Err(e) => {
                println!("Error : {}", e);
                println!("Failed to open SharedMem...");
                assert!(false);
                return;
            }
        };

        sdk.set_device_event_cb(connect_cb);

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
            let mut device_infos: Vec<DeviceInfoPointer> = vec![Default::default(), Default::default()];
            assert_eq!(sdk.get_device_info(device_infos.as_mut()).0, Ok(1));
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
            let mut device_infos: Vec<DeviceInfoPointer> = vec![Default::default(), Default::default()];
            assert_eq!(sdk.get_device_info(device_infos.as_mut()).0, Ok(0));
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
        assert_eq!(sdk.read_analog(analog_key as u16, 0).0, Ok(f_analog_val));
        //Check we get the val with the device_id we use
        assert_eq!(sdk.read_analog(analog_key as u16, device_id).0, Ok(f_analog_val));
        //Check we don't get a val with invalid device id
        assert_eq!(sdk.read_analog(analog_key as u16, device_id+1).0, Err(WootingAnalogResult::NoDevices));
        //Check if the next value is 0
        assert_eq!(sdk.read_analog((analog_key+1) as u16, device_id).0, Ok(0.0));

        //Check that it does code mapping
        sdk.keycode_mode = KeycodeType::ScanCode1;
        assert_eq!(sdk.read_analog( hid_to_code( analog_key as u16, &sdk.keycode_mode).unwrap(), device_id).0, Ok(f_analog_val));
        sdk.keycode_mode = KeycodeType::HID;


        let buffer_len = 5;
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

        assert_eq!(sdk.read_full_buffer(code_buffer.as_mut(), analog_buffer.as_mut(), device_id).0, Ok(0));
        assert_eq!(sdk.read_analog(analog_key as u16, 0).0, Ok(0.0));

        sdk.clear_device_event_cb();
        {
            let mut shared_state = get_wlock(&mut shmem);
            shared_state.device_connected = false;
        }
        ::std::thread::sleep(Duration::from_secs(1));
        //This shouldn't have updated if the cb is not there
        assert!(*Arc::clone(&got_connected).lock().unwrap());
    }

    #[test]
    fn unitialised_sdk_functions_new() {
        shared_init();

        let mut sdk = AnalogSDK::new();
        uninitialised_sdk_functions(&mut sdk);
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

    extern "C" fn cb(event: DeviceEventType, device: DeviceInfoPointer) {}

    fn uninitialised_sdk_functions(sdk: &mut AnalogSDK) {
        assert_eq!(
            sdk.set_device_event_cb(cb),
            WootingAnalogResult::UnInitialized
        );
        assert_eq!(
            sdk.clear_device_event_cb(),
            WootingAnalogResult::UnInitialized
        );
        assert_eq!(
            sdk.read_analog(0, 0).0,
            Err(WootingAnalogResult::UnInitialized)
        );
        let mut buf = vec![];
        assert_eq!(
            sdk.get_device_info(buf.as_mut()).0,
            Err(WootingAnalogResult::UnInitialized)
        );

        let mut buf = vec![];
        let mut buf2 = vec![];
        assert_eq!(
            sdk.read_full_buffer(buf.as_mut(), buf2.as_mut(), 0).0,
            Err(WootingAnalogResult::UnInitialized)
        );
    }
}
