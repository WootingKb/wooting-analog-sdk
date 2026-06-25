use std::{
    collections::{HashMap, hash_map},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use log::{error, info, warn};
use shared_memory::ShmemConf;

use crate::device::{DeviceEventType, DeviceInfo, DeviceSupportLevel, DeviceType};

type Callback = Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send + Sync>;

#[derive(Debug, PartialEq)]
pub struct SharedState {
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

// TODO: when implementing Rust API refactor this to use mpsc::channel()
// TODO: when v2 protocol is implemented transition to KeyCode and AnalogValue
#[derive(Default)]
pub struct VirtualKeyboard {
    device_event_cb: Arc<Mutex<Option<Callback>>>,
    buffer: Arc<Mutex<HashMap<u16, f32>>>,
}

impl VirtualKeyboard {
    pub fn new() -> Self {
        Self::default()
    }

    // Note: can't do <F> where F: FnOnce(DeviceEventType, &DeviceInfo) + Send + 'static
    // because the plugin trait defined Box<dyn Fn(...) and we have to keep the trait dyn compatible
    pub fn with_device_events(&mut self, cb: Arc<Mutex<Option<Callback>>>) {
        self.device_event_cb = cb;
    }

    pub fn attach(&self) {
        let t_device_event_cb2 = Arc::clone(&self.device_event_cb);

        let device: Arc<Mutex<Option<DeviceInfo>>> = Arc::new(Mutex::new(None));
        let device_connected: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
        let thread_running: Arc<AtomicBool> = Arc::new(AtomicBool::new(true));

        let t_buffer = Arc::clone(&self.buffer);
        let t_device = Arc::clone(&device);
        let t_device_connected = Arc::clone(&device_connected);
        let t_thread_running = Arc::clone(&thread_running);

        thread::spawn(move || {
            let link_path = std::env::temp_dir().join("wooting-test-plugin.link");
            let my_shmem = {
                match ShmemConf::new()
                    .size(4096)
                    .flink(link_path.as_os_str())
                    .open()
                {
                    Ok(v) => v,
                    Err(e) => {
                        if link_path.exists() {
                            warn!("Error : {}", e);
                            warn!(
                                "Attempted to open exist SharedMemFailed... Falling back to creation"
                            );
                            if let Err(e) = std::fs::remove_file(&link_path) {
                                error!("Could not delete old link file: {}", e);
                            }
                        }
                        match ShmemConf::new()
                            .size(4096)
                            .flink(link_path.as_os_str())
                            .create()
                        {
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

            info!("{:?}", my_shmem.get_flink_path());

            {
                let shared_state = unsafe { &mut *(my_shmem.as_ptr() as *mut SharedState) };
                shared_state.vendor_id = 0x03eb;
                shared_state.product_id = 0xFFFF;
                shared_state.device_type = DeviceType::Keyboard;
                shared_state.device_connected = false;
                shared_state.dirty_device_info = false;
                let src = b"Wooting\x00";
                shared_state.manufacturer_name[0..src.len()].copy_from_slice(src);
                let src = b"Test Device\x00";
                shared_state.device_name[0..src.len()].copy_from_slice(src);
                shared_state.analog_values = [0; 0xFF];
            }

            let mut vals = vec![0; 0xFF];
            loop {
                if !t_thread_running.load(Ordering::SeqCst) {
                    break;
                }

                {
                    let state = unsafe { &mut *(my_shmem.as_ptr() as *mut SharedState) };

                    if state.dirty_device_info || t_device.lock().unwrap().is_none() {
                        state.dirty_device_info = false;
                        let dev = DeviceInfo::new_with_id(
                            state.vendor_id,
                            state.product_id,
                            from_ut8f_to_null(
                                &state.manufacturer_name[..],
                                state.manufacturer_name.len(),
                            )
                            .to_string(),
                            from_ut8f_to_null(&state.device_name[..], state.device_name.len())
                                .to_string(),
                            1,
                            state.device_type.clone(),
                            DeviceSupportLevel::Limited,
                        );
                        t_device.lock().unwrap().replace(dev);
                    }

                    if *t_device_connected.lock().unwrap() != state.device_connected {
                        *t_device_connected.lock().unwrap() = state.device_connected;
                        if let Some(device) = t_device.lock().unwrap().as_ref() {
                            t_device_event_cb2.lock().unwrap().as_ref().map(|cb| {
                                cb(
                                    if state.device_connected {
                                        DeviceEventType::Connected
                                    } else {
                                        DeviceEventType::Disconnected
                                    },
                                    device,
                                );
                                0
                            });
                        }
                    }

                    if !state.device_connected {
                        thread::sleep(Duration::from_millis(500));
                        continue;
                    }

                    vals.copy_from_slice(&state.analog_values[..]);
                }

                {
                    let mut m = t_buffer.lock().unwrap();
                    m.clear();
                    m.extend(vals.iter().enumerate().filter_map(|(i, &val)| {
                        if val > 0 {
                            Some((i as u16, f32::from(val) / 255_f32))
                        } else {
                            None
                        }
                    }));
                }

                thread::sleep(Duration::from_millis(10));
            }
        });
    }

    pub fn iter_over<F, R>(&self, f: F) -> R
    where
        F: FnOnce(hash_map::Iter<'_, u16, f32>) -> R,
    {
        let guard = self.buffer.lock().unwrap();
        f(guard.iter())
    }
}

fn from_ut8f_to_null(bytes: &[u8], max_len: usize) -> &str {
    use std::str::from_utf8_unchecked;
    for i in 0..max_len {
        if bytes[i] == 0 {
            return unsafe { from_utf8_unchecked(&bytes[0..i]) };
        }
    }
    panic!("Couldnt find null terminator.");
}
