> [!INFO] Starting from `v0.9.0` the SDK has had a major project structure overhaul, the
functionality is still relatively similar.

## Migration guide: `<=0.8.0` to `0.9.0`

The most important breaking changes is that `wooting_analog_wrapper` no longer exists. The SDK
should be bundled and shipped with your app using `wooting_analog_sdk_dist` instead.

Your app should always bundle `wooting_analog_sdk_dist`. The distributable version of the SDK will
then delegate calls to any locally installed versions of the SDK if they are compatible.

The [virtual keyboard](./VIRTUAL_KEYBOARD.md) now only works on development builds of the SDK. When
developing use `debug/wooting_analog_sdk_dist`, the virtual keyboard can then hook into your running
process. On release of your game/app replace it with `release/wooting_analog_sdk_dist`.

- Replace the wooting_analog_wrapper shared library with the wooting_analog_sdk shared library:
    - Windows: `wooting_analog_wrapper.dll` -> `wooting_analog_sdk_dist.dll`.
    - Linux: `wooting_analog_wrapper.so` -> `wooting_analog_sdk_dist.so`.
    - MacOS: `wooting_analog_wrapper.dylib` -> `wooting_analog_sdk_dist.dylib`.
- If the import library is used on windows change it from `wooting_analog_wrapper.dll.lib` to 
  `wooting_analog_sdk_dist.dll.lib`.
- When calling from C/C++: change the `#include "wooting_analog_wrapper.h"` to 
  `#include "wooting_analog_sdk.h"`.
- [Reinstall](./INSTALL.md) the release version of the SDK:
    - Windows: `C:\Program Files\wooting-analog-sdk`
    - Linux: `/usr/lib`
    - MacOS: `/usr/local/lib`
