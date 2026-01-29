[![wooting-analog-common
Crates.io](https://img.shields.io/crates/v/wooting-analog-common?label=crates.io%20wooting-analog-common)](https://crates.io/crates/wooting-analog-common)
[![wooting-analog-plugin-dev
Crates.io](https://img.shields.io/crates/v/wooting-analog-plugin-dev?label=crates.io%20wooting-analog-plugin-dev)](https://crates.io/crates/wooting-analog-plugin-dev)
[![Documentation](https://img.shields.io/badge/Docs-Docs-green)](https://dev.wooting.nl/wooting-analog-sdk-guide/introduction/)

# Wooting Analog SDK

The Wooting Analog SDK is the open driver for Analog keyboards. It's goal is to create native
support for Analog keyboards in any game or application. The repository is mostly aimed at
developers looking to implement the Analog SDK or for users looking to dig a little deeper. If you
want to use the Analog SDK just jump to the [installing](#installing) section. On Windows it will
automatically get installed with the Wootility.

This repo contains all the core cross-platform components of the Wooting Analog SDK. The SDK and
most of the components are built on Rust and should run on Windows, Mac and Linux, following the
same steps for each platform unless otherwise specified.

## Migration Guide

See the [Migration Guide](MIGRATION_GUIDE.md) on how to upgrade to the latest version of the SDK,
the most important details and breaking changes will be there. Starting from `v0.9.0` the SDK
has had a major project structure overhaul, the functionality is still relatively similar.

## Installing

### Windows

For developers: use the `wooting_analog_sdk_dist` in your games/apps. You can package that along
your game/app, it will work regardless if the user has anything SDK related installed on their system.

On Windows the system SDK will be installed & updated automatically through Wootility (>= v3.4). If
you wish to install manually, download the latest `.msi` from the [latest
release](https://github.com/WootingKb/wooting-analog-sdk/releases)

### Linux

#### Ubuntu / Debian / Pop!\_OS etc

On Linux the primarily installation method is the `deb` package, which can be found on the [latest
release](https://github.com/WootingKb/wooting-analog-sdk/releases).

[Here is some helpful information](https://linuxhint.com/install_deb_packages_ubuntu/) for
installing a `deb` package manually. For all (terminal instructions) and Ubuntu (GUI instructions)

#### Manual

- Download & Extract the [latest release](https://github.com/WootingKb/wooting-analog-sdk/releases)
  `wooting-analog-sdk-v*.*.*-x86_64-unknown-linux-gnu.tar.gz`
- Copy `$extract/release/libwooting_analog_sdk.so` to `/usr/lib`. (Or to some directory and add
  that path to the `LD_LIBRARY_PATH` environment variable)

For use in your game/app package `$extract/release/libwooting_analog_sdk_dist.so` with your
game/app's files. During development you could use the debug build of the distributable, giving the
benefit of attaching a virtual keyboard for testing.
- Copy `$extract/release/libwooting_analog_sdk_dist.so` to your game/app to develop against directly.

### Mac

#### Homebrew (Recommended)

We now have a [Homebrew](https://brew.sh/) package available to install the `sdk` from our own [tap](https://github.com/WootingKb/homebrew-wooting).

1. Install Homebrew from [their website](https://brew.sh/)
1. Open the terminal and enter the command below:

```
brew install wootingkb/wooting/wooting-analog-sdk
```

#### Manual

- Download & Extract the [latest release](https://github.com/WootingKb/wooting-analog-sdk/releases)
  `wooting-analog-sdk-v*.*.*-x86_64-apple-darwin.tar.gz`
- Copy `$extract/release/libwooting_analog_sdk.dylib` to `/usr/local/lib`. (Or to some directory
  and add that path to the `DYLD_LIBRARY_PATH` environment variable)
- Additionally, you may need to adjust security settings for OSX to let it run. For
  [reference](https://github.com/hashicorp/terraform/issues/23033#issuecomment-542302933)

## Plugins

This SDK uses Plugins to provide support for Analog hardware, these must be located in a
subdirectory of `WootingAnalogPlugins`. Which can be found in these places on each platform:

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

The core documentation can be found in [SDK usage](SDK_USAGE.md) for a guide on how to use the SDK
and the [Plugin introduction](PLUGINS.md) for information on creating plugins.

## Virtual Keyboard

The SDK includes a 'Virtual Keyboard' app which will emulate an Analog Keyboard and allows you to
test with the Analog SDK without needing a keyboard. To use this, ensure you have the
`debug/wooting-analog-sdk_dist` installed.

You need to have your game/app running and polling for keyboard state before you can hook the
virtual keyboard into your process. Only the debug versions of the SDK support using the virtual keyboard.

You can get the virtual keyboard by downloading the `.tar.gz` archive for your platfrom from the
[latest release](https://github.com/WootingKb/wooting-analog-sdk/releases) and find the
`wooting-analog-virtual-control` executable.

## Components

- `wooting-analog-sdk`: The core Analog SDK which handles loading of plugins. This is installed
  systemwide and is updated separately
- `wooting-analog-dist`: This is the SDK distributable which is what Applications should use. The
  linked dll should be shipped with the application using it.
- `wooting-analog-virtual-kb`: Virtual Keyboard using GTK which allows to set the analog value of
  all the keys through the `debug/wooting_analog_sdk_dist`. This allows you to test an Analog SDK
  implementation without an analog device
- `wooting-analog-sdk-updater`: Updater tool to update the Analog SDK from Github releases

### Headers

- `wooting-analog-sdk.h`: This is the header which includes everything that you need to use the
  SDK.
- `plugin.h`: This is the header which plugins should use to define all functions that need to be
  exported for a plugin to work

## Building

### Build Dependencies

- [rust](https://www.rust-lang.org/)
- [cbindgen](https://github.com/eqrion/cbindgen) (For verifying/generating headers. Should be
  installed automatically if necessary)
- [wixtoolset](https://wixtoolset.org/releases/) If you want to build the windows installer for the
  sdk **[Windows]**

### How to Build

Everything can be built using this command. All the outputs will be under `target/debug`

```bash
# Normal debug build without any extra features
cargo build

# Builds the library with all the exported ffi symbols
cargo build --features ffi

# Builds the library with support for the virtual keyboard to hook into the SDK
cargo build --features virtual-input
```

The current build process is setup to verify the existing generated headers in the test phase. If
you decide to make changes which effect these outputs, you can update the headers by running:

```bash
cargo install cbindgen@0.29.2

# Rebuild headers
cbindgen --crate wooting-analog-sdk --output ./includes/wooting-analog-sdk.h

# Verify
cbindgen --crate wooting-analog-sdk --output ./includes/wooting-analog-sdk.h --verify
```

To run the virtual keyboard (The Analog SDK must be running for this to work):

```bash
cargo run -p wooting-analog-virtual-control
```

To build the windows installer for the SDK:

```bash
cargo install cargo-wix@0.3.9
cargo wix -p wooting-analog-sdk --nocapture
```

The installer will be located in `$gitroot/target/wix`

To build the deb package for the SDK:

```bash
cargo install cargo-deb@3.6.2
cargo deb
```

The deb package will be located in `$gitroot/target/debian`

### Outputs

All build outputs can be found under `target/debug`, with generated headers coming under the
`includes` directory.

Currently the headers have to be manually generated and kept in the repo. When intentional changes
are made, the testing phase verifies that the pre-generated headers match what would be generated
now to ensure that accidental changes aren't made to the output of the header generation.

### Contributing Note

The headers generated for the SDK crate are verified in the CI to
ensure that the current headers are up to date and that we can review any changes which are made to
the headers (rather than purely generating them and potentially not knowing exactly what has
changed). Before commiting (if you've made changes to any part of the SDK crate) you
should run `cbindgen --crate wooting-analog-sdk --output ./includes/wooting-analog-sdk.h --verify` to ensure that your headers are up to date, if this fails due
to them being different, run `cbindgen --crate wooting-analog-sdk --output ./includes/wooting-analog-sdk.h` and review the changes to the headers before
commiting.

## Related Repositories

- [wooting-analog-midi](https://github.com/WootingKb/wooting-analog-midi) Cross-platform virtual
  MIDI device for (Wooting) analog keyboards! Inspired by Microdee's WootingPiano below
- [WootingPiano](https://github.com/simon-wh/WootingPiano) (Originally by Microdee) Sets up the
  Wooting keyboard to be used as a MIDI keyboard input
- [wooting-analog-plugin](https://github.com/WootingKb/wooting-analog-plugin): This is Wooting's
  Plugin which is written in Rust and serves as a good reference implementation
- [wooting-analog-plugin-examples](https://github.com/WootingKb/wooting-analog-plugin-examples):
  This repo contains all plugin examples that have been collected
- [wooting-analog-wrappers](https://github.com/WootingKb/wooting-analog-wrappers): Official language
  wrappers for the Wooting Analog SDK
