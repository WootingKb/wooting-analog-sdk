#[macro_use]
extern crate log;
#[macro_use]
extern crate wooting_analog_plugin_dev;
extern crate hidapi;
#[macro_use]
extern crate objekt;
extern crate chrono;
extern crate timer;

use hidapi::DeviceInfo as DeviceInfoHID;
use hidapi::{HidApi, HidDevice};
use log::{error, info};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::os::raw::{c_float, c_ushort};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{str, thread};
use timer::{Guard, Timer};
use wooting_analog_plugin_dev::wooting_analog_common::*;
use wooting_analog_plugin_dev::*;

extern crate env_logger;

const ANALOG_BUFFER_SIZE: usize = 48;
const ANALOG_MAX_SIZE: usize = 40;
const WOOTING_VID: u16 = 0x31e3;

/// Struct holding the information we need to find the device and the analog interface
struct DeviceHardwareID {
    vid: u16,
    pid: u16,
    usage_page: u16,
}

/// Trait which defines how the Plugin can communicate with a particular device
trait DeviceImplementation: objekt::Clone + Send {
    /// Gives the device hardware ID that can be used to obtain the analog interface for this device
    fn device_hardware_id(&self) -> DeviceHardwareID;

    /// Used to determine if the given `device` matches the hardware id given by `device_hardware_id`
    fn matches(&self, device: &DeviceInfoHID) -> bool {
        let hid = self.device_hardware_id();
        //Check if the pid & hid match
        device.product_id().eq(&hid.pid)
            && device.vendor_id().eq(&hid.vid)
            && device.usage_page().eq(&hid.usage_page)
    }

    /// Convert the given raw `value` into the appropriate float value. The given value should be 0.0f-1.0f
    fn analog_value_to_float(&self, value: u8) -> f32 {
        (f32::from(value) / 255_f32).min(1.0)
    }

    /// Get the current set of pressed keys and their analog values from the given `device`. Using `buffer` to read into
    ///
    /// `max_length` is not the max length of the report, it is the max number of key + analog value pairs to read
    fn get_analog_buffer(
        &self,
        device: &HidDevice,
        max_length: usize,
    ) -> SDKResult<HashMap<c_ushort, c_float>> {
        let mut buffer: [u8; ANALOG_BUFFER_SIZE] = [0; ANALOG_BUFFER_SIZE];
        let res = device.read(&mut buffer);
        if let Err(e) = res {
            error!("Failed to read buffer: {}", e);

            return WootingAnalogResult::DeviceDisconnected.into();
        }
        //println!("{:?}", buffer);
        Ok(buffer
            .chunks_exact(3) //Split it into groups of 3 as the analog report is in the format of 2 byte code + 1 byte analog value
            .take(max_length) //Only take up to the max length of results. Doing this
            .filter(|&s| s[2] != 0) //Get rid of entries where the analog value is 0
            .map(|s| {
                (
                    ((u16::from(s[0])) << 8) | u16::from(s[1]), // Convert the first 2 bytes into the u16 code
                    self.analog_value_to_float(s[2]), //Convert the remaining byte into the float analog value
                )
            })
            .collect())
        .into()
    }

    /// Get the unique device ID from the given `device_info`
    fn get_device_id(&self, device_info: &DeviceInfoHID) -> DeviceID {
        wooting_analog_plugin_dev::generate_device_id(
            device_info.serial_number().as_ref().unwrap_or(&"NO SERIAL"),
            device_info.vendor_id(),
            device_info.product_id(),
        )
    }
}

clone_trait_object!(DeviceImplementation);

#[derive(Debug, Clone)]
struct WootingOne();

impl DeviceImplementation for WootingOne {
    fn device_hardware_id(&self) -> DeviceHardwareID {
        DeviceHardwareID {
            vid: 0x03EB,
            pid: 0xFF01,
            usage_page: 0xFF54,
        }
    }

    fn analog_value_to_float(&self, value: u8) -> f32 {
        ((f32::from(value) * 1.2) / 255_f32).min(1.0)
    }
}

#[derive(Debug, Clone)]
struct WootingTwo();

