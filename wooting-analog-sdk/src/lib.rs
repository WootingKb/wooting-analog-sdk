//external crates
#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;
extern crate ffi_support;
extern crate scancode;
mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {}
}
#[macro_use]
extern crate lazy_static;
extern crate env_logger;
#[cfg(windows)]
extern crate winapi;
extern crate wooting_analog_sdk_common;

//library modules
pub mod ffi;
pub mod keycode;
pub mod sdk;
mod cplugin;
