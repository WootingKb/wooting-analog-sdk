//use analog_sdk::*;
use analog_sdk::sdk::*;

fn main() { 
    let mut _pm = AnalogSDK::new();
    //let working_dir = std::env::current_dir().unwrap();
    _pm.initialise();
    /*unsafe {
        match _pm.load_plugins(&working_dirr.join("plugins")) {
            Ok(i) => println!("Loaded {} plugins", i),
            Err(e) => println!("Error: {}", e)
        }
    }*/
    println!("Yo, yo, yo: 9+10={:?}", _pm.add(9,10));
}