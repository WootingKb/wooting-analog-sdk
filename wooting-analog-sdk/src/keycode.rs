//! Inspect key codes, associated metadata and helper functions.

use bimap::BiMap;
use enum_primitive_derive::Primitive;
use log::warn;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, sync::LazyLock};

//<HID code, Scancode>
static SCANCODE_MAP: LazyLock<BiMap<u8, u16>> = LazyLock::new(|| {
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
    //bimap.insert(0x74, 0x0000); //OPEN

    bimap.insert(0x75, 0xe03b); //HELP

    //bimap.insert(0x77, 0x0000); //SELECT

    //bimap.insert(0x79, 0x0000); //AGAIN
    bimap.insert(0x7a, 0xe008); //UNDO
    bimap.insert(0x7b, 0xe017); //CUT
    bimap.insert(0x7c, 0xe018); //COPY
    bimap.insert(0x7d, 0xe00a); //PASTE
    //bimap.insert(0x7e, 0x0000); //FIND
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
});

//<HID code, VirtualKey> based on US layout
static HID_TO_VK_MAP_US: LazyLock<BiMap<u8, u8>> = LazyLock::new(|| {
    let mut bimap: BiMap<u8, u8> = BiMap::new();
    bimap.insert(0x04, 0x41); // US_A
    bimap.insert(0x05, 0x42); // US_B
    bimap.insert(0x06, 0x43); // US_C
    bimap.insert(0x07, 0x44); // US_D
    bimap.insert(0x08, 0x45); // US_E
    bimap.insert(0x09, 0x46); // US_F
    bimap.insert(0x0a, 0x47); // US_G
    bimap.insert(0x0b, 0x48); // US_H
    bimap.insert(0x0c, 0x49); // US_I
    bimap.insert(0x0d, 0x4A); // US_J
    bimap.insert(0x0e, 0x4B); // US_K
    bimap.insert(0x0f, 0x4C); // US_L
    bimap.insert(0x10, 0x4D); // US_M
    bimap.insert(0x11, 0x4E); // US_N
    bimap.insert(0x12, 0x4F); // US_O
    bimap.insert(0x13, 0x50); // US_P
    bimap.insert(0x14, 0x51); // US_Q
    bimap.insert(0x15, 0x52); // US_R
    bimap.insert(0x16, 0x53); // US_S
    bimap.insert(0x17, 0x54); // US_T
    bimap.insert(0x18, 0x55); // US_U
    bimap.insert(0x19, 0x56); // US_V
    bimap.insert(0x1a, 0x57); // US_W
    bimap.insert(0x1b, 0x58); // US_X
    bimap.insert(0x1c, 0x59); // US_Y
    bimap.insert(0x1d, 0x5A); // US_Z

    bimap.insert(0x1e, 0x31); // DIGIT1
    bimap.insert(0x1f, 0x32); // DIGIT2
    bimap.insert(0x20, 0x33); // DIGIT3
    bimap.insert(0x21, 0x34); // DIGIT4
    bimap.insert(0x22, 0x35); // DIGIT5
    bimap.insert(0x23, 0x36); // DIGIT6
    bimap.insert(0x24, 0x37); // DIGIT7
    bimap.insert(0x25, 0x38); // DIGIT8
    bimap.insert(0x26, 0x39); // DIGIT9
    bimap.insert(0x27, 0x30); // DIGIT0

    bimap.insert(0x29, 0x1B); // ESCAPE
    bimap.insert(0x2a, 0x08); // BACKSPACE
    bimap.insert(0x2b, 0x09); // TAB
    bimap.insert(0x2c, 0x20); // SPACE
    bimap.insert(0x2d, 0xBD); // MINUS
    bimap.insert(0x2e, 0xBB); // EQUAL
    bimap.insert(0x2f, 0xDB); // BRACKET_LEFT

    bimap.insert(0x30, 0xDD); // BRACKET_RIGHT
    bimap.insert(0x31, 0xDC); // BACKSLASH

    bimap.insert(0x33, 0xBA); // SEMICOLON
    bimap.insert(0x34, 0xDE); // QUOTE
    bimap.insert(0x35, 0xC0); // BACKQUOTE
    bimap.insert(0x36, 0xBC); // COMMA
    bimap.insert(0x37, 0xBE); // PERIOD

    bimap.insert(0x38, 0xBF); // SLASH
    bimap.insert(0x39, 0x14); // CAPS_LOCK

    bimap.insert(0x3a, 0x70); // F1
    bimap.insert(0x3b, 0x71); // F2
    bimap.insert(0x3c, 0x72); // F3
    bimap.insert(0x3d, 0x73); // F4
    bimap.insert(0x3e, 0x74); // F5
    bimap.insert(0x3f, 0x75); // F6
    bimap.insert(0x40, 0x76); // F7
    bimap.insert(0x41, 0x77); // F8
    bimap.insert(0x42, 0x78); // F9
    bimap.insert(0x43, 0x79); // F10
    bimap.insert(0x44, 0x7A); // F11
    bimap.insert(0x45, 0x7B); // F12

    bimap.insert(0x46, 0x2C); // PRINT_SCREEN
    bimap.insert(0x47, 0x91); // SCROLL_LOCK

    bimap.insert(0x48, 0x13); // PAUSE
    bimap.insert(0x49, 0x2D); // INSERT
    bimap.insert(0x4a, 0x24); // HOME
    bimap.insert(0x4b, 0x21); // PAGE_UP
    bimap.insert(0x4c, 0x2E); // DELETE
    bimap.insert(0x4d, 0x23); // END
    bimap.insert(0x4e, 0x22); // PAGE_DOWN

    bimap.insert(0x4f, 0x27); // ARROW_RIGHT
    bimap.insert(0x50, 0x25); // ARROW_LEFT
    bimap.insert(0x51, 0x28); // ARROW_DOWN
    bimap.insert(0x52, 0x26); // ARROW_UP

    bimap.insert(0x53, 0x90); // NUM_LOCK
    bimap.insert(0x54, 0x6F); // NUMPAD_DIVIDE
    bimap.insert(0x55, 0x6A); // NUMPAD_MULTIPLY
    bimap.insert(0x56, 0x6D); // NUMPAD_SUBTRACT
    bimap.insert(0x57, 0x6B); // NUMPAD_ADD
    bimap.insert(0x58, 0x0D); // NUMPAD_ENTER
    bimap.insert(0x59, 0x61); // NUMPAD1
    bimap.insert(0x5a, 0x62); // NUMPAD2
    bimap.insert(0x5b, 0x63); // NUMPAD3
    bimap.insert(0x5c, 0x64); // NUMPAD4
    bimap.insert(0x5d, 0x65); // NUMPAD5
    bimap.insert(0x5e, 0x66); // NUMPAD6
    bimap.insert(0x5f, 0x67); // NUMPAD7
    bimap.insert(0x60, 0x68); // NUMPAD8
    bimap.insert(0x61, 0x69); // NUMPAD9
    bimap.insert(0x62, 0x60); // NUMPAD0
    bimap.insert(0x63, 0x6E); // NUMPAD_DECIMAL

    bimap.insert(0x28, 0x0D); // ENTER (moved below NUMPAD_ENTER to ensure vk_to_hid(0x0D) responds with 0x28 instead of 0x58)

    bimap.insert(0x64, 0xE2); // INTL_BACKSLASH
    bimap.insert(0x65, 0x5D); // CONTEXT_MENU
    bimap.insert(0x67, 0x0B); // NUMPAD_EQUAL

    bimap.insert(0x68, 0x7C); // F13
    bimap.insert(0x69, 0x7D); // F14
    bimap.insert(0x6a, 0x7E); // F15
    bimap.insert(0x6b, 0x7F); // F16
    bimap.insert(0x6c, 0x80); // F17
    bimap.insert(0x6d, 0x81); // F18
    bimap.insert(0x6e, 0x82); // F19
    bimap.insert(0x6f, 0x83); // F20
    bimap.insert(0x70, 0x84); // F21
    bimap.insert(0x71, 0x85); // F22
    bimap.insert(0x72, 0x86); // F23
    bimap.insert(0x73, 0x87); // F24

    bimap.insert(0x75, 0x2F); // HELP

    bimap.insert(0xe0, 0xa2); // CONTROL_LEFT
    bimap.insert(0xe1, 0xa0); // SHIFT_LEFT
    bimap.insert(0xe2, 0xa4); // ALT_LEFT
    bimap.insert(0xe3, 0x5b); // META_LEFT
    bimap.insert(0xe4, 0xa3); // CONTROL_RIGHT
    bimap.insert(0xe5, 0xa1); // SHIFT_RIGHT
    bimap.insert(0xe6, 0xa5); // ALT_RIGHT
    bimap.insert(0xe7, 0x5c); // META_RIGHT

    bimap
});

