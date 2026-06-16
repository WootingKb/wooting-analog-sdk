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

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[repr(C)]
pub struct KeyPosition {
    pub x: u8,
    pub y: u8,
}

#[derive(Copy, Clone, PartialEq, PartialOrd, Debug, Default)]
#[repr(C)]
pub struct KeyState {
    pub(crate) value: f32,
    pub(crate) keycode: KeyCode,
    pub(crate) actuated: bool,
}

#[derive(Copy, Clone, PartialEq, PartialOrd)]
#[repr(C)]
pub struct PhysicalKey {
    pub(crate) pos: KeyPosition,
    pub(crate) active_key_count: u8, // sadly not usize for ffi safety
    pub(crate) states: [KeyState; MAX_KEY_STATES],
}

impl Default for PhysicalKey {
    fn default() -> Self {
        Self {
            pos: KeyPosition::default(),
            active_key_count: 0,
            states: [KeyState::default(); MAX_KEY_STATES],
        }
    }
}

impl std::fmt::Debug for PhysicalKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PhysicalKey")
            .field("pos", &self.pos)
            .field("active_key_count", &self.active_key_count)
            .field("states", &&self.states[..self.active_key_count as usize])
            .finish()
    }
}

impl PhysicalKey {
    pub(crate) fn new(pos: KeyPosition) -> Self {
        Self {
            pos,
            active_key_count: 0,
            states: [KeyState::default(); MAX_KEY_STATES],
        }
    }

    pub(crate) fn push_state(&mut self, state: KeyState) -> bool {
        if (self.active_key_count as usize) < MAX_KEY_STATES {
            self.states[self.active_key_count as usize] = state;
            self.active_key_count += 1;
            true
        } else {
            false
        }
    }

    pub fn states(&self) -> &[KeyState] {
        &self.states[..self.active_key_count as usize]
    }

    pub fn max_value(&self) -> f32 {
        self.states().iter().map(|s| s.value).fold(0.0, f32::max)
    }

    pub fn is_advanced_key(&self) -> bool {
        self.states.iter().any(|s| s.keycode.is_advanced_key())
    }
}
