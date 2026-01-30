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

#### Homebrew (Outdated)

> [!WARNING]
> The Homebrew package is currently outdated. We plan on updating it in the near future.

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