[![Build Status](https://travis-ci.com/WootingKb/wooting-analog-plugin.svg?branch=develop)](https://travis-ci.com/WootingKb/wooting-analog-plugin)

# Wooting Analog SDK Plugin

This is Wooting's plugin for the [Analog SDK](https://github.com/simon-wh/Analog-SDK) providing easily extensible support for the Wooting One, Wooting Two, Wooting Two Lekker Edition and Wooting Two HE.

## Getting Started

**The easiest way to get the plugin is through installing it & updating it through Wootility (Currently Windows only)**

This plugin is packaged with the [Wooting Analog SDK installer](https://github.com/WootingKb/wooting-analog-sdk/releases), so you probably want to use that, unless you want to install it separately. There is a deb package available on the [releases](https://github.com/WootingKb/wooting-analog-plugin/releases) for installing on Linux.

If you wish to install it manually, you can find the plugin from the Analog SDK tar [release](https://github.com/WootingKb/wooting-analog-sdk/releases), you can find the plugin under `wrapper/sdk/`. Y    ou'll need to put it into a sub-directory of the `WootingAnalogPlugins` directory.
So it should look something like this:

| OS      | Install Path                                                                                 |
| ------- | -------------------------------------------------------------------------------------------- |
| Windows | `C:\Program Files\WootingAnalogPlugins\wooting-analog-plugin\wooting_analog_plugin.dll`      |
| Linux   | `/usr/local/share/WootingAnalogPlugins/wooting-analog-plugin/libwooting_analog_plugin.so`    |
| Mac     | `/usr/local/share/WootingAnalogPlugins/wooting-analog-plugin/libwooting_analog_plugin.dylib` |
