#[macro_use]
extern crate log;
#[macro_use]
extern crate analog_sdk;
extern crate hidapi;
#[macro_use]
extern crate objekt;

use hidapi::{HidApi, HidDevice, HidDeviceInfo};
use std::str;
use analog_sdk::sdk::{Plugin, DeviceID, DeviceInfoPointer, DeviceInfo, AnalogSDKError, SDKResult, DeviceEventType, AnalogSDK};
use std::os::raw::{c_ushort, c_float, c_int};
use std::hash::Hasher;
use std::ffi::CString;
use std::collections::HashMap;
use log::{info, error};

extern crate env_logger;


const ANALOG_BUFFER_SIZE: usize = 48;

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

    fn analog_value_to_float(&self, value: u8) -> f32 {
        (((value as f32)*1.2) /0xFF as f32).min(1.0)
    }

    fn refresh_buffer(&self, buffer: &mut [u8], device: &HidDevice, max_length: usize) -> SDKResult<HashMap<c_ushort, c_float>> {
        let res = device.read_timeout(buffer, 0);
        if let Err(e) = res {
            error!("Failed to read buffer: {}", e);

            return AnalogSDKError::DeviceDisconnected.into();
        }
        //println!("{:?}", buffer);
        let ret: HashMap<c_ushort, c_float> = buffer.chunks_exact(3).take(max_length).filter(|&s| s[2] != 0).map(|s| (((s[0] as u16) << 8) | s[1] as u16, self.analog_value_to_float(s[2]))).collect();
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

#[derive(Clone)]
struct WootingTwo();

impl DeviceImplementation for WootingTwo {
    fn device_hardware_id(&self) -> DeviceHardwareID {
        DeviceHardwareID {
            vid: 0x03EB,
            pid: 0xFF02,
            usage_page: 0x1338,
            interface_n: 6
        }
    }
}


struct Device {
    device: HidDevice,
    pub device_info: DeviceInfoPointer,
    device_impl: Box<dyn DeviceImplementation>,
    buffer: [u8; ANALOG_BUFFER_SIZE]
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
            buffer: [0; ANALOG_BUFFER_SIZE]
        })
    }

    fn read_analog(&mut self, code: u16) -> SDKResult<c_float> {
        match self.device_impl.refresh_buffer(&mut self.buffer, &self.device, ANALOG_BUFFER_SIZE).into() {
            Ok(data) => (*data.get(&code).unwrap_or(&0.0)).into(),
            Err(e) => Err(e).into()
        }
    }

    fn read_full_buffer(&mut self, max_length: usize) -> SDKResult<HashMap<c_ushort, c_float>> {
        self.device_impl.refresh_buffer(&mut self.buffer, &self.device, max_length)
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        //TODO: drop DeviceInfoPointer
        self.device_info.clone().drop();
    }
}


#[derive( Default)]//Debug
pub struct TestPlugin {
    initialised: bool,
    device_event_cb: Option<extern fn(DeviceEventType, DeviceInfoPointer)>,
    devices: HashMap<DeviceID, Device>,
    device_impls: Vec<Box<dyn DeviceImplementation>>,
    hid_api: Option<HidApi>
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
            device_impls: vec![Box::new(WootingOne()), Box::new(WootingTwo())],
            hid_api: None
        }
    }

    fn call_cb(&self, device: &Device, event_type: DeviceEventType) {
        if let Some(cb) = self.device_event_cb {
            cb(event_type, device.device_info.clone());
        }
    }

    fn init_device(&mut self) -> AnalogSDKError {
        self.hid_api.as_mut().map(|api| api.refresh_devices());

        match &self.hid_api {
            Some(api) => {
                for device_info in api.devices() {
                    for device_impl in self.device_impls.iter() {
                        debug!("{:#?}", device_info);
                        if device_impl.matches(device_info) && !self.devices.contains_key(&device_impl.get_device_id(device_info)) {
                            match device_info.open_device(&api){
                                Ok(dev) => {
                                    let (id, device) = Device::new(device_info, dev, device_impl.clone());
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

                //debug!("Finished with devices");
            },
            None => {
                return AnalogSDKError::UnInitialized;
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
        match HidApi::new() {
            Ok(api) => {
                self.hid_api = Some(api);
            },
            Err(e) => {
                error!("Error: {}", e);
                return AnalogSDKError::Failure;
            }
        }
        let ret = self.init_device();
        self.initialised = ret.is_ok();
        if self .initialised {
            info!("{} initialised", PLUGIN_NAME);
        }
        ret
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
            for (id, device) in self.devices.iter_mut() {
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

                    }
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
            let ret = match self.devices.get_mut(&device_id) {
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
            for (id, device) in self.devices.iter_mut() {
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
            let ret = match self.devices.get_mut(&device_id) {
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
        for (_id, device) in self.devices.iter() {
            buffer[count] = device.device_info.clone();
            count = count + 1;
        }
        (count as c_int).into()
    }
}

declare_plugin!(TestPlugin, TestPlugin::new);
