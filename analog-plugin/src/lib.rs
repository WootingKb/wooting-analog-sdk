#[macro_use]
extern crate log;
#[macro_use]
extern crate analog_sdk;
extern crate hidapi;
#[macro_use]
extern crate objekt;

use hidapi::{HidApi, HidDevice, HidDeviceInfo};
use std::str;
use analog_sdk::sdk::{Plugin, DeviceID, DeviceInfoPointer, DeviceInfo, AnalogSDKError, SDKResult, DeviceEventType};
use std::os::raw::{c_ushort, c_float, c_int};
use std::hash::Hasher;
use std::ffi::CString;
use std::collections::HashMap;
use log::{info, error};
use std::any::Any;
extern crate env_logger;


const WOOTING_ONE_VID: u16 = 0x03EB;
const WOOTING_ONE_PID: u16  = 0xFF01;
//const WOOTING_ONE_ANALOG_USAGE_PAGE: u16  = 0x1338;
const ANALOG_BUFFER_SIZE: usize = 32;

struct DeviceHardwareID {
    vid: u16, pid: u16, usage_page: u16, interface_n: i32
}

trait DeviceImplementation: objekt::Clone {
    fn device_hardware_id(&self) -> DeviceHardwareID;

    fn matches(&self, device: &HidDeviceInfo) -> bool {
        let hid = self.device_hardware_id();
        //Check if the pid & hid match
        device.product_id.eq(&hid.pid)
        && device.vendor_id.eq(&hid.vid)
        &&  if device.usage_page != 0 && hid.usage_page != 0 //check if the usage_page is valid to check
                {device.usage_page.eq(&hid.usage_page)} //if it is, check if they are the same
            else //otherwise, check if the defined interface number is correct
                {(hid.interface_n.eq(&device.interface_number))}
    }

    fn refresh_buffer(&self, device: &HidDevice) -> SDKResult<HashMap<c_ushort, c_float>> {
        let mut buffer: [u8; ANALOG_BUFFER_SIZE] = Default::default();
        let res = device.read_timeout(&mut buffer, 0);
        if let Err(e) = res {
            error!("Failed to read buffer: {}", e);

            return AnalogSDKError::DeviceDisconnected.into();
        }
        let ret: HashMap<c_ushort, c_float> = buffer.chunks_exact(3).filter(|&s| s[2] != 0).map(|s| (((s[0] as u16) << 8) | s[1] as u16, s[2] as f32/0xFF as f32)).collect();
        Ok(ret).into()
    }

    fn get_device_id(&self, device_info: &HidDeviceInfo) -> DeviceID {
        use std::collections::hash_map::DefaultHasher;
        device_info.serial_number.as_ref().cloned().map_or(0, |str| {
            let mut s = DefaultHasher::new();
            s.write_u16(device_info.vendor_id);
            s.write_u16(device_info.product_id);
            s.write(str.as_bytes());
            let hash = s.finish();
            hash
        })
    }
}

clone_trait_object!(DeviceImplementation);


#[derive(Clone)]
struct WootingOne();

impl DeviceImplementation for WootingOne {
    fn device_hardware_id(&self) -> DeviceHardwareID {
        DeviceHardwareID {
            vid: 0x03EB,
            pid: 0xFF01,
            usage_page: 0x1338,
            interface_n: 6
        }
    }
}


struct Device {
    device: HidDevice,
    pub device_info: DeviceInfoPointer,
    buffer: [u8; ANALOG_BUFFER_SIZE],
    device_impl: Box<dyn DeviceImplementation>
}

