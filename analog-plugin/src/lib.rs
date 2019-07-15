#[macro_use]
extern crate log;
#[macro_use]
extern crate analog_sdk;
extern crate hidapi;


use hidapi::{HidApi, HidDevice, HidDeviceInfo};
use std::str;
use analog_sdk::sdk::{Plugin, DeviceID, DeviceInfoPointer, DeviceInfo};
use std::os::raw::{c_ushort, c_float, c_uint, c_int};

const WOOTING_ONE_VID: u16 = 0x03EB;
const WOOTING_ONE_PID: u16  = 0xFF01;
const WOOTING_ONE_ANALOG_USAGE_PAGE: u16  = 0x1338;
const ANALOG_BUFFER_SIZE: usize = 32;


#[derive( Default)]//Debug
pub struct TestPlugin {
    initialised: bool,
    device: Option<HidDevice>,
    buffer: [u8; ANALOG_BUFFER_SIZE]

}

unsafe impl Send for TestPlugin {}
unsafe impl Sync for TestPlugin {}

impl TestPlugin {
    fn refresh_buffer(&mut self) -> bool {
        if !self.initialised {
            return false;
        }

        match &self.device {
            Some(dev) => {
                let res = dev.read_timeout(&mut self.buffer, 0);
                res.is_ok()
            },
            None => false
        }
    }
}

impl Plugin for TestPlugin {
    fn name(&mut self) -> Option<&'static str>  {
        Some("Test Plugin")
    }

    fn initialise(&mut self) -> bool {
        println!("TestPlugin initialised");
        match HidApi::new() {
            Ok(api) => {
                let mut highest_dev: Option<&HidDeviceInfo> = None;
                let mut interface_no: i32 = 0;
                for device in api.devices() {
                    //println!("{:#?}", device);
                    if device.vendor_id == WOOTING_ONE_VID && device.product_id == WOOTING_ONE_PID {
                        if device.interface_number > interface_no {
                            interface_no = device.interface_number;
                            highest_dev = Some(device);
                        }
                    }
                }
                if let Some(device) = highest_dev {
                    match device.open_device(&api){
                        Ok(dev) => {
                            self.device = Some(dev);

                            println!("Found and opened the Wooting One successfully!");
                            self.initialised = true;
                        },
                        Err(e) => {
                            println!("Error: {}", e);
                        }
                    }
                }
                else {
                    println!("Couldn't find a WootingOne");
                }

                println!("Finished with devices");
            },
            Err(e) => {
                println!("Error: {}", e);
                self.initialised = false;
                return self.initialised;
            },
        }

        self.initialised
    }

    fn is_initialised(&mut self) -> bool {
        self.initialised
    }

    fn unload(&mut self) {
        println!("TestPlugin unloaded");
    }

    fn add(&mut self, x: u32, y: u32) -> Option<u32> {
        Some(x + y)
    }

    fn read_analog(&mut self, code: u16) -> Option<f32>{
        if !self.refresh_buffer() {
            return None;
        }
        match self.buffer.iter().position(|&x| x==code as u8) {
            Some(x) if x % 2 == 0 => {
                Some(self.buffer[x as usize +1] as f32 /0xFF as f32)
            }
            _ => Some(0.0)
        }
    }

    fn read_full_buffer(&mut self, max_length: usize, device: DeviceID) -> Option<Vec<(c_ushort, c_float)>> {
        if !self.initialised {
            return None;
        }

        if !self.refresh_buffer() {
            return None;
        }

        Some(self.buffer.chunks_exact(2).filter(|&s| s[1] != 0).map(|s| (s[0] as u16, s[1] as f32/0xFF as f32)).collect())
    }

    fn device_info(&mut self, buffer: &mut [DeviceInfoPointer]) -> c_int {
        buffer[0] = Box::into_raw(Box::new(DeviceInfo { name: b"Plugin Device Yeet\0" as *const u8, val:20 }));

        1
    }
}

declare_plugin!(TestPlugin, TestPlugin::default);
