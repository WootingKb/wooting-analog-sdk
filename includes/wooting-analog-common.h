/* This is a generated header file providing the common items to everything related to the Analog SDK */

/* Warning, this file is autogenerated by cbindgen. Don't modify this manually. */

#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef enum {
  WootingAnalog_DeviceEventType_Connected = 1,
  WootingAnalog_DeviceEventType_Disconnected,
} WootingAnalog_DeviceEventType;

typedef enum {
  WootingAnalog_KeycodeType_HID,
  WootingAnalog_KeycodeType_ScanCode1,
  WootingAnalog_KeycodeType_VirtualKey,
  WootingAnalog_KeycodeType_VirtualKeyTranslate,
} WootingAnalog_KeycodeType;

typedef enum {
  WootingAnalogResult_Ok = 1,
  WootingAnalogResult_UnInitialized = -2000,
  WootingAnalogResult_NoDevices,
  WootingAnalogResult_DeviceDisconnected,
  WootingAnalogResult_Failure,
  WootingAnalogResult_InvalidArgument,
  WootingAnalogResult_NoPlugins,
  WootingAnalogResult_FunctionNotFound,
  WootingAnalogResult_NoMapping,
  /**
   * Indicates that it isn't available on this platform
   */
  WootingAnalogResult_NotAvailable,
} WootingAnalogResult;

typedef uint64_t WootingAnalog_DeviceID;

/**
 * The core `DeviceInfo` struct which contains all the interesting information
 * for a particular device
 */
typedef struct {
  /**
   * Device Vendor ID `vid`
   */
  uint16_t vendor_id;
  /**
   * Device Product ID `pid`
   */
  uint16_t product_id;
  /**
   * Device Manufacturer name
   */
  const char *manufacturer_name;
  /**
   * Device name
   */
  const char *device_name;
  /**
   * Unique device ID, which should be generated using `generate_device_id`
   */
  WootingAnalog_DeviceID device_id;
} WootingAnalog_DeviceInfo;