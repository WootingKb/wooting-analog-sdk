# Introduction

If you wish to use the SDK, you should be dynamically linking to the `wooting-analog-wrapper` library and shipping it with your application. The way the SDK works is that it uses the wrapper to try and find the SDK at runtime, so it may gracefully error if the SDK isn’t found. The wrapper library just passes through SDK calls to the actual SDK, so if there is an SDK update with new features, you only need to update your wrapper when you wish to use them.

## Getting Started

To get started, make sure you have the SDK & Wooting plugin installed. Follow the installation instructions from the [SDK Readme](https://github.com/WootingKb/wooting-analog-sdk).

### C/C++

Then download & extract the `.tar.gz` for the platform you’re targeting from the [latest release](https://github.com/WootingKb/wooting-analog-sdk/releases). Inside the `$extract/wrapper` directory you’ll find the wrapper lib you should link to and all the headers you may need.

### Rust

Add the following to your crate's dependencies:

```TOML
wooting-analog-wrapper = { git = "https://github.com/WootingKb/wooting-analog-sdk" }
```

## Keycodes

By default the SDK will use the [USB HID codes (see table 10.6)](https://www.win.tue.nl/~aeb/linux/kbd/scancodes-10.html#scancodesets) to identify keys. This can be changed using the [Set Keycode Mode](#set-keycode-mode) function, which changes the keycodes taken by `read_analog` and the keycodes given in `read_full_buffer`. The available options are:

- `HID`: The standard USB HID codes (default) [List on table 10.6](https://www.win.tue.nl/~aeb/linux/kbd/scancodes-10.html#scancodesets)
- `ScanCode1`: Scan codes set 1, see [Set 1 column on table 10.6](https://www.win.tue.nl/~aeb/linux/kbd/scancodes-10.html#scancodesets) (Escape codes can be given as either a 0x1 or 0xE0 prefix)
- `VirtualKey`: [Windows Virtual Key codes](https://docs.microsoft.com/en-gb/windows/win32/inputdev/virtual-key-codes)
- `VirtualKeyTranslate`: [Windows Virtual Key codes](https://docs.microsoft.com/en-gb/windows/win32/inputdev/virtual-key-codes) but they are translated based on layout, so requesting the letter Q gets the key that inputs Q on the selected layout, rather than always getting the key right of tab (the standard Q position) like `VirtualKey` would.

## General API Notes

In the C API interface, every function has the possibility to return an error, even though some of the functions will return something like a `float` . In these cases the error will be returned cast as that type. All errors in the enum have been defined as negative numbers to prevent interference with the actual returns of the function. So any return from a function that's negative is an error, the particular error can be determined by comparing the number to the `WootingAnalogResult` enum.

Each function below has listed what the expected returns from the function are, however, there are some general cases, which are:

- `WootingAnalogResult::FunctionNotFound`: Indicates that the wrapper could not find the function in the SDK
- `WootingAnalogResult::DLLNotFound`: Indicates that the wrapper could not find the Analog SDK, which could be due to a corrupt or non-existent install of the SDK
- `WootingAnalogResult::IncompatibleVersion`: Indicates that the wrapper you're using is designed for a different major version of the SDK than what is installed. Major versions are unlikely to change very often

These apply to every function unless otherwise specified

## Initialisation

### Initialise

```c
WootingAnalogResult wooting_analog_initialise();
```

Initialises the Analog SDK, this needs to be successfully called before any other functions of the SDK can be called

### Expected Returns

- `ret>=0`: Meaning the SDK initialised successfully and the number indicates the number of devices that were found on plugin initialisation
- `WootingAnalogResult::NoPlugins`: Meaning that either no plugins were found or some were found but none were successfully initialised

### Is Initialised

```c
bool wooting_analog_is_initialised();
```

Returns a bool indicating if the Analog SDK has been initialised

### UnInitialise

```c
WootingAnalogResult wooting_analog_uninitialise();
```

Uninitialises the SDK, returning it to an empty state, similar to how it would be before first initialisation

### Expected Returns

- `WootingAnalogResult::Ok`: Indicates that the SDK was successfully uninitialised

## Get Connected Devices Info

```c
int wooting_analog_get_connected_devices_info(WootingAnalog_DeviceInfo_FFI **buffer,unsigned int len);
```

Fills up the given `buffer`(that has length `len`) with pointers to the DeviceInfo structs for all connected devices (as many that can fit in the buffer)

### Notes

- The memory of the returned structs will only be kept until the next call of this function, so if you wish to use any data from them, please copy it or ensure you don’t reuse references to old memory after calling this function again.

### Expected Returns

Similar to wooting_analog_read_analog, the errors and returns are encoded into one type. Values >=0 indicate the number of items filled into the buffer, with `<0` being of type WootingAnalogResult

- `ret>=0`: The number of connected devices that have been filled into the buffer
- `WootingAnalogResult::UnInitialized`: Indicates that the AnalogSDK hasn’t been initialised

## Set Keycode Mode

```c
WootingAnalogResult wooting_analog_set_keycode_mode(WootingAnalog_KeycodeType mode);
```

Sets the type of Keycodes the Analog SDK will receive (in `read_analog`) and output (in `read_full_buffer`).

By default, the mode is set to HID

### Notes

- `VirtualKey` and `VirtualKeyTranslate` are only available on Windows
- With all modes except `VirtualKeyTranslate`, the key identifier will point to the physical key on the standard layout. i.e. if you ask for the Q key, it will be the key right to tab regardless of the layout you have selected
- With `VirtualKeyTranslate`, if you request Q, it will be the key that inputs Q on the current layout, not the key that is Q on the standard layout.

### Expected Returns

- `WootingAnalogResult::Ok`: The Keycode mode was changed successfully
- `WootingAnalogResult::InvalidArgument`: The given `KeycodeType` is not one supported by the SDK
- `WootingAnalogResult::NotAvailable`: The given `KeycodeType` is present, but not supported on the current platform
- `WootingAnalogResult::UnInitialized`: The SDK is not initialised

## Device Event Callback

### Set

```c
WootingAnalogResult wooting_analog_set_device_event_cb(void (*cb)(WootingAnalog_DeviceEventType, WootingAnalog_DeviceInfo_FFI*));
```

Set the callback which is called when there is a DeviceEvent. Currently these events can either be Disconnected or Connected(Currently not properly implemented). The callback gets given the type of event `DeviceEventType` and a pointer to the DeviceInfo struct that the event applies to

### Notes

- You must copy the DeviceInfo struct or its data if you wish to use it after the callback has completed, as the memory will be freed straight after
- The execution of the callback is performed in a separate thread so it is fine to put time consuming code and further SDK calls inside your callback

### Expected Returns

- `WootingAnalogResult::Ok`: The callback was set successfully
- `WootingAnalogResult::UnInitialized`: The SDK is not initialised

### Clear

```c
WootingAnalogResult wooting_analog_clear_device_event_cb();
```

Clears the device event callback that has been set

### Expected Returns

- `WootingAnalogResult::Ok`: The callback was cleared successfully
- `WootingAnalogResult::UnInitialized`: The SDK is not initialised

## Read Single Analog value

```c
float wooting_analog_read_analog(unsigned short code);
float wooting_analog_read_analog_device(unsigned short code, WootingAnalog_DeviceID device_id);
```

Reads the Analog value of the key with identifier `code` from any connected device (or the device with id `device_id` if specified). The set of key identifiers that is used depends on the Keycode mode set using [Set Keycode Mode](#set-keycode-mode).

The `device_id` can be found through calling [Get Device Info](#get-connected-devices-info) and getting the DeviceID from one of the DeviceInfo structs

### Examples

```
wooting_analog_set_mode(KeycodeType::ScanCode1);
wooting_analog_read_analog(0x10); //This will get you the value for the key which is Q in the standard US layout (The key just right to tab)
wooting_analog_set_mode(KeycodeType::VirtualKey); //This will only work on Windows
wooting_analog_read_analog(0x51); //This will get you the value for the key that is Q on the standard layout
wooting_analog_set_mode(KeycodeType::VirtualKeyTranslate); //Also will only work on Windows
wooting_analog_read_analog(0x51); //This will get you the value for the key that inputs Q on the current layout
```

### Expected Returns

The float return value can be either a 0->1 analog value, or (if < 0) is part of the `WootingAnalogResult` enum, which is how errors are given back on this call.

So if the value is below 0, you should cast it as `WootingAnalogResult` to see what the error is.

- `0.0f - 1.0f`: The Analog value of the key with the given id `code` from device with id `device_id`(if specified)
- `WootingAnalogResult::NoMapping`: No keycode mapping was found from the selected mode (set by [Set Keycode Mode](about:blank#set-keycode-mode)) and HID.
- `WootingAnalogResult::UnInitialized`: The SDK is not initialised
- `WootingAnalogResult::NoDevices`: There are no connected devices (with id `device_id` if specified)

## Read All Analog values

```c
int wooting_analog_read_full_buffer(unsigned short *code_buffer,float *analog_buffer,unsigned int len);
int wooting_analog_read_full_buffer_device(unsigned short *code_buffer,float *analog_buffer,unsigned int len,WootingAnalog_DeviceID device_id);
```

Reads all the analog values for pressed keys for all devices and combines their values (or reads from a single device with id `device_id` [if specified]), filling up `code_buffer` with the keycode identifying the pressed key and fills up `analog_buffer` with the corresponding float analog values. i.e. The analog value for they key at index 0 of code_buffer, is at index 0 of analog_buffer.

### Notes

- `len` is the length of code_buffer & analog_buffer, if the buffers are of unequal length, then pass the lower of the two, as it is the max amount of key & analog value pairs that can be filled in.
- The codes that are filled into the `code_buffer` are of the KeycodeType set with wooting_analog_set_mode
- If two devices have the same key pressed, the greater value will be given (if no `device_id` has been given)
- When a key is released it will be returned with an analog value of 0.0f in the first `read_full_buffer` call after the key has been released

### Expected Returns

Similar to other functions like [Get Device Info](#get-connected-devices-info), the return value encodes both errors and the return value we want. Where >=0 is the actual return, and < 0 should be cast as WootingAnalogResult to find the error.

- `>=0` means the value indicates how many keys & analog values have been read into the buffers
- `WootingAnalogResult::UnInitialized`: Indicates that the AnalogSDK hasn’t been initialised
- `WootingAnalogResult::NoDevices`: Indicates no devices are connected (or that there is no device with id `device_id` [if specified])

# Structures and Enums

## DeviceEventType

```c
typedef enum {
	/**
	* Device has been connected
	*/
	WootingAnalog_DeviceEventType_Connected = 1,
	/**
	* Device has been disconnected
	*/
	WootingAnalog_DeviceEventType_Disconnected,
} WootingAnalog_DeviceEventType;
```

## DeviceType

```c
typedef enum {
	/**
	* Device is of type Keyboard
	*/
	WootingAnalog_DeviceType_Keyboard = 1,
	/**
	* Device is of type Keypad
	*/
	WootingAnalog_DeviceType_Keypad,
	/**
	* Device
	*/
	WootingAnalog_DeviceType_Other,
} WootingAnalog_DeviceType;
```

## KeycodeType

```c
typedef enum {
	/**
	* USB HID Keycodes https://www.usb.org/document-library/hid-usage-tables-112 pg53
	*/
	WootingAnalog_KeycodeType_HID,
	/**
	* Scan code set 1
	*/
	WootingAnalog_KeycodeType_ScanCode1,
	/**
	* Windows Virtual Keys
	*/
	WootingAnalog_KeycodeType_VirtualKey,
	/**
	* Windows Virtual Keys which are translated to the current keyboard locale
	*/
	WootingAnalog_KeycodeType_VirtualKeyTranslate,
} WootingAnalog_KeycodeType;
```

## WootingAnalogResult

```c
typedef enum {
	WootingAnalogResult_Ok = 1,
	/**
	* Item hasn't been initialized
	*/
	WootingAnalogResult_UnInitialized = -2000,
	/**
	* No Devices are connected
	*/
	WootingAnalogResult_NoDevices,
	/**
	* Device has been disconnected
	*/
	WootingAnalogResult_DeviceDisconnected,
	/**
	* Generic Failure
	*/
	WootingAnalogResult_Failure,
	/**
	* A given parameter was invalid
	*/
	WootingAnalogResult_InvalidArgument,
	/**
	* No Plugins were found
	*/
	WootingAnalogResult_NoPlugins,
	/**
	* The specified function was not found in the library
	*/
	WootingAnalogResult_FunctionNotFound,
	/**
	* No Keycode mapping to HID was found for the given Keycode
	*/
	WootingAnalogResult_NoMapping,
	/**
	* Indicates that it isn't available on this platform
	*/
	WootingAnalogResult_NotAvailable,
	/**
	* Indicates that the operation that is trying to be used is for an older version
	*/
	WootingAnalogResult_IncompatibleVersion,
} WootingAnalogResult;
```

## DeviceInfo

```c
/**
* The core `DeviceInfo` struct which contains all the interesting information
* for a particular device. This is the version which the consumer of the SDK will receive
* through the wrapper.
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
	char *manufacturer_name;
	/**
	* Device name
	*/
	char *device_name;
	/**
	* Unique device ID, which should be generated using `generate_device_id`
	*/
	WootingAnalog_DeviceID device_id;
	/**
	* Hardware type of the Device
	*/
	WootingAnalog_DeviceType device_type;
} WootingAnalog_DeviceInfo_FFI;
```
