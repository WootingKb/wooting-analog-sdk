use wooting_analog_sdk::sdk::AnalogSDK;
use wooting_analog_sdk::sdk::AnalogSDKTest;

fn main() {
    // let mut sdk = AnalogSDK::new();
    // sdk.initialise().unwrap();

    let mut sdk = AnalogSDKTest::new().initialise();

    loop {
        let buffer = sdk.read_full_buffer(5, 0).0.unwrap();

        if !buffer.is_empty() {
            println!("{buffer:?}");
        }
    }
}