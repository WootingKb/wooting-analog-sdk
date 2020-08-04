[![Build Status](https://travis-ci.com/WootingKb/wooting-analog-sdk.svg?branch=develop)](https://travis-ci.com/WootingKb/wooting-analog-sdk)
[![wooting-analog-common Crates.io](https://img.shields.io/crates/v/wooting-analog-common?label=crates.io%20wooting-analog-common)](https://crates.io/crates/wooting-analog-common)
[![wooting-analog-plugin-dev Crates.io](https://img.shields.io/crates/v/wooting-analog-plugin-dev?label=crates.io%20wooting-analog-plugin-dev)](https://crates.io/crates/wooting-analog-plugin-dev)
[![Documentation](https://img.shields.io/badge/Docs-Docs-green)](https://dev.wooting.nl/wooting-analog-sdk-guide/introduction/)

# Wooting Analog SDK

This repo contains all the core cross-platform components of the Wooting Analog SDK. The SDK and most of the components are built on Rust and should run on Windows, Mac and Linux, following the same steps for each platform unless otherwise specified.

## Installing

### Windows

On Windows the SDK & Wooting Plugin will be installed & updated automatically through Wootility (>= v3.4). If you wish to install manually, download the latest `.msi` from the [latest release](https://github.com/WootingKb/wooting-analog-sdk/releases)

### Linux

On Linux the primarily installation method is the `deb` package, which includes both the SDK and the Wooting Plugin, which can be found on the [latest release](https://github.com/WootingKb/wooting-analog-sdk/releases)

To install manually:

- Download & Extract the [latest release](https://github.com/WootingKb/wooting-analog-sdk/releases) `wooting-analog-sdk-v*.*.*-x86_64-unknown-linux-gnu.tar.gz`
- Copy `$extract/wrapper/sdk/libwooting_analog_sdk.so` to `/usr/lib`. (Or to some directory and add that path to the `LD_LIBRARY_PATH` environment variable)
- Follow the installation instructions for the [Wooting Analog Plugin](https://github.com/WootingKb/wooting-analog-plugin)

### Mac

Currently there is no installer available for Mac, so you will have to install manually.

- Download & Extract the [latest release](https://github.com/WootingKb/wooting-analog-sdk/releases) `wooting-analog-sdk-v*.*.*-x86_64-apple-darwin.tar.gz`
- Copy `$extract/wrapper/sdk/libwooting_analog_sdk.dylib` to `/usr/local/lib`. (Or to some directory and add that path to the `DYLD_LIBRARY_PATH` environment variable)
- Additionally, you may need to adjust security settings for OSX to let it run. For [reference](https://github.com/hashicorp/terraform/issues/23033#issuecomment-542302933)
- Follow the installation instructions for the [Wooting Analog Plugin](https://github.com/WootingKb/wooting-analog-plugin)

## Plugins

This SDK uses Plugins to provide support for Analog hardware, these must be located in a subdirectory of `WootingAnalogPlugins`. Which can be found in these places on each platform:

| OS      | Plugins Directory                        |
| ------- | ---------------------------------------- |
| Windows | `C:\Program Files\WootingAnalogPlugins\` |
| Linux   | `/usr/local/share/WootingAnalogPlugins/` |
| Mac     | `/usr/local/share/WootingAnalogPlugins/` |

So an example path on Windows would be:

```
C:\Program Files\WootingAnalogPlugins\wooting-analog-plugin\wooting_analog_plugin.dll
```

## Documentation

The core documentation can be found on [dev.wooting.io](https://dev.wooting.nl/wooting-analog-sdk-guide/introduction/).

Additionally, some of the crucial docs can be found in [SDK usage](SDK_USAGE.md) for a guide on how to use the SDK and the [Plugin introduction](PLUGINS.md) for information on creating plugins.

## Virtual Keyboard

The SDK includes a 'Virtual Keyboard' app which will emulate an Analog Keyboard and allows you to test with the Analog SDK without needing a keyboard. To use this, ensure you have the `wooting-analog-test-plugin` installed, on windows the installer allows you to choose if you want to install the feature. On Linux it is currently installed automatically with the `deb` package.
If you wish to install it otherwise, you can find it in the `.tar.gz` for your platform from the [latest release](https://github.com/WootingKb/wooting-analog-sdk/releases) under `$extract/wrapper/sdk/{lib}wooting_analog_test_plugin.{dll/so/dylib}`, install it as discussed above in the [Plugins section](#Plugins)

To get the virtual keyboard, right now there are only Windows builds available from the [latest release](https://github.com/WootingKb/wooting-analog-sdk/releases), for other platforms you'll need to build it yourself as described below.

## Components

- `wooting-analog-sdk`: The core Analog SDK which handles loading of plugins. This is installed systemwide and is updated separately
- `wooting-analog-common`: This library contains all common Analog SDK definitions which are used by every part
- `wooting-analog-plugin-dev`: This library contains all common elements needed for designing plugins. This re-exports `wooting-analog-common`, so it is not required for plugins to separately depend on `wooting-analog-common`
- `wooting-analog-wrapper`: This is the SDK wrapper which is what Applications should use to communicate with the SDK. The linked dll should be shipped with the application using it.
- `wooting-analog-test`: This is a C# test application which can be used to test the SDK through the wrapper.
- `wooting-analog-test-plugin`: Dummy plugin which uses shared memory so other processes can control the output of the plugin. This is used for unit testing of the SDK and allows the `wooting-analog-virtual-kb` to work
- `wooting-analog-virtual-kb`: Virtual Keyboard using GTK which allows to set the analog value of all the keys through the dummy plugin. This allows you to test an Analog SDK implementation without an analog device
- `wooting-analog-sdk-updater`: Updater tool to update the Analog SDK from Github releases

### Headers

- `wooting-analog-wrapper.h`: This is the header which includes everything that you need to use the SDK. (This uses `wooting-analog-common.h` which defines all relevant enums & structs)
- `wooting-analog-common.h`: This defines all common enums, headers & structs which are needed by plugins & SDK users
- `wooting-analog-plugin-dev.h`: This includes `wooting-analog-common.h` & additional functions which are obtained from statically linking to the analog-sdk-common library. (FOR USE WITH PLUGINS)
- `plugin.h`: This is the header which plugins should use to define all functions that need to be exported for a plugin to work

## Building

### Build Dependencies

- [rust](https://www.rust-lang.org/)
- [cargo-make](https://github.com/sagiegurari/cargo-make)
- [cbindgen](https://github.com/eqrion/cbindgen) (For verifying/generating headers. Should be installed automatically if necessary)
- [dotnet-core](https://dotnet.microsoft.com/download) If you want to use `wooting-analog-test`
- [wixtoolset](https://wixtoolset.org/releases/) If you want to build the windows installer for the sdk

### How to Build

Everything can be built using this command. All the outputs will be under `target/debug`

```bash
# Build debug
cargo make build
# Build release
cargo make build -- --release
# Build & run tests (To verify headers you'll need the nightly toolchain installed)
cargo make
```

To test:

```bash
cargo make test-flow
```

The current build process is setup to verify the existing generated headers in the test phase. If you decide to make changes which effect these outputs, you can update the headers by running:

```
cargo make gen-headers
```

To run the test application:

```
cargo make test-app
```

To run the virtual keyboard (The Analog SDK must be running for this to work):

```
cargo make virtual-kb
```

To build the windows installer for the SDK:

```
cd wooting-analog-sdk
cargo make win-installer
```

The installer will be located in `$gitroot/target/wix`

To build the deb package for the SDK:

```
cargo make build-deb
```

The deb package will be located in `$gitroot/target/debian`

### Outputs

All build outputs can be found under `target/debug`, with generated headers coming under the `includes` and `includes-cpp` directories.

Currently the headers have to be manually generated and kept in the repo. When intentional changes are made, the testing phase verifies that the pre-generated headers match what would be generated now to ensure that accidental changes aren't made to the output of the header generation.

### Contributing Note

The headers generated for the `wrapper`, `common` and `plugin-dev` crates are verified in the CI to ensure that the current headers are up to date and that we can review any changes which are made to the headers (rather than purely generating them and potentially not knowing exactly what has changed).
Before commiting (if you've made changes to any of the previously mentioned crates) you should run `cargo make verify-headers` to ensure that your headers are up to date, if this fails due to them being different, run `cargo make gen-headers` and review the changes to the headers before commiting.

## Related Repositories

- [WootingPiano](https://github.com/simon-wh/WootingPiano) (Originally by Microdee) Sets up the Wooting keyboard to be used as a MIDI keyboard input
- [wooting-analog-plugin](https://github.com/WootingKb/wooting-analog-plugin): This is Wooting's Plugin which is written in Rust and serves as a good reference implementation
- [wooting-analog-plugin-examples](https://github.com/WootingKb/wooting-analog-plugin-examples): This repo contains all plugin examples that have been collected
- [wooting-analog-wrappers](https://github.com/WootingKb/wooting-analog-wrappers): Official language wrappers for the Wooting Analog SDK

## TODO

- [ ] Improve docs & crates readme for `common` and `plugin-dev` packages
