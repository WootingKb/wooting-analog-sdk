use std::path::PathBuf;

use wooting_analog_sdk::sdk::AnalogSDK;
use wooting_analog_sdk::sdk::AnalogSDKTest;

fn main() {
    // let mut sdk = AnalogSDK::new();
    // sdk.initialise().unwrap();

    println!("{:?}", std::env::current_dir().unwrap());

    let mut sdk = AnalogSDKTest::new()
        .with_plugin_directories([
            // "../target/debug",
            "./test",
            "test",
            "C:\\Program Files\\WootingAnalogPlugins",
        ])
        .initialise();

    for mut plugin in sdk.state.plugins.iter_mut() {
        println!("main: {:?}", plugin.name());
    }

    // loop {
    //     let buffer = sdk.read_full_buffer(5, 0).0.unwrap();

    //     if !buffer.is_empty() {
    //         println!("{buffer:?}");
    //     }
    // }
}