impl DeviceImplementation for WootingTwo {
    fn device_hardware_id(&self) -> DeviceHardwareID {
        DeviceHardwareID {
            vid: 0x03EB,
            pid: 0xFF02,
            usage_page: 0xFF54,
        }
    }

    fn analog_value_to_float(&self, value: u8) -> f32 {
        ((f32::from(value) * 1.2) / 255_f32).min(1.0)
    }
}

#[derive(Debug, Clone)]
struct WootingLekker();

impl DeviceImplementation for WootingLekker {
    fn device_hardware_id(&self) -> DeviceHardwareID {
        DeviceHardwareID {
            vid: WOOTING_VID,
            pid: 0x1210,
            usage_page: 0xFF54,
        }
    }

    fn analog_value_to_float(&self, value: u8) -> f32 {
        (f32::from(value)) / 255_f32
    }
}

#[derive(Debug, Clone)]
struct WootingTwoHE();

impl DeviceImplementation for WootingTwoHE {
    fn device_hardware_id(&self) -> DeviceHardwareID {
        DeviceHardwareID {
            vid: WOOTING_VID,
            pid: 0x1220,
            usage_page: 0xFF54,
        }
    }

    fn analog_value_to_float(&self, value: u8) -> f32 {
        (f32::from(value)) / 255_f32
    }
}

/// A fully contained device which uses `device_impl` to interface with the `device`
struct Device {
    pub device_info: DeviceInfo,
    buffer: Arc<Mutex<HashMap<c_ushort, c_float>>>,
    connected: Arc<AtomicBool>,
    pressed_keys: Vec<u16>,
    worker: Option<JoinHandle<i32>>,
}
unsafe impl Send for Device {}

impl Device {
    fn new(
        device_info: &DeviceInfoHID,
        device: HidDevice,
        device_impl: Box<dyn DeviceImplementation>,
    ) -> (DeviceID, Self) {
        let id_hash = device_impl.get_device_id(device_info);

        let buffer: Arc<Mutex<HashMap<c_ushort, c_float>>> =
            Arc::new(Mutex::new(Default::default()));
        let connected = Arc::new(AtomicBool::new(true));

        let worker = {
            let t_buffer = Arc::clone(&buffer);
            let t_connected = Arc::clone(&connected);

            thread::spawn(move || loop {
                if !t_connected.load(Ordering::Relaxed) {
                    return 0;
                }

                match device_impl
                    .get_analog_buffer(&device, ANALOG_MAX_SIZE)
                    .into()
                {
                    Ok(data) => {
                        let mut m = t_buffer.lock().unwrap();
                        m.clear();
                        m.extend(data);
                    }
                    Err(e) => {
                        if e != WootingAnalogResult::DeviceDisconnected {
                            error!("Read failed from device that isn't DeviceDisconnected, we got {:?}. Disconnecting device...", e);
                        }
                        t_connected.store(false, Ordering::Relaxed);
                        return 0;
                    }
                }
            })
        };

        (
            id_hash,
            Device {
                device_info: DeviceInfo::new_with_id(
                    device_info.vendor_id(),
                    device_info.product_id(),
                    device_info
                        .manufacturer_string()
                        .unwrap_or("ERR COULD NOT BE FOUND")
                        .to_string(),
                    device_info
                        .product_string()
                        .unwrap_or("ERR COULD NOT BE FOUND")
                        .to_string(),
                    id_hash,
                    DeviceType::Keyboard,
                ),
                connected,
                buffer,
                pressed_keys: vec![],
                worker: Some(worker),
            },
        )
    }

    fn read_analog(&mut self, code: u16) -> SDKResult<c_float> {
        (*self.buffer.lock().unwrap().get(&code).unwrap_or(&0.0)).into()
    }

    fn read_full_buffer(&mut self, _max_length: usize) -> SDKResult<HashMap<c_ushort, c_float>> {
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
}

impl Drop for Device {
    fn drop(&mut self) {
        //self.device_info.clone().drop();
        //Set the device to connected so the thread will stop if it hasn't already
        self.connected.store(false, Ordering::Relaxed);
        if let Some(worker) = self.worker.take() {
            worker
                .join()
                .expect("Couldn't join on the associated thread");
        }
    }
}

pub struct WootingPlugin {
    initialised: bool,
    device_event_cb: Arc<Mutex<Option<Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send>>>>,
    devices: Arc<Mutex<HashMap<DeviceID, Device>>>,
    timer: Timer,
    worker_guard: Option<Guard>,
}

const PLUGIN_NAME: &str = "Wooting Official Plugin";
impl WootingPlugin {
    fn new() -> Self {
        WootingPlugin {
            initialised: false,
            device_event_cb: Arc::new(Mutex::new(None)),
            devices: Arc::new(Mutex::new(Default::default())),
            timer: timer::Timer::new(),
            worker_guard: None,
        }
    }

