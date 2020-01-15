#[macro_use]
extern crate wooting_analog_plugin_dev;
#[macro_use]
extern crate log;
extern crate env_logger;
use wooting_analog_plugin_dev::*;
use wooting_analog_plugin_dev::wooting_analog_common::*;
use std::collections::HashMap;
use shared_memory::*;
use std::os::raw::{c_char};
use log::{error, info};
use std::thread;
use std::sync::{Arc, Mutex};
use std::path::Path;
use std::string::ToString;

struct WootingAnalogTestPlugin {
    //shmem: SharedMem,
    device_connected: Arc<Mutex<bool>>,
    device_event_cb: Arc<Mutex<Option<Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send>>>>,
    device: Arc<Mutex<Option<DeviceInfo>>>,
    buffer: Arc<Mutex<HashMap<u16, f32>>>,
    device_id: Arc<Mutex<DeviceID>>,
    pressed_keys: Vec<u16>
}

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
    pub device_id: DeviceID,

    pub device_type: DeviceType,

    pub device_connected: bool,
    pub dirty_device_info: bool,

    pub analog_values: [u8; 0xFF]
}

unsafe impl SharedMemCast for SharedState {}

impl WootingAnalogTestPlugin{
    fn new() -> Self {
        use env_logger::Env;
        env_logger::from_env(Env::default().default_filter_or("trace")).try_init();


        let device: Arc<Mutex<Option<DeviceInfo>>> = Arc::new(Mutex::new(None));
        let buffer: Arc<Mutex<HashMap<u16, f32>>> = Arc::new(Mutex::new(HashMap::new()));
        let device_id: Arc<Mutex<DeviceID>> = Arc::new(Mutex::new(1));
        let device_event_cb: Arc<Mutex<Option<Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send>>>> = Arc::new(Mutex::new(None));
        let device_connected: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));

        let t_buffer = Arc::clone(&buffer);
        let t_device = Arc::clone(&device);
        let t_device_id = Arc::clone(&device_id);
        let t_device_event_cb = Arc::clone(&device_event_cb);
        let t_device_connected = Arc::clone(&device_connected);

        let worker_thread = thread::spawn(move || {
            let link_path = std::env::temp_dir().join("wooting-test-plugin.link");

            let mut my_shmem = {
                match SharedMem::open_linked(link_path.as_os_str()) {
                    Ok(v) => v,
                    Err(e) => {
                        if link_path.exists() {
                            warn!("Error : {}", e);
                            warn!("Attempted to open exist SharedMemFailed... Falling back to creation");
                            if let Err(e) = std::fs::remove_file(&link_path) {
                                error!("Could not delete old link file: {}", e);
                            }
                        }
                        match SharedMem::create_linked(link_path.as_os_str(), LockType::Mutex, 4096) {
                            Ok(m) => m,
                            Err(e) => {
                                error!("Test Plugin Error : {}", e);
                                error!("Test Plugin Failed to create SharedMem closing!");
                                //return;
                                return;
                            }
                        }
                    }
                }
            };

            info!("{:?}", my_shmem.get_link_path());

            {
                let mut shared_state = match my_shmem.wlock::<SharedState>(0) {
                    Ok(v) => v,
                    Err(_) => {
                        error!("Test plugin Failed to acquire write lock! Stopping...");
                        return;
                    }
                };
                shared_state.vendor_id = 0x03eb;
                shared_state.product_id = 0xFFFF;
                shared_state.device_id = 1;
                shared_state.device_type = DeviceType::Keyboard;
                shared_state.device_connected = false;
                shared_state.dirty_device_info = false;
                let src = b"Wooting\x00";
                shared_state.manufacturer_name[0..src.len()].copy_from_slice(src);
                let src = b"Test Device\x00";
                shared_state.device_name[0..src.len()].copy_from_slice(src);
            }

            let mut vals = vec![0; 0xFF];
            loop {
                {
                    let mut state = match my_shmem.wlock::<SharedState>(0) {
                        Ok(v) => v,
                        Err(_) => {
                            warn!("failed to get lock");
                            continue;
                        },
                    };

                    if state.dirty_device_info || t_device.lock().unwrap().is_none() {
                        state.dirty_device_info = false;
                        let dev = DeviceInfo::new_with_id(
                            state.vendor_id,
                            state.product_id,
                            from_ut8f_to_null(&state.manufacturer_name[..], state.manufacturer_name.len()).to_string(),
                            from_ut8f_to_null(&state.device_name[..], state.device_name.len()).to_string(),
                            state.device_id,
                            state.device_type.clone()
                            
                        );
                        *t_device_id.lock().unwrap() = state.device_id;
                        t_device.lock().unwrap().replace(dev);


                    }
                    if *t_device_connected.lock().unwrap() != state.device_connected {
                        *t_device_connected.lock().unwrap() = state.device_connected;
                        if let Some(device) = t_device.lock().unwrap().as_ref() {
                            t_device_event_cb.lock().unwrap().as_ref().and_then(|cb| {cb(if state.device_connected {DeviceEventType::Connected } else {DeviceEventType::Disconnected}, device);Some(0)});
                        }
                    }

                    if !state.device_connected {
                        //make sure we drop the state so we're not holding the lock while the thread is sleeping
                        drop(state);
                        thread::sleep_ms(500);
                        continue;
                    }

                    vals.copy_from_slice(&state.analog_values[..]);
                }

                let analog: HashMap<u16, f32> = vals.iter().enumerate().filter_map(|(i, &val)| {
                    if val > 0 {
                        Some((i as u16, f32::from(val) / 255_f32))
                    }else {
                        None
                    }
                } ).collect();
                {
                    let mut m = t_buffer.lock().unwrap();
                    m.clear();
                    m.extend(analog);
                }
                //t_buffer.lock().unwrap().
                thread::sleep_ms(10);
            }
        });

        WootingAnalogTestPlugin {
            device_connected,
            device_event_cb,
            device,
            buffer,
            device_id,
            pressed_keys: vec!()
        }
    }
}