//<VirtualKey, Scancode>
static VIRTUALKEY_OVERRIDE: LazyLock<BiMap<u8, u16>> = LazyLock::new(|| {
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
});

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Clone, Primitive, Default)]
#[repr(C)]
pub enum KeycodeType {
    /// USB HID Keycodes <https://www.usb.org/document-library/hid-usage-tables-112> pg53
    #[default]
    HID = 0,
    /// Scan code set 1
    ScanCode1 = 1,
    /// Windows Virtual Keys
    VirtualKey = 2,
    /// Windows Virtual Keys which are translated to the current keyboard locale
    VirtualKeyTranslate = 3,
}

/// A group of Wooting key namespaces.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
#[repr(C)]
pub enum KeyNamespace {
    HidNormal = 0,
    HidFunction = 3,
    CustomFunction = 4,
    GamepadBinding = 5,
    AdvancedKey = 6,
    Unknown = 255,
}

impl From<u8> for KeyNamespace {
    fn from(value: u8) -> Self {
        match value {
            0 => KeyNamespace::HidNormal,
            3 => KeyNamespace::HidFunction,
            4 => KeyNamespace::CustomFunction,
            5 => KeyNamespace::GamepadBinding,
            6 => KeyNamespace::AdvancedKey,
            other => {
                warn!("missing or invalid key namespace: {other}");
                KeyNamespace::Unknown
            }
        }
    }
}

