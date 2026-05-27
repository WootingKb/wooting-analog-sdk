pub(crate) mod c;

use dyn_clone::DynClone;
use hidapi::DeviceInfo as DeviceInfoHID;
use hidapi::{HidApi, HidDevice};
use log::*;
use log::{error, info};
use std::borrow::Borrow;
use std::collections::HashMap;
use std::os::raw::{c_float, c_ushort};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::{str, thread};

#[cfg(feature = "virtual-input")]
use crate::virtual_input::VirtualKeyboard;
use crate::{
    AnalogValue, DeviceEventType, DeviceID, DeviceInfo, DeviceType, KeyCode, KeyMetadata,
    KeySource, Position, SDKResult, ValueMetadata, WootingAnalogResult,
};

#[cfg(target_os = "macos")]
pub const DEFAULT_PLUGIN_DIR: &str = "/usr/local/share/WootingAnalogPlugins";
#[cfg(target_os = "linux")]
pub const DEFAULT_PLUGIN_DIR: &str = "/usr/local/share/WootingAnalogPlugins";
#[cfg(target_os = "windows")]
pub const DEFAULT_PLUGIN_DIR: &str = "C:\\Program Files\\WootingAnalogPlugins";

pub static ANALOG_SDK_PLUGIN_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The core Plugin trait which needs to be implemented for an Analog Plugin to function
pub trait Plugin {
    /// Get a name describing the `Plugin`.
    fn name(&mut self) -> SDKResult<&'static str>;

    /// Initialise the plugin with the given function for device events. Returns an int indicating the number of connected devices
    fn initialise(
        &mut self,
        callback: Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send>,
    ) -> SDKResult<u32>;

    /// A function fired to check if the plugin is currently initialised
    fn is_initialised(&mut self) -> bool;

    /// This function is fired by the SDK to collect up all Device Info structs. The memory for the struct should be retained and only dropped
    /// when the device is disconnected or the plugin is unloaded. This ensures that the Device Info is not garbled when it's being accessed by the client.
    ///
    /// # Notes
    ///
    /// Although, the client should be copying any data they want to use for a prolonged time as there is no lifetime guarantee on the data.
    fn device_info(&mut self) -> SDKResult<Vec<DeviceInfo>>;

    /// A callback fired immediately before the plugin is unloaded. Use this if
    /// you need to do any cleanup.
    fn unload(&mut self) {}

    /// Function called to get the analog value for a particular HID key `code` from the device with ID `device`.
    /// If `device` is 0 then no specific device is specified and the value should be read from all devices and combined
    fn read_analog(&mut self, code: u16, device_id: DeviceID) -> SDKResult<f32>;

    // TODO: return type - might want a AnalogV2Value or something and convert AnalogValue to that.
    fn read_analog_with_ctx(
        &mut self,
        key_source: KeySource,
        device: DeviceID,
    ) -> SDKResult<AnalogValue>;

    /// Function called to get the full analog read buffer for a particular device with ID `device`. `max_length` is the maximum amount
    /// of keys that can be accepted, any more beyond this will be ignored by the SDK.
    /// If `device` is 0 then no specific device is specified and the data should be read from all devices and combined
    fn read_full_buffer(
        &mut self,
        max_length: usize,
        device: DeviceID,
    ) -> SDKResult<HashMap<c_ushort, c_float>>;
}

// // TODO: prototype
// #[derive(Debug)]
// #[repr(C)]
// pub struct AnalogV2Value {
//     pub actuated: bool,
//     pub position: Position,
//     pub value: f32,
//     // TODO: akc_active: bool
// }

const ANALOG_BUFFER_SIZE_V1: usize = 48;
const ANALOG_BUFFER_SIZE_V2: usize = 64;
const ANALOG_INTERFACE_V1: u16 = 0xFF54;
const ANALOG_INTERFACE_V2: u16 = 0xFF53;
const ANALOG_MAX_SIZE: usize = 40;
const WOOTING_VID: u16 = 0x31e3;
const WOOTING_PID_MODE_MASK: u16 = 0xFFF0;

