# Wooting Analog SDK

The Wooting Analog SDK is the open driver for Analog keyboards. It's goal is to create native
support for Analog keyboards in any game or application. The repository is mostly aimed at
developers looking to implement the Analog SDK or for users looking to dig a little deeper. If you
want to use the Analog SDK just jump to the [installing](docs/INSTALL.md) section. On Windows it
will automatically get installed with the Wootility.

This repo contains all the core cross-platform components of the Wooting Analog SDK. The SDK and
most of the components are built on Rust and should run on Windows, Mac and Linux, following the
same steps for each platform unless otherwise specified.

## Documentation

- [SDK Usage](docs/SDK_USAGE.md) on how to use the SDK
- [Installation Guide](docs/INSTALL.md) on how to use and install the distributable and the system
  SDK.

### Developers

- [Contributing Guide](docs/CONTRIBUTING.md) to help you with your first contribution.
- [Build Instructions](docs/BUILD.md) on how to build the SDK using Rust.
- [Migration Guide](docs/MIGRATION_GUIDE.md) on how to upgrade to the latest version of the SDK.
- [Virtual Keyboard](docs/VIRTUAL_KEYBOARD.md) on how to setup and use the virtual keyboard for
  development without any hardware required.
- [Plugin Introduction](docs/PLUGINS.md) for information on creating plugins.

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
