use std::{borrow::Borrow, string::ToString, sync::LazyLock};

use env_logger::Env;
use iced::{
    Alignment, Border, Color, Element, Length, Settings, Shadow, alignment,
    border::Radius,
    widget::{Checkbox, Column, Container, Row, Slider, Text, container},
    window,
};
use log::{error, info};
use shared_memory::*;
use wooting_analog_sdk::device::DeviceType;

const KEY_WIDTH: u16 = 60;
const KEY_SPACING: u16 = 10;
const WIDGET_PADDING: u16 = 5;

#[derive(Debug, Clone, Copy)]
enum Message {
    SliderChanged((usize, usize), f32),
    ConnectedChanged(bool),
}

struct AppState {
    keys: Vec<Vec<Key>>,
    shared_mem: Shmem,
}

impl AppState {
    fn new() -> Self {
        let shmem = ShmemConf::new()
            .flink(
                std::env::temp_dir()
                    .join("wooting-test-plugin.link")
                    .as_os_str(),
            )
            .open()
            .inspect_err(|e| info!("Error: {e}"))
            .expect("unable to attach virtual keyboard: running process should use dev build of the SDK with feature `virtual-input` enabled");

        //Tell the plugin that we've connected
        {
            let shared_state = unsafe { &mut *(shmem.as_ptr() as *mut SharedState) };
            shared_state.device_connected = true;
        }
        let mut keys = vec![];
        {
            let state = unsafe { &mut *(shmem.as_ptr() as *mut SharedState) };
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
}

impl Drop for AppState {
    fn drop(&mut self) {
        //Perform cleanup
        let shared_state = unsafe { &mut *(self.shared_mem.as_ptr() as *mut SharedState) };

        shared_state.device_connected = false;
        shared_state.analog_values.iter_mut().for_each(|x| *x = 0);
    }
}

struct Key {
    keycode: u16,
    label: String,
    width: u16,
    _height: u16,
    value: f32,
    xy: (usize, usize),
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
            keycode,
            label,
            width,
            _height: height,
            value,
            xy,
        }
    }

    fn width(&self) -> Length {
        Length::FillPortion(KEY_WIDTH * self.width + (KEY_SPACING * (self.width - 1)))
    }

    fn height(&self) -> Length {
        Length::FillPortion(KEY_WIDTH)
    }

    fn view(&'_ self) -> Element<'_, Message> {
        let width = self.width();
        let height = self.height();

        if self.label.is_empty() {
            return Container::new(Column::new())
                .width(width)
                .height(height)
                .into();
        }

        Container::new(
            Column::new()
                .padding(5)
                .align_x(Alignment::Center)
                .push(Text::new(self.label.as_str()).align_x(alignment::Horizontal::Center))
                .push(
                    Text::new(format!("{:.3}", self.value.trunc() / 255f32)).color(
                        Color::from_rgb8(255 - self.value as u8, self.value as u8, 0),
                    ),
                )
                .push(Slider::new(0.0..=255.0, self.value, move |val| {
                    Message::SliderChanged(self.xy, val)
                })),
        )
        .height(height)
        .width(width)
        .style(|_theme| container::Style {
            text_color: None,
            background: None,
            border: Border {
                color: Color::BLACK,
                width: 1.0,
                radius: Radius::new(1.0),
            },
            shadow: Shadow::default(),
        })
        .into()
    }

    fn update(&mut self, shared_state: &mut Shmem, value: f32) {
        self.value = value;

        let v = unsafe { &mut *(shared_state.as_ptr() as *mut SharedState) };
        v.analog_values[self.keycode as usize] = self.value as u8;
    }
}

#[derive(Debug, PartialEq)]
pub struct SharedState {
    pub vendor_id: u16,
    /// Device Product ID `pid`
    pub product_id: u16,
    //TODO: Consider switching these to FFiStr
    /// Device Manufacturer name
    pub manufacturer_name: [u8; 20],
    /// Device name
    pub device_name: [u8; 20],

    pub device_type: DeviceType,

    pub device_connected: bool,
    pub dirty_device_info: bool,

    pub analog_values: [u8; 0xFF],
}

fn main() -> Result<(), iced::Error> {
    if let Err(e) =
        env_logger::Builder::from_env(Env::default().default_filter_or("info")).try_init()
    {
        error!("Failed to init env_logger: {}", e)
    }
    let kb: &Vec<Vec<KeyboardKey>> = KEYBOARD_LAYOUT.borrow();
    let max_key_width = kb.iter().fold(0, |current: u32, item| {
        current.max(
            item.iter()
                .fold(0, |width: u32, key: &KeyboardKey| width + key.2 as u32),
        )
    });
    let width: u32 = max_key_width * KEY_WIDTH as u32
        + ((max_key_width - 1) * KEY_SPACING as u32)
        + WIDGET_PADDING as u32 * 2;
    let rows = kb.len() as u32;
    // Add 1 to the number of rows for the Height for the extra row of controls
    let height =
        (rows + 1) * (KEY_WIDTH as u32) + (rows * KEY_SPACING as u32) + WIDGET_PADDING as u32 * 2;

    iced::application("Wooting Analog Virtual Keyboard", update, view)
        .window(window::Settings {
            size: iced::Size {
                width: width as f32,
                height: height as f32,
            },
            resizable: false,
            decorations: true,
            max_size: None,
            min_size: None,
            icon: None,
            transparent: false,
            level: window::Level::AlwaysOnTop,
            ..window::Settings::default()
        })
        .settings(Settings {
            default_text_size: iced::Pixels(18.0),
            ..Settings::default()
        })
        .exit_on_close_request(true)
        .run_with(|| (AppState::new(), iced::Task::none()))
}

fn update(state: &mut AppState, message: Message) {
    match message {
        Message::SliderChanged((x, y), val) => {
            state
                .keys
                .get_mut(y)
                .unwrap()
                .get_mut(x)
                .unwrap()
                .update(&mut state.shared_mem, val);
        }
        Message::ConnectedChanged(connection_changed) => {
            let shared_state = unsafe { &mut *(state.shared_mem.as_ptr() as *mut SharedState) };
            shared_state.device_connected = connection_changed;
        }
    }
}

fn view(state: &'_ AppState) -> Element<'_, Message> {
    let mut col = Column::new().spacing(KEY_SPACING);
    for key_row in state.keys.iter() {
        let mut row = Row::new();
        for key in key_row.iter() {
            row = row.push(key.view());
        }
        col = col.push(row.spacing(KEY_SPACING));
    }
    col.push(
        Row::new().push(
            Checkbox::new(
                "Device Connected",
                unsafe { &mut *(state.shared_mem.as_ptr() as *mut SharedState) }.device_connected,
            )
            .on_toggle(Message::ConnectedChanged),
        ),
    )
    .padding(WIDGET_PADDING)
    .into()
}

type KeyboardKey = (&'static str, u16, u16, u16);

static KEYBOARD_LAYOUT: LazyLock<Vec<Vec<KeyboardKey>>> = LazyLock::new(|| {
    vec![
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
    ]
});
