extern crate env_logger;
extern crate log;
#[macro_use]
extern crate lazy_static;
use log::{error, info};
use shared_memory::*;

use wooting_analog_common::DeviceType;

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
    pub device_type: DeviceType,

    pub device_connected: bool,
    pub dirty_device_info: bool,

    pub analog_values: [u8; 0xFF],
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
lazy_static! {
    static ref KEYBOARD_LAYOUT: Vec<Vec<(&'static str, u16, u16, u16)>> = vec![
        vec![
            ("Esc", 41, 1, 1),
            ("", 0, 1, 1),
            ("F1", 58, 1, 1),
            ("F2", 59, 1, 1),
            ("F3", 60, 1, 1),
            ("F4", 61, 1, 1),
            ("F5", 62, 1, 1),
            ("F6", 63, 1, 1),
            ("F7", 64, 1, 1),
            ("F8", 65, 1, 1),
            ("F9", 66, 1, 1),
            ("F10", 67, 1, 1),
            ("F11", 68, 1, 1),
            ("F12", 69, 1, 1),
            ("Prnt", 70, 1, 1),
            ("Pse", 72, 1, 1),
            ("Scrl", 71, 1, 1),
            ("A1", 0, 1, 1),
            ("A2", 0, 1, 1),
            ("A3", 0, 1, 1),
            ("Mode", 0, 1, 1),
        ],
        vec![
            ("`", 53, 1, 1),
            ("1", 30, 1, 1),
            ("2", 31, 1, 1),
            ("3", 32, 1, 1),
            ("4", 33, 1, 1),
            ("5", 34, 1, 1),
            ("6", 35, 1, 1),
            ("7", 36, 1, 1),
            ("8", 37, 1, 1),
            ("9", 38, 1, 1),
            ("0", 39, 1, 1),
            ("-", 45, 1, 1),
            ("=", 46, 1, 1),
            ("<-", 42, 1, 1),
            ("Ins", 73, 1, 1),
            ("Hme", 74, 1, 1),
            ("PgUp", 75, 1, 1),
            ("NumLck", 83, 1, 1),
            ("/", 84, 1, 1),
            ("*", 85, 1, 1),
            ("-", 86, 1, 1),
        ],
        vec![
            ("Tab", 43, 1, 1),
            ("Q", 20, 1, 1),
            ("W", 26, 1, 1),
            ("E", 8, 1, 1),
            ("R", 21, 1, 1),
            ("T", 23, 1, 1),
            ("Y", 28, 1, 1),
            ("U", 24, 1, 1),
            ("I", 12, 1, 1),
            ("O", 18, 1, 1),
            ("P", 19, 1, 1),
            ("[", 47, 1, 1),
            ("]", 48, 1, 1),
            ("#", 49, 1, 1),
            ("Del", 76, 1, 1),
            ("End", 77, 1, 1),
            ("PgDn", 78, 1, 1),
            ("7", 95, 1, 1),
            ("8", 96, 1, 1),
            ("9", 97, 1, 1),
            ("+", 87, 1, 2),
        ],
        vec![
            ("Caps", 57, 1, 1),
            ("A", 4, 1, 1),
            ("S", 22, 1, 1),
            ("D", 7, 1, 1),
            ("F", 9, 1, 1),
            ("G", 10, 1, 1),
            ("H", 11, 1, 1),
            ("J", 13, 1, 1),
            ("K", 14, 1, 1),
            ("L", 15, 1, 1),
            (";", 51, 1, 1),
            ("'", 52, 1, 1),
            ("Enter", 40, 2, 1),
            ("", 0, 1, 1),
            ("", 0, 1, 1),
            ("", 0, 1, 1),
            ("4", 92, 1, 1),
            ("5", 93, 1, 1),
            ("6", 94, 1, 1),
        ],
        vec![
            ("Shift", 225, 1, 1),
            ("Z", 29, 1, 1),
            ("X", 27, 1, 1),
            ("C", 6, 1, 1),
            ("V", 25, 1, 1),
            ("B", 5, 1, 1),
            ("N", 17, 1, 1),
            ("M", 16, 1, 1),
            (",", 54, 1, 1),
            (".", 55, 1, 1),
            ("/", 56, 1, 1),
            ("Shift", 229, 3, 1),
            ("", 0, 1, 1),
            ("^", 82, 1, 1),
            ("", 0, 1, 1),
            ("1", 89, 1, 1),
            ("2", 90, 1, 1),
            ("3", 91, 1, 1),
            ("Enter", 88, 1, 2),
        ],
        vec![
            ("Ctrl", 224, 1, 1),
            ("Win", 227, 1, 1),
            ("Alt", 226, 1, 1),
            ("Space", 44, 7, 1),
            ("Alt", 230, 1, 1),
            ("Win", 231, 1, 1),
            ("Fn", 0, 1, 1),
            ("Ctrl", 228, 1, 1),
            ("<", 80, 1, 1),
            ("v", 81, 1, 1),
            (">", 79, 1, 1),
            ("0", 98, 2, 1),
            (".", 99, 1, 1),
            ("", 0, 1, 1),
        ],
    ];
}
// use iced::{slider,HorizontalAlignment, Length, Column, Container, Element, Row, Sandbox, Settings, Slider, Text};

