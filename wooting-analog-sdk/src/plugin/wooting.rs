use hidapi::{DeviceInfo as DeviceInfoHID, HidApi};
use log::{debug, error, info, warn};
use std::{
    borrow::Borrow,
    collections::HashMap,
    os::raw::{c_float, c_ushort},
    str,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread::{self, JoinHandle},
};

#[cfg(feature = "virtual-input")]
use crate::virtual_input::VirtualKeyboard;
use crate::{
    AnalogValue, KeyCode, KeyPosition, PhysicalKey, Plugin,
    analog_value::ValueMetadata,
    device::{
        Device, DeviceEventType, DeviceID, DeviceImplementation, DeviceInfo,
        WootingAnalogProtocolV2, WootingNewFirmware, WootingOne, WootingTwo,
    },
    err::{DeviceError, PluginError, ReadError},
    key::{Key, KeyState},
};

pub(crate) const ANALOG_BUFFER_SIZE_V1: usize = 48;
pub(crate) const ANALOG_BUFFER_SIZE_V2: usize = 64;
pub(crate) const ANALOG_INTERFACE_V1: u16 = 0xFF54;
pub(crate) const ANALOG_INTERFACE_V2: u16 = 0xFF53;
pub(crate) const ANALOG_MAX_SIZE: usize = 40;
pub(crate) const WOOTING_PID_MODE_MASK: u16 = 0xFFF0;

/// The legacy Wooting Vendor ID
pub(crate) const LEGACY_WOOTING_VID: u16 = 0x03EB;
/// The most-recent Wooting Vendor ID
pub const WOOTING_VID: u16 = 0x31e3;

type DeviceEvents = Arc<Mutex<Option<Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send + Sync>>>>;

