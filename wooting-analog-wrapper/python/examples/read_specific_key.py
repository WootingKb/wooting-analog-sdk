import wooting_analog_wrapper
import sys

key = wooting_analog_wrapper.HIDCodes.A

def main():
    wooting_analog_wrapper.initialise()

    print(wooting_analog_wrapper.get_connected_devices_info(5))

    last_analog = -1
    has_error = False
    while True:
        analog = 0
        try:
            analog = wooting_analog_wrapper.read_analog(key)
            if last_analog != analog:
                print("{0:X}: {1:.2f} ".format(key, analog))
                last_analog = analog

            has_error = False
        except KeyboardInterrupt:
            break
        except:
            if not has_error:
               print("Error reading analog value:", sys.exc_info()[0])
               has_error = True

    wooting_analog_wrapper.uninitialise()

main()