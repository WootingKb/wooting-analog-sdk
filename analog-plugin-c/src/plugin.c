#include "plugin.h"
#include "string.h"
#include "hidapi.h"


#define ANALOG_BUFFER_SIZE 48
#define WOOTING_ONE_VID 0x03EB
#define WOOTING_ONE_PID 0xFF01
#define WOOTING_ONE_ANALOG_USAGE_PAGE 0x1338

static hid_device* keyboard_handle = NULL;
static device_event callback = NULL;
static unsigned char hid_read_buffer[ANALOG_BUFFER_SIZE];

static char device_name[20];
static char manufacturer_name[20];

static DeviceInfo dev_info;
static bool initialised = false;

const char* c_name(){
    return "C Test Plugin";
}

bool is_initialised() {
    return initialised;
}

static void wooting_keyboard_disconnected() {
    hid_close(keyboard_handle);
    keyboard_handle = NULL;

    if (callback) {
        callback(Disconnected, &dev_info);
    }
    initialised = false;
}

static bool wooting_find_keyboard() {
    struct hid_device_info* hid_info = hid_enumerate(WOOTING_ONE_VID, WOOTING_ONE_PID);

    if (hid_info == NULL) {
        return false;
    }

    // The amount of interfaces is variable, so we need to look for the analog interface
    // In the Wooting one keyboard the analog interface is always the highest number
    struct hid_device_info* hid_info_walker = hid_info;
    uint8_t interfaceNr = 0;
    while (hid_info_walker) {
        if (hid_info_walker->interface_number > interfaceNr) {
            interfaceNr = hid_info_walker->interface_number;
        }
        hid_info_walker = hid_info_walker->next;
    }

    bool keyboard_found = false;
    // Reset walker to top and search for the interface number
    hid_info_walker = hid_info;
    while (hid_info_walker) {
        if (hid_info_walker->interface_number == interfaceNr) {
            keyboard_handle = hid_open_path(hid_info_walker->path);
            if (keyboard_handle) {
                keyboard_found = true;
            }

            break;
        }

        hid_info_walker = hid_info_walker->next;
    }
    if (keyboard_found) {
        dev_info.vendor_id = hid_info->vendor_id;
        dev_info.product_id = hid_info->product_id;
        sprintf(device_name, "%s", hid_info->product_string );
        dev_info.device_name = device_name;
        sprintf(manufacturer_name, "%s", hid_info->manufacturer_string );
        dev_info.manufacturer_name = manufacturer_name;
        //TODO: Hash a proper device id, also figure out how to keep consistency across plugins for device ids
        dev_info.device_id = hid_info->interface_number;
    }

    hid_free_enumeration(hid_info);
    return keyboard_found;
}

AnalogSDKError initialise() {
    if (initialised)
        return Ok;

    return initialised = wooting_find_keyboard();
}

void unload() {

}

static bool wooting_refresh_buffer() {
    if (!keyboard_handle) {
        if (!wooting_find_keyboard()) {
            return false;
        }
    }

    int hid_res = hid_read_timeout(keyboard_handle, hid_read_buffer, ANALOG_BUFFER_SIZE, 0);

    // If the read response is -1 the keyboard is disconnected
    if (hid_res == -1) {
        wooting_keyboard_disconnected();
        return false;
    }
    else {
        return true;
    }
}

int c_read_full_buffer(uint16_t code_buffer[], float analog_buffer[], int len, DeviceID device) {
    if (!initialised)
        return (float)UnInitialized;

    if (device != 0 && dev_info.device_id != device)
        return (float)NoDevices;

    if (!wooting_refresh_buffer())
        return (float)DeviceDisconnected;


    int items_written = 0;
    int read_length = len;

    // Cap elements to read
    if (len > ANALOG_BUFFER_SIZE) {
        read_length = ANALOG_BUFFER_SIZE/3;
    }

    for (int i = 0; i < read_length*3; i += 3) {
        uint16_t code = (hid_read_buffer[i] << 8) | hid_read_buffer[i+1];
        uint8_t analog_value = hid_read_buffer[i + 2];

        if (analog_value > 0) {
            code_buffer[items_written] = code;

            // Cap out values to a maximum
            if (analog_value > 225) {
                analog_value = 255;
            }
            analog_buffer[items_written] = (float)analog_value / 255.0;

            items_written++;
        }
        else {
            // There will be no other keys once an analog value is 0
            return items_written;
        }
    }

    return items_written;
}

float read_analog(uint16_t code, DeviceID device) {
    if (!initialised)
        return (float)UnInitialized;

    if (device != 0 && dev_info.device_id != device)
        return (float)NoDevices;

    if (!wooting_refresh_buffer())
        return (float)DeviceDisconnected;



    for (int i = 0; i < ANALOG_BUFFER_SIZE && hid_read_buffer[i+2] > 0; i += 3) {
        uint16_t read_code = (hid_read_buffer[i] << 8) | hid_read_buffer[i+1];
        if (read_code == code) {
            // Cap out values to a maximum
            float val = ((float)hid_read_buffer[i+2] * 1.2) / (float)255;
            if (val > 1.0)
                val = 1.0;
            return val;
        }
    }

    return 0.0;
}

int c_device_info(DeviceInfo* buffer[], int len) {
    if (!initialised)
        return UnInitialized;

    buffer[0] = &dev_info;
    return 1;
}

AnalogSDKError set_device_event_cb(device_event cb) {
    callback = cb;
}

AnalogSDKError clear_device_event_cb() {
    callback = NULL;
}