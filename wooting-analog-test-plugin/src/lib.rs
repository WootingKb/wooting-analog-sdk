#[macro_use]
extern crate wooting_analog_plugin_dev;
#[macro_use]
extern crate log;
use wooting_analog_plugin_dev::*;
use wooting_analog_common::*;
use std::collections::HashMap;
use shared_memory::*;
use std::os::raw::{c_char};
use log::{error, info};


struct WootingAnalogTestPlugin {
    shmem: SharedMem,
    device_event_cb: Option<extern "C" fn(DeviceEventType, DeviceInfoPointer)>,
    device: Option<DeviceInfoPointer>
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

    pub device_connected: bool,
    pub dirty_device_info: bool,

    pub analog_values: [u8; 0xFF]
}

unsafe impl SharedMemCast for SharedState {}

impl WootingAnalogTestPlugin{
    fn new() -> Self {
        let mut my_shmem = match SharedMem::create_linked("wooting-test-plugin.link", LockType::Mutex, 4096) {
            Ok(m) => m,
            Err(e) => {
                println!("Error : {}", e);
                println!("Failed to create SharedMem !");
                //return;
                panic!();
            }
        };
        println!("{:?}", my_shmem.get_link_path());

        {
            let mut shared_state = match my_shmem.wlock::<SharedState>(0) {
                Ok(v) => v,
                Err(_) => panic!("Failed to acquire write lock !"),
            };
            shared_state.vendor_id = 0x03eb;
            shared_state.product_id = 0xFFFF;
            shared_state.device_id = 1;
            shared_state.device_connected = true;
            shared_state.dirty_device_info = false;
            let src = b"Wooting\x00";
            shared_state.manufacturer_name[0..src.len()].copy_from_slice(src);
            let src = b"Test Device\x00";
            shared_state.device_name[0..src.len()].copy_from_slice(src);
        }
        WootingAnalogTestPlugin {
            shmem: my_shmem,
            device_event_cb: None,
            device: None
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

    fn initialise(&mut self) -> WootingAnalogResult {
        WootingAnalogResult::Ok
    }

    fn is_initialised(&mut self) -> bool {
        true
    }

    fn set_device_event_cb(
        &mut self,
        cb: extern "C" fn(DeviceEventType, DeviceInfoPointer),
    ) -> WootingAnalogResult {
        //if !self.initialised {
        //    return WootingAnalogResult::UnInitialized;
        //}
        debug!("disconnected cb set");
        self.device_event_cb = Some(cb);
        WootingAnalogResult::Ok
    }

    fn clear_device_event_cb(&mut self) -> WootingAnalogResult {
        //if !self.initialised {
        //    return WootingAnalogResult::UnInitialized;
        //}

        debug!("disconnected cb cleared");
        self.device_event_cb = None;
        WootingAnalogResult::Ok
    }

    fn device_info(&mut self, buffer: &mut [DeviceInfoPointer]) -> SDKResult<i32> {
        //if !self.initialised {
        //    return WootingAnalogResult::UnInitialized.into();
        //}
        let shared_state = self.shmem.rlock::<SharedState>(0);


        let dev_ptr: DeviceInfoPointer = match shared_state {
            Ok(state) => {
                if state.dirty_device_info || self.device.is_none() {
                    let dev = DeviceInfo::new_with_id(
                        state.vendor_id,
                        state.product_id,
                        from_ut8f_to_null(&state.manufacturer_name[..], state.manufacturer_name.len()),
                        from_ut8f_to_null(&state.device_name[..], state.device_name.len()),
                        state.device_id,
                    ).to_ptr();
                    self.device = Some(dev);
                }

                self.device.as_ref().unwrap()
            },
            Err(_e) => {
                match &self.device {
                    Some(ptr) => ptr,
                    None => {
                        return WootingAnalogResult::Failure.into();
                    }
                }
            }
        }.clone();
        buffer[0] = dev_ptr;


        1.into()
    }

    fn read_analog(&mut self, code: u16, device: u64) -> SDKResult<f32> {
        let shared_state = self.shmem.rlock::<SharedState>(0);

        if let Ok(state) = shared_state {
            if device == 0 || device == state.device_id {
                (f32::from(state.analog_values[code as usize]) / 255_f32).into()
            }else {
                WootingAnalogResult::NoDevices.into()
            }
        }
        else {
            WootingAnalogResult::Failure.into()
        }
    }

    fn read_full_buffer(&mut self, max_length: usize, device: u64) -> SDKResult<HashMap<u16, f32>> {
        let mut vals = vec![0; 0xFF];
        let shared_state = self.shmem.rlock::<SharedState>(0);
        if let Ok(state) = shared_state {
            if device == 0 || device == state.device_id {
                vals.copy_from_slice(&state.analog_values[..]);
                drop(state);

                let analog: HashMap<u16, f32> = vals.iter().enumerate().filter_map(|(i, &val)| {
                    if val > 0 {
                        Some((i as u16, f32::from(val) / 255_f32))
                    }else {
                        None
                    }
                } ).collect();
                Ok(analog).into()
            }else {
                WootingAnalogResult::NoDevices.into()
            }
        }
        else {
            WootingAnalogResult::Failure.into()
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