pub struct WootingPlugin {
    initialised: Arc<AtomicBool>,
    device_event_cb: DeviceEvents,
    devices: Arc<Mutex<HashMap<DeviceID, Device>>>,
    #[cfg(feature = "virtual-input")]
    virtual_keyboard: VirtualKeyboard,
    thread: Option<JoinHandle<()>>,
    analog_data: AnalogData,
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
            analog_data: AnalogData::new(),
        }
    }

    fn init_worker(&mut self) -> Result<u32, DeviceError> {
        let init_device_closure =
            |hid: &HidApi,
             devices: &Arc<Mutex<HashMap<DeviceID, Device>>>,
             device_event_cb: &DeviceEvents,
             device_impls: &Vec<Box<dyn DeviceImplementation>>| {
                let device_infos: Vec<&DeviceInfoHID> = hid.device_list().collect();

                for device_info in &device_infos {
                    for device_impl in device_impls {
                        if device_impl.matches(device_info)
                            && !devices
                                .lock()
                                .unwrap()
                                .contains_key(&device_impl.get_device_id(device_info))
                        {
                            // info!("Found device impl match: {:?}", device_info);
                            match device_info.open_device(hid) {
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

                                    device_event_cb.lock().unwrap().as_ref().map(|cb| {
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
                return Err(DeviceError::zero_devices());
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
                        for (&id, device) in &*t_devices.lock().unwrap() {
                            if !device.connected.load(Ordering::Relaxed) {
                                disconnected.push(id);
                            }
                        }

                        for id in &disconnected {
                            let device = t_devices.lock().unwrap().remove(id).unwrap();
                            t_device_event_cb.lock().unwrap().as_ref().map(|cb| {
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
        Ok(self.devices.lock().unwrap().len() as u32)
    }
}

impl Plugin for WootingPlugin {
    fn name(&mut self) -> Result<&'static str, PluginError> {
        Ok(PLUGIN_NAME)
    }

    fn initialise(
        &mut self,
        callback: Box<dyn Fn(DeviceEventType, &DeviceInfo) + Send + Sync>,
    ) -> Result<u32, ReadError> {
        if let Err(e) = env_logger::try_init() {
            warn!("Unable to initialize Env Logger: {}", e);
        }

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

        self.initialised.store(true, Ordering::Relaxed);

        match self.init_worker() {
            Ok(ret) => Ok(ret),
            Err(err) => {
                self.initialised.store(false, Ordering::Relaxed);
                Err(err.into())
            }
        }
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

    fn read_analog(&mut self, code: u16, device_id: DeviceID) -> Result<f32, ReadError> {
        self.read_keycode(KeyCode::with_namespace(code), device_id)
            .map(|v| v.inner)
    }

    fn read_keycode(
        &mut self,
        code: KeyCode,
        device_id: DeviceID,
    ) -> Result<AnalogValue, ReadError> {
        if !self.initialised.load(Ordering::Relaxed) {
            return Err(ReadError::Uninitialized);
        }

        if self.devices.lock().unwrap().is_empty() {
            return Err(ReadError::Device(DeviceError::zero_devices()));
        }

        if device_id == 0 {
            let mut analog = AnalogValue::from(-1.0);

            for device in self.devices.lock().unwrap().values_mut() {
                analog = analog.max(device.read_keycode(code));
            }

            Ok(analog)
        } else {
            match self.devices.lock().unwrap().get_mut(&device_id) {
                Some(device) => Ok(device.read_keycode(code)),
                None => Err(ReadError::Device(DeviceError::zero_devices())),
            }
        }
    }

    fn read_position(
        &mut self,
        position: KeyPosition,
        device_id: DeviceID,
    ) -> Result<PhysicalKey, ReadError> {
        if !self.initialised.load(Ordering::Relaxed) {
            return Err(ReadError::Uninitialized);
        }

        if self.devices.lock().unwrap().is_empty() {
            return Err(ReadError::Device(DeviceError::zero_devices()));
        }

        if device_id == 0 {
            let mut result = PhysicalKey::new(position);

            for device in self.devices.lock().unwrap().values_mut() {
                let pk = device.read_position(position);

                for i in 0..pk.active_key_count {
                    result.push_state(pk.state[i as usize]);
                }
            }

            Ok(result)
        } else {
            match self.devices.lock().unwrap().get_mut(&device_id) {
                Some(device) => Ok(device.read_position(position)),
                None => Err(ReadError::Device(DeviceError::zero_devices())),
            }
        }
    }

    fn read_full_buffer(
        &mut self,
        device_id: DeviceID,
    ) -> Result<HashMap<c_ushort, c_float>, ReadError> {
        if !self.initialised.load(Ordering::Relaxed) {
            return Err(ReadError::Uninitialized);
        }

        if self.devices.lock().unwrap().is_empty() {
            return Err(ReadError::Device(DeviceError::zero_devices()));
        }

        //If the Device ID is 0 we want to go through all the connected devices
        //and combine the analog values
        if device_id == 0 {
            let mut analog: HashMap<c_ushort, c_float> = HashMap::new();
            let mut any_read = false;
            let mut error = None;
            for device in self.devices.lock().unwrap().values_mut() {
                match device.read_full_with_ctx() {
                    Ok(val) => {
                        for (k, v) in val.iter().map(|k| (u16::from(k.code), f32::from(k.value))) {
                            analog
                                .entry(k)
                                .and_modify(|value| {
                                    if &v > value {
                                        *value = v;
                                    }
                                })
                                .or_insert(v);
                        }
                        any_read = true;
                    }
                    Err(e) => {
                        error = Some(e);
                    }
                }
            }

            if let Some(err) = error
                && !any_read
            {
                return Err(ReadError::Device(err));
            }

            #[cfg(feature = "virtual-input")]
            self.virtual_keyboard.iter_over(|iter| {
                for (key, value) in iter {
                    analog
                        .entry(*key)
                        .and_modify(|v| *v = v.max(*value))
                        .or_insert(*value);
                }
            });

            Ok(analog)
        } else
        //If the device id is not 0, we try and find a connected device with that ID and read from it
        {
            match self.devices.lock().unwrap().get_mut(&device_id) {
                Some(device) => device
                    .read_full_with_ctx()
                    .map(|values| {
                        values
                            .iter()
                            .map(|k| (u16::from(k.code), f32::from(k.value)))
                            .collect()
                    })
                    .map_err(ReadError::Device),
                None => Err(ReadError::Device(DeviceError::zero_devices())),
            }
        }
    }

    fn read_keycodes(
        &mut self,
        device_id: DeviceID,
    ) -> Result<HashMap<KeyCode, AnalogValue>, ReadError> {
        if !self.initialised.load(Ordering::Relaxed) {
            return Err(ReadError::Uninitialized);
        }

        if self.devices.lock().unwrap().is_empty() {
            return Err(ReadError::Device(DeviceError::zero_devices()));
        }

        // If the Device ID is 0 we want to go through all the connected devices
        // and combine the analog values
        if device_id == 0 {
            let mut any_read = false;
            let mut error = None;

            for device in self.devices.lock().unwrap().values_mut() {
                match device.read_full_with_ctx() {
                    Ok(keys) => {
                        self.analog_data.merge(keys);
                        any_read = true;
                    }
                    Err(e) => {
                        error = Some(e);
                    }
                }
            }

            if let Some(err) = error
                && !any_read
            {
                return Err(ReadError::Device(err));
            }

            // TODO: convert virtual keyboard to use KeyCode and AnalogValue
            #[cfg(feature = "virtual-input")]
            self.virtual_keyboard.iter_over(|iter| {
                for (key, value) in iter {
                    self.analog_data
                        .insert_keycode(KeyCode::with_namespace(*key), AnalogValue::from(*value));
                }
            });

            Ok(std::mem::take(&mut self.analog_data.keycodes))
        } else {
            // If the device id is not 0, we try and find a connected device with that ID and read from it
            match self.devices.lock().unwrap().get_mut(&device_id) {
                Some(device) => device
                    .read_full_with_ctx()
                    .map(|keys| {
                        self.analog_data.merge(keys);
                        std::mem::take(&mut self.analog_data.keycodes)
                    })
                    .map_err(ReadError::Device),
                None => Err(ReadError::Device(DeviceError::zero_devices())),
            }
        }
    }

    fn read_positions(
        &mut self,
        device_id: DeviceID,
    ) -> Result<HashMap<KeyPosition, PhysicalKey>, ReadError> {
        if !self.initialised.load(Ordering::Relaxed) {
            return Err(ReadError::Uninitialized);
        }

        if self.devices.lock().unwrap().is_empty() {
            return Err(ReadError::Device(DeviceError::zero_devices()));
        }

        // If the Device ID is 0 we want to go through all the connected devices
        // and combine the analog values
        if device_id == 0 {
            let mut any_read = false;
            let mut error = None;

            for device in self.devices.lock().unwrap().values_mut() {
                match device.read_full_with_ctx() {
                    Ok(keys) => {
                        self.analog_data.merge(keys);
                        any_read = true;
                    }
                    Err(e) => {
                        error = Some(e);
                    }
                }
            }

            if let Some(err) = error
                && !any_read
            {
                return Err(ReadError::Device(err));
            }

            // TODO: convert virtual keyboard to use KeyCode and AnalogValue
            // #[cfg(feature = "virtual-input")]
            // self.virtual_keyboard.iter_over(|iter| {
            //     for (key, value) in iter {
            //         analog
            //             .entry(*key)
            //             .and_modify(|v| *v = v.max(*value))
            //             .or_insert(*value);
            //     }
            // });

            Ok(std::mem::take(&mut self.analog_data.positions))
        } else {
            // If the device id is not 0, we try and find a connected device with that ID and read from it
            match self.devices.lock().unwrap().get_mut(&device_id) {
                Some(device) => device
                    .read_full_with_ctx()
                    .map(|keys| {
                        self.analog_data.merge(keys);
                        std::mem::take(&mut self.analog_data.positions)
                    })
                    .map_err(ReadError::Device),
                None => Err(ReadError::Device(DeviceError::zero_devices())),
            }
        }
    }

    fn device_info(&mut self) -> Result<Vec<DeviceInfo>, ReadError> {
        if !self.initialised.load(Ordering::Relaxed) {
            return Err(ReadError::Uninitialized);
        }

        Ok(self
            .devices
            .lock()
            .unwrap()
            .values()
            .map(|device| device.device_info.clone())
            .collect())
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct AnalogData {
    keycodes: HashMap<KeyCode, AnalogValue>,
    positions: HashMap<KeyPosition, PhysicalKey>,
}

impl AnalogData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_keycode(&mut self, code: KeyCode, value: AnalogValue) {
        self.keycodes
            .entry(code)
            .and_modify(|existing| {
                if &value > existing {
                    *existing = value;
                }
            })
            .or_insert(value);
    }

    pub fn insert_position_with(&mut self, position: KeyPosition, state: KeyState) {
        self.positions
            .entry(position)
            .and_modify(|existing| {
                existing.push_state(state);
            })
            .or_insert_with(|| {
                let mut pk = PhysicalKey::new(position);
                pk.push_state(state);
                pk
            });
    }

    pub fn merge(&mut self, keys: Vec<Key>) {
        for key in keys {
            if let ValueMetadata::Basic {
                position: pos,
                actuated,
            } = key.value.metadata
            {
                let key_state = KeyState {
                    value: key.value.inner,
                    keycode: key.code,
                    actuated,
                };
                self.insert_position_with(pos, key_state);
            }

            self.insert_keycode(key.code, key.value);
        }
    }
}

impl From<Vec<Key>> for AnalogData {
    fn from(keys: Vec<Key>) -> Self {
        let mut data = Self::new();

        data.merge(keys);

        data
    }
}
