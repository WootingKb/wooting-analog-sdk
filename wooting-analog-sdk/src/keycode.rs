//use scancode::Scancode;
use bimap::BiMap;
use wooting_analog_common::KeycodeType;

lazy_static! {
    //<HID code, Scancode>
    static ref SCANCODE_MAP: BiMap<u8, u16> = {
        let mut bimap: BiMap<u8, u16> = BiMap::new();
        bimap.insert(0x04, 0x001e); //US_A
        bimap.insert(0x05, 0x0030); //US_B
        bimap.insert(0x06, 0x002e); //US_C
        bimap.insert(0x07, 0x0020); //US_D

        bimap.insert(0x08, 0x0012); //US_E
        bimap.insert(0x09, 0x0021); //US_F
        bimap.insert(0x0a, 0x0022); //US_G
        bimap.insert(0x0b, 0x0023); //US_H
        bimap.insert(0x0c, 0x0017); //US_I
        bimap.insert(0x0d, 0x0024); //US_J
        bimap.insert(0x0e, 0x0025); //US_K
        bimap.insert(0x0f, 0x0026); //US_L

        bimap.insert(0x10, 0x0032); //US_M
        bimap.insert(0x11, 0x0031); //US_N
        bimap.insert(0x12, 0x0018); //US_O
        bimap.insert(0x13, 0x0019); //US_P
        bimap.insert(0x14, 0x0010); //US_Q
        bimap.insert(0x15, 0x0013); //US_R
        bimap.insert(0x16, 0x001f); //US_S
        bimap.insert(0x17, 0x0014); //US_T

        bimap.insert(0x18, 0x0016); //US_U
        bimap.insert(0x19, 0x002f); //US_V
        bimap.insert(0x1a, 0x0011); //US_W
        bimap.insert(0x1b, 0x002d); //US_X
        bimap.insert(0x1c, 0x0015); //US_Y
        bimap.insert(0x1d, 0x002c); //US_Z
        bimap.insert(0x1e, 0x0002); //DIGIT1
        bimap.insert(0x1f, 0x0003); //DIGIT2

        bimap.insert(0x20, 0x0004); //DIGIT3
        bimap.insert(0x21, 0x0005); //DIGIT4
        bimap.insert(0x22, 0x0006); //DIGIT5
        bimap.insert(0x23, 0x0007); //DIGIT6
        bimap.insert(0x24, 0x0008); //DIGIT7
        bimap.insert(0x25, 0x0009); //DIGIT8
        bimap.insert(0x26, 0x000a); //DIGIT9
        bimap.insert(0x27, 0x000b); //DIGIT0

        bimap.insert(0x28, 0x001c); //ENTER
        bimap.insert(0x29, 0x0001); //ESCAPE
        bimap.insert(0x2a, 0x000e); //BACKSPACE
        bimap.insert(0x2b, 0x000f); //TAB
        bimap.insert(0x2c, 0x0039); //SPACE
        bimap.insert(0x2d, 0x000c); //MINUS
        bimap.insert(0x2e, 0x000d); //EQUAL
        bimap.insert(0x2f, 0x001a); //BRACKET_LEFT

        bimap.insert(0x30, 0x001b); //BRACKET_RIGHT
        bimap.insert(0x31, 0x002b); //BACKSLASH

        //bimap.insert(0x32, 0x0000); //INTL_HASH

        bimap.insert(0x33, 0x0027); //SEMICOLON
        bimap.insert(0x34, 0x0028); //QUOTE
        bimap.insert(0x35, 0x0029); //BACKQUOTE
        bimap.insert(0x36, 0x0033); //COMMA
        bimap.insert(0x37, 0x0034); //PERIOD

        bimap.insert(0x38, 0x0035); //SLASH
        bimap.insert(0x39, 0x003a); //CAPS_LOCK
        bimap.insert(0x3a, 0x003b); //F1
        bimap.insert(0x3b, 0x003c); //F2
        bimap.insert(0x3c, 0x003d); //F3
        bimap.insert(0x3d, 0x003e); //F4
        bimap.insert(0x3e, 0x003f); //F5
        bimap.insert(0x3f, 0x0040); //F6

        bimap.insert(0x40, 0x0041); //F7
        bimap.insert(0x41, 0x0042); //F8
        bimap.insert(0x42, 0x0043); //F9
        bimap.insert(0x43, 0x0044); //F10
        bimap.insert(0x44, 0x0057); //F11
        bimap.insert(0x45, 0x0058); //F12
        bimap.insert(0x46, 0xe037); //PRINT_SCREEN
        bimap.insert(0x47, 0x0046); //SCROLL_LOCK

        bimap.insert(0x48, 0xe11d); //PAUSE
        bimap.insert(0x49, 0xe052); //INSERT
        bimap.insert(0x4a, 0xe047); //HOME
        bimap.insert(0x4b, 0xe049); //PAGE_UP
        bimap.insert(0x4c, 0xe053); //DEL
        bimap.insert(0x4d, 0xe04f); //END
        bimap.insert(0x4e, 0xe051); //PAGE_DOWN
        bimap.insert(0x4f, 0xe04d); //ARROW_RIGHT

        bimap.insert(0x50, 0xe04b); //ARROW_LEFT
        bimap.insert(0x51, 0xe050); //ARROW_DOWN
        bimap.insert(0x52, 0xe048); //ARROW_UP
        bimap.insert(0x53, 0x0045); //NUM_LOCK
        bimap.insert(0x54, 0xe035); //NUMPAD_DIVIDE
        bimap.insert(0x55, 0x0037); //NUMPAD_MULTIPLY
        bimap.insert(0x56, 0x004a); //NUMPAD_SUBTRACT
        bimap.insert(0x57, 0x004e); //NUMPAD_ADD

        bimap.insert(0x58, 0xe01c); //NUMPAD_ENTER
        bimap.insert(0x59, 0x004f); //NUMPAD1
        bimap.insert(0x5a, 0x0050); //NUMPAD2
        bimap.insert(0x5b, 0x0051); //NUMPAD3
        bimap.insert(0x5c, 0x004b); //NUMPAD4
        bimap.insert(0x5d, 0x004c); //NUMPAD5
        bimap.insert(0x5e, 0x004d); //NUMPAD6
        bimap.insert(0x5f, 0x0047); //NUMPAD7

        bimap.insert(0x60, 0x0048); //NUMPAD8
        bimap.insert(0x61, 0x0049); //NUMPAD9
        bimap.insert(0x62, 0x0052); //NUMPAD0
        bimap.insert(0x63, 0x0053); //NUMPAD_DECIMAL
        bimap.insert(0x64, 0x0056); //INTL_BACKSLASH
        bimap.insert(0x65, 0xe05d); //CONTEXT_MENU
        bimap.insert(0x66, 0xe05e); //POWER
        bimap.insert(0x67, 0x0059); //NUMPAD_EQUAL

        bimap.insert(0x68, 0x0064); //F13
        bimap.insert(0x69, 0x0065); //F14
        bimap.insert(0x6a, 0x0066); //F15
        bimap.insert(0x6b, 0x0067); //F16
        bimap.insert(0x6c, 0x0068); //F17
        bimap.insert(0x6d, 0x0069); //F18
        bimap.insert(0x6e, 0x006a); //F19
        bimap.insert(0x6f, 0x006b); //F20

        bimap.insert(0x70, 0x006c); //F21
        bimap.insert(0x71, 0x006d); //F22
        bimap.insert(0x72, 0x006e); //F23

        bimap.insert(0x73, 0x0076); //F24
        bimap.insert(0x74, 0x0000); //OPEN

        bimap.insert(0x75, 0xe03b); //HELP

        //bimap.insert(0x77, 0x0000); //SELECT

        bimap.insert(0x79, 0x0000); //AGAIN
        bimap.insert(0x7a, 0xe008); //UNDO
        bimap.insert(0x7b, 0xe017); //CUT
        bimap.insert(0x7c, 0xe018); //COPY
        bimap.insert(0x7d, 0xe00a); //PASTE
        bimap.insert(0x7e, 0x0000); //FIND
        bimap.insert(0x7f, 0xe020); //VOLUME_MUTE

        bimap.insert(0x80, 0xe030); //VOLUME_UP
        bimap.insert(0x81, 0xe02e); //VOLUME_DOWN
        bimap.insert(0x85, 0x007e); //NUMPAD_COMMA

        bimap.insert(0x87, 0x0073); //INTL_RO
        bimap.insert(0x88, 0x0070); //KANA_MODE
        bimap.insert(0x89, 0x007d); //INTL_YEN
        bimap.insert(0x8a, 0x0079); //CONVERT
        bimap.insert(0x8b, 0x007b); //NON_CONVERT
        bimap.insert(0x90, 0x0072); //LANG1
        bimap.insert(0x91, 0x0071); //LANG2
        bimap.insert(0x92, 0x0078); //LANG3
        bimap.insert(0x93, 0x0077); //LANG4

        bimap.insert(0xe0, 0x001d); //CONTROL_LEFT
        bimap.insert(0xe1, 0x002a); //SHIFT_LEFT
        bimap.insert(0xe2, 0x0038); //ALT_LEFT
        bimap.insert(0xe3, 0xe05b); //META_LEFT
        bimap.insert(0xe4, 0xe01d); //CONTROL_RIGHT
        bimap.insert(0xe5, 0x0036); //SHIFT_RIGHT
        bimap.insert(0xe6, 0xe038); //ALT_RIGHT
        bimap.insert(0xe7, 0xe05c); //META_RIGHT
        bimap
    };

                                            //VirtualKey, Scancode
     static ref VIRTUALKEY_OVERRIDE: BiMap<u8, u16> = {
        let mut bimap: BiMap<u8, u16> = BiMap::new();
        bimap.insert(0x60, 0x0052); //NUMPAD0
        bimap.insert(0x61, 0x004f); //NUMPAD1
        bimap.insert(0x62, 0x0050); //NUMPAD2
        bimap.insert(0x63, 0x0051); //NUMPAD3
        bimap.insert(0x64, 0x004b); //NUMPAD4
        bimap.insert(0x65, 0x004c); //NUMPAD5
        bimap.insert(0x66, 0x004d); //NUMPAD6
        bimap.insert(0x67, 0x0047); //NUMPAD7
        bimap.insert(0x68, 0x0048); //NUMPAD8
        bimap.insert(0x69, 0x0049); //NUMPAD9
        bimap.insert(0x6E, 0x0053); //NUMPAD_DECIMAL
        bimap
    };

}

