#[macro_use]
extern crate log;
#[macro_use]
extern crate analog_sdk;
extern crate hidapi;


use hidapi::{HidApi, HidDevice, HidDeviceInfo};
use std::str;
use analog_sdk::sdk::{Plugin, DeviceID, DeviceInfoPointer, DeviceInfo, AnalogSDKError};
use std::os::raw::{c_ushort, c_float, c_uint, c_int, c_char};
use std::hash::Hasher;
use std::ffi::CString;
use std::borrow::Borrow;

const WOOTING_ONE_VID: u16 = 0x03EB;
const WOOTING_ONE_PID: u16  = 0xFF01;
const WOOTING_ONE_ANALOG_USAGE_PAGE: u16  = 0x1338;
const ANALOG_BUFFER_SIZE: usize = 32;


#[derive( Default)]//Debug
pub struct TestPlugin {
    initialised: bool,
    device: Option<HidDevice>,
    buffer: [u8; ANALOG_BUFFER_SIZE],
    device_info: Option<DeviceInfoPointer>,
    disconnected_cb: Option<extern fn(DeviceInfoPointer)>

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
                if let Err(e) = res {
                    println!("{}", e);
                    if let Some(cb) = self.disconnected_cb {
                        if let Some(ptr) = self.device_info.borrow() {
                            cb(ptr.clone());
                        }
                    }
                    self.device = None;
                    self.initialised = false;
                    self.device_info.take().map(|d| d.drop());
                    return false;
                }
                return true;
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
                    if device.vendor_id == WOOTING_ONE_VID && device.product_id == WOOTING_ONE_PID {
                        if device.interface_number > interface_no {
                            interface_no = device.interface_number;
                            highest_dev = Some(device);
                        }
                    }
                }
                if let Some(device) = highest_dev {
                    println!("{:#?}", device);
                    match device.open_device(&api){
                        Ok(dev) => {
                            use std::ptr;
                            use std::collections::hash_map::DefaultHasher;
                            self.device = Some(dev);//name: b"Plugin Device Yeet\0" as *const u8
                            self.device_info = Some(Box::into_raw(Box::new(DeviceInfo {
                                vendor_id: device.vendor_id,
                                product_id: device.product_id,
                                manufacturer_name: device.manufacturer_string.as_ref().cloned().map_or(ptr::null(), |str| CString::new(str).unwrap().into_raw()),//b"\0" as *const u8
                                device_name: device.product_string.as_ref().cloned().map_or(ptr::null(), |str| CString::new(str).unwrap().into_raw()),
                                device_id: device.serial_number.as_ref().cloned().map_or(0, |str| {
                                    let mut s = DefaultHasher::new();
                                    s.write_u16(device.vendor_id);
                                    s.write_u16(device.product_id);
                                    s.write(str.as_bytes());
                                    s.finish()})
                            })).into());
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
        if self.device_info.is_some() {
            let dev = self.device_info.take();
            dev.unwrap().drop();
        }
    }

    fn set_disconnected_cb(&mut self, cb: extern fn(DeviceInfoPointer)) -> AnalogSDKError {
        if !self.initialised {
            return AnalogSDKError::UNINITIALIZED;
        }
        println!("disconnected cb set");
        self.disconnected_cb = Some(cb);
        AnalogSDKError::OK
    }

    fn clear_disconnected_cb(&mut self) -> AnalogSDKError {
        if !self.initialised {
            return AnalogSDKError::UNINITIALIZED;
        }

        println!("disconnected cb cleared");
        self.disconnected_cb = None;
        AnalogSDKError::OK
    }

    fn read_analog(&mut self, code: u16) -> Option<f32>{
        if !self.refresh_buffer() {
            return None;
        }
        match self.buffer.chunks_exact(3).find(|x| (((x[0] as u16) << 8) | x[1] as u16)==code) {
            Some(x) => {
                Some(x[2] as f32 /0xFF as f32)
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

        Some(self.buffer.chunks_exact(3).filter(|&s| s[2] != 0).map(|s| (((s[0] as u16) << 8) | s[1] as u16, s[2] as f32/0xFF as f32)).collect())
    }

    fn device_info(&mut self, buffer: &mut [DeviceInfoPointer]) -> c_int {
        if let Some(ptr) = self.device_info.borrow() {
            buffer[0] = ptr.clone();
            1
        }else{
            0
        }
    }
}

declare_plugin!(TestPlugin, TestPlugin::default);
