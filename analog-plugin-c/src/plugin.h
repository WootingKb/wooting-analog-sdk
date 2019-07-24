#include <stdint.h>
#include <stdio.h>
#include <stdbool.h>

typedef uint64_t DeviceID;

typedef struct DeviceInfo {
    uint16_t vendor_id;
    uint16_t product_id;
    const char* manufacturer_name;
    const char* device_name;
    DeviceID device_id;
} DeviceInfo;

typedef enum {
    Connected = 1,
    Disconnected
} DeviceEventType;

typedef enum AnalogSDKError  {
    Ok = 1,
    UnInitialized = -2000,
    NoDevices,
    DeviceDisconnected,
    //Generic Failure
    Failure,
    InvalidArgument,
    NoPlugins,
    FunctionNotFound,
    //No Keycode mapping to HID was found for the given Keycode
    NoMapping

} AnalogSDKError;

typedef void(*device_event)(DeviceEventType, DeviceInfo*);

const char* c_name();
bool is_initialised();
AnalogSDKError initialise();
void unload();
int c_read_full_buffer(uint16_t code_buffer[], float analog_buffer[], int len, DeviceID device);
int c_device_info(DeviceInfo* buffer[], int len);
float read_analog(uint16_t code, DeviceID device);
AnalogSDKError set_device_event_cb(device_event cb);
AnalogSDKError clear_device_event_cb();