#include "../../../includes/plugin.h"

static bool initialised = false;
static WootingAnalog_DeviceInfo deviceInfo;


/// Get a name describing the `Plugin`.
ANALOGSDK_API const char* _name() {
    return "C Test plugin";
}



/// A callback fired immediately after the plugin is loaded. Usually used
/// for initialization.
ANALOGSDK_API WootingAnalogResult _initialise(device_event cb) {
    initialised = true;
    deviceInfo.vendor_id = 5;
    deviceInfo.product_id = 6;
    deviceInfo.device_id = 7;
    deviceInfo.device_name = "Yeet";
    deviceInfo.device_type = WootingAnalog_DeviceType_Keyboard;
    return 1;
}

/// A function fired to check if the plugin is currently initialised
ANALOGSDK_API bool is_initialised(){
    return initialised;
}

/// A callback fired immediately before the plugin is unloaded. Use this if
/// you need to do any cleanup.
ANALOGSDK_API void unload() {

}

/// Function called to get the full analog read buffer for a particular device with ID `device`. `len` is the maximum amount
/// of keys that can be accepted, any more beyond this will be ignored by the SDK.
/// If `device` is 0 then no specific device is specified and the data should be read from all devices and combined
ANALOGSDK_API int _read_full_buffer(uint16_t code_buffer[], float analog_buffer[], int len, WootingAnalog_DeviceID device){
    code_buffer[0] = 5;
    analog_buffer[0] = 0.4f;
    return 1;
}

/// This function is fired by the SDK to collect up all Device Info structs. The memory for the struct should be retained and only dropped
/// when the device is disconnected or the plugin is unloaded. This ensures that the Device Info is not garbled when it's being accessed by the client.
///
/// # Notes
///
/// Although, the client should be copying any data they want to use for a prolonged time as there is no lifetime guarantee on the data.
ANALOGSDK_API int _device_info(WootingAnalog_DeviceInfo* buffer[], int len) {
    buffer[0] = &deviceInfo;
    return 1;
}

/// Function called to get the analog value for a particular HID key `code` from the device with ID `device`.
/// If `device` is 0 then no specific device is specified and the value should be read from all devices and combined
ANALOGSDK_API float read_analog(uint16_t code, WootingAnalog_DeviceID device) {
    return 0.56f;
}