    fn init_worker(&mut self) -> SDKResult<u32> {
        let init_device_closure =
            |hid: &HidApi,
             devices: &Arc<Mutex<HashMap<DeviceID, Device>>>,
             device_event_cb: &Arc<
                Mutex<Option<Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send>>>,
            >,
             device_impls: &Vec<Box<dyn DeviceImplementation>>| {
                let mut device_infos: Vec<&DeviceInfoHID> = hid.device_list().collect();

                for device_info in device_infos.iter() {
                    for device_impl in device_impls.iter() {
                        if device_impl.matches(device_info)
                            && !devices
                                .lock()
                                .unwrap()
                                .contains_key(&device_impl.get_device_id(device_info))
                        {
                            // info!("Found device impl match: {:?}", device_info);
                            match device_info.open_device(&hid) {
                                Ok(dev) => {
                                    let (id, device) =
                                        Device::new(device_info, dev, device_impl.clone());
                                    {
                                        devices.lock().unwrap().insert(id, device);
                                    }

                                    info!(
                                        "Found and opened the {:?} successfully!",
                                        device_info.product_string()
                                    );

                                    device_event_cb.lock().unwrap().as_ref().and_then(|cb| {
                                        cb(
                                            DeviceEventType::Connected,
                                            devices
                                                .lock()
                                                .unwrap()
                                                .get(&id)
                                                .unwrap()
                                                .device_info
                                                .borrow(),
                                        );
                                        Some(0)
                                    });
                                }
                                Err(e) => {
                                    error!("Error opening HID Device: {}", e);
                                    //return WootingAnalogResult::Failure.into();
                                }
                            }
                        }
                    }
                }
            };

        let device_impls: Vec<Box<dyn DeviceImplementation>> = vec![
            Box::new(WootingOne()),
            Box::new(WootingTwo()),
            Box::new(WootingLekker()),
            Box::new(WootingTwoHE()),
        ];
        let mut hid = match HidApi::new() {
            Ok(mut api) => {
                //An attempt at trying to ensure that all the devices have been found in the initialisation of the plugins
                if let Err(e) = api.refresh_devices() {
                    error!("We got error while refreshing devices. Err: {}", e);
                }
                api
            }
            Err(e) => {
                error!("Error obtaining HIDAPI: {}", e);
                return Err(WootingAnalogResult::Failure).into();
            }
        };

        //We wanna call it in this thread first so we can get hold of any connected devices now so we can return an accurate result for initialise
        init_device_closure(&hid, &self.devices, &self.device_event_cb, &device_impls);

        self.worker_guard = Some({
            let t_devices = Arc::clone(&self.devices);
            let t_device_event_cb = Arc::clone(&self.device_event_cb);
            self.timer
                .schedule_repeating(chrono::Duration::milliseconds(500), move || {
                    //Check if any of the devices have disconnected and get rid of them if they have
                    {
                        let mut disconnected: Vec<u64> = vec![];
                        for (&id, device) in t_devices.lock().unwrap().iter() {
                            if !device.connected.load(Ordering::Relaxed) {
                                disconnected.push(id);
                            }
                        }

                        for id in disconnected.iter() {
                            let device = t_devices.lock().unwrap().remove(id).unwrap();
                            t_device_event_cb.lock().unwrap().as_ref().and_then(|cb| {
                                cb(DeviceEventType::Disconnected, &device.device_info);
                                Some(0)
                            });
                        }
                    }

                    if let Err(e) = hid.refresh_devices() {
                        error!("We got error while refreshing devices. Err: {}", e);
                    }
                    init_device_closure(&hid, &t_devices, &t_device_event_cb, &device_impls);
                })
        });
        debug!("Started timer");
        Ok(self.devices.lock().unwrap().len() as u32).into()
    }
}

impl Plugin for WootingPlugin {
    fn name(&mut self) -> SDKResult<&'static str> {
        Ok(PLUGIN_NAME).into()
    }