/// Struct holding the information we need to find the device and the analog interface
struct DeviceHardwareID {
    vid: u16,
    pid: Option<u16>,
    usage_page: u16,
    has_modes: bool,
}

/// Trait which defines how the Plugin can communicate with a particular device
trait DeviceImplementation: DynClone + Send {
    /// Gives the device hardware ID that can be used to obtain the analog interface for this device
    fn device_hardware_id(&self) -> DeviceHardwareID;

    /// Used to determine if the given `device` matches the hardware id given by `device_hardware_id`
    fn matches(&self, device: &DeviceInfoHID) -> bool {
        let hid = self.device_hardware_id();
        let pid = if hid.has_modes {
            device.product_id() & WOOTING_PID_MODE_MASK
        } else {
            device.product_id()
        };
        //Check if the pid & hid match
        (hid.pid.is_none() || hid.pid.map_or(false, |hid_pid| pid == hid_pid))
            && device.vendor_id().eq(&hid.vid)
            && device.usage_page().eq(&hid.usage_page)
    }

    /// Convert the given raw `value` into the appropriate float value. The given value should be 0.0f-1.0f
    fn analog_value_to_float(&self, value: u16) -> f32 {
        (f32::from(value) / 255_f32).min(1.0)
    }

    /// Get the current set of pressed keys and their analog values from the given `device`. Using `buffer` to read into
    ///
    /// `max_length` is not the max length of the report, it is the max number of key + analog value pairs to read
    fn get_analog_buffer(
        &self,
        device: &HidDevice,
        max_length: usize,
    ) -> SDKResult<Option<HashMap<c_ushort, c_float>>> {
        let mut buffer: [u8; ANALOG_BUFFER_SIZE_V1] = [0; ANALOG_BUFFER_SIZE_V1];
        let res = device.read_timeout(&mut buffer, 50);

        match res {
            Ok(len) => {
                // If the length is 0 then that means the read timed out, so we shouldn't use it to update values
                if len == 0 {
                    return Ok(None).into();
                }
            }
            Err(e) => {
                error!("Failed to read buffer: {}", e);

                return Err(WootingAnalogResult::DeviceDisconnected).into();
            }
        }
        Ok(Some(
            buffer
                .chunks_exact(3) //Split it into groups of 3 as the analog report is in the format of 2 byte code + 1 byte analog value
                .take(max_length) //Only take up to the max length of results. Doing this
                .filter(|&s| s[2] != 0) //Get rid of entries where the analog value is 0
                .map(|s| {
                    (
                        ((u16::from(s[0])) << 8) | u16::from(s[1]), // Convert the first 2 bytes into the u16 code
                        self.analog_value_to_float(u16::from(s[2])), //Convert the remaining byte into the float analog value
                    )
                })
                .collect(),
        ))
        .into()
    }

    fn get_analog_buffer_with_ctx(
        &self,
        _device: &HidDevice,
        _max_length: usize,
    ) -> SDKResult<Option<HashMap<KeyCode, AnalogValue>>> {
        SDKResult(Ok(None))
    }

    /// Get the unique device ID from the given `device_info`
    fn get_device_id(&self, device_info: &DeviceInfoHID) -> DeviceID {
        crate::generate_device_id(
            device_info.serial_number().as_ref().unwrap_or(&"NO SERIAL"),
            device_info.vendor_id(),
            device_info.product_id(),
        )
    }
}

dyn_clone::clone_trait_object!(DeviceImplementation);

#[derive(Debug, Clone)]
struct WootingOne;

impl DeviceImplementation for WootingOne {
    fn device_hardware_id(&self) -> DeviceHardwareID {
        DeviceHardwareID {
            vid: 0x03EB,
            pid: Some(0xFF01),
            usage_page: 0xFF54,
            has_modes: false,
        }
    }