#[allow(unused)]
pub fn vk_to_scancode(code: u16, translate: bool) -> Option<u16> {
    #[cfg(windows)]
    unsafe {
        if let Some(&code) = VIRTUALKEY_OVERRIDE.get_by_left(&(code as u8)) {
            return Some(code);
        }

        use winapi::um::winuser::{
            GetForegroundWindow, GetKeyboardLayout, GetWindowThreadProcessId, MapVirtualKeyA,
            MapVirtualKeyExA,
        };
        let scancode: u32;
        if translate {
            let window_handle = GetForegroundWindow();
            let thread = GetWindowThreadProcessId(window_handle, 0 as *mut u32);
            let layout = GetKeyboardLayout(thread);
            scancode = MapVirtualKeyExA(code.into(), 4, layout);
        //println!("Window handle: {:?}, thread: {:?}, layout: {:?}, code: {} scancode: {}", window_handle, thread, layout, code, scancode);
        } else {
            scancode = MapVirtualKeyA(code.into(), 0);
        }

        if scancode == 0 {
            return None;
        }

        return Some(scancode as u16);
    }

    #[cfg(not(windows))]
    None
}

#[allow(unused)]
pub fn scancode_to_vk(code: u16, translate: bool) -> Option<u16> {
    #[cfg(windows)]
    unsafe {
        if let Some(&code) = VIRTUALKEY_OVERRIDE.get_by_right(&code) {
            return Some(code as u16);
        }

        use winapi::um::winuser::{
            GetForegroundWindow, GetKeyboardLayout, GetWindowThreadProcessId, MapVirtualKeyA,
            MapVirtualKeyExA,
        };
        let scancode: u32;
        if translate {
            let window_handle = GetForegroundWindow();
            let thread = GetWindowThreadProcessId(window_handle, 0 as *mut u32);
            let layout = GetKeyboardLayout(thread);
            scancode = MapVirtualKeyExA(code.into(), 3, layout);
        //println!("Window handle: {:?}, thread: {:?}, layout: {:?}, code: {} scancode: {}", window_handle, thread, layout, code, scancode);
        } else {
            scancode = MapVirtualKeyA(code.into(), 3);
        }

        if scancode == 0 {
            return None;
        }

        return Some(scancode as u16);
    }

    #[cfg(not(windows))]
    None
}