/// Any additional information a [`KeyCode`] can contain.
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[repr(C, u8)]
pub enum KeyMetadata {
    #[default]
    None,
    Basic {
        namespace: KeyNamespace,
    },

    // Reserve 8 bytes to ensure we can avoid shifting the memory layout of the union a litte while
    // longer. As soon as this type does shift a new enum should be created and used instead, while
    // also keeping this one around for backwards compatibility.
    #[doc(hidden)]
    _Reserved([u8; 8]) = 255,
}

/// An analog key code with optional metadata.
///
/// The metadata is only present if the device supplying the data has support for it in the analog
/// protocol. Some plugins or devices might run older firmware or simply don’t have the extra data
/// associated with a key press yielding no valuable extra data, other than the key code.
#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct KeyCode {
    pub(crate) inner: u16,
    pub(crate) metadata: KeyMetadata,
}

impl KeyCode {
    pub(crate) fn with_namespace(raw: u16) -> Self {
        let namespace = KeyNamespace::from((raw >> 8) as u8);

        Self {
            inner: raw,
            metadata: KeyMetadata::Basic { namespace },
        }
    }

    pub fn metadata(&self) -> &KeyMetadata {
        &self.metadata
    }

    pub fn is_advanced_key(&self) -> bool {
        matches!(
            self.metadata,
            KeyMetadata::Basic {
                namespace: KeyNamespace::AdvancedKey,
                ..
            }
        )
    }
}

impl std::fmt::Display for KeyCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl std::hash::Hash for KeyCode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl Eq for KeyCode {}

impl PartialEq for KeyCode {
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl PartialOrd for KeyCode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for KeyCode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl From<u8> for KeyCode {
    fn from(value: u8) -> Self {
        KeyCode {
            inner: u16::from(value),
            metadata: KeyMetadata::None,
        }
    }
}

impl From<u16> for KeyCode {
    fn from(value: u16) -> Self {
        KeyCode {
            inner: value,
            metadata: KeyMetadata::None,
        }
    }
}

impl From<KeyCode> for u16 {
    fn from(value: KeyCode) -> Self {
        value.inner
    }
}

impl From<&KeyCode> for u16 {
    fn from(value: &KeyCode) -> Self {
        value.inner
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, PartialEq, Clone, Hash, Eq, Primitive)]
#[repr(C)]
pub enum HIDCodes {
    A = 0x04,
    B = 0x05, //US_B
    C = 0x06, //US_C
    D = 0x07, //US_D

    E = 0x08, //US_E
    F = 0x09, //US_F
    G = 0x0a, //US_G
    H = 0x0b, //US_H
    I = 0x0c, //US_I
    J = 0x0d, //US_J
    K = 0x0e, //US_K
    L = 0x0f, //US_L

    M = 0x10, //US_M
    N = 0x11, //US_N
    O = 0x12, //US_O
    P = 0x13, //US_P
    Q = 0x14, //US_Q
    R = 0x15, //US_R
    S = 0x16, //US_S
    T = 0x17, //US_T

    U = 0x18,  //US_U
    V = 0x19,  //US_V
    W = 0x1a,  //US_W
    X = 0x1b,  //US_X
    Y = 0x1c,  //US_Y
    Z = 0x1d,  //US_Z
    N1 = 0x1e, //DIGIT1
    N2 = 0x1f, //DIGIT2

