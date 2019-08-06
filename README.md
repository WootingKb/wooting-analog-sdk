# Analog SDK

This repo contains all the core components of the Analog SDK.

NOTE: Use the `ANALOG_SDK_PLUGINS_PATH` environment variable to tell the SDK where to search for plugins.

## Components
* `analog-sdk`: The core Analog SDK which handles loading of plugins. This is installed systemwide and is updated separately
* `analog-sdk-common`: This library contains all common Analog SDK code, this is used by plugins and the SDK itself
* `analog-sdk-wrapper`: This is the SDK wrapper which is what Applications should use to communicate with the SDK. The linked dll should be shipped with the application using it.
* `analog-sdk-test`: This is a C# test application which can be used to test the SDK through the wrapper.

## Building 
### Build Dependencies
* [rust]()
* [cargo-make](https://github.com/sagiegurari/cargo-make)
* [cbindgen](https://github.com/eqrion/cbindgen) (Should be installed automatically through cargo make if necessary)
* [dotnet-core]() If you want to use `analog-sdk-test`


### How to Build
Everything can be built using this command. All the outputs will be under `target/debug`
```
cargo make
```

To run the test application:
```
cargo make test-app
```