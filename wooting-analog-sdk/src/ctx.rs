//! Analog data collection with extra context for each key press in various formats.
//!
//! When polling data from the [`AnalogSdk`](crate::AnalogSdk) through
//! [`read_keycodes`](crate::AnalogSdk::read_keycodes) or
//! [`read_positions`](crate::AnalogSdk::read_positions) the data format is based on what filter
//! is applied to `Ctx`.
//! - [`Ctx<KeyCodeFilter>`] will iterate over the plain keycodes and analog values the underlying
//!   device reported when it was polled.
//! - [`Ctx<PositionFilter>`] will have a different format where some post processing was done by
//!   grouping all active keycodes and values together into a [`PhysicalKey`] for each
//!   physical [`KeyPosition`].
//!
//! ```no_run
//! # use wooting_analog_sdk::AnalogSdk;
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # let analog_sdk = AnalogSdk::new().initialise()?;
//! // Polls and operates over the `Ctx<KeyCodeFilter>` format.
//! analog_sdk.read_keycodes(|ctx| {
//!     for (keycode, value) in ctx.iter() {
//!         println!("read keycode: {keycode} with value: {value}");
//!     }
//! })?;
//!
//! // Same as read_keycodes but with the `Ctx<PositionFilter>` format instead.
//! analog_sdk.read_positions(|ctx| {
//!     for physical_key in ctx.iter() {
//!         println!(
//!             "read from position: {} with values: {:?}",
//!             physical_key.position,
//!             physical_key.state(),
//!         );
//!     }
//! })?;
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;

use crate::{AnalogValue, KeyCode, KeyPosition, PhysicalKey};

/// Configure [`Ctx`] to contain poll results based on keycodes and analog values.
#[derive(Debug)]
pub struct KeyCodeFormat<'ctx> {
    data: &'ctx mut HashMap<KeyCode, AnalogValue>,
}

/// Configure [`Ctx`] to contain poll results based on matrix position and physical keys.
#[derive(Debug)]
pub struct PositionFormat<'ctx> {
    data: &'ctx mut HashMap<KeyPosition, PhysicalKey>,
}

/// An analog data collection with a specific set of filters to poll analog data with different
/// formats.
#[derive(Debug)]
pub struct Ctx<F> {
    // Use generic type parameters for formats instead of explicit standalone types to prevent a
    // breaking change when formats might need to share state. These generics give no extra
    // overhead compared to explicit types, but it does give us more flexibility in the future.
    format: F,
}

impl<'ctx> Ctx<KeyCodeFormat<'ctx>> {
    pub(crate) fn with_keycodes(data: &'ctx mut HashMap<KeyCode, AnalogValue>) -> Self {
        Self {
            format: KeyCodeFormat { data },
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&KeyCode, &AnalogValue)> {
        self.format.data.iter()
    }

    pub fn keys(&self) -> impl Iterator<Item = &KeyCode> {
        self.format.data.keys()
    }

    pub fn values(&self) -> impl Iterator<Item = &AnalogValue> {
        self.format.data.values()
    }

    pub fn get(&self, key: &KeyCode) -> Option<&AnalogValue> {
        self.format.data.get(key)
    }

    pub fn contains(&self, key: &KeyCode) -> bool {
        self.format.data.contains_key(key)
    }

    pub fn len(&self) -> usize {
        self.format.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.format.data.is_empty()
    }

    pub fn remove(&mut self, key: &KeyCode) -> Option<AnalogValue> {
        self.format.data.remove(key)
    }
}

impl<'ctx> Ctx<PositionFormat<'ctx>> {
    pub(crate) fn with_positions(data: &'ctx mut HashMap<KeyPosition, PhysicalKey>) -> Self {
        Self {
            format: PositionFormat { data },
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &PhysicalKey> {
        self.format.data.values()
    }

    pub fn get(&self, position: &KeyPosition) -> Option<&PhysicalKey> {
        self.format.data.get(position)
    }

    pub fn contains(&self, position: &KeyPosition) -> bool {
        self.format.data.contains_key(position)
    }

    pub fn len(&self) -> usize {
        self.format.data.len()
    }

    pub fn is_empty(&self) -> bool {
        self.format.data.is_empty()
    }

    pub fn remove(&mut self, key: &KeyPosition) -> Option<PhysicalKey> {
        self.format.data.remove(key)
    }
}

impl<'ctx> IntoIterator for &'ctx Ctx<KeyCodeFormat<'ctx>> {
    type Item = (&'ctx KeyCode, &'ctx AnalogValue);
    type IntoIter = std::collections::hash_map::Iter<'ctx, KeyCode, AnalogValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.format.data.iter()
    }
}

impl<'ctx> IntoIterator for &'ctx Ctx<PositionFormat<'ctx>> {
    type Item = &'ctx PhysicalKey;
    type IntoIter = std::collections::hash_map::Values<'ctx, KeyPosition, PhysicalKey>;

    fn into_iter(self) -> Self::IntoIter {
        self.format.data.values()
    }
}
