//external crates
#[macro_use]
extern crate log;
#[macro_use]
extern crate anyhow;
extern crate ffi_support;
extern crate scancode;

#[macro_use]
extern crate lazy_static;
#[cfg(windows)]
extern crate winapi;
extern crate wooting_analog_common;
extern crate wooting_analog_plugin_dev;

#[cfg(test)]
#[macro_use]
extern crate shared_memory;

//library modules
mod cplugin;
pub mod ffi;
pub mod keycode;
pub mod sdk;