pub fn hid_to_scancode(code: u16) -> Option<u16> {
    SCANCODE_MAP.get_by_left(&(code as u8)).copied()
}

pub fn scancode_to_hid(code: u16) -> Option<u16> {
    let sc = {
        if (code & 0xFF00) == 0x100 {
            0xE000 | (code & 0xFF)
        } else {
            code
        }
    };
    SCANCODE_MAP.get_by_right(&sc).map(|&c| u16::from(c))
    /*match Scancode::new(code as u8) {
        Some(hid) => {
            Some(hid as u16)
        },
        None => {
            None
        }
    }*/
}

pub fn code_to_hid(code: u16, mode: &KeycodeType) -> Option<u16> {
    let prefix = (code & 0xFF00) >> 8;
    //Check if the code is a custom key, if it is, just straight return it. Additionally checking it isn't prefixed with the ScanCode 1 escape code
    if code >= 0x200 && prefix != 0xE0 {
        return Some(code);
    } else if prefix != 0 {
        return None;
    }

    match &mode {
        KeycodeType::HID => {
            if prefix == 0xE0 || prefix == 0x1 {
                None
            } else {
                Some(code)
            }
        }
        KeycodeType::ScanCode1 => scancode_to_hid(code),
        KeycodeType::VirtualKey => vk_to_scancode(code, false).and_then(scancode_to_hid),
        KeycodeType::VirtualKeyTranslate => vk_to_scancode(code, true).and_then(scancode_to_hid),
    }
}

