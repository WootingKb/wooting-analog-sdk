import wooting_analog_wrapper

def main():
    wooting_analog_wrapper.initialise()

    devices = wooting_analog_wrapper.get_connected_devices_info(5)
    for i, device in enumerate(devices):
        print(f"Device {i}: {device}")
        

    print(f"{len(devices)} total devices")

    wooting_analog_wrapper.uninitialise()

main()