use env_logger::Env;
use iced::widget::container::Style;
use iced::{
    container, slider, window, Align, Checkbox, Color, Column, Container, Element,
    HorizontalAlignment, Length, Row, Sandbox, Settings, Slider, Text,
};
use std::borrow::Borrow;

struct Key {
    slider_state: slider::State,
    keycode: u16,
    label: String,
    width: u16,
    height: u16,
    value: f32,
    xy: (usize, usize),
}

const KEY_WIDTH: u16 = 60;
const KEY_SPACING: u16 = 10;
const WIDGET_PADDING: u16 = 5;

struct KeyStyle;
// impl From<KeyStyle> for Box<dyn container::StyleSheet> {
//     fn from(_: KeyStyle) -> Self {
//         container::Style {text_color: None,
//             background: None,
//             border_radius: 1,
//             border_width: 1,
//             border_color: Color::BLACK}.into()
//     }
// }

impl container::StyleSheet for KeyStyle {
    fn style(&self) -> Style {
        container::Style {
            text_color: None,
            background: None,
            border_radius: 1,
            border_width: 1,
            border_color: Color::BLACK,
        }
    }
}

impl Key {
    fn new(
        keycode: u16,
        label: String,
        width: u16,
        height: u16,
        value: f32,
        xy: (usize, usize),
    ) -> Self {
        Key {
            slider_state: Default::default(),
            keycode,
            label,
            width,
            height,
            value,
            xy,
        }
    }

    fn width(&self) -> Length {
        Length::Units(KEY_WIDTH * self.width + (KEY_SPACING * (self.width - 1)))
    }

    fn height(&self) -> Length {
        Length::Units(KEY_WIDTH)
    }

    fn view(&mut self) -> Element<Message> {
        let width = self.width();
        let height = self.height();

        if self.label.is_empty() {
            return Container::new(Column::new())
                .width(width)
                .height(height)
                .into();
        }

        let inner_xy = self.xy.clone();
        Container::new(
            Column::new()
                .padding(5)
                .align_items(Align::Center)
                .push(
                    Text::new(self.label.as_str())
                        .horizontal_alignment(HorizontalAlignment::Center),
                )
                .push(
                    Text::new(format!("{:.3}", self.value.trunc() / 255f32)).color(
                        Color::from_rgb8(255 - self.value as u8, self.value as u8, 0),
                    ),
                )
                .push(Slider::new(
                    &mut self.slider_state,
                    0.0..=255.0,
                    self.value,
                    move |val| Message::SliderChanged(inner_xy, val),
                )),
        )
        .height(height)
        .width(width)
        .style(KeyStyle)
        .into()
    }

    fn update(&mut self, shared_state: &mut SharedMem, value: f32) {
        self.value = value;
        match shared_state.wlock::<SharedState>(0) {
            Ok(mut v) => {
                v.analog_values[self.keycode as usize] = self.value as u8;
                // info!("Updated key: {}, to {}", self.keycode, self.value);
            }
            Err(_) => panic!("Failed to acquire write lock !"),
        };
    }
}

struct AppState {
    keys: Vec<Vec<Key>>,
    shared_mem: SharedMem,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    SliderChanged((usize, usize), f32),
    ConnectedChanged(bool),
}

impl Sandbox for AppState {
    type Message = Message;

