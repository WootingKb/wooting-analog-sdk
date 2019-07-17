#[macro_use]
extern crate log;
#[macro_use]
extern crate analog_sdk;
extern crate hidapi;


use hidapi::{HidApi, HidDevice, HidDeviceInfo};
use std::str;
use analog_sdk::sdk::{Plugin, DeviceID, DeviceInfoPointer, DeviceInfo, AnalogSDKError, SDKResult, DeviceEventType};
use std::os::raw::{c_ushort, c_float, c_uint, c_int, c_char};
use std::hash::Hasher;
use std::ffi::CString;
use std::borrow::Borrow;
use log::{info, warn, error};
extern crate env_logger;


const WOOTING_ONE_VID: u16 = 0x03EB;
const WOOTING_ONE_PID: u16  = 0xFF01;
//const WOOTING_ONE_ANALOG_USAGE_PAGE: u16  = 0x1338;
const ANALOG_BUFFER_SIZE: usize = 32;


#[derive( Default)]//Debug
pub struct TestPlugin {
    initialised: bool,
    device: Option<HidDevice>,
    buffer: [u8; ANALOG_BUFFER_SIZE],
    device_info: Option<DeviceInfoPointer>,
    device_id: DeviceID,
    disconnected_cb: Option<extern fn(DeviceEventType, DeviceInfoPointer)>

}

unsafe impl Send for TestPlugin {}
unsafe impl Sync for TestPlugin {}

const PLUGIN_NAME: &'static str = "Test Plugin";
impl TestPlugin {

    fn call_cb(&self, eventType: DeviceEventType) {
        if let Some(cb) = self.disconnected_cb {
            let ptr =  match self.device_info.borrow() {
                Some(ptr) => ptr.clone(),
                None => Default::default()
            };
            cb(eventType, ptr);
        }
    }

    fn init_device(&mut self) -> AnalogSDKError {
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
                    debug!("{:#?}", device);
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
                                    let hash = s.finish();
                                    self.device_id = hash;
                                    hash
                                })
                            })).into());
                            info!("Found and opened the Wooting One successfully!");
                            self.call_cb(DeviceEventType::Connected);
                        },
                        Err(e) => {
                            error!("Error opening HID Device: {}", e);
                            return AnalogSDKError::Failure;
                        }
                    }
                }
                else {
                    return AnalogSDKError::NoDevices
                }

                debug!("Finished with devices");
            },
            Err(e) => {
                error!("Error: {}", e);
                return AnalogSDKError::Failure;
            },
        }
        AnalogSDKError::Ok
    }

    fn refresh_buffer(&mut self) -> AnalogSDKError {
        if !self.initialised {
            return AnalogSDKError::UnInitialized;
        }

        if self.device.is_none() {
            self.init_device();
        }

        match &self.device {
            Some(dev) => {
                let res = dev.read_timeout(&mut self.buffer, 0);
                if let Err(e) = res {
                    error!("Failed to read buffer: {}", e);

                    self.call_cb(DeviceEventType::Disconnected);

                    self.device = None;
                    self.device_info.take().map(|d| d.drop());
                    return AnalogSDKError::DeviceDisconnected;
                }
                AnalogSDKError::Ok
            },
            None => AnalogSDKError::DeviceDisconnected
        }
    }


}

impl Plugin for TestPlugin {

    fn name(&mut self) -> SDKResult<&'static str>  {
        Ok(PLUGIN_NAME).into()
    }

    fn initialise(&mut self) -> AnalogSDKError {
        env_logger::init();
        info!("{} initialised", PLUGIN_NAME);
        let ret = self.init_device();
        self.initialised = ret.is_ok();
        AnalogSDKError::Ok
    }



    fn is_initialised(&mut self) -> bool {
        self.initialised
    }


    fn unload(&mut self) {
        info!("{} unloaded", PLUGIN_NAME);
        if self.device_info.is_some() {
            let dev = self.device_info.take();
            dev.unwrap().drop();
        }
    }

    fn set_device_event_cb(&mut self, cb: extern fn(DeviceEventType, DeviceInfoPointer)) -> AnalogSDKError {
        if !self.initialised {
            return AnalogSDKError::UnInitialized;
        }
        debug!("disconnected cb set");
        self.disconnected_cb = Some(cb);
        AnalogSDKError::Ok
    }

    fn clear_device_event_cb(&mut self) -> AnalogSDKError {
        if !self.initialised {
            return AnalogSDKError::UnInitialized;
        }

        debug!("disconnected cb cleared");
        self.disconnected_cb = None;
        AnalogSDKError::Ok
    }

    fn read_analog(&mut self, code: u16, device: DeviceID) -> SDKResult<f32>{
        let ret = self.refresh_buffer();
        if !ret.is_ok() {
            return ret.into();
        }

        if device != 0 && self.device_id != device {
            return AnalogSDKError::NoDevices.into();
        }

        match self.buffer.chunks_exact(3).find(|x| (((x[0] as u16) << 8) | x[1] as u16)==code) {
            Some(x) => {
                Ok(x[2] as f32 /0xFF as f32)
            }
            _ => Ok(0.0)
        }.into()
    }

    fn read_full_buffer(&mut self, max_length: usize, device: DeviceID) -> SDKResult<Vec<(c_ushort, c_float)>> {
        let ret = self.refresh_buffer();
        if !ret.is_ok() {
            return ret.into();
        }

        if !self.refresh_buffer().is_ok() {
            return AnalogSDKError::DeviceDisconnected.into();
        }

        if device != 0 && self.device_id != device {
            return AnalogSDKError::NoDevices.into();
        }

        Ok(self.buffer.chunks_exact(3).take(max_length).filter(|&s| s[2] != 0).map(|s| (((s[0] as u16) << 8) | s[1] as u16, s[2] as f32/0xFF as f32)).collect()).into()
    }

    fn device_info(&mut self, buffer: &mut [DeviceInfoPointer]) -> SDKResult<c_int> {
        if let Some(ptr) = self.device_info.borrow() {
            buffer[0] = ptr.clone();
            Ok(1).into()
        }else{
            Ok(0).into()
        }
    }
}

declare_plugin!(TestPlugin, TestPlugin::default);
