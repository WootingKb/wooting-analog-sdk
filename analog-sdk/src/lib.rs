//external crates
#[macro_use]
extern crate log;
#[macro_use]
extern crate error_chain;
extern crate ffi_support;
extern crate scancode;
#[macro_use]
extern crate enum_primitive;
mod errors {
    // Create the Error, ErrorKind, ResultExt, and Result types
    error_chain! {}
}
#[macro_use]
extern crate lazy_static;
extern crate analog_sdk_common;
extern crate env_logger;
#[cfg(windows)]
extern crate winapi;

//library modules
pub mod ffi;
pub mod keycode;
pub mod sdk;
