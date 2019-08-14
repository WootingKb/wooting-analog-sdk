# Wooting Analog SDK Plugins

## General Info

The Analog SDK can accept plugins created using Rust, C or anything that provides the defined C ABI. Rust is the recommended choice, but we decided to ensure support for C plugins as not everyone is going to be able/want to use Rust.

The purpose of Plugins are to add support for new Devices through the Analog SDK, exposing an interface the SDK can use to read analog key information from the device.

The SDK uses the `WOOTING_ANALOG_SDK_PLUGINS_PATH` environment variable to search for plugins, this is a semi-colon separated list of directories. So if you've created a plugin, you should add the build output directory to the path for development/testing and the install directory of the plugin for deployment.

## Rust Plugins

Rust Plugins are fairly straight forward to get started with, have a look at the [wooting plugin](https://github.com/simon-wh/wooting-analog-sdk-plugin) for a reference implementation:

* Create a library with crate-type `cdylib`
* Add a dependency to `wooting-analog-sdk-common`. (Currently while in the Alpha stage, this should be done by adding the SDK repo as a submodule and adding a relative reference to the library)
* Create a struct that implements the `Plugin` trait from `wooting-analog-sdk-common`
* Declare the plugin using the `declare_plugin!` macro. e.g. `declare_plugin!(ExamplePlugin, ExamplePlugin::new)`

## C Plugins

