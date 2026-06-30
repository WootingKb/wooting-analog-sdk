use crate::{AnalogValue, KeyCode};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Maximum number of active binds per physical key
pub const MAX_KEY_STATES: usize = 5;

#[derive(Clone, PartialEq, PartialOrd, Debug, Default)]
#[repr(C)]
pub(crate) struct Key {
    pub(crate) code: KeyCode,
    pub(crate) value: AnalogValue,
}

/// A matrix position for a key on the keyboard.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[repr(C)]
pub struct KeyPosition {
    pub x: u8,
    pub y: u8,
}

impl std::fmt::Display for KeyPosition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(x: {}, y: {})", self.x, self.y)
    }
}

/// State for each key bind that is active on a [`PhysicalKey`].
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
#[repr(C)]
pub struct KeyState {
    pub value: f32,
    pub keycode: KeyCode,
    pub actuated: bool,
}

/// A physical key on the keyboard with all active key binds, actuation state and press depth.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct PhysicalKey {
    pub position: KeyPosition,
    pub(crate) active_key_count: u8, // sadly not usize for ffi safety
    pub(crate) state: [KeyState; MAX_KEY_STATES],
}

impl Default for PhysicalKey {
    fn default() -> Self {
        Self {
            position: KeyPosition::default(),
            active_key_count: 0,
            state: [KeyState::default(); MAX_KEY_STATES],
        }
    }
}

impl std::fmt::Debug for PhysicalKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhysicalKey")
            .field("pos", &self.position)
            .field("active_key_count", &self.active_key_count)
            .field("states", &&self.state[..self.active_key_count as usize])
            .finish()
    }
}

impl PhysicalKey {
    pub(crate) fn new(position: KeyPosition) -> Self {
        Self {
            position,
            active_key_count: 0,
            state: [KeyState::default(); MAX_KEY_STATES],
        }
    }

    pub(crate) fn push_state(&mut self, state: KeyState) -> bool {
        if (self.active_key_count as usize) < MAX_KEY_STATES {
            self.state[self.active_key_count as usize] = state;
            self.active_key_count += 1;
            true
        } else {
            false
        }
    }

    pub fn state(&self) -> &[KeyState] {
        &self.state[..self.active_key_count as usize]
    }

    pub fn active_key_count(&self) -> usize {
        self.active_key_count as usize
    }

    pub fn max_value(&self) -> f32 {
        self.state.iter().map(|s| s.value).fold(0.0, f32::max)
    }

    pub fn is_advanced_key(&self) -> bool {
        self.state.iter().any(|s| s.keycode.is_advanced_key())
    }
}
