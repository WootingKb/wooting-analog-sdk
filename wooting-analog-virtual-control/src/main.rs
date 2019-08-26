#[macro_use]
extern crate log;
use shared_memory::*;
use log::{error, info};

extern crate gtk;
extern crate gio;

// To import all needed traits.
use gtk::prelude::*;
use gio::prelude::*;

use std::env;
use gtk::BoxBuilder;

struct SharedState {
    pub vendor_id: u16,
    /// Device Product ID `pid`
    pub product_id: u16,
    //TODO: Consider switching these to FFiStr
    /// Device Manufacturer name
    pub manufacturer_name: [u8; 20],
    /// Device name
    pub device_name: [u8; 20],
    /// Unique device ID, which should be generated using `generate_device_id`
    pub device_id: u64,

    pub device_connected: bool,
    pub dirty_device_info: bool,

    pub analog_values: [u8; 0xFF]
}

unsafe impl SharedMemCast for SharedState {}

/*fn main() {
    let mut my_shmem = match SharedMem::open_linked("/home/simonw/wooting-test-plugin.link") {
        Ok(v) => v,
        Err(e) => {
            println!("Error : {}", e);
            println!("Failed to open SharedMem...");
            return;
        }
    };

    println!("Opened link file with info : {}", my_shmem);

    //Make sure at least one lock exists before using it...
    if my_shmem.num_locks() != 1 {
        println!("Expected to only have 1 lock in shared mapping !");
        return;
    }
    let mut index = 1;
    loop {
        {
            let mut shared_state = match my_shmem.wlock::<SharedState>(0) {
                Ok(v) => v,
                Err(_) => panic!("Failed to acquire write lock !"),
            };
            shared_state.analog_values[index - 1] = 0;
            shared_state.analog_values[index] = 255;
        }
        index = (index + 1).min(254);
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}*/


const keyboard_layout: [[(&str, u16, i32); 14]; 2] = [
    [ ("Esc", 41, 1), ("", 0, 0), ("F1", 58, 1), ("F2", 59, 1), ("F3", 60, 1), ("F4", 61, 1), ("F5", 62, 1), ("F6", 63, 1), ("F7", 64, 1), ("F8", 65, 1), ("F9", 66, 1), ("F10", 67, 1), ("F11", 68, 1), ("F12", 69, 1) ],
    [ ("Tab", 43, 1), ("Q", 20, 1), ("W", 26, 1), ("E", 8, 1), ("R", 21, 1), ("T", 23, 1), ("Y", 28, 1), ("U", 24, 1), ("I", 12, 1), ("O", 18, 1), ("P", 19, 1), ("[", 47, 1), ("]", 48, 1), ("#", 49, 1) ]
];

fn main() {
    use gtk::{Box, ButtonBuilder, Button, ButtonExt, GridBuilder, VolumeButton, VolumeButtonBuilder};
    use std::cell::RefCell;
    use std::rc::Rc;



    //println!("Opened link file with info : {:?}", my_shmem);

    //Make sure at least one lock exists before using it...
    /*if my_shmem.num_locks() != 1 {
        println!("Expected to only have 1 lock in shared mapping !");
        return;
    }*/


    let uiapp = gtk::Application::new(Some("org.gtkrsnotes.demo"),
                                      gio::ApplicationFlags::FLAGS_NONE)
        .expect("Application::new failed");
    uiapp.connect_activate(|app| {
        let mut shmem = match SharedMem::open_linked("/home/simonw/wooting-test-plugin.link") {
            Ok(v) => v,
            Err(e) => {
                println!("Error : {}", e);
                println!("Failed to open SharedMem...");
                return;
            }
        };

        println!("Opened link file with info : {}", shmem);

        //Make sure at least one lock exists before using it...
        if shmem.num_locks() != 1 {
            println!("Expected to only have 1 lock in shared mapping !");
            return;
        }
        let my_shmem = Rc::new(RefCell::new(shmem));

        // We create the main window.
        let win = gtk::ApplicationWindow::new(app);

        // Then we set its size and a title.
        win.set_default_size(320, 200);
        win.set_title("Basic example");
        let grid = GridBuilder::new().build();
        for (y, items) in keyboard_layout.iter().enumerate() {
            for (x, &(name, code, width)) in items.iter().enumerate() {
                let button = VolumeButtonBuilder::new()
                    .label(format!("{}\n{}\n{:.2}", name, code,0_f32).as_str())
                    .build();

                let mem_clone = my_shmem.clone();
                button.connect_value_changed(move |but, val| {
                    let mut sem = mem_clone.borrow_mut();
                    let mut shared_state = match sem.wlock::<SharedState>(0) {
                        Ok(v) => v,
                        Err(_) => panic!("Failed to acquire write lock !"),
                    };
                    shared_state.analog_values[code as usize] =  (val*255_f64) as u8;
                    but.set_label(format!("{}\n{}\n{:.2}", name, code,val).as_str());
                });
                grid.attach(&button, x as i32, y as i32, width,1);
            }
        }

        /*for x in 0..20 {
            for y in 0..5 {
                //let container = BoxBuilder::(gtk::Orientation::Vertical, 5);
                let button = ButtonBuilder::new()
                    .label(format!("({},{})", x, y).as_str()).build();
                button.connect_clicked(|but| {
                    println!("Clicked!");
                });
                grid.attach(&button, x, y, 1,1);
            }
        }*/
        win.add(&grid);

        // Don't forget to make all widgets visible.
        win.show_all();
    });
    uiapp.run(&env::args().collect::<Vec<_>>());
}