fn from_ut8f_to_null(bytes: &[u8], max_len: usize) -> &str {
    use std::str::from_utf8_unchecked;
    for i in 0..max_len {
        if bytes[i] == 0 {
            return unsafe {from_utf8_unchecked(&bytes[0..i])};
        }
    }
    panic!("Couldnt find null terminator.");
}

impl Plugin for WootingAnalogTestPlugin {
    fn name(&mut self) -> SDKResult<&'static str> {
        Ok("Wooting Analog Test Plugin").into()
    }

    fn initialise(&mut self, cb: Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send>) -> SDKResult<u32> {
        info!("init");
        let ret = if *self.device_connected.lock().unwrap() { Ok(1) } else { Ok(0) }.into();
        self.device_event_cb.lock().unwrap().replace(cb);
        ret
    }

    fn is_initialised(&mut self) -> bool {
        true
    }

    fn device_info(&mut self) -> SDKResult<Vec<DeviceInfo>> {
        debug!("asked for devices {:?}", *self.device_connected.lock().unwrap());

        let mut devices = vec![];
        if *self.device_connected.lock().unwrap() {
            let dev_ptr = self.device.lock().unwrap().clone().unwrap();
            devices.push(dev_ptr);
        }
        debug!("Finished with devices");

        Ok(devices).into()
    }

    fn read_analog(&mut self, code: u16, device: u64) -> SDKResult<f32> {
        if !*self.device_connected.lock().unwrap() {
            return WootingAnalogResult::NoDevices.into();
        }

        if device == 0 || device == *self.device_id.lock().unwrap() {
            Ok(self.buffer.lock().unwrap().get(&code).cloned().or(Some(0.0)).unwrap()).into()
        }
        else {
            WootingAnalogResult::NoDevices.into()
        }
    }

    fn read_full_buffer(&mut self, max_length: usize, device: u64) -> SDKResult<HashMap<u16, f32>> {
        if !*self.device_connected.lock().unwrap() {
            return WootingAnalogResult::NoDevices.into();
        }

        if device == 0 || device == *self.device_id.lock().unwrap() {
            let mut buffer = self.buffer.lock().unwrap().clone();
            //Collect the new pressed keys
            let new_pressed_keys: Vec<u16> = buffer.keys().map(|x| *x).collect();

            //Put the old pressed keys into the buffer
            for key in self.pressed_keys.drain(..) {
                if !buffer.contains_key(&key) {
                    buffer.insert(key, 0.0);
                }
            }

            //Store the newPressedKeys for the next call
            self.pressed_keys = new_pressed_keys;

            Ok(buffer).into()
        }
        else {
            WootingAnalogResult::NoDevices.into()
        }
    }
}


declare_plugin!(WootingAnalogTestPlugin, WootingAnalogTestPlugin::new);

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
