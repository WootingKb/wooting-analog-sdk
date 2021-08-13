# import the contents of the Rust library into the Python extension
from .wooting_analog_wrapper import *
from enum import IntEnum

class KeycodeType(IntEnum):
    # USB HID Keycodes https:#www.usb.org/document-library/hid-usage-tables-112 pg53
    HID = 0
    # Scan code set 1
    ScanCode1 = 1
    # Windows Virtual Keys
    VirtualKey = 2
    # Windows Virtual Keys which are translated to the current keyboard locale
    VirtualKeyTranslate = 3

class HIDCodes(IntEnum):
    A = 0x04
    B = 0x05 #US_B
    C = 0x06 #US_C
    D = 0x07 #US_D

    E = 0x08 #US_E
    F = 0x09 #US_F
    G = 0x0a #US_G
    H = 0x0b #US_H
    I = 0x0c #US_I
    J = 0x0d #US_J
    K = 0x0e #US_K
    L = 0x0f #US_L

    M = 0x10 #US_M
    N = 0x11 #US_N
    O = 0x12 #US_O
    P = 0x13 #US_P
    Q = 0x14 #US_Q
    R = 0x15 #US_R
    S = 0x16 #US_S
    T = 0x17 #US_T

    U = 0x18  #US_U
    V = 0x19  #US_V
    W = 0x1a  #US_W
    X = 0x1b  #US_X
    Y = 0x1c  #US_Y
    Z = 0x1d  #US_Z
    N1 = 0x1e #DIGIT1
    N2 = 0x1f #DIGIT2

    N3 = 0x20 #DIGIT3
    N4 = 0x21 #DIGIT4
    N5 = 0x22 #DIGIT5
    N6 = 0x23 #DIGIT6
    N7 = 0x24 #DIGIT7
    N8 = 0x25 #DIGIT8
    N9 = 0x26 #DIGIT9
    N0 = 0x27 #DIGIT0

    Enter = 0x28       #ENTER
    Escape = 0x29      #ESCAPE
    Backspace = 0x2a   #BACKSPACE
    Tab = 0x2b         #TAB
    Space = 0x2c       #SPACE
    Minus = 0x2d       #MINUS
    Equal = 0x2e       #EQUAL
    BracketLeft = 0x2f #BRACKET_LEFT

    BracketRight = 0x30 #BRACKET_RIGHT
    Backslash = 0x31    #BACKSLASH

    # = 0x32 #INTL_HASH
    Semicolon = 0x33 #SEMICOLON
    Quote = 0x34     #QUOTE
    Backquote = 0x35 #BACKQUOTE
    Comma = 0x36     #COMMA
    Period = 0x37    #PERIOD

    Slash = 0x38    #SLASH
    CapsLock = 0x39 #CAPS_LOCK
    F1 = 0x3a       #F1
    F2 = 0x3b       #F2
    F3 = 0x3c       #F3
    F4 = 0x3d       #F4
    F5 = 0x3e       #F5
    F6 = 0x3f       #F6

    F7 = 0x40          #F7
    F8 = 0x41          #F8
    F9 = 0x42          #F9
    F10 = 0x43         #F10
    F11 = 0x44         #F11
    F12 = 0x45         #F12
    PrintScreen = 0x46 #PRINT_SCREEN
    ScrollLock = 0x47  #SCROLL_LOCK

    PauseBreak = 0x48 #PAUSE
    Insert = 0x49     #INSERT
    Home = 0x4a       #HOME
    PageUp = 0x4b     #PAGE_UP
    Delete = 0x4c     #DEL
    End = 0x4d        #END
    PageDown = 0x4e   #PAGE_DOWN
    ArrowRight = 0x4f #ARROW_RIGHT

    ArrowLeft = 0x50      #ARROW_LEFT
    ArrowDown = 0x51      #ARROW_DOWN
    ArrowUp = 0x52        #ARROW_UP
    NumLock = 0x53        #NUM_LOCK
    NumpadDivide = 0x54   #NUMPAD_DIVIDE
    NumpadMultiply = 0x55 #NUMPAD_MULTIPLY
    NumpadSubtract = 0x56 #NUMPAD_SUBTRACT
    NumpadAdd = 0x57      #NUMPAD_ADD

    NumpadEnter = 0x58 #NUMPAD_ENTER
    Numpad1 = 0x59     #NUMPAD1
    Numpad2 = 0x5a     #NUMPAD2
    Numpad3 = 0x5b     #NUMPAD3
    Numpad4 = 0x5c     #NUMPAD4
    Numpad5 = 0x5d     #NUMPAD5
    Numpad6 = 0x5e     #NUMPAD6
    Numpad7 = 0x5f     #NUMPAD7

    Numpad8 = 0x60                #NUMPAD8
    Numpad9 = 0x61                #NUMPAD9
    Numpad0 = 0x62                #NUMPAD0
    NumpadDecimal = 0x63          #NUMPAD_DECIMAL
    InternationalBackslash = 0x64 #INTL_BACKSLASH
    ContextMenu = 0x65            #CONTEXT_MENU
    Power = 0x66                  #POWER
    NumpadEqual = 0x67            #NUMPAD_EQUAL

    F13 = 0x68 #F13
    F14 = 0x69 #F14
    F15 = 0x6a #F15
    F16 = 0x6b #F16
    F17 = 0x6c #F17
    F18 = 0x6d #F18
    F19 = 0x6e #F19
    F20 = 0x6f #F20

    F21 = 0x70 #F21
    F22 = 0x71 #F22
    F23 = 0x72 #F23

    F24 = 0x73  #F24
    Open = 0x74 #OPEN

    Help = 0x75 #HELP

    # = 0x77 #SELECT
    Again = 0x79      #AGAIN
    Undo = 0x7a       #UNDO
    Cut = 0x7b        #CUT
    Copy = 0x7c       #COPY
    Paste = 0x7d      #PASTE
    Find = 0x7e       #FIND
    VolumeMute = 0x7f #VOLUME_MUTE

    VolumeUp = 0x80    #VOLUME_UP
    VolumeDown = 0x81  #VOLUME_DOWN
    NumpadComma = 0x85 #NUMPAD_COMMA

    InternationalRO = 0x87  #INTL_RO
    KanaMode = 0x88         #KANA_MODE
    InternationalYen = 0x89 #INTL_YEN
    Convert = 0x8a          #CONVERT
    NonConvert = 0x8b       #NON_CONVERT
    Lang1 = 0x90            #LANG1
    Lang2 = 0x91            #LANG2
    Lang3 = 0x92            #LANG3
    Lang4 = 0x93            #LANG4

    LeftCtrl = 0xe0   #CONTROL_LEFT
    LeftShift = 0xe1  #SHIFT_LEFT
    LeftAlt = 0xe2    #ALT_LEFT
    LeftMeta = 0xe3   #META_LEFT
    RightCtrl = 0xe4  #CONTROL_RIGHT
    RightShift = 0xe5 #SHIFT_RIGHT
    RightAlt = 0xe6   #ALT_RIGHT
    RightMeta = 0xe7  #META_RIGHT