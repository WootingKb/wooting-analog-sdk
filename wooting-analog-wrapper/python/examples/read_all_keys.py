import wooting_analog_wrapper
import sys

def main():
    wooting_analog_wrapper.initialise()

    print(wooting_analog_wrapper.get_connected_devices_info(5))

    last_str = ""
    while True:
        str = ""
        try:
            buff = wooting_analog_wrapper.read_full_buffer(20)
            # We need to sort the buffer items so that when multiple keys are pressed that they remain in the same position in the string
            for key, value in sorted(buff.items()):
                str += "{0:X}: {1:.2f} ".format(key, value)           

            # This allows us to break out of the script by holding Ctrl + E
            # if buff.get(wooting_analog_wrapper.HIDCodes.LeftCtrl) != None and buff.get(wooting_analog_wrapper.HIDCodes.E) != None:
            #     break;
        except KeyboardInterrupt:
            break
        except:
               str = "Error reading buffer:", sys.exc_info()[0]
        finally:
             if last_str != str:
                print(str)
                last_str = str

    wooting_analog_wrapper.uninitialise()

main()