    fn initialise(
        &mut self,
        callback: Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send>,
    ) -> SDKResult<u32> {
        if let Err(e) = env_logger::try_init() {
            error!("Unable to initialize Env Logger: {}", e);
        }

        let ret = self.init_worker();
        self.device_event_cb.lock().unwrap().replace(callback);
        self.initialised = ret.is_ok();
        ret
    }

    fn is_initialised(&mut self) -> bool {
        self.initialised
    }

    fn unload(&mut self) {
        info!("{} unloaded", PLUGIN_NAME);
        self.devices.lock().unwrap().drain();
        drop(self.worker_guard.take());
        self.initialised = false;
        //for dev in self.devices.clone().unwrap().drain(..) {
        //handle_device_event(self.device_event_cb.lock().unwrap().borrow(), &dev, DeviceEventType::Disconnected);
        //}
    }

    fn read_analog(&mut self, code: u16, device_id: DeviceID) -> SDKResult<f32> {
        if !self.initialised {
            return WootingAnalogResult::UnInitialized.into();
        }

        if self.devices.lock().unwrap().is_empty() {
            return WootingAnalogResult::NoDevices.into();
        }

        //If the Device ID is 0 we want to go through all the connected devices
        //and combine the analog values
        if device_id == 0 {
            let mut analog: f32 = -1.0;
            let mut error: WootingAnalogResult = WootingAnalogResult::Ok;
            for (_id, device) in self.devices.lock().unwrap().iter_mut() {
                match device.read_analog(code).into() {
                    Ok(val) => {
                        analog = analog.max(val);
                    }
                    Err(e) => {
                        error = e;
                    }
                }
            }

            if analog < 0.0 {
                error.into()
            } else {
                analog.into()
            }
        } else
        //If the device id is not 0, we try and find a connected device with that ID and read from it
        {
            match self.devices.lock().unwrap().get_mut(&device_id) {
                Some(device) => match device.read_analog(code).into() {
                    Ok(val) => val.into(),
                    Err(e) => Err(e).into(),
                },
                None => WootingAnalogResult::NoDevices.into(),
            }
        }
    }

    fn read_full_buffer(
        &mut self,
        max_length: usize,
        device_id: DeviceID,
    ) -> SDKResult<HashMap<c_ushort, c_float>> {
        if !self.initialised {
            return WootingAnalogResult::UnInitialized.into();
        }

        if self.devices.lock().unwrap().is_empty() {
            return WootingAnalogResult::NoDevices.into();
        }

        //If the Device ID is 0 we want to go through all the connected devices
        //and combine the analog values
        if device_id == 0 {
            let mut analog: HashMap<c_ushort, c_float> = HashMap::new();
            let mut any_read = false;
            let mut error: WootingAnalogResult = WootingAnalogResult::Ok;
            for (_id, device) in self.devices.lock().unwrap().iter_mut() {
                match device.read_full_buffer(max_length).into() {
                    Ok(val) => {
                        any_read = true;
                        analog.extend(val);
                    }
                    Err(e) => {
                        error = e;
                    }
                }
            }

            if !any_read {
                error.into()
            } else {
                Ok(analog).into()
            }
        } else
        //If the device id is not 0, we try and find a connected device with that ID and read from it
        {
            match self.devices.lock().unwrap().get_mut(&device_id) {
                Some(device) => match device.read_full_buffer(max_length).into() {
                    Ok(val) => Ok(val).into(),
                    Err(e) => Err(e).into(),
                },
                None => WootingAnalogResult::NoDevices.into(),
            }
        }
    }

    fn device_info(&mut self) -> SDKResult<Vec<DeviceInfo>> {
        if !self.initialised {
            return WootingAnalogResult::UnInitialized.into();
        }

        let mut devices = vec![];
        for (_id, device) in self.devices.lock().unwrap().iter() {
            devices.push(device.device_info.clone());
        }

        Ok(devices).into()
    }
}

declare_plugin!(WootingPlugin, WootingPlugin::new);
