//! Inspect analog values, associated metadata and helper functions.

use std::{cmp::Ordering, fmt, ops};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::key::KeyPosition;

/// Any additional information an [`AnalogValue`] can contain.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
#[repr(C, u8)]
pub enum ValueMetadata {
    /// The analog protocol does not support metadata.
    #[default]
    None,
    Basic {
        position: KeyPosition,
        actuated: bool,
    },

    // Reserve 8 bytes to ensure we can avoid shifting the memory layout of the union a litte while
    // longer. As soon as this type does shift a new enum should be created and used instead, while
    // also keeping this one around for backwards compatibility.
    #[cfg_attr(feature = "serde", serde(skip))]
    #[doc(hidden)]
    _Reserved([u8; 8]) = 255,
}

/// An analog float value with optional metadata.
///
/// The metadata is only present if the device supplying the data has support for it in the
/// analog protocol. Some plugins or devices might run older firmware or simply don't have the
/// extra data associated with a key press yielding no valuable extra data, other than the
/// press depth.
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct AnalogValue {
    pub(crate) inner: f32,
    pub(crate) metadata: ValueMetadata,
}

impl AnalogValue {
    pub(crate) fn with_metadata(mut self, meta: ValueMetadata) -> Self {
        self.metadata = meta;
        self
    }

    pub fn metadata(&self) -> &ValueMetadata {
        &self.metadata
    }
}

impl Eq for AnalogValue {}

impl PartialEq for AnalogValue {
    fn eq(&self, other: &Self) -> bool {
        self.inner.total_cmp(&other.inner) == Ordering::Equal
    }
}

impl PartialOrd for AnalogValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AnalogValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.total_cmp(&other.inner)
    }
}

impl PartialEq<f32> for AnalogValue {
    fn eq(&self, other: &f32) -> bool {
        self.inner.total_cmp(other) == Ordering::Equal
    }
}

impl PartialOrd<f32> for AnalogValue {
    fn partial_cmp(&self, other: &f32) -> Option<Ordering> {
        self.inner.partial_cmp(other)
    }
}

impl From<f32> for AnalogValue {
    fn from(value: f32) -> Self {
        Self {
            inner: value,
            metadata: ValueMetadata::None,
        }
    }
}

impl From<AnalogValue> for f32 {
    fn from(value: AnalogValue) -> Self {
        value.inner
    }
}

impl From<AnalogValue> for f64 {
    fn from(value: AnalogValue) -> Self {
        f64::from(value.inner)
    }
}

impl From<&AnalogValue> for f32 {
    fn from(value: &AnalogValue) -> Self {
        value.inner
    }
}

impl From<&AnalogValue> for f64 {
    fn from(value: &AnalogValue) -> Self {
        f64::from(value.inner)
    }
}

impl fmt::Display for AnalogValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

macro_rules! operator {
    ($(($trait:ident, $method:ident)),* $(,)?) => {
        $(
            impl ops::$trait for AnalogValue {
                type Output = Self;

                fn $method(self, rhs: Self) -> Self::Output {
                    Self::from(self.inner.$method(rhs.inner))
                }
            }

            impl ops::$trait<f32> for AnalogValue {
                type Output = Self;

                fn $method(self, rhs: f32) -> Self::Output {
                    Self::from(self.inner.$method(rhs))
                }
            }
        )*
    }
}

operator![(Add, add), (Sub, sub), (Mul, mul), (Div, div), (Rem, rem)];

macro_rules! operator_inplace {
    ($(($trait:ident, $method:ident)),* $(,)?) => {
        $(
            impl ops::$trait for AnalogValue {
                fn $method(&mut self, rhs: Self) {
                    self.inner.$method(rhs.inner);
                }
            }

            impl ops::$trait<f32> for AnalogValue {
                fn $method(&mut self, rhs: f32) {
                    self.inner.$method(rhs);
                }
            }
        )*
    }
}

operator_inplace![
    (AddAssign, add_assign),
    (SubAssign, sub_assign),
    (MulAssign, mul_assign),
    (DivAssign, div_assign),
    (RemAssign, rem_assign),
];

impl ops::Neg for AnalogValue {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::from(-self.inner)
    }
}
