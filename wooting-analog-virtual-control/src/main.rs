#[macro_use]
extern crate log;
use shared_memory::*;
use log::{error, info};

extern crate gtk;
extern crate gio;
extern crate gdk;

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
    let mut my_shmem = match SharedMem::open_linked("wooting-test-plugin.link") {
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


const keyboard_layout: [[(&str, u16, i32, i32); 21]; 6] = [
    [ ("Esc", 41, 1,1), ("", 0, 0,1), ("F1", 58, 1,1), ("F2", 59, 1,1), ("F3", 60, 1,1), ("F4", 61, 1,1), ("F5", 62, 1,1), ("F6", 63, 1,1), ("F7", 64, 1,1), ("F8", 65, 1,1), ("F9", 66, 1,1), ("F10", 67, 1,1), ("F11", 68, 1,1), ("F12", 69, 1,1), ("Prnt", 70, 1,1), ("Pse", 72, 1,1), ("Scrl", 71, 1,1), ("A1", 0, 1,1), ("A2", 0, 1,1), ("A3", 0, 1,1), ("Mode", 0, 1, 1) ],
    [ ("`", 53, 1,1), ("1", 30, 1,1), ("2", 31, 1,1), ("3", 32, 1,1), ("4", 33, 1,1), ("5", 34, 1,1), ("6", 35, 1,1), ("7", 36, 1,1), ("8", 37, 1,1), ("9", 38, 1,1), ("0", 39, 1,1), ("-", 45, 1,1), ("=", 46, 1,1), ("<-", 42, 1,1), ("Ins", 73, 1,1), ("Hme", 74, 1,1), ("PgUp", 75, 1,1), ("NumLck", 83, 1,1), ("/", 84, 1,1), ("*", 85, 1,1), ("-", 86, 1, 1) ],
    [ ("Tab", 43, 1,1), ("Q", 20, 1,1), ("W", 26, 1,1), ("E", 8, 1,1), ("R", 21, 1,1), ("T", 23, 1,1), ("Y", 28, 1,1), ("U", 24, 1,1), ("I", 12, 1,1), ("O", 18, 1,1), ("P", 19, 1,1), ("[", 47, 1,1), ("]", 48, 1,1), ("#", 49, 1,1), ("Del", 76, 1,1), ("End", 77, 1,1), ("PgDn", 78, 1,1), ("7", 95, 1,1), ("8", 96, 1,1), ("9", 97, 1,1), ("+", 87, 1, 2) ],
    [ ("Caps", 57, 1,1), ("A", 4, 1,1), ("S", 22, 1,1), ("D", 7, 1,1), ("F", 9, 1,1), ("G", 10, 1,1), ("H", 11, 1,1), ("J", 13, 1,1), ("K", 14, 1,1), ("L", 15, 1,1), (";", 51, 1,1), ("'", 52, 1,1), ("Enter", 40, 2,1), ("", 0, 0,0), ("", 0, 0,0), ("", 0, 0,1), ("", 0, 0,0), ("4", 92, 1,1), ("5", 93, 1,1), ("6", 94, 1,1), ("", 0, 0, 0) ],
    [ ("Shift", 225, 1,1), ("Z", 29, 1,1), ("X", 27, 1,1), ("C", 6, 1,1), ("V", 25, 1,1), ("B", 5, 1,1), ("N", 17, 1,1), ("M", 16, 1,1), (",", 54, 1,1), (".", 55, 1,1), ("/", 56, 1,1), ("Shift", 229, 3,1), ("", 0, 0,0), ("", 0, 0,0), ("", 0, 0,0), ("^", 82, 1,1), ("", 0, 0,1), ("1", 89, 1,1), ("2", 90, 1,1), ("3", 91, 1,1), ("Enter", 88, 1, 2) ],
    [ ("Ctrl", 224, 1,1), ("Win", 227, 1,1), ("Alt", 226, 1,1), ("Space", 44, 7,1), ("", 0, 0,0), ("", 0, 0,0), ("", 0, 0,0), ("", 0, 0,0), ("", 0, 0,0), ("", 0, 0,0), ("Alt", 230, 1,1), ("Win", 231, 1,1), ("Fn", 0, 1,1), ("Ctrl", 228, 1,1), ("<", 80, 1,1), ("v", 81, 1,1), (">", 79, 1,1), ("0", 98, 2,1), ("", 0, 0,0), (".", 99, 1,1), ("", 0, 0, 0) ]
];

fn main() {
    use gtk::{Box, ButtonBuilder, Button, ButtonExt, GridBuilder, VolumeButton, VolumeButtonBuilder, CheckButtonBuilder};
    use std::cell::RefCell;
    use std::rc::Rc;



    //println!("Opened link file with info : {:?}", my_shmem);

    //Make sure at least one lock exists before using it...
    /*if my_shmem.num_locks() != 1 {
        println!("Expected to only have 1 lock in shared mapping !");
        return;
    }*/

    let mut shmem = match SharedMem::open_linked("wooting-test-plugin.link") {
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

    //Tell the plugin that we've connected
    {
        let mut shared_state = match shmem.wlock::<SharedState>(0) {
            Ok(v) => v,
            Err(_) => panic!("Failed to acquire write lock !"),
        };
        shared_state.device_connected = true;
    }
    let og_my_shmem = Rc::new(RefCell::new(shmem));
    let my_shmem = og_my_shmem.clone();

    let uiapp = gtk::Application::new(Some("org.gtkrsnotes.demo"),
                                      gio::ApplicationFlags::FLAGS_NONE)
        .expect("Application::new failed");
    uiapp.connect_activate(move |app| {
        

        // We create the main window.
        let win = gtk::ApplicationWindow::new(app);

        // Then we set its size and a title.
        win.set_default_size(320, 200);
        win.set_title("Wooting Analog Virtual Keyboard");
        win.set_resizable(false);
        let grid = GridBuilder::new().build();
        for (y, items) in keyboard_layout.iter().enumerate() {
            for (x, &(name, code, width, height)) in items.iter().enumerate() {
                if width == 0 {
                    continue;
                }

                let button = VolumeButtonBuilder::new()
                    .label(format!("{}\n{}\n{:.2}", name, code,0_f32).as_str()).border_width(5).relief(gtk::ReliefStyle::Normal)
                    .build();
                button.override_color(gtk::StateFlags::NORMAL, Some(&gdk::RGBA { red:1f64,green:0f64,blue:0f64,alpha:1f64 }));

                let mem_clone = my_shmem.clone();
                button.connect_value_changed(move |but, val| {
                    {
                        let mut sem = mem_clone.borrow_mut();
                        let mut shared_state = match sem.wlock::<SharedState>(0) {
                            Ok(v) => v,
                            Err(_) => panic!("Failed to acquire write lock !"),
                        };
                        shared_state.analog_values[code as usize] =  (val*255_f64) as u8;
                        //shared_state.dirty_device_info = true;
                    }
                    but.set_label(format!("{}\n{}\n{:.2}", name, code,val).as_str());
                    but.override_color(gtk::StateFlags::NORMAL, Some(&gdk::RGBA { red:1f64-val,green:val,blue:0f64,alpha:1f64 }));
                });
                grid.attach(&button, x as i32, y as i32, width,height);
            }
        }
        let edit_grid = GridBuilder::new().build();

        let connected_btn = CheckButtonBuilder::new().label("Device Connected").build();
        connected_btn.set_active(true);
        let mem_clone = my_shmem.clone();
        connected_btn.connect_toggled(move |btn| {
            let mut sem = mem_clone.borrow_mut();
            let mut shared_state = match sem.wlock::<SharedState>(0) {
                Ok(v) => v,
                Err(_) => panic!("Failed to acquire write lock !"),
            };
            shared_state.device_connected = btn.get_active();
        });
        edit_grid.attach(&connected_btn, 0, 0, 1, 1);

        grid.attach(&edit_grid, 0, keyboard_layout.len() as i32, (keyboard_layout[0].len() as i32)-1, 1);

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

    //Perform cleanup
    let mut sem = og_my_shmem.borrow_mut();
    let mut shared_state = match sem.wlock::<SharedState>(0) {
        Ok(v) => v,
        Err(_) => panic!("Failed to acquire write lock !"),
    };

    shared_state.device_connected = false;
    shared_state.analog_values.iter_mut().for_each(|x| *x = 0);

}