# Wooting Analog SDK Plugins

## General Info

The Analog SDK can accept plugins created using Rust, C or anything that provides the defined C ABI. Rust is the recommended choice, but we decided to ensure support for C plugins as not everyone is going to be able/want to use Rust.

The purpose of Plugins are to add support for new Devices through the Analog SDK, exposing an interface the SDK can use to read analog key information from the device.

The SDK uses the `WOOTING_ANALOG_SDK_PLUGINS_PATH` environment variable to search for plugins, this is a semi-colon separated list of directories. So if you've created a plugin, you should add the build output directory to the path for development/testing and the install directory of the plugin for deployment.

Plugins are required to statically link to `wooting-analog-common` as it includes the `ANALOG_SDK_PLUGIN_ABI_VERSION` constant, which ends up being exported in the plugin and tells the SDK which version of the plugin interface your plugin is using. This is so that if breaking changes are made to the interface, backwards compatibility can be made for older plugins.

## A note about custom keys

If your device has keys which are not defined in the HID standard keys, then you should output a number with prefix of 0x2 or higher, excluding 0xE0. e.g. 0x0201 would be a custom key. These numbers will not be converted into different keycode sets.

## Plugin Requirements

### Rust

Rust Plugins are fairly straight forward to get started with, have a look at the [wooting plugin](https://github.com/simon-wh/wooting-analog-sdk-plugin) for a reference implementation:

* Rust library with crate-type `cdylib`
* Dependency to `wooting-analog-common`. (Currently while in the Alpha stage, this should be done by adding the SDK repo as a submodule and adding a relative reference to the library)
* A struct that implements the `Plugin` trait from `wooting-analog-common`
* Declare the plugin using the `declare_plugin!` macro. e.g. `declare_plugin!(ExamplePlugin, ExamplePlugin::new)`

### C

Have a look at the [example c plugin](https://github.com/simon-wh/analog-sdk-plugin-examples) for a reference of what should be done.

* The library must define the functions from `includes/plugin.h`, use it as the header for your source file
* The library should statically link to `wooting_analog_common`, using `wooting-analog-plugin-dev.h`(which is included in `plugin.h`) to call shared functions such as `generate_device_id`.

An important thing to note with c plugins, is that for functions like `read_analog`, which returns only a float, errors in the form of WootingAnalogResult should be returned, cast as a float. The same as how the errors are passed through from the SDK to the developer.