pub fn hid_to_code(code: u16, mode: &KeycodeType) -> Option<u16> {
    let prefix = (code & 0xFF00) >> 8;
    //Check if the code is a custom key, if it is, just straight return it. Additionally checking it isn't prefixed with the ScanCode 1 escape code
    if code >= 0x200 && prefix != 0xE0 {
        return Some(code);
    } else if prefix != 0 {
        return None;
    }

    match &mode {
        KeycodeType::HID => {
            if prefix == 0xE0 || prefix == 0x1 {
                None
            } else {
                Some(code)
            }
        }
        KeycodeType::ScanCode1 => hid_to_scancode(code),
        KeycodeType::VirtualKey => {
            hid_to_scancode(code).and_then(|code| scancode_to_vk(code, false))
        }
        KeycodeType::VirtualKeyTranslate => {
            hid_to_scancode(code).and_then(|code| scancode_to_vk(code, true))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn t_code_to_hid() {
        #[cfg(windows)]
        let keycode_types = [
            KeycodeType::HID,
            KeycodeType::ScanCode1,
            KeycodeType::VirtualKey,
            KeycodeType::VirtualKeyTranslate,
        ];
        #[cfg(not(windows))]
        let keycode_types = [KeycodeType::HID, KeycodeType::ScanCode1];
        for code in 0..0xFFFF {
            let prefix = (code & 0xFF00) >> 8;
            match prefix {
                //Initial range
                0x00 => {
                    for keycode in keycode_types.iter() {
                        if *keycode == KeycodeType::ScanCode1 {
                            let val = hid_to_code(code, keycode).unwrap_or(0);
                            assert!(val < 0x100 || (val & 0xFF00) == 0xE000|| (val & 0xFF00) == 0xE100);
                        } else {
                            assert!(hid_to_code(code, keycode).unwrap_or(0) < 0x100);
                        }
                    }
                }

                //ScanCode protected
                0x01 | 0xE0 => {
                    for keycode in keycode_types.iter() {
                        if *keycode == KeycodeType::ScanCode1 {
                            continue;
                        }
                        //Ensure that the special code is unused in all mappings other than scancode
                        assert!(code_to_hid(code, keycode).is_none());
                        assert!(hid_to_code(code, keycode).is_none());
                    }
                }

                //Custom keys
                _x => {
                    //Ensure custom codes are unchanged in all keycode types
                    for keycode in keycode_types.iter() {
                        assert_eq!(code_to_hid(code, keycode).unwrap(), code);
                        assert_eq!(hid_to_code(code, keycode).unwrap(), code);
                    }
                }
            }
        }
    }

    #[test]
    fn scancode_test() {
        assert_eq!(scancode_to_hid(0x0001).unwrap(), 0x29);
        //Test if the 0x1 is translated to the 0xE0 correctly
        assert_eq!(scancode_to_hid(0x152).unwrap(), 0x49);
    }
}