    N3 = 0x20, //DIGIT3
    N4 = 0x21, //DIGIT4
    N5 = 0x22, //DIGIT5
    N6 = 0x23, //DIGIT6
    N7 = 0x24, //DIGIT7
    N8 = 0x25, //DIGIT8
    N9 = 0x26, //DIGIT9
    N0 = 0x27, //DIGIT0

    Enter = 0x28,       //ENTER
    Escape = 0x29,      //ESCAPE
    Backspace = 0x2a,   //BACKSPACE
    Tab = 0x2b,         //TAB
    Space = 0x2c,       //SPACE
    Minus = 0x2d,       //MINUS
    Equal = 0x2e,       //EQUAL
    BracketLeft = 0x2f, //BRACKET_LEFT

    BracketRight = 0x30, //BRACKET_RIGHT
    Backslash = 0x31,    //BACKSLASH

    // = 0x32, //INTL_HASH
    Semicolon = 0x33, //SEMICOLON
    Quote = 0x34,     //QUOTE
    Backquote = 0x35, //BACKQUOTE
    Comma = 0x36,     //COMMA
    Period = 0x37,    //PERIOD

    Slash = 0x38,    //SLASH
    CapsLock = 0x39, //CAPS_LOCK
    F1 = 0x3a,       //F1
    F2 = 0x3b,       //F2
    F3 = 0x3c,       //F3
    F4 = 0x3d,       //F4
    F5 = 0x3e,       //F5
    F6 = 0x3f,       //F6

    F7 = 0x40,          //F7
    F8 = 0x41,          //F8
    F9 = 0x42,          //F9
    F10 = 0x43,         //F10
    F11 = 0x44,         //F11
    F12 = 0x45,         //F12
    PrintScreen = 0x46, //PRINT_SCREEN
    ScrollLock = 0x47,  //SCROLL_LOCK

    PauseBreak = 0x48, //PAUSE
    Insert = 0x49,     //INSERT
    Home = 0x4a,       //HOME
    PageUp = 0x4b,     //PAGE_UP
    Delete = 0x4c,     //DEL
    End = 0x4d,        //END
    PageDown = 0x4e,   //PAGE_DOWN
    ArrowRight = 0x4f, //ARROW_RIGHT

    ArrowLeft = 0x50,      //ARROW_LEFT
    ArrowDown = 0x51,      //ARROW_DOWN
    ArrowUp = 0x52,        //ARROW_UP
    NumLock = 0x53,        //NUM_LOCK
    NumpadDivide = 0x54,   //NUMPAD_DIVIDE
    NumpadMultiply = 0x55, //NUMPAD_MULTIPLY
    NumpadSubtract = 0x56, //NUMPAD_SUBTRACT
    NumpadAdd = 0x57,      //NUMPAD_ADD

    NumpadEnter = 0x58, //NUMPAD_ENTER
    Numpad1 = 0x59,     //NUMPAD1
    Numpad2 = 0x5a,     //NUMPAD2
    Numpad3 = 0x5b,     //NUMPAD3
    Numpad4 = 0x5c,     //NUMPAD4
    Numpad5 = 0x5d,     //NUMPAD5
    Numpad6 = 0x5e,     //NUMPAD6
    Numpad7 = 0x5f,     //NUMPAD7

    Numpad8 = 0x60,                //NUMPAD8
    Numpad9 = 0x61,                //NUMPAD9
    Numpad0 = 0x62,                //NUMPAD0
    NumpadDecimal = 0x63,          //NUMPAD_DECIMAL
    InternationalBackslash = 0x64, //INTL_BACKSLASH
    ContextMenu = 0x65,            //CONTEXT_MENU
    Power = 0x66,                  //POWER
    NumpadEqual = 0x67,            //NUMPAD_EQUAL

    F13 = 0x68, //F13
    F14 = 0x69, //F14
    F15 = 0x6a, //F15
    F16 = 0x6b, //F16
    F17 = 0x6c, //F17
    F18 = 0x6d, //F18
    F19 = 0x6e, //F19
    F20 = 0x6f, //F20

    F21 = 0x70, //F21
    F22 = 0x71, //F22
    F23 = 0x72, //F23

    F24 = 0x73,  //F24
    Open = 0x74, //OPEN

    Help = 0x75, //HELP

    // = 0x77, //SELECT
    Again = 0x79,      //AGAIN
    Undo = 0x7a,       //UNDO
    Cut = 0x7b,        //CUT
    Copy = 0x7c,       //COPY
    Paste = 0x7d,      //PASTE
    Find = 0x7e,       //FIND
    VolumeMute = 0x7f, //VOLUME_MUTE

