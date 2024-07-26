#include "../../../includes/plugin.h"

static bool initialised = false;
static WootingAnalog_DeviceInfo_FFI deviceInfo;

/// Get a name describing the `Plugin`.
const char *name() { return "C Test plugin"; }

static void const *cb_data;
static device_event cb;

/// A callback fired immediately after the plugin is loaded. Usually used
/// for initialization.
int initialise(void const *callback_data, device_event callback) {

  cb_data = callback_data;
  cb = callback;

  initialised = true;
  // deviceInfo = new_device_info(5,6, "Yeet", "Yeet", 7,
  // WootingAnalog_DeviceType_Keyboard);
  deviceInfo.vendor_id = 5;
  deviceInfo.product_id = 6;
  deviceInfo.device_name = "Yeet";
  deviceInfo.manufacturer_name = "Yeet";
  deviceInfo.device_id = 7;
  deviceInfo.device_type = WootingAnalog_DeviceType_Keyboard;

  return 1;
}

/// A function fired to check if the plugin is currently initialised
bool is_initialised() { return initialised; }

/// A callback fired immediately before the plugin is unloaded. Use this if
/// you need to do any cleanup.
void unload() {
  initialised = false;
  cb_data = NULL;
  cb = NULL;
}

/// Function called to get the full analog read buffer for a particular device
/// with ID `device`. `len` is the maximum amount of keys that can be accepted,
/// any more beyond this will be ignored by the SDK. If `device` is 0 then no
/// specific device is specified and the data should be read from all devices
/// and combined
int read_full_buffer(uint16_t code_buffer[], float analog_buffer[], int len,
                      WootingAnalog_DeviceID device) {
  code_buffer[0] = 5;
  analog_buffer[0] = 0.4f;
  return 1;
}

/// This function is fired by the SDK to collect up all Device Info structs. The
/// memory for the struct should be retained and only dropped when the device is
/// disconnected or the plugin is unloaded. This ensures that the Device Info is
/// not garbled when it's being accessed by the client.
///
/// # Notes
///
/// Although, the client should be copying any data they want to use for a
/// prolonged time as there is no lifetime guarantee on the data.
int device_info(const WootingAnalog_DeviceInfo_FFI *buffer[], int len) {
  buffer[0] = &deviceInfo;
  return 1;
}

/// Function called to get the analog value for a particular HID key `code` from
/// the device with ID `device`. If `device` is 0 then no specific device is
/// specified and the value should be read from all devices and combined
float read_analog(uint16_t code, WootingAnalog_DeviceID device) {
  printf("Calling cb, cb: %p, cb_data: %p, devInfo: %p\n", cb, cb_data,
         &deviceInfo);
  if (cb != NULL && cb_data != NULL) {
    cb(cb_data, WootingAnalog_DeviceEventType_Connected, &deviceInfo);
  } else {
    printf("Attempted to execute a NULL callback\n");
  }

  return 0.56f;
}