impl Device {
    fn new(device_info: &HidDeviceInfo, device: HidDevice, device_impl: Box<DeviceImplementation>) -> (DeviceID, Self) {
        use std::ptr;
        let id_hash = device_impl.get_device_id(device_info);
        (id_hash, Device {
            device,//name: b"Plugin Device Yeet\0" as *const u8
            device_info: Box::into_raw(Box::new(DeviceInfo {
                vendor_id: device_info.vendor_id,
                product_id: device_info.product_id,
                manufacturer_name: device_info.manufacturer_string.as_ref().cloned().map_or(ptr::null(), |str| CString::new(str).unwrap().into_raw()),//b"\0" as *const u8
                device_name: device_info.product_string.as_ref().cloned().map_or(ptr::null(), |str| CString::new(str).unwrap().into_raw()),
                device_id: id_hash
            })).into(),
            device_impl,
            buffer: Default::default()
        })
    }

    fn read_analog(&self, code: u16) -> SDKResult<c_float> {
        match self.device_impl.refresh_buffer(&self.device).into() {
            Ok(data) => (*data.get(&code).unwrap_or(&0.0)).into(),
            Err(e) => Err(e).into()
        }
    }

    fn read_full_buffer(&self, max_length: usize) -> SDKResult<HashMap<c_ushort, c_float>> {
        self.device_impl.refresh_buffer(&self.device)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        //TODO: drop DeviceInfoPointer
    }
}


#[derive( Default)]//Debug
pub struct TestPlugin {
    initialised: bool,
    device_event_cb: Option<extern fn(DeviceEventType, DeviceInfoPointer)>,
    devices: HashMap<DeviceID, Device>,
    deviceImpls: Vec<Box<dyn DeviceImplementation>>

}

unsafe impl Send for TestPlugin {}
unsafe impl Sync for TestPlugin {}

const PLUGIN_NAME: &'static str = "Test Plugin";
impl TestPlugin {

    fn new() -> Self {
        TestPlugin {
            initialised: false,
            device_event_cb: None,
            devices: Default::default(),
            deviceImpls: vec![Box::new(WootingOne())]
        }
    }

    fn call_cb(&self, device: &Device, eventType: DeviceEventType) {
        if let Some(cb) = self.device_event_cb {
            cb(eventType, device.device_info.clone());
        }
    }