    VolumeUp = 0x80,    //VOLUME_UP
    VolumeDown = 0x81,  //VOLUME_DOWN
    NumpadComma = 0x85, //NUMPAD_COMMA

    InternationalRO = 0x87,  //INTL_RO
    KanaMode = 0x88,         //KANA_MODE
    InternationalYen = 0x89, //INTL_YEN
    Convert = 0x8a,          //CONVERT
    NonConvert = 0x8b,       //NON_CONVERT
    Lang1 = 0x90,            //LANG1
    Lang2 = 0x91,            //LANG2
    Lang3 = 0x92,            //LANG3
    Lang4 = 0x93,            //LANG4

    LeftCtrl = 0xe0,   //CONTROL_LEFT
    LeftShift = 0xe1,  //SHIFT_LEFT
    LeftAlt = 0xe2,    //ALT_LEFT
    LeftMeta = 0xe3,   //META_LEFT
    RightCtrl = 0xe4,  //CONTROL_RIGHT
    RightShift = 0xe5, //SHIFT_RIGHT
    RightAlt = 0xe6,   //ALT_RIGHT
    RightMeta = 0xe7,  //META_RIGHT
}

pub(crate) fn vk_to_hid(vk: u16) -> Option<u16> {
    if let Some(&hid) = HID_TO_VK_MAP_US.get_by_right(&(vk as u8)) {
        Some(hid as u16)
    } else {
        None
    }
}

#[allow(unused)] // Suppress warning about 'vk' being unused on non-windows
pub(crate) fn vk_to_hid_translate(vk: u16) -> Option<u16> {
    #[cfg(windows)]
    {
        let scancode: u16;
        if let Some(&code) = VIRTUALKEY_OVERRIDE.get_by_left(&(vk as u8)) {
            scancode = code;
        } else {
            use winapi::um::winuser::MapVirtualKeyA;
            scancode = unsafe { MapVirtualKeyA(vk.into(), 0) as u16 };

            if scancode == 0 {
                return None;
            }
        }
        scancode_to_hid(scancode)
    }

    #[cfg(not(windows))]
    None
}

pub(crate) fn hid_to_vk(hid: u16) -> Option<u16> {
    if let Some(&vk) = HID_TO_VK_MAP_US.get_by_left(&(hid as u8)) {
        Some(vk as u16)
    } else {
        None
    }
}

#[allow(unused)] // Suppress warning about 'hid' being unused on non-windows
pub(crate) fn hid_to_vk_translate(hid: u16) -> Option<u16> {
    #[cfg(windows)]
    if let Some(scancode) = hid_to_scancode(hid) {
        if let Some(&hid) = VIRTUALKEY_OVERRIDE.get_by_right(&scancode) {
            return Some(hid as u16);
        }

        use winapi::um::winuser::MapVirtualKeyA;
        let vk: u32 = unsafe { MapVirtualKeyA(scancode.into(), 3) };

        if vk == 0 {
            return None;
        }

        Some(vk as u16)
    } else {
        None
    }

    #[cfg(not(windows))]
    None
}

pub(crate) fn hid_to_scancode(code: u16) -> Option<u16> {
    SCANCODE_MAP.get_by_left(&(code as u8)).copied()
}

pub(crate) fn scancode_to_hid(code: u16) -> Option<u16> {
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

pub(crate) fn code_to_hid(code: u16, mode: &KeycodeType) -> Option<u16> {
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
        KeycodeType::VirtualKey => vk_to_hid(code),
        KeycodeType::VirtualKeyTranslate => vk_to_hid_translate(code),
    }
}

pub(crate) fn hid_to_code<T>(code: T, mode: &KeycodeType) -> Option<u16>
where
    T: Into<u16>,
{
    let code = code.into();

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
        KeycodeType::VirtualKey => hid_to_vk(code),
        KeycodeType::VirtualKeyTranslate => hid_to_vk_translate(code),
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
        let keycode_types = [
            KeycodeType::HID,
            KeycodeType::ScanCode1,
            KeycodeType::VirtualKey,
        ];
        for code in 0..0xFFFF {
            let prefix = (code & 0xFF00) >> 8;
            match prefix {
                //Initial range
                0x00 => {
                    for keycode in keycode_types.iter() {
                        if *keycode == KeycodeType::ScanCode1 {
                            let val = hid_to_code(code, keycode).unwrap_or(0);
                            assert!(
                                val < 0x100 || (val & 0xFF00) == 0xE000 || (val & 0xFF00) == 0xE100
                            );
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
