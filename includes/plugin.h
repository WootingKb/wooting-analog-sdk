#include <stdint.h>
#include <stdio.h>
#include <stdbool.h>
#include "wooting-analog-sdk-common-plugin.h"

#if defined(_WIN32) || defined(WIN32)
#ifdef ANALOGSDK_EXPORTS
#define ANALOGSDK_API __declspec(dllexport)
#else
#define ANALOGSDK_API __declspec(dllimport)
#endif
#else
#define ANALOGSDK_API
#endif

typedef void(*device_event)(WASDK_DeviceEventType, WASDK_DeviceInfo*);

ANALOGSDK_API const char* _name();
ANALOGSDK_API bool is_initialised();
ANALOGSDK_API AnalogSDKResult initialise();
ANALOGSDK_API void unload();
ANALOGSDK_API int _read_full_buffer(uint16_t code_buffer[], float analog_buffer[], int len, WASDK_DeviceID device);
ANALOGSDK_API int _device_info(WASDK_DeviceInfo* buffer[], int len);
ANALOGSDK_API float read_analog(uint16_t code, WASDK_DeviceID device);
ANALOGSDK_API AnalogSDKResult set_device_event_cb(device_event cb);
ANALOGSDK_API AnalogSDKResult clear_device_event_cb();