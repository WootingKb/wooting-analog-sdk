#include "wooting-analog-plugin-dev.h"
#include <stdbool.h>
#include <stdint.h>
#include <stdio.h>

#if defined(_WIN32) || defined(WIN32)
#ifdef ANALOGSDK_EXPORTS
#define ANALOGSDK_EXPORT __declspec(dllexport)
#else
#define ANALOGSDK_EXPORT __declspec(dllimport)
#endif
#else
#define ANALOGSDK_EXPORT
#endif

#ifdef __cplusplus
#define ANALOGSDK_API extern "C" ANALOGSDK_EXPORT
#else
#define ANALOGSDK_API ANALOGSDK_EXPORT
#endif

ANALOGSDK_API const uint32_t ANALOG_SDK_PLUGIN_ABI_VERSION = 1;

typedef void (*device_event)(void const *, WootingAnalog_DeviceEventType,
                             const WootingAnalog_DeviceInfo_FFI *);

/// Get a name describing the `Plugin`.
ANALOGSDK_API const char *name();

/// A callback fired immediately after the plugin is loaded. Usually used
/// for initialization.
ANALOGSDK_API int initialise(void const *callback_data, device_event callback);

/// A function fired to check if the plugin is currently initialised
ANALOGSDK_API bool is_initialised();

/// A callback fired immediately before the plugin is unloaded. Use this if
/// you need to do any cleanup. Any references to `callback_data` or `callback`
/// should be dropped as these will be invalidated after this function returns.
ANALOGSDK_API void unload();

/// Function called to get the full analog read buffer for a particular device
/// with ID `device`. `len` is the maximum amount of keys that can be accepted,
/// any more beyond this will be ignored by the SDK. If `device` is 0 then no
/// specific device is specified and the data should be read from all devices
/// and combined
ANALOGSDK_API int read_full_buffer(uint16_t code_buffer[],
                                    float analog_buffer[], int len,
                                    WootingAnalog_DeviceID device);

/// This function is fired by the SDK to collect up all Device Info structs. The
/// memory for the struct should be retained and only dropped when the device is
/// disconnected or the plugin is unloaded. This ensures that the Device Info is
/// not garbled when it's being accessed by the client.
///
/// # Notes
///
/// Although, the client should be copying any data they want to use for a
/// prolonged time as there is no lifetime guarantee on the data.
ANALOGSDK_API int device_info(const WootingAnalog_DeviceInfo_FFI *buffer[],
                               int len);

/// Function called to get the analog value for a particular HID key `code` from
/// the device with ID `device`. If `device` is 0 then no specific device is
/// specified and the value should be read from all devices and combined
ANALOGSDK_API float read_analog(uint16_t code, WootingAnalog_DeviceID device);