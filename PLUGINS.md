# Wooting Analog SDK Plugins

## General Info

The Analog SDK can accept plugins created using Rust, C or anything that provides the defined C ABI. Rust is the recommended choice, but we decided to ensure support for C plugins as not everyone is going to be able/want to use Rust.

The purpose of Plugins are to add support for new Devices through the Analog SDK, exposing an interface the SDK can use to read analog key information from the device.

Plugins must be located in a subdirectory of `WootingAnalogPlugins`. Which can be found in these places on each platform:

| OS      | Plugins Directory                        |
|---------|------------------------------------------|
| Windows | `C:\Program Files\WootingAnalogPlugins\` |
| Linux   | `/usr/local/share/WootingAnalogPlugins/` |
| Mac     | `/Library/WootingAnalogPlugins/`         |

So an example path on Windows would be:
```
C:\Program Files\WootingAnalogPlugins\wooting-analog-plugin\wooting_analog_plugin.dll
```

Plugins are required to statically link to `wooting-analog-plugin-dev` as it includes the `ANALOG_SDK_PLUGIN_ABI_VERSION` constant, which ends up being exported in the plugin and tells the SDK which version of the plugin interface your plugin is using. This is so that if breaking changes are made to the interface, backwards compatibility can be made for older plugins. (This is subject to change, we're considering using this variable to store the semver of the plugin-dev crate to have more information about the exact ABI of the loaded plugin)

## A note about custom keys

If your device has keys which are not defined in the HID standard keys, then you should output a number with prefix of 0x2 or higher, excluding 0xE0. e.g. 0x0201 would be a custom key. These numbers will not be converted into different keycode sets.

## Plugin Requirements

### Rust

Rust Plugins are fairly straight forward to get started with, have a look at the [wooting plugin](https://github.com/simon-wh/wooting-analog-sdk-plugin) for a reference implementation:

* Rust library with crate-type `cdylib`
* Add a dependency to [`wooting-analog-plugin-dev` (crates.io)](https://crates.io/crates/wooting-analog-plugin-dev)
* Import all relevant items through `use wooting_analog_plugin_dev::` and `use wooting_analog_plugin_dev::wooting_analog_common::`
* A struct that implements the `Plugin` trait from `wooting-analog-plugin-dev`
* Declare the plugin using the `declare_plugin!` macro. e.g. `declare_plugin!(ExamplePlugin, ExamplePlugin::new)`

### C

Have a look at the [example c plugin](https://github.com/simon-wh/analog-sdk-plugin-examples) for a reference of what should be done.

* The library must define the functions from `includes/plugin.h`, use it as the header for your source file
* The library should statically link to `wooting_analog_common`, using `wooting-analog-plugin-dev.h`(which is included in `plugin.h`) to call shared functions such as `generate_device_id`.

An important thing to note with c plugins, is that for functions like `read_analog`, which returns only a float, errors in the form of WootingAnalogResult should be returned, cast as a float. The same as how the errors are passed through from the SDK to the developer.