    fn analog_value_to_float(&self, value: u16) -> f32 {
        ((f32::from(value) * 1.2) / 255_f32).min(1.0)
    }
}

#[derive(Debug, Clone)]
struct WootingTwo;

impl DeviceImplementation for WootingTwo {
    fn device_hardware_id(&self) -> DeviceHardwareID {
        DeviceHardwareID {
            vid: 0x03EB,
            pid: Some(0xFF02),
            usage_page: 0xFF54,
            has_modes: false,
        }
    }

    fn analog_value_to_float(&self, value: u16) -> f32 {
        ((f32::from(value) * 1.2) / 255_f32).min(1.0)
    }
}

#[derive(Debug, Clone)]
struct WootingNewFirmware;

impl DeviceImplementation for WootingNewFirmware {
    fn device_hardware_id(&self) -> DeviceHardwareID {
        DeviceHardwareID {
            vid: WOOTING_VID,
            pid: None,
            usage_page: 0xFF54,
            has_modes: true,
        }
    }
}

#[derive(Debug, Clone)]
struct WootingAnalogProtocolV2;

impl DeviceImplementation for WootingAnalogProtocolV2 {
    fn device_hardware_id(&self) -> DeviceHardwareID {
        DeviceHardwareID {
            vid: WOOTING_VID,
            pid: None,
            usage_page: 0xFF53,
            has_modes: true,
        }
    }

    fn get_analog_buffer(
        &self,
        device: &HidDevice,
        max_length: usize,
    ) -> SDKResult<Option<HashMap<c_ushort, c_float>>> {
        SDKResult(
            match self.get_analog_buffer_with_ctx(device, max_length).0 {
                Ok(data) => match data {
                    Some(data) => Ok(Some(
                        data.iter()
                            .map(|(k, v)| (k.as_u16(), v.as_f32()))
                            .collect::<HashMap<c_ushort, c_float>>(),
                    )),
                    None => Ok(None),
                },
                Err(e) => Err(e),
            },
        )
    }

    fn get_analog_buffer_with_ctx(
        &self,
        device: &HidDevice,
        max_length: usize,
    ) -> SDKResult<Option<HashMap<KeyCode, AnalogValue>>> {
        let mut buffer: [u8; ANALOG_BUFFER_SIZE_V2] = [0; ANALOG_BUFFER_SIZE_V2];
        let res = device.read_timeout(&mut buffer, 50);

        match res {
            Ok(len) => {
                // If the length is 0 then that means the read timed out, so we shouldn't use it to update values
                if len == 0 {
                    return Ok(None).into();
                }
            }
            Err(e) => {
                error!("Failed to read buffer: {}", e);

                return Err(WootingAnalogResult::DeviceDisconnected).into();
            }
        }
        Ok(Some(
            buffer
                .chunks_exact(4)
                .take(max_length)
                .filter(|&b| b[3] != 0)
                .map(|b| {
                    let matrix_pos = b[0];
                    let key = b[1];
                    let packed = b[2];
                    let value = b[3];

                    let row = (matrix_pos >> 5) & 0x07;
                    let col = matrix_pos & 0x1F;
                    let actuated = (packed & 0x01) != 0;
                    let _reserved = packed >> 1;
                    let key_namespace = (packed >> 2) & 0x0F;
                    let value_part = (packed >> 6) & 0x03;

                    let value = (u16::from(value) << 2) | u16::from(value_part);

                    (
                        KeyCode::from((u16::from(key_namespace) << 8) | u16::from(key))
                            .with_metadata(KeyMetadata::Basic {
                                namespace: key_namespace,
                            }),
                        AnalogValue::from(self.analog_value_to_float(value)).with_metadata(
                            ValueMetadata::Basic {
                                pos: Position::new(col, row),
                                actuated,
                            },
                        ),
                    )
                })
                .collect(),
        ))
        .into()
    }

    fn analog_value_to_float(&self, value: u16) -> f32 {
        (f32::from(value) / 1023.).min(1.0)
    }
}