    fn init_device(&mut self) -> AnalogSDKError {
        match HidApi::new() {
            Ok(api) => {
                for device_info in api.devices() {
                    for deviceImpl in self.deviceImpls.iter() {
                        debug!("{:#?}", device_info);
                        if deviceImpl.matches(device_info) && !self.devices.contains_key(&deviceImpl.get_device_id(device_info)) {
                            match device_info.open_device(&api){
                                Ok(dev) => {
                                    let (id, device) = Device::new(device_info, dev, deviceImpl.clone());
                                    self.devices.insert(id, device);
                                    info!("Found and opened the {:?} successfully!", device_info.product_string);
                                    self.handle_device_event(self.devices.get(&id).unwrap(),DeviceEventType::Connected);
                                },
                                Err(e) => {
                                    error!("Error opening HID Device: {}", e);
                                    return AnalogSDKError::Failure.into();
                                }
                            }
                        }
                    }

                }

                if self.devices.len() == 0 {
                    return AnalogSDKError::NoDevices;
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

    fn handle_device_event(&self, device: &Device, cb_type: DeviceEventType) {
        self.call_cb(device,cb_type);
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
        //TODO: drop devices

        /*if self.device_info.is_some() {
            let dev = self.device_info.take();
            dev.unwrap().drop();
        }*/
    }

    fn set_device_event_cb(&mut self, cb: extern fn(DeviceEventType, DeviceInfoPointer)) -> AnalogSDKError {
        if !self.initialised {
            return AnalogSDKError::UnInitialized;
        }
        debug!("disconnected cb set");
        self.device_event_cb = Some(cb);
        AnalogSDKError::Ok
    }

    fn clear_device_event_cb(&mut self) -> AnalogSDKError {
        if !self.initialised {
            return AnalogSDKError::UnInitialized;
        }

        debug!("disconnected cb cleared");
        self.device_event_cb = None;
        AnalogSDKError::Ok
    }



    fn read_analog(&mut self, code: u16, device_id: DeviceID) -> SDKResult<f32>{
        if !self.initialised {
            return AnalogSDKError::UnInitialized.into();
        }

        if self.devices.len() == 0 {
            return AnalogSDKError::NoDevices.into();
        }

        if device_id == 0 {
            let mut analog: f32 = -1.0;
            let mut error: AnalogSDKError = AnalogSDKError::Ok;
            let mut dc = Vec::new();
            for (id, device) in self.devices.iter() {
                match device.read_analog(code).into() {
                    Ok(val) => {
                        analog = analog.max(val);
                    },
                    Err(AnalogSDKError::DeviceDisconnected) => {
                        dc.push(*id);
                        error = AnalogSDKError::DeviceDisconnected;

                    }
                    Err(e) => {
                        error = e;

                    },
                }
            };
            for dev in dc.drain(..) {
                let device = self.devices.remove(&dev).unwrap();
                self.handle_device_event(&device, DeviceEventType::Disconnected);
            }

            if analog < 0.0 {
                error.into()
            }else {
                analog.into()
            }
        }
        else{
            let mut disconnected = false;
            let ret = match self.devices.get(&device_id) {
                Some(device) => {
                    match device.read_analog(code).into() {
                        Ok(val) => val.into(),
                        Err(AnalogSDKError::DeviceDisconnected) => {
                            disconnected = true;
                            AnalogSDKError::DeviceDisconnected.into()
                        },
                        Err(e) => Err(e).into()
                    }
                },
                None => AnalogSDKError::NoDevices.into()
            };
            if disconnected {
                let dev = self.devices.remove(&device_id).unwrap();
                self.handle_device_event(&dev, DeviceEventType::Disconnected);
            }

            ret
        }
    }

    fn read_full_buffer(&mut self, max_length: usize, device_id: DeviceID) -> SDKResult<HashMap<c_ushort, c_float>> {
        if !self.initialised {
            return AnalogSDKError::UnInitialized.into();
        }

        if self.devices.len() == 0 {
            return AnalogSDKError::NoDevices.into();
        }

        if device_id == 0 {
            let mut analog: HashMap<c_ushort, c_float> = HashMap::new();
            let mut any_read = false;
            let mut error: AnalogSDKError = AnalogSDKError::Ok;
            let mut dc = Vec::new();
            for (id, device) in self.devices.iter() {
                match device.read_full_buffer(max_length).into() {
                    Ok(val) => {
                        any_read = true;
                        analog.extend(val);
                    },
                    Err(AnalogSDKError::DeviceDisconnected) => {
                        dc.push(*id);
                        error = AnalogSDKError::DeviceDisconnected;

                    }
                    Err(e) => {
                        error = e;

                    },
                }
            };
            for dev in dc.drain(..) {
                let device = self.devices.remove(&dev).unwrap();
                self.handle_device_event(&device, DeviceEventType::Disconnected);
            }

            if !any_read {
                error.into()
            }else {
                Ok(analog).into()
            }
        }
        else{
            let mut disconnected = false;
            let ret = match self.devices.get(&device_id) {
                Some(device) => {
                    match device.read_full_buffer(max_length).into() {
                        Ok(val) => Ok(val).into(),
                        Err(AnalogSDKError::DeviceDisconnected) => {
                            disconnected = true;
                            AnalogSDKError::DeviceDisconnected.into()
                        },
                        Err(e) => Err(e).into()
                    }
                },
                None => AnalogSDKError::NoDevices.into()
            };
            if disconnected {
                let dev = self.devices.remove(&device_id).unwrap();
                self.handle_device_event(&dev, DeviceEventType::Disconnected);
            }

            ret
        }
    }

    fn device_info(&mut self, buffer: &mut [DeviceInfoPointer]) -> SDKResult<c_int> {
        if !self.initialised {
            return AnalogSDKError::UnInitialized.into();
        }

        let mut count = 0;
        for (id, device) in self.devices.iter() {
            buffer[count] = device.device_info.clone();
            count = count + 1;
        }
        (count as c_int).into()
    }
}

declare_plugin!(TestPlugin, TestPlugin::new);