    fn new() -> Self {
        let mut shmem = match SharedMem::open_linked(
            std::env::temp_dir()
                .join("wooting-test-plugin.link")
                .as_os_str(),
        ) {
            Ok(v) => v,
            Err(e) => {
                info!("Error : {}", e);
                panic!("Failed to open SharedMem...");
            }
        };

        info!("Opened link file with info : {}", shmem);

        //Make sure at least one lock exists before using it...
        if shmem.num_locks() != 1 {
            println!("Expected to only have 1 lock in shared mapping !");
            panic!();
        }

        //Tell the plugin that we've connected
        {
            let mut shared_state = match shmem.wlock::<SharedState>(0) {
                Ok(v) => v,
                Err(_) => panic!("Failed to acquire write lock !"),
            };
            shared_state.device_connected = true;
        }
        let mut keys = vec![];
        {
            let state = match shmem.rlock::<SharedState>(0) {
                Ok(v) => v,
                Err(_) => panic!("Failed to acquire read lock !"),
            };
            for (y, items) in KEYBOARD_LAYOUT.iter().enumerate() {
                let mut row: Vec<Key> = vec![];
                for (x, &(name, code, width, height)) in items.iter().enumerate() {
                    // if width == 0 {
                    //     continue;
                    // }
                    row.push(Key::new(
                        code,
                        name.to_string(),
                        width,
                        height,
                        state.analog_values[code as usize].into(),
                        (x, y),
                    ))
                }
                keys.push(row);
            }
        }

        Self {
            keys,
            shared_mem: shmem,
        }
    }

    fn title(&self) -> String {
        String::from("Wooting Analog Virtual Keyboard")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::SliderChanged((x, y), val) => {
                self.keys
                    .get_mut(y)
                    .unwrap()
                    .get_mut(x)
                    .unwrap()
                    .update(&mut self.shared_mem, val);
            }
            Message::ConnectedChanged(state) => {
                match self.shared_mem.wlock::<SharedState>(0) {
                    Ok(mut shared_state) => shared_state.device_connected = state,
                    Err(_) => panic!("Failed to acquire read lock !"),
                };
            }
        }
    }

    fn view(&mut self) -> Element<Message> {
        let mut col = Column::new().spacing(KEY_SPACING);
        for key_row in self.keys.iter_mut() {
            let mut row = Row::new();
            for key in key_row.iter_mut() {
                row = row.push(key.view());
            }
            col = col.push(row.spacing(KEY_SPACING));
        }
        col.push(
            Row::new().push(Checkbox::new(
                self.shared_mem
                    .rlock::<SharedState>(0)
                    .unwrap()
                    .device_connected,
                "Device Connected",
                Message::ConnectedChanged,
            )),
        )
        .padding(WIDGET_PADDING)
        .into()
    }
}

impl Drop for AppState {
    fn drop(&mut self) {
        //Perform cleanup
        let mut shared_state = match self.shared_mem.wlock::<SharedState>(0) {
            Ok(v) => v,
            Err(_) => panic!("Failed to acquire write lock !"),
        };

        shared_state.device_connected = false;
        shared_state.analog_values.iter_mut().for_each(|x| *x = 0);
    }
}

fn main() {
    if let Err(e) = env_logger::from_env(Env::default().default_filter_or("info")).try_init() {
        error!("Failed to init env_logger: {}", e)
    }
    let kb: &Vec<Vec<(&'static str, u16, u16, u16)>> = KEYBOARD_LAYOUT.borrow();
    let max_key_width = kb.iter().fold(0, |current: u32, item| {
        current.max(
            item.iter()
                .fold(0, |width: u32, key: &(&'static str, u16, u16, u16)| {
                    width + key.2 as u32
                }),
        )
    });
    let width: u32 = max_key_width * KEY_WIDTH as u32
        + ((max_key_width - 1) * KEY_SPACING as u32)
        + WIDGET_PADDING as u32 * 2;
    let rows = kb.len() as u32;
    // Add 1 to the number of rows for the Height for the extra row of controls
    let height =
        (rows + 1) * (KEY_WIDTH as u32) + (rows * KEY_SPACING as u32) + WIDGET_PADDING as u32 * 2;

    AppState::run(Settings {
        window: window::Settings {
            size: (width, height),
            resizable: false,
            decorations: true,
        },
        default_font: None,
        antialiasing: false,
        flags: ()
    })
}
