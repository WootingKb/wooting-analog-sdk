#[macro_use] extern crate lazy_static;
use libloading as libl;
use std::ops::Deref;
use std::os::raw::c_char;


macro_rules! dynamic_extern {
    (@as_item $i:item) => {$i};

    (
        #[link=$lib:tt]
        extern $cconv:tt {
            $(
                fn $fn_names:ident($($fn_arg_names:ident: $fn_arg_tys:ty),*) $(-> $fn_ret_tys:ty)*;
            )*
        }
    ) => {
        lazy_static! {
            static ref LIB : Option<libl::Library> = {
                #[cfg(unix)]
                let lib_path = concat!($lib, ".so");
                #[cfg(windows)]
                let lib_path = $lib;

                //Attempt to load the library, if it fails print the error and discard the error
                libl::Library::new(lib_path).map_err(|e| {
                    println!("Unable to load library: {}\nErr: {}", lib_path, e);
                }).ok()
            };
        }
        $(
            dynamic_extern! {
                @as_item
                #[no_mangle]
                unsafe extern fn $fn_names($($fn_arg_names: $fn_arg_tys),*) $(-> $fn_ret_tys)* {
                    type FnPtr = unsafe extern $cconv fn($($fn_arg_tys),*) $(-> $fn_ret_tys)*;
                    
                    lazy_static! {
                        static ref FUNC: Option<libl::Symbol<'static, FnPtr>> = {
                            LIB.as_ref().and_then(|lib| unsafe {
                                //Get func, print and discard error as we don't need it again
                                lib.get(stringify!($fn_names).as_bytes()).map_err(|e| {
                                    println!("{}", e);
                                }).ok()
                            })
                        };
                    }
                    match FUNC.deref() {
                        Some(f) => f($($fn_arg_names),*),
                        _ => Default::default()
                    }
                }
            }
        )*
    };
}

dynamic_extern! {
    #[link="libanalog_sdk"]
    extern "C" {
        fn sdk_initialise() -> bool;
        fn sdk_is_initialised() -> bool;
        fn sdk_uninitialise() -> bool;
        fn sdk_add(x: u32, y: u32) -> u32;
        fn sdk_read_analog_hid(code: u8) -> f32;
        fn sdk_read_analog_vk(code: u8, translate: bool) -> f32;
        fn sdk_read_analog_sc(code: u8) -> f32;
        fn sdk_set_disconnected_cb(cb: extern fn(*const c_char));
        fn sdk_clear_disconnected_cb();
    }
}

/*fn main() {
    unsafe { println!("We got {}", test_function(16)); };
}*/