/// A fully contained device which uses `device_impl` to interface with the `device`
struct Device {
    pub device_info: DeviceInfo,
    buffer: Arc<Mutex<HashMap<KeyCode, AnalogValue>>>,
    connected: Arc<AtomicBool>,
    pressed_keys: Vec<KeyCode>,
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

        let buffer: Arc<Mutex<HashMap<KeyCode, AnalogValue>>> = Default::default();
        let connected = Arc::new(AtomicBool::new(true));

        let worker = {
            let t_buffer = Arc::clone(&buffer);
            let t_connected = Arc::clone(&connected);

            thread::spawn(move || {
                loop {
                    if !t_connected.load(Ordering::Relaxed) {
                        return 0;
                    }

                    match device_impl.device_hardware_id().usage_page {
                        ANALOG_INTERFACE_V1 => {
                            match device_impl
                                .get_analog_buffer(&device, ANALOG_MAX_SIZE)
                                .into()
                            {
                                Ok(data) => {
                                    if let Some(data) = data {
                                        let mut map = t_buffer.lock().unwrap();
                                        map.clear();
                                        map.extend(data.iter().map(|(k, v)| {
                                            (KeyCode::from(*k), AnalogValue::from(*v))
                                        }));
                                    }
                                }
                                Err(e) => {
                                    if e != WootingAnalogResult::DeviceDisconnected {
                                        error!(
                                            "Read failed from device that isn't DeviceDisconnected, we got {:?}. Disconnecting device...",
                                            e
                                        );
                                    }
                                    t_connected.store(false, Ordering::Relaxed);
                                    return 0;
                                }
                            }
                        }
                        ANALOG_INTERFACE_V2 => {
                            match device_impl
                                .get_analog_buffer_with_ctx(&device, ANALOG_MAX_SIZE)
                                .into()
                            {
                                Ok(data) => {
                                    if let Some(data) = data {
                                        let mut map = t_buffer.lock().unwrap();
                                        map.clear();
                                        map.extend(data);
                                    }
                                }
                                Err(e) => {
                                    if e != WootingAnalogResult::DeviceDisconnected {
                                        error!(
                                            "Read failed from device that isn't DeviceDisconnected, we got {:?}. Disconnecting device...",
                                            e
                                        );
                                    }
                                    t_connected.store(false, Ordering::Relaxed);
                                    return 0;
                                }
                            }
                        }
                        _ => unreachable!(),
                    };
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

    fn read_analog_with_ctx(&mut self, key_source: KeySource) -> SDKResult<AnalogValue> {
        let value = match key_source {
            KeySource::Code(code) => *self
                .buffer
                .lock()
                .unwrap()
                // TODO: could add this directly to KeySource?
                .get(&KeyCode::from(code))
                .unwrap_or(&AnalogValue::default()),
            KeySource::Position(position) => self
                .buffer
                .lock()
                .unwrap()
                .iter()
                .find_map(|(_, v)| match v.metadata {
                    ValueMetadata::None => None,
                    ValueMetadata::Basic { pos, .. } => {
                        if pos == position {
                            Some(*v)
                        } else {
                            None
                        }
                    }
                })
                .unwrap_or(AnalogValue::default()),
        };

        SDKResult(Ok(value))
    }

    fn read_full_with_ctx(&mut self) -> SDKResult<HashMap<KeyCode, AnalogValue>> {
        let mut buffer = self.buffer.lock().unwrap().clone();
        //Collect the new pressed keys
        let new_pressed_keys: Vec<KeyCode> = buffer.keys().cloned().collect();

        //Put the old pressed keys into the buffer
        for key in self.pressed_keys.drain(..) {
            buffer.entry(key).or_default();
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
    initialised: Arc<AtomicBool>,
    device_event_cb: Arc<Mutex<Option<Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send>>>>,
    devices: Arc<Mutex<HashMap<DeviceID, Device>>>,
    #[cfg(feature = "virtual-input")]
    virtual_keyboard: VirtualKeyboard,
    thread: Option<JoinHandle<()>>,
}

const PLUGIN_NAME: &str = "Wooting Official Plugin";
impl WootingPlugin {
    pub(crate) fn new() -> Self {
        WootingPlugin {
            initialised: Arc::new(false.into()),
            device_event_cb: Arc::new(Mutex::new(None)),
            devices: Arc::new(Mutex::new(Default::default())),
            #[cfg(feature = "virtual-input")]
            virtual_keyboard: VirtualKeyboard::new(),
            thread: None,
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
                let device_infos: Vec<&DeviceInfoHID> = hid.device_list().collect();

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

        let refresh_devices = |hid: &mut HidApi| -> hidapi::HidResult<()> {
            hid.reset_devices()?;
            hid.add_devices(WOOTING_VID, 0)?;
            hid.add_devices(0x03EB, 0xFF01)?;
            hid.add_devices(0x03EB, 0xFF02)?;
            Ok(())
        };

        let device_impls: Vec<Box<dyn DeviceImplementation>> = vec![
            Box::new(WootingOne),
            Box::new(WootingTwo),
            Box::new(WootingNewFirmware),
            Box::new(WootingAnalogProtocolV2),
        ];
        let mut hid = match HidApi::new_without_enumerate() {
            Ok(mut api) => {
                //An attempt at trying to ensure that all the devices have been found in the initialisation of the plugins
                if let Err(e) = refresh_devices(&mut api) {
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

        let t_initialised = Arc::clone(&self.initialised);
        let t_devices = Arc::clone(&self.devices);
        let t_device_event_cb = Arc::clone(&self.device_event_cb);

        self.thread = Some(thread::spawn(move || {
            let mut i = 0;
            while t_initialised.load(Ordering::Relaxed) {
                if i == 500 {
                    i = 0;

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

                    if let Err(e) = refresh_devices(&mut hid) {
                        error!("We got error while refreshing devices. Err: {}", e);
                    }
                    init_device_closure(&hid, &t_devices, &t_device_event_cb, &device_impls);
                }
                thread::sleep(std::time::Duration::from_millis(10));
                i += 10;
            }
        }));

        #[cfg(feature = "virtual-input")]
        self.virtual_keyboard.attach();

        debug!("Started thread");
        Ok(self.devices.lock().unwrap().len() as u32).into()
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn _plugin_create() -> *mut dyn Plugin {
        let boxed: Box<dyn Plugin> = Box::new(Self::new());
        Box::into_raw(boxed)
    }

    #[unsafe(no_mangle)]
    pub extern "C" fn plugin_version() -> &'static str {
        ANALOG_SDK_PLUGIN_VERSION
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
            warn!("Unable to initialize Env Logger: {}", e);
        }

        let ret = self.init_worker();

        #[cfg(feature = "virtual-input")]
        {
            let cb = Arc::new(Mutex::new(Some(callback)));
            self.device_event_cb = Arc::clone(&cb);
            self.virtual_keyboard.with_device_events(cb);
        }

        #[cfg(not(feature = "virtual-input"))]
        {
            self.device_event_cb = Arc::new(Mutex::new(Some(callback)));
        }

        self.initialised.store(ret.is_ok(), Ordering::Relaxed);
        ret
    }

    fn is_initialised(&mut self) -> bool {
        self.initialised.load(Ordering::Relaxed)
    }

    fn unload(&mut self) {
        self.initialised.store(false, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            t.join().unwrap();
        };
        self.devices.lock().unwrap().drain();

        info!("{} unloaded", PLUGIN_NAME);
    }

    fn read_analog(&mut self, code: u16, device_id: DeviceID) -> SDKResult<f32> {
        let value = self.read_analog_with_ctx(KeySource::Code(code), device_id);

        match value.0 {
            Ok(v) => v.inner.into(),
            Err(e) => Err(e).into(),
        }
    }

    fn read_analog_with_ctx(
        &mut self,
        key_source: KeySource,
        device_id: DeviceID,
    ) -> SDKResult<AnalogValue> {
        if !self.initialised.load(Ordering::Relaxed) {
            return Err(WootingAnalogResult::UnInitialized).into();
        }

        if self.devices.lock().unwrap().is_empty() {
            return Err(WootingAnalogResult::NoDevices).into();
        }

        //If the Device ID is 0 we want to go through all the connected devices
        //and combine the analog values
        if device_id == 0 {
            let mut analog: AnalogValue = AnalogValue::from(-1.0);
            let mut error: WootingAnalogResult = WootingAnalogResult::Ok;
            for (_id, device) in self.devices.lock().unwrap().iter_mut() {
                match device.read_analog_with_ctx(key_source).into() {
                    Ok(val) => analog = analog.max(val),
                    Err(e) => error = e,
                }
            }

            if analog < 0.0
            /*|| analog.metadata == ValueMetadata::None*/
            {
                Err(error).into()
            } else {
                SDKResult(Ok(analog))
            }
        } else
        //If the device id is not 0, we try and find a connected device with that ID and read from it
        {
            match self.devices.lock().unwrap().get_mut(&device_id) {
                Some(device) => match device.read_analog_with_ctx(key_source).into() {
                    Ok(val) => Ok(val).into(),
                    Err(e) => Err(e).into(),
                },
                None => Err(WootingAnalogResult::NoDevices).into(),
            }
        }
    }

    fn read_full_buffer(
        &mut self,
        max_length: usize,
        device_id: DeviceID,
    ) -> SDKResult<HashMap<c_ushort, c_float>> {
        if !self.initialised.load(Ordering::Relaxed) {
            return Err(WootingAnalogResult::UnInitialized).into();
        }

        if self.devices.lock().unwrap().is_empty() {
            return Err(WootingAnalogResult::NoDevices).into();
        }

        //If the Device ID is 0 we want to go through all the connected devices
        //and combine the analog values
        if device_id == 0 {
            let mut analog: HashMap<c_ushort, c_float> = HashMap::new();
            let mut any_read = false;
            let mut error: WootingAnalogResult = WootingAnalogResult::Ok;
            for (_id, device) in self.devices.lock().unwrap().iter_mut() {
                match device.read_full_with_ctx().into() {
                    Ok(val) => {
                        for (k, v) in val.iter().map(|(k, v)| (k.as_u16(), v.as_f32())) {
                            analog
                                .entry(k)
                                .and_modify(|value| {
                                    if &v > value {
                                        *value = v;
                                    }
                                })
                                .or_insert(v);
                            any_read = true;
                        }
                    }
                    Err(e) => {
                        error = e;
                    }
                }
            }

            if !any_read {
                Err(error).into()
            } else {
                #[cfg(feature = "virtual-input")]
                self.virtual_keyboard.iter_over(|iter| {
                    for (key, value) in iter {
                        analog
                            .entry(*key)
                            .and_modify(|v| *v = v.max(*value))
                            .or_insert(*value);
                    }
                });

                Ok(analog).into()
            }
        } else
        //If the device id is not 0, we try and find a connected device with that ID and read from it
        {
            match self.devices.lock().unwrap().get_mut(&device_id) {
                Some(device) => match device.read_full_with_ctx().into() {
                    Ok(val) => {
                        Ok(val.iter().map(|(k, v)| (k.as_u16(), v.as_f32())).collect()).into()
                    }
                    Err(e) => Err(e).into(),
                },
                None => Err(WootingAnalogResult::NoDevices).into(),
            }
        }
    }

    fn device_info(&mut self) -> SDKResult<Vec<DeviceInfo>> {
        if !self.initialised.load(Ordering::Relaxed) {
            return Err(WootingAnalogResult::UnInitialized).into();
        }

        let mut devices = vec![];
        for (_id, device) in self.devices.lock().unwrap().iter() {
            devices.push(device.device_info.clone());
        }

        Ok(devices).into()
    }
}
