## Virtual Keyboard

The SDK includes a 'Virtual Keyboard' app which will emulate an Analog Keyboard and allows you to
test with the Analog SDK without needing a keyboard. To use this, ensure you have the
`debug/wooting-analog-sdk_dist` installed.

You need to have your game/app running and polling for keyboard state before you can hook the
virtual keyboard into your process. Only the debug versions of the SDK support using the virtual keyboard.

You can get the virtual keyboard by downloading the `.tar.gz` archive for your platfrom from the
[latest release](https://github.com/WootingKb/wooting-analog-sdk/releases) and find the
`wooting-analog-virtual-control` executable.