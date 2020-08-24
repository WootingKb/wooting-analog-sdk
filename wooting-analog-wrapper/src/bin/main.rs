use sdk::{DeviceEventType, DeviceInfo, DeviceInfo_FFI, SDKResult};
// use wooting_analog_common::{DeviceInfo, SDKResult};
use wooting_analog_wrapper as sdk;
extern crate ctrlc;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

fn main() {
    println!("Starting Wooting Analog SDK!");
    assert!(!sdk::is_initialised());
    let init_result: SDKResult<u32> = sdk::initialise();

    match init_result.0 {
        Ok(device_num) => {
            assert!(sdk::is_initialised());
            println!("SDK Successfully initialised with {} devices", device_num);
            use_sdk(device_num);
            println!("Finishing up...");
            sdk::uninitialise();
            assert!(!sdk::is_initialised());
        }
        Err(e) => {
            println!("SDK Failed to initialise. Error: {:?}", e);
        }
    }
}

extern "C" fn callback_handler(event: DeviceEventType, device: *mut DeviceInfo_FFI) {
    let device: DeviceInfo = unsafe { device.as_ref().unwrap().into_device_info() };

    println!("Event: {:?} on device: {:?}", event, device);
}

const DEVICE_BUFFER_MAX: usize = 5;
const ANALOG_BUFFER_READ_MAX: usize = 10;
fn use_sdk(device_num: u32) {
    sdk::set_device_event_cb(callback_handler);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        println!("");
    })
    .expect("Error setting Ctrl-C handler");

    // get_connected_devices_info() -> SDKResult<Vec<DeviceInfo>>
    let devices: Vec<DeviceInfo> = sdk::get_connected_devices_info(DEVICE_BUFFER_MAX)
        .0
        .unwrap();
    assert_eq!(device_num, devices.len() as u32);
    for (i, device) in devices.iter().enumerate() {
        println!("Device {} is {:?}", i, device);
    }

    let mut last_output = String::new();
    while running.load(Ordering::SeqCst) {
        let mut new_output: Option<String> = None;
        let read_result: SDKResult<HashMap<u16, f32>> =
            sdk::read_full_buffer(ANALOG_BUFFER_READ_MAX);
        match read_result.0 {
            Ok(analog_data) => {
                let mut sorted_data = analog_data.iter().collect::<Vec<(&u16, &f32)>>();
                sorted_data.sort_by_key(|x| x.0);
                // analog_data.
                let data_out = sorted_data
                    .iter()
                    .map(|(code, value)| format!("({},{})", code, value))
                    .fold(String::new(), |mut total, x| {
                        total.push_str(x.as_str());
                        total
                    });
                if !data_out.is_empty() {
                    new_output = Some(data_out);
                }
            }
            Err(e) => {
                new_output = Some(format!("Error reading full buffer, {:?}", e));
            }
        };

        if let Some(output) = new_output {
            if !last_output.eq(&output) {
                println!("{}", output);
                last_output = output;
            }
